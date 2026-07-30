//! CPU topology detection (VV-HAL-001) and core role assignment (VV-HAL-025).
//!
//! ## Why a database needs this and `available_parallelism()` is not enough
//!
//! `std::thread::available_parallelism()` returns a single number. Three facts it
//! hides will each silently cost you latency:
//!
//! 1. **SMT siblings share L1 and L2.** On x86 a "logical core" is usually a
//!    hyperthread — *half* a physical core. Pin two write shards to sibling
//!    threads and they evict each other's hot index lines all day. The count says
//!    16; the number of independent caches is 8.
//!
//! 2. **Hybrid CPUs have unequal cores.** Intel's P-cores and E-cores differ in
//!    clock and width. Pinning a latency-critical writer to an E-core is a
//!    performance bug that a core *count* cannot warn you about. This is not
//!    exotic — it is every consumer Intel chip since Alder Lake, and ARM
//!    big.LITTLE before that.
//!
//! 3. **NUMA distance is real.** Memory attached to another socket costs roughly
//!    1.5–2× the local latency. Which cores share a node determines where an
//!    arena should live.
//!
//! ## What this module does *not* do
//!
//! Detection only. It reads, classifies, and recommends — it never pins a thread,
//! reserves a page, or changes a system setting. Acting on the recommendation is
//! VV-HAL-024 (pinning) and VV-HAL-031 (huge pages), deliberately separate so this
//! module stays safe to call anywhere, including in tests and on a laptop.
//!
//! ## Detection sources
//!
//! | Platform | Mechanism |
//! |---|---|
//! | Linux | `/sys/devices/system/cpu/*` and `/sys/devices/system/node/*` — plain file reads, no syscalls |
//! | Windows | `GetLogicalProcessorInformationEx`, which is the only API exposing `EfficiencyClass` for P/E detection |
//! | other | Degrades to `available_parallelism()` with everything else `Unknown` |
//!
//! Every field is best-effort. A missing value is `None` or `Unknown`, never a
//! guess — a fabricated topology would produce a confidently wrong core map.

use crate::cache::CACHE_LINE;

/// Performance class of a physical core on a hybrid CPU.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CoreClass {
    /// Slower, power-efficient core. Intel E-core, ARM LITTLE.
    /// Fine for background work; wrong for a latency-critical writer.
    Efficient,
    /// Full-speed core. Intel P-core, ARM big.
    Performance,
    /// Not a hybrid CPU, or the class could not be determined. Treat as uniform.
    Unknown,
}

/// One logical processor (a hyperthread on SMT systems, else a whole core).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogicalCore {
    /// OS-visible id — the value you would pass to a pinning call.
    pub id: usize,
    /// Physical core this belongs to. Two logical cores sharing this share L1/L2.
    pub physical_id: usize,
    /// Socket / package id.
    pub socket_id: usize,
    /// NUMA node id.
    pub numa_node: usize,
    /// Other logical ids on the same physical core. Empty when SMT is off.
    pub smt_siblings: Vec<usize>,
    pub class: CoreClass,
}

impl LogicalCore {
    /// True if this logical core has the physical core to itself.
    #[inline]
    pub fn is_exclusive(&self) -> bool {
        self.smt_siblings.is_empty()
    }
}

/// Cache sizes in bytes. `None` where the platform did not report a level.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct CacheHierarchy {
    pub l1d_bytes: Option<usize>,
    pub l1i_bytes: Option<usize>,
    pub l2_bytes: Option<usize>,
    pub l3_bytes: Option<usize>,
    pub line_bytes: usize,
}

impl CacheHierarchy {
    /// Largest reported level — the budget an index must fit inside to stay
    /// cache-resident, which S1a measurements showed matters more than vector width.
    pub fn largest(&self) -> Option<usize> {
        self.l3_bytes.or(self.l2_bytes).or(self.l1d_bytes)
    }
}

/// A NUMA node and the logical cores attached to it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NumaNode {
    pub id: usize,
    pub cpus: Vec<usize>,
    pub memory_bytes: Option<usize>,
}

/// Detected CPU layout.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CpuTopology {
    pub logical: Vec<LogicalCore>,
    pub numa_nodes: Vec<NumaNode>,
    pub cache: CacheHierarchy,
    /// True when cores of differing [`CoreClass`] were found.
    pub hybrid: bool,
    /// Set when platform detection failed and values are a fallback.
    pub degraded: bool,
}

impl CpuTopology {
    /// Detect the topology of the running machine.
    ///
    /// Never fails: on any error it degrades to a single-node, uniform,
    /// non-SMT view sized by `available_parallelism()` and sets
    /// [`degraded`](Self::degraded).
    pub fn detect() -> Self {
        #[cfg(target_os = "linux")]
        {
            linux::detect().unwrap_or_else(Self::fallback)
        }
        #[cfg(target_os = "windows")]
        {
            windows::detect().unwrap_or_else(Self::fallback)
        }
        #[cfg(not(any(target_os = "linux", target_os = "windows")))]
        {
            Self::fallback()
        }
    }

