# Layer 11 — Observability · 13 — Security · 14 — Prefetch · 15 — Projections · 16 — Hardware · 17 — Autonomy
Task-level traceability. Status: ✅ done · 🟡 partial · ⬜ pending · 🧊 env-gated · 🔬 research

Generated from `VECTORVOLT_FEATURE_MATRIX.md` (Layers 11, 13, 14, 15, 16, 17), `docs/vision/PREDICTIVE_PREFETCH_ENGINE.md`,
`docs/vision/HARDWARE_FRONTIER.md`, `docs/vision/RESEARCH_FRONTIER.md`, `docs/vision/VECTORVOLT_STORAGE_TIERING.md` (Parts 4-5),
`docs/vision/VECTORVOLT_CORE_ARCHITECTURE.md` §11, `docs/security/MFA_SESSION_MANDATE.md`, `docs/security/BACKUP_STRATEGY.md`,
and a line-by-line read of `vectorvolt-store/src/projection.rs` and `vectorvolt-wal/src/metrics.rs`.

## Rollup
| Layer | Features | Tasks total | ✅ | ⬜ | 🧊 | 🔬 |
|---|---|---|---|---|---|---|
| 11 — Observability | 11 | 71 | 11 | 52 | 8 | 0 |
| 13 — Security | 9 | 59 | 0 | 49 | 10 | 0 |
| 14 — Prefetch | 8 | 51 | 0 | 51 | 0 | 0 |
| 15 — Projections | 9 | 54 | 9 | 45 | 0 | 0 |
| 16 — Hardware | 5 | 25 | 1 | 12 | 7 | 5 |
| 17 — Autonomy | 10 | 34 | 0 | 9 | 5 | 20 |
| **Total** | **52** | **294** | **21** | **218** | **30** | **25** |

*(Counts are task-row tallies, not feature tallies — a single feature can contain a mix of ✅/⬜/🧊/🔬 rows.
Verified by machine-counting every `VV-XXX-NNN.N` task row's status glyph in this file — re-run that count
if the tables below are ever edited without updating this rollup.)*

## Scope honesty note

**Genuinely VectorVolt core (the database):** Layer 11 (Observability) and Layer 13 (Security) are core
database concerns — every serious OLTP engine ships metrics, tracing, TLS, encryption at rest, and
AuthN/AuthZ. Layer 15 (Projections/CQRS) is core too: the `Projection` trait and dispatcher already live
in `vectorvolt-store`, and the WAL-as-event-log pattern is a storage-engine feature, not an EKOS add-on.

**EKOS-adjacent, built *on top of* VectorVolt:** Layer 14 (Prefetch Intelligence) is explicitly authored
as an EKOS component in its own source doc — `PREDICTIVE_PREFETCH_ENGINE.md` §9 places the crate at
`ekos/crates/ekos-prefetch/`, not under `vectorvolt/crates/`. It consumes VectorVolt primitives (Cuckoo
filter, CSR graph, huge pages) but the Markov-chain intelligence itself is EKOS. Layer 17 (Autonomy — ARP,
CIP, Shadowbox, in-process LLM, self-upgrade) is unambiguously EKOS: the feature matrix itself titles the
layer "EKOS-adjacent, deferred until VectorVolt core is done," and every one of its ten features either
needs an LLM runtime, Linux namespaces, or both.

**Unscheduled research, by the source docs' own admission:** `HARDWARE_FRONTIER.md` and
`RESEARCH_FRONTIER.md` both open with an explicit disclaimer that they carry no roadmap commitment.
But "frontier" is not uniformly "unscheduled" — read closely, the hardware doc places Intel QAT at
Phase 3, Intel DSA at Phase 4, RDMA at Phase 5, and ARM-as-default at "now," all with concrete
integration targets. Those four are marked 🧊 (or ⬜ for the ARM software work) in this document, not 🔬,
because they are scheduled but blocked on hardware the owner doesn't have yet (QAT card, DSA, RDMA NIC,
ARM box) — 🧊 means "we know what to build and when, we're missing the box." CXL 3.0 CAS-consensus
(VV-HW-004 / VV-CLU-022) and FPGA constraint validation (VV-HW-005) are different: the hardware doc says
outright "nobody has done this yet" and "$10,000+/chip... explicitly not now" with no phase number at
all — those are marked 🔬. Everything in Layer 17 plus VV-SEC-009 (behavioral fingerprinting) and the
Layer 14 speculative items that cite `RESEARCH_FRONTIER.md` inherit the same 🔬 treatment for the same
reason: no phase, no algorithm chosen, no schedule.

**Linux-gated (🧊), independent of the research question:** PAM MFA (VV-SEC-007), eBPF system metrics
(VV-OBS-008), always-on perf/flame-graph profiling (VV-OBS-010), and Shadowbox namespace isolation
(VV-AUT-003) are fully designed in their source docs — they are not research, they are waiting on a
Linux box. The owner has two AWS EC2 Linux instances available for exactly this work; the design/portable
subtasks within these features (e.g. `MfaGuard`'s HMAC verification logic, which touches no Linux-only
API) are marked ⬜ and can proceed on Windows today, while the PAM/eBPF/perf/`unshare` subtasks are 🧊
until run against those two boxes.

---

# Layer 11 — Observability

## VV-OBS-001 — `tracing` structured logging  ✅
**Spec:** VECTORVOLT_CORE_ARCHITECTURE.md §11.3 (observability first-class requirement)
**Blocked by:** none
**Env:** none

| Task | Status | Detail |
|---|---|---|
| VV-OBS-001.1 | ✅ | `tracing_subscriber::fmt()` + `EnvFilter` init in `vvctl` — `vectorvolt/crates/vvctl/src/main.rs:125-128` |
| VV-OBS-001.2 | ✅ | `tracing::{debug, info}` calls in WAL writer — `vectorvolt/crates/vectorvolt-wal/src/writer.rs:15` |
| VV-OBS-001.3 | ✅ | `tracing::debug` calls in WAL reader — `vectorvolt/crates/vectorvolt-wal/src/reader.rs:10` |
| VV-OBS-001.4 | ✅ | `tracing::{debug, error, warn}` in group-commit batcher — `vectorvolt/crates/vectorvolt-wal/src/group_commit.rs:23` |
| VV-OBS-001.5 | ⬜ | Add `tracing` spans/events to `vectorvolt-store`, `vectorvolt-txn`, `vectorvolt-engine`, `vectorvolt-query`, `vectorvolt-index` — grep confirms zero `tracing::` usage in these 5 crates today |
| VV-OBS-001.6 | ⬜ | JSON log formatter option (`tracing_subscriber::fmt().json()`) for production log shipping |
| VV-OBS-001.7 | ⬜ | Wire log level to config system instead of hard-coded `EnvFilter::new(level)` |
| VV-OBS-001.8 | ⬜ | Test: `RUST_LOG` env override takes precedence over configured default level |

## VV-OBS-002 — `BatcherMetrics` (lock-free atomics + EMA)  ✅
**Spec:** VECTORVOLT_CORE_ARCHITECTURE.md §11.1
**Blocked by:** none
**Env:** none

| Task | Status | Detail |
|---|---|---|
| VV-OBS-002.1 | ✅ | Atomic `fsync_count`/`frames_committed`/`bytes_committed`/`batch_count` — `vectorvolt/crates/vectorvolt-wal/src/metrics.rs:16-33` |
| VV-OBS-002.2 | ✅ | EMA batch-size + EMA latency (α=0.1, fixed-point) — `metrics.rs:68-78` |
| VV-OBS-002.3 | ✅ | Peak batch-size CAS loop, `frames_dropped` backpressure counter — `metrics.rs:54-66, 81-83` |
| VV-OBS-002.4 | ✅ | `MetricsSnapshot` + `Display` impl for human-readable dump — `metrics.rs:121-163` |
| VV-OBS-002.5 | ⬜ | Same atomics+EMA pattern for `vectorvolt-txn` write-path counters (commits/aborts/latency) |
| VV-OBS-002.6 | ⬜ | Same pattern for `vectorvolt-query` (lookups/scans/cache hit-rate) |
| VV-OBS-002.7 | ⬜ | Same pattern for `vectorvolt-storage` compaction (merges/reclaimed tombstones/bytes) |
| VV-OBS-002.8 | ⬜ | Wire `BatcherMetrics::snapshot()` into a `vvctl status` output |
| VV-OBS-002.9 | ⬜ | Test: EMA converges to true average within N samples under synthetic load |

