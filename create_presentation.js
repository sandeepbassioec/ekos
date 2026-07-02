const pptxgen = require("pptxgenjs");

const pres = new pptxgen();
pres.layout = "LAYOUT_16x9";
pres.author = "VectorVolt Team";
pres.title = "VectorVolt — Sub-Microsecond OLTP Data Operating System";

// ═══════════════════════════════════════════
// COLOR PALETTE: Midnight Executive + Electric
// ═══════════════════════════════════════════
const C = {
  bg: "0B1120",        // Deep midnight navy
  bgCard: "131D35",    // Card background
  bgCardAlt: "1A2744", // Alternate card
  accent: "00B4D8",    // Electric cyan
  accent2: "0077B6",   // Deep blue
  accent3: "48CAE4",   // Light cyan
  white: "FFFFFF",
  gray: "8899AA",
  dimText: "6B7B8D",
  green: "00E676",
  orange: "FF9100",
  red: "FF5252",
  yellow: "FFD600",
  purple: "B388FF",
};

// Helper: fresh shadow factory
const cardShadow = () => ({ type: "outer", blur: 8, offset: 2, angle: 135, color: "000000", opacity: 0.3 });

// ═══════════════════════════════════════════
// SLIDE 1: TITLE
// ═══════════════════════════════════════════
let s1 = pres.addSlide();
s1.background = { color: C.bg };
// Top accent bar
s1.addShape(pres.shapes.RECTANGLE, { x: 0, y: 0, w: 10, h: 0.06, fill: { color: C.accent } });
// Title
s1.addText("VECTORVOLT", {
  x: 0.8, y: 1.2, w: 8.4, h: 1.0,
  fontSize: 52, fontFace: "Calibri", bold: true, color: C.white,
  charSpacing: 8, align: "left", margin: 0
});
s1.addText("Sub-Microsecond OLTP Data Operating System", {
  x: 0.8, y: 2.1, w: 8.4, h: 0.6,
  fontSize: 22, fontFace: "Calibri", color: C.accent3, align: "left", margin: 0
});
// Divider line
s1.addShape(pres.shapes.LINE, { x: 0.8, y: 2.9, w: 2.5, h: 0, line: { color: C.accent, width: 3 } });
// Subtitle chips
const chips = [".NET 10.0", "Zero-GC", "Kernel Bypass", "SIMD Accelerated"];
chips.forEach((label, i) => {
  s1.addShape(pres.shapes.RECTANGLE, {
    x: 0.8 + i * 2.2, y: 3.3, w: 2.0, h: 0.45,
    fill: { color: C.bgCard }, line: { color: C.accent2, width: 1 }
  });
  s1.addText(label, {
    x: 0.8 + i * 2.2, y: 3.3, w: 2.0, h: 0.45,
    fontSize: 11, fontFace: "Calibri", color: C.accent3, align: "center", valign: "middle", margin: 0
  });
});
// Version
s1.addText("Version 3.0  |  Research & PhD Project  |  April 2026", {
  x: 0.8, y: 4.8, w: 8.4, h: 0.4,
  fontSize: 11, fontFace: "Calibri", color: C.dimText, align: "left", margin: 0
});
// Bottom accent bar
s1.addShape(pres.shapes.RECTANGLE, { x: 0, y: 5.565, w: 10, h: 0.06, fill: { color: C.accent } });


// ═══════════════════════════════════════════
// HELPER: Standard slide header
// ═══════════════════════════════════════════
function addHeader(slide, title, subtitle) {
  slide.background = { color: C.bg };
  slide.addShape(pres.shapes.RECTANGLE, { x: 0, y: 0, w: 10, h: 0.06, fill: { color: C.accent } });
  slide.addText(title, {
    x: 0.6, y: 0.2, w: 8.8, h: 0.55,
    fontSize: 28, fontFace: "Calibri", bold: true, color: C.white, align: "left", margin: 0
  });
  if (subtitle) {
    slide.addText(subtitle, {
      x: 0.6, y: 0.72, w: 8.8, h: 0.35,
      fontSize: 13, fontFace: "Calibri", color: C.gray, align: "left", margin: 0
    });
  }
  // Page number placeholder
  slide.addText("", {
    x: 8.8, y: 5.2, w: 0.8, h: 0.3,
    fontSize: 9, fontFace: "Calibri", color: C.dimText, align: "right"
  });
}

// HELPER: Info card
function addCard(slide, x, y, w, h, title, bullets, accentColor) {
  accentColor = accentColor || C.accent;
  slide.addShape(pres.shapes.RECTANGLE, { x, y, w, h, fill: { color: C.bgCard }, shadow: cardShadow() });
  slide.addShape(pres.shapes.RECTANGLE, { x, y, w: 0.06, h, fill: { color: accentColor } });
  slide.addText(title, {
    x: x + 0.2, y: y + 0.08, w: w - 0.3, h: 0.35,
    fontSize: 13, fontFace: "Calibri", bold: true, color: accentColor, align: "left", margin: 0
  });
  const items = bullets.map((b, i) => ({
    text: b, options: { bullet: true, color: C.white, fontSize: 10.5, fontFace: "Calibri", breakLine: i < bullets.length - 1 }
  }));
  slide.addText(items, {
    x: x + 0.2, y: y + 0.42, w: w - 0.35, h: h - 0.52,
    valign: "top", margin: 0, paraSpaceAfter: 4
  });
}


// ═══════════════════════════════════════════
// SLIDE 2: VISION & PHILOSOPHY
// ═══════════════════════════════════════════
let s2 = pres.addSlide();
addHeader(s2, "Vision & Core Philosophy", "Eliminating 90% of traditional database overhead");

// Big stat callouts
const stats = [
  { val: "20 ns", label: "Engine Latency", color: C.green },
  { val: "0", label: "GC Pauses", color: C.accent },
  { val: "1M+", label: "Ops/Sec/Shard", color: C.orange },
  { val: "0", label: "Data Loss", color: C.yellow },
];
stats.forEach((st, i) => {
  const cx = 0.6 + i * 2.35;
  s2.addShape(pres.shapes.RECTANGLE, { x: cx, y: 1.3, w: 2.1, h: 1.1, fill: { color: C.bgCard }, shadow: cardShadow() });
  s2.addText(st.val, {
    x: cx, y: 1.32, w: 2.1, h: 0.65,
    fontSize: 30, fontFace: "Calibri", bold: true, color: st.color, align: "center", valign: "middle", margin: 0
  });
  s2.addText(st.label, {
    x: cx, y: 1.92, w: 2.1, h: 0.4,
    fontSize: 10, fontFace: "Calibri", color: C.gray, align: "center", valign: "middle", margin: 0
  });
});