    /// Uniform single-node view. Correct in shape, unaware of SMT/hybrid/NUMA.
    fn fallback() -> Self {
        let n = std::thread::available_parallelism()
            .map(|v| v.get())
            .unwrap_or(1);
        CpuTopology {
            logical: (0..n)
                .map(|id| LogicalCore {
                    id,
                    physical_id: id,
                    socket_id: 0,
                    numa_node: 0,
                    smt_siblings: Vec::new(),
                    class: CoreClass::Unknown,
                })
                .collect(),
            numa_nodes: vec![NumaNode {
                id: 0,
                cpus: (0..n).collect(),
                memory_bytes: None,
            }],
            cache: CacheHierarchy {
                line_bytes: CACHE_LINE,
                ..Default::default()
            },
            hybrid: false,
            degraded: true,
        }
    }

    pub fn logical_count(&self) -> usize {
        self.logical.len()
    }

    /// Distinct physical cores — the number of independent L1/L2 caches, which is
    /// what actually bounds how many non-contending shards you can run.
    pub fn physical_count(&self) -> usize {
        let mut ids: Vec<usize> = self.logical.iter().map(|c| c.physical_id).collect();
        ids.sort_unstable();
        ids.dedup();
        ids.len()
    }

    pub fn socket_count(&self) -> usize {
        let mut ids: Vec<usize> = self.logical.iter().map(|c| c.socket_id).collect();
        ids.sort_unstable();
        ids.dedup();
        ids.len()
    }

    /// True when more than one logical core shares a physical core.
    pub fn smt_enabled(&self) -> bool {
        self.logical.iter().any(|c| !c.is_exclusive())
    }

    /// One logical core per physical core, lowest id first.
    ///
    /// **This is the list to pin latency-critical threads to.** Using it
    /// guarantees no two pinned threads contend for the same L1/L2.
    pub fn physical_representatives(&self) -> Vec<usize> {
        let mut seen: Vec<usize> = Vec::new();
        let mut out = Vec::new();
        let mut sorted: Vec<&LogicalCore> = self.logical.iter().collect();
        sorted.sort_by_key(|c| c.id);
        for c in sorted {
            if !seen.contains(&c.physical_id) {
                seen.push(c.physical_id);
                out.push(c.id);
            }
        }
        out
    }

    /// Logical ids whose class matches, lowest first.
    pub fn cores_of_class(&self, class: CoreClass) -> Vec<usize> {
        let mut v: Vec<usize> = self
            .logical
            .iter()
            .filter(|c| c.class == class)
            .map(|c| c.id)
            .collect();
        v.sort_unstable();
        v
    }

    /// One representative per *performance* physical core. Falls back to all
    /// physical representatives on a non-hybrid CPU.
    pub fn performance_representatives(&self) -> Vec<usize> {
        if !self.hybrid {
            return self.physical_representatives();
        }
        let mut seen: Vec<usize> = Vec::new();
        let mut out = Vec::new();
        let mut sorted: Vec<&LogicalCore> = self
            .logical
            .iter()
            .filter(|c| c.class == CoreClass::Performance)
            .collect();
        sorted.sort_by_key(|c| c.id);
        for c in sorted {
            if !seen.contains(&c.physical_id) {
                seen.push(c.physical_id);
                out.push(c.id);
            }
        }
        out
    }

    pub fn core(&self, logical_id: usize) -> Option<&LogicalCore> {
        self.logical.iter().find(|c| c.id == logical_id)
    }

    /// One-line summary for logs and `vvctl hal`.
    pub fn summary(&self) -> String {
        let mut s = format!(
            "{} logical / {} physical / {} socket(s) / {} numa node(s)",
            self.logical_count(),
            self.physical_count(),
            self.socket_count(),
            self.numa_nodes.len()
        );
        if self.smt_enabled() {
            s.push_str(", SMT on");
        }
        if self.hybrid {
            let p = self.cores_of_class(CoreClass::Performance).len();
            let e = self.cores_of_class(CoreClass::Efficient).len();
            s.push_str(&format!(", HYBRID {p}P/{e}E"));
        }
        if self.degraded {
            s.push_str(" [DEGRADED — detection failed, values are a fallback]");
        }
        s
    }
}

// ── Core role map (VV-HAL-025) ────────────────────────────────────────────────

/// Which cores each subsystem should run on.
///
/// Advice, not action — pinning is VV-HAL-024.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CoreRoleMap {
    /// Left to the OS and interrupt handling. Never pinned by us.
    pub os_reserved: Vec<usize>,
    /// Network polling.
    pub network: Vec<usize>,
    /// Write shards. Latency-critical: P-cores, one per physical core.
    pub writers: Vec<usize>,
    /// Read shards. Latency-sensitive but tolerant of SMT sharing with readers.
    pub readers: Vec<usize>,
    /// Compaction, flush, sinks, projections. Throughput work, E-cores are fine.
    pub background: Vec<usize>,
    /// Control plane and admin. Lowest priority.
    pub admin: Vec<usize>,
}

