# Hardware Frontier
## VectorVolt — Next-Decade Hardware Acceleration Targets

**Status:** Architecture Capture — Research/Future (not on any current phase roadmap)
**Captured:** 2026-07-09

This document captures hardware technologies that are either emerging (CXL), expensive/niche
(FPGA), or not yet production-ready at commodity price points. None of these are required for
VectorVolt Phase 1–5. They represent the architectural direction for Phase 6+ and beyond.

---

## 1. CXL-Based Distributed Commit

### The Problem It Solves

Distributed OLTP commits today require 2-3 Raft/Paxos network round-trips:

```
Node A → "Prepare" → Nodes B, C   (network RTT 1)
Nodes B, C → "OK" → Node A        (network RTT 2)
Node A → "Commit" → Nodes B, C    (network RTT 3)

Total: 2–5 milliseconds per distributed commit
```

### The CXL Approach

**CXL (Compute Express Link) 3.0+** enables fabric-attached memory: multiple nodes physically
share a pool of persistent memory over PCIe. A distributed commit becomes a single hardware
operation:

```
All 3 replica nodes share a 64-byte cache line in CXL memory

Node A: atomic CAS(old_log_index, new_log_index) on the shared cache line

CXL fabric MESI protocol broadcasts the cache line update to Node B and Node C
automatically — no software networking stack involved

Node B and Node C's CPUs see the updated cache line in ~200 nanoseconds

Total distributed commit: ~1 microsecond (1000x faster than Raft)
```

### Why This Isn't Built Yet

- CXL 3.0 hardware with multi-host memory sharing is only now arriving (2025–2026)
- Every database engine alive was designed around "network packets" as the fundamental
  distributed primitive — rewriting the transaction manager for CAS-as-consensus requires
  abandoning all existing consensus code
- Nobody has done this yet for a production OLTP engine

### VectorVolt Readiness Requirement

For VectorVolt to exploit CXL commits when the hardware matures:
- The distributed transaction manager must be factored as a pluggable consensus backend
- A `ConsensusBackend` trait with two implementations: `RaftConsensus` (today) and
  `CxlCasConsensus` (Phase 6+ when hardware is available)
- Internal design should never assume "a commit requires a network packet"

---

## 2. FPGA Semantic Constraints

### The Problem It Solves

Every `INSERT` or `UPDATE` checks constraints (NOT NULL, CHECK, UNIQUE, FOREIGN KEY) by
executing CPU instructions. For high-throughput systems, constraint evaluation is measurable
overhead.

### The FPGA Approach

An FPGA sitting between the NVMe controller and DRAM validates rows **at memory-bus speed**:

```
VectorVolt → write page to storage
             ↓
             FPGA intercepts the page write
             ↓
             FPGA validates each row against hardcoded business rules
             (balance >= 0, order_total == sum(line_items))
             ↓
             If constraint fails: FPGA marks page invalid before CPU sees it
             ↓
             CPU never wastes cycles checking constraints that hardware already verified
```

Validation happens at memory-bus speed (sub-nanosecond) rather than CPU instruction speed.

### Why This Isn't the Right Investment Now

- A high-end FPGA (Xilinx VU13P) costs $10,000+ per chip vs $2,000 for a server CPU
- Development requires hardware engineers, specialized tooling ($10k–$50k/year licenses)
- For 99% of OLTP constraint workloads, software constraint evaluation in VectorVolt is
  fast enough — the bottleneck is elsewhere (network, lock contention, memory bandwidth)
- Intel QAT and DSA (already documented) solve real bottlenecks at zero extra cost

### When FPGA Becomes Relevant

For VectorVolt deployments at:
- Financial trading infrastructure where every nanosecond matters
- Real-time risk systems with millions of constraint-heavy writes per second
- Customers who have HFT-style hardware requirements and budgets to match

At that scale, FPGA constraint offload becomes cost-justified. Until then, software constraints
with SIMD vectorization (see READ_PATH_ARCHITECTURE.md) are the correct investment.

### VectorVolt Readiness Requirement