// Four pillars
const pillars = [
  { t: "In-Memory First", d: "Disk is only for WAL durability. All domain logic executes in memory." },
  { t: "Single-Writer/Shard", d: "One thread owns one shard. Zero locks, zero contention, maximum throughput." },
  { t: "Zero-Copy Networking", d: "DPDK/io_uring bypass the OS kernel. Data flows NIC \u2192 user space directly." },
  { t: "Hardware Sympathy", d: "Cache-aligned, branchless, SIMD-accelerated. Written for the CPU, not against it." },
];
pillars.forEach((p, i) => {
  const cx = 0.6 + i * 2.35;
  s2.addShape(pres.shapes.RECTANGLE, { x: cx, y: 2.7, w: 2.1, h: 1.35, fill: { color: C.bgCard }, shadow: cardShadow() });
  s2.addShape(pres.shapes.RECTANGLE, { x: cx, y: 2.7, w: 2.1, h: 0.05, fill: { color: C.accent } });
  s2.addText(p.t, {
    x: cx + 0.12, y: 2.82, w: 1.86, h: 0.32,
    fontSize: 11.5, fontFace: "Calibri", bold: true, color: C.accent3, align: "left", margin: 0
  });
  s2.addText(p.d, {
    x: cx + 0.12, y: 3.15, w: 1.86, h: 0.82,
    fontSize: 9.5, fontFace: "Calibri", color: C.gray, align: "left", valign: "top", margin: 0
  });
});

// Bottom quote
s2.addText("\"Traditional databases spend 90% of time on overhead. VectorVolt inverts this.\"", {
  x: 0.6, y: 4.4, w: 8.8, h: 0.5,
  fontSize: 12, fontFace: "Calibri", italic: true, color: C.dimText, align: "center", margin: 0
});


// ═══════════════════════════════════════════
// SLIDE 3: ARCHITECTURE OVERVIEW
// ═══════════════════════════════════════════
let s3 = pres.addSlide();
addHeader(s3, "Architecture Overview", "7 components, ~120 core classes");

const layers = [
  { name: "HAL", desc: "MemoryArena, CacheLines, HugePages, NUMA, CPU Dispatch", color: "1565C0", classes: "~12" },
  { name: "Execution Kernel", desc: "ShardExecutor, RingBuffer, WaitStrategies, DAGs, Two-Phase Txn", color: "0277BD", classes: "~25" },
  { name: "Durability (WAL)", desc: "WalWriter, HashChain, GroupCommit, Snapshots, Projections", color: "00838F", classes: "~20" },
  { name: "Networking", desc: "DPDK, RDMA, eBPF/XDP, RIO, PTP, Cloud Adapters", color: "00695C", classes: "~14" },
  { name: "Read Fabric", desc: "DocumentStore, ColumnarStore, SIMD Scanner, Materialized Views", color: "2E7D32", classes: "~22" },
  { name: "Control Plane", desc: "Gateway, SDKs, Tenancy, Scaling, Observability, QoS, Versionstamps", color: "558B2F", classes: "~30" },
  { name: "CLI (vvctl)", desc: "Status, Query, Metrics, Dashboard (TUI), Migration, Ops", color: "827717", classes: "~15" },
];

layers.forEach((l, i) => {
  const ly = 1.2 + i * 0.58;
  // Layer bar
  s3.addShape(pres.shapes.RECTANGLE, { x: 0.6, y: ly, w: 1.8, h: 0.48, fill: { color: l.color } });
  s3.addText(l.name, {
    x: 0.7, y: ly, w: 1.7, h: 0.48,
    fontSize: 11, fontFace: "Calibri", bold: true, color: C.white, align: "left", valign: "middle", margin: 0
  });
  // Description
  s3.addShape(pres.shapes.RECTANGLE, { x: 2.5, y: ly, w: 6.2, h: 0.48, fill: { color: C.bgCard } });
  s3.addText(l.desc, {
    x: 2.6, y: ly, w: 5.4, h: 0.48,
    fontSize: 10, fontFace: "Calibri", color: C.gray, align: "left", valign: "middle", margin: 0
  });
  // Class count
  s3.addText(l.classes, {
    x: 8.1, y: ly, w: 0.55, h: 0.48,
    fontSize: 10, fontFace: "Calibri", bold: true, color: C.accent, align: "center", valign: "middle", margin: 0
  });
});

// Arrow label
s3.addText("Hot Path \u2192", {
  x: 0.6, y: 5.35, w: 2.0, h: 0.25,
  fontSize: 9, fontFace: "Calibri", color: C.dimText, align: "left", margin: 0
});


// ═══════════════════════════════════════════
// SLIDE 4: PERFORMANCE TARGETS
// ═══════════════════════════════════════════
let s4 = pres.addSlide();
addHeader(s4, "Performance Targets", "Latency budget breakdown for a single CRUD operation");

// Latency breakdown as stacked bars
const budget = [
  { op: "Ring buffer read", ns: 3, note: "Prefetched via lookahead" },
  { op: "Entity prefetch", ns: 0, note: "Issued during previous command" },
  { op: "Hash index lookup", ns: 5, note: "SIMD-accelerated, L1/L2" },
  { op: "State mutation", ns: 2, note: "Compiled YAML, branchless" },
  { op: "WAL batch append", ns: 5, note: "Group commit, HugePages" },
];

s4.addShape(pres.shapes.RECTANGLE, { x: 0.6, y: 1.2, w: 5.4, h: 3.4, fill: { color: C.bgCard }, shadow: cardShadow() });
s4.addText("Engine Internal Latency Budget", {
  x: 0.8, y: 1.28, w: 5.0, h: 0.35,
  fontSize: 13, fontFace: "Calibri", bold: true, color: C.accent, margin: 0
});

budget.forEach((b, i) => {
  const by = 1.75 + i * 0.5;
  const barW = Math.max(b.ns * 0.15, 0.08);
  s4.addText(b.op, {
    x: 0.8, y: by, w: 2.0, h: 0.35,
    fontSize: 10, fontFace: "Calibri", color: C.white, align: "right", valign: "middle", margin: 0
  });
  s4.addShape(pres.shapes.RECTANGLE, { x: 2.9, y: by + 0.07, w: barW, h: 0.22, fill: { color: C.accent } });
  s4.addText(`${b.ns} ns`, {
    x: 2.9 + barW + 0.1, y: by, w: 0.6, h: 0.35,
    fontSize: 10, fontFace: "Calibri", bold: true, color: C.green, align: "left", valign: "middle", margin: 0
  });
  s4.addText(b.note, {
    x: 3.6 + barW, y: by, w: 2.2, h: 0.35,
    fontSize: 8.5, fontFace: "Calibri", color: C.dimText, align: "left", valign: "middle", margin: 0
  });
});

// Total
s4.addShape(pres.shapes.LINE, { x: 0.8, y: 4.2, w: 5.0, h: 0, line: { color: C.accent2, width: 1 } });
s4.addText("Total: ~15 ns (best case)  |  ~22 ns (typical)", {
  x: 0.8, y: 4.25, w: 5.0, h: 0.3,
  fontSize: 11, fontFace: "Calibri", bold: true, color: C.green, align: "center", margin: 0
});