impl CoreRoleMap {
    /// Derive an assignment from a detected topology.
    ///
    /// ## The rules, and why each exists
    ///
    /// 1. **Core 0 and its SMT sibling go to the OS.** On Linux most IRQs land on
    ///    CPU 0 by default; sharing it with a writer means a network interrupt can
    ///    preempt a commit.
    /// 2. **Writers get performance cores, one per physical core.** Two writers on
    ///    SMT siblings fight over the same L1, which is the opposite of the
    ///    shard-per-core design. On a hybrid CPU an E-core writer would silently
    ///    run at a fraction of the speed.
    /// 3. **A writer and a reader never share a physical core** (VV-HAL-026).
    ///    Their access patterns cause cache-line ping-pong through the coherence
    ///    fabric — a bounce costs more than a DRAM read.
    /// 4. **Background and admin get efficiency cores** where they exist.
    ///    Compaction and sink delivery are throughput work with no latency SLA.
    /// 5. **Small machines share rather than starve.** Below ~4 physical cores
    ///    there is nothing to partition, so roles overlap and the wait strategy
    ///    should not spin (see VV-CFG-035 clamping).
    pub fn derive(topo: &CpuTopology) -> Self {
        let phys = topo.physical_representatives();

        // Too small to partition meaningfully — overlap and say so via the doc
        // contract rather than handing out empty role lists.
        if phys.len() < 4 {
            let all = phys.clone();
            return CoreRoleMap {
                os_reserved: Vec::new(),
                network: all.clone(),
                writers: all.clone(),
                readers: all.clone(),
                background: all.clone(),
                admin: all,
            };
        }

        // Rule 1: reserve physical core 0 (and its siblings) for the OS.
        let first = phys[0];
        let mut os_reserved = vec![first];
        if let Some(c) = topo.core(first) {
            os_reserved.extend(c.smt_siblings.iter().copied());
        }
        os_reserved.sort_unstable();

        // Candidate pool: performance representatives, minus the OS core.
        let perf: Vec<usize> = topo
            .performance_representatives()
            .into_iter()
            .filter(|id| !os_reserved.contains(id))
            .collect();
        let eff: Vec<usize> = topo
            .cores_of_class(CoreClass::Efficient)
            .into_iter()
            .filter(|id| !os_reserved.contains(id))
            .collect();

        // If the CPU is not hybrid, `perf` is every physical core: split it.
        // Rule 2 + 3: writers and readers come from disjoint physical cores.
        let mut pool = perf;

        // Rule 6: give the network poller its own physical core whenever at least
        // four remain. For a kernel-bypass transport the poll loop is on the
        // critical path of *every* request, so sharing it with a write shard adds
        // that shard's cache pressure to every inbound packet.
        //
        // The threshold counts the pool *after* the OS reservation. An earlier
        // version tested the pool before it, which meant a 6-P-core machine — the
        // shape of a current laptop — ended up with no network core at all.
        let net_take = usize::from(pool.len() >= 4);
        let network: Vec<usize> = pool.drain(..net_take).collect();

        // Half the remainder to writers, half to readers — disjoint by
        // construction because `pool` holds one entry per physical core.
        let split = pool.len().div_ceil(2);
        let writers: Vec<usize> = pool.drain(..split).collect();
        let readers: Vec<usize> = pool;

        // Rule 4: background prefers efficiency cores; if none, borrow the SMT
        // siblings of reader cores — background work tolerates cache sharing.
        let background = if !eff.is_empty() {
            eff.clone()
        } else {
            readers
                .iter()
                .filter_map(|id| topo.core(*id))
                .flat_map(|c| c.smt_siblings.iter().copied())
                .collect()
        };

        // Admin gets whatever background got; it is idle almost always.
        let admin = background.clone();

        CoreRoleMap {
            os_reserved,
            network,
            writers,
            readers,
            background,
            admin,
        }
    }

    /// Check the invariant from rule 3: no writer shares a physical core with a
    /// reader. Returns the offending `(writer, reader)` pairs.
    ///
    /// Exposed rather than merely asserted internally so a custom role map from
    /// config can be validated the same way (VV-CFG-035 reconciliation).
    pub fn writer_reader_conflicts(&self, topo: &CpuTopology) -> Vec<(usize, usize)> {
        let mut out = Vec::new();
        for w in &self.writers {
            let wp = topo.core(*w).map(|c| c.physical_id);
            for r in &self.readers {
                if wp.is_some() && wp == topo.core(*r).map(|c| c.physical_id) {
                    out.push((*w, *r));
                }
            }
        }
        out
    }

