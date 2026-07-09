# VectorVolt — Distributed Replication Architecture
## SWIM Gossip, Virtual Nodes, Raft Leader Election, Failover

**Status:** Architecture reference — locked decisions implemented in Phase 3+  
**Related:** `VECTORVOLT_LOCKED_DECISIONS.md` (LD-SWIM, LD-VNODE, LD-EPOCH, LD-FAILOVER, LD-QUORUM)  
**Captured:** 2026-07-09

---

## Overview: Two-Layer Cluster Coordination

VectorVolt uses two distinct protocols for two distinct concerns:

```
┌─────────────────────────────────────────────────────────────┐
│ Layer 1: Cluster Membership (SWIM — gossip-based)           │
│   Which nodes exist? Which are alive? Which just died?      │
│   Scales to 800+ nodes. Eventually consistent. O(log n).    │
├─────────────────────────────────────────────────────────────┤
│ Layer 2: Shard Leadership (Raft — consensus-based)          │
│   Who is the shard leader? Is this WAL entry committed?     │
│   Per-shard group. Strongly consistent. Quorum write.       │
└─────────────────────────────────────────────────────────────┘
```

The confusion between "which protocol" is resolved by the concern:
- **Who is alive (cluster-wide):** SWIM
- **Who leads a shard (per-shard group):** Raft

---

## Part 1: SWIM Membership Protocol

### Why SWIM, Not Raft for Membership

Raft is a strongly consistent protocol that requires a quorum acknowledgment for every
state change. In a 500-node cluster, electing a new membership leader every time a node
joins or leaves creates a bottleneck. SWIM provides the same information at O(log n) cost.

### SWIM Mechanics

```
Detection:    Each node periodically sends PING to a random subset of peers
              If no ACK within timeout → node is "suspicious"
              Ping-req: ask k other nodes to probe the suspicious node
              If still no ACK → node is declared DEAD

Propagation:  State changes (JOIN/LEAVE/DEAD) are piggybacked on existing
              PING/ACK messages (infection-style propagation)
              Reaches all nodes in O(log n) message rounds

Convergence:  All nodes converge to the same membership view
              Trade-off: briefly inconsistent (acceptable for membership)
```

### SWIM Configuration (VectorVolt defaults)

| Parameter | Default | Description |
|---|---|---|
| `ping_interval_ms` | 200ms | How often each node probes a random peer |
| `ping_timeout_ms` | 50ms | Timeout before marking peer as suspicious |
| `suspicion_multiplier` | 4 | DEAD = suspicion × ping_interval (800ms default) |
| `indirect_checks` | 3 | Number of nodes asked to verify suspicious peer |
| `gossip_fanout` | 3 | Number of peers to include in each gossip round |

### Membership State Machine

```
UNKNOWN → ALIVE        when JOIN message received
ALIVE   → SUSPECT      when direct PING times out
SUSPECT → DEAD         when suspicion timeout expires with no ACK
ALIVE   → LEFT         when node sends graceful LEAVE
DEAD    → ALIVE        when dead node sends REJOIN (e.g., false positive)
```

### Integration with Raft

When SWIM marks a Raft leader as DEAD:
1. SWIM membership update propagates to all cluster nodes
2. Each shard group whose leader just died detects leadership vacancy
3. Raft election is triggered within that shard group
4. New Raft leader elected; epoch incremented (LD-EPOCH fencing)
5. ShardCatalog updated with new shard leadership topology

---

## Part 2: Virtual Node Hash Ring (LD-VNODE)

### Consistent Hashing with 4096 VNodes

```
Shard Key Hash Space: [0, 2^32)

Virtual nodes: 4096 tokens distributed across the ring
Physical shards: 1 to 65,536

Example: 8 physical shards, 512 VNodes each

VNode 0–511:     → Physical Shard 0
VNode 512–1023:  → Physical Shard 1
VNode 1024–1535: → Physical Shard 2
...
VNode 3584–4095: → Physical Shard 7
```

### Routing a Command to a Shard

```rust
pub fn route_command(shard_key: &[u8]) -> ShardId {
    let hash = xxhash64(shard_key);          // deterministic, non-cryptographic
    let vnode = (hash % 4096) as u16;        // map to virtual node space
    SHARD_CATALOG.vnode_to_shard(vnode)      // ShardCatalog lookup (L1 cache hit)
}
```

### Shard Addition: Minimal Data Movement

When adding Physical Shard 8 to the 8-shard cluster above:
```
Before: 8 shards × 512 VNodes = 4096 total
After:  9 shards, VNodes rebalanced

New distribution:
  - Shard 8 receives ~512/9 ≈ 455 VNodes
  - Each existing shard donates ~57 VNodes (1/9 of their allocation)
  - Only ~11% of data moves total (not 50%, not 100%)
```

This is why VNodes matter: without virtual nodes, adding shard 8 moves 50%
of data from one shard. With 4096 VNodes, movement is ~1/N_shards.