// End-to-end targets
const targets = [
  { label: "Engine Internal", val: "~20-50 ns", color: C.green },
  { label: "End-to-End (RDMA)", val: "< 1 \u03BCs", color: C.accent },
  { label: "End-to-End (DPDK)", val: "1-2 \u03BCs", color: C.accent3 },
  { label: "GC Pauses", val: "ZERO", color: C.yellow },
  { label: "Recovery Time", val: "Seconds", color: C.orange },
  { label: "Data Loss", val: "ZERO", color: C.green },
];
targets.forEach((t, i) => {
  const tx = 6.3;
  const ty = 1.3 + i * 0.58;
  s4.addShape(pres.shapes.RECTANGLE, { x: tx, y: ty, w: 3.2, h: 0.48, fill: { color: C.bgCard } });
  s4.addText(t.label, {
    x: tx + 0.12, y: ty, w: 1.6, h: 0.48,
    fontSize: 10, fontFace: "Calibri", color: C.gray, align: "left", valign: "middle", margin: 0
  });
  s4.addText(t.val, {
    x: tx + 1.7, y: ty, w: 1.4, h: 0.48,
    fontSize: 12, fontFace: "Calibri", bold: true, color: t.color, align: "right", valign: "middle", margin: 0
  });
});


// ═══════════════════════════════════════════
// SLIDE 5: TECHNOLOGY STACK
// ═══════════════════════════════════════════
let s5 = pres.addSlide();
addHeader(s5, "Technology Stack", "Hardware-native acceleration across every layer");

const techRows = [
  [{ text: "Layer", options: { bold: true, color: C.bg, fill: { color: C.accent } } },
   { text: "Technology", options: { bold: true, color: C.bg, fill: { color: C.accent } } },
   { text: "Purpose", options: { bold: true, color: C.bg, fill: { color: C.accent } } }],
  [{ text: "Runtime", options: { color: C.white, fill: { color: C.bgCard } } },
   { text: ".NET 10.0 (LTS)", options: { color: C.accent3, fill: { color: C.bgCard } } },
   { text: "Managed runtime + unsafe/intrinsics", options: { color: C.gray, fill: { color: C.bgCard } } }],
  [{ text: "Memory", options: { color: C.white, fill: { color: C.bgCardAlt } } },
   { text: "NativeMemory, Span<T>, ref struct", options: { color: C.accent3, fill: { color: C.bgCardAlt } } },
   { text: "Zero-GC, cache-line alignment", options: { color: C.gray, fill: { color: C.bgCardAlt } } }],
  [{ text: "Acceleration", options: { color: C.white, fill: { color: C.bgCard } } },
   { text: "AVX2/AVX-512, CRC32, POPCNT", options: { color: C.accent3, fill: { color: C.bgCard } } },
   { text: "SIMD scanning, hardware hash, bit ops", options: { color: C.gray, fill: { color: C.bgCard } } }],
  [{ text: "I/O (Linux)", options: { color: C.white, fill: { color: C.bgCardAlt } } },
   { text: "DPDK 26.03, io_uring", options: { color: C.accent3, fill: { color: C.bgCardAlt } } },
   { text: "Kernel bypass networking + disk", options: { color: C.gray, fill: { color: C.bgCardAlt } } }],
  [{ text: "Replication", options: { color: C.white, fill: { color: C.bgCard } } },
   { text: "RDMA (RoCE v2)", options: { color: C.accent3, fill: { color: C.bgCard } } },
   { text: "Sub-\u03BCs cross-node memory transfer", options: { color: C.gray, fill: { color: C.bgCard } } }],
  [{ text: "Pre-filter", options: { color: C.white, fill: { color: C.bgCardAlt } } },
   { text: "eBPF/XDP", options: { color: C.accent3, fill: { color: C.bgCardAlt } } },
   { text: "NIC-level packet validation ~10-15 ns", options: { color: C.gray, fill: { color: C.bgCardAlt } } }],
  [{ text: "Cloud", options: { color: C.white, fill: { color: C.bgCard } } },
   { text: "AWS EFA, Azure IB, GCP gVNIC", options: { color: C.accent3, fill: { color: C.bgCard } } },
   { text: "Cloud-adapted kernel bypass", options: { color: C.gray, fill: { color: C.bgCard } } }],
  [{ text: "Timing", options: { color: C.white, fill: { color: C.bgCardAlt } } },
   { text: "RDTSC + PTP (IEEE 1588)", options: { color: C.accent3, fill: { color: C.bgCardAlt } } },
   { text: "Sub-ns telemetry + cross-node ordering", options: { color: C.gray, fill: { color: C.bgCardAlt } } }],
];

s5.addTable(techRows, {
  x: 0.6, y: 1.15, w: 8.8,
  colW: [1.5, 3.3, 4.0],
  fontSize: 10, fontFace: "Calibri",
  border: { pt: 0.5, color: "1A2744" },
  rowH: [0.4, 0.38, 0.38, 0.38, 0.38, 0.38, 0.38, 0.38, 0.38],
  valign: "middle"
});


// ═══════════════════════════════════════════
// SLIDE 6: MEMORY ARCHITECTURE
// ═══════════════════════════════════════════
let s6 = pres.addSlide();
addHeader(s6, "Memory Architecture", "Zero-GC allocation with hardware-native alignment");

addCard(s6, 0.6, 1.2, 4.3, 2.0, "MemoryArena (Bump Allocator)", [
  "NativeMemory.AlignedAlloc — bypasses .NET GC entirely",
  "O(1) allocation: just increment offset pointer",
  "O(1) reset: zero the offset — all memory reclaimed instantly",
  "Cache-line aligned: (size + 63) & ~63",
  "Arena reset after every command cycle"
], C.accent);

addCard(s6, 5.1, 1.2, 4.3, 2.0, "Cache-Line Alignment (64 Bytes)", [
  "All hot-path structs: exactly 64 bytes",
  "CommandFrame: 64B (SequenceNumber, Timestamp, AggregateId...)",
  "WalRecord: 64B (with PrevChecksum hash chain)",
  "Prevents false sharing between CPU cores",
  "[StructLayout(LayoutKind.Explicit, Size = 64)]"
], C.accent3);

addCard(s6, 0.6, 3.45, 4.3, 1.65, "HugePages (TLB Optimization)", [
  "Standard 4KB pages: thousands of TLB misses",
  "2MB HugePages: reduces TLB misses 500x",
  "1GB HugePages: 4 TLB entries covers 4GB",
  "Near-zero TLB miss overhead on hot path"
], C.green);

addCard(s6, 5.1, 3.45, 4.3, 1.65, "NUMA Affinity", [
  "Every thread pinned to physical core",
  "Memory allocated from local NUMA node only",
  "Remote NUMA access costs 2x — never allowed",
  "P/Invoke: sched_setaffinity (Linux) / SetThreadAffinityMask (Win)"
], C.orange);


// ═══════════════════════════════════════════
// SLIDE 7: EXECUTION KERNEL
// ═══════════════════════════════════════════
let s7 = pres.addSlide();
addHeader(s7, "Execution Kernel", "LMAX-inspired single-writer engine with advanced pipeline orchestration");

addCard(s7, 0.6, 1.2, 2.8, 2.0, "Single-Writer Principle", [
  "One thread owns one shard",
  "Zero locks, mutexes, semaphores",
  "Serializable execution (strongest isolation)",
  "Millions of ops/sec per shard"
], C.accent);

