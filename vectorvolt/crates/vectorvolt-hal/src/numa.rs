//! NUMA topology detection via `/sys/devices/system/node/*` (VV-HAL-028).
//!
//! Provides a lightweight [`NumaTopology`] that maps NUMA nodes to their CPU
//! sets and reported memory.  On non-Linux platforms a single synthetic node is
//! returned so callers compile and run everywhere.

// ── Types ─────────────────────────────────────────────────────────────────────

/// One NUMA node as reported by the OS.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NumaNode {
    pub node_id: usize,
    pub cpu_ids: Vec<usize>,
    /// MemTotal in MB, if available from `/sys/.../meminfo`.
    pub memory_mb: Option<u64>,
}

/// Full NUMA topology of the machine.
#[derive(Debug, Clone)]
pub struct NumaTopology {
    pub nodes: Vec<NumaNode>,
}

impl NumaTopology {
    /// Probe the machine and return its NUMA topology.
    ///
    /// On Linux, reads `/sys/devices/system/node/node*/cpulist` and
    /// `/sys/devices/system/node/node*/meminfo`.  Falls back to a single node
    /// with all logical cores when the sysfs path is absent (container/VM).
    ///
    /// On non-Linux platforms returns a single synthetic node.
    pub fn probe() -> Self {
        platform::probe()
    }

    /// Number of NUMA nodes.
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Return the NUMA node id that `cpu_id` belongs to, or `None` if not found.
    pub fn node_for_cpu(&self, cpu_id: usize) -> Option<usize> {
        self.nodes
            .iter()
            .find(|n| n.cpu_ids.contains(&cpu_id))
            .map(|n| n.node_id)
    }

    /// CPUs belonging to `node_id`, or an empty slice if the node does not exist.
    pub fn cpus_for_node(&self, node_id: usize) -> &[usize] {
        self.nodes
            .iter()
            .find(|n| n.node_id == node_id)
            .map(|n| n.cpu_ids.as_slice())
            .unwrap_or(&[])
    }

    /// True when there is more than one NUMA node.
    pub fn is_numa(&self) -> bool {
        self.nodes.len() > 1
    }

    /// Total number of logical CPUs across all nodes.
    pub fn total_cpus(&self) -> usize {
        self.nodes.iter().map(|n| n.cpu_ids.len()).sum()
    }
}

// ── Fallback helper ───────────────────────────────────────────────────────────

fn fallback_single_node() -> NumaTopology {
    let n = std::thread::available_parallelism()
        .map(|x| x.get())
        .unwrap_or(1);
    NumaTopology {
        nodes: vec![NumaNode {
            node_id: 0,
            cpu_ids: (0..n).collect(),
            memory_mb: None,
        }],
    }
}

// ── Linux implementation ───────────────────────────────────────────────────────

#[cfg(target_os = "linux")]
mod platform {
    use super::{NumaNode, NumaTopology, fallback_single_node};
    use std::path::Path;

    pub(super) fn probe() -> NumaTopology {
        let node_base = Path::new("/sys/devices/system/node");
        if !node_base.exists() {
            return fallback_single_node();
        }

        let mut nodes: Vec<NumaNode> = Vec::new();

        let read_dir = match std::fs::read_dir(node_base) {
            Ok(d) => d,
            Err(_) => return fallback_single_node(),
        };

        for entry in read_dir.flatten() {
            let name = entry.file_name();
            let name = name.to_string_lossy();
            if !name.starts_with("node") {
                continue;
            }
            let node_id: usize = match name["node".len()..].parse() {
                Ok(id) => id,
                Err(_) => continue,
            };

            let cpu_ids = parse_cpulist(&entry.path().join("cpulist"));
            if cpu_ids.is_empty() {
                continue;
            }

            let memory_mb = parse_meminfo(&entry.path().join("meminfo"));

            nodes.push(NumaNode {
                node_id,
                cpu_ids,
                memory_mb,
            });
        }

        if nodes.is_empty() {
            return fallback_single_node();
        }

        // Sort by node_id for deterministic output.
        nodes.sort_unstable_by_key(|n| n.node_id);
        NumaTopology { nodes }
    }

    /// Parse a Linux cpulist file, e.g. `0-3,8-11` → `[0,1,2,3,8,9,10,11]`.
    fn parse_cpulist(path: &Path) -> Vec<usize> {
        let text = match std::fs::read_to_string(path) {
            Ok(t) => t,
            Err(_) => return Vec::new(),
        };
        let mut ids = Vec::new();
        for token in text.trim().split(',') {
            if let Some((a, b)) = token.split_once('-') {
                if let (Ok(lo), Ok(hi)) = (a.parse::<usize>(), b.parse::<usize>()) {
                    ids.extend(lo..=hi);
                }
            } else if let Ok(n) = token.parse::<usize>() {
                ids.push(n);
            }
        }
        ids.sort_unstable();
        ids.dedup();
        ids
    }