### ShardCatalog

The ShardCatalog is the authoritative routing table:

```rust
pub struct ShardCatalog {
    pub vnode_map: [ShardId; 4096],          // maps VNode → PhysicalShard
    pub shard_topology: HashMap<ShardId, ShardGroup>,  // shard → replica set
    pub epoch: u64,                           // incremented on every topology change
}

pub struct ShardGroup {
    pub leader: NodeId,
    pub followers: Vec<NodeId>,
    pub epoch: u64,
}
```

The ShardCatalog lives in the Control Plane (vectorvolt-control binary). Every
vectorvolt-server and vectorvolt-reader node caches a local copy and refreshes it
when they receive a topology change notification.

---

## Part 3: Raft Consensus (Per Shard Group)

### What Raft Does in VectorVolt

Raft is used **only** for:
1. WAL log replication (every WAL entry must be acknowledged by majority of replicas)
2. Leader election within a shard group
3. Epoch fencing (preventing split-brain writes)

Raft does NOT handle cluster membership — that's SWIM's job.

### Raft Crate: `raft-rs`

VectorVolt uses the `raft-rs` crate (TiKV's Raft implementation). The storage
backend for Raft is implemented against the `raft::Storage` trait backed by the
VectorVolt WAL:

```rust
impl raft::Storage for WalBackedRaftStorage {
    fn initial_state(&self) -> raft::Result<RaftState> { ... }
    fn entries(&self, lo: u64, hi: u64, ...) -> raft::Result<Vec<Entry>> { ... }
    fn term(&self, idx: u64) -> raft::Result<u64> { ... }
    fn first_index(&self) -> raft::Result<u64> { ... }
    fn last_index(&self) -> raft::Result<u64> { ... }
    fn snapshot(&self, request_index: u64) -> raft::Result<Snapshot> { ... }
}
```

### Shard Group Topology

Minimum viable shard group: **3 replicas** (tolerates 1 failure).
Production recommendation: **5 replicas** (tolerates 2 failures).

```
Shard Group 0 (3 replicas):
  Node A → Leader   (receives writes, fans out to followers)
  Node B → Follower (WAL replicated, can serve stale reads)
  Node C → Follower (WAL replicated, can serve stale reads)

Write commit requires: ACK from 2 of 3 nodes (majority)
```

### WAL Replication Protocol

```
1. Client sends CommandFrame to shard leader (gRPC, port 50051)
2. Leader assigns LSN (Log Sequence Number), appends to WAL
3. Leader sends AppendEntries RPC to all followers in parallel
4. Followers append to their WAL, ACK the leader
5. When majority ACK received: entry is committed (LD-QUORUM)
6. Leader executes command against arena state
7. Leader responds to client with CommandResult
8. Followers execute entry from their WAL on their next tick
```

### Binary WAL Replication Format

WAL segments are replicated in a binary delta-compressed format:

```
WAL Segment Header (32 bytes):
  [shard_id: u32][epoch: u64][segment_id: u64][first_lsn: u64][crc32: u32]

WAL Entry (variable, minimum 24 bytes):
  [lsn: u64][timestamp_ns: u64][payload_len: u32][payload: bytes][crc32: u32]

Delta Compression (applied to segments before network transmission):
  - Identical timestamps: delta-encoded (saves 8 bytes per typical entry)
  - LZ4 block compression applied to payload region
  - Typical compression ratio: 3-5x for structured command payloads
```

---

## Part 4: Epoch Fencing (Split-Brain Prevention)

### The Split-Brain Scenario

```
5-node cluster: A (leader), B, C, D, E

Network partition: {A, B} isolated from {C, D, E}

Without epoch fencing:
  - A continues as leader in partition {A, B}
  - C, D, E elect a new leader (say C)
  - Both A and C accept writes
  - When partition heals: conflicting write history

With epoch fencing:
  - When C is elected, epoch increments from 5 → 6
  - A is still at epoch 5
  - Any follower receiving a write from A at epoch 5 rejects it
  - A's writes after the partition are silently dropped
```

### Epoch Fencing Implementation

```rust
pub struct EpochGuard {
    pub epoch: AtomicU64,
}

impl EpochGuard {
    pub fn check(&self, claimed_epoch: u64) -> Result<(), FencingError> {
        let current = self.epoch.load(Ordering::Acquire);
        if claimed_epoch < current {
            Err(FencingError::StaleEpoch { claimed: claimed_epoch, current })
        } else {
            Ok(())
        }
    }
}
```

Every WAL AppendEntries RPC includes the sender's epoch. Receivers check before
appending. Stale-epoch entries are silently rejected (not an error — the sender
just doesn't know it has been fenced).

---

## Part 5: Failover Sequence (< 2 Second Target)

### Timing Budget

```
T+0ms:      Leader heartbeat missed by followers
T+500ms:    Heartbeat timeout expires (configurable, 500ms default)
T+500ms:    Followers enter election mode simultaneously
T+600ms:    First election completes (typically 1-2 RTTs at 10-50ms RTT)
T+650ms:    New epoch announced via SWIM gossip (O(log n) propagation)
T+800ms:    ShardCatalog updated with new leader topology
T+900ms:    Clients receive new topology from ShardCatalog refresh
T+1000ms:   Clients begin routing writes to new leader

Total failover time: ~1 second typical, < 2 seconds p99.9
```

### Client-Side Failover Behavior

All VectorVolt SDKs implement automatic failover:

```
1. Client sends CommandFrame to leader A
2. Leader A is dead — gRPC call returns UNAVAILABLE
3. SDK: exponential backoff starting at 10ms
4. SDK: fetches fresh ShardCatalog from Control Plane
5. SDK: retries CommandFrame to new leader B
6. Command succeeds

Client timeout during failover: configurable, default 5 seconds
Retry budget: 3 retries with exponential backoff + jitter
```

---

## Part 6: Read Replica Routing

### Stale Read (Default)

```
Client → vectorvolt-reader → any healthy replica in shard group
                           → serves reads from that replica's snapshot
                           → may be behind leader by up to 50ms (configurable)
```

### Consistent Read (Opt-In)

```
Client → vectorvolt-reader → routes to shard leader
                           → leader serves read from its latest committed state
                           → guaranteed to see all writes as of query time
```

### Read Fabric Load Balancing

The Read Fabric (`vectorvolt-reader`) applies the following replica selection:
1. Filter: only healthy replicas (SWIM status = ALIVE)
2. Filter: only replicas where replica_lag < configured stale_read_tolerance
3. Select: lowest latency replica (measured via recent ping RTT)
4. Fallback: if no replica meets lag threshold, route to leader

---

## Part 7: PostgreSQL Wire Protocol Compatibility

VectorVolt's Read Fabric speaks the PostgreSQL wire protocol on the SQL port:

```
Port 5432 (configurable): libpq-compatible PostgreSQL wire protocol
  → SELECT queries (read fabric, stale by default)
  → SHOW commands (cluster topology, shard info)
  → Limited DDL (translated to VectorVolt schema operations)

Port 50051 (configurable): gRPC (native performance interface)
  → CommandFrame submission (writes)
  → Direct WAL access (replication, backup)
  → Admin operations (shard management, topology changes)
```

### PostgreSQL Wire Protocol Implementation

Implemented using the `pgwire` Rust crate. The query flow:
1. Receive SQL query bytes (libpq framing)
2. Parse SQL (using `sqlparser-rs`)
3. Translate parsed AST to VectorVolt internal query plan
4. Execute via Read Fabric (SIMD scan, index lookup, etc.)
5. Serialize result set as PostgreSQL row format
6. Return via libpq wire protocol

**Unsupported PostgreSQL features** (return `not supported` error):
- `BEGIN / COMMIT / ROLLBACK` (use gRPC command API for transactions)
- `INSERT / UPDATE / DELETE` via SQL (use command API)
- Stored procedures, functions, triggers
- PostgreSQL-specific system tables (`pg_stat_user_tables`, etc.)

---

## Cluster Topology Reference

### Single-Node Deployment (Development)

```
Machine: all three binaries on one node

vectorvolt-server :50051    (write engine)
vectorvolt-reader :5432     (read fabric, SQL)
vectorvolt-control :8443    (control plane, admin)

Replication factor: 1 (no replicas — development only)
SWIM: localhost-only gossip
```

### 3-Node Production Minimum

```
Node 0:  vectorvolt-server (Shard 0 Leader) + vectorvolt-reader
Node 1:  vectorvolt-server (Shard 0 Follower) + vectorvolt-reader
Node 2:  vectorvolt-server (Shard 0 Follower) + vectorvolt-control + vectorvolt-reader

Tolerated failures: 1 node
Availability: 99.9% with single-region deployment
```

### 6-Node Two-Shard Cluster

```
Shard 0 group:  Nodes 0, 1, 2 (Node 0 = leader)
Shard 1 group:  Nodes 3, 4, 5 (Node 3 = leader)
Control Plane:  Node 5 (also Shard 1 follower — control plane is lightweight)

VNode distribution: 2048 VNodes → Shard 0, 2048 VNodes → Shard 1
Throughput: 1M+ commands/sec (500K per shard × 2 shards)
```

---

*Source decisions: LD-SWIM, LD-VNODE, LD-EPOCH, LD-FAILOVER, LD-QUORUM, LD-READ-01, LD-RUNTIME-01, LD-PG-WIRE*  
*From: `C:\vega\vega_requirements.md`, `C:\vega\vega_lld_standards.md`, `C:\ultra-fast-oltp\docs\state-continuation.md`*
