# Layer 7 — Network  ·  Layer 8 — Cluster  ·  Layer 9 — Control Plane  ·  Layer 10 — SDKs
Task-level traceability. Status: ✅ done · 🟡 partial · ⬜ pending · 🧊 env-gated · 🔬 research

**Generated:** 2026-07-30
**Scope:** VV-NET-001..015, VV-CLU-001..022, VV-CTL-001..015, VV-SDK-001..013 — all four crates
(`vectorvolt-net`/`vectorvolt-server`, `vectorvolt-cluster`, `vectorvolt-control`,
`vectorvolt-sdk-*`) are greenfield. Verified absent: `C:\VectorVolt\vectorvolt\crates\` contains
only `vectorvolt-engine`, `vectorvolt-hal`, `vectorvolt-index`, `vectorvolt-query`,
`vectorvolt-storage`, `vectorvolt-store`, `vectorvolt-txn`, `vectorvolt-wal`, `vvctl`. No
net/server/cluster/control/sdk crate exists. `vvctl` (`C:\VectorVolt\vectorvolt\crates\vvctl\src\main.rs`)
currently exposes `table`, `bench`, `hal`, `store`, `wal`, `version` — every command below that
extends the CLI (`vvctl status`, `vvctl failover`, `vvctl drain`, etc.) is a net-new subcommand.

**Sources:** `VECTORVOLT_FEATURE_MATRIX.md` (feature IDs + sprint assignment),
`vision/VECTORVOLT_DISTRIBUTED_REPLICATION.md` (SWIM/vnode/Raft/epoch/failover/read-routing/PG-wire
mechanics — read in full), `vision/VECTORVOLT_LOCKED_DECISIONS.md` (LD-SWIM, LD-VNODE, LD-EPOCH,
LD-FAILOVER, LD-QUORUM, LD-READ-01/02, LD-RUNTIME-01, LD-SDK-01, LD-PG-WIRE, LD-MT-01..08, LD-BACK),
`vision/VECTORVOLT_CORE_ARCHITECTURE.md` §6 (network tiers), §8 (cluster/catalog/multi-tenancy),
§9 (vvctl), §11.4 (admin panel), §12 (SDKs).

---

## Rollup

| Layer | Features | Tasks total | ✅ | ⬜ | 🧊 | 🔬 |
|---|---|---|---|---|---|---|
| L7 — Network (VV-NET) | 15 | 85 | 0 | 11 | 4 | 0 |
| L8 — Cluster (VV-CLU) | 22 | 133 | 0 | 21 | 0 | 1 |
| L9 — Control Plane (VV-CTL) | 15 | 75 | 0 | 15 | 0 | 0 |
| L10 — SDKs (VV-SDK) | 13 | 60 | 0 | 13 | 0 | 0 |
| **Total** | **65** | **353** | **0** | **60** | **4** | **1** |

Feature-level status columns mirror `VECTORVOLT_FEATURE_MATRIX.md` exactly (✅/⬜/🧊/🔬 counts are
per-*feature*, not per-task — a 🧊 feature like VV-NET-011 still decomposes into some ⬜ tasks that
are buildable on Windows today, see below). "Tasks total" is the sum of `.N` rows across every
feature in the layer.

---

## Test environment requirements

The owner has **two AWS EC2 Linux instances** available. This table states precisely what those two
boxes unblock, what still needs more, and where a deterministic single-process simulation is an
honest substitute vs. where it is not.

| Feature | What's actually being tested | Minimum honest environment | Do the 2 EC2 Linux boxes unblock it? |
|---|---|---|---|
| VV-NET-001..010, 013 | Server binary, wire protocol, TCP, gRPC, TLS, connection pool, transport trait | Any OS (Windows dev box is fine — plain sockets, no kernel feature) | N/A — already unblocked today |
| VV-NET-010 (`SO_BUSY_POLL` variant) | Linux-specific busy-poll sockopt | 1 Linux host | Yes, either box |
| VV-NET-011 eBPF/AF_XDP | Real XDP program load + AF_XDP socket | 1 Linux host, kernel ≥ 5.x. Generic-mode (SKB) XDP works on any NIC incl. AWS ENA; **native-mode** XDP needs a driver with XDP support, which ENA does not have | Partial — generic-mode AF_XDP yes on 1 box; native-mode / full 5–20µs target: no, ENA lacks it |
| VV-NET-012 DPDK | Poll-mode driver bypassing the kernel entirely | 1 Linux host **and** a DPDK-supported NIC. On EC2 this means an ENA-compatible instance type with SR-IOV/Enhanced Networking enabled (e.g. `c5n`, `m5n`, `r5n` families) plus the `dpdk-devbind` / `vfio-pci` setup | Only if the two EC2 instances are already one of those families with ENA + SR-IOV enabled; standard `t3`/`m5` without enhanced networking cannot run DPDK's ENA PMD. Confirm instance type before scheduling this work |
| VV-NET-014 io_uring | Linux syscall interface | 1 Linux host, kernel ≥ 5.1 (5.6+ recommended) | Yes, either box (check kernel version) |
| VV-NET-015 RDMA | RDMA NIC send/recv, zero-CPU receive path | RDMA-capable NIC (Mellanox/ConnectX, InfiniBand) or AWS EFA. Standard EC2 ENA NICs are **not** RDMA-capable | No — needs a different instance family (e.g. `p4d`, `hpc6a`, or any EFA-enabled type) not just "a Linux box" |
| VV-CLU-001/002/003 SWIM | Gossip convergence, failure detection timing | Logic: 1-process deterministic sim (mock clock, in-memory transport) is sufficient and preferred for CI. Realistic detection latency (the 800ms suspicion window) needs real network RTT | Deterministic-sim tasks: no host needed. Realistic-latency tasks: yes, the 2 EC2 boxes give a real 2-node timing sample (not the full 800-node scale claim, but validates the state machine against real jitter) |
| VV-CLU-004/005/006/007 hash ring, catalog | Pure data-structure logic | Windows dev box, no network needed | N/A |
| VV-CLU-008 master-slave replication | WAL shipping over a real network link, replication lag under load, follower catch-up, leader-death data-loss check | **2 Linux hosts minimum** (this is exactly the "2-node minimum" the doc's Part 3 describes) | **Yes — this is precisely what the two EC2 boxes are for.** Logic (frame encode/quorum tracking) still gets a deterministic single-process sim first; the 2 hosts validate real network behavior (lag, partial writes, process-kill data-loss) |
| VV-CLU-009 Raft consensus | Leader election, log replication, quorum commit under partition | Minimum viable shard group is **3 replicas**. With only 2 physical hosts you can run 2 replica processes on one host + 1 on the other to exercise quorum math, but that is not a true 3-independent-host test (a single host failure would take out 2 replicas at once, which the algorithm can't distinguish from bad luck) | Partial — enough for functional smoke-testing of raft-rs wiring and quorum arithmetic; genuine 3-node fault-independence and network-partition realism needs a 3rd host |
| VV-CLU-010/011 raft::Storage, quorum writes | WAL-backed storage trait impl, majority commit tracking | Deterministic sim (in-process 3+ RaftNodes, mock transport/clock) is the primary and sufficient test vehicle — this is standard practice for Raft implementations (etcd/TiKV test this way) | Sim-only is acceptable here; real hosts are a nice-to-have follow-up, not a blocker |
| VV-CLU-012 epoch fencing / split-brain | The doc's worked example is a **5-node** cluster split {A,B} vs {C,D,E} | Full-fidelity test needs ≥ 5 hosts (or 5 processes across fewer hosts, accepting the same fault-independence caveat as Raft above). A reduced 2-vs-1 or minimal 3-node odd-quorum split can be exercised on the 2 EC2 boxes + 1 extra process | Deterministic sim is mandatory here regardless (network partition is not reproducible by killing a process — it needs an actual firewall rule / `iptables DROP` between the 2 hosts, which the 2 EC2 boxes CAN do for a 2-host partition scenario). Full 5-node fidelity: no |
| VV-CLU-013 <2s failover | End-to-end timing budget from the doc's T+0..T+1000ms table | Real RTT between independent hosts matters — loopback numbers are meaningless here | Yes for a 2-node signal; note the target assumes realistic cluster RTT (10-50ms), so EC2 same-region/AZ hosts are the right shape of test even if the full topology is smaller than production |
| VV-CLU-015 rebalancing | Data migration under live write load between shards | 2 Linux hosts is enough to move data shard-to-shard across a real network link | Yes |
| VV-CLU-019 vectorvolt-reader binary | CQRS at the process level — a separate reader process | No kernel feature required; 2 processes on 1 box (or split across the 2 EC2 boxes) both work | Yes, either way |
| VV-CLU-021 shard recovery <60s/100GB | Real disk I/O at scale (mmap of ~100GB of micro-partitions) | 1 Linux host with enough local NVMe/EBS to hold the fixture. This is a storage-hardware-dependent number, not just a code question — EC2 gp3 vs instance-store NVMe will produce very different results | Yes for functional validation; be explicit in the report about which storage class the EC2 box actually has before quoting a number as representative |
| VV-CTL-001..015 (control plane, admin API, auth, UI) | Ordinary async Rust + HTTP + a build step | Windows dev box is fine throughout | N/A |
| VV-SDK-001..013 (protobuf, all language SDKs, packaging) | Codegen + client logic + package-manager publishing | Windows dev box is fine for Rust/C#/TS. Python/Go/Java toolchains are cross-platform too — no Linux requirement anywhere in this layer | N/A |

**Bottom line:** the two EC2 boxes are the right and sufficient environment for VV-CLU-008
(master-slave replication — exactly a 2-node scenario), VV-NET-014 (io_uring), generic-mode
VV-NET-011 (AF_XDP), VV-CLU-015 (rebalancing), VV-CLU-019 (reader binary), and give a partial/reduced
signal for VV-CLU-009/012/013 (which are specified against 3- and 5-node topologies). They do
**not** unblock VV-NET-012 (DPDK — needs a specific ENA+SR-IOV instance family, confirm before
assuming) or VV-NET-015 (RDMA — needs EFA/InfiniBand, a different instance family entirely).

---

# LAYER 7 — Network (`vectorvolt-net`, `vectorvolt-server`)

Nothing in the workspace opens a socket today. `tokio` exists only as WAL group-commit's task
scheduler. All 15 features are greenfield.

### VV-NET-001 — `vectorvolt-server` daemon binary  ⬜
**Spec:** CORE_ARCHITECTURE.md §6.5, §15 crate map
**Blocked by:** none (can start immediately)
**Needs:** none — Windows dev box

| Task | Status | Detail |
|---|---|---|
| VV-NET-001.1 | ⬜ | Scaffold `vectorvolt-server` crate: `Cargo.toml`, `src/main.rs`, `src/lib.rs`, clap CLI (`--config`, `--bind`, `--grpc-port`) |
| VV-NET-001.2 | ⬜ | Server config struct + loader (bind addrs, shard assignment, log level) — placeholder until VV-ENG-017 config system lands, then swap in |
| VV-NET-001.3 | ⬜ | Startup sequence: open WAL/engine via `vectorvolt-engine`, bind listener sockets, log ready state |
| VV-NET-001.4 | ⬜ | Signal handling: SIGTERM/SIGINT (Unix) and `Ctrl+C` handler (Windows) trigger graceful shutdown |
| VV-NET-001.5 | ⬜ | Graceful shutdown: stop accepting new connections, drain in-flight requests, flush WAL, close listeners |
| VV-NET-001.6 | ⬜ | Integration test: start server as a subprocess, connect a raw TCP client, issue shutdown, assert clean exit code |

### VV-NET-002 — Binary wire protocol (frame + opcode + zero-copy payload)  ⬜
**Spec:** CORE_ARCHITECTURE.md §3.2 (cache-aligned struct pattern), §6.5
**Blocked by:** none
**Needs:** none

| Task | Status | Detail |
|---|---|---|
| VV-NET-002.1 | ⬜ | Define `WireFrameHeader` (magic, version, opcode, flags, payload_len, request_id) — fixed-size, cache-line-aware layout per project's `#[repr(C, align(64))]` convention |
| VV-NET-002.2 | ⬜ | `Opcode` enum: `Execute, Query, TxnBegin, TxnCommit, TxnAbort, Health, Ping` |
| VV-NET-002.3 | ⬜ | Encode/decode: header + payload via `bytes::Bytes` — decode returns a zero-copy view into the read buffer, no payload memcpy |
| VV-NET-002.4 | ⬜ | Length-prefixed stream framing: handle partial reads/writes across TCP segment boundaries |
| VV-NET-002.5 | ⬜ | Frame checksum: hardware CRC-32C via VV-HAL-017 kernel |
| VV-NET-002.6 | ⬜ | Round-trip unit tests: encode→decode identity for every opcode |
| VV-NET-002.7 | ⬜ | Fuzz test (`cargo-fuzz`, ties VV-TST-011): truncated frame, oversized `payload_len`, bad magic bytes never panic — always a typed decode error |