addCard(s7, 3.55, 1.2, 2.8, 2.0, "Ring Buffer (LMAX-Style)", [
  "Pre-allocated, power-of-2 sized",
  "Cache-line padded head/tail",
  "Two-phase publish: claim \u2192 copy \u2192 publish",
  "POPCNT for O(1) slot checking"
], C.accent3);

addCard(s7, 6.5, 1.2, 2.9, 2.0, "Wait Strategies", [
  "BusySpin: ~ns wake-up (hot path)",
  "Yielding: ~\u03BCs (WAL writer)",
  "Sleeping: ~ms (projections)",
  "Blocking: kernel event (background)"
], C.green);

addCard(s7, 0.6, 3.45, 2.8, 1.75, "Consumer DAGs", [
  "Parallel + gated pipeline stages",
  "Journal + Replicate in parallel",
  "Execute after both complete",
  "YAML-defined topology"
], C.purple);

addCard(s7, 3.55, 3.45, 2.8, 1.75, "Back-Pressure", [
  "Block: producer waits (safest)",
  "Reject: fast failure for client",
  "DropOldest: telemetry/metrics",
  "Per-tenant policy support"
], C.orange);

addCard(s7, 6.5, 3.45, 2.9, 1.75, "Prefetch & Branchless", [
  "Sse.Prefetch0(): lookahead prefetch",
  "Process cmd N, prefetch N+1's entity",
  "Branchless: CMOV, SIMD masks",
  "~15 ns saved per misprediction"
], C.yellow);


// ═══════════════════════════════════════════
// SLIDE 8: TRANSACTION MODEL
// ═══════════════════════════════════════════
let s8 = pres.addSlide();
addHeader(s8, "Transaction Model", "YAML-compiled mutations with two-phase transaction support");

addCard(s8, 0.6, 1.2, 4.3, 2.0, "YAML-Compiled Mutations", [
  "Developers define logic in YAML, not C#",
  "Engine compiles YAML \u2192 Expression Trees \u2192 delegates",
  "Zero-allocation, branchless machine code at startup",
  "Complex logic \u2192 DLL Sandbox (post-commit, separate process)",
  "90% YAML (hot path) + 10% DLL (off path)"
], C.accent);

addCard(s8, 5.1, 1.2, 4.3, 2.0, "Two-Phase Transactions", [
  "Pending: reserve funds/inventory/seats",
  "Post: confirm (full or partial amount)",
  "Void: cancel and release reservation",
  "Timeout: auto-void after configurable seconds",
  "All transitions WAL-logged for replay safety"
], C.orange);

addCard(s8, 0.6, 3.45, 4.3, 1.65, "Idempotency Keys (Exactly-Once)", [
  "Client-generated 128-bit key per command",
  "Bloom filter: O(1) duplicate detection (~1 ns)",
  "Idempotency log: verification on bloom hit",
  "Duplicate \u2192 return cached result, no re-execution"
], C.green);

addCard(s8, 5.1, 3.45, 4.3, 1.65, "Transaction Lifecycle", [
  "Pipeline: Command \u2192 Validate \u2192 WAL \u2192 Mutate \u2192 ACK",
  "Per-shard monotonic sequencing (total order)",
  "Deterministic: same inputs = same outputs",
  "Recovery: replay commands through ShardExecutor"
], C.accent3);


// ═══════════════════════════════════════════
// SLIDE 9: DURABILITY & WAL
// ═══════════════════════════════════════════
let s9 = pres.addSlide();
addHeader(s9, "Durability & Write-Ahead Log", "Zero data loss with append-only sequential writes");

addCard(s9, 0.6, 1.2, 4.3, 1.5, "Append-Only WAL", [
  "256MB segments (configurable), sequential NVMe writes",
  "O_DIRECT + io_uring: bypasses OS page cache",
  "fsync policies: sync | group-commit | async",
  "Zero-copy replication to followers via RDMA"
], C.accent);

addCard(s9, 5.1, 1.2, 4.3, 1.5, "Hash Chain Integrity", [
  "PrevChecksum in every WalRecord (tamper-evident chain)",
  "CRC32 hardware intrinsic (Sse42.Crc32): ~1 cycle/8 bytes",
  "Detects: torn writes, bit rot, silent corruption",
  "Optional: write-verify (read-after-write confirmation)"
], C.red);

addCard(s9, 0.6, 2.95, 4.3, 1.5, "Snapshots & Recovery", [
  "Time-based, count-based, or adaptive scheduling",
  "Recovery: Snapshot \u2192 WAL replay \u2192 Ready (seconds)",
  "Deterministic replay: commands through ShardExecutor",
  "Point-in-time recovery to any WAL LSN"
], C.green);

addCard(s9, 5.1, 2.95, 4.3, 1.5, "Advanced Features", [
  "Group commit: batch 1000s of commands per fsync",
  "Compaction: key-based + time-based retention",
  "Offline WAL Compiler: CSV/SQL \u2192 WAL at 5 GB/sec",
  "Tiered: Hot (NVMe) \u2192 Warm (SQL) \u2192 Cold (S3)"
], C.purple);


// ═══════════════════════════════════════════
// SLIDE 10: KERNEL BYPASS NETWORKING
// ═══════════════════════════════════════════
let s10 = pres.addSlide();
addHeader(s10, "Kernel Bypass Networking", "OS kernel eliminated from the data path");

addCard(s10, 0.6, 1.2, 4.3, 1.5, "DPDK (Client \u2192 Engine)", [
  "Polls NIC hardware queues directly in user space",
  "Zero system calls, zero context switches",
  "~1-2 \u03BCs end-to-end latency",
  "Linux only (RIO fallback on Windows)"
], C.accent);

addCard(s10, 5.1, 1.2, 4.3, 1.5, "RDMA (Engine \u2192 Engine)", [
  "Direct memory-to-memory across nodes",
  "Sub-microsecond WAL replication",
  "RoCE v2 via libibverbs P/Invoke",
  "Bypasses both CPU and kernel on target"
], C.accent3);

addCard(s10, 0.6, 2.95, 4.3, 1.5, "eBPF/XDP (Pre-Filter)", [
  "Runs at NIC driver level (Ring 0, before kernel)",
  "Filters non-business traffic: ~10-15 ns",
  "20-30M packets/sec dropped with near-zero CPU",
  "Written in Restricted C, compiled to eBPF bytecode"
], C.green);

addCard(s10, 5.1, 2.95, 4.3, 1.5, "PTP & Cloud Adapters", [
  "PTP (IEEE 1588): <100 ns cross-node clock sync",
  "NIC hardware timestamps (not DateTime.UtcNow)",
  "AWS EFA, Azure InfiniBand, GCP gVNIC",
  "INetworkTransport abstraction for all platforms"
], C.orange);


// ═══════════════════════════════════════════
// SLIDE 11: READ FABRIC
// ═══════════════════════════════════════════
let s11 = pres.addSlide();
addHeader(s11, "Read Fabric", "85% of operations are reads — handled at nanosecond speed inside the engine");

addCard(s11, 0.6, 1.2, 2.8, 2.0, "Document Mode", [
  "By-ID lookups: ~5-10 ns",
  "Pre-joined, denormalized entities",
  "Contiguous memory blocks",
  "One fetch = full entity + relationships"
], C.accent);

