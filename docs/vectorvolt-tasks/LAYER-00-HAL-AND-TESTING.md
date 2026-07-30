# Layer 0 — Silicon / HAL  ·  Layer 12 — Testing
Task-level traceability. Status: ✅ done · 🟡 partial · ⬜ pending · 🧊 env-gated (needs Linux/hardware) · 🔬 research

**Feature IDs** come from `docs/VECTORVOLT_FEATURE_MATRIX.md` (LAYER 0 and LAYER 12 tables).
**Code audited on disk** at `vectorvolt/crates/vectorvolt-hal/` (branch `feat/hal-simd`, currently
checked out) and `vectorvolt/crates/vvctl/src/commands/hal.rs`. CI audited at
`.github/workflows/vectorvolt-ci.yml`, `vectorvolt/.github/workflows/ci.yml`,
`vectorvolt/.github/workflows/release.yml`.

**Two statuses in this file upgrade past the top-line table in `VECTORVOLT_FEATURE_MATRIX.md`**
(that table's per-row marks predate the S1a sprint and were not edited row-by-row afterward —
the matrix's own Sprint Log section already documents the delta):
- **VV-HAL-022** (ARM `PRFM` prefetch): matrix row says ⬜, sprint log says implemented-but-unverified.
  Code exists and compiles for aarch64 (`src/prefetch.rs`); marked 🟡 here, not ✅, because no one has
  confirmed on real ARM hardware that the hint measurably helps rather than being a harmless no-op.
- **VV-HAL-023** (prefetch-distance tuned scan loop): matrix row says ⬜. The generic primitive
  (`stream_distance`, `prefetch_ahead`) already exists and is unit-tested, but it is not called from
  any real scan loop in `vectorvolt-storage` or `vectorvolt-query` yet — only from its own doctest.
  Marked 🟡.

**Cross-cutting finding, load-bearing for reading every ✅ below:** `vectorvolt-hal` is a dependency
of exactly one crate — `vvctl` (verified via `grep -l vectorvolt-hal crates/*/Cargo.toml`). None of
`vectorvolt-storage`, `vectorvolt-query`, `vectorvolt-index`, `vectorvolt-engine`, or
`vectorvolt-txn` depend on it. Every kernel marked ✅ below is real, tested, and measured — but
**nothing in the actual read/write hot path calls into this crate yet.** The SIMD/prefetch/cache
wins are proven in isolation; wiring them into production code paths is Sprint 2 work and is called
out as its own pending task under each feature that has a natural integration point.

---

## Rollup

| Layer | Features | Tasks total | ✅ | 🟡 | ⬜ | 🧊 |
|---|---|---|---|---|---|---|
| L0 HAL | 37 | 223 | 94 | 0 | 121 | 8 |
| L12 Testing | 16 | 90 | 32 | 0 | 58 | 0 |
| **Total** | **53** | **313** | **126** | **0** | **179** | **8** |

Feature-level rollup (37 HAL features, by their own overall mark): ✅ 16 · 🟡 3 (HAL-022, HAL-023,
HAL-032) · 🧊 2 (HAL-030, HAL-035) · ⬜ 16.
Testing (16 features): ✅ 5 (TST-001..005) · ⬜ 11.

---

## VV-HAL-001 — CpuTopology detection  ⬜
**Spec:** VECTORVOLT_CORE_ARCHITECTURE.md §3.1 (`CpuTopology` struct definition); CPU_CACHE_NUMA_ISOLATION.md §4.2 (NUMA detection)
**Blocked by:** nothing — foundational, everything else in this layer wants to consume it
**Needs:** Linux for `/proc/cpuinfo` + `/sys/devices/system/node/` + `/sys/.../cache/index*/`; Windows `GetLogicalProcessorInformationEx` FFI; macOS `sysctlbyname` fallback

No `CpuTopology`, `NumaNode`, or `/proc/cpuinfo` parsing exists anywhere in the workspace
(`grep -rl "CpuTopology\|NumaNode" crates/` — zero hits). `cache.rs` only has hardcoded *default*
sizes (`L1D_DEFAULT_BYTES = 32 * 1024`, etc.) used as sizing guesses before any real detection runs.

| Task | Status | Detail |
|---|---|---|
| VV-HAL-001.1 | ⬜ | Define `CpuTopology { sockets, physical_cores, logical_cores, numa_nodes, cache: CacheHierarchy, simd_features: SimdTier }` in a new `vectorvolt-hal::topology` module |
| VV-HAL-001.2 | ⬜ | Linux: parse `/proc/cpuinfo` for `physical id` / `core id` / `processor` → socket/core/logical mapping |
| VV-HAL-001.2.1 | ⬜ | Hyperthread sibling detection via `/sys/devices/system/cpu/cpuN/topology/thread_siblings_list` |
| VV-HAL-001.3 | ⬜ | Linux: parse `/sys/devices/system/cpu/cpu0/cache/index{0,1,2,3}/{level,size,shared_cpu_list}` for real L1/L2/L3 sizes (replaces the `*_DEFAULT_BYTES` guesses in `cache.rs`) |
| VV-HAL-001.4 | ⬜ | Windows: `GetLogicalProcessorInformationEx` FFI wrapper (`RelationProcessorCore`, `RelationCache`, `RelationNumaNode`) |
| VV-HAL-001.4.1 | ⬜ | Parse `SYSTEM_LOGICAL_PROCESSOR_INFORMATION_EX` cache descriptors for L1/L2/L3 |
| VV-HAL-001.5 | ⬜ | macOS: `sysctlbyname("hw.physicalcpu"/"hw.logicalcpu"/"hw.l1dcachesize"/"hw.l2cachesize"/"hw.l3cachesize")` fallback |
| VV-HAL-001.6 | ⬜ | Graceful degradation when topology is unreadable (containers, restricted `/proc`): fall back to `std::thread::available_parallelism()` + cache defaults, log a warning rather than panic |
| VV-HAL-001.7 | ⬜ | Fold `simd::tier()` (already exists, see VV-HAL-004) into `CpuTopology.simd_features` |
| VV-HAL-001.8 | ⬜ | `vvctl hal`: print full topology (sockets/cores/NUMA/cache sizes), not just core count and cache-line constant as today |
| VV-HAL-001.9 | ⬜ | Test: parse mocked `/proc/cpuinfo` fixtures (HT-enabled dual-socket, HT-disabled single-socket) |
| VV-HAL-001.10 | ⬜ | Test: Windows FFI struct layout size assertions match the OS ABI |
| VV-HAL-001.11 | ⬜ | Test: detection never panics when `/proc` is unreadable (permission denied / sandboxed container) |
| VV-HAL-001.12 | ⬜ | Test: single-socket non-NUMA machine reports exactly one `NumaNode` covering all cores |

---

## VV-HAL-002 — Cache-line alignment primitives `#[repr(align(64))]`  ✅
**Spec:** CPU_CACHE_NUMA_ISOLATION.md §2.2; VECTORVOLT_CORE_ARCHITECTURE.md §3.2
**Blocked by:** nothing
**Needs:** nothing — portable

Evidence: `vectorvolt-hal/src/cache.rs`.

| Task | Status | Detail |
|---|---|---|
| VV-HAL-002.1 | ✅ | `CacheAligned<T>` — `#[repr(C, align(64))]`, no tail padding (cache.rs:58-87) |
| VV-HAL-002.2 | ✅ | `CachePadded<T>` — size rounds up to 64 B so neighbours never share a line (cache.rs:99-140) |
| VV-HAL-002.3 | ✅ | `CACHE_LINE: usize = 64` constant + `lines_for(bytes)` helper (cache.rs:41, 144-146) |
| VV-HAL-002.4 | ✅ | `is_line_aligned` / `fits_one_line` address predicates (cache.rs:149-159) |
| VV-HAL-002.5 | ✅ | Compile-time size/alignment assertion tests — `cache_padded_is_line_sized_and_aligned`, `cache_aligned_is_aligned` |
| VV-HAL-002.6 | ✅ | `Deref`/`DerefMut` ergonomics so wrapped atomics are used transparently — test `deref_gives_access_to_inner` |
| VV-HAL-002.7 | ✅ | `lines_for` boundary test (`lines_for_rounds_up`: 0→0, 1→1, 64→1, 65→2, 128→2) |

---

## VV-HAL-003 — False-sharing avoidance (one-directional ownership)  ✅
**Spec:** CPU_CACHE_NUMA_ISOLATION.md §2.1–2.3; §5.4 (writer/reader ownership pattern)
**Blocked by:** nothing for the primitive; retrofitting existing hot-path structs is blocked by VV-TXN-008 (ring buffer doesn't exist yet)
**Needs:** nothing

`grep -rn "CachePadded\|CacheAligned" crates/vectorvolt-wal` returns nothing — the primitive from
VV-HAL-002 is built and proven, but has not yet been applied to any real struct outside its own
tests. `vectorvolt-wal/src/metrics.rs::Inner` is 8 plain `AtomicU64` fields with no padding at all,
so `fsync_count`, `frames_committed`, etc. can currently false-share across cache lines today.

| Task | Status | Detail |
|---|---|---|
| VV-HAL-003.1 | ✅ | `CachePadded<T>` guarantees one value per cache line by construction (reuses VV-HAL-002) |
| VV-HAL-003.2 | ✅ | Regression test: 8 consecutive `CachePadded<AtomicU64>` never share a line — `padded_neighbours_do_not_share_a_line` |
| VV-HAL-003.3 | ✅ | One-directional ownership rule documented (writer holds `&mut`, readers hold `&`) — cache.rs module doc + CPU_CACHE_NUMA_ISOLATION.md §2.3 |
| VV-HAL-003.4 | ⬜ | Apply the pattern to the future ring-buffer sequence counter (blocked by VV-TXN-008 — no ring buffer exists yet) |
| VV-HAL-003.5 | ⬜ | Retrofit `vectorvolt-wal::BatcherMetrics::Inner` fields into `CachePadded` — currently 8 unpadded `AtomicU64`s that can false-share under concurrent reader/writer access |

---

## VV-HAL-004 — Runtime SIMD feature detection (`cpuid`)  ✅
**Spec:** CPU_CACHE_NUMA_ISOLATION.md §6.1
**Blocked by:** nothing
**Needs:** an x86-64 CPU for the AVX2 path to mean anything; the aarch64 branch is a hardcoded `Scalar` today (see VV-HAL-008)

Evidence: `vectorvolt-hal/src/simd/mod.rs::detect()`, `tier()`, `has_hw_crc32c()`.

| Task | Status | Detail |
|---|---|---|
| VV-HAL-004.1 | ✅ | `detect()` probes AVX2 via `is_x86_feature_detected!("avx2")` (mod.rs:92-106) |
| VV-HAL-004.2 | ✅ | `has_hw_crc32c()` probes SSE4.2 via `is_x86_feature_detected!("sse4.2")` (mod.rs:114-127) |
| VV-HAL-004.3 | ✅ | Result cached in `OnceLock` so `cpuid` (serializing, ~hundreds of cycles) runs once per process (mod.rs:85-86) |
| VV-HAL-004.4 | ✅ | Stability test: `tier()` called twice agrees with itself and with a fresh `detect()` — `tier_is_cached_and_stable` |
| VV-HAL-004.5 | ⬜ | ARM feature detection (`is_aarch64_feature_detected!("neon")`/`"sve"`) — `detect()` unconditionally returns `Scalar` on non-x86_64 today (mod.rs:102-105) |
| VV-HAL-004.6 | ⬜ | AVX-512 probe (`is_x86_feature_detected!("avx512f")`) — deliberately not probed; `SimdTier::Avx512` is a reserved enum variant `detect()` never returns (mod.rs:95, doc comment on `SimdTier::Avx512`) |

---

## VV-HAL-005 — SIMD dispatch table (compile all, select at startup)  ✅
**Spec:** CPU_CACHE_NUMA_ISOLATION.md §6.1 dispatch example
**Blocked by:** VV-HAL-004
**Needs:** nothing further

Evidence: `simd/mod.rs` — six public dispatcher functions, each branching on the cached `tier()`.

| Task | Status | Detail |
|---|---|---|
| VV-HAL-005.1 | ✅ | `SimdTier` enum: `Scalar` / `Avx2` / `Avx512` / `Neon` with `lanes_u64()` and `name()` (mod.rs:42-83) |
| VV-HAL-005.2 | ✅ | Six dispatch functions (`filter_range`, `count_range`, `min_max`, `sum`, `find_first_eq`, `crc32c`) each forwarding to `avx2::` or `scalar::` behind an `unsafe` block justified by the caller's `tier()` check (mod.rs:131-211) |
| VV-HAL-005.3 | ✅ | Basic dispatch-matches-oracle test — `dispatch_matches_oracle_on_a_basic_case` |
| VV-HAL-005.4 | ✅ | Lane-count / name metadata tests — `lane_counts_are_sane`, `names_are_stable_for_logging` |
| VV-HAL-005.5 | ⬜ | Precomputed fn-pointer table (vs. re-branching on `tier()` at every call site) — current design is correct but re-checks the enum on every call; a `OnceLock<KernelTable>` of function pointers would shave the branch, not yet built |

---

## VV-HAL-006 — AVX2 baseline kernels (256-bit, 4×u64/cycle)  ✅
**Spec:** CPU_CACHE_NUMA_ISOLATION.md §6.1, §6.3 (SIMD applications table)
**Blocked by:** VV-HAL-004, VV-HAL-005
**Needs:** an AVX2-capable CPU (Haswell 2013+ / Ryzen 2017+) to execute; compiles unconditionally on x86-64

Evidence: `vectorvolt-hal/src/simd/avx2.rs`. Measured via `vvctl hal`: 1.24×–2.62× depending on
kernel and working-set tier (sprint log table, `VECTORVOLT_FEATURE_MATRIX.md` S1a section).

| Task | Status | Detail |
|---|---|---|
| VV-HAL-006.1 | ✅ | XOR-`i64::MIN` range-shift trick so signed `_mm256_cmpgt_epi64` gives correct unsigned ordering (avx2.rs:8-21, 41-42) |
| VV-HAL-006.2 | ✅ | `filter_range` — bitmask-to-indices kernel via `_mm256_movemask_pd` (avx2.rs:69-109) |
| VV-HAL-006.3 | ✅ | `count_range` kernel (avx2.rs:116-146) |
| VV-HAL-006.4 | ✅ | `min_max` — 4-way independent-accumulator kernel; fixed a measured 0.76× regression from a naive single-accumulator version (avx2.rs:152-236, `UNROLL = 4`) |
| VV-HAL-006.5 | ✅ | `sum` — 4-way independent-accumulator kernel, wrapping semantics (avx2.rs:243-286) |
| VV-HAL-006.6 | ✅ | `find_first_eq` kernel, returns lowest matching lane via `trailing_zeros` (avx2.rs:293-319) |
| VV-HAL-006.7 | ✅ | `crc32c` hardware kernel, separate `sse4.2` feature gate from the `u64` kernels' `avx2` gate (avx2.rs:329-349) |
| VV-HAL-006.8 | ✅ | Scalar tail-loop for `len % 4 != 0` remainder in every kernel — verified for every length 0–300 in `tests/simd_equivalence.rs` |
| VV-HAL-006.9 | ✅ | `vvctl hal` bench harness measuring scalar-vs-AVX2 speedup per kernel (`crates/vvctl/src/commands/hal.rs`) |

---

## VV-HAL-007 — AVX-512 opportunistic kernels (512-bit, 8×u64/cycle)  ⬜
**Spec:** CPU_CACHE_NUMA_ISOLATION.md §6.1
**Blocked by:** VV-HAL-004.6 (AVX-512 probe)
**Needs:** Ice Lake+ / Sapphire Rapids / Zen 4 CPU to validate; deliberately deferred per S1a sprint log ("intrinsic signatures still churning across toolchains; absent on most consumer CPUs; downclocks some Xeons")

| Task | Status | Detail |
|---|---|---|
| VV-HAL-007.1 | ⬜ | `avx512f` runtime detection, `SimdTier::Avx512` actually returned from `detect()` |
| VV-HAL-007.2 | ⬜ | 8-lane `min_max` / `count_range` / `filter_range` AVX-512 kernels |
| VV-HAL-007.3 | ⬜ | Native `_mm512_cmpgt_epu64` unsigned compare — no XOR-flip trick needed (AVX-512 has real unsigned compare, unlike AVX2) |
| VV-HAL-007.4 | ⬜ | Downclock-aware dispatch: some Xeons drop clock speed on sustained 512-bit use, worth avoiding for short bursts |
| VV-HAL-007.5 | ⬜ | Equivalence tests against the scalar oracle, same rigor as `tests/simd_equivalence.rs` gives the AVX2 path |

---

## VV-HAL-008 — ARM NEON / SVE kernels  ⬜
**Spec:** CPU_CACHE_NUMA_ISOLATION.md §6.2
**Blocked by:** nothing structurally
**Needs:** real aarch64 hardware (Graviton, or the GitHub `macos-latest` CI runner, which is Apple Silicon) — "shipping untested vector intrinsics is worse than the scalar fallback, which is correct everywhere" (S1a sprint log rationale)

`SimdTier::Neon` is a reserved enum variant; `detect()` never returns it — aarch64 reports `Scalar`.

| Task | Status | Detail |
|---|---|---|
| VV-HAL-008.1 | ⬜ | NEON runtime detection (`is_aarch64_feature_detected!("neon")`) |
| VV-HAL-008.2 | ⬜ | NEON `filter_range` / `count_range` (128-bit, 2×u64/instr via `vceqq`/`vcgtq`) |
| VV-HAL-008.3 | ⬜ | NEON `min_max` / `sum` |
| VV-HAL-008.4 | ⬜ | NEON `find_first_eq` |
| VV-HAL-008.5 | ⬜ | SVE variable-width kernels (Graviton3+, 128–2048 bit) |
| VV-HAL-008.6 | 🧊 | Equivalence tests executed on real aarch64 hardware — CI's `macos-latest` job already runs `cargo test --workspace` on Apple Silicon today, so once kernels exist this can validate there without new infra |

---

## VV-HAL-009 — Scalar fallback kernels (portable)  ✅
**Spec:** module doc, `simd/scalar.rs`; correctness-oracle role described in CPU_CACHE_NUMA_ISOLATION.md §6
**Blocked by:** nothing
**Needs:** nothing — this is the fallback that runs everywhere

Evidence: `vectorvolt-hal/src/simd/scalar.rs`.

| Task | Status | Detail |
|---|---|---|
| VV-HAL-009.1 | ✅ | `filter_range` scalar (scalar.rs:14-20) |
| VV-HAL-009.2 | ✅ | `count_range` scalar (scalar.rs:24-26) |
| VV-HAL-009.3 | ✅ | `min_max` scalar (scalar.rs:30-43) |
| VV-HAL-009.4 | ✅ | `sum` scalar, wrapping (scalar.rs:47-49) |
| VV-HAL-009.5 | ✅ | `find_first_eq` scalar (scalar.rs:53-55) |
| VV-HAL-009.6 | ✅ | `popcount` scalar via `count_ones()` (scalar.rs:63-65) |
| VV-HAL-009.7 | ✅ | `crc32c` scalar bitwise reference, polynomial `0x82F63B78` (scalar.rs:75-86) |
| VV-HAL-009.8 | ✅ | Known-answer test against the CRC-32C spec's own conformance check value `0xE3069283` — `crc32c_matches_known_vectors` |

---

## VV-HAL-010 — SIMD kernel: range filter (`lo <= v <= hi` → bitmask)  ✅
**Spec:** CPU_CACHE_NUMA_ISOLATION.md §6.3 ("Columnar range scan, ~8×")
**Blocked by:** VV-HAL-006
**Needs:** nothing further for the kernel; storage/query integration needs VV-STO-013/VV-QRY-006 (S2)

| Task | Status | Detail |
|---|---|---|
| VV-HAL-010.1 | ✅ | AVX2 `filter_range` via bitmask extraction (`avx2.rs::filter_range`) |
| VV-HAL-010.2 | ✅ | Scalar oracle (`scalar::filter_range`) |
| VV-HAL-010.3 | ✅ | Equivalence across every length 0–300 — `tests/simd_equivalence.rs::every_length_up_to_300_agrees_on_{sequential,random}_data` |
| VV-HAL-010.4 | ✅ | Adversarial `i64::MAX`-boundary tests — `adversarial_values_around_the_signed_boundary_agree` |
| VV-HAL-010.5 | ✅ | Append-not-clear contract test (callers reuse a scratch `Vec` across partitions) — `filter_range_appends_without_clearing` |
| VV-HAL-010.6 | ✅ | Measured speedup 1.26×–1.38× via `vvctl hal` — capped low because the kernel still does a scalar `Vec::push` per match (sprint log finding #3) |
| VV-HAL-010.7 | ⬜ | Late-materialization redesign: emit a bitmask directly instead of pushing indices one at a time, to remove the `Vec::push` cap (tracked jointly with VV-QRY-009) |

---

## VV-HAL-011 — SIMD kernel: equality scan / find-first  ✅
**Spec:** CPU_CACHE_NUMA_ISOLATION.md §6.3
**Blocked by:** VV-HAL-006
**Needs:** nothing further

| Task | Status | Detail |
|---|---|---|
| VV-HAL-011.1 | ✅ | AVX2 `find_first_eq` via `_mm256_cmpeq_epi64` + `movemask` (avx2.rs:293-319) |
| VV-HAL-011.2 | ✅ | Scalar oracle (`scalar::find_first_eq`) |
| VV-HAL-011.3 | ✅ | "First match, not any match" contract test — `find_first_eq_finds_first_not_any`, `single_point_range_matches_exact_duplicates` |
| VV-HAL-011.4 | ✅ | Equivalence across every length 0–300 |
| VV-HAL-011.5 | ✅ | Measured speedup 1.75×–2.62× via `vvctl hal` |

---

## VV-HAL-012 — SIMD kernel: min/max (zone map build)  ✅
**Spec:** CPU_CACHE_NUMA_ISOLATION.md §6.3
**Blocked by:** VV-HAL-006
**Needs:** wiring into `vectorvolt-storage`'s zone-map builder needs nothing hardware-special, just the integration work (S2)

| Task | Status | Detail |
|---|---|---|
| VV-HAL-012.1 | ✅ | Single-accumulator first attempt, superseded (historical — see finding below) |
| VV-HAL-012.2 | ✅ | 4-way unrolled independent-accumulator fix: measured 0.76× (slower than scalar) → 1.46× after the fix. Lesson recorded in sprint log: "hand-written SIMD must beat `-O3` LLVM, not beat naive scalar" |
| VV-HAL-012.3 | ✅ | Sub-`BLOCK`-size fallback delegates to the scalar kernel when `n < 16` (avx2.rs:157-158) |
| VV-HAL-012.4 | ✅ | Horizontal 4-lane reduction + range-unshift back to unsigned domain (avx2.rs:194-232) |
| VV-HAL-012.5 | ✅ | Equivalence + adversarial-boundary tests (shared harness with all kernels) |
| VV-HAL-012.6 | ⬜ | Wire into `vectorvolt-storage`'s zone-map construction (VV-STO-006) — not called from storage today; only exercised via `vvctl hal` and the test suite |

---

## VV-HAL-013 — SIMD kernel: popcount / bitmap iterate  ⬜
**Spec:** CPU_CACHE_NUMA_ISOLATION.md §6.3 ("Roaring Bitmap iterate, ~4×")
**Blocked by:** VV-HAL-006 (for the kernel), VV-IDX-003 (Roaring bitmap index, already done) for wiring
**Needs:** nothing hardware-special

`scalar::popcount` exists and uses `u64::count_ones()` (already a single hardware `POPCNT` on any
modern CPU), but there is deliberately **no vector variant** — the module doc explains "SIMD popcount
only wins on bitmaps far larger than our index buckets," a claim that has not actually been
benchmarked at Roaring-bitmap scale.

| Task | Status | Detail |
|---|---|---|
| VV-HAL-013.1 | ⬜ | AVX2 vectorized popcount (`vpshufb` nibble-lookup, since AVX2 lacks `vpopcntq`) |
| VV-HAL-013.2 | ⬜ | Benchmark at actual Roaring-bitmap scale to find the real crossover point instead of assuming one |
| VV-HAL-013.3 | ⬜ | SIMD bitmap-iterate kernel (extract set-bit *positions*, not just count them — needed for VV-QRY-008) |
| VV-HAL-013.4 | ⬜ | Wire into `vectorvolt-index`'s Roaring bitmap secondary-index scan (VV-IDX-003) |
| VV-HAL-013.5 | ⬜ | Equivalence tests vs. the scalar `popcount` oracle |

---

## VV-HAL-014 — SIMD kernel: sum / aggregate  ✅
**Spec:** CPU_CACHE_NUMA_ISOLATION.md §6.3
**Blocked by:** VV-HAL-006
**Needs:** nothing further for the kernel; aggregate-pushdown extension needs VV-QRY-014

| Task | Status | Detail |
|---|---|---|
| VV-HAL-014.1 | ✅ | AVX2 4-way unrolled `sum` accumulator (avx2.rs:243-286) |
| VV-HAL-014.2 | ✅ | Wrapping-overflow semantics matching the scalar kernel exactly — `sum_wraps_identically_to_scalar` |
| VV-HAL-014.3 | ✅ | Scalar oracle (`scalar::sum`) |
| VV-HAL-014.4 | ✅ | Equivalence across every length 0–300 |
| VV-HAL-014.5 | ✅ | Measured speedup 1.24× — lowest of the vector kernels, because `sum` is memory-bandwidth-bound, not compute-bound (sprint log finding #2) |
| VV-HAL-014.6 | ⬜ | Extend to combined single-pass COUNT/AVG/MIN/MAX (currently only a standalone `sum`) for VV-QRY-014 aggregate pushdown |
| VV-HAL-014.7 | ⬜ | Wire into `vectorvolt-query`'s aggregate execution path |

---

## VV-HAL-015 — SIMD kernel: dictionary decode (`vpshufb`)  ⬜
**Spec:** CPU_CACHE_NUMA_ISOLATION.md §6.3 ("Dictionary decode, ~4×")
**Blocked by:** VV-HAL-006 (kernel), VV-STO-003 (dictionary encoding — already ✅ in `vectorvolt-storage`) for wiring
**Needs:** nothing hardware-special

| Task | Status | Detail |
|---|---|---|
| VV-HAL-015.1 | ⬜ | `vpshufb`-based dictionary code → value lookup-table kernel |
| VV-HAL-015.2 | ⬜ | Multi-shuffle passes for dictionaries with >16 entries (a single `vpshufb` only addresses a 16-byte table) |
| VV-HAL-015.3 | ⬜ | Wire into `vectorvolt-storage`'s dictionary-encoded column reads |
| VV-HAL-015.4 | ⬜ | Scalar oracle + equivalence tests |

---

## VV-HAL-016 — SIMD kernel: cosine similarity FMA (vector search)  ⬜
**Spec:** CPU_CACHE_NUMA_ISOLATION.md §6.3; VECTORVOLT_CORE_ARCHITECTURE.md §7.4 (Vector/Embedding Engine)
**Blocked by:** VV-QRY-020 (`VECTOR(n)` column type doesn't exist yet)
**Needs:** nothing hardware-special beyond AVX2/AVX-512 FMA support

| Task | Status | Detail |
|---|---|---|
| VV-HAL-016.1 | ⬜ | AVX2 FMA dot-product kernel (`_mm256_fmadd_pd`) |
| VV-HAL-016.2 | ⬜ | Vector-norm kernel (exact `sqrt` or fast-inverse-sqrt approximation) |
| VV-HAL-016.3 | ⬜ | AVX-512 16-wide FMA variant (depends on VV-HAL-007) |
| VV-HAL-016.4 | ⬜ | Requires `VECTOR(n)` column type to exist first — nothing to accelerate without it |
| VV-HAL-016.5 | ⬜ | Scalar oracle + equivalence tests with float tolerance (not bit-exact, unlike the integer kernels) |

---

## VV-HAL-017 — Hardware CRC-32C (`_mm_crc32_u64`)  ✅
**Spec:** CPU_CACHE_NUMA_ISOLATION.md §6.3 ("CRC-32C checksum, ~20×")
**Blocked by:** nothing
**Needs:** SSE4.2 (universal on any CPU since 2008)

Evidence: `avx2.rs::crc32c` (329-349), `scalar.rs::crc32c` (75-86). Measured 44.6× vs. the bitwise
scalar reference (S1a sprint log) — faster than the spec's own ~20× estimate because the scalar
baseline here is bitwise, not `crc32fast`.

| Task | Status | Detail |
|---|---|---|
| VV-HAL-017.1 | ✅ | SSE4.2 `_mm_crc32_u64` (8 bytes/instr) + `_mm_crc32_u8` tail kernel |
| VV-HAL-017.2 | ✅ | SSE4.2 runtime detection, cached (`has_hw_crc32c()`) |
| VV-HAL-017.3 | ✅ | Scalar bitwise reference, CRC-32C polynomial `0x82F63B78` (not ISO-HDLC) |
| VV-HAL-017.4 | ✅ | Known-answer test against the CRC-32C spec's conformance check value |
| VV-HAL-017.5 | ✅ | Equivalence across every length 0–300 — `crc32c_agrees_across_paths_on_all_lengths` |
| VV-HAL-017.6 | ✅ | Documented non-interchangeability with the WAL's existing CRC-32/ISO-HDLC — different polynomial, different values for the same input (sprint log finding #5, module docs) |
| VV-HAL-017.7 | ⬜ | Migrate the WAL frame checksum to CRC-32C — a **breaking format change**, needs a version-bump/migration plan before it can happen; deliberately not attempted yet |

---

## VV-HAL-018 — L1 prefetch (`_mm_prefetch` `_MM_HINT_T0`)  ✅
**Spec:** CPU_CACHE_NUMA_ISOLATION.md §3.4
**Blocked by:** nothing
**Needs:** nothing — prefetch is a hint on every x86-64/aarch64 CPU

Evidence: `vectorvolt-hal/src/prefetch.rs`.

| Task | Status | Detail |
|---|---|---|
| VV-HAL-018.1 | ✅ | x86-64 `_mm_prefetch::<_MM_HINT_T0>` (prefetch.rs:73-90) |
| VV-HAL-018.2 | ✅ | aarch64 `prfm pldl1keep` inline asm (prefetch.rs:98-101) — see VV-HAL-022 for the hardware-verification caveat |
| VV-HAL-018.3 | ✅ | Portable no-op fallback on other architectures (prefetch.rs:118-122) |
| VV-HAL-018.4 | ✅ | Safe function signature (no `unsafe`) with a documented safety argument: a prefetch is architecturally a hint, cannot fault on any address |
| VV-HAL-018.5 | ✅ | Null/dangling-pointer no-op test — `prefetch_tolerates_null_and_dangling` |
| VV-HAL-018.6 | ⬜ | Wire into an actual pointer-chasing hot path — row-descriptor → partition-offset lookup (VV-QRY-001) — not called anywhere outside this crate's own tests today |

---

## VV-HAL-019 — L2 prefetch (`_MM_HINT_T1`)  ✅
**Spec:** CPU_CACHE_NUMA_ISOLATION.md §3.4
**Blocked by:** nothing
**Needs:** nothing

| Task | Status | Detail |
|---|---|---|
| VV-HAL-019.1 | ✅ | x86-64 `_MM_HINT_T1` (prefetch.rs:85) |
| VV-HAL-019.2 | ✅ | aarch64 `prfm pldl2keep` (prefetch.rs:102-105) |
| VV-HAL-019.3 | ✅ | Covered by `prefetch_all_localities_is_a_noop_on_valid_memory` |
| VV-HAL-019.4 | ⬜ | Wire into Delta Log entry / Zone Map entry access (CPU_CACHE_NUMA_ISOLATION.md §3.2, "what lives in L2") — no caller yet |

---

## VV-HAL-020 — L3 prefetch (`_MM_HINT_T2`)  ✅
**Spec:** CPU_CACHE_NUMA_ISOLATION.md §3.4
**Blocked by:** nothing
**Needs:** nothing

| Task | Status | Detail |
|---|---|---|
| VV-HAL-020.1 | ✅ | x86-64 `_MM_HINT_T2` (prefetch.rs:86) |
| VV-HAL-020.2 | ✅ | aarch64 `prfm pldl3keep` (prefetch.rs:106-109) |
| VV-HAL-020.3 | ✅ | Covered by `prefetch_all_localities_is_a_noop_on_valid_memory` |
| VV-HAL-020.4 | ⬜ | Wire into Cuckoo Hash / Cuckoo Filter bulk staging (CPU_CACHE_NUMA_ISOLATION.md §3.3, Tier-1 structures) — no caller yet |

---

## VV-HAL-021 — Non-temporal / RAM streaming prefetch (`_MM_HINT_NTA`)  ✅
**Spec:** CPU_CACHE_NUMA_ISOLATION.md §3.4
**Blocked by:** nothing
**Needs:** nothing

| Task | Status | Detail |
|---|---|---|
| VV-HAL-021.1 | ✅ | x86-64 `_MM_HINT_NTA` (prefetch.rs:87) |
| VV-HAL-021.2 | ✅ | aarch64 `prfm pldl1strm` (prefetch.rs:110-113) |
| VV-HAL-021.3 | ✅ | `stream_distance<T>()` — sizes lookahead to ~1 DRAM latency (8 cache lines), expressed in elements (prefetch.rs:149-161) — test `stream_distance_scales_with_element_size` |
| VV-HAL-021.4 | ✅ | `prefetch_ahead()` — bounds-checked slice-index-offset helper (prefetch.rs:176-183) — test `prefetch_ahead_never_reads_out_of_bounds` |
| VV-HAL-021.5 | ⬜ | Wire into a full-column sequential scan in `vectorvolt-storage` (VV-STO-014) so streaming scans don't evict the Tier-1 index from L3 — no caller yet outside the module's own doctest |

---

## VV-HAL-022 — ARM `PRFM` prefetch equivalents  🟡
**Spec:** CPU_CACHE_NUMA_ISOLATION.md §3.4 (implies ARM parity, no dedicated ARM section)
**Blocked by:** nothing to write the code; verification blocked by lack of dedicated ARM hardware in this session
**Needs:** real aarch64 hardware to confirm the hints help rather than being inert — GitHub's `macos-latest` CI runner is Apple Silicon and already executes this code path on every `cargo test --workspace` run, but no one has measured whether it improves latency there

| Task | Status | Detail |
|---|---|---|
| VV-HAL-022.1 | ✅ | `prfm pldl1keep/pldl2keep/pldl3keep/pldl1strm` for reads, all four localities (prefetch.rs:98-113) |
| VV-HAL-022.2 | ✅ | `prfm pstl1keep` for write-intent prefetch (prefetch.rs:133-141) |
| VV-HAL-022.3 | ⬜ | Confirm on real ARM hardware that the hints measurably help — S1a sprint log records this explicitly as "implemented (compiles for aarch64; unverified on hardware)" |
| VV-HAL-022.4 | ⬜ | Tune PRFM lookahead depth for ARM microarchitectures — `stream_distance()`'s 8-line default was chosen for x86 DRAM latency and has not been re-validated for ARM |

---

## VV-HAL-023 — Prefetch-distance tuned scan loop  🟡
**Spec:** CPU_CACHE_NUMA_ISOLATION.md §3.4 (prefetch-ahead-of-consumption example)
**Blocked by:** nothing for the primitive (done); wiring blocked by nothing technical, just not done (S2 scope)
**Needs:** nothing hardware-special

The generic building block exists and is tested; it is not yet used by any real scan loop.

| Task | Status | Detail |
|---|---|---|
| VV-HAL-023.1 | ✅ | `stream_distance<T>()` sizing (8-line lookahead) — same code as VV-HAL-021.3 |
| VV-HAL-023.2 | ✅ | `prefetch_ahead()` bounds-checked lookahead-in-loop helper — same code as VV-HAL-021.4 |
| VV-HAL-023.3 | ✅ | Doctest demonstrating the scan-loop usage pattern (prefetch.rs module doc, lines 163-175) |
| VV-HAL-023.4 | ⬜ | Wire into `vectorvolt-storage`'s `PartitionReader` sequential column read |
| VV-HAL-023.5 | ⬜ | Wire into `vectorvolt-query`'s `range_scan` |
| VV-HAL-023.6 | ⬜ | Empirically tune the lookahead distance per kernel — the 8-line default is a documented estimate ("roughly one DRAM latency"), never measured against real scan throughput |
| VV-HAL-023.7 | ⬜ | Benchmark: prefetched vs. unprefetched scan latency on cold (DRAM-resident, past-L3) data |

---

## VV-HAL-024 — CPU core pinning (`sched_setaffinity`/`SetThreadAffinityMask`)  ⬜
**Spec:** CPU_CACHE_NUMA_ISOLATION.md §5.3; VECTORVOLT_CORE_ARCHITECTURE.md §4.2
**Blocked by:** soft-blocked by VV-HAL-001 (topology needed for safe core-id bounds checking; raw pinning by index doesn't strictly require it)
**Needs:** Linux for verified `sched_setaffinity`/readback; Windows for `SetThreadAffinityMask`

`grep -rl "sched_setaffinity\|SetThreadAffinityMask\|core_affinity" crates/` returns nothing — zero
pinning code exists anywhere in the workspace today.

| Task | Status | Detail |
|---|---|---|
| VV-HAL-024.1 | ⬜ | Linux: `libc::sched_setaffinity` wrapper, per the exact code in CPU_CACHE_NUMA_ISOLATION.md §5.3 |
| VV-HAL-024.2 | ⬜ | Windows: `SetThreadAffinityMask` wrapper |
| VV-HAL-024.3 | ⬜ | macOS: `thread_policy_set(THREAD_AFFINITY_POLICY)` — best-effort only; macOS has no strict pinning API |
| VV-HAL-024.4 | ⬜ | `pin_current_thread_to_core(core_id)` cross-platform façade |
| VV-HAL-024.5 | ⬜ | `pin_thread_to_cores(&[usize])` core-group variant (VECTORVOLT_CORE_ARCHITECTURE.md §4.2) |
| VV-HAL-024.6 | ⬜ | Fail loudly if requested `core_id` exceeds detected topology bounds |
| VV-HAL-024.7 | 🧊 | Integration test: spawn a thread, pin it, verify via `sched_getaffinity` readback — Linux only |
| VV-HAL-024.8 | ⬜ | Windows verification test via `GetThreadAffinityMask` diff-based probe |

---

## VV-HAL-025 — Core role map (OS/net/writer/reader/LLM/bg/admin groups)  ⬜
**Spec:** CPU_CACHE_NUMA_ISOLATION.md §5.1 (full 48-core worked example)
**Blocked by:** VV-HAL-001, VV-HAL-024, VV-HAL-028
**Needs:** nothing hardware-special beyond its dependencies

| Task | Status | Detail |
|---|---|---|
| VV-HAL-025.1 | ⬜ | `CoreRole` enum (`Os`, `Network`, `Writer`, `Reader`, `Llm`, `Background`, `Fingerprint`, `Admin`, `Overflow`) |
| VV-HAL-025.2 | ⬜ | `CoreRoleMap` builder deriving ranges from detected `CpuTopology` |
| VV-HAL-025.3 | ⬜ | Per-NUMA-node role assignment algorithm mirroring the 24-cores-per-socket worked example |
| VV-HAL-025.4 | ⬜ | Config override so an operator can pin specific roles to specific cores (needs VV-ENG-017 config system) |
| VV-HAL-025.5 | ⬜ | `vvctl hal --topology` command printing the resolved role map |
| VV-HAL-025.6 | ⬜ | Test: role map never assigns Core 0 (OS/IRQ) to any VectorVolt role |
| VV-HAL-025.7 | ⬜ | Test: role map partitions every logical core into exactly one role — no gaps, no overlaps |

---

## VV-HAL-026 — Writer/reader core isolation invariant  ⬜
**Spec:** CPU_CACHE_NUMA_ISOLATION.md §5.4
**Blocked by:** VV-HAL-024, VV-HAL-025, VV-TXN-009 (one-writer-per-shard), VV-TXN-010/011 (MVCC — the invariant's safety argument depends on snapshot isolation existing)
**Needs:** nothing hardware-special beyond its dependencies

| Task | Status | Detail |
|---|---|---|
| VV-HAL-026.1 | ⬜ | Assign each shard's single writer thread a dedicated core from the Writer role group |
| VV-HAL-026.2 | ⬜ | Runtime assertion: no reader thread is ever pinned to a core that also hosts a writer |
| VV-HAL-026.3 | ⬜ | Document/verify the MVCC argument for why this is safe (readers see a pre-write snapshot) — blocked on VV-TXN-010/011, which do not exist yet |
| VV-HAL-026.4 | ⬜ | Test: attempting to pin a reader to a writer's core is rejected at startup |

---

## VV-HAL-027 — Thread-per-core runtime  ⬜
**Spec:** no dedicated section — inferred from VECTORVOLT_CORE_ARCHITECTURE.md §4.1 (Disruptor pipeline) + §4.2 (core isolation model)
**Blocked by:** VV-HAL-024, VV-HAL-025, VV-TXN-008 (ring buffer)
**Needs:** nothing hardware-special

| Task | Status | Detail |
|---|---|---|
| VV-HAL-027.1 | ⬜ | Thread-per-core executor for hot-path work (glommio/monoio-style, or hand-rolled), replacing/augmenting the general-purpose Tokio scheduler for these threads |
| VV-HAL-027.2 | ⬜ | Per-core task queue with no work-stealing across the writer/reader isolation boundary (VV-HAL-026) |
| VV-HAL-027.3 | ⬜ | Reconcile with existing Tokio usage — `vectorvolt-wal`'s group commit currently runs on Tokio's default multi-thread runtime, which is exactly the scheduling model this feature replaces on hot-path cores |
| VV-HAL-027.4 | ⬜ | Benchmark thread-per-core vs. current Tokio scheduler tail latency |

---

## VV-HAL-028 — NUMA topology detection  ⬜
**Spec:** CPU_CACHE_NUMA_ISOLATION.md §4.2
**Blocked by:** nothing (can be built standalone, ideally unified with VV-HAL-001)
**Needs:** Linux for `/sys/devices/system/node/`; Windows `GetNumaHighestNodeNumber`/`GetNumaNodeProcessorMaskEx`

| Task | Status | Detail |
|---|---|---|
| VV-HAL-028.1 | ⬜ | Linux: enumerate `/sys/devices/system/node/node*` directories |
| VV-HAL-028.2 | ⬜ | Linux: parse `nodeN/cpulist` for core membership |
| VV-HAL-028.3 | ⬜ | Linux: parse `nodeN/meminfo` for per-node memory size |
| VV-HAL-028.4 | ⬜ | Windows: `GetNumaHighestNodeNumber` + `GetNumaNodeProcessorMaskEx` |
| VV-HAL-028.5 | ⬜ | Single-socket / non-NUMA fallback: report one node covering all cores |
| VV-HAL-028.6 | ⬜ | `NumaNode { id, cpus, memory_bytes }` struct, folded into `CpuTopology` (VV-HAL-001) |
| VV-HAL-028.7 | ⬜ | Test: mocked multi-node `/sys` fixture parses correctly |

---

## VV-HAL-029 — `LocalNumaArena` bump allocator  ⬜
**Spec:** VECTORVOLT_CORE_ARCHITECTURE.md §3.3
**Blocked by:** VV-HAL-028
**Needs:** Linux `mbind`; Windows `VirtualAllocExNuma`

| Task | Status | Detail |
|---|---|---|
| VV-HAL-029.1 | ⬜ | `LocalNumaArena { node_id, base_ptr, cursor: AtomicUsize, capacity }` struct |
| VV-HAL-029.2 | ⬜ | Linux backing allocation: `mmap` + `mbind(MPOL_BIND)` |
| VV-HAL-029.3 | ⬜ | Windows backing allocation: `VirtualAllocExNuma` |
| VV-HAL-029.4 | ⬜ | Lock-free bump `alloc(size)` via `AtomicUsize::fetch_add` |
| VV-HAL-029.5 | ⬜ | `!Send` marker so a `LocalNumaArena` cannot be moved to a different thread/NUMA node — compile-time locality enforcement per VECTORVOLT_CORE_ARCHITECTURE.md §3.3 |
| VV-HAL-029.6 | ⬜ | Arena-exhaustion handling: new arena vs. error vs. spill to DRAM tier |
| VV-HAL-029.7 | ⬜ | Compile-fail (`trybuild`) test proving a cross-thread move does not compile |
| VV-HAL-029.8 | ⬜ | Test: allocations never straddle arena capacity |

---

## VV-HAL-030 — NUMA node binding (`mbind`/`MPOL_BIND`)  🧊
**Spec:** CPU_CACHE_NUMA_ISOLATION.md §4.3; VECTORVOLT_CORE_ARCHITECTURE.md §3.3
**Blocked by:** VV-HAL-028, VV-HAL-029
**Needs:** Linux — `mbind` is a Linux-only syscall. Windows uses the structurally different `VirtualAllocExNuma` API (tracked as VV-HAL-029.3, not gated on Linux)

| Task | Status | Detail |
|---|---|---|
| VV-HAL-030.1 | 🧊 | Linux `libc::mbind` FFI wrapper — needs a Linux box to compile and test against the real syscall |
| VV-HAL-030.2 | ⬜ | Windows equivalent via `VirtualAllocExNuma` — not Linux-gated, buildable today (duplicate-tracked at VV-HAL-029.3) |
| VV-HAL-030.3 | 🧊 | Verify binding via `move_pages`/`get_mempolicy` readback — Linux only |

---

## VV-HAL-031 — 2MB huge pages (`MAP_HUGETLB` / Win large pages)  ⬜
**Spec:** CPU_CACHE_NUMA_ISOLATION.md §7; VECTORVOLT_CORE_ARCHITECTURE.md §3.4
**Blocked by:** nothing to build the primitive; wiring into the index is blocked by VV-IDX-007
**Needs:** Linux `vm.nr_hugepages` sysctl reservation for real testing; Windows `SeLockMemoryPrivilege` for `MEM_LARGE_PAGES`

| Task | Status | Detail |
|---|---|---|
| VV-HAL-031.1 | ⬜ | Linux: `mmap(MAP_HUGETLB \| MAP_POPULATE)` wrapper |
| VV-HAL-031.2 | ⬜ | Size round-up to the 2 MB boundary |
| VV-HAL-031.3 | ⬜ | `mlock()` the allocation immediately after (shared implementation target with VV-HAL-033) |
| VV-HAL-031.4 | ⬜ | Windows: `VirtualAlloc(MEM_LARGE_PAGES)` (requires `SeLockMemoryPrivilege`) |
| VV-HAL-031.5 | ⬜ | Graceful fallback to normal 4 KB `mmap` when huge pages are unavailable (`/proc/sys/vm/nr_hugepages` == 0) |
| VV-HAL-031.6 | ⬜ | Wire into Cuckoo Hash PK index allocation (VV-IDX-007) |
| VV-HAL-031.7 | ⬜ | Startup validator: check the `vm.nr_hugepages` sysctl, warn if insufficient for the configured Tier-1 budget |
| VV-HAL-031.8 | 🧊 | Test: allocation succeeds/fails predictably in a container without hugepage reservation — Linux only |

---

## VV-HAL-032 — `MAP_POPULATE` pre-faulting  🟡
**Spec:** CPU_CACHE_NUMA_ISOLATION.md §3.3, §7.2 (Explicit vs. Transparent Huge Pages)
**Blocked by:** nothing for a HAL-level wrapper
**Needs:** Linux to observe real `MAP_POPULATE` behavior — on Windows `memmap2::populate()` is a documented no-op

Evidence: `vectorvolt-storage/src/reader.rs::PartitionReader::open` — already uses
`memmap2::MmapOptions::new().populate().map(&file)` (reader.rs:67), with the crate's own doc comment
acknowledging the Windows gap: "On Linux, `MmapOptions::populate()` pre-faults all pages... On other
platforms, `populate()` is a no-op; demand-paging still applies" (reader.rs:10-12).

| Task | Status | Detail |
|---|---|---|
| VV-HAL-032.1 | ✅ | `memmap2::MmapOptions::populate()` used directly in `PartitionReader::open` (reader.rs:67) |
| VV-HAL-032.2 | ⬜ | HAL-level wrapper (`vectorvolt_hal::mmap_populate(path)`) so every crate shares one implementation instead of each depending on `memmap2` directly |
| VV-HAL-032.3 | ⬜ | Real Windows pre-fault: `populate()` is a no-op there today, so a genuine implementation (e.g., touch every page after `MapViewOfFile`) is still needed |
| VV-HAL-032.4 | 🧊 | Test confirming zero minor page faults on first access after populate, via `/proc/self/stat` — Linux only |

---

## VV-HAL-033 — `mlock()` tier-1 pinning  ⬜
**Spec:** CPU_CACHE_NUMA_ISOLATION.md §3.3, §7.1 (Tier-1 budget)
**Blocked by:** VV-HAL-001 (L3 size needed to tune the budget)
**Needs:** Linux `mlock`/`RLIMIT_MEMLOCK`; Windows `VirtualLock`

| Task | Status | Detail |
|---|---|---|
| VV-HAL-033.1 | ⬜ | Linux `mlock()` FFI wrapper |
| VV-HAL-033.2 | ⬜ | Windows `VirtualLock()` equivalent |
| VV-HAL-033.3 | ⬜ | Tier-1 budget accounting — maximum rows whose PKs get `mlock`'d, tuned against the detected L3 size |
| VV-HAL-033.4 | ⬜ | `munlock()` / arena-shrink path when the budget is exceeded |
| VV-HAL-033.5 | ⬜ | `RLIMIT_MEMLOCK` check + operator-facing error when the process ulimit is too low to honor the budget |
| VV-HAL-033.6 | ⬜ | Wire into Cuckoo Hash (VV-IDX-001) / Cuckoo Filter (VV-IDX-005) allocation |

---

## VV-HAL-034 — jemalloc NUMA arenas (`percpu_arena`)  ⬜
**Spec:** CPU_CACHE_NUMA_ISOLATION.md §4.4
**Blocked by:** nothing
**Needs:** Linux for the value proposition — jemalloc NUMA arenas are a Linux-specific optimization

`grep -rn jemalloc Cargo.toml crates/*/Cargo.toml` — zero hits; no `jemallocator` dependency anywhere.

| Task | Status | Detail |
|---|---|---|
| VV-HAL-034.1 | ⬜ | Add `tikv-jemallocator` dependency + `#[global_allocator]`, Linux-only build config |
| VV-HAL-034.2 | ⬜ | `MALLOC_CONF=narenas:<N>,percpu_arena:percpu` environment wiring at process start |
| VV-HAL-034.3 | ⬜ | Verify the default allocator stays system malloc on Windows/macOS (jemalloc arenas are Linux-only per spec) |
| VV-HAL-034.4 | ⬜ | Benchmark incidental heap allocations (log strings, error paths) for cross-NUMA traffic, before vs. after |

---

## VV-HAL-035 — `isolcpus`/`nohz_full`/`rcu_nocbs` boot-param docs + validator  🧊
**Spec:** CPU_CACHE_NUMA_ISOLATION.md §5.2
**Blocked by:** nothing for the documentation half; the validator needs Linux
**Needs:** Linux boot configuration access to validate against `/proc/cmdline`

| Task | Status | Detail |
|---|---|---|
| VV-HAL-035.1 | ✅ | Operator documentation with the exact `GRUB_CMDLINE_LINUX` recipe — already written in CPU_CACHE_NUMA_ISOLATION.md §5.2 |
| VV-HAL-035.2 | 🧊 | Startup validator: read `/proc/cmdline`, warn if `isolcpus`/`nohz_full` are absent for the cores VectorVolt intends to pin — Linux only |
| VV-HAL-035.3 | 🧊 | Validator: confirm isolated cores are not still visible to `/proc/schedstat` load balancing — Linux only |
| VV-HAL-035.4 | ⬜ | `vvctl doctor` subcommand surfacing the validator's findings — the command scaffolding itself is not Linux-gated, only the underlying check is |

---

## VV-HAL-036 — Wait strategies: BusySpin / Yielding / Blocking  ⬜
**Spec:** VECTORVOLT_CORE_ARCHITECTURE.md §4.3
**Blocked by:** VV-TXN-008 (ring buffer doesn't exist yet — nothing to wait on)
**Needs:** nothing hardware-special

| Task | Status | Detail |
|---|---|---|
| VV-HAL-036.1 | ⬜ | `WaitStrategy` trait/enum |
| VV-HAL-036.2 | ⬜ | `BusySpin` — CAS loop, never yields |
| VV-HAL-036.3 | ⬜ | `Yielding` — `std::hint::spin_loop()` |
| VV-HAL-036.4 | ⬜ | `Blocking` — condition variable |
| VV-HAL-036.5 | ⬜ | Per-shard configurable strategy (financial-grade = `BusySpin`, cloud = `Yielding`/`Blocking`, per the VECTORVOLT_CORE_ARCHITECTURE.md §4.3 table) |
| VV-HAL-036.6 | ⬜ | Latency/CPU-use benchmark comparing all three strategies |
| VV-HAL-036.7 | ⬜ | Depends on the ring buffer existing (VV-TXN-008) to have something to wait on |

---

## VV-HAL-037 — RDTSC / cycle-accurate timer  ✅
**Spec:** no dedicated vision-doc section — engineering necessity documented in the crate's own module doc (`timer.rs:1-44`), which explains why `std::time::Instant` (20–30 ns/call) can't measure a <100 ns operation honestly
**Blocked by:** nothing
**Needs:** nothing — falls back to a portable monotonic clock on non-x86/aarch64

Evidence: `vectorvolt-hal/src/timer.rs`.

| Task | Status | Detail |
|---|---|---|
| VV-HAL-037.1 | ✅ | `cycles()` via `RDTSC` on x86-64 (timer.rs:53-60) |
| VV-HAL-037.2 | ✅ | `cycles()` via `CNTVCT_EL0` on aarch64 (timer.rs:61-70) |
| VV-HAL-037.3 | ✅ | Portable monotonic-nanosecond fallback on other architectures (timer.rs:79-85) |
| VV-HAL-037.4 | ✅ | `CycleTimer::calibrate()` — busy-wait TSC-vs-wall-clock ratio measurement (~2 ms) (timer.rs:105-130) |
| VV-HAL-037.5 | ✅ | `CycleTimer::identity()` — const-constructible, 1:1 ratio, for tests/static contexts (timer.rs:134-136) |
| VV-HAL-037.6 | ✅ | `Stopwatch` RAII cycle counter with `elapsed_ns`/`elapsed_cycles`/`reset` (timer.rs:186-214) |
| VV-HAL-037.7 | ✅ | Saturating subtraction guards against apparent backwards ticks from cross-core migration (timer.rs:200-202) |
| VV-HAL-037.8 | ✅ | Plausibility test: calibrated clock lands within 0.3–12 GHz — `calibration_yields_a_plausible_clock` |
| VV-HAL-037.9 | ✅ | Monotonicity test — `cycles_advance_monotonically` |
| VV-HAL-037.10 | ✅ | Documented that cross-core TSC comparison is unsound without pinning (blocked feature: VV-HAL-024) — timer.rs:28-30 |

---
---

## VV-TST-001 — 140 unit + integration tests  ✅
**Spec:** no dedicated doc section — quality baseline tracked in `VECTORVOLT_FEATURE_MATRIX.md`'s headline count and sprint log
**Blocked by:** nothing
**Needs:** nothing

Baseline was 140 at S0; S1a added the HAL crate's tests, bringing the workspace to 195 per the
sprint log. This audit independently counted 180 `#[test]` attributes via
`grep -rn "#\[test\]" --include=*.rs crates/ | wc -l` — the difference from 195 is accounted for by
`#[tokio::test]`-style async tests and doctests, which the plain grep does not match.

| Task | Status | Detail |
|---|---|---|
| VV-TST-001.1 | ✅ | Unit tests across every crate (`vectorvolt-hal`, `-wal`, `-index`, `-storage`, `-txn`, `-engine`, `vvctl`) |
| VV-TST-001.2 | ✅ | Integration tests under crate `tests/` directories (e.g. `vectorvolt-hal/tests/simd_equivalence.rs`) |
| VV-TST-001.3 | ✅ | Doc-tests compiled and run via `cargo test --workspace --doc` (`vectorvolt/.github/workflows/ci.yml:32-33`) |
| VV-TST-001.4 | ✅ | Test-count growth tracked in the traceability sprint log (140 → 195, `VECTORVOLT_FEATURE_MATRIX.md`) |

---

## VV-TST-002 — Zero-clippy-warning gate  ✅
**Spec:** headline count in `VECTORVOLT_FEATURE_MATRIX.md` ("Zero clippy warnings workspace-wide")
**Blocked by:** nothing
**Needs:** nothing

| Task | Status | Detail |
|---|---|---|
| VV-TST-002.1 | ✅ | `cargo clippy --manifest-path vectorvolt/Cargo.toml --all-targets -- -D warnings` (`.github/workflows/vectorvolt-ci.yml:42`) |
| VV-TST-002.2 | ✅ | `cargo clippy --workspace --all-features -- -D warnings` in the per-crate CI (`vectorvolt/.github/workflows/ci.yml:27`) |
| VV-TST-002.3 | ✅ | `cargo fmt --all --check` gate present in both CI configs |

---

## VV-TST-003 — CI pipeline (build/test/clippy/fmt)  ✅
**Spec:** none — operational baseline
**Blocked by:** nothing
**Needs:** nothing

Two overlapping workflow files both do this today: `.github/workflows/vectorvolt-ci.yml` (path-gated
on `vectorvolt/**`, triggers on push to `main` and PRs) and `vectorvolt/.github/workflows/ci.yml`
(triggers on every push to `main`/`develop` and PRs to `main`, plus an OS matrix — see VV-TST-004).
Functionally redundant; not wrong, just undocumented duplication.

| Task | Status | Detail |
|---|---|---|
| VV-TST-003.1 | ✅ | GitHub Actions workflow triggered on push/PR (`.github/workflows/vectorvolt-ci.yml`) |
| VV-TST-003.2 | ✅ | Cargo registry + target directory caching (`actions/cache@v4`) |
| VV-TST-003.3 | ✅ | Format-check step |
| VV-TST-003.4 | ✅ | Clippy step |
| VV-TST-003.5 | ✅ | Test step (`cargo test --all` / `cargo test --workspace --all-features`) |
| VV-TST-003.6 | ✅ | Second, OS-matrixed workflow at `vectorvolt/.github/workflows/ci.yml` running the same checks plus doc-tests and a release-build smoke test |
| VV-TST-003.7 | ⬜ | Consolidate the two overlapping CI workflow files into one — currently tech debt, not a functional gap |

---

## VV-TST-004 — Multi-platform release CI (5 targets)  ✅
**Spec:** none — operational baseline; matches `VECTORVOLT_FEATURE_MATRIX.md`'s Hardware Compatibility Matrix intent
**Blocked by:** nothing
**Needs:** nothing

Evidence: `vectorvolt/.github/workflows/release.yml`. Confirmed 5 build targets plus a 3-OS test
matrix, packaging, and release automation.

| Task | Status | Detail |
|---|---|---|
| VV-TST-004.1 | ✅ | Pre-release test matrix: `ubuntu-latest`, `macos-latest`, `windows-latest` (release.yml:17-23) |
| VV-TST-004.2 | ✅ | Build target: `x86_64-unknown-linux-musl` (static, runs on any Linux) (release.yml:40) |
| VV-TST-004.3 | ✅ | Build target: `aarch64-unknown-linux-musl` via `cross` (release.yml:43, 78-84) |
| VV-TST-004.4 | ✅ | Build target: `aarch64-apple-darwin` (release.yml:48) |
| VV-TST-004.5 | ✅ | Build target: `x86_64-apple-darwin` (release.yml:52) |
| VV-TST-004.6 | ✅ | Build target: `x86_64-pc-windows-msvc` (release.yml:56) |
| VV-TST-004.7 | ✅ | tar.gz/zip packaging + SHA-256 checksums per target (release.yml:91-114) |
| VV-TST-004.8 | ✅ | `.deb` packaging (amd64) (release.yml:126-156) |
| VV-TST-004.9 | ✅ | `.rpm` packaging (x86_64) (release.yml:159-187) |
| VV-TST-004.10 | ✅ | GitHub Release creation with auto-generated notes (release.yml:190-209) |

---

## VV-TST-005 — SIMD-vs-scalar equivalence tests  ✅
**Spec:** VECTORVOLT_CORE_ARCHITECTURE.md §7.1 (scalar as fallback/oracle); module doc of `tests/simd_equivalence.rs`
**Blocked by:** nothing for AVX2/scalar (done); extending to future tiers is blocked by those tiers existing
**Needs:** nothing further for the current AVX2/scalar pair

Evidence: `vectorvolt-hal/tests/simd_equivalence.rs` — 9 test functions covering every length 0–300,
random data, adversarial signed-boundary data, and degenerate ranges.

| Task | Status | Detail |
|---|---|---|
| VV-TST-005.1 | ✅ | Exhaustive length sweep 0–300 on sequential data — `every_length_up_to_300_agrees_on_sequential_data` |
| VV-TST-005.2 | ✅ | Exhaustive length sweep 0–300 on random data (seeded xorshift64*, reproducible) — `every_length_up_to_300_agrees_on_random_data` |
| VV-TST-005.3 | ✅ | Adversarial values around `i64::MAX` / the signed-compare boundary — `adversarial_values_around_the_signed_boundary_agree` |
| VV-TST-005.4 | ✅ | High-bit-set regression case — the exact class of bug the XOR-flip trick prevents — `all_values_above_i64_max_are_found` |
| VV-TST-005.5 | ✅ | Degenerate ranges: inverted, empty, single-point, full-domain — `inverted_and_empty_ranges_yield_nothing`, `single_point_range_matches_exact_duplicates` |
| VV-TST-005.6 | ✅ | `filter_range` append-not-clear contract — `filter_range_appends_without_clearing` |
| VV-TST-005.7 | ✅ | `sum` overflow-wrap equivalence — `sum_wraps_identically_to_scalar` |
| VV-TST-005.8 | ✅ | `crc32c` equivalence across every length 0–300 — `crc32c_agrees_across_paths_on_all_lengths` |
| VV-TST-005.9 | ✅ | Dispatch sanity check: detected tier is one that has implemented kernels — `dispatch_reports_a_usable_tier` |
| VV-TST-005.10 | ⬜ | Extend the harness to AVX-512 kernels once VV-HAL-007 lands |
| VV-TST-005.11 | ⬜ | Extend the harness to NEON kernels once VV-HAL-008 lands |

---

## VV-TST-006 — Property-based tests (`proptest`)  ⬜
**Spec:** none
**Blocked by:** nothing
**Needs:** nothing

No `proptest` dependency exists anywhere in the workspace.

| Task | Status | Detail |
|---|---|---|
| VV-TST-006.1 | ⬜ | Add `proptest` dev-dependency to `vectorvolt-hal` and other crates as needed |
| VV-TST-006.2 | ⬜ | Property: SIMD kernel output equals the scalar oracle for arbitrary `Vec<u64>` and arbitrary `lo`/`hi` — generalizes the hand-rolled equivalence tests in VV-TST-005 |
| VV-TST-006.3 | ⬜ | Property: WAL replay is idempotent/order-preserving for arbitrary command sequences |
| VV-TST-006.4 | ⬜ | Property: Cuckoo Hash never loses a key across arbitrary insert/delete sequences within its load factor |
| VV-TST-006.5 | ⬜ | Shrinking-driven minimal-failing-case reporting on any property failure |

---

## VV-TST-007 — Criterion benchmark suite  ⬜
**Spec:** none — `vvctl hal`/`vvctl bench` currently do ad-hoc best-of-N timing instead
**Blocked by:** nothing
**Needs:** nothing

No `criterion` dependency exists anywhere in the workspace; `vvctl hal`'s `best_of()` (hal.rs:164-182)
is a hand-rolled minimum-of-N-runs timer, not a statistically rigorous benchmark harness.

| Task | Status | Detail |
|---|---|---|
| VV-TST-007.1 | ⬜ | Add `criterion` dev-dependency + `[[bench]]` targets |
| VV-TST-007.2 | ⬜ | Migrate `vvctl hal`'s ad-hoc timing to Criterion benches (outlier detection, confidence intervals) |
| VV-TST-007.3 | ⬜ | Benches for WAL group-commit throughput |
| VV-TST-007.4 | ⬜ | Benches for the O(1) `read_value` point-lookup latency (currently only measured via `vvctl bench`) |
| VV-TST-007.5 | ⬜ | HTML report generation + baseline comparison (`cargo bench -- --save-baseline`) |

---

## VV-TST-008 — CI performance gate (>5% p99 regression → reject)  ⬜
**Spec:** `VECTORVOLT_FEATURE_MATRIX.md` note "LD-GATE"
**Blocked by:** VV-TST-007 (needs Criterion baselines to compare against)
**Needs:** nothing hardware-special, but shared CI-runner noise is a real methodological risk

| Task | Status | Detail |
|---|---|---|
| VV-TST-008.1 | ⬜ | Store baseline Criterion results on `main` |
| VV-TST-008.2 | ⬜ | CI step comparing a PR's benchmark run against the `main` baseline |
| VV-TST-008.3 | ⬜ | Threshold logic: fail the PR if any tracked kernel's p99 regresses >5% |
| VV-TST-008.4 | ⬜ | Noisy-neighbor mitigation on shared GitHub runners — runner-to-runner variance can itself exceed 5%, the hardest part of this feature |
| VV-TST-008.5 | ⬜ | Historical-trend dashboard/artifact per kernel |

---

## VV-TST-009 — Deterministic simulation testing (mock clock/net)  ⬜
**Spec:** VECTORVOLT_CORE_ARCHITECTURE.md §10 (Deterministic Simulation Testing)
**Blocked by:** VV-CLU-009 (Raft — nothing to simulate failover of yet)
**Needs:** nothing hardware-special

| Task | Status | Detail |
|---|---|---|
| VV-TST-009.1 | ⬜ | `SimulationRuntime` with seeded deterministic scheduling |
| VV-TST-009.2 | ⬜ | Mock clock — virtual time advance, no wall-clock dependency |
| VV-TST-009.3 | ⬜ | Mock network layer — in-memory message passing with injectable delay/loss |
| VV-TST-009.4 | ⬜ | Network-partition injection API (`sim.partition_nodes(...)`, per the spec's own example) |
| VV-TST-009.5 | ⬜ | Depends on cluster/Raft existing to have something meaningful to simulate |
| VV-TST-009.6 | ⬜ | Reproducibility test: same seed produces the same execution trace |

---

## VV-TST-010 — Chaos testing (partition/disk/CPU stall/clock skew)  ⬜
**Spec:** VECTORVOLT_CORE_ARCHITECTURE.md §10 (Chaos Testing)
**Blocked by:** shares infrastructure with VV-TST-009
**Needs:** nothing hardware-special

| Task | Status | Detail |
|---|---|---|
| VV-TST-010.1 | ⬜ | Network-partition injection between arbitrary node subsets |
| VV-TST-010.2 | ⬜ | Disk I/O failure injection on specific WAL segments |
| VV-TST-010.3 | ⬜ | CPU-stall simulation (sleep injection on the hot path) |
| VV-TST-010.4 | ⬜ | Memory-pressure injection (arena capacity limiting) |
| VV-TST-010.5 | ⬜ | Clock-skew injection between simulated nodes |
| VV-TST-010.6 | ⬜ | Wire the chaos harness into CI as a scheduled (not per-PR) job |

---

## VV-TST-011 — Fuzzing (`cargo-fuzz`) on format parsers  ⬜
**Spec:** none
**Blocked by:** nothing
**Needs:** nothing

| Task | Status | Detail |
|---|---|---|
| VV-TST-011.1 | ⬜ | Add a `cargo-fuzz` fuzz-targets crate |
| VV-TST-011.2 | ⬜ | Fuzz target: WAL frame parser (`read_all`/`read_range`) |
| VV-TST-011.3 | ⬜ | Fuzz target: `.vvsst` micro-partition header/column parser |
| VV-TST-011.4 | ⬜ | Fuzz target: differential fuzzing of SIMD kernels against arbitrary byte-reinterpreted `Vec<u64>` input vs. the scalar oracle |
| VV-TST-011.5 | ⬜ | CI job running each fuzz target for a bounded time budget (e.g., 60 s smoke-fuzz per PR) |
| VV-TST-011.6 | ⬜ | Corpus seed collection from real WAL/partition files |

---

## VV-TST-012 — Synthetic data generator (FK-aware, histogram-matched)  ⬜
**Spec:** none
**Blocked by:** nothing
**Needs:** nothing

| Task | Status | Detail |
|---|---|---|
| VV-TST-012.1 | ⬜ | Generator core: deterministic seeded row generation per schema |
| VV-TST-012.2 | ⬜ | FK-awareness: generated child rows reference an existing parent PK (or an intentionally dangling one, for negative tests) |
| VV-TST-012.3 | ⬜ | Histogram matching: generated column-value distributions approximate a supplied real-world histogram |
| VV-TST-012.4 | ⬜ | CLI integration (`vvctl gen` or similar) |
| VV-TST-012.5 | ⬜ | Use in Criterion benchmarks (VV-TST-007) to replace unrealistic synthetic-sequential test data |

---

## VV-TST-013 — Reference domain: Financial Ledger (RD-1)  ⬜
**Spec:** `VECTORVOLT_FEATURE_MATRIX.md` note "RD-1"
**Blocked by:** VV-TST-012
**Needs:** nothing hardware-special

| Task | Status | Detail |
|---|---|---|
| VV-TST-013.1 | ⬜ | Schema design (accounts, transactions, ledger entries, double-entry invariants) |
| VV-TST-013.2 | ⬜ | Load generator using VV-TST-012 |
| VV-TST-013.3 | ⬜ | Correctness invariant checks (balances reconcile, no disallowed negative balances) |
| VV-TST-013.4 | ⬜ | End-to-end demo/benchmark scenario document |

---

## VV-TST-014 — Reference domain: Workflow Engine (RD-2)  ⬜
**Spec:** `VECTORVOLT_FEATURE_MATRIX.md` note "RD-2"
**Blocked by:** VV-TST-012
**Needs:** nothing hardware-special

| Task | Status | Detail |
|---|---|---|
| VV-TST-014.1 | ⬜ | Schema design (workflow definitions, instances, task states, transitions) |
| VV-TST-014.2 | ⬜ | State-machine transition-legality checks under concurrent updates |
| VV-TST-014.3 | ⬜ | Load generator using VV-TST-012 |
| VV-TST-014.4 | ⬜ | End-to-end demo scenario |

---

## VV-TST-015 — Reference domain: Order Management (RD-3)  ⬜
**Spec:** `VECTORVOLT_FEATURE_MATRIX.md` note "RD-3"
**Blocked by:** VV-TST-012; VV-QRY-013 for the cascade-delete check specifically
**Needs:** nothing hardware-special

| Task | Status | Detail |
|---|---|---|
| VV-TST-015.1 | ⬜ | Schema design (orders, line items, inventory, fulfillment states) |
| VV-TST-015.2 | ⬜ | FK cascade-behavior checks (depends on VV-QRY-013, cascade DELETE) |
| VV-TST-015.3 | ⬜ | Load generator using VV-TST-012 |
| VV-TST-015.4 | ⬜ | End-to-end demo scenario |

---

## VV-TST-016 — Loom concurrency tests  ⬜
**Spec:** `VECTORVOLT_FEATURE_MATRIX.md` note "For lock-free ring buffer"
**Blocked by:** full ring-buffer coverage needs VV-TXN-008; two sub-tasks below do not
**Needs:** nothing hardware-special

No `loom` dependency exists anywhere in the workspace.

| Task | Status | Detail |
|---|---|---|
| VV-TST-016.1 | ⬜ | Add `loom` dev-dependency |
| VV-TST-016.2 | ⬜ | Loom model for the future LMAX ring buffer's single-writer/multi-reader ordering (blocked by VV-TXN-008) |
| VV-TST-016.3 | ⬜ | Loom model for `CachePadded` atomics under concurrent access — can be built today, does not require the ring buffer |
| VV-TST-016.4 | ⬜ | Loom model for the `OnceLock`-cached SIMD tier detection under a concurrent first-call race — can be built today, exercises `simd::tier()` |
| VV-TST-016.5 | ⬜ | CI job running loom models as a separate, less-frequent job (loom models are slow) |

---

## Summary

- **HAL (37 features, 223 tasks):** 94 ✅ / 121 ⬜ / 8 🧊. The S1a sprint (branch `feat/hal-simd`)
  delivered real, tested, measured primitives for cache alignment, SIMD dispatch + AVX2 kernels,
  hardware CRC-32C, all four prefetch localities, and a cycle-accurate timer. The largest gap is not
  correctness but **integration**: `vectorvolt-hal` is depended on only by `vvctl` today — none of
  the kernels are yet called from the storage, query, index, or engine crates that would make them
  matter for the <100 ns read target. Topology detection, NUMA, core pinning, huge pages, and
  `mlock` — everything requiring OS-level FFI beyond `cpuid`/`RDTSC`/intrinsics — has zero code.
- **Testing (16 features, 90 tasks):** 32 ✅ / 58 ⬜. CI/lint/multi-platform release infrastructure is
  solid and genuinely done. Everything past "does it build and pass tests" — property testing,
  Criterion, a performance regression gate, deterministic simulation, chaos testing, fuzzing,
  synthetic data, reference domains, and loom — is unbuilt.
