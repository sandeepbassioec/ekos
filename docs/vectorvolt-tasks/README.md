# VectorVolt — Task-Level Traceability Index

**2,127 tasks** across **413 features** across **19 layers**. Generated 2026-07-30 by auditing every
planning document against every line of code in `vectorvolt/crates/`.

Status: ✅ done · 🟡 partial · ⬜ pending · 🧊 env-gated (needs Linux/specific hardware) · 🔬 unscheduled research

| Layers | File | Features | Tasks |
|---|---|---|---|
| **0** Silicon/HAL · **12** Testing | [LAYER-00-HAL-AND-TESTING.md](LAYER-00-HAL-AND-TESTING.md) | 53 | **313** |
| **1** WAL · **2** Index · **3** Storage | [LAYER-01-03-WAL-INDEX-STORAGE.md](LAYER-01-03-WAL-INDEX-STORAGE.md) | 56 | **328** |
| **4** Transactions · **5** Query · **6** Engine/Partitioning | [LAYER-04-06-TXN-QUERY-ENGINE.md](LAYER-04-06-TXN-QUERY-ENGINE.md) | 64 | **382** |
| **7** Network · **8** Cluster · **9** Control Plane · **10** SDKs | [LAYER-07-10-NET-CLUSTER-CONTROL-SDK.md](LAYER-07-10-NET-CLUSTER-CONTROL-SDK.md) | 65 | **353** |
| **11** Observability · **13** Security · **14** Prefetch · **15** Projections · **16** Hardware · **17** Autonomy | [LAYER-11-17-OBS-SEC-PREFETCH-PROJ-HW-AUTONOMY.md](LAYER-11-17-OBS-SEC-PREFETCH-PROJ-HW-AUTONOMY.md) | 52 | **294** |
| **18** Deployment & Auto-Configuration | [LAYER-18-DEPLOYMENT-AUTOCONFIG.md](LAYER-18-DEPLOYMENT-AUTOCONFIG.md) | 39 | **248** |
| **19** Sinks & External Integration | [LAYER-19-SINKS-AND-INTEGRATION.md](LAYER-19-SINKS-AND-INTEGRATION.md) | 52 | **209** |

**Layer 18 added 2026-07-30**, closing a gap the original audit missed: runtime dispatch handles CPU
*instructions*, but nothing handled *system configuration* — huge-page reservation, core role maps,
arena sizing, WAL device selection. Those need root, or a reboot, or knowledge of what is mounted, so
they cannot be resolved at runtime. It is what makes "one codebase, self-configuring" real at install
time rather than aspirational.

Feature-level rollup and sprint plan: [../VECTORVOLT_FEATURE_MATRIX.md](../VECTORVOLT_FEATURE_MATRIX.md)

## ID scheme

```
VV-ENG-010          feature   — partition control by entity
VV-ENG-010.2        task      — declare partition key in DDL      (< 1 day)
VV-ENG-010.2.1      subtask   — reject non-deterministic key exprs
```

Grep any ID to find it: `rg "VV-ENG-010" docs/vectorvolt-tasks/`

---

## Where the finish line actually is

"Complete VectorVolt" is **Layers 0–16**. Layer 17 (ARP autonomous remediation, CIP capability
injection, Shadowbox, self-upgrading binary, in-process LLM) is the **EKOS autonomy layer built on
top of** VectorVolt, and the source docs — `RESEARCH_FRONTIER.md`, `HARDWARE_FRONTIER.md` — explicitly
disclaim any schedule for it. Excluding it is not scope-cutting; it is reading the docs correctly.

Of the 2,127 tasks, **50 are 🔬 unscheduled research** and **111 are 🧊 hardware-gated**. The
buildable core is roughly **1,500 tasks**, of which about **415 are done**.

---

## Cross-cutting findings

These came out of verifying docs against code. Each is a real discrepancy, not a style note.

### 1. Two competing point-lookup implementations — one is 400× slower

`vectorvolt-engine::get()` uses the O(1) `read_value()` path (200 ns p50).
`vectorvolt-query::QueryEngine::fetch_row()` still uses `read_column()`, which decodes the **entire
column** to return one cell — the exact cost profile we removed in the `feat/lazy-column-read` work.

Two divergent read paths where one is 400× slower than the other is a correctness-of-architecture
problem, not just a performance gap: whichever layer a future caller picks silently determines
whether they get 200 ns or 80 µs. **Fix in S2** — consolidate on one path.
Refs: VV-QRY-001, VV-STO-007.