addCard(s11, 3.55, 1.2, 2.8, 2.0, "Columnar Mode", [
  "Range scans + filters",
  "Each field: contiguous array",
  "AVX2: 8 values per instruction",
  "\"Amount > 500 AND Status = Placed\""
], C.accent3);

addCard(s11, 6.5, 1.2, 2.9, 2.0, "Compiled Query Engine", [
  "Expression Trees \u2192 Func<> delegates",
  "Cached, zero-GC execution",
  "Compound filters compiled once",
  "As fast as hand-written code"
], C.green);

addCard(s11, 0.6, 3.45, 4.3, 1.65, "Materialized Views (Continuous Aggregations)", [
  "Pre-computed COUNT, SUM, AVG, MIN, MAX per group key",
  "Incrementally updated on every write (atomic with mutation)",
  "Query result in ~5 ns (direct lookup, no scan)",
  "YAML-defined, survive restart via WAL replay"
], C.orange);

addCard(s11, 5.1, 3.45, 4.3, 1.65, "Tiered Read Routing", [
  "Hot (in-memory): ~50 ns — MemoryArena",
  "Warm (NVMe SSD): ~10-50 \u03BCs — SSD index scan",
  "Cold (S3/Blob): ~50-200 ms — object store",
  "Transparent routing: developer sends one query"
], C.purple);


// ═══════════════════════════════════════════
// SLIDE 12: DEVELOPER EXPERIENCE
// ═══════════════════════════════════════════
let s12 = pres.addSlide();
addHeader(s12, "Developer Experience", "YAML-first configuration with polyglot SDK support");

addCard(s12, 0.6, 1.2, 4.3, 2.0, "YAML-Defined Everything", [
  "Entities, mutations, validations, events",
  "Topology, sharding, retention policies",
  "Scaling rules, tenancy, projections",
  "Compiled to branchless delegates at boot",
  "Developers never write engine-internal code"
], C.accent);

addCard(s12, 5.1, 1.2, 4.3, 2.0, "Polyglot Client SDKs", [
  "Protocol Buffers wire protocol (one .proto file)",
  "Tier 1: C#, Java, Python, Node.js, Go, Rust, C++",
  "Tier 2: Ruby, PHP, Kotlin, Swift, Dart",
  "Thin wrappers (~200-500 lines each)",
  "Connection pooling, retry, tenant auth (JWT/mTLS)"
], C.accent3);

// Transport table
const transportRows = [
  [{ text: "Protocol", options: { bold: true, color: C.bg, fill: { color: C.accent } } },
   { text: "Overhead", options: { bold: true, color: C.bg, fill: { color: C.accent } } },
   { text: "Best For", options: { bold: true, color: C.bg, fill: { color: C.accent } } }],
  [{ text: "Raw TCP/UDP", options: { color: C.accent3, fill: { color: C.bgCard } } },
   { text: "~1-5 \u03BCs", options: { color: C.green, fill: { color: C.bgCard } } },
   { text: "C++, Rust (direct CommandFrame)", options: { color: C.gray, fill: { color: C.bgCard } } }],
  [{ text: "gRPC (protobuf)", options: { color: C.accent3, fill: { color: C.bgCardAlt } } },
   { text: "~10-50 \u03BCs", options: { color: C.accent, fill: { color: C.bgCardAlt } } },
   { text: "Java, C#, Go, Python, Node.js", options: { color: C.gray, fill: { color: C.bgCardAlt } } }],
  [{ text: "HTTP/REST (JSON)", options: { color: C.accent3, fill: { color: C.bgCard } } },
   { text: "~50-500 \u03BCs", options: { color: C.orange, fill: { color: C.bgCard } } },
   { text: "Browser clients, legacy teams", options: { color: C.gray, fill: { color: C.bgCard } } }],
];

s12.addTable(transportRows, {
  x: 0.6, y: 3.5, w: 8.8, colW: [2.5, 1.8, 4.5],
  fontSize: 10, fontFace: "Calibri",
  border: { pt: 0.5, color: "1A2744" },
  rowH: [0.35, 0.33, 0.33, 0.33],
  valign: "middle"
});


// ═══════════════════════════════════════════
// SLIDE 13: MULTI-TENANCY
// ═══════════════════════════════════════════
let s13 = pres.addSlide();
addHeader(s13, "Multi-Tenancy & QoS", "Engine-level isolation with workload prioritization");

addCard(s13, 0.6, 1.2, 4.3, 2.0, "Isolation Modes", [
  "Shared Shard: multiple tenants, logical isolation",
  "Dedicated Shard: one tenant, physical isolation",
  "Hybrid: critical tenants dedicated, long-tail shared",
  "Per-tenant: memory limits, throughput quotas",
  "Encryption keys, RBAC, retention policies"
], C.accent);

addCard(s13, 5.1, 1.2, 4.3, 2.0, "Workload Prioritization (QoS)", [
  "Share-based scheduling (1-1000 shares per tier)",
  "Critical (1000): always runs first, never preempted",
  "Interactive (500): low latency, API responses",
  "Batch (10): runs in gaps, analytics/exports",
  "Per-tenant tier assignment via YAML"
], C.orange);

addCard(s13, 0.6, 3.45, 8.8, 1.65, "Tenant Boundary Enforcement", [
  "Every CommandFrame, WAL record, snapshot, projection scoped by TenantId",
  "Tenant boundaries preserved during WAL replay, snapshot restore, and failover",
  "Noisy-tenant back-pressure: reject policy for exceeding quota while critical tenants continue",
  "Cross-tenant data leakage: impossible by design (TenantId in every struct, validated at every layer)"
], C.accent3);


// ═══════════════════════════════════════════
// SLIDE 14: OPERATIONAL TOOLING
// ═══════════════════════════════════════════
let s14 = pres.addSlide();
addHeader(s14, "Operational Tooling (vvctl)", "Unified CLI + interactive TUI dashboard");

const vvctlCmds = [
  { grp: "Config", cmds: "init, apply, validate, tenant, schema" },
  { grp: "Health", cmds: "status, nodes, shards, health" },
  { grp: "Data", cmds: "entities, get, query, wal tail, wal inspect" },
  { grp: "Metrics", cmds: "metrics, bench, profile, cores" },
  { grp: "Projections", cmds: "projections, pause/resume, replay" },
  { grp: "Ops", cmds: "snapshot, drain, failover, rebalance, logs" },
  { grp: "Migration", cmds: "analyze --dll, analyze --db, import, shadow, cdc" },
  { grp: "Dashboard", cmds: "vvctl dashboard (full-screen TUI, live-updating)" },
];