## VV-OBS-003 — `vvctl bench` latency percentiles (p50/p99/p999)  ✅
**Spec:** VECTORVOLT_CORE_ARCHITECTURE.md §9 (`vvctl` CLI surface)
**Blocked by:** none
**Env:** none

| Task | Status | Detail |
|---|---|---|
| VV-OBS-003.1 | ✅ | Warm-up sample discard before percentile calc — `vectorvolt/crates/vvctl/src/commands/bench.rs:19` |
| VV-OBS-003.2 | ✅ | Sorted-index p50/p99/p999 computation — `bench.rs:163-182` |
| VV-OBS-003.3 | ✅ | Formatted percentile printout (`fmt_ns`) — `bench.rs:153-155` |
| VV-OBS-003.4 | ⬜ | Extend bench beyond point-GET (comment at `bench.rs:53` notes GET-only focus today) to range-scan and write-path percentiles |
| VV-OBS-003.5 | ⬜ | JSON output mode for CI perf-gate consumption |
| VV-OBS-003.6 | ⬜ | Per-shard percentile breakdown |
| VV-OBS-003.7 | ⬜ | Test: percentile calc correctness against a known synthetic distribution (e.g. uniform 0..10000) |

## VV-OBS-004 — Per-operation latency histograms (HDR)  ⬜
**Spec:** VECTORVOLT_CORE_ARCHITECTURE.md §11.1 (`vvwrite_latency_ns`, `vvread_latency_ns`, `vvgraph_latency_ns`)
**Blocked by:** none (foundational — VV-PRE-001 depends on this for its access-timing signal)
**Env:** none

