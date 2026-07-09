# Research Frontier
## VectorVolt + EKOS — Ideas Beyond the Current Phase Roadmap

**Status:** Research Capture — No implementation schedule. Document exists to record ideas
before they are forgotten and to inform future architecture decisions.
**Captured:** 2026-07-09

These are not features. They are research directions. Some may never be implemented.
Some may reshape the architecture if they prove feasible.

---

## 1. Double-Write Execution (Live Parallel Validation)

### The Idea

Instead of the current Shadowbox model (patch runs in isolation, production runs separately),
run **both** the old and new code paths on every live production transaction simultaneously:

```
Incoming transaction
        ↓
    ┌───┴───┐
    │       │
OLD CODE   NEW CODE (patched version)
runs       also runs (result discarded, not returned to client)
    │       │
    └───┬───┘
        │
   Compare outputs:
   If identical → new code is correct so far
   If different → flag for ARP review
        │
   After N transactions with identical output:
   Atomic pointer swap → new code becomes the live code
   Old code path dropped
```

### Why This Is Stronger Than Shadowbox

Shadowbox validates against replayed historical traffic. Double-write validates against
**live traffic in real time** — including the exact timing, concurrency patterns, and edge
cases that actually occur in production. No simulation, no replay lag.

### The Constraint That Makes It Hard

Both code paths must run against the same data state. If the old code path mutates data
(writes), running the new code path on the same mutation is not valid — the new path would
see a double-write. This limits double-write execution to:
- Read-only code paths (safe — no mutation conflict)
- Write paths where the new code is a pure function of the input (no side effects besides
  the KoStore write, which both paths write to the same shadow overlay)

For write-path patches, the current Shadowbox model remains more appropriate.

### When This Becomes Feasible

Phase 5+ after Shadowbox overlay architecture is stable. The overlay KoStore already
handles the "two views of the same data" problem. Double-write execution is the online
version of the same concept.

---

## 2. Retroactive Rollback (Time-Travel Transactions)

### The Idea

Today, once a transaction commits, it is permanently part of history. ACID's "D" (Durability)
means exactly this.

The frontier concept: **what if a committed transaction could be retroactively invalidated
and re-executed** when a higher-priority transaction arrives that would have changed the outcome?

```
T1 commits at t=100: balance = 500 - 200 = 300
T2 commits at t=105: balance = 300 - 400 = -100  (overdraft! business rule violation)

Retroactive system at t=106:
  Rewind to t=99
  Execute T2 (the higher-priority, larger transaction) FIRST: balance = 500 - 400 = 100
  Re-execute T1 in the new history: balance = 100 - 200 = -100 → T1 now ABORTS
  Result: T2 succeeds, T1 correctly fails for insufficient funds
```

### Why Nobody Has Built This

- "Committed means committed" is the foundational contract of every OLTP database
- Implementing retroactive rollback requires:
  - An append-only undo log that retains pre-images forever (massive storage)
  - A scheduler that can identify conflicts across the causal graph
  - A consensus mechanism for deciding which transactions are "higher priority"
- The complexity of the scheduler is non-polynomial in the general case

### Where This Could Apply in VectorVolt

VectorVolt's Transaction QoS system (STORAGE_ENGINE_ADVANCED.md) already assigns
priorities. A limited form of retroactive rollback — only between Priority 1 and
Priority 4 transactions within a short window — might be achievable:

```
Priority 4 (telemetry) commits at t=100
Priority 1 (settlement) arrives at t=101, conflicts with t=100 commit
  → Within a 10ms retroactive window, Priority 4 is reverted
  → Priority 1 executes first, then Priority 4 re-executes on the new state
```

This is safe only for Priority 4 transactions where the application explicitly acknowledges
retryability at declaration time. Not a general mechanism.

**This is a research item.** No phase placement until the Priority QoS system is proven
and we have production data showing where priority conflicts actually occur.

---

## 3. Zero-Overhead Observability (Telemetry Embedded in MVCC Headers)

### The Problem

Every observability tool (eBPF, OpenTelemetry, Prometheus) steals CPU cycles. The more you
monitor, the slower the system. This is the observer effect at the software layer.

### The Idea

VectorVolt's MVCC version chain already stores, for every row:
- The transaction ID that wrote this version
- The timestamp
- The previous version pointer

**What if the MVCC row header also stored the execution fingerprint of the transaction that
wrote this version?**

```rust
pub struct MvccRowHeader {
    pub txn_id:       u64,
    pub timestamp_ns: u64,
    pub prev_version: Option<RowDescriptor>,
    // NEW: execution telemetry, stored inline
    pub latency_ns:   u32,     // 4 bytes — how long the write took
    pub tier_source:  u8,      // which memory tier served the read before this write
    pub latch_waits:  u8,      // how many times we waited for a latch (saturating at 255)
    pub index_hits:   u8,      // how many index lookups this txn performed
}
```

Total overhead: 7 extra bytes per row version. For a 1M-row Micro-Partition, this adds
7 MB of storage — acceptable.

### The Payoff

Now observability is **free**:
- To answer "how many rows were written with latch contention last hour?": scan the MVCC
  headers of rows written in the last hour — no separate trace infrastructure needed
- To answer "which tier served the most reads before writes?": same scan
- To answer "what's the p99 write latency for this table?": same scan

The data *is* the log. No separate tracing agent. No CPU overhead at write time beyond the
7-byte header fill (one cache-line store).

### What This Does NOT Replace

- Infra-level metrics (CPU, memory, network) — still need the OS/eBPF layer
- Cross-request tracing (which client triggered which chain) — still need trace IDs
- Error events — still need ObservabilityEvent structure

What it replaces: per-operation performance telemetry that today requires a separate
tracing pipeline consuming 3-8% of CPU.

**Phase placement:** Phase 3 (the MVCC header is being designed now — add these 7 bytes while
the format is still malleable. Retrofitting this into an existing header format later is much harder).

---

## 4. Knowledge Compiler (Long-Term Vision)

### The Idea from the Research

ChatGPT raised the question: what if knowledge can be optimized the way code is optimized?

```
Source code compilation:
  Dead-code elimination  → remove unused functions
  Constant folding       → evaluate compile-time expressions
  Inlining               → replace function calls with their bodies
  Loop unrolling         → replicate loop bodies for speed

Knowledge compilation:
  Dead-knowledge elimination → remove KOs no workflow ever reads
  Knowledge inlining         → denormalize frequently co-accessed KOs
  Knowledge folding          → pre-compute derived KOs instead of computing at query time
  Knowledge loop unrolling   → materialize recursive graph traversals
```

### Current Reality

EKOS already does some of these informally:
- The Predictive Pre-fetch Engine pre-computes Hot Subgraphs (a form of knowledge inlining)
- HITL Index Advisor suggests materializing derived views (a form of knowledge folding)
- The MVCC compaction process removes stale versions (a form of dead-knowledge elimination)

A formal "Knowledge Compiler" is the long-term architecture where these become a unified
pipeline with a shared cost model.

**Phase placement:** Not yet. The individual components (pre-fetch, index advisor, compaction)
are built independently in Phases 3–5. A unifying Knowledge Compiler abstraction is a
Phase 7+ concern once the individual pieces are proven.

---

*These are seeds, not plans.*
*The right time to implement a seed is when you have enough production data to know it solves a real problem.*