### 2. Compaction is marked done but is not in the working tree

`VV-STO-008` / `VV-ENG-004` are ✅ in the feature matrix, but there is no `compact()` in the tree —
it exists only on the unmerged `feat/compaction` branch. **Three features are blocked on that merge**
(VV-STO-015 tiered compaction, VV-STO-016 cross-partition checksum, VV-WAL-013 WAL log compaction).
Both trackers mark it 🟡 with the branch named.

### 3. HAL is built but not wired to anything

`vectorvolt-hal` is a dependency of `vvctl` **only**. No SIMD kernel, prefetch hint, or cache
primitive is called from the storage, query, index, or engine hot paths. The 2.6× kernel speedups are
microbenchmarks; **no real query is faster yet**. Every ✅ kernel task is therefore paired with an
explicit ⬜ integration task. This is precisely S2's job.

### 4. `WalReader::verify()` reports a fabricated hash on chain break

`vectorvolt-wal/src/reader.rs:148` returns `GENESIS_HASH` as the `actual` value instead of the
frame's real `prev_hash` — the code comment admits it is a placeholder. The WAL's SHA3-256 chain
exists to *prove* tampering; the one moment it fires, it misreports what it found.

Second defect in the same block: the re-walk only detects `prev_hash` mismatches, but `scan()` also
stops on a *self*-hash mismatch. A frame with a corrupt own-hash but intact chain link yields
`broken_at = 0`, blaming the wrong frame.

### 5. `TxnManager.zone_map` is written every flush and never read

Real range pruning uses a separate per-column zone table in the storage layer. The txn-level
`ZoneMap` looks like a duplicate structure kept up to date for nobody. Either wire it in or delete
it — a maintained-but-unused index is a latent correctness trap.
Ref: VV-TXN-005.

### 6. Roaring bitmap set operations are documented but not implemented

`READ_PATH_ARCHITECTURE.md` describes SIMD `AND`/`OR`/`NOT` combination of secondary indexes for
multi-predicate queries. `SecondaryIndex` exposes no such methods. Multi-predicate queries therefore
cannot be served by index intersection today.
Ref: VV-IDX-003.

### 7. Two S1a claims corrected downward

- `VV-HAL-022` ARM `PRFM` prefetch: ✅ → 🟡. Compiles for aarch64, never executed. Needs a Graviton
  instance — **not** covered by the two x86 boxes available.
- `VV-HAL-023` prefetch-ahead: ✅ → 🟡. Primitive exists and is tested; no real scan loop calls it.

---

## Test environment requirements

Run [`scripts/vv-probe-linux.sh`](../../scripts/vv-probe-linux.sh) on each host first — read-only, no
root, changes nothing.

| Needs | Count | Features |
|---|---|---|
| Windows dev box (current) | — | All of L0 SIMD/cache/timer, L1–L6 logic, L7 protocol + TCP transport |
| **1 Linux host** | 1 | NUMA binding (`mbind`), 2 MB huge pages, `isolcpus`/`nohz_full`, `mlock`, io_uring, `O_DIRECT`, AF_XDP generic mode, PAM MFA, `perf`/eBPF profiling |
| **2 Linux hosts** ← *available* | 2 | **Master-slave replication, SWIM 2-node timing, replication lag, shard rebalancing, reader binary, failover basics** |
| 3+ hosts | 3–5 | Full Raft quorum fidelity, split-brain/partition scenarios, majority-quorum edge cases |
| ENA + SR-IOV instance | 1 | DPDK poll-mode driver |
| EFA / InfiniBand | 2 | RDMA WAL shipping |
| Graviton (ARM) | 1 | NEON kernels, ARM `PRFM` verification |
| Intel QAT card | 1 | Crypto/compression offload |
| Intel DSA | 1 | memcpy/CRC offload |
| Nvidia T4 ← *available* | 1 | In-process LLM embedding (L17 only — **irrelevant to the database core**) |

**Honest note on benchmarking:** consensus, failover, and gossip cannot be validated in one process —
a single-node "cluster test" proves nothing. Conversely, a deterministic single-process simulation
harness is a *better* tool than real hosts for reproducing partition scenarios, because it is
seedable and replayable. Both are needed; neither substitutes for the other. And nanosecond latency
figures measured on a burstable (`t2`/`t3`) instance are not trustworthy — CPU-credit throttling and
hypervisor steal time swamp the signal.