| Task | Status | Detail |
|---|---|---|
| VV-OBS-004.1 | ⬜ | Add `hdrhistogram` crate dependency to workspace |
| VV-OBS-004.2 | ⬜ | `OpHistogram` wrapper: `record_ns(u64)`, `percentile(f64) -> u64`, lock-free record path |
| VV-OBS-004.3 | ⬜ | Wire into `vectorvolt-txn` write path → `vvwrite_latency_ns{shard, priority}` |
| VV-OBS-004.4 | ⬜ | Wire into `vectorvolt-query` read path → `vvread_latency_ns{table, index}` |
| VV-OBS-004.5 | ⬜ | Wire into graph traversal path → `vvgraph_latency_ns{depth}` — blocked by VV-QRY-015 (traversal doesn't exist yet) |
| VV-OBS-004.6 | ⬜ | Expose histogram snapshot via a `vvctl debug latency` command |
| VV-OBS-004.7 | ⬜ | Test: HDR bucket error stays within documented precision vs raw-sample percentiles |

## VV-OBS-005 — Prometheus `/metrics` endpoint (150+ counters)  ⬜
**Spec:** VECTORVOLT_CORE_ARCHITECTURE.md §11.3
**Blocked by:** VV-NET-001 (server daemon binary doesn't exist — nothing opens a socket today)
**Env:** none beyond the server existing

| Task | Status | Detail |
|---|---|---|
| VV-OBS-005.1 | ⬜ | Add `axum` + a Prometheus text-exposition crate to workspace |
| VV-OBS-005.2 | ⬜ | Define the metrics registry (counters/gauges/histograms, namespaced `vv_*`) |
| VV-OBS-005.3 | ⬜ | Register existing `BatcherMetrics` fields into the registry |
| VV-OBS-005.4 | ⬜ | Register query/txn/storage counters (from VV-OBS-002.5-7) into the registry |
| VV-OBS-005.5 | ⬜ | Expose `GET /metrics` route on the server binary — blocked by VV-NET-001 |
| VV-OBS-005.6 | ⬜ | Add `shard_id` label dimension — blocked by VV-OBS-007 |
| VV-OBS-005.7 | ⬜ | Test: scraped output is valid Prometheus text format (parse with `prometheus-parse`) |
| VV-OBS-005.8 | ⬜ | Load test: `/metrics` scrape overhead stays under 1% of a core under sustained load |

## VV-OBS-006 — OpenTelemetry OTLP trace export  ⬜
**Spec:** VECTORVOLT_CORE_ARCHITECTURE.md §11.3
**Blocked by:** VV-NET-001 (server binary), VV-NET-004 (gRPC surface needed to propagate trace context client→server→storage)
**Env:** none beyond the server/gRPC layer existing

| Task | Status | Detail |
|---|---|---|
| VV-OBS-006.1 | ⬜ | Add `opentelemetry` + `opentelemetry-otlp` dependencies |
| VV-OBS-006.2 | ⬜ | Tracer-provider init at server startup (OTLP gRPC or HTTP exporter) |
| VV-OBS-006.3 | ⬜ | Instrument request-handling spans across client → server → storage |
| VV-OBS-006.4 | ⬜ | Propagate W3C trace-context over gRPC metadata — blocked by VV-NET-004 |
| VV-OBS-006.5 | ⬜ | Config: OTLP endpoint + sampling rate via TOML — blocked by VV-ENG-017 (no config system exists) |
| VV-OBS-006.6 | ⬜ | Test: exported spans parse correctly against a local OTLP collector (e.g. `otel-cli` or Jaeger) |

## VV-OBS-007 — Per-shard metric dimensions  ⬜
**Spec:** VECTORVOLT_CORE_ARCHITECTURE.md §11.1 (`shard="0"` label in the example histogram output)
**Blocked by:** VV-CLU-005 (1-65,536 physical shard support), VV-ENG-016 (shard-per-core mapping)
**Env:** none

| Task | Status | Detail |
|---|---|---|
| VV-OBS-007.1 | ⬜ | Add `shard_id` label to all existing counters/histograms |
| VV-OBS-007.2 | ⬜ | Make `BatcherMetrics` (and its VV-OBS-002.5-7 siblings) per-shard-instantiated rather than global |
| VV-OBS-007.3 | ⬜ | Cross-shard rollup aggregation for the admin panel — blocked by VV-CTL-011 |
| VV-OBS-007.4 | ⬜ | Test: metrics correctly attribute to the originating shard under concurrent multi-shard load |

## VV-OBS-008 — eBPF system metrics (CPU stall, TLB miss, cache miss)  🧊
**Spec:** VECTORVOLT_CORE_ARCHITECTURE.md §11.3 ("Phase 6+"); LAYER-0 note ties this family to `aya-rs`
**Blocked by:** VV-OBS-005 (needs a metrics registry to publish into)
**Env:** Linux only (`aya-rs`, `perf_event_open`); owner has two AWS EC2 Linux boxes for this

| Task | Status | Detail |
|---|---|---|
| VV-OBS-008.1 | ⬜ | Design a `SystemMetricsCollector` trait with a portable no-op default (compiles on Windows) |
| VV-OBS-008.2 | 🧊 | Linux impl: CPU stall cycles via `aya-rs` eBPF program |
| VV-OBS-008.3 | 🧊 | Linux impl: TLB miss rate via `perf_event_open` |
| VV-OBS-008.4 | 🧊 | Linux impl: cache miss rate via `perf_event_open` |
| VV-OBS-008.5 | 🧊 | Wire collector output into the Prometheus registry (VV-OBS-005) |
| VV-OBS-008.6 | 🧊 | Test on an AWS EC2 Linux box: collector counters agree with `perf stat` within tolerance |

## VV-OBS-009 — `ratatui` zero-alloc TUI dashboard  ⬜
**Spec:** VECTORVOLT_CORE_ARCHITECTURE.md §9 (dashboard mock-up: nodes/shards/wal/indexes/prefetch/arp panels)
**Blocked by:** VV-NET-004 (gRPC client needed for live status feed), VV-CTL-001 (control node as data source)
**Env:** none

| Task | Status | Detail |
|---|---|---|
| VV-OBS-009.1 | ⬜ | Add `ratatui` + `crossterm` dependencies to `vvctl` |
| VV-OBS-009.2 | ⬜ | Static layout matching the CORE_ARCHITECTURE.md §9 mock-up (nodes/shards/latency/throughput/memory panels) |
| VV-OBS-009.3 | ⬜ | Zero-alloc render loop using pre-allocated buffers |
| VV-OBS-009.4 | ⬜ | Wire live data via gRPC status stream — blocked by VV-NET-004 |
| VV-OBS-009.5 | ⬜ | Keyboard navigation between panels (`[nodes] [shards] [wal] [indexes] [prefetch] [arp] [quit]`) |
| VV-OBS-009.6 | ⬜ | Test: render loop allocates 0 bytes/frame (verify via `dhat` heap profiler) |

## VV-OBS-010 — Always-on profiling (perf / flame graphs)  🧊
**Spec:** VECTORVOLT_CORE_ARCHITECTURE.md §11.3 ("Phase 6+"); LD-OBS locked decision
**Blocked by:** none additional
**Env:** Linux (`perf`); owner has two AWS EC2 Linux boxes

| Task | Status | Detail |
|---|---|---|
| VV-OBS-010.1 | ⬜ | Integrate `pprof-rs` (or raw `perf_event`) sampling profiler abstraction, portable no-op on Windows |
| VV-OBS-010.2 | 🧊 | Always-on low-rate sampling (~100 Hz) background thread on Linux |
| VV-OBS-010.3 | 🧊 | Flame-graph export via `inferno`, triggered by `vvctl debug flamegraph` |
| VV-OBS-010.4 | ⬜ | Admin-panel flame-graph viewer — blocked by VV-CTL-011 |
| VV-OBS-010.5 | 🧊 | Test on an AWS EC2 Linux box: profiling overhead stays under 2% CPU at 100 Hz |

## VV-OBS-011 — Projection consumer lag metric  ⬜
**Spec:** VECTORVOLT_STORAGE_TIERING.md Part 4 ("Consumer Lag as First-Class Metric")
**Blocked by:** VV-PRJ-003 (projection workers must exist), VV-PRJ-004 (per-worker offset tracking is the numerator/denominator)
**Env:** none

| Task | Status | Detail |
|---|---|---|
| VV-OBS-011.1 | ⬜ | Define `vv_projection_consumer_lag{worker, shard}` gauge |
| VV-OBS-011.2 | ⬜ | Compute `lag = latest_wal_lsn - worker.current_offset` — blocked by VV-PRJ-004 |
| VV-OBS-011.3 | ⬜ | Register into Prometheus (VV-OBS-005) |
| VV-OBS-011.4 | ⬜ | Alert rule: `consumer_lag > 10000` for 5 minutes → page on-call (per storage-tiering doc example) |
| VV-OBS-011.5 | ⬜ | Test: lag metric tracks correctly through an induced worker pause (VV-PRJ-006) and subsequent DLQ replay |

---

# Layer 13 — Security

## VV-SEC-001 — TLS on wire protocol  ⬜
**Spec:** VECTORVOLT_CORE_ARCHITECTURE.md §6.5 (gRPC server layer)
**Blocked by:** VV-NET-001, VV-NET-002, VV-NET-003 (no server, protocol, or transport exists yet)
**Env:** none

| Task | Status | Detail |
|---|---|---|
| VV-SEC-001.1 | ⬜ | Add `rustls` + `tokio-rustls` dependencies |
| VV-SEC-001.2 | ⬜ | `TlsConfig` (cert path, key path, min TLS version) |
| VV-SEC-001.3 | ⬜ | Wrap the TCP listener with a TLS acceptor — blocked by VV-NET-003 |
| VV-SEC-001.4 | ⬜ | Client-side TLS connector — blocked by VV-SDK-002 |
| VV-SEC-001.5 | ⬜ | Enable/disable + cert paths via TOML — blocked by VV-ENG-017 |
| VV-SEC-001.6 | ⬜ | Test: handshake succeeds against a self-signed cert in CI |
| VV-SEC-001.7 | ⬜ | Test: plaintext connection rejected when TLS is mandated |

## VV-SEC-002 — mTLS client certs  ⬜
**Spec:** VECTORVOLT_CORE_ARCHITECTURE.md §6.5
**Blocked by:** VV-SEC-001
**Env:** none

| Task | Status | Detail |
|---|---|---|
| VV-SEC-002.1 | ⬜ | CA cert loading + client-cert verification config |
| VV-SEC-002.2 | ⬜ | Per-client-cert → identity mapping (feeds VV-SEC-003 AuthZ) |
| VV-SEC-002.3 | ⬜ | Reject connections presenting no client cert or an untrusted CA |
| VV-SEC-002.4 | ⬜ | `vvctl cert generate` helper subcommand for dev/test certs |
| VV-SEC-002.5 | ⬜ | Test: connection rejected on expired or wrong-CA client cert |

## VV-SEC-003 — AuthN / AuthZ (users, roles, grants)  ⬜
**Spec:** VECTORVOLT_CORE_ARCHITECTURE.md §8.3 (multi-tenancy), §9 (`vvctl` surface)
**Blocked by:** VV-CTL-002 (cluster metadata store to hold users/roles), VV-ENG-012 (partition control by schema — grant targets)
**Env:** none

| Task | Status | Detail |
|---|---|---|
| VV-SEC-003.1 | ⬜ | `User` / `Role` / `Grant` schema in control-plane metadata store — blocked by VV-CTL-002 |
| VV-SEC-003.2 | ⬜ | Password hashing (argon2 or bcrypt) for local users |
| VV-SEC-003.3 | ⬜ | Role-based grant check enforced at table/schema access boundary — blocked by VV-ENG-012 |
| VV-SEC-003.4 | ⬜ | Session token issuance (JWT, short TTL) |
| VV-SEC-003.5 | ⬜ | `vvctl user create` / `grant` / `revoke` commands |
| VV-SEC-003.6 | ⬜ | Test: query against an ungranted table is rejected |
| VV-SEC-003.7 | ⬜ | Test: a revoked grant takes effect without a server restart |

## VV-SEC-004 — Encryption at rest (AES-256-GCM)  ⬜
**Spec:** BACKUP_STRATEGY.md §3 (backup key derivation pattern reused here for at-rest data)
**Blocked by:** VV-WAL-013 (WAL compaction/archive path to encrypt before L2→L3 move), VV-STO-017 (snapshot format to encrypt)
**Env:** none

| Task | Status | Detail |
|---|---|---|
| VV-SEC-004.1 | ⬜ | Add `aes-gcm` crate dependency |
| VV-SEC-004.2 | ⬜ | Key derivation via HKDF from a master key (mirrors BACKUP_STRATEGY.md §3 pattern) |
| VV-SEC-004.3 | ⬜ | Encrypt WAL segments before L2→L3 archive — blocked by VV-WAL-013 |
| VV-SEC-004.4 | ⬜ | Encrypt partition (`.vvsst`) and snapshot files at rest — blocked by VV-STO-017 |
| VV-SEC-004.5 | ⬜ | Key-rotation procedure (re-encrypt or envelope-key rewrap) |
| VV-SEC-004.6 | ⬜ | Test: round-trip encrypt/decrypt correctness |
| VV-SEC-004.7 | ⬜ | Test: tampered ciphertext detected via the GCM authentication tag |

## VV-SEC-005 — `EncryptionBackend` trait (QAT / software)  ⬜
**Spec:** HARDWARE_FRONTIER.md §3 (QAT scheduled Phase 3 — code sample shown verbatim in the doc)
**Blocked by:** VV-SEC-004 (software backend is the default/fallback and must exist first)
**Env:** software path none; QAT path needs a QAT card (see VV-HW-001)

| Task | Status | Detail |
|---|---|---|
| VV-SEC-005.1 | ⬜ | Define `EncryptionBackend` trait (`encrypt`/`decrypt`) exactly as sketched in HARDWARE_FRONTIER.md §3 |
| VV-SEC-005.2 | ⬜ | `AesGcmSoftwareBackend` default impl — blocked by VV-SEC-004 |
| VV-SEC-005.3 | 🧊 | `QatBackend` impl behind a `qat` feature flag — needs a QAT card |
| VV-SEC-005.4 | ⬜ | `detect_backend()` auto-select at startup (QAT if present, else software) — detection logic is portable, only the QAT arm is hardware-gated |
| VV-SEC-005.5 | ⬜ | Test: software backend round-trips correctly (runs today) |
| VV-SEC-005.6 | 🧊 | Test on QAT-equipped host: QAT and software backends produce byte-identical ciphertext for the same key/nonce |

## VV-SEC-006 — Audit log (immutable)  ⬜
**Spec:** VECTORVOLT_CORE_ARCHITECTURE.md §9 (`vvctl` surface implies audit visibility); BACKUP_STRATEGY.md treats audit KOs as a distinct namespace
**Blocked by:** VV-CTL-013 (control-plane immutable audit log — the same capability, viewed from the control-plane side), VV-WAL-004 (SHA3-256 hash chain — reused for tamper detection, already ✅ done)
**Env:** none

| Task | Status | Detail |
|---|---|---|
| VV-SEC-006.1 | ⬜ | `AuditEvent` schema: actor, action, target, timestamp, result |
| VV-SEC-006.2 | ⬜ | Append audit events to a dedicated WAL-backed append-only log |
| VV-SEC-006.3 | ⬜ | Enforce no update/delete on audit records at the API layer |
| VV-SEC-006.4 | ⬜ | `vvctl audit tail` / `vvctl audit query` commands |
| VV-SEC-006.5 | ⬜ | Wire control-plane actions (node join/drain, failover, rebalance) into the audit log — blocked by VV-CTL-001 |
| VV-SEC-006.6 | ⬜ | Test: audit log survives a crash and replays correctly (reuses WAL crash-recovery semantics, VV-WAL-006 ✅) |
| VV-SEC-006.7 | ⬜ | Test: direct file tamper is detected via the existing SHA3-256 hash chain (VV-WAL-004 ✅) |

## VV-SEC-007 — MFA session gate (`MfaGuard`)  🧊
**Spec:** MFA_SESSION_MANDATE.md (full design captured — token format §3, PAM config §4, Rust impl §5)
**Blocked by:** none
**Env:** Linux + PAM (`pam_google_authenticator`/`pam_oath`); owner has two AWS EC2 Linux boxes

| Task | Status | Detail |
|---|---|---|
| VV-SEC-007.1 | ⬜ | Scaffold an `ekos-security` crate (`mfa.rs`, `session.rs`) per MFA_SESSION_MANDATE.md §8 — no crate exists in `ekos/crates/` today (only `ekos-kernel`, `ekosd`) |
| VV-SEC-007.2 | ⬜ | `MfaGuard::verify_startup()` — token read + HMAC-SHA256 verify (MFA_SESSION_MANDATE.md §5); portable logic, only the token *path* (`/run/ekos-auth/...`, `/proc/self/sessionid`) is Linux-specific |
| VV-SEC-007.3 | 🧊 | PAM exec script `ekos-mfa-token-writer` (§3-4) |
| VV-SEC-007.4 | 🧊 | `/etc/pam.d/sshd` integration on a real SSH host |
| VV-SEC-007.5 | ⬜ | Periodic recheck task (5-min tick) + graceful shutdown on expiry/revocation (§5, `recheck()`) |
| VV-SEC-007.6 | ⬜ | Integrate `MfaGuard::verify_startup()` into `ekosd` and `vvctl` `main()` (§5 integration snippets) |
| VV-SEC-007.7 | ⬜ | `mfa_enforcement` TOML config (`enabled`/`disabled`/`warn-only`, §7) — blocked by VV-ENG-017 |
| VV-SEC-007.8 | 🧊 | Test on an AWS EC2 Linux box: full SSH → TOTP → token-write → binary-starts flow |
| VV-SEC-007.9 | 🧊 | Test on an AWS EC2 Linux box: deleting the token file triggers graceful shutdown within the 5-minute recheck window |

## VV-SEC-008 — SEV-SNP / MKTME per-tenant memory encryption  🧊
**Spec:** feature matrix row only — no dedicated design doc found in `docs/vision` or `docs/security`; scope inferred from the row description
**Blocked by:** VV-ENG-020 (multi-tenancy shard groups — this only matters once tenants share a host)
**Env:** CPU with AMD SEV-SNP or Intel TME/MKTME/TDX support — not present in the current dev environment

| Task | Status | Detail |
|---|---|---|
| VV-SEC-008.1 | ⬜ | `MemoryEncryptionBackend` trait design with a no-op default (portable) |
| VV-SEC-008.2 | 🧊 | CPU feature detection (`cpuid` SEV-SNP / MKTME bits) — needs supporting silicon |
| VV-SEC-008.3 | 🧊 | Per-tenant encrypted memory region allocation |
| VV-SEC-008.4 | 🧊 | Attestation report validation |
| VV-SEC-008.5 | 🧊 | Test on SEV-SNP-capable host: memory ciphertext is opaque to the hypervisor |

## VV-SEC-009 — Behavioral fingerprinting / anomaly scoring  ⬜
**Spec:** PREDICTIVE_PREFETCH_ENGINE.md §9 ("shares CPU affinity with behavioral fingerprinting... on the observer core group")
**Blocked by:** VV-PRE-001 (dependency graph — fingerprinting reuses the same per-PK access signal), VV-AUT-002 (evidence correlation engine consumes the anomaly score)
**Env:** none

| Task | Status | Detail |
|---|---|---|
| VV-SEC-009.1 | ⬜ | `AccessFingerprint`: per-session PK access-sequence hash |
| VV-SEC-009.2 | ⬜ | Baseline model of "normal" access sequences per user/role |
| VV-SEC-009.3 | ⬜ | Anomaly score = deviation from baseline |
| VV-SEC-009.4 | ⬜ | Alert integration (score above threshold → admin panel flag) — blocked by VV-CTL-011 |
| VV-SEC-009.5 | ⬜ | Test: a synthetic anomalous access sequence scores above threshold |
| VV-SEC-009.6 | ⬜ | Test: false-positive rate on normal production-shaped synthetic traffic stays under an agreed bound |

---

# Layer 14 — Prefetch Intelligence

*Owned by EKOS (`ekos/crates/ekos-prefetch/`, per PREDICTIVE_PREFETCH_ENGINE.md §9), consuming VectorVolt
primitives. All 8 features are ⬜ — none are hardware- or research-gated in themselves; they are blocked
purely on other software (Cuckoo filter aside, which is already ✅).*

## VV-PRE-001 — Markov transaction dependency graph  ⬜
**Spec:** PREDICTIVE_PREFETCH_ENGINE.md §2
**Blocked by:** VV-OBS-004 (per-op histograms feed the access-timing signal), VV-TXN-016 (`TransactionContext` — needed to emit a per-transaction PK-access sequence)
**Env:** none

| Task | Status | Detail |
|---|---|---|
| VV-PRE-001.1 | ⬜ | `TransactionDependencyGraph` adjacency-list layout keyed by PK (`DashMap<u64, Vec<(u64,f32)>>` per §2) |
| VV-PRE-001.2 | ⬜ | Add `dashmap` dependency to a new `ekos-prefetch` crate — not present anywhere in the workspace today (grep confirms) |
| VV-PRE-001.3 | ⬜ | Edge-weight increment on transaction PK-access-sequence emission (`Customer(999) → Order(101)`: weight += 1.0) |
| VV-PRE-001.4 | ⬜ | Hook transaction commit path to emit the access sequence — blocked by VV-TXN-016 |
| VV-PRE-001.5 | ⬜ | Crate scaffold per §9: `dependency_graph.rs`, `prefetch_loop.rs`, `speculative.rs`, `hot_subgraph.rs`, `negative_cache.rs` |
| VV-PRE-001.6 | ⬜ | Test: edge weight converges to the expected probability under synthetic Markov-generated traffic |
| VV-PRE-001.7 | ⬜ | Test: `top_predictions(window_ms)` returns correctly ranked PKs |

## VV-PRE-002 — Temporal decay (5-min half-life)  ⬜
**Spec:** PREDICTIVE_PREFETCH_ENGINE.md §2 ("The Decay Factor")
**Blocked by:** VV-PRE-001
**Env:** none

| Task | Status | Detail |
|---|---|---|
| VV-PRE-002.1 | ⬜ | `decay_rate` field + 5-minute background timer |
| VV-PRE-002.2 | ⬜ | Apply `weight *= decay_rate` (default 0.5) across all edges per tick |
| VV-PRE-002.3 | ⬜ | Config: decay rate + interval via TOML — blocked by VV-ENG-017 |
| VV-PRE-002.4 | ⬜ | Decay-rate auto-tuning based on workload variability (explicitly placed at "Phase 5" in §10 — scheduled, not research) |
| VV-PRE-002.5 | ⬜ | Test: edge weight halves after exactly one 5-minute tick |
| VV-PRE-002.6 | ⬜ | Test: edges below a floor threshold are pruned (bounds graph memory growth) |

## VV-PRE-003 — Tiered promotion (>95%→mlock, 60-95%→DRAM, <60%→NVMe)  ⬜
**Spec:** PREDICTIVE_PREFETCH_ENGINE.md §3
**Blocked by:** VV-HAL-033 (`mlock()` tier-1 pinning), VV-HAL-031 (2MB huge pages), VV-PRE-001
**Env:** none

| Task | Status | Detail |
|---|---|---|
| VV-PRE-003.1 | ⬜ | `HugePagePool` (Tier 1) + `BufferPool` (Tier 2) wrapper types — blocked by VV-HAL-031/033 |
| VV-PRE-003.2 | ⬜ | Probability → tier classification function (0.95 / 0.60 thresholds per §3) |
| VV-PRE-003.3 | ⬜ | Tier-1 promotion: `tier1.pin(pk)` via `mlock` — blocked by VV-HAL-033 |
| VV-PRE-003.4 | ⬜ | Tier-2 promotion: `tier2.prefetch(pk)` async DRAM load |
| VV-PRE-003.5 | ⬜ | Config: promotion thresholds tunable via TOML — blocked by VV-ENG-017 |
| VV-PRE-003.6 | ⬜ | Test: a PK above 95% probability lands in Tier 1 within one prefetch tick |
| VV-PRE-003.7 | ⬜ | Test: demotion occurs once probability decays below a tier threshold |

## VV-PRE-004 — Background promote/demote loop (50ms tick)  ⬜
**Spec:** PREDICTIVE_PREFETCH_ENGINE.md §3 (`prefetch_loop` code sample)
**Blocked by:** VV-PRE-001, VV-PRE-003, VV-HAL-024 (CPU core pinning), VV-HAL-025 (observer core-group role map)
**Env:** none

| Task | Status | Detail |
|---|---|---|
| VV-PRE-004.1 | ⬜ | `prefetch_loop` async task, 50ms `tokio::time::sleep` tick, exactly as sketched in §3 |
| VV-PRE-004.2 | ⬜ | Pin the loop to a dedicated background CPU core ("observer" group) — blocked by VV-HAL-024 |
| VV-PRE-004.3 | ⬜ | Budget cap (`prefetch_budget`: max pages promoted per tick) |
| VV-PRE-004.4 | ⬜ | Backpressure when Tier-2 pool is full (skip rather than block) |
| VV-PRE-004.5 | ⬜ | Test: loop respects `prefetch_budget` under a high-volume prediction stream |
| VV-PRE-004.6 | ⬜ | Test: loop does not starve OLTP cores — blocked by VV-HAL-026 (writer/reader core isolation invariant) |

## VV-PRE-005 — Per-query AST speculation on spare core  ⬜
**Spec:** PREDICTIVE_PREFETCH_ENGINE.md §4
**Blocked by:** VV-QRY-018 (SQL parser/AST — today's queries are direct API calls with no parsed AST to inspect), VV-PRE-001, VV-HAL-025
**Env:** none

| Task | Status | Detail |
|---|---|---|
| VV-PRE-005.1 | ⬜ | Extract PKs from query predicates — blocked by VV-QRY-018 (no AST exists yet; direct-API predicate inspection is a fallback worth prototyping first) |
| VV-PRE-005.2 | ⬜ | Dependency-graph lookup per extracted PK — blocked by VV-PRE-001 |
| VV-PRE-005.3 | ⬜ | Low-priority speculative read thread launched on a spare core — blocked by VV-HAL-025 |
| VV-PRE-005.4 | ⬜ | Speculative-promotion rule: 2 actual accesses within 1s → Tier 1 (per §4) — blocked by VV-PRE-003 |
| VV-PRE-005.5 | ⬜ | Test: speculative load completes before the next client request arrives, in a synthetic ~200µs-gap benchmark (mirrors the §4 worked example) |
| VV-PRE-005.6 | ⬜ | Test: speculation never evicts a higher-confidence Tier-1 entry |

## VV-PRE-006 — Hot subgraph pre-computation (top 1%)  ⬜
**Spec:** PREDICTIVE_PREFETCH_ENGINE.md §5
**Blocked by:** VV-IDX-013 (CSR graph adjacency doesn't exist yet — nothing to pre-join from), VV-QRY-015 (graph traversal), VV-PRE-001
**Env:** none

| Task | Status | Detail |
|---|---|---|
| VV-PRE-006.1 | ⬜ | `HotSubgraph` struct: `root_node_id`, `depth`, `nodes`, `edges`, `last_access` (§5) |
| VV-PRE-006.2 | ⬜ | Top-1%-by-access-frequency node identification from the dependency graph — blocked by VV-PRE-001 |
| VV-PRE-006.3 | ⬜ | Subgraph builder: depth-2 traversal → contiguous blob — blocked by VV-IDX-013 |
| VV-PRE-006.4 | ⬜ | Store in a dedicated Hot Graph Buffer (Tier 1, mlock'd) — blocked by VV-HAL-033 |
| VV-PRE-006.5 | ⬜ | Invalidation on edge INSERT/DELETE within a tracked subgraph — blocked by VV-QRY-016 (`CREATE GRAPH_EDGE`) |
| VV-PRE-006.6 | ⬜ | Async rebuild after invalidation |
| VV-PRE-006.7 | ⬜ | Test: querying a hot subgraph avoids CSR pointer-chasing (compare op count vs cold traversal) |
| VV-PRE-006.8 | ⬜ | Test: a stale subgraph is correctly invalidated after an edge mutation |

## VV-PRE-007 — Negative caching via cuckoo filter  ⬜
**Spec:** PREDICTIVE_PREFETCH_ENGINE.md §6
**Blocked by:** none — VV-IDX-005 (Cuckoo Filter) is already ✅ in `vectorvolt-index`, making this the most immediately startable feature in Layer 14
**Env:** none

| Task | Status | Detail |
|---|---|---|
| VV-PRE-007.1 | ⬜ | `negative_cache.rs`: query the existing Cuckoo Filter (VV-IDX-005 ✅) before issuing a speculative prefetch |
| VV-PRE-007.2 | ⬜ | Mark filter-confirmed-absent predictions as "known miss," skip the prefetch entirely (per §6) |
| VV-PRE-007.3 | ⬜ | Metric: prefetch bandwidth saved by negative caching — blocked by VV-OBS-005 |
| VV-PRE-007.4 | ⬜ | Test: a PK confirmed absent via the filter never triggers a Tier-2 load |
| VV-PRE-007.5 | ⬜ | Test: a filter false-positive still falls through to the normal miss path without corrupting correctness |

## VV-PRE-008 — Working-set boundary policy (5%/40%/rest)  ⬜
**Spec:** PREDICTIVE_PREFETCH_ENGINE.md §7
**Blocked by:** VV-PRE-003, VV-CTL-011 (admin panel to surface working-set coverage per §7)
**Env:** none

| Task | Status | Detail |
|---|---|---|
| VV-PRE-008.1 | ⬜ | Working-set size estimator (total predicted PKs vs. available DRAM) |
| VV-PRE-008.2 | ⬜ | Policy branch: full-residency (working set ≤ DRAM) vs. constrained 5%/40%/rest split (§7) |
| VV-PRE-008.3 | ⬜ | Admin panel: Tier 1/2/3 coverage % of the last hour's queries (§7) — blocked by VV-CTL-011 |
| VV-PRE-008.4 | ⬜ | Hardware-sizing recommendation surfaced from coverage stats — blocked by VV-AUT-002 (ARP correlates resource vs. code issues per §8) |
| VV-PRE-008.5 | ⬜ | Test: policy falls back to full Tier-2 residency when the working set fits in DRAM |
| VV-PRE-008.6 | ⬜ | Test: 5%/40%/rest split is enforced under a synthetic working set that exceeds DRAM |

---

# Layer 15 — Projections / CQRS

## VV-PRJ-001 — `Projection` trait + Identity/KeyOnly/Filter/Pipeline  ✅
**Spec:** VECTORVOLT_STORAGE_TIERING.md Part 4
**Blocked by:** none
**Env:** none

| Task | Status | Detail |
|---|---|---|
| VV-PRJ-001.1 | ✅ | `Projection` trait definition — `vectorvolt/crates/vectorvolt-store/src/projection.rs:45-47` |
| VV-PRJ-001.2 | ✅ | `IdentityProjection` — `projection.rs:50-56` |
| VV-PRJ-001.3 | ✅ | `KeyOnlyProjection` (zeroes value, existence checks) — `projection.rs:59-68` |
| VV-PRJ-001.4 | ✅ | `FilterProjection<F>` (predicate-based) — `projection.rs:71-82` |
| VV-PRJ-001.5 | ✅ | `PipelineProjection` (chained, short-circuiting via `try_fold`) — `projection.rs:168-174` |
| VV-PRJ-001.6 | ✅ | Unit tests: identity/filter/pipeline behavior — `projection.rs:181-221` |
| VV-PRJ-001.7 | ⬜ | JSON field-extraction / schema-projection impl for EKOS Knowledge Object queries — named as deferred "Future work (Phase 4+)" in the module doc comment, `projection.rs:24-29` |
| VV-PRJ-001.8 | ⬜ | Redaction projection impl (also named in the same doc comment) |

## VV-PRJ-002 — `ProjectionDispatcher`  ✅
**Spec:** VECTORVOLT_STORAGE_TIERING.md Part 4
**Blocked by:** none
**Env:** none

| Task | Status | Detail |
|---|---|---|
| VV-PRJ-002.1 | ✅ | `scan` / `scan_with` core dispatch over `KvStore::scan_raw` — `projection.rs:98-122` |
| VV-PRJ-002.2 | ✅ | `scan_prefix` / `scan_prefix_limit` convenience wrappers — `projection.rs:125-136` |
| VV-PRJ-002.3 | ✅ | `count` / `keys` convenience via `KeyOnlyProjection` — `projection.rs:139-150` |
| VV-PRJ-002.4 | ⬜ | Bloom filter skip for point lookups — named as deferred "Future work (Phase 4+)" in `projection.rs:27` |
| VV-PRJ-002.5 | ⬜ | Vector-similarity projection for EKOS KO queries — same doc comment, `projection.rs:28`; blocked by VV-QRY-020, VV-IDX-012 |
| VV-PRJ-002.6 | ⬜ | Test: dispatcher merge order is newest-wins across memtable + N SSTables under adversarial key overlap |

## VV-PRJ-003 — Projection workers reading WAL as event log  ⬜
**Spec:** VECTORVOLT_STORAGE_TIERING.md Part 4 (`ProjectionWorker` code sample)
**Blocked by:** VV-WAL-014 (WAL-as-event-log for projections / LSN offsets — WAL today has per-frame sequence numbers but no dedicated event-log read API, confirmed by grep of `vectorvolt-wal/src`)
**Env:** none

| Task | Status | Detail |
|---|---|---|
| VV-PRJ-003.1 | ⬜ | `ProjectionWorker` struct: `worker_id`, `shard_id`, `current_offset`, `batch_size`, `sql_writer` |
| VV-PRJ-003.2 | ⬜ | `wal_reader.read_events(shard_id, offset, batch_size)` API — blocked by VV-WAL-014 |
| VV-PRJ-003.3 | ⬜ | `tick()` async loop: read batch → `bulk_insert` → advance offset |
| VV-PRJ-003.4 | ⬜ | `SqlWriter` trait + at least one in-process concrete implementation |
| VV-PRJ-003.5 | ⬜ | Worker scheduling — one task per registered projection |
| VV-PRJ-003.6 | ⬜ | Test: worker resumes from the last committed offset after a restart — blocked by VV-PRJ-004 |
| VV-PRJ-003.7 | ⬜ | Test: projection output eventually matches the source event stream under sustained write load |

## VV-PRJ-004 — Per-worker LSN offset tracking  ⬜
**Spec:** VECTORVOLT_STORAGE_TIERING.md Part 4 (`projection_offsets` table example)
**Blocked by:** VV-PRJ-003
**Env:** none

| Task | Status | Detail |
|---|---|---|
| VV-PRJ-004.1 | ⬜ | `projection_offsets` durable table: `worker_id`, `shard_id`, `last_processed_lsn` |
| VV-PRJ-004.2 | ⬜ | Durable offset writes (must survive crash — WAL-backed or fsync'd) |
| VV-PRJ-004.3 | ⬜ | `offset_store.update()` called after every successful `bulk_insert` |
| VV-PRJ-004.4 | ⬜ | Startup recovery: resume from the last durable offset |
| VV-PRJ-004.5 | ⬜ | Test: offset persists across a process restart |
| VV-PRJ-004.6 | ⬜ | Test: a crash between `bulk_insert` and offset update re-processes safely (at-least-once, idempotent projection writes) |

## VV-PRJ-005 — Eventual consistency guarantee (never sync on command path)  ⬜
**Spec:** VECTORVOLT_STORAGE_TIERING.md Part 4; LD-PROJ locked decision
**Blocked by:** VV-PRJ-003
**Env:** none

| Task | Status | Detail |
|---|---|---|
| VV-PRJ-005.1 | ⬜ | Architectural invariant: the write (command) path never blocks on projection completion |
| VV-PRJ-005.2 | ⬜ | Async fire-and-forget wakeup from WAL append to waiting projection workers |
| VV-PRJ-005.3 | ⬜ | Document + assert the bounded consistency-window (target lag under normal load) |
| VV-PRJ-005.4 | ⬜ | Test: write throughput is unaffected by a paused or slow projection worker |
| VV-PRJ-005.5 | ⬜ | Test: projection converges to source-of-truth state within the bounded lag under load |

## VV-PRJ-006 — Dead Letter Queue + worker pause + alert  ⬜
**Spec:** VECTORVOLT_STORAGE_TIERING.md Part 5; LD-DLQ locked decision
**Blocked by:** VV-PRJ-003
**Env:** none

| Task | Status | Detail |
|---|---|---|
| VV-PRJ-006.1 | ⬜ | `projection_dlq` schema: `worker_id`, `shard_id`, `event_lsn`, `event_payload`, `error`, `failed_at` (Part 5) |
| VV-PRJ-006.2 | ⬜ | Retry-then-DLQ logic (N retries before writing to DLQ) |
| VV-PRJ-006.3 | ⬜ | Worker pauses (stops consuming new events) on DLQ write — never silently skips (Part 5 explicit requirement) |
| VV-PRJ-006.4 | ⬜ | Alert emission ("ProjectionWorker 'x' paused — events in DLQ") — blocked by VV-OBS-005/VV-CTL-011 |
| VV-PRJ-006.5 | ⬜ | Test: worker pauses exactly once per unrecoverable event, with no silent skip |
| VV-PRJ-006.6 | ⬜ | Test: DLQ entry carries enough information to diagnose and replay |

## VV-PRJ-007 — `vvctl projection replay-dlq`  ⬜
**Spec:** VECTORVOLT_STORAGE_TIERING.md Part 5
**Blocked by:** VV-PRJ-006
**Env:** none

| Task | Status | Detail |
|---|---|---|
| VV-PRJ-007.1 | ⬜ | `vvctl projection replay-dlq --worker <id> --shard <n>` command |
| VV-PRJ-007.2 | ⬜ | Replay DLQ entries in LSN order |
| VV-PRJ-007.3 | ⬜ | Resume normal consumption after a successful replay |
| VV-PRJ-007.4 | ⬜ | `vvctl projection status` (paused workers + DLQ depth) |
| VV-PRJ-007.5 | ⬜ | Test: `replay-dlq` clears the queue and the worker resumes at the correct offset |

## VV-PRJ-008 — Parallel SSTable reads  ⬜
**Spec:** deferred directly in code — `vectorvolt-store/src/projection.rs:26` doc comment: "Parallel SSTable reads (one Tokio task per SSTable for large scans)"
**Blocked by:** none — pure performance work on the existing `scan_raw` path
**Env:** none

| Task | Status | Detail |
|---|---|---|
| VV-PRJ-008.1 | ⬜ | Spawn one Tokio task per live SSTable for range scans |
| VV-PRJ-008.2 | ⬜ | Merge results preserving newest-wins-on-key semantics (must match today's serial merge exactly) |
| VV-PRJ-008.3 | ⬜ | Bound parallelism to avoid task explosion on many small SSTables |
| VV-PRJ-008.4 | ⬜ | Benchmark: parallel vs. serial scan latency across N SSTables — blocked by VV-TST-007 (criterion suite) |
| VV-PRJ-008.5 | ⬜ | Test: parallel scan produces results identical to the serial `scan_raw` oracle |

## VV-PRJ-009 — WASM-sandboxed projections  ⬜
**Spec:** VECTORVOLT_STORAGE_TIERING.md Part 4 context; feature matrix row
**Blocked by:** VV-PRJ-001 (base `Projection` trait — already ✅, this is the only real unblock in the feature)
**Env:** none

| Task | Status | Detail |
|---|---|---|
| VV-PRJ-009.1 | ⬜ | Evaluate `wasmtime` embedding for user-supplied projection logic |
| VV-PRJ-009.2 | ⬜ | Define a WASM ABI mapping to `Projection::apply` semantics |
| VV-PRJ-009.3 | ⬜ | Sandboxed resource limits (fuel metering, memory cap) |
| VV-PRJ-009.4 | ⬜ | `WasmProjection` impl of the `Projection` trait |
| VV-PRJ-009.5 | ⬜ | Test: a malicious/infinite-loop WASM module is terminated by the fuel limit |
| VV-PRJ-009.6 | ⬜ | Test: a WASM projection produces the same output as an equivalent native `FilterProjection` |

---

# Layer 16 — Hardware Acceleration

## VV-HW-001 — Intel QAT (AES/DEFLATE/SHA offload) + auto-detect  🧊
**Spec:** HARDWARE_FRONTIER.md §3 — scheduled Phase 3, "available now... should be integrated"
**Blocked by:** VV-SEC-005 (shared `EncryptionBackend` trait)
**Env:** Intel Xeon Scalable with QAT; owner does not have a QAT-equipped host today

| Task | Status | Detail |
|---|---|---|
| VV-HW-001.1 | ⬜ | `qat` feature flag + survey of `qatzip`/`intel-ipsec-mb` Rust bindings — desk research, no hardware required |
| VV-HW-001.2 | 🧊 | `QatBackend` impl: AES-256-GCM, DEFLATE compression, SHA-256 HMAC offload |
| VV-HW-001.3 | 🧊 | Auto-detect QAT presence at startup (`qat::is_available()` per HARDWARE_FRONTIER.md §3 sample) |
| VV-HW-001.4 | ⬜ | Software fallback path (exercised without hardware) — shares code with VV-SEC-005.4 |
| VV-HW-001.5 | 🧊 | Test on a QAT-equipped host: throughput vs. the software baseline |

## VV-HW-002 — Intel DSA (memcpy/CRC offload)  🧊
**Spec:** HARDWARE_FRONTIER.md §3 — scheduled Phase 4
**Blocked by:** VV-STO-017 (Copy-on-Write snapshots — the buffer-pool bulk-fill use case)
**Env:** Intel Xeon Scalable with DSA + Linux `idxd` driver; owner does not have DSA hardware today

| Task | Status | Detail |
|---|---|---|
| VV-HW-002.1 | ⬜ | `DataMovementBackend` trait (bulk copy, CRC-32C offload) |
| VV-HW-002.2 | 🧊 | DSA descriptor submission via the Linux `idxd` driver |
| VV-HW-002.3 | ⬜ | Wire into buffer-pool bulk fills for backup snapshot operations — blocked by VV-STO-017 |
| VV-HW-002.4 | ✅ | Software fallback already available: hardware CRC-32C via `_mm_crc32_u64` — `vectorvolt/crates/vectorvolt-hal` (S1a delivered VV-HAL-017, measured 44.6× vs. bitwise scalar) |
| VV-HW-002.5 | 🧊 | Test on a DSA-equipped host: bulk-copy throughput vs. software `memcpy` |

## VV-HW-003 — ARM as default deployment target  ⬜
**Spec:** HARDWARE_FRONTIER.md §5 — "not a frontier... should be the default deployment target... available today on cloud"
**Blocked by:** VV-HAL-008 (ARM NEON/SVE kernels — explicitly deferred in the S1a sprint log: "cannot execute aarch64 here... aarch64 reports `Scalar` today")
**Env:** ARM hardware (Graviton/Axion) for verification; cross-compilation needs no hardware

| Task | Status | Detail |
|---|---|---|
| VV-HW-003.1 | ⬜ | NEON kernel implementations mirroring the 6 existing AVX2 kernels — blocked by VV-HAL-008 |
| VV-HW-003.2 | ⬜ | `aarch64-unknown-linux-gnu`/`musl` CI build target (cross-compile only, no ARM hardware needed) |
| VV-HW-003.3 | ⬜ | `aarch64-apple-darwin` build target for local Apple Silicon dev testing |
| VV-HW-003.4 | 🧊 | Graviton/Axion NEON correctness test suite — needs real ARM hardware (e.g. an EC2 Graviton instance) |
| VV-HW-003.5 | ⬜ | Update `SimdTier::detect()` to select NEON on aarch64 once VV-HAL-008 lands |
| VV-HW-003.6 | 🧊 | Benchmark: NEON vs. AVX2 vs. scalar on a Graviton instance |
| VV-HW-003.7 | ⬜ | "Deploy on ARM by default" runbook doc |

## VV-HW-004 — CXL 3.0 fabric-attached memory  🔬
**Spec:** HARDWARE_FRONTIER.md §1 — explicitly "nobody has done this yet for a production OLTP engine," no phase number
**Blocked by:** VV-CLU-009 (Raft consensus — the `RaftConsensus` variant the `ConsensusBackend` trait must support today)
**Env:** CXL 3.0 multi-host memory-sharing hardware — "only now arriving (2025-2026)" per the source doc; none available

| Task | Status | Detail |
|---|---|---|
| VV-HW-004.1 | ⬜ | `ConsensusBackend` trait scaffold (`RaftConsensus` today / `CxlCasConsensus` future) — this readiness work is the doc's own explicit "VectorVolt Readiness Requirement" (§1) and is doable now with no CXL hardware |
| VV-HW-004.2 | 🔬 | Research spike: CXL 3.0 multi-host memory-sharing hardware availability and cost |
| VV-HW-004.3 | 🔬 | CAS-as-consensus protocol design (replacing the 3-RTT Prepare/OK/Commit sequence) |
| VV-HW-004.4 | 🔬 | Prototype on CXL-capable hardware once available — no schedule, no hardware, no algorithm chosen yet |

## VV-HW-005 — FPGA semantic constraint validation  🔬
**Spec:** HARDWARE_FRONTIER.md §2 — explicitly "$10,000+ per chip... not the right investment now"
**Blocked by:** VV-HAL-010 (SIMD range filter — the `SoftwareConstraints` default this trait must wrap first)
**Env:** Xilinx VU13P-class FPGA ($10k+) + hardware-engineering tooling — not available, not budgeted

| Task | Status | Detail |
|---|---|---|
| VV-HW-005.1 | ⬜ | `ConstraintBackend` trait scaffold (`SoftwareConstraints` default / `FpgaConstraints` future) — this is the doc's own explicit "VectorVolt Readiness Requirement" (§2) and needs no FPGA |
| VV-HW-005.2 | ⬜ | SIMD-vectorized `SoftwareConstraints` impl (NOT NULL/CHECK/UNIQUE/FK) reusing VV-HAL-010's range-filter kernel |
| VV-HW-005.3 | 🔬 | Research spike: FPGA vendor/tooling survey (Xilinx VU13P class) |
| VV-HW-005.4 | 🔬 | Prototype constraint bitstream for a financial/HFT-style deployment — no budget, no schedule, no chip |

---

# Layer 17 — Autonomy (EKOS, not VectorVolt core)

*Every feature in this layer depends on an LLM runtime, Linux namespaces, or both. The feature matrix
itself labels the layer "EKOS-adjacent, deferred until VectorVolt core is done." Decomposed only to the
depth that is useful — most tasks here are research/design spikes, not implementable day-units.*

## VV-AUT-001 — ARP autonomous remediation protocol  🔬
**Spec:** feature matrix row; PREDICTIVE_PREFETCH_ENGINE.md §8 (interaction with the pre-fetch engine)
**Blocked by:** VV-AUT-002, VV-AUT-003, VV-AUT-005
**Env:** needs an LLM runtime + Shadowbox; no schedule

| Task | Status | Detail |
|---|---|---|
| VV-AUT-001.1 | ⬜ | `ArpEvent` schema (anomaly classification, evidence references) — design-only, no hardware required |
| VV-AUT-001.2 | ⬜ | Integration point with the Pre-fetch Engine state to distinguish "resource issue" from "code bug" (PREDICTIVE_PREFETCH_ENGINE.md §8) — blocked by VV-PRE-001, VV-PRE-008 |
| VV-AUT-001.3 | 🔬 | Remediation-action taxonomy (patch synthesis / resource recommendation / no-op) |
| VV-AUT-001.4 | 🔬 | End-to-end detect → correlate → remediate → verify loop — blocked by VV-AUT-002, VV-AUT-003, VV-AUT-005 |

## VV-AUT-002 — Evidence correlation engine  🔬
**Spec:** feature matrix row only
**Blocked by:** VV-OBS-005, VV-OBS-006 (metrics/traces as evidence sources), VV-PRE-001 (prefetch-engine state as an evidence source)
**Env:** none beyond its blockers

| Task | Status | Detail |
|---|---|---|
| VV-AUT-002.1 | ⬜ | Evidence-source adapters: metrics, logs, traces, prefetch-engine state — blocked by VV-OBS-005/006, VV-PRE-001 |
| VV-AUT-002.2 | 🔬 | Correlation scoring across evidence sources for a single anomaly |
| VV-AUT-002.3 | 🔬 | Structured `AnomalyReport` output for ARP consumption |

## VV-AUT-003 — Shadowbox namespace isolation  🧊
**Spec:** RESEARCH_FRONTIER.md §1 (overlay KoStore referenced as already-existing infrastructure this depends on)
**Blocked by:** none additional
**Env:** Linux namespaces (`unshare`, `clone(CLONE_NEWNS|...)`, cgroups); owner has two AWS EC2 Linux boxes

| Task | Status | Detail |
|---|---|---|
| VV-AUT-003.1 | ⬜ | Namespace design: mount/pid/net isolation scope per patch-test run — design-only |
| VV-AUT-003.2 | 🧊 | Linux `unshare`/cgroups-based sandbox implementation |
| VV-AUT-003.3 | 🧊 | Overlay KoStore ("two views of the same data," RESEARCH_FRONTIER.md §1) |
| VV-AUT-003.4 | 🧊 | Test on an AWS EC2 Linux box: sandboxed process cannot affect host filesystem or network |

## VV-AUT-004 — CIP capability injection (WASM)  🔬
**Spec:** feature matrix row only
**Blocked by:** VV-PRJ-009 (shared `wasmtime` sandboxing infrastructure)
**Env:** none beyond its blocker

| Task | Status | Detail |
|---|---|---|
| VV-AUT-004.1 | ⬜ | Capability-manifest schema (what a patch/plugin may access) — design-only |
| VV-AUT-004.2 | 🔬 | WASM capability injection at instantiation, sharing infra with VV-PRJ-009 |
| VV-AUT-004.3 | 🔬 | Revocation / least-privilege enforcement |

## VV-AUT-005 — In-process LLM embedding  🔬
**Spec:** feature matrix row; PREDICTIVE_PREFETCH_ENGINE.md §8 context (ARP needs an LLM to synthesize patches)
**Blocked by:** none additional
**Env:** GPU inference benefits from the owner's Nvidia Tesla T4 — relevant here specifically, **not** to the database core

| Task | Status | Detail |
|---|---|---|
| VV-AUT-005.1 | ⬜ | Model-runtime survey: `llama.cpp` / `candle` / ONNX Runtime embeddable in a Rust binary — desk research, doable now |
| VV-AUT-005.2 | 🧊 | GPU inference path targeting the owner's Tesla T4 (CUDA build of `candle` or `llama.cpp`) — hardware is present, this is env-gated not unscheduled |
| VV-AUT-005.3 | 🔬 | In-process embedding API (`ekosd` calls the model with no external service) — architecture undecided |
| VV-AUT-005.4 | ⬜ | Resource isolation from OLTP cores (inference must not steal write-path cores) — blocked by VV-HAL-025 |
| VV-AUT-005.5 | 🧊 | Test on the Tesla T4 box: inference latency budget vs. OLTP core interference |

## VV-AUT-006 — Self-upgrading core binary (kexec fd handoff)  🔬
**Spec:** feature matrix row only
**Blocked by:** none additional
**Env:** Linux `kexec`; no schedule

| Task | Status | Detail |
|---|---|---|
| VV-AUT-006.1 | 🔬 | `kexec` fd-handoff research spike (Linux-specific mechanism) |
| VV-AUT-006.2 | 🔬 | Live binary swap without dropping open connections (fd-passing over a Unix socket as a fallback design) |
| VV-AUT-006.3 | 🔬 | Rollback-on-failure safety net |

## VV-AUT-007 — Dependency self-upgrade (Class A/B/C)  🔬
**Spec:** feature matrix row only
**Blocked by:** VV-CTL-014 (HITL Index Advisor — the approval-UX pattern this reuses for Class B/C)
**Env:** none beyond its blocker

| Task | Status | Detail |
|---|---|---|
| VV-AUT-007.1 | ⬜ | Classification scheme: Class A (patch, auto) / B (minor, reviewed) / C (major, blocking) — design-only |
| VV-AUT-007.2 | 🔬 | Automated Class A upgrade pipeline |
| VV-AUT-007.3 | 🔬 | Class B/C human-approval gate — blocked by VV-CTL-014 |

## VV-AUT-008 — Double-write execution  🔬
**Spec:** RESEARCH_FRONTIER.md §1 — explicitly "Phase 5+ after Shadowbox overlay architecture is stable," no committed schedule
**Blocked by:** VV-AUT-003 (Shadowbox overlay KoStore reused for the write-path variant)
**Env:** none beyond its blocker

| Task | Status | Detail |
|---|---|---|
| VV-AUT-008.1 | 🔬 | Read-only-path double-write prototype (old code + new code, compare outputs, per RESEARCH_FRONTIER.md §1) |
| VV-AUT-008.2 | 🔬 | Overlay-KoStore reuse for the constrained write-path case (pure-function patches only) — blocked by VV-AUT-003 |
| VV-AUT-008.3 | 🔬 | Atomic pointer-swap after N consecutive identical-output transactions |

## VV-AUT-009 — Retroactive rollback  🔬
**Spec:** RESEARCH_FRONTIER.md §2 — explicitly "No phase placement... research item," conflict-scheduling stated to be NP-hard in the general case
**Blocked by:** VV-TXN-017 (`TransactionPriority` QoS — the only currently-considered safe scope is Priority-4-only)
**Env:** none beyond its blocker

| Task | Status | Detail |
|---|---|---|
| VV-AUT-009.1 | 🔬 | Append-only undo log retaining pre-images forever — storage-cost feasibility study |
| VV-AUT-009.2 | 🔬 | Constrained Priority-4-only 10ms retroactive window (the only scope RESEARCH_FRONTIER.md §2 considers currently safe) — blocked by VV-TXN-017 |
| VV-AUT-009.3 | 🔬 | Conflict-scheduling research — explicitly NP-hard in the general case per the source doc; no algorithm chosen |

## VV-AUT-010 — Knowledge compiler  🔬
**Spec:** RESEARCH_FRONTIER.md §4 — explicitly "Not yet... a Phase 7+ concern once the individual pieces are proven"
**Blocked by:** VV-PRE-006 (hot-subgraph pre-computation), VV-CTL-014 (HITL Index Advisor), VV-STO-008 (MVCC/partition compaction) — the three "informal instances" the doc says must mature first
**Env:** none beyond its blockers

| Task | Status | Detail |
|---|---|---|
| VV-AUT-010.1 | ⬜ | Survey the three existing informal instances (prefetch hot-subgraph, HITL index advisor, compaction) as prior art — pure documentation/design task, doable now |
| VV-AUT-010.2 | 🔬 | Unified cost-model design spike |
| VV-AUT-010.3 | 🔬 | Dead-knowledge-elimination pass prototype — blocked by VV-AUT-010.2 |