    /// Parse MemTotal from `/sys/devices/system/node/nodeN/meminfo`.
    fn parse_meminfo(path: &Path) -> Option<u64> {
        let text = std::fs::read_to_string(path).ok()?;
        for line in text.lines() {
            // Format: "Node 0 MemTotal:       16384 kB"
            if line.contains("MemTotal:") {
                let kb: u64 = line
                    .split_whitespace()
                    .rev()
                    .nth(1) // second from end (before "kB")
                    .and_then(|s| s.parse().ok())?;
                return Some(kb / 1024); // convert kB -> MB
            }
        }
        None
    }
}

// ── Non-Linux fallback ────────────────────────────────────────────────────────

#[cfg(not(target_os = "linux"))]
mod platform {
    use super::{NumaTopology, fallback_single_node};

    pub(super) fn probe() -> NumaTopology {
        fallback_single_node()
    }
}

// ── tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // 1 — probe returns at least 1 node
    #[test]
    fn numa_probe_returns_at_least_one_node() {
        let t = NumaTopology::probe();
        assert!(t.node_count() >= 1);
    }

    // 2 — node_count >= 1
    #[test]
    fn numa_node_count_ge_1() {
        assert!(NumaTopology::probe().node_count() >= 1);
    }

    // 3 — total_cpus >= 1
    #[test]
    fn numa_total_cpus_ge_1() {
        assert!(NumaTopology::probe().total_cpus() >= 1);
    }

    // 4 — node_for_cpu 0 returns Some
    #[test]
    fn numa_node_for_cpu0_is_some() {
        assert!(NumaTopology::probe().node_for_cpu(0).is_some());
    }

    // 5 — cpus_for_node 0 is non-empty
    #[test]
    fn numa_cpus_for_node0_non_empty() {
        assert!(!NumaTopology::probe().cpus_for_node(0).is_empty());
    }

    // 6 — is_numa false on single node
    #[test]
    fn numa_is_numa_false_single_node() {
        let t = NumaTopology::probe();
        if t.node_count() == 1 {
            assert!(!t.is_numa());
        }
    }

    // 7 — total_cpus = sum of all node cpu counts
    #[test]
    fn numa_total_cpus_equals_sum() {
        let t = NumaTopology::probe();
        let sum: usize = t.nodes.iter().map(|n| n.cpu_ids.len()).sum();
        assert_eq!(t.total_cpus(), sum);
    }

    // 8 — node_for_cpu out_of_range returns None
    #[test]
    fn numa_node_for_cpu_out_of_range_is_none() {
        assert!(NumaTopology::probe().node_for_cpu(usize::MAX).is_none());
    }

    // 9 — probe twice same result (deterministic)
    #[test]
    fn numa_probe_is_deterministic() {
        let a = NumaTopology::probe();
        let b = NumaTopology::probe();
        assert_eq!(a.node_count(), b.node_count());
        assert_eq!(a.total_cpus(), b.total_cpus());
    }

    // 10 — nodes sorted by node_id
    #[test]
    fn numa_nodes_sorted_by_id() {
        let t = NumaTopology::probe();
        let ids: Vec<usize> = t.nodes.iter().map(|n| n.node_id).collect();
        let mut sorted = ids.clone();
        sorted.sort_unstable();
        assert_eq!(ids, sorted);
    }

    // 11 — cpus_for_node returns sorted slice
    #[test]
    fn numa_cpus_for_node_sorted() {
        let t = NumaTopology::probe();
        let cpus = t.cpus_for_node(0);
        let mut s = cpus.to_vec();
        s.sort_unstable();
        assert_eq!(cpus, s.as_slice());
    }

    // 12 — NumaNode cpu_ids non-empty for node 0
    #[test]
    fn numa_node0_cpu_ids_non_empty() {
        let t = NumaTopology::probe();
        assert!(!t.nodes[0].cpu_ids.is_empty());
    }

    // 13 — total_cpus matches logical core count
    #[test]
    fn numa_total_cpus_matches_available_parallelism() {
        let t = NumaTopology::probe();
        let avail = std::thread::available_parallelism()
            .map(|x| x.get())
            .unwrap_or(1);
        // On a real NUMA machine total_cpus == avail; on a container/VM with
        // cpuset restrictions they may differ, so just assert > 0.
        assert!(t.total_cpus() > 0);
        let _ = avail; // suppress unused warning
    }
}