    pub fn summary(&self) -> String {
        fn f(v: &[usize]) -> String {
            if v.is_empty() {
                "-".into()
            } else {
                v.iter()
                    .map(|x| x.to_string())
                    .collect::<Vec<_>>()
                    .join(",")
            }
        }
        format!(
            "os=[{}] net=[{}] writers=[{}] readers=[{}] bg=[{}] admin=[{}]",
            f(&self.os_reserved),
            f(&self.network),
            f(&self.writers),
            f(&self.readers),
            f(&self.background),
            f(&self.admin)
        )
    }
}

// ── Linux detection ───────────────────────────────────────────────────────────

#[cfg(target_os = "linux")]
mod linux {
    use super::*;
    use std::fs;
    use std::path::Path;

    fn read_usize(p: impl AsRef<Path>) -> Option<usize> {
        fs::read_to_string(p).ok()?.trim().parse().ok()
    }

    /// Parse a Linux cpu list: "0-3,8,10-11".
    pub(super) fn parse_cpu_list(s: &str) -> Vec<usize> {
        let mut out = Vec::new();
        for part in s.trim().split(',').filter(|p| !p.is_empty()) {
            match part.split_once('-') {
                Some((a, b)) => {
                    if let (Ok(a), Ok(b)) = (a.trim().parse::<usize>(), b.trim().parse::<usize>()) {
                        out.extend(a..=b);
                    }
                }
                None => {
                    if let Ok(v) = part.trim().parse() {
                        out.push(v);
                    }
                }
            }
        }
        out
    }

    /// `size` files look like "32K" or "1024K" or "8M".
    fn parse_size(s: &str) -> Option<usize> {
        let s = s.trim();
        let (num, mult) = match s.chars().last()? {
            'K' | 'k' => (&s[..s.len() - 1], 1024),
            'M' | 'm' => (&s[..s.len() - 1], 1024 * 1024),
            'G' | 'g' => (&s[..s.len() - 1], 1024 * 1024 * 1024),
            _ => (s, 1),
        };
        num.trim().parse::<usize>().ok().map(|v| v * mult)
    }

    pub(super) fn detect() -> Option<CpuTopology> {
        let cpu_root = Path::new("/sys/devices/system/cpu");
        let online = fs::read_to_string(cpu_root.join("online")).ok()?;
        let cpus = parse_cpu_list(&online);
        if cpus.is_empty() {
            return None;
        }

        // Hybrid signal. `cpu_capacity` is the portable one (ARM big.LITTLE and
        // Intel hybrid both expose it); differing values mean unequal cores.
        let capacities: Vec<Option<usize>> = cpus
            .iter()
            .map(|c| read_usize(cpu_root.join(format!("cpu{c}/cpu_capacity"))))
            .collect();
        let max_cap = capacities.iter().flatten().copied().max();
        let min_cap = capacities.iter().flatten().copied().min();
        let hybrid = matches!((min_cap, max_cap), (Some(a), Some(b)) if a != b);

        let mut logical = Vec::with_capacity(cpus.len());
        for (idx, &c) in cpus.iter().enumerate() {
            let topo_dir = cpu_root.join(format!("cpu{c}/topology"));
            let physical_id = read_usize(topo_dir.join("core_id")).unwrap_or(c);
            let socket_id = read_usize(topo_dir.join("physical_package_id")).unwrap_or(0);
            let smt_siblings = fs::read_to_string(topo_dir.join("thread_siblings_list"))
                .ok()
                .map(|s| parse_cpu_list(&s))
                .unwrap_or_default()
                .into_iter()
                .filter(|&s| s != c)
                .collect();

            let class = match (capacities[idx], max_cap) {
                (Some(cap), Some(max)) if hybrid && cap == max => CoreClass::Performance,
                (Some(_), Some(_)) if hybrid => CoreClass::Efficient,
                _ => CoreClass::Unknown,
            };

            logical.push(LogicalCore {
                id: c,
                // Combine with socket: core_id is only unique within a package.
                physical_id: socket_id * 100_000 + physical_id,
                socket_id,
                numa_node: 0, // filled below
                smt_siblings,
                class,
            });
        }

        // NUMA nodes.
        let node_root = Path::new("/sys/devices/system/node");
        let mut numa_nodes = Vec::new();
        if let Ok(entries) = fs::read_dir(node_root) {
            for e in entries.flatten() {
                let name = e.file_name();
                let name = name.to_string_lossy();
                if let Some(id) = name.strip_prefix("node").and_then(|n| n.parse().ok()) {
                    let cpus = fs::read_to_string(e.path().join("cpulist"))
                        .ok()
                        .map(|s| parse_cpu_list(&s))
                        .unwrap_or_default();
                    for cpu in &cpus {
                        if let Some(l) = logical.iter_mut().find(|l| l.id == *cpu) {
                            l.numa_node = id;
                        }
                    }
                    numa_nodes.push(NumaNode {
                        id,
                        cpus,
                        memory_bytes: None,
                    });
                }
            }
        }
        numa_nodes.sort_by_key(|n| n.id);
        if numa_nodes.is_empty() {
            numa_nodes.push(NumaNode {
                id: 0,
                cpus: cpus.clone(),
                memory_bytes: None,
            });
        }

        // Cache sizes, read from the first CPU's cache indices.
        let mut cache = CacheHierarchy {
            line_bytes: CACHE_LINE,
            ..Default::default()
        };
        let cache_dir = cpu_root.join(format!("cpu{}/cache", cpus[0]));
        if let Ok(entries) = fs::read_dir(&cache_dir) {
            for e in entries.flatten() {
                let p = e.path();
                let level = read_usize(p.join("level"));
                let size = fs::read_to_string(p.join("size"))
                    .ok()
                    .and_then(|s| parse_size(&s));
                let ctype = fs::read_to_string(p.join("type"))
                    .ok()
                    .map(|s| s.trim().to_string())
                    .unwrap_or_default();
                if let (Some(level), Some(size)) = (level, size) {
                    match (level, ctype.as_str()) {
                        (1, "Data") => cache.l1d_bytes = Some(size),
                        (1, "Instruction") => cache.l1i_bytes = Some(size),
                        (2, _) => cache.l2_bytes = Some(size),
                        (3, _) => cache.l3_bytes = Some(size),
                        _ => {}
                    }
                }
                if let Some(ls) = read_usize(p.join("coherency_line_size"))
                    && ls > 0
                {
                    cache.line_bytes = ls;
                }
            }
        }

        Some(CpuTopology {
            logical,
            numa_nodes,
            cache,
            hybrid,
            degraded: false,
        })
    }
}

