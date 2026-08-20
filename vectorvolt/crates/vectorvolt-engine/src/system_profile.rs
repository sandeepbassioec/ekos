//! Runtime system profile (VV-CFG-001).
//!
//! [`SystemProfile::probe()`] interrogates the host at startup and returns a
//! typed snapshot of the capabilities available to VectorVolt. Downstream
//! subsystems (config system, flush-threshold auto-tuner, partition router)
//! query this instead of re-probing the OS independently.

use vectorvolt_hal::{CpuTopology, HugePageBuf, SimdTier, simd};

// ── SystemProfile ─────────────────────────────────────────────────────────────

/// Capabilities snapshot captured once at startup.
#[derive(Debug, Clone, PartialEq)]
pub struct SystemProfile {
    pub logical_cores: usize,
    pub physical_cores: usize,
    pub numa_node_count: usize,
    pub simd_tier: SimdTier,
    pub huge_pages_available: bool,
    pub os_page_size: usize,
    /// Suggested flush threshold (rows) derived from CPU cache hierarchy and row size.
    pub recommended_flush_threshold: usize,
    /// Suggested partition count derived from logical core count.
    pub recommended_partition_count: usize,
}

impl SystemProfile {
    /// Probe the host and return a [`SystemProfile`].
    ///
    /// This call is moderately expensive (reads `/proc/cpuinfo` on Linux,
    /// queries WinAPI on Windows). Call once at startup and share the result.
    pub fn probe() -> Self {
        let topo = CpuTopology::detect();
        let logical = topo.logical_count().max(1);
        let physical = topo.physical_count().max(1);
        let numa = topo.numa_nodes.len().max(1);
        let simd = simd::detect();
        let huge_pages_available = HugePageBuf::alloc(2 * 1024 * 1024).is_ok();
        let os_page_size = page_size();

        let recommended_flush_threshold = derive_flush_threshold(&topo);
        let recommended_partition_count = derive_partition_count(logical);

        Self {
            logical_cores: logical,
            physical_cores: physical,
            numa_node_count: numa,
            simd_tier: simd,
            huge_pages_available,
            os_page_size,
            recommended_flush_threshold,
            recommended_partition_count,
        }
    }

    /// Human-readable one-line summary (shown in `vvctl version`).
    pub fn summary(&self) -> String {
        format!(
            "{} logical / {} physical / {} NUMA / SIMD={} / huge_pages={} / page={}B / flush_threshold={} / partitions={}",
            self.logical_cores,
            self.physical_cores,
            self.numa_node_count,
            self.simd_tier.name(),
            self.huge_pages_available,
            self.os_page_size,
            self.recommended_flush_threshold,
            self.recommended_partition_count,
        )
    }
}

// ── Derivation helpers ────────────────────────────────────────────────────────

/// Derive a flush threshold from the CPU topology.
///
/// Heuristic: target the largest cache level divided by an 8-byte row estimate.
/// Clamped to [1024, 1_048_576].
fn derive_flush_threshold(topo: &CpuTopology) -> usize {
    const ROW_BYTES: usize = 64; // conservative estimate
    let cache_bytes = topo.cache.largest().unwrap_or(4 * 1024 * 1024);
    (cache_bytes / ROW_BYTES).clamp(1_024, 1_048_576)
}

/// Derive a recommended partition count from logical core count.
///
/// Heuristic: 1 partition per writer core (half of logical cores), clamped [1, 64].
fn derive_partition_count(logical_cores: usize) -> usize {
    let writers = (logical_cores / 2).max(1);
    writers.clamp(1, 64)
}

/// Return the OS page size in bytes (defaults to 4 096 if undetectable).
fn page_size() -> usize {
    // 4 KiB is the universal default; HAL probes the real value elsewhere.
    4096
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn probe_returns_positive_core_count() {
        let p = SystemProfile::probe();
        assert!(p.logical_cores >= 1);
        assert!(p.physical_cores >= 1);
    }

    #[test]
    fn probe_numa_node_count_at_least_one() {
        assert!(SystemProfile::probe().numa_node_count >= 1);
    }

    #[test]
    fn probe_simd_tier_is_valid() {
        let tier = SystemProfile::probe().simd_tier;
        let s = tier.name();
        assert!(["scalar", "avx2", "avx512", "neon"].contains(&s));
    }

    #[test]
    fn probe_os_page_size_power_of_two() {
        let ps = SystemProfile::probe().os_page_size;
        assert!(ps >= 4096);
        assert_eq!(ps & (ps - 1), 0, "page size must be power of two");
    }

    #[test]
    fn recommended_flush_threshold_in_bounds() {
        let t = SystemProfile::probe().recommended_flush_threshold;
        assert!(t >= 1_024);
        assert!(t <= 1_048_576);
    }

    #[test]
    fn recommended_partition_count_in_bounds() {
        let n = SystemProfile::probe().recommended_partition_count;
        assert!(n >= 1);
        assert!(n <= 64);
    }

    #[test]
    fn summary_contains_simd() {
        let s = SystemProfile::probe().summary();
        assert!(s.contains("SIMD="));
    }

    #[test]
    fn summary_contains_partitions() {
        let s = SystemProfile::probe().summary();
        assert!(s.contains("partitions="));
    }

    #[test]
    fn derive_flush_threshold_clamps_low() {
        let topo = CpuTopology::detect();
        let t = super::derive_flush_threshold(&topo);
        assert!(t >= 1_024);
    }

    #[test]
    fn derive_flush_threshold_clamps_high() {
        let topo = CpuTopology::detect();
        let t = super::derive_flush_threshold(&topo);
        assert!(t <= 1_048_576);
    }

    #[test]
    fn derive_partition_count_single_core() {
        assert_eq!(derive_partition_count(1), 1);
    }

    #[test]
    fn derive_partition_count_many_cores() {
        assert!(derive_partition_count(128) <= 64);
    }

    #[test]
    fn two_probes_agree() {
        let a = SystemProfile::probe();
        let b = SystemProfile::probe();
        assert_eq!(a.logical_cores, b.logical_cores);
        assert_eq!(a.simd_tier, b.simd_tier);
        assert_eq!(a.recommended_partition_count, b.recommended_partition_count);
    }
}