### VV-NET-003 — TCP transport (baseline, portable)  ⬜
**Spec:** CORE_ARCHITECTURE.md §6.5
**Blocked by:** VV-NET-002 (frame format)
**Needs:** none — Windows dev box (loopback)

| Task | Status | Detail |
|---|---|---|
| VV-NET-003.1 | ⬜ | `tokio::net::TcpListener` accept loop, one task spawned per connection |
| VV-NET-003.2 | ⬜ | Per-connection read loop: buffered read → `WireFrame` decode → dispatch to opcode handler |
| VV-NET-003.3 | ⬜ | Per-connection write half: response frame encode over a bounded `mpsc` channel (backpressure if the socket write is slow) |
| VV-NET-003.4 | ⬜ | `TCP_NODELAY` set on accept; document why (Nagle's algorithm defeats sub-millisecond latency goals) |
| VV-NET-003.5 | ⬜ | Connection cap + idle-timeout eviction |
| VV-NET-003.6 | ⬜ | Integration test: 100 concurrent TCP clients round-trip Execute/Query against a local server |

### VV-NET-004 — gRPC / tonic surface (Execute/Query/Txn*/Health)  ⬜
**Spec:** CORE_ARCHITECTURE.md §6.5
**Blocked by:** VV-SDK-001 (shared protobuf schema — this feature *defines* the service the SDKs consume, so it and VV-SDK-001 land together)
**Needs:** none

| Task | Status | Detail |
|---|---|---|
| VV-NET-004.1 | ⬜ | Write `proto/vectorvolt.proto`: `VectorVoltService` with `Execute`, `Query`, `TxnBegin`, `TxnCommit`, `TxnAbort`, `Health` RPCs |
| VV-NET-004.2 | ⬜ | Wire `tonic-build` codegen in `build.rs` |
| VV-NET-004.3 | ⬜ | Implement `Execute` handler → dispatch into `vectorvolt-engine` write path |
| VV-NET-004.4 | ⬜ | Implement `Query` handler → dispatch into read path (point/range/secondary per VV-QRY-001..003) |
| VV-NET-004.5 | ⬜ | Implement `TxnBegin`/`TxnCommit`/`TxnAbort` — return `UNIMPLEMENTED` status until VV-TXN-014 explicit transactions land; wire real logic once unblocked |
| VV-NET-004.6 | ⬜ | Implement `Health` per `grpc.health.v1.Health` (ties VV-NET-009) |
| VV-NET-004.7 | ⬜ | Integration test via a generated tonic test client hitting every RPC |

### VV-NET-005 — Zero-copy request → ring buffer routing  ⬜
**Spec:** CORE_ARCHITECTURE.md §4.1 (LMAX Disruptor pipeline diagram)
**Blocked by:** VV-TXN-008 (LMAX Disruptor ring buffer — not built yet), VV-NET-002
**Needs:** none

| Task | Status | Detail |
|---|---|---|
| VV-NET-005.1 | ⬜ | Define `RingBufferProducer` handle usable from a network task (thin wrapper the network layer holds; real ring buffer lands with VV-TXN-008) |
| VV-NET-005.2 | ⬜ | Route decoded frame payload (`Bytes`, no copy) into a ring buffer slot |
| VV-NET-005.3 | ⬜ | Correlate `request_id` → response channel via a `oneshot` per in-flight request |
| VV-NET-005.4 | ⬜ | Backpressure: ring buffer full → reject with retriable error (ties LD-BACK / VV-WAL-012 token bucket) |
| VV-NET-005.5 | ⬜ | Test: pointer-identity assertion — payload's data pointer in the network read buffer equals the pointer the ring-buffer slot references (proves zero-copy, not just zero-`memcpy`-by-convention) |

### VV-NET-006 — Connection pool + per-core shard routing  ⬜
**Spec:** CORE_ARCHITECTURE.md §4.2 core isolation model
**Blocked by:** VV-ENG-016 (shard-per-core mapping)
**Needs:** none

| Task | Status | Detail |
|---|---|---|
| VV-NET-006.1 | ⬜ | `ShardRouter` trait: `shard_id -> core/thread handle` |
| VV-NET-006.2 | ⬜ | Local round-robin implementation reading VV-ENG-016's shard-per-core map |
| VV-NET-006.3 | ⬜ | Connection acceptor distributes new connections across reactor cores (`SO_REUSEPORT` where available, manual dispatch otherwise) |
| VV-NET-006.4 | ⬜ | Per-shard queue-depth gauge (feeds VV-OBS-005 metrics) |
| VV-NET-006.5 | ⬜ | Test: requests for shard N always execute on shard N's designated core — no cross-core hop |

### VV-NET-007 — TLS + mTLS  ⬜
**Spec:** CORE_ARCHITECTURE.md §6.5; LAYER 13 VV-SEC-001/002
**Blocked by:** VV-NET-003, VV-NET-004
**Needs:** none

| Task | Status | Detail |
|---|---|---|
| VV-NET-007.1 | ⬜ | Wire `rustls` server config (cert + key from file) into the tonic gRPC server |
| VV-NET-007.2 | ⬜ | mTLS: verify client cert against a CA bundle, extract client identity into request context |
| VV-NET-007.3 | ⬜ | Apply the same `rustls` config to the raw `TcpTransport` path (VV-NET-003), not just gRPC |
| VV-NET-007.4 | ⬜ | Cert rotation: watch cert file, reload `rustls::ServerConfig` without restarting listeners |
| VV-NET-007.5 | ⬜ | Test: self-signed handshake succeeds; invalid/expired client cert is rejected; cert-rotation-mid-flight test |

### VV-NET-008 — `x-vv-txn` fingerprint response metadata  ⬜
**Spec:** CORE_ARCHITECTURE.md §6.5
**Blocked by:** VV-TXN-015 (128-bit FNV-1a transaction fingerprint — not built yet)
**Needs:** none

| Task | Status | Detail |
|---|---|---|
| VV-NET-008.1 | ⬜ | Define fingerprint wire format matching VV-TXN-015's 128-bit value |
| VV-NET-008.2 | ⬜ | Attach fingerprint as a gRPC trailer/metadata entry `x-vv-txn` |
| VV-NET-008.3 | ⬜ | Attach fingerprint as a `WireFrame` field for the raw TCP path |
| VV-NET-008.4 | ⬜ | Test: same logical command produces a stable, deterministic fingerprint across both transports |

### VV-NET-009 — gRPC health probe  ⬜
**Spec:** CORE_ARCHITECTURE.md §6.5
**Blocked by:** VV-NET-004
**Needs:** none

| Task | Status | Detail |
|---|---|---|
| VV-NET-009.1 | ⬜ | Implement standard `grpc.health.v1.Health/Check` and `/Watch` |
| VV-NET-009.2 | ⬜ | Wire real signals (WAL open, shards ready, not draining) instead of an always-`SERVING` stub |
| VV-NET-009.3 | ⬜ | `vvctl status` subcommand consumes this endpoint |
| VV-NET-009.4 | ⬜ | Test: health flips to `NOT_SERVING` during a simulated graceful-shutdown drain |

### VV-NET-010 — Busy-poll socket read (no syscall per packet)  ⬜
**Spec:** CORE_ARCHITECTURE.md §6.1
**Blocked by:** VV-NET-003
**Needs:** 1 Linux host for the `SO_BUSY_POLL` variant only (see environment table)

| Task | Status | Detail |
|---|---|---|
| VV-NET-010.1 | ⬜ | Baseline: document/verify tokio's epoll (Linux) / IOCP (Windows) edge-triggered read path is already close to optimal without extra work |
| VV-NET-010.2 | ⬜ | Optional busy-poll mode behind a config flag: spin `recv()` on a pinned core instead of awaiting reactor wakeup (ties VV-HAL-024 core pinning) |
| VV-NET-010.3 | ⬜ | Benchmark: busy-poll vs reactor-driven p50/p99 latency under load (Criterion, ties VV-TST-007) |
| VV-NET-010.4 | 🧊 | Linux-only `SO_BUSY_POLL` sockopt variant — needs 1 Linux host |

### VV-NET-011 — eBPF / AF_XDP kernel bypass (~5–20µs RTT)  🧊
**Spec:** CORE_ARCHITECTURE.md §6.2; ekos-ebpf crate mentioned in doc
**Blocked by:** VV-NET-013 (`Transport` trait must exist first so this plugs in behind it)
**Needs:** 1 Linux host (generic/SKB mode works on any NIC incl. AWS ENA; native-mode driver support does not exist on ENA — see environment table)

Portable part — buildable on Windows now:

| Task | Status | Detail |
|---|---|---|
| VV-NET-011.1 | ⬜ | Transport capability negotiation: probe for AF_XDP support at startup, fall back to `TcpTransport` (VV-NET-003) if unavailable |
| VV-NET-011.2 | ⬜ | Fallback-path test: force AF_XDP init failure, assert clean fallback to TCP with no dropped connections |

Linux-only part — 🧊, lands once a Linux host is scheduled:

| Task | Status | Detail |
|---|---|---|
| VV-NET-011.3 | 🧊 | `aya-rs` XDP program: parse Ethernet/IP/UDP headers, filter VectorVolt's port, `XDP_REDIRECT` matching packets to the AF_XDP socket |
| VV-NET-011.4 | 🧊 | AF_XDP socket setup: UMEM registration, Fill/Completion/RX/TX ring configuration |
| VV-NET-011.5 | 🧊 | `AfXdpTransport`: implements the `Transport` trait (VV-NET-013) over the user-space rings |
| VV-NET-011.6 | 🧊 | NIC/driver compatibility check at startup: native XDP (ixgbe, i40e, mlx5) vs generic/SKB-mode fallback (works on any NIC incl. ENA, lower speedup) — log which mode is active |
| VV-NET-011.7 | 🧊 | Privilege setup docs: `CAP_NET_ADMIN`/`CAP_BPF` requirements, systemd unit capability grant example |
| VV-NET-011.8 | 🧊 | Benchmark: measure real round-trip vs the TCP baseline; report against the 5–20µs target, noting which mode (native/generic) was measured |

### VV-NET-012 — DPDK poll-mode driver (~1–3µs RTT)  🧊
**Spec:** CORE_ARCHITECTURE.md §6.3
**Blocked by:** VV-NET-013
**Needs:** 1 Linux host **with** an ENA-compatible, SR-IOV/Enhanced-Networking-enabled EC2 instance type (`c5n`/`m5n`/`r5n` or similar) — confirm the two available boxes are this family before scheduling

Portable part:

| Task | Status | Detail |
|---|---|---|
| VV-NET-012.1 | ⬜ | Define the `Transport` trait's DPDK backend surface now (empty/stub impl compiles on Windows) |
| VV-NET-012.2 | ⬜ | `dpdk-sys` binding evaluation: vendored bindgen bindings vs. hand-rolled FFI — write the decision as a doc comment |

Linux + specific-NIC part — 🧊:

| Task | Status | Detail |
|---|---|---|
| VV-NET-012.3 | 🧊 | EAL init: huge pages, core mask, PCI device binding to `vfio-pci`/`igb_uio` |
| VV-NET-012.4 | 🧊 | Poll-mode RX/TX loop pinned to a dedicated core (ties VV-HAL-024) |
| VV-NET-012.5 | 🧊 | Zero-copy protobuf parse directly out of the DPDK `mbuf` — no copy into application memory |
| VV-NET-012.6 | 🧊 | Document the AWS-specific requirement explicitly: DPDK's `ena` PMD needs SR-IOV enabled and a supported instance family; a plain `t3`/`m5` will not work |
| VV-NET-012.7 | 🧊 | Benchmark against the 1–3µs target |
| VV-NET-012.8 | 🧊 | Fallback test: DPDK EAL init failure → clean fallback to `TcpTransport` |

### VV-NET-013 — `Transport` trait abstraction (TCP/XDP/DPDK swap)  ⬜
**Spec:** CORE_ARCHITECTURE.md §6 (all sub-sections); the feature matrix explicitly calls this
"buildable now — enables 011/012 later"
**Blocked by:** VV-NET-002
**Needs:** none — this is the load-bearing task that de-risks the whole Linux-only backlog

| Task | Status | Detail |
|---|---|---|
| VV-NET-013.1 | ⬜ | Define `trait Transport: Send + Sync { fn send(&self, frame: &[u8]) -> io::Result<()>; fn recv(&self) -> io::Result<Bytes>; fn local_addr(&self) -> SocketAddr; ... }` |
| VV-NET-013.2 | ⬜ | `TransportKind` enum (`Tcp \| AfXdp \| Dpdk \| Rdma`) selecting backend at startup with an ordered fallback chain |
| VV-NET-013.3 | ⬜ | `TcpTransport`: reference implementation wrapping VV-NET-003 |
| VV-NET-013.4 | ⬜ | `MockTransport`: in-memory implementation for unit-testing the network layer without real sockets |
| VV-NET-013.5 | ⬜ | Object-safety test (trait must be usable as `Box<dyn Transport>`) |
| VV-NET-013.6 | ⬜ | Wire transport selection into `vectorvolt-server` startup, config-driven |

### VV-NET-014 — io_uring socket I/O  🧊
**Spec:** CORE_ARCHITECTURE.md §6.1
**Blocked by:** VV-NET-013
**Needs:** 1 Linux host, kernel ≥ 5.1 (5.6+ recommended)

| Task | Status | Detail |
|---|---|---|
| VV-NET-014.1 | ⬜ | Portable: confirm the reactor abstraction behind `Transport` has no epoll-specific assumptions baked in, so io_uring can plug in later without a rewrite |
| VV-NET-014.2 | 🧊 | Evaluate `tokio-uring` vs raw `io-uring` crate; write the decision |
| VV-NET-014.3 | 🧊 | io_uring accept/read/write submission-queue wiring |
| VV-NET-014.4 | 🧊 | Benchmark vs the epoll baseline (VV-NET-003) |

### VV-NET-015 — RDMA transport  🧊
**Spec:** CORE_ARCHITECTURE.md §6.4
**Blocked by:** VV-NET-013, VV-WAL-015/016
**Needs:** RDMA-capable NIC (Mellanox/ConnectX, InfiniBand, or AWS EFA) — **not** provided by the two
generic EC2 boxes; requires a different instance family

| Task | Status | Detail |
|---|---|---|
| VV-NET-015.1 | ⬜ | `Transport` trait RDMA backend stub — compiles everywhere, returns `Unsupported` when no RDMA NIC is present |
| VV-NET-015.2 | 🧊 | `rdma-core` binding evaluation |
| VV-NET-015.3 | 🧊 | Queue-pair setup + memory registration for WAL segment buffers |
| VV-NET-015.4 | 🧊 | RDMA WRITE-based WAL shipping (ties VV-WAL-015) — replica CPU not involved in receive path |
| VV-NET-015.5 | 🧊 | Fallback to TCP WAL streaming when no RDMA NIC is detected at startup |
| VV-NET-015.6 | 🧊 | Benchmark: 1–5µs target vs ~100µs TCP baseline |

---

# LAYER 8 — Cluster (`vectorvolt-cluster`)

Two protocols, two concerns, per `VECTORVOLT_DISTRIBUTED_REPLICATION.md`: **SWIM** for "who is
alive" (cluster-wide, gossip, eventually consistent), **Raft** for "who leads this shard" (per
shard-group, strongly consistent, quorum). Do not conflate them — SWIM never elects a shard leader,
Raft never decides cluster membership.