vvctlCmds.forEach((v, i) => {
  const vy = 1.15 + i * 0.48;
  s14.addShape(pres.shapes.RECTANGLE, { x: 0.6, y: vy, w: 1.5, h: 0.4, fill: { color: C.accent2 } });
  s14.addText(v.grp, {
    x: 0.7, y: vy, w: 1.4, h: 0.4,
    fontSize: 10, fontFace: "Calibri", bold: true, color: C.white, align: "left", valign: "middle", margin: 0
  });
  s14.addShape(pres.shapes.RECTANGLE, { x: 2.2, y: vy, w: 7.2, h: 0.4, fill: { color: C.bgCard } });
  s14.addText(v.cmds, {
    x: 2.35, y: vy, w: 7.0, h: 0.4,
    fontSize: 10, fontFace: "Consolas", color: C.accent3, align: "left", valign: "middle", margin: 0
  });
});

// TUI preview hint
s14.addText("TUI Dashboard: live cluster map, shard topology, latency histograms, WAL throughput, core assignments", {
  x: 0.6, y: 5.1, w: 8.8, h: 0.35,
  fontSize: 9.5, fontFace: "Calibri", italic: true, color: C.dimText, align: "center", margin: 0
});


// ═══════════════════════════════════════════
// SLIDE 15: HA & SCALING
// ═══════════════════════════════════════════
let s15 = pres.addSlide();
addHeader(s15, "High Availability & Scaling", "Zero-downtime operations with automatic rebalancing");

addCard(s15, 0.6, 1.2, 4.3, 1.5, "Epoch Fencing (Split-Brain Prevention)", [
  "Monotonic epoch numbers per shard",
  "Stale leader's writes rejected (epoch too low)",
  "New leader starts with incremented epoch",
  "Prevents dual-leader data corruption"
], C.accent);

addCard(s15, 5.1, 1.2, 4.3, 1.5, "Auto-Scaling & Shard Splitting", [
  "CPU, memory, queue depth thresholds trigger split",
  "Policy Engine re-hashes subset to new shard",
  "Linear horizontal scaling across cores + nodes",
  "Configurable max_shards_per_node"
], C.green);

addCard(s15, 0.6, 2.95, 4.3, 1.5, "Live Shard Migration (Zero-Downtime)", [
  "Pause \u2192 Snapshot \u2192 Transfer \u2192 WAL Catchup \u2192 Switch",
  "Drain protocol: migrate all shards before maintenance",
  "Clients reconnect transparently",
  "Max concurrent migrations configurable"
], C.orange);

addCard(s15, 5.1, 2.95, 4.3, 1.5, "Cluster-Wide Versionstamps", [
  "10-byte monotonic tokens (8B version + 2B sequence)",
  "Total ordering across all shards",
  "Cross-shard change feeds with guaranteed order",
  "Per-shard SequenceNumber for intra-shard (zero overhead)"
], C.purple);


// ═══════════════════════════════════════════
// SLIDE 16: PROJECTION DISPATCHER
// ═══════════════════════════════════════════
let s16 = pres.addSlide();
addHeader(s16, "Projection Dispatcher", "Async WAL consumption to external databases — never on the hot path");

addCard(s16, 0.6, 1.2, 4.3, 2.8, "How It Works", [
  "Consumes committed WAL events asynchronously",
  "Cursor-based: tracks WAL offset per target",
  "On crash: resumes from last committed cursor",
  "Bulk/batch: TVP (SQL Server), COPY (Postgres)",
  "Multi-target: one WAL feeds N databases",
  "Rebuildable: add target later, replay from start",
  "Eventual consistency: ~100-500 ms behind engine"
], C.accent);

addCard(s16, 5.1, 1.2, 4.3, 2.8, "Supported Targets", [
  "SQL Server (bulk TVP inserts)",
  "PostgreSQL (COPY protocol)",
  "MongoDB (bulk insert)",
  "ElasticSearch (bulk index)",
  "Any custom store: IProjectionTarget interface",
  "CDC: stream changes to Kafka/RabbitMQ",
  "Pluggable, configurable per entity in YAML"
], C.accent3);

s16.addText("Real-time reads: query engine\u2019s Read Fabric directly  |  Historical/analytics: query projected database", {
  x: 0.6, y: 4.3, w: 8.8, h: 0.35,
  fontSize: 10, fontFace: "Calibri", italic: true, color: C.dimText, align: "center", margin: 0
});


// ═══════════════════════════════════════════
// SLIDE 17: LEGACY MIGRATION
// ═══════════════════════════════════════════
let s17 = pres.addSlide();
addHeader(s17, "Legacy Application Migration", "Brownfield strategy: never rewrite everything at once");

addCard(s17, 0.6, 1.2, 2.8, 1.8, "Command Gateway", [
  "Anti-Corruption Layer",
  "JSON/HTTP \u2192 CommandFrame",
  "GC boundary (~5-50 \u03BCs)",
  "Legacy code never touches kernel"
], C.accent);

addCard(s17, 3.55, 1.2, 2.8, 1.8, "eBPF Shadow Mode", [
  "Zero-touch migration",
  "XDP filter + uretprobe",
  "Clone traffic at NIC level",
  "Run months in parallel"
], C.green);

addCard(s17, 6.5, 1.2, 2.9, 1.8, "CDC (Debezium)", [
  "Read legacy DB's own WAL",
  "SQL changes \u2192 CommandFrames",
  "Build in-memory replica",
  "Most practical for 90%"
], C.orange);

addCard(s17, 0.6, 3.25, 2.8, 1.8, "Assembly Analyzer", [
  "IL decompilation of .NET DLLs",
  "Auto-classify every method",
  "\u2705 Pure logic \u2192 YAML",
  "\u26A0\uFE0F Mixed \u2192 manual split",
], C.purple);

addCard(s17, 3.55, 3.25, 2.8, 1.8, "Wasm Sandbox", [
  "NativeAOT-LLVM \u2192 Wasm",
  "GC trapped in linear memory",
  "~100-200 ns (research tier)",
  "Phase 8 research module"
], C.accent3);

addCard(s17, 6.5, 3.25, 2.9, 1.8, "Offline WAL Compiler", [
  "CSV/SQL \u2192 WAL segments",
  "~5 GB/sec bulk import",
  "10B records in seconds",
  "No live engine needed"
], C.yellow);


// ═══════════════════════════════════════════
// SLIDE 18: ADVANCED FEATURES
// ═══════════════════════════════════════════
let s18 = pres.addSlide();
addHeader(s18, "Advanced Features", "Competitive parity with LMAX, TigerBeetle, Redis, ScyllaDB, FoundationDB");

addCard(s18, 0.6, 1.2, 2.8, 1.65, "Probabilistic Structures", [
  "Bloom Filter: O(1) membership",
  "HyperLogLog: cardinality est.",
  "Count-Min Sketch: frequency",
  "Zero-alloc, MemoryArena-native"
], C.accent);

addCard(s18, 3.55, 1.2, 2.8, 1.65, "Transactional Watches", [
  "Client watches entity/key",
  "Push notification on change",
  "Fire-once or continuous mode",
  "~\u03BCs delivery, <2 ns overhead"
], C.green);

addCard(s18, 6.5, 1.2, 2.9, 1.65, "TTL & Auto-Expiration", [
  "Per-record time-to-live",
  "Branchless expiry check",
  "Background reaper thread",
  "Optional expiry events"
], C.orange);