// ── Windows detection ─────────────────────────────────────────────────────────

#[cfg(target_os = "windows")]
mod windows {
    use super::*;
    use std::mem;
    use windows_sys::Win32::System::SystemInformation::{
        GetLogicalProcessorInformationEx, RelationAll, RelationCache, RelationNumaNode,
        RelationProcessorCore, SYSTEM_LOGICAL_PROCESSOR_INFORMATION_EX,
    };

    /// Logical ids set in a `GROUP_AFFINITY`-style mask.
    ///
    /// Processor groups above 0 (>64 logical CPUs) are offset by 64 per group.
    fn mask_to_ids(mask: usize, group: u16) -> Vec<usize> {
        let base = group as usize * 64;
        (0..usize::BITS as usize)
            .filter(|b| mask & (1usize << b) != 0)
            .map(|b| base + b)
            .collect()
    }

    pub(super) fn detect() -> Option<CpuTopology> {
        // Two-call pattern: ask for the size, then fill.
        let mut len: u32 = 0;
        // SAFETY: a null buffer with len=0 is the documented way to query the
        // required size; the call writes only to `len` and returns FALSE.
        unsafe {
            GetLogicalProcessorInformationEx(RelationAll, std::ptr::null_mut(), &mut len);
        }
        if len == 0 {
            return None;
        }

        let mut buf = vec![0u8; len as usize];
        // SAFETY: `buf` is `len` bytes, exactly what the previous call requested.
        let ok = unsafe {
            GetLogicalProcessorInformationEx(
                RelationAll,
                buf.as_mut_ptr() as *mut SYSTEM_LOGICAL_PROCESSOR_INFORMATION_EX,
                &mut len,
            )
        };
        if ok == 0 {
            return None;
        }

        let mut cores: Vec<(Vec<usize>, u8)> = Vec::new(); // (logical ids, efficiency class)
        let mut numa: Vec<(usize, Vec<usize>)> = Vec::new();
        let mut cache = CacheHierarchy {
            line_bytes: CACHE_LINE,
            ..Default::default()
        };

        // Walk the variable-length records using each header's `Size` field.
        let mut off = 0usize;
        while off + mem::size_of::<u32>() * 2 <= buf.len() {
            let rec = unsafe {
                &*(buf.as_ptr().add(off) as *const SYSTEM_LOGICAL_PROCESSOR_INFORMATION_EX)
            };
            let size = rec.Size as usize;
            if size == 0 || off + size > buf.len() {
                break; // malformed; stop rather than read out of bounds
            }

            // Deliberately `if`/`else if` on equality rather than `match` with
            // constant patterns. The Win32 constants are camelCase, and a
            // camelCase identifier in a match pattern is an *irrefutable
            // binding*, not a constant comparison — the first arm would swallow
            // every record and silently mis-parse the whole buffer. Using `==`
            // makes it a value comparison, so removing an import becomes a
            // compile error instead of a silent behaviour change.
            let rel = rec.Relationship;
            if rel == RelationProcessorCore {
                // SAFETY: Relationship discriminates the union.
                let p = unsafe { &rec.Anonymous.Processor };
                let mut ids = Vec::new();
                // GroupMask is an ANYSIZE array of GroupCount entries.
                for g in 0..p.GroupCount as usize {
                    let gm = unsafe { &*p.GroupMask.as_ptr().add(g) };
                    ids.extend(mask_to_ids(gm.Mask, gm.Group));
                }
                if !ids.is_empty() {
                    cores.push((ids, p.EfficiencyClass));
                }
            } else if rel == RelationNumaNode {
                // SAFETY: Relationship discriminates the outer union.
                let n = unsafe { &rec.Anonymous.NumaNode };
                // `NUMA_NODE_RELATIONSHIP` carries its affinity in a nested
                // union: a single `GroupMask`, or `GroupMasks[GroupCount]` when
                // the node spans processor groups (>64 logical CPUs). GroupCount
                // is 0 on older Windows, where only the single form exists — treat
                // that as one group.
                let groups = (n.GroupCount as usize).max(1);
                let mut ids = Vec::new();
                for g in 0..groups {
                    // SAFETY: both union arms begin with a GROUP_AFFINITY, so
                    // reading index 0 is valid either way; indices beyond 0 are
                    // only reached when GroupCount says they exist.
                    let gm = unsafe { &*n.Anonymous.GroupMasks.as_ptr().add(g) };
                    ids.extend(mask_to_ids(gm.Mask, gm.Group));
                }
                numa.push((n.NodeNumber as usize, ids));
            } else if rel == RelationCache {
                // SAFETY: Relationship discriminates the union.
                let c = unsafe { &rec.Anonymous.Cache };
                let sz = c.CacheSize as usize;
                if c.LineSize > 0 {
                    cache.line_bytes = c.LineSize as usize;
                }
                // CacheType: 0=Unified 1=Instruction 2=Data 3=Trace.
                // L2/L3 are reported once per sharing group, so keep the largest.
                match (c.Level, c.Type) {
                    (1, 2) => cache.l1d_bytes = Some(sz),
                    (1, 1) => cache.l1i_bytes = Some(sz),
                    (2, _) => cache.l2_bytes = Some(cache.l2_bytes.unwrap_or(0).max(sz)),
                    (3, _) => cache.l3_bytes = Some(cache.l3_bytes.unwrap_or(0).max(sz)),
                    _ => {}
                }
            }
            off += size;
        }

        if cores.is_empty() {
            return None;
        }

        // EfficiencyClass: higher is more performant. Differing values = hybrid.
        let max_eff = cores.iter().map(|(_, e)| *e).max().unwrap_or(0);
        let min_eff = cores.iter().map(|(_, e)| *e).min().unwrap_or(0);
        let hybrid = max_eff != min_eff;

        let mut logical = Vec::new();
        for (phys_idx, (ids, eff)) in cores.iter().enumerate() {
            let class = if !hybrid {
                CoreClass::Unknown
            } else if *eff == max_eff {
                CoreClass::Performance
            } else {
                CoreClass::Efficient
            };
            for &id in ids {
                logical.push(LogicalCore {
                    id,
                    physical_id: phys_idx,
                    socket_id: 0,
                    numa_node: numa
                        .iter()
                        .find(|(_, cpus)| cpus.contains(&id))
                        .map(|(n, _)| *n)
                        .unwrap_or(0),
                    smt_siblings: ids.iter().copied().filter(|&s| s != id).collect(),
                    class,
                });
            }
        }
        logical.sort_by_key(|c| c.id);

        let mut numa_nodes: Vec<NumaNode> = numa
            .into_iter()
            .map(|(id, cpus)| NumaNode {
                id,
                cpus,
                memory_bytes: None,
            })
            .collect();
        numa_nodes.sort_by_key(|n| n.id);
        if numa_nodes.is_empty() {
            numa_nodes.push(NumaNode {
                id: 0,
                cpus: logical.iter().map(|c| c.id).collect(),
                memory_bytes: None,
            });
        }

        Some(CpuTopology {
            logical,
            numa_nodes,
            cache,
            hybrid,
            degraded: false,
        })
    }
}

// ── tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    /// Synthetic 2-socket, SMT-on, non-hybrid machine: 8 physical, 16 logical.
    fn uniform_smt(physical: usize) -> CpuTopology {
        let mut logical = Vec::new();
        for p in 0..physical {
            let a = p * 2;
            let b = p * 2 + 1;
            for (id, sib) in [(a, b), (b, a)] {
                logical.push(LogicalCore {
                    id,
                    physical_id: p,
                    socket_id: if p < physical / 2 { 0 } else { 1 },
                    numa_node: if p < physical / 2 { 0 } else { 1 },
                    smt_siblings: vec![sib],
                    class: CoreClass::Unknown,
                });
            }
        }
        CpuTopology {
            logical,
            numa_nodes: vec![
                NumaNode {
                    id: 0,
                    cpus: (0..physical).collect(),
                    memory_bytes: None,
                },
                NumaNode {
                    id: 1,
                    cpus: (physical..physical * 2).collect(),
                    memory_bytes: None,
                },
            ],
            cache: CacheHierarchy {
                line_bytes: 64,
                ..Default::default()
            },
            hybrid: false,
            degraded: false,
        }
    }

    /// Synthetic hybrid, shaped like Meteor Lake: 6 P-cores with SMT, 8 E-cores without.
    fn hybrid_meteor_like() -> CpuTopology {
        let mut logical = Vec::new();
        let mut id = 0usize;
        for p in 0..6 {
            let (a, b) = (id, id + 1);
            id += 2;
            for (lid, sib) in [(a, b), (b, a)] {
                logical.push(LogicalCore {
                    id: lid,
                    physical_id: p,
                    socket_id: 0,
                    numa_node: 0,
                    smt_siblings: vec![sib],
                    class: CoreClass::Performance,
                });
            }
        }
        for p in 6..14 {
            logical.push(LogicalCore {
                id,
                physical_id: p,
                socket_id: 0,
                numa_node: 0,
                smt_siblings: Vec::new(),
                class: CoreClass::Efficient,
            });
            id += 1;
        }
        CpuTopology {
            logical,
            numa_nodes: vec![NumaNode {
                id: 0,
                cpus: (0..id).collect(),
                memory_bytes: None,
            }],
            cache: CacheHierarchy {
                line_bytes: 64,
                ..Default::default()
            },
            hybrid: true,
            degraded: false,
        }
    }

    #[test]
    fn detection_on_this_machine_is_self_consistent() {
        let t = CpuTopology::detect();
        assert!(t.logical_count() >= 1, "no logical cores detected");
        assert!(
            t.physical_count() <= t.logical_count(),
            "more physical ({}) than logical ({}) cores is impossible",
            t.physical_count(),
            t.logical_count()
        );
        assert!(
            !t.numa_nodes.is_empty(),
            "must always report at least one node"
        );
        assert_eq!(t.cache.line_bytes % 8, 0, "implausible cache line size");
        // Every SMT sibling must exist and be reciprocal.
        for c in &t.logical {
            for s in &c.smt_siblings {
                let sib = t.core(*s).expect("sibling id not present in topology");
                assert_eq!(
                    sib.physical_id, c.physical_id,
                    "siblings on different physical cores"
                );
                assert!(
                    sib.smt_siblings.contains(&c.id),
                    "sibling link is not reciprocal"
                );
            }
        }
        println!("detected: {}", t.summary());
        println!("roles:    {}", CoreRoleMap::derive(&t).summary());
    }

    #[test]
    fn physical_representatives_pick_one_per_physical_core() {
        let t = uniform_smt(8);
        let reps = t.physical_representatives();
        assert_eq!(
            reps.len(),
            8,
            "should be one per physical core, not per thread"
        );
        assert_eq!(
            reps,
            vec![0, 2, 4, 6, 8, 10, 12, 14],
            "should pick the lower sibling"
        );
        assert_eq!(t.logical_count(), 16);
        assert!(t.smt_enabled());
    }

    #[test]
    fn hybrid_detection_separates_p_from_e() {
        let t = hybrid_meteor_like();
        assert!(t.hybrid);
        assert_eq!(
            t.cores_of_class(CoreClass::Performance).len(),
            12,
            "6 P-cores x 2 threads"
        );
        assert_eq!(
            t.cores_of_class(CoreClass::Efficient).len(),
            8,
            "8 E-cores, no SMT"
        );
        assert_eq!(t.logical_count(), 20);
        assert_eq!(t.physical_count(), 14);
        // Performance representatives: one per P physical core only.
        assert_eq!(t.performance_representatives(), vec![0, 2, 4, 6, 8, 10]);
    }

    #[test]
    fn role_map_never_puts_a_writer_and_reader_on_one_physical_core() {
        // Rule 3 / VV-HAL-026, the invariant that matters most.
        for physical in [4usize, 6, 8, 16, 32] {
            let t = uniform_smt(physical);
            let m = CoreRoleMap::derive(&t);
            assert!(
                m.writer_reader_conflicts(&t).is_empty(),
                "writer/reader share a physical core at {physical} physical cores: {:?}",
                m.writer_reader_conflicts(&t)
            );
            assert!(!m.writers.is_empty(), "no writers assigned at {physical}");
            assert!(!m.readers.is_empty(), "no readers assigned at {physical}");
        }
    }

    #[test]
    fn role_map_keeps_writers_off_efficiency_cores() {
        // Rule 2: an E-core writer would silently run at a fraction of speed.
        let t = hybrid_meteor_like();
        let m = CoreRoleMap::derive(&t);
        let eff = t.cores_of_class(CoreClass::Efficient);
        for w in &m.writers {
            assert!(
                !eff.contains(w),
                "writer pinned to E-core {w} — the exact hybrid bug this rule exists to prevent"
            );
        }
        assert!(m.writer_reader_conflicts(&t).is_empty());
        // Background should land on E-cores when they exist.
        assert!(
            m.background.iter().all(|b| eff.contains(b)),
            "background should prefer E-cores: {:?}",
            m.background
        );
    }

    #[test]
    fn role_map_reserves_core_zero_for_the_os() {
        let t = uniform_smt(8);
        let m = CoreRoleMap::derive(&t);
        assert!(m.os_reserved.contains(&0), "core 0 must be OS-reserved");
        // Its SMT sibling too — an IRQ on either half preempts the physical core.
        assert!(
            m.os_reserved.contains(&1),
            "core 0's sibling must also be reserved"
        );
        for role in [&m.writers, &m.readers, &m.network] {
            assert!(
                !role.iter().any(|c| m.os_reserved.contains(c)),
                "a latency role was assigned an OS-reserved core"
            );
        }
    }

    #[test]
    fn network_gets_a_core_on_any_reasonably_sized_machine() {
        // Regression guard. The first version applied the threshold to the pool
        // *before* the OS reservation, so a 6-physical-P-core machine (a current
        // laptop) got zero network cores — and for a kernel-bypass transport the
        // poll loop is on the critical path of every request.
        let hybrid = hybrid_meteor_like();
        let m = CoreRoleMap::derive(&hybrid);
        assert!(
            !m.network.is_empty(),
            "6 P-cores must yield a network core; got roles: {}",
            m.summary()
        );

        for physical in [6usize, 8, 16, 32, 64] {
            let t = uniform_smt(physical);
            let m = CoreRoleMap::derive(&t);
            assert!(
                !m.network.is_empty(),
                "no network core at {physical} physical cores: {}",
                m.summary()
            );
            // And it must not double as a writer or reader.
            for n in &m.network {
                assert!(!m.writers.contains(n), "network core {n} also a writer");
                assert!(!m.readers.contains(n), "network core {n} also a reader");
            }
        }
    }

    #[test]
    fn every_latency_role_is_disjoint_on_a_real_sized_machine() {
        // os / network / writers / readers must not overlap at all.
        let t = uniform_smt(16);
        let m = CoreRoleMap::derive(&t);
        let groups: [(&str, &Vec<usize>); 4] = [
            ("os", &m.os_reserved),
            ("network", &m.network),
            ("writers", &m.writers),
            ("readers", &m.readers),
        ];
        for (i, (na, a)) in groups.iter().enumerate() {
            for (nb, b) in groups.iter().skip(i + 1) {
                let overlap: Vec<usize> = a.iter().copied().filter(|x| b.contains(x)).collect();
                assert!(overlap.is_empty(), "{na} and {nb} overlap on {overlap:?}");
            }
        }
    }

    #[test]
    fn tiny_machines_share_rather_than_starve() {
        // Below 4 physical cores there is nothing to partition; every role must
        // still get somewhere to run rather than an empty list.
        for physical in [1usize, 2, 3] {
            let t = uniform_smt(physical);
            let m = CoreRoleMap::derive(&t);
            assert!(
                !m.writers.is_empty(),
                "no writers at {physical} physical cores"
            );
            assert!(
                !m.readers.is_empty(),
                "no readers at {physical} physical cores"
            );
            assert!(!m.background.is_empty(), "no background at {physical}");
        }
    }

    #[test]
    fn fallback_is_shaped_correctly() {
        let t = CpuTopology::fallback();
        assert!(t.degraded, "fallback must flag itself as degraded");
        assert_eq!(t.numa_nodes.len(), 1);
        assert!(!t.hybrid);
        assert!(
            !t.smt_enabled(),
            "fallback cannot know about SMT, so must claim none"
        );
        assert_eq!(t.physical_count(), t.logical_count());
        assert!(
            t.summary().contains("DEGRADED"),
            "degraded state must be visible: {}",
            t.summary()
        );
    }

    #[test]
    fn cache_largest_prefers_the_outermost_level() {
        let c = CacheHierarchy {
            l1d_bytes: Some(48 * 1024),
            l2_bytes: Some(2 * 1024 * 1024),
            l3_bytes: Some(24 * 1024 * 1024),
            ..Default::default()
        };
        assert_eq!(c.largest(), Some(24 * 1024 * 1024));
        let c2 = CacheHierarchy {
            l1d_bytes: Some(32 * 1024),
            ..Default::default()
        };
        assert_eq!(c2.largest(), Some(32 * 1024));
        assert_eq!(CacheHierarchy::default().largest(), None);
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn linux_cpu_list_parsing() {
        assert_eq!(linux::parse_cpu_list("0-3"), vec![0, 1, 2, 3]);
        assert_eq!(linux::parse_cpu_list("0,2,4"), vec![0, 2, 4]);
        assert_eq!(linux::parse_cpu_list("0-1,4,6-7"), vec![0, 1, 4, 6, 7]);
        assert_eq!(linux::parse_cpu_list(" 5 \n"), vec![5]);
        assert!(linux::parse_cpu_list("").is_empty());
    }
}