### VV-CLU-001 — SWIM gossip membership (O(log n), 800+ nodes)  ⬜
**Spec:** DISTRIBUTED_REPLICATION.md Part 1; LD-SWIM
**Blocked by:** VV-NET-013 (needs some transport to send PING/ACK over)
**Needs:** deterministic sim for logic (no host); 2 Linux hosts for a real detection-latency sample

| Task | Status | Detail |
|---|---|---|
| VV-CLU-001.1 | ⬜ | `MemberState` enum (`Alive, Suspect, Dead, Left`) + `Member { node_id, addr, incarnation, state }` |
| VV-CLU-001.2 | ⬜ | `SwimConfig` with the doc's exact defaults: `ping_interval_ms=200`, `ping_timeout_ms=50`, `suspicion_multiplier=4`, `indirect_checks=3`, `gossip_fanout=3` |
| VV-CLU-001.3 | ⬜ | PING/ACK/PING-REQ wire messages (small binary protocol, reuse VV-NET-002 framing) |
| VV-CLU-001.4 | ⬜ | `MembershipTable`: local view with incarnation-number conflict resolution (higher incarnation wins) |
| VV-CLU-001.5 | ⬜ | Periodic ping task: pick a random peer, send PING, await ACK within `ping_timeout_ms` |
| VV-CLU-001.6 | ⬜ | Piggyback gossip payload on PING/ACK (infection-style propagation), bounded per-message gossip buffer |
| VV-CLU-001.7 | ⬜ | JOIN protocol: new node contacts seed list, seed replies with full membership snapshot |
| VV-CLU-001.8 | ⬜ | LEAVE protocol: graceful broadcast, distinct from a detected DEAD |
| VV-CLU-001.9 | ⬜ | Deterministic single-process simulation: N simulated nodes, mock clock/transport, assert full-membership convergence — primary test vehicle |
| VV-CLU-001.10 | ⬜ | Real 2-Linux-host test: two EC2 boxes join via seed list, observe convergence over the real network |

### VV-CLU-002 — Ping / ack / ping-req indirect failure detection  ⬜
**Spec:** DISTRIBUTED_REPLICATION.md Part 1 (membership state machine)
**Blocked by:** VV-CLU-001
**Needs:** deterministic sim; 2 Linux hosts for a real timing sample