addCard(s18, 0.6, 3.1, 2.8, 1.65, "Warm Restart", [
  "MemoryArena in shared memory",
  "Process restart: ms not seconds",
  "Epoch + checksum validation",
  "Fallback: snapshot + WAL replay"
], C.purple);

addCard(s18, 3.55, 3.1, 2.8, 1.65, "Write-Verify", [
  "Read-after-write verification",
  "Catches silent disk corruption",
  "Modes: disabled/sampled/full",
  "Shadow thread: zero hot-path cost"
], C.red);

addCard(s18, 6.5, 3.1, 2.9, 1.65, "Load Generator", [
  "vvctl loadgen: built-in benchmark",
  "Entity-level CRUD simulation",
  "Zipfian key distribution",
  "p50/p99/p99.9 reporting"
], C.accent3);


// ═══════════════════════════════════════════
// SLIDE 19: TESTING STRATEGY
// ═══════════════════════════════════════════
let s19 = pres.addSlide();
addHeader(s19, "Testing Strategy", "FoundationDB/TigerBeetle-proven methodology");

addCard(s19, 0.6, 1.2, 4.3, 2.2, "Deterministic Simulation Testing (DST)", [
  "Replace clock, network, disk with mock hardware",
  "Run entire cluster in single thread",
  "Simulate millions of failures per hour",
  "Power loss, disk corruption, partition, split-brain",
  "Because VectorVolt is deterministic, DST is natural fit",
  "Simulate 10 years of uptime in 1-hour test run"
], C.accent);

addCard(s19, 5.1, 1.2, 4.3, 2.2, "Test Pyramid", [
  "Unit Tests (70%): < 100ms each, hot-path logic",
  "Integration Tests (20%): WAL write/replay, failover",
  "E2E Tests (10%): full pipeline, client to read query",
  "BenchmarkDotNet: every hot-path function",
  "VTune/perf: cache misses, branch mispredictions",
  "Deterministic replay: identical state from WAL"
], C.accent3);

addCard(s19, 0.6, 3.65, 4.3, 1.3, "Chaos Testing", [
  "Kill leader mid-transaction \u2192 verify failover",
  "Corrupt WAL segment \u2192 verify detection",
  "Network partition \u2192 verify epoch fencing",
  "Fill memory \u2192 verify warm tier eviction"
], C.orange);

addCard(s19, 5.1, 3.65, 4.3, 1.3, "Load Testing & Load Generator", [
  "Sustained 1M+ ops/sec load test",
  "p50/p99/p99.9 latency measurement",
  "Zero GC pauses verified (.NET diagnostics)",
  "vvctl loadgen: built-in, entity-level CRUD"
], C.green);


// ═══════════════════════════════════════════
// SLIDE 20: RESEARCH MODULES
// ═══════════════════════════════════════════
let s20 = pres.addSlide();
addHeader(s20, "Research Modules (Phase 8)", "PhD differentiators \u2014 each could be a thesis chapter");

addCard(s20, 0.6, 1.2, 4.3, 1.5, "Learned Indexes", [
  "MIT 2018: replace B-Tree with ML model",
  "Linear regression predicts memory offset",
  "B-Tree: ~150 ns \u2192 Learned: ~3 ns",
  "Train on key distribution at snapshot time"
], C.accent);

addCard(s20, 5.1, 1.2, 4.3, 1.5, "P4 / Programmable Switches", [
  "Intel Tofino ASICs: stamp sequence at line rate",
  "NOPaxos: eliminates consensus overhead entirely",
  "Packets arrive pre-ordered",
  "CommandFrame.SequenceNumber receives switch stamp"
], C.accent3);

addCard(s20, 0.6, 2.95, 4.3, 1.5, "Unikernel Deployment", [
  "NativeAOT \u2192 Nanos/NanoVMs \u2192 bootable .img",
  "No OS: app IS the operating system (Ring 0)",
  "Boot: ~5-20 ms (vs Linux ~10-30 sec)",
  "KubeVirt: run as VM in Kubernetes"
], C.green);

addCard(s20, 5.1, 2.95, 4.3, 1.5, "WebAssembly Sandbox", [
  "Legacy C# \u2192 NativeAOT-LLVM \u2192 Wasm",
  "GC physically trapped in linear memory",
  "Wasmtime runtime (Fastly, Cloudflare-proven)",
  "~100-200 ns (compatibility tier, not performance)"
], C.orange);

s20.addText("Additional: Coroutine Latency Hiding (MosaicDB pattern) \u2014 deferred until .NET gets stackless coroutines", {
  x: 0.6, y: 4.7, w: 8.8, h: 0.35,
  fontSize: 9.5, fontFace: "Calibri", italic: true, color: C.dimText, align: "center", margin: 0
});


// ═══════════════════════════════════════════
// SLIDE 21: COMPETITIVE LANDSCAPE
// ═══════════════════════════════════════════
let s21 = pres.addSlide();
addHeader(s21, "Competitive Landscape", "Feature parity achieved across 8 industry-leading systems");

const compRows = [
  [{ text: "Feature", options: { bold: true, color: C.bg, fill: { color: C.accent }, fontSize: 8.5 } },
   { text: "VectorVolt", options: { bold: true, color: C.bg, fill: { color: C.accent }, fontSize: 8.5 } },
   { text: "LMAX", options: { bold: true, color: C.bg, fill: { color: C.accent }, fontSize: 8.5 } },
   { text: "TigerBeetle", options: { bold: true, color: C.bg, fill: { color: C.accent }, fontSize: 8.5 } },
   { text: "VoltDB", options: { bold: true, color: C.bg, fill: { color: C.accent }, fontSize: 8.5 } },
   { text: "Redis", options: { bold: true, color: C.bg, fill: { color: C.accent }, fontSize: 8.5 } },
   { text: "ScyllaDB", options: { bold: true, color: C.bg, fill: { color: C.accent }, fontSize: 8.5 } },
   { text: "FDB", options: { bold: true, color: C.bg, fill: { color: C.accent }, fontSize: 8.5 } }],
];

const features = [
  ["Zero-GC Hot Path",     "\u2705","\u2705","\u2705","\u274C","\u274C","\u2705","\u2705"],
  ["Single-Writer/Shard",  "\u2705","\u2705","\u2705","\u2705","\u274C","\u2705","\u274C"],
  ["Kernel Bypass (DPDK)", "\u2705","\u274C","\u274C","\u274C","\u274C","\u2705","\u274C"],
  ["SIMD/AVX2 Scanning",   "\u2705","\u274C","\u274C","\u274C","\u274C","\u2705","\u274C"],
  ["Two-Phase Txns",       "\u2705","\u274C","\u2705","\u2705","\u274C","\u274C","\u274C"],
  ["DST Simulation",       "\u2705","\u274C","\u2705","\u274C","\u274C","\u274C","\u2705"],
  ["Materialized Views",   "\u2705","\u274C","\u274C","\u2705","\u274C","\u274C","\u274C"],
  ["YAML-Defined Logic",   "\u2705","\u274C","\u274C","\u274C","\u274C","\u274C","\u274C"],
  ["Polyglot SDKs",        "\u2705","\u274C","\u2705","\u2705","\u2705","\u2705","\u2705"],
  ["QoS / Prioritization", "\u2705","\u274C","\u274C","\u274C","\u274C","\u2705","\u274C"],
];