Keep constraint evaluation behind a `ConstraintBackend` abstraction:
- `SoftwareConstraints` (default, SIMD-accelerated)
- `FpgaConstraints` (future, for specialized deployments)
- The hot path never hard-codes the implementation

---

## 3. Intel QAT and DSA (Available Today)

These are NOT frontier — they are available now and should be integrated in Phase 3.

### Intel QAT (QuickAssist Technology)

Built into Intel Xeon Scalable processors. Offloads to dedicated hardware at zero extra cost:
- AES-256-GCM encryption (used for source KOs and backup payloads)
- DEFLATE compression (WAL segments, backup packages)
- SHA-256 HMAC (session tokens, integrity checks)

**Integration target:** Phase 3. Route all VectorVolt encryption and compression through QAT
when available, with CPU fallback when not.

```rust
pub trait EncryptionBackend {
    fn encrypt(&self, plaintext: &[u8], key: &[u8]) -> Vec<u8>;
    fn decrypt(&self, ciphertext: &[u8], key: &[u8]) -> Vec<u8>;
}

// Auto-detected at startup:
pub fn detect_backend() -> Box<dyn EncryptionBackend> {
    if qat::is_available() {
        Box::new(QatBackend::new())
    } else {
        Box::new(AesGcmSoftwareBackend::new())
    }
}
```

### Intel DSA (Data Streaming Accelerator)

Also built into Intel Xeon Scalable processors. Offloads:
- Large memory copies (buffer pool page movement)
- CRC-32C checksum computation (cross-partition checksum verification)
- Data movement between NVMe and DRAM buffer pool

**Integration target:** Phase 4. Use DSA for bulk buffer pool fills during Shadowbox
initialization and backup snapshot operations — frees OLTP cores for transaction processing.

---

## 4. RDMA (Remote Direct Memory Access)

Available today on servers with Mellanox/NVIDIA ConnectX NICs or InfiniBand.

### What It Enables

RDMA allows one server to read/write another server's RAM without involving the remote CPU:

```
Without RDMA:    Node A → TCP/IP stack → Node B's CPU → Node B's RAM
                 Latency: ~100 µs

With RDMA:       Node A → NIC → Node B's NIC → Node B's RAM (CPU not involved)
                 Latency: ~1–5 µs
```

### VectorVolt Use Case

WAL streaming to the hot-standby replica (Layer 1 of BACKUP_STRATEGY.md) is the primary
beneficiary. With RDMA, WAL segment delivery to standby drops from ~100 µs to ~5 µs —
RPO (recovery point objective) improves by 20x.

**Integration target:** Phase 5 (cluster mode). Implement WAL shipping with RDMA transport
when available, TCP fallback otherwise.

---

## 5. ARM Architecture (Available Today on Cloud)

Not a frontier — ARM is mature and should be the default deployment target.

AWS Graviton5, Google Axion (ARM-based): **35–65% better price-performance** for database
workloads vs x86 at the same cost. The Rust OLTP path compiles cleanly for
`aarch64-unknown-linux-musl` (already in the toolchain matrix in BUILD_SANDBOX_ADMIN.md).

**The simple rule:** Deploy VectorVolt on ARM unless there is a specific reason to use x86.
The pre-fetch engine, SIMD paths, and crypto operations all have ARM NEON equivalents that
Rust's standard libraries emit automatically.

---

## Phase Readiness Summary

| Technology | Cost | VectorVolt Benefit | When to Invest |
|---|---|---|---|
| ARM instances | $0 (already cheaper) | 35-65% better price-perf | Now |
| Intel QAT | $0 (built-in) | Free crypto/compression offload | Phase 3 |
| Intel DSA | $0 (built-in) | Free memory copy acceleration | Phase 4 |
| RDMA (SmartNIC) | $200–$1,500/NIC | 20x WAL shipping improvement | Phase 5 |
| CXL 3.0 | Hardware cost TBD | 1000x distributed commit latency | Phase 6+ |
| FPGA constraints | $10,000+ | Sub-ns constraint evaluation | Specialized deployments only |

---

*Hardware accelerates the vision. It does not define it.*
*The algorithms and architecture are the competitive moat. Hardware multiplies them.*