| Task | Status | Detail |
|---|---|---|
| VV-CLU-002.1 | ⬜ | PING-REQ: ask `indirect_checks=3` peers to probe the suspicious target on this node's behalf |
| VV-CLU-002.2 | ⬜ | Suspicion timer state machine: `Alive->Suspect` on direct-ping timeout, `Suspect->Dead` after `suspicion_multiplier × ping_interval` (800ms default) with no ACK from anyone |
| VV-CLU-002.3 | ⬜ | REJOIN handling: `Dead->Alive` when a "dead" node's heartbeat is observed again (false-positive recovery), incarnation number bumped |
| VV-CLU-002.4 | ⬜ | Deterministic sim test: inject a dropped ACK, assert the Suspect→Dead transition fires at exactly the configured timeout |
| VV-CLU-002.5 | ⬜ | Real 2-host test: `kill -9` one node's process, measure actual wall-clock detection latency against the 800ms suspicion budget |

### VV-CLU-003 — Infection-style state propagation  ⬜
**Spec:** DISTRIBUTED_REPLICATION.md Part 1
**Blocked by:** VV-CLU-001
**Needs:** deterministic sim (this is specifically about O(log n) convergence math, which is honestly testable at any scale in-process)

| Task | Status | Detail |
|---|---|---|
| VV-CLU-003.1 | ⬜ | Gossip message struct: bounded list of recent state changes (JOIN/LEAVE/DEAD/incarnation bump) |
| VV-CLU-003.2 | ⬜ | Piggyback selection policy: prioritize least-disseminated updates first, bound total message size |
| VV-CLU-003.3 | ⬜ | Anti-entropy fallback: periodic full membership exchange to bound worst-case convergence time |
| VV-CLU-003.4 | ⬜ | Deterministic sim: simulate 50 / 200 / 800 nodes, measure rounds-to-convergence, assert O(log n) scaling |
| VV-CLU-003.5 | ⬜ | Metric: convergence-rounds histogram exported for VV-OBS |

### VV-CLU-004 — 4096 virtual nodes on consistent hash ring  ⬜
**Spec:** DISTRIBUTED_REPLICATION.md Part 2; LD-VNODE
**Blocked by:** none
**Needs:** none — pure data structure

| Task | Status | Detail |
|---|---|---|
| VV-CLU-004.1 | ⬜ | `HashRing { vnode_map: [ShardId; 4096] }` |
| VV-CLU-004.2 | ⬜ | `route_command(shard_key: &[u8]) -> ShardId` matching the doc's exact signature: `xxhash64(shard_key) % 4096` → vnode → shard |
| VV-CLU-004.3 | ⬜ | Initial ring construction for N physical shards: even vnode distribution (4096/N with remainder handling) |
| VV-CLU-004.4 | ⬜ | Ring mutation: add shard (donor shards each give up ~1/(N+1) of their vnodes) / remove shard (its vnodes redistributed) |
| VV-CLU-004.5 | ⬜ | Property test: for N shards in 1..64, verify distribution variance is bounded and add/remove moves ≈ 1/N of vnodes, matching the doc's 8→9 shard worked example (~11% movement) |
| VV-CLU-004.6 | ⬜ | Serialize ring state for `ShardCatalog` persistence |

### VV-CLU-005 — 1–65,536 physical shard support  ⬜
**Spec:** LD-VNODE
**Blocked by:** VV-CLU-004
**Needs:** none

| Task | Status | Detail |
|---|---|---|
| VV-CLU-005.1 | ⬜ | `ShardId` as `u32` (supports the full 65,536+ range) |
| VV-CLU-005.2 | ⬜ | Validation: reject configs where shard count exceeds 4096 vnodes with a clear error (shard count should never exceed vnode count in practice — document this as an operational limit) |
| VV-CLU-005.3 | ⬜ | Stress test: construct a ring at max shard count, verify vnode assignment doesn't silently give some shards zero vnodes without erroring |
| VV-CLU-005.4 | ⬜ | Benchmark: ring lookup stays O(1) at max shard count |

### VV-CLU-006 — `ShardCatalog` authoritative routing table  ⬜
**Spec:** DISTRIBUTED_REPLICATION.md Part 2 (`ShardCatalog`/`ShardGroup` structs, verbatim)
**Blocked by:** VV-CLU-004
**Needs:** none

| Task | Status | Detail |
|---|---|---|
| VV-CLU-006.1 | ⬜ | `ShardCatalog { vnode_map: [ShardId; 4096], shard_topology: HashMap<ShardId, ShardGroup>, epoch: u64 }` exactly per doc |
| VV-CLU-006.2 | ⬜ | `ShardGroup { leader: NodeId, followers: Vec<NodeId>, epoch: u64 }` |
| VV-CLU-006.3 | ⬜ | Persistence: catalog stored via `vectorvolt-store` KO put/get so it survives a control-plane restart |
| VV-CLU-006.4 | ⬜ | Mutation API: `add_shard`, `remove_shard`, `update_leader`, `increment_epoch` — every mutation bumps epoch |
| VV-CLU-006.5 | ⬜ | Versioned snapshot read (so consumers can detect "my cached copy is stale") |
| VV-CLU-006.6 | ⬜ | Unit tests: mutation sequences produce the expected topology |

### VV-CLU-007 — Catalog caching per node + per SDK  ⬜
**Spec:** DISTRIBUTED_REPLICATION.md Part 2 ("Every vectorvolt-server ... caches a local copy")
**Blocked by:** VV-CLU-006
**Needs:** none

| Task | Status | Detail |
|---|---|---|
| VV-CLU-007.1 | ⬜ | `CachedShardCatalog`: local copy + last-refreshed epoch |
| VV-CLU-007.2 | ⬜ | Push invalidation: control plane notifies nodes on topology change (piggyback on SWIM gossip, or a dedicated notify RPC) |
| VV-CLU-007.3 | ⬜ | Pull fallback: periodic poll in case a push notification was missed |
| VV-CLU-007.4 | ⬜ | Wire contract for SDK-side cache: `GetCatalog` RPC + delta-update stream (consumed by VV-SDK-005) |
| VV-CLU-007.5 | ⬜ | Staleness test: epoch mismatch detected, triggers refresh, routing decision corrected |

### VV-CLU-008 — Master-slave (leader-follower) replication  ⬜
**Spec:** VECTORVOLT_DISTRIBUTED_REPLICATION.md Part 3; LD-QUORUM
**Blocked by:** VV-NET-001 (server), VV-WAL-015 (WAL shipper), VV-CLU-006 (shard catalog)
**Needs:** 2 Linux hosts minimum — this is the feature the two EC2 boxes exist for

| Task | Status | Detail |
|---|---|---|
| VV-CLU-008.1 | ⬜ | Define `ReplicaRole { Leader, Follower, Learner }` state machine per shard |
| VV-CLU-008.2 | ⬜ | `ReplicationSession`: leader-side per-follower stream state (`last_acked_lsn`, in-flight count, connection handle) |
| VV-CLU-008.3 | ⬜ | WAL segment shipping protocol: stream the doc's binary format (32-byte segment header `[shard_id:u32][epoch:u64][segment_id:u64][first_lsn:u64][crc32:u32]` + variable WAL entries) over gRPC |
| VV-CLU-008.4 | ⬜ | Follower-side: append shipped frames to its own WAL, send `ack(lsn)` back to leader |
| VV-CLU-008.5 | ⬜ | Leader-side majority-quorum commit tracker: per-LSN ack count, mark committed at majority (ties VV-CLU-011) |
| VV-CLU-008.6 | ⬜ | Sync vs. async replication mode config (wait-for-quorum before client ack, vs. fire-and-forget) |
| VV-CLU-008.7 | ⬜ | Follower catch-up / full resync: a new or badly-lagging follower requests a snapshot + WAL-from-LSN when the gap exceeds retained segments |
| VV-CLU-008.8 | ⬜ | Follower promotion: follower → leader on a failover trigger (ties VV-CLU-013), correctly replaying any locally-uncommitted entries first |
| VV-CLU-008.9 | ⬜ | Replication-lag metric (bytes/LSNs behind) surfaced per follower (feeds VV-CLU-016 stale-read routing) |
| VV-CLU-008.10 | ⬜ | Deterministic single-process simulation: 1 leader + 2 followers in-process, fault injection (dropped acks, delayed frames) — acceptable substitute for pure protocol-logic correctness |
| VV-CLU-008.11 | ⬜ | **Real 2-Linux-host test:** leader on host A, follower on host B, verify WAL replicates correctly and lag stays bounded under sustained write load — mandatory, cannot be honestly simulated in one process |
| VV-CLU-008.12 | ⬜ | **Real 2-host test:** kill the leader process, verify the follower has the full WAL up to its last-acked LSN with zero data loss for committed writes |

### VV-CLU-009 — Raft consensus (`raft-rs`) per shard group  ⬜
**Spec:** DISTRIBUTED_REPLICATION.md Part 3; LD-EPOCH, LD-FAILOVER
**Blocked by:** VV-CLU-010 (storage impl), VV-CLU-006
**Needs:** deterministic sim for correctness (mandatory, primary test vehicle); 2 Linux hosts for a partial/functional multi-host signal — true 3-independent-host fidelity needs a 3rd host