features.forEach((row, i) => {
  const bgColor = i % 2 === 0 ? C.bgCard : C.bgCardAlt;
  compRows.push(row.map((cell, j) => ({
    text: cell,
    options: {
      color: j === 0 ? C.white : (cell === "\u2705" ? C.green : C.red),
      fill: { color: j === 1 ? "0D2137" : bgColor },
      fontSize: j === 0 ? 8.5 : 11,
      align: j === 0 ? "left" : "center",
      bold: j === 1
    }
  })));
});

s21.addTable(compRows, {
  x: 0.3, y: 1.1, w: 9.4,
  colW: [2.0, 1.15, 0.9, 1.15, 0.9, 0.85, 1.1, 0.85],
  fontSize: 9, fontFace: "Calibri",
  border: { pt: 0.5, color: "1A2744" },
  rowH: [0.33, 0.33, 0.33, 0.33, 0.33, 0.33, 0.33, 0.33, 0.33, 0.33, 0.33],
  valign: "middle"
});


// ═══════════════════════════════════════════
// SLIDE 22: IMPLEMENTATION PLAN
// ═══════════════════════════════════════════
let s22 = pres.addSlide();
addHeader(s22, "8-Phase Implementation Plan", "Learn \u2192 Test \u2192 Build \u2192 Prove — one concept at a time");

const phases = [
  { ph: "Phase 1", name: "Memory & CPU", items: "Cache lines, MemoryArena, HugePages, NUMA, CPU dispatch, OS jitter", color: "1565C0" },
  { ph: "Phase 2", name: "Execution Kernel", items: "Ring buffers, wait strategies, DAGs, back-pressure, idempotency, two-phase txns", color: "0277BD" },
  { ph: "Phase 3", name: "Durability", items: "Direct I/O, group commit, hash chains, recovery, projections, bulk load", color: "00838F" },
  { ph: "Phase 4", name: "Networking", items: "eBPF/XDP, DPDK, RDMA, RIO, PTP, zero-copy, cloud adapters", color: "00695C" },
  { ph: "Phase 5", name: "Read Fabric", items: "CQRS, columnar, SIMD/AVX2, query filters, materialized views", color: "2E7D32" },
  { ph: "Phase 6", name: "Control Plane", items: "Gateway, SDKs, tenancy, scaling, migration, observability, QoS, vvctl", color: "558B2F" },
  { ph: "Phase 7", name: "Proof", items: "DST, load test, chaos test, migration toolkit, sample app, research docs", color: "827717" },
  { ph: "Phase 8", name: "Research", items: "Learned indexes, P4, Unikernel, Wasm, TTL, watches, warm restart", color: "6D4C41" },
];

phases.forEach((p, i) => {
  const py = 1.15 + i * 0.52;
  s22.addShape(pres.shapes.RECTANGLE, { x: 0.6, y: py, w: 0.8, h: 0.42, fill: { color: p.color } });
  s22.addText(p.ph, {
    x: 0.6, y: py, w: 0.8, h: 0.42,
    fontSize: 9, fontFace: "Calibri", bold: true, color: C.white, align: "center", valign: "middle", margin: 0
  });
  s22.addShape(pres.shapes.RECTANGLE, { x: 1.5, y: py, w: 2.0, h: 0.42, fill: { color: C.bgCard } });
  s22.addText(p.name, {
    x: 1.6, y: py, w: 1.9, h: 0.42,
    fontSize: 10, fontFace: "Calibri", bold: true, color: C.accent3, align: "left", valign: "middle", margin: 0
  });
  s22.addShape(pres.shapes.RECTANGLE, { x: 3.6, y: py, w: 5.8, h: 0.42, fill: { color: C.bgCardAlt } });
  s22.addText(p.items, {
    x: 3.7, y: py, w: 5.6, h: 0.42,
    fontSize: 9, fontFace: "Calibri", color: C.gray, align: "left", valign: "middle", margin: 0
  });
});

s22.addText("\"Learning first, speed second. This may take years. That is acceptable. Understanding is the goal.\"", {
  x: 0.6, y: 5.1, w: 8.8, h: 0.35,
  fontSize: 10, fontFace: "Calibri", italic: true, color: C.dimText, align: "center", margin: 0
});


// ═══════════════════════════════════════════
// SLIDE 23: CLOSING
// ═══════════════════════════════════════════
let s23 = pres.addSlide();
s23.background = { color: C.bg };
s23.addShape(pres.shapes.RECTANGLE, { x: 0, y: 0, w: 10, h: 0.06, fill: { color: C.accent } });

s23.addText("VECTORVOLT", {
  x: 0.8, y: 1.0, w: 8.4, h: 0.8,
  fontSize: 48, fontFace: "Calibri", bold: true, color: C.white, charSpacing: 8, align: "center", margin: 0
});
s23.addText("Where Physics Meets Data", {
  x: 0.8, y: 1.75, w: 8.4, h: 0.5,
  fontSize: 22, fontFace: "Calibri", italic: true, color: C.accent3, align: "center", margin: 0
});

s23.addShape(pres.shapes.LINE, { x: 3.5, y: 2.5, w: 3.0, h: 0, line: { color: C.accent, width: 2 } });

// Final stats
const finalStats = [
  { val: "20 ns", label: "Engine Latency" },
  { val: "Zero GC", label: "On Hot Path" },
  { val: "Kernel Bypass", label: "DPDK + RDMA" },
  { val: "Billions", label: "Of Records" },
];
finalStats.forEach((fs, i) => {
  const fx = 0.8 + i * 2.3;
  s23.addShape(pres.shapes.RECTANGLE, { x: fx, y: 2.9, w: 2.0, h: 1.1, fill: { color: C.bgCard }, shadow: cardShadow() });
  s23.addText(fs.val, {
    x: fx, y: 2.92, w: 2.0, h: 0.65,
    fontSize: 22, fontFace: "Calibri", bold: true, color: C.accent, align: "center", valign: "middle", margin: 0
  });
  s23.addText(fs.label, {
    x: fx, y: 3.55, w: 2.0, h: 0.38,
    fontSize: 10, fontFace: "Calibri", color: C.gray, align: "center", valign: "middle", margin: 0
  });
});

s23.addText(".NET 10.0  |  C#  |  SIMD/AVX2  |  Zero-Allocation  |  Research & PhD Project", {
  x: 0.8, y: 4.4, w: 8.4, h: 0.4,
  fontSize: 11, fontFace: "Calibri", color: C.dimText, align: "center", margin: 0
});

s23.addShape(pres.shapes.RECTANGLE, { x: 0, y: 5.565, w: 10, h: 0.06, fill: { color: C.accent } });


// ═══════════════════════════════════════════
// SAVE
// ═══════════════════════════════════════════
pres.writeFile({ fileName: "C:\\VectorVolt\\VectorVolt_Features.pptx" })
  .then(() => console.log("Presentation saved to C:\\VectorVolt\\VectorVolt_Features.pptx"))
  .catch(err => console.error("Error:", err));