| Task | Status | Detail |
|---|---|---|
| VV-CLU-009.1 | ⬜ | Vendor `raft-rs`; `RaftNode` wrapper per shard group (raw_node + storage + transport handle) |
| VV-CLU-009.2 | ⬜ | Raft message transport: carry `raft-rs`'s `Message` proto (AppendEntries, RequestVote, Snapshot) over gRPC between shard-group peers |
| VV-CLU-009.3 | ⬜ | Tick driver: dedicated tokio task calling `raw_node.tick()` on a configurable interval, draining `Ready` and applying entries |
| VV-CLU-009.4 | ⬜ | Apply committed entries to shard arena state (`Ready.committed_entries` → engine command dispatch) |
| VV-CLU-009.5 | ⬜ | Election timing config: `election_tick`/`heartbeat_tick` tuned to the doc's T+500ms/T+600ms failover budget |
| VV-CLU-009.6 | ⬜ | Shard-group bootstrap: 3-replica minimum (`ConfState` with 3 voters) and 5-replica production config |
| VV-CLU-009.7 | ⬜ | Snapshot send/receive path for lagging followers (ties VV-CLU-010's `snapshot()`) |
| VV-CLU-009.8 | ⬜ | **Deterministic single-process simulation**: 3+ `RaftNode`s in one process, in-memory mock transport, mock clock — the primary correctness-test vehicle for election and log replication (standard practice, matches etcd/TiKV's own test approach and this project's LD-CHAOS philosophy) |
| VV-CLU-009.9 | ⬜ | Chaos test in the sim: simulated network partition (drop messages between a node subset), assert no dual-leader exists within a single epoch |
| VV-CLU-009.10 | ⬜ | Real multi-host smoke test: 2 replica processes on one EC2 box + 1 on the other exercises quorum wiring end-to-end; document explicitly that this is *not* a fault-independent 3-node test (losing 1 host loses 2 replicas at once) |
| VV-CLU-009.11 | ⬜ | Benchmark: measure real election time on the 2-host/3-process setup against the <2s target, reporting it as a partial signal pending a true 3rd host |

### VV-CLU-010 — WAL-backed `raft::Storage` impl  ⬜
**Spec:** DISTRIBUTED_REPLICATION.md Part 3 (exact trait signature)
**Blocked by:** VV-WAL-007 (read_range — already ✅), VV-STO-017 (COW snapshots)
**Needs:** none — pure storage-layer logic, deterministic-sim-testable

| Task | Status | Detail |
|---|---|---|
| VV-CLU-010.1 | ⬜ | `initial_state()`: read persisted `HardState`/`ConfState` from a WAL metadata region |
| VV-CLU-010.2 | ⬜ | `entries(lo, hi, max_size)`: read WAL frames by LSN range, reusing `VV-WAL-007 read_range` |
| VV-CLU-010.3 | ⬜ | `term(idx)`: index→term lookup; maintain a small in-memory index to avoid a full WAL scan per call |
| VV-CLU-010.4 | ⬜ | `first_index()` / `last_index()` derived from WAL segment bounds |
| VV-CLU-010.5 | ⬜ | `snapshot(request_index)`: trigger or return a cached engine snapshot (ties VV-STO-017) |
| VV-CLU-010.6 | ⬜ | `compact(index)`: truncate WAL entries below the given index once snapshotted (ties VV-WAL-013 log compaction) |
| VV-CLU-010.7 | ⬜ | Persist Raft `HardState` (term, vote, commit) durably and atomically alongside the WAL append it corresponds to |
| VV-CLU-010.8 | ⬜ | Unit tests covering every trait method's edge cases (empty log, single entry, index at segment boundary) |

### VV-CLU-011 — Majority quorum writes (configurable sync/async)  ⬜
**Spec:** DISTRIBUTED_REPLICATION.md Part 3 ("Write commit requires: ACK from 2 of 3"); LD-QUORUM
**Blocked by:** VV-CLU-009, VV-CLU-010
**Needs:** deterministic sim (3-node in-process is sufficient for this logic)

| Task | Status | Detail |
|---|---|---|
| VV-CLU-011.1 | ⬜ | `QuorumConfig { Majority \| All }` per shard group, config-driven per LD-QUORUM |
| VV-CLU-011.2 | ⬜ | Commit-index calculation: track `match_index` per follower (raft-rs exposes this), compute the majority commit point |
| VV-CLU-011.3 | ⬜ | Client ack gating: leader responds to the client only once the entry reaches the configured quorum level |
| VV-CLU-011.4 | ⬜ | Async mode: leader acks after local WAL append, quorum catch-up proceeds in background — document the weaker durability tradeoff explicitly |
| VV-CLU-011.5 | ⬜ | Test: 3-node sim, kill 1 follower → majority (2/3) still commits; kill 2 → writes correctly block/reject |

### VV-CLU-012 — Epoch fencing (split-brain prevention)  ⬜
**Spec:** DISTRIBUTED_REPLICATION.md Part 4 (`EpochGuard` struct, verbatim); LD-EPOCH
**Blocked by:** VV-CLU-009
**Needs:** deterministic sim mandatory (network partition is unreproducible by killing a process — it needs a real `iptables DROP`); 2 Linux hosts for a reduced 2-vs-1 partition; full 5-node fidelity needs ≥5 hosts

| Task | Status | Detail |
|---|---|---|
| VV-CLU-012.1 | ⬜ | `EpochGuard { epoch: AtomicU64 }` + `check(claimed_epoch) -> Result<(), FencingError::StaleEpoch>` exactly per doc |
| VV-CLU-012.2 | ⬜ | Wire epoch-increment triggers at every call site: topology change, leader election, shard split/merge, failover |
| VV-CLU-012.3 | ⬜ | Attach epoch to every AppendEntries/WAL-replication RPC; receiver checks before appending |
| VV-CLU-012.4 | ⬜ | Persist epoch durably (survives restart), stored alongside Raft `HardState` |
| VV-CLU-012.5 | ⬜ | Deterministic sim: reproduce the doc's exact 5-node scenario — partition `{A,B}` vs `{C,D,E}`, `{C,D,E}` elects a new leader and bumps epoch, assert A's post-partition writes are silently rejected once observed at the new epoch |
| VV-CLU-012.6 | ⬜ | Real 2-host test: apply an `iptables DROP` rule between the two EC2 hosts to simulate a genuine network partition (not a process kill), verify the isolated side's writes are fenced on reconnection — explicitly note this validates a 2-node reduced case, not the doc's full 5-node scenario |

### VV-CLU-013 — <2 second failover (500ms heartbeat → election)  ⬜
**Spec:** DISTRIBUTED_REPLICATION.md Part 5 (T+0ms..T+1000ms timing table); LD-FAILOVER
**Blocked by:** VV-CLU-001 (SWIM DEAD detection), VV-CLU-009, VV-CLU-006
**Needs:** deterministic sim for the full budget; 2 Linux hosts for a real (if smaller-topology) timing sample

| Task | Status | Detail |
|---|---|---|
| VV-CLU-013.1 | ⬜ | Heartbeat timeout wired to 500ms default (maps to raft-rs `election_tick`/`heartbeat_tick`) |
| VV-CLU-013.2 | ⬜ | End-to-end orchestration implementing the doc's sequence: SWIM DEAD → shard-group election trigger → epoch bump → ShardCatalog update → gossip propagate |
| VV-CLU-013.3 | ⬜ | ShardCatalog update path fires the moment a new leader is elected (control plane pushes new topology) |
| VV-CLU-013.4 | ⬜ | Client notification path: gossip or catalog-refresh signal reaches the SDK cache (ties VV-CLU-007 / VV-SDK-005) |
| VV-CLU-013.5 | ⬜ | Deterministic sim: run the full timing budget across 100 randomized seeds, assert total elapsed < 2s p99.9 |
| VV-CLU-013.6 | ⬜ | Real 2-host test: kill the leader process, measure wall-clock time until a follower is servicing writes; report the number explicitly as a partial signal (true validation needs the full 3–5 node topology the doc specifies) |

### VV-CLU-014 — Client-side automatic failover + backoff  ⬜
**Spec:** DISTRIBUTED_REPLICATION.md Part 5 ("Client-Side Failover Behavior")
**Blocked by:** VV-CLU-007, VV-SDK-006 (shared retry/backoff module)
**Needs:** none — mockable server, no real cluster needed

| Task | Status | Detail |
|---|---|---|
| VV-CLU-014.1 | ⬜ | SDK-side failure detection: gRPC `UNAVAILABLE` / connection-refused triggers the failover path |
| VV-CLU-014.2 | ⬜ | Exponential backoff + jitter starting at 10ms, 3-retry budget, 5s default total client timeout — exactly per doc |
| VV-CLU-014.3 | ⬜ | Fresh `ShardCatalog` fetch triggered on failover |
| VV-CLU-014.4 | ⬜ | Retry against the new leader; note idempotency consideration (ties VV-TXN-015 fingerprint dedup) |
| VV-CLU-014.5 | ⬜ | Test: mock server returns `UNAVAILABLE` then succeeds — verify retry count and backoff timing match policy, and the call ultimately succeeds within budget |

### VV-CLU-015 — Shard rebalancing (move ~1/N on scale)  ⬜
**Spec:** DISTRIBUTED_REPLICATION.md Part 2 ("Shard Addition: Minimal Data Movement")
**Blocked by:** VV-CLU-004, VV-CLU-008
**Needs:** deterministic sim for the vnode math; 2 Linux hosts for real cross-network data migration under load

| Task | Status | Detail |
|---|---|---|
| VV-CLU-015.1 | ⬜ | Rebalance planner: given current ring + new shard count, compute the vnode reassignment plan (which vnodes move from which shard to which) |
| VV-CLU-015.2 | ⬜ | Data migration protocol: source shard streams affected key ranges to the destination shard (bulk WAL-record replay or snapshot+delta) |
| VV-CLU-015.3 | ⬜ | Dual-read window: destination shard is not authoritative until migration completes; router consults both source and destination during transition |
| VV-CLU-015.4 | ⬜ | Cutover: atomic `ShardCatalog` update (epoch bump) once migration is confirmed complete |
| VV-CLU-015.5 | ⬜ | Rollback path: abort migration, ring reverts to prior state if the destination fails mid-transfer |
| VV-CLU-015.6 | ⬜ | Deterministic sim: add a shard to an 8-shard ring, verify ~11% of data moves — matches the doc's worked example exactly |
| VV-CLU-015.7 | ⬜ | Real 2-host test: rebalance under live write load, measure data-moved percentage and confirm zero dropped/duplicated writes during cutover |

### VV-CLU-016 — Read replica routing (stale default ≤50ms)  ⬜
**Spec:** DISTRIBUTED_REPLICATION.md Part 6; LD-READ-02
**Blocked by:** VV-CLU-008 (needs a follower to route to)
**Needs:** deterministic sim (artificial lag injection is sufficient)

| Task | Status | Detail |
|---|---|---|
| VV-CLU-016.1 | ⬜ | `stale_read_tolerance_ms` config, default 50ms |
| VV-CLU-016.2 | ⬜ | Per-replica lag tracking: leader stamps each committed LSN with wall-clock time, follower reports `lag = now - leader_commit_time` |
| VV-CLU-016.3 | ⬜ | Router: select any `Alive` (SWIM) replica with `lag < tolerance` |
| VV-CLU-016.4 | ⬜ | Fallback: no replica within tolerance → route to leader |
| VV-CLU-016.5 | ⬜ | Deterministic sim test: artificial follower delay, verify inclusion/exclusion exactly at the threshold boundary |

### VV-CLU-017 — Consistent read opt-in (`/*+ consistent */`)  ⬜
**Spec:** DISTRIBUTED_REPLICATION.md Part 6; LD-READ-02
**Blocked by:** VV-QRY-018 (SQL parser — for the SQL-comment-hint path), VV-CLU-016
**Needs:** none

| Task | Status | Detail |
|---|---|---|
| VV-CLU-017.1 | ⬜ | SQL comment-hint recognition for `/*+ consistent */` (blocked on VV-QRY-018 landing first) |
| VV-CLU-017.2 | ⬜ | gRPC `Query` request `consistent: bool` flag for the non-SQL path (buildable now, doesn't wait on the SQL parser) |
| VV-CLU-017.3 | ⬜ | Router: `consistent=true` forces routing to the shard leader regardless of replica lag |
| VV-CLU-017.4 | ⬜ | Test: verify a consistent query always reflects the most recent committed write (read-your-writes check) |

### VV-CLU-018 — Read replica load balancer (health + lag + RTT)  ⬜
**Spec:** DISTRIBUTED_REPLICATION.md Part 6 ("Read Fabric Load Balancing", 4-step algorithm verbatim)
**Blocked by:** VV-CLU-001, VV-CLU-016
**Needs:** deterministic sim

| Task | Status | Detail |
|---|---|---|
| VV-CLU-018.1 | ⬜ | Health filter: only replicas with SWIM status `Alive` |
| VV-CLU-018.2 | ⬜ | Lag filter: apply VV-CLU-016's threshold |
| VV-CLU-018.3 | ⬜ | RTT tracking: rolling average ping RTT per replica (piggyback on SWIM pings or a dedicated health-check RTT probe) |
| VV-CLU-018.4 | ⬜ | Selection: lowest-RTT replica among the filtered candidate set |
| VV-CLU-018.5 | ⬜ | Fallback chain: no eligible replica → route to leader — implements the doc's exact 4-step algorithm |
| VV-CLU-018.6 | ⬜ | Test: 3 simulated replicas with distinct lag/RTT profiles, verify selection order matches the expected priority |

### VV-CLU-019 — `vectorvolt-reader` separate binary  ⬜
**Spec:** LD-READ-01 (CQRS at the process level)
**Blocked by:** VV-QRY-019 (PG wire, cross-layer), VV-CLU-007
**Needs:** none — two processes work fine on one Windows box; 2 EC2 hosts give a more realistic split

| Task | Status | Detail |
|---|---|---|
| VV-CLU-019.1 | ⬜ | Scaffold `vectorvolt-reader` crate as a standalone binary |
| VV-CLU-019.2 | ⬜ | PG wire listener on port 5432 (ties VV-QRY-019, cross-layer dependency) |
| VV-CLU-019.3 | ⬜ | Read-only engine handle: opens WAL/partitions read-only, reuses `vectorvolt-engine`'s read path, no write path linked in at all |
| VV-CLU-019.4 | ⬜ | Catalog cache client (consumes VV-CLU-007) |
| VV-CLU-019.5 | ⬜ | Read-replica registration with the control plane (announces itself as a routable replica) |
| VV-CLU-019.6 | ⬜ | Test: `vectorvolt-reader` serves a query while a separate `vectorvolt-server` process is writing concurrently — minimum 2 processes, either on one box or split across the 2 EC2 hosts |

### VV-CLU-020 — Role-based binary deployment (write/read/control)  ⬜
**Spec:** LD-RUNTIME-01
**Blocked by:** VV-NET-001, VV-CLU-019, VV-CTL-001
**Needs:** none

| Task | Status | Detail |
|---|---|---|
| VV-CLU-020.1 | ⬜ | Deployment config: a node declares its role(s) — `write`, `read`, `control`; single-node dev mode runs all three in one process |
| VV-CLU-020.2 | ⬜ | Process-supervision documentation/scripts for both 3-binaries-per-node and 1-binary-per-node topologies |
| VV-CLU-020.3 | ⬜ | Role-specific core-pinning defaults applied at startup (ties VV-HAL-025 core role map) |
| VV-CLU-020.4 | ⬜ | Test: a node configured `role=write` refuses to serve read-fabric endpoints, and vice versa |

### VV-CLU-021 — Shard recovery <60s for 100GB  ⬜
**Spec:** LOCKED_DECISIONS.md Performance Targets table
**Blocked by:** VV-STO-017/018 (snapshots), VV-TXN-006 (index rebuild on reopen — already ✅)
**Needs:** 1 Linux host with enough real disk to hold a 100GB fixture — the number is storage-hardware-dependent, not purely a code question

| Task | Status | Detail |
|---|---|---|
| VV-CLU-021.1 | ⬜ | Recovery orchestration: on restart, load the last snapshot then replay WAL from the snapshot LSN |
| VV-CLU-021.2 | ⬜ | Parallel partition reopen: mmap all micro-partitions concurrently instead of serially |
| VV-CLU-021.3 | ⬜ | Progress reporting during recovery (feeds `vvctl status`) |
| VV-CLU-021.4 | ⬜ | Benchmark harness: generate a synthetic 100GB shard fixture, measure cold-start recovery wall-clock time |
| VV-CLU-021.5 | ⬜ | Real Linux-host test on the EC2 box: confirm and record which storage class backs the instance (instance-store NVMe vs. EBS gp3) before quoting the recovery number as representative — the two are very different latency profiles |

### VV-CLU-022 — CXL 3.0 CAS-based consensus  🔬
**Spec:** CORE_ARCHITECTURE.md §8.1 ("nobody has done this yet")
**Blocked by:** hardware that does not exist yet
**Needs:** CXL 3.0 fabric-attached memory hardware — not available, not schedulable

| Task | Status | Detail |
|---|---|---|
| VV-CLU-022.1 | ⬜ | Define a `ConsensusBackend` trait now (buildable), with `RaftConsensus` as the sole real implementation and a reserved `CxlCasConsensus` variant that is never constructed |
| VV-CLU-022.2 | 🔬 | Feasibility/literature note captured in code comments only — no implementation scheduled until CXL 3.0 hardware access exists |

---

# LAYER 9 — Control Plane (`vectorvolt-control`)

### VV-CTL-001 — `vectorvolt-control` node binary  ⬜
**Spec:** CORE_ARCHITECTURE.md §8, §15
**Blocked by:** none
**Needs:** none

| Task | Status | Detail |
|---|---|---|
| VV-CTL-001.1 | ⬜ | Scaffold crate + `main.rs`, clap CLI (`--config`, `--http-port 8443`) |
| VV-CTL-001.2 | ⬜ | Config loading: role=control, cluster seed list |
| VV-CTL-001.3 | ⬜ | Startup: initialize `ShardCatalog` store, SWIM member list, HTTP server bind |
| VV-CTL-001.4 | ⬜ | Graceful shutdown: drain, persist catalog |
| VV-CTL-001.5 | ⬜ | Test: start/stop lifecycle, clean exit |

### VV-CTL-002 — Cluster metadata store  ⬜
**Spec:** CORE_ARCHITECTURE.md §8.2
**Blocked by:** VV-CLU-006
**Needs:** none

| Task | Status | Detail |
|---|---|---|
| VV-CTL-002.1 | ⬜ | Storage backend: persist via `vectorvolt-store` KO put/get — self-hosted, no external ZooKeeper/etcd dependency (per CORE_ARCHITECTURE.md §13) |
| VV-CTL-002.2 | ⬜ | Metadata schema: nodes, shard groups, epoch, topology version |
| VV-CTL-002.3 | ⬜ | Internal Rust read/write API |
| VV-CTL-002.4 | ⬜ | Durability: metadata writes go through the WAL like any other KO write |
| VV-CTL-002.5 | ⬜ | Test: restart the control node, verify metadata survives |

### VV-CTL-003 — Schema / catalog registry service  ⬜
**Spec:** CORE_ARCHITECTURE.md §8.2; ties VV-ENG-007
**Blocked by:** VV-ENG-007 (schema registry — not built yet in the engine layer either; this is the control-plane-facing copy)
**Needs:** none

| Task | Status | Detail |
|---|---|---|
| VV-CTL-003.1 | ⬜ | Schema data model: table name → column definitions |
| VV-CTL-003.2 | ⬜ | CRUD API for schema registration (internal first, HTTP later via VV-CTL-005) |
| VV-CTL-003.3 | ⬜ | Schema version history (feeds VV-SDK-013 N-2 compatibility) |
| VV-CTL-003.4 | ⬜ | Test: register/alter/fetch schema round-trip |

### VV-CTL-004 — Topology management API  ⬜
**Spec:** CORE_ARCHITECTURE.md §8
**Blocked by:** VV-CLU-006
**Needs:** none

| Task | Status | Detail |
|---|---|---|
| VV-CTL-004.1 | ⬜ | Internal API: `add_node`, `remove_node`, `add_shard`, `list_topology` |
| VV-CTL-004.2 | ⬜ | Wires into `ShardCatalog` mutation (VV-CLU-006) with epoch bump on every change |
| VV-CTL-004.3 | ⬜ | Validation: reject topology changes that would violate minimum-replica invariants |
| VV-CTL-004.4 | ⬜ | Test: add/remove node updates topology consistently |

### VV-CTL-005 — Admin HTTP API (:8443, axum, HTTPS)  ⬜
**Spec:** CORE_ARCHITECTURE.md §11.4
**Blocked by:** VV-NET-007 (TLS cert loading, reused here)
**Needs:** none

| Task | Status | Detail |
|---|---|---|
| VV-CTL-005.1 | ⬜ | Scaffold `axum` router bound to `:8443` |
| VV-CTL-005.2 | ⬜ | TLS via `rustls` (reuse VV-NET-007's cert-loading code) |
| VV-CTL-005.3 | ⬜ | Routes: `/api/v1/nodes`, `/api/v1/shards`, `/api/v1/health`, `/api/v1/schema` |
| VV-CTL-005.4 | ⬜ | `serde`-based JSON serialization for all responses |
| VV-CTL-005.5 | ⬜ | Error-handling middleware producing a consistent error envelope |
| VV-CTL-005.6 | ⬜ | Test: HTTP client integration test hitting every route |

### VV-CTL-006 — Node join / drain / decommission  ⬜
**Spec:** CORE_ARCHITECTURE.md §9 (`vvctl drain`)
**Blocked by:** VV-CTL-004
**Needs:** none

| Task | Status | Detail |
|---|---|---|
| VV-CTL-006.1 | ⬜ | Join: node registers with the control plane, receives an initial shard assignment + catalog snapshot |
| VV-CTL-006.2 | ⬜ | Drain: mark node "draining" — stop routing new shard leadership to it, migrate existing leaderships off |
| VV-CTL-006.3 | ⬜ | Decommission: once drained with no remaining leaderships, remove the node from topology entirely |
| VV-CTL-006.4 | ⬜ | State-machine test: `join -> active -> draining -> decommissioned` transitions |
| VV-CTL-006.5 | ⬜ | API endpoints: `POST /nodes/{id}/drain`, `DELETE /nodes/{id}` |
| VV-CTL-006.6 | ⬜ | `vvctl drain <node>` CLI wiring |

### VV-CTL-007 — Manual failover trigger  ⬜
**Spec:** CORE_ARCHITECTURE.md §9 (`vvctl failover <shard>`)
**Blocked by:** VV-CLU-009 (Raft leadership transfer)
**Needs:** none for the API/orchestration; a real cluster (see VV-CLU-008/009) to observe an actual leadership move

| Task | Status | Detail |
|---|---|---|
| VV-CTL-007.1 | ⬜ | API endpoint: `POST /shards/{id}/failover` |
| VV-CTL-007.2 | ⬜ | Trigger forced Raft leadership transfer (`raft-rs` `transfer_leader`) for the target shard group |
| VV-CTL-007.3 | ⬜ | Epoch bump + catalog update on completion |
| VV-CTL-007.4 | ⬜ | `vvctl failover <shard>` CLI wiring |
| VV-CTL-007.5 | ⬜ | Test: manual trigger completes, catalog reflects the new leader |

### VV-CTL-008 — Rebalance orchestration  ⬜
**Spec:** CORE_ARCHITECTURE.md §9 (`vvctl rebalance`)
**Blocked by:** VV-CLU-015 (rebalance planner/executor)
**Needs:** none for orchestration API; 2 Linux hosts to observe real rebalance progress

| Task | Status | Detail |
|---|---|---|
| VV-CTL-008.1 | ⬜ | API endpoint: `POST /rebalance` (drives VV-CLU-015) |
| VV-CTL-008.2 | ⬜ | Orchestration state machine: `planning -> migrating -> cutover -> complete`, with progress reporting |
| VV-CTL-008.3 | ⬜ | Cancel/pause support for an in-flight rebalance |
| VV-CTL-008.4 | ⬜ | `vvctl rebalance` CLI wiring |
| VV-CTL-008.5 | ⬜ | Test: trigger via API, poll status until complete |

### VV-CTL-009 — QoS rule injection at runtime  ⬜
**Spec:** CORE_ARCHITECTURE.md §9 (`vvctl qos override`)
**Blocked by:** VV-TXN-017 (`TransactionPriority` QoS tiers — not built yet)
**Needs:** none

| Task | Status | Detail |
|---|---|---|
| VV-CTL-009.1 | ⬜ | QoS rule schema: per-tenant or per-shard rate/priority overrides |
| VV-CTL-009.2 | ⬜ | API endpoint: `PUT /qos/{scope}` |
| VV-CTL-009.3 | ⬜ | Propagation to affected write nodes via a control-plane notify channel |
| VV-CTL-009.4 | ⬜ | `vvctl qos override` CLI wiring |
| VV-CTL-009.5 | ⬜ | Test: inject a rule, verify the write node's scheduler config updates live |

### VV-CTL-010 — Admin auth (bcrypt + TOTP, JWT 4h)  ⬜
**Spec:** CORE_ARCHITECTURE.md §11.4
**Blocked by:** VV-CTL-002 (credential storage)
**Needs:** none

| Task | Status | Detail |
|---|---|---|
| VV-CTL-010.1 | ⬜ | User credential store (bcrypt password hash) in the control-plane metadata store |
| VV-CTL-010.2 | ⬜ | TOTP enrollment + verification (RFC 6238) |
| VV-CTL-010.3 | ⬜ | Login endpoint issuing a signed JWT with 4h expiry |
| VV-CTL-010.4 | ⬜ | Auth middleware validating JWT on every protected route |
| VV-CTL-010.5 | ⬜ | Test: login flow succeeds; expired-token rejection; TOTP mismatch rejection |

### VV-CTL-011 — Svelte SPA admin panel (embedded `include_bytes!`)  ⬜
**Spec:** CORE_ARCHITECTURE.md §11.4
**Blocked by:** VV-CTL-005, VV-CTL-010
**Needs:** none (Node.js build toolchain, no Linux requirement)

| Task | Status | Detail |
|---|---|---|
| VV-CTL-011.1 | ⬜ | Scaffold a minimal Svelte app: login, node list, shard map, WAL-lag view |
| VV-CTL-011.2 | ⬜ | Build pipeline: `npm run build` output embedded via `include_bytes!`/`rust-embed` into the control binary |
| VV-CTL-011.3 | ⬜ | Serve embedded static assets from `axum` at `/` |
| VV-CTL-011.4 | ⬜ | Wire panel API calls to VV-CTL-005 routes + VV-CTL-010 auth |
| VV-CTL-011.5 | ⬜ | Test: the build produces a single binary that serves a working UI (manual smoke test is acceptable for a UI-shape feature) |

### VV-CTL-012 — Emergency stop / resume  ⬜
**Spec:** CORE_ARCHITECTURE.md §11.4
**Blocked by:** VV-CTL-005
**Needs:** none

| Task | Status | Detail |
|---|---|---|
| VV-CTL-012.1 | ⬜ | API endpoint: `POST /emergency/stop` — broadcast to all write nodes to reject new writes immediately (circuit breaker) |
| VV-CTL-012.2 | ⬜ | API endpoint: `POST /emergency/resume` |
| VV-CTL-012.3 | ⬜ | Propagation mechanism: control-plane push to all nodes (gossip or a dedicated control channel) |
| VV-CTL-012.4 | ⬜ | Audit-log entry for every emergency action (ties VV-CTL-013) |
| VV-CTL-012.5 | ⬜ | Test: stop then attempt a write (expect rejection); resume then retry (expect success) |

### VV-CTL-013 — Immutable audit log for all control actions  ⬜
**Spec:** CORE_ARCHITECTURE.md §11.4
**Blocked by:** VV-CTL-002
**Needs:** none

| Task | Status | Detail |
|---|---|---|
| VV-CTL-013.1 | ⬜ | Audit event schema: actor, action, target, timestamp, result |
| VV-CTL-013.2 | ⬜ | Append-only log storage (reuse WAL-style append, never mutated) |
| VV-CTL-013.3 | ⬜ | Middleware: every mutating control-plane API call auto-logs an audit event |
| VV-CTL-013.4 | ⬜ | Query API: `GET /audit?since=...` |
| VV-CTL-013.5 | ⬜ | Test: every mutation produces exactly one audit entry; confirm no update/delete API exists for the log |

### VV-CTL-014 — HITL Index Advisor  ⬜
**Spec:** CORE_ARCHITECTURE.md §11.4
**Blocked by:** VV-OBS-004 (per-operation latency histograms)
**Needs:** none

| Task | Status | Detail |
|---|---|---|
| VV-CTL-014.1 | ⬜ | Query pattern collector: sample slow/frequent queries |
| VV-CTL-014.2 | ⬜ | Advisor heuristic: suggest secondary-index candidates from scan-vs-index-hit ratio |
| VV-CTL-014.3 | ⬜ | API: `GET /advisor/suggestions` |
| VV-CTL-014.4 | ⬜ | Human-approval workflow: suggestions require explicit operator approval before any index is auto-created (Human-In-The-Loop, never fully automatic) |
| VV-CTL-014.5 | ⬜ | Test: a synthetic workload produces the expected suggestion |

### VV-CTL-015 — Backup status dashboard  ⬜
**Spec:** CORE_ARCHITECTURE.md §11.4
**Blocked by:** VV-STO-023 (PITR + catalog)
**Needs:** none

| Task | Status | Detail |
|---|---|---|
| VV-CTL-015.1 | ⬜ | Backup verification job status schema (last run, success/fail, verified checksum) |
| VV-CTL-015.2 | ⬜ | API: `GET /backup/status` |
| VV-CTL-015.3 | ⬜ | Wire to snapshot/PITR completion events (VV-STO layer) |
| VV-CTL-015.4 | ⬜ | Dashboard panel in the Svelte UI (ties VV-CTL-011) |
| VV-CTL-015.5 | ⬜ | Test: simulate a backup-completion event, verify status reflects it |

---

# LAYER 10 — SDKs (`vectorvolt-sdk-*`)

### VV-SDK-001 — Shared protobuf schema (single source for all SDKs)  ⬜
**Spec:** LD-SDK-01; CORE_ARCHITECTURE.md §12
**Blocked by:** none — this is the foundation every SDK and VV-NET-004 build on
**Needs:** none

| Task | Status | Detail |
|---|---|---|
| VV-SDK-001.1 | ⬜ | `proto/vectorvolt.proto`: top-level service + message definitions (Execute, Query, TxnBegin/Commit/Abort, Health, GetCatalog) |
| VV-SDK-001.2 | ⬜ | Versioning scheme for the `.proto` file (semver + N-2 compatibility notes, ties VV-SDK-013) |
| VV-SDK-001.3 | ⬜ | Build tooling: a single `buf`/`protoc` invocation generating Rust (tonic-build), C#, Python, Go, Java, and TS stubs into each SDK directory |
| VV-SDK-001.4 | ⬜ | CI job: proto lint + breaking-change detector (`buf breaking` or an equivalent diff check) |
| VV-SDK-001.5 | ⬜ | Doc generation from proto comments (`protoc-gen-doc`) |

### VV-SDK-002 — Rust SDK (`vectorvolt-client`)  ⬜
**Spec:** CORE_ARCHITECTURE.md §12
**Blocked by:** VV-SDK-001, VV-NET-004
**Needs:** none

| Task | Status | Detail |
|---|---|---|
| VV-SDK-002.1 | ⬜ | Scaffold `vectorvolt-client` crate, tonic-generated client wrapper |
| VV-SDK-002.2 | ⬜ | Async connection management (channel pooling, shared with VV-SDK-005/006) |
| VV-SDK-002.3 | ⬜ | Ergonomic `Client::execute()` / `Client::query()` wrappers over raw gRPC calls |
| VV-SDK-002.4 | ⬜ | Error type mapping: gRPC status → typed `VectorVoltError` enum |
| VV-SDK-002.5 | ⬜ | Integration test against a local `vectorvolt-server` instance |

### VV-SDK-003 — Typed query builder API  ⬜
**Spec:** CORE_ARCHITECTURE.md §12
**Blocked by:** VV-SDK-002
**Needs:** none

| Task | Status | Detail |
|---|---|---|
| VV-SDK-003.1 | ⬜ | Fluent builder: `.table("x").filter(...).limit(...)` |
| VV-SDK-003.2 | ⬜ | Column-name safety: generated types from the schema registry (VV-CTL-003) where available, macro-based fallback for MVP |
| VV-SDK-003.3 | ⬜ | Builder → wire `QueryRequest` serialization |
| VV-SDK-003.4 | ⬜ | Unit tests: builder produces the expected request for representative queries |

### VV-SDK-004 — Transaction builder  ⬜
**Spec:** CORE_ARCHITECTURE.md §12
**Blocked by:** VV-SDK-002, VV-TXN-014 (explicit transactions)
**Needs:** none

| Task | Status | Detail |
|---|---|---|
| VV-SDK-004.1 | ⬜ | `TxnBuilder`: `begin()` → add commands → `commit()`/`abort()` |
| VV-SDK-004.2 | ⬜ | Client-side fingerprint tracking, correlated with VV-NET-008's `x-vv-txn` header |
| VV-SDK-004.3 | ⬜ | Auto-retry-on-fencing-error hook (surfaces VV-CLU-012 stale-epoch errors to the client) |
| VV-SDK-004.4 | ⬜ | Test: multi-command transaction round-trip against a local server |

### VV-SDK-005 — Client-side shard catalog cache (direct-to-node, no hop)  ⬜
**Spec:** DISTRIBUTED_REPLICATION.md Part 2; CORE_ARCHITECTURE.md §12
**Blocked by:** VV-CLU-007
**Needs:** none

| Task | Status | Detail |
|---|---|---|
| VV-SDK-005.1 | ⬜ | `CatalogCache`: vnode_map + shard_topology + last-refresh epoch, mirroring VV-CLU-006's shape |
| VV-SDK-005.2 | ⬜ | Fetch-on-first-use + periodic background refresh task |
| VV-SDK-005.3 | ⬜ | `route(shard_key)`: local hash-ring lookup avoiding a coordinator hop, reusing VV-CLU-004's routing function client-side |
| VV-SDK-005.4 | ⬜ | Push-invalidation subscription to the server's catalog-change notification stream (VV-CLU-007) |
| VV-SDK-005.5 | ⬜ | Test: cache produces correct routing decisions matching the server-side catalog; stale-cache-triggers-refresh test |

### VV-SDK-006 — Exponential backoff + retry + jitter  ⬜
**Spec:** DISTRIBUTED_REPLICATION.md Part 5; LD-BACK
**Blocked by:** VV-SDK-002
**Needs:** none

| Task | Status | Detail |
|---|---|---|
| VV-SDK-006.1 | ⬜ | Generic `RetryPolicy` (max_retries=3, base_delay=10ms, jitter, max_delay) shared across every retry-needing call |
| VV-SDK-006.2 | ⬜ | Retryable-error classifier: `UNAVAILABLE`, `429`+`Retry-After`, stale-epoch → retry; validation errors → fail fast, never retried |
| VV-SDK-006.3 | ⬜ | Wire into Execute/Query calls plus catalog refresh |
| VV-SDK-006.4 | ⬜ | Test: simulate transient failures, verify retry count/backoff timing matches policy; verify non-retryable errors fail immediately |

### VV-SDK-007 — C# / .NET SDK (`VectorVolt.Client` NuGet)  ⬜
**Spec:** CORE_ARCHITECTURE.md §12 ("The original VectorVolt prototype reborn as SDK")
**Blocked by:** VV-SDK-001
**Needs:** none

| Task | Status | Detail |
|---|---|---|
| VV-SDK-007.1 | ⬜ | Scaffold `VectorVolt.Client` `.csproj` targeting net8/net9, `Grpc.Net.Client`-based generated stubs |
| VV-SDK-007.2 | ⬜ | Port ergonomic wrapper patterns from the original C# prototype (per project history: reborn as SDK, not rebuilt from scratch) |
| VV-SDK-007.3 | ⬜ | Async/await client API mirroring the Rust SDK surface (Execute/Query/Txn) |
| VV-SDK-007.4 | ⬜ | NuGet packaging metadata (icon, license, README, version aligned to proto version) |
| VV-SDK-007.5 | ⬜ | Integration test project against a local server |

### VV-SDK-008 — LINQ query provider  ⬜
**Spec:** CORE_ARCHITECTURE.md §12
**Blocked by:** VV-SDK-007
**Needs:** none

| Task | Status | Detail |
|---|---|---|
| VV-SDK-008.1 | ⬜ | `IQueryable<T>` provider translating LINQ expression trees → `QueryRequest` |
| VV-SDK-008.2 | ⬜ | Explicitly document the supported operator subset (`Where`, `Select`, `OrderBy`, `Take`); unsupported operators throw `NotSupportedException` immediately, not at execution time |
| VV-SDK-008.3 | ⬜ | Expression-visitor implementation + a unit test per supported operator |
| VV-SDK-008.4 | ⬜ | Integration test: a LINQ query executes and returns correctly typed results |

### VV-SDK-009 — Python SDK (+ pandas integration)  ⬜
**Spec:** CORE_ARCHITECTURE.md §12
**Blocked by:** VV-SDK-001
**Needs:** none

| Task | Status | Detail |
|---|---|---|
| VV-SDK-009.1 | ⬜ | Scaffold `vectorvolt-py` package (`grpcio`-generated stubs) |
| VV-SDK-009.2 | ⬜ | Pythonic client wrapper (context-manager-based connection) |
| VV-SDK-009.3 | ⬜ | `to_dataframe()`: query result → pandas `DataFrame` conversion |
| VV-SDK-009.4 | ⬜ | PyPI packaging (`pyproject.toml`, `twine` publish workflow) |
| VV-SDK-009.5 | ⬜ | Test: query round-trip + DataFrame shape/dtype assertions |

### VV-SDK-010 — Go SDK  ⬜
**Spec:** CORE_ARCHITECTURE.md §12
**Blocked by:** VV-SDK-001
**Needs:** none

| Task | Status | Detail |
|---|---|---|
| VV-SDK-010.1 | ⬜ | Scaffold `vectorvolt-go` module (`protoc-gen-go` + `protoc-gen-go-grpc` stubs) |
| VV-SDK-010.2 | ⬜ | Idiomatic Go client wrapper (`context.Context` propagation, wrapped errors) |
| VV-SDK-010.3 | ⬜ | Go module publishing (tagged releases, `pkg.go.dev` discoverability) |
| VV-SDK-010.4 | ⬜ | Test: round-trip integration test |

### VV-SDK-011 — Java / Kotlin SDK (reactive streams)  ⬜
**Spec:** CORE_ARCHITECTURE.md §12
**Blocked by:** VV-SDK-001
**Needs:** none

| Task | Status | Detail |
|---|---|---|
| VV-SDK-011.1 | ⬜ | Scaffold Maven/Gradle project, `protoc-gen-grpc-java` stubs |
| VV-SDK-011.2 | ⬜ | Reactive wrapper (Project Reactor or RxJava `Mono`/`Flux`) over the async stub |
| VV-SDK-011.3 | ⬜ | Kotlin coroutine extension functions |
| VV-SDK-011.4 | ⬜ | Maven Central publishing setup (GPG signing, Sonatype/Central Portal) |
| VV-SDK-011.5 | ⬜ | Test: reactive round-trip test |

### VV-SDK-012 — TypeScript / JS SDK  ⬜
**Spec:** CORE_ARCHITECTURE.md §12
**Blocked by:** VV-SDK-001
**Needs:** none

| Task | Status | Detail |
|---|---|---|
| VV-SDK-012.1 | ⬜ | Scaffold npm package, `@grpc/grpc-js` + `ts-proto` generated types |
| VV-SDK-012.2 | ⬜ | Promise-based client wrapper; note the browser-vs-Node transport split (grpc-web for browsers, native grpc for Node) |
| VV-SDK-012.3 | ⬜ | npm packaging (`package.json`, `tsconfig`, dual ESM/CJS build) |
| VV-SDK-012.4 | ⬜ | CI-gated `npm publish` workflow |
| VV-SDK-012.5 | ⬜ | Test: round-trip integration test in a Node environment |

### VV-SDK-013 — N-2 schema version compatibility  ⬜
**Spec:** LD-SDK-01
**Blocked by:** VV-SDK-001, VV-CTL-003
**Needs:** none

| Task | Status | Detail |
|---|---|---|
| VV-SDK-013.1 | ⬜ | Schema-version tagging on every command/query wire message (proto field) |
| VV-SDK-013.2 | ⬜ | Server-side compatibility layer: accept current + 2 prior schema versions, translate old→new internally |
| VV-SDK-013.3 | ⬜ | Migration handler registration API — explicit, per LD-SDK-01 (a table of version → migration function, never implicit/automatic) |
| VV-SDK-013.4 | ⬜ | Deprecation warning surfaced to the client when it's using an N-1/N-2 schema version |
| VV-SDK-013.5 | ⬜ | Test: a client pinned to schema version N-2 successfully round-trips against a server running version N |

---

*This document decomposes `VECTORVOLT_FEATURE_MATRIX.md`'s Layer 7–10 rows to implementable,
sub-day task units. It does not change any status, sprint assignment, or locked decision recorded
in the source documents — it is a traceability expansion, not a re-plan.*
