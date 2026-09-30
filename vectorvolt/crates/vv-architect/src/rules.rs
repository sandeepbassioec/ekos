//! Cluster sizing rule engine — `vv-architect size`.
//!
//! Honest scope: these are transparent, documented heuristic formulas over
//! user-supplied workload estimates (TPS + data volume), not a claim of
//! profiling a real running system. Every constant below has a comment
//! explaining where it came from; change them freely if real benchmark data
//! (e.g. `vectorvolt-engine`'s own `tests/perf_gate.rs` floors) suggests
//! better ones later. There is no hidden magic: the same inputs always
//! produce the same outputs, and every field in [`SizingPlan`] is derived
//! from a one-line formula you can audit in `compute`.

use serde::Serialize;

/// Workload estimates the caller supplies — this tool has no way to measure
/// a real system's actual TPS or data volume, so these are honest inputs,
/// not something "detected".
#[derive(Debug, Clone, Copy)]
pub struct SizingInputs {
    pub estimated_tps: u64,
    pub estimated_data_gb: f64,
    /// Number of replicas per shard in addition to the primary (plan's
    /// example: "Repl factor: 2" -> 3 nodes = 1 primary + 2 replicas).
    pub replication_factor: u32,
}

#[derive(Debug, Clone, Serialize)]
pub struct SizingPlan {
    pub node_count: u32,
    pub cpu_cores_per_node: u32,
    pub writer_cores: u32,
    pub reader_cores: u32,
    pub ram_gb_per_node: u32,
    pub working_set_gb: u32,
    pub wal_buffer_gb: u32,
    pub disk_gb_per_node: u32,
    pub disk_type: &'static str,
    pub shard_count: u32,
    pub shards_per_node: u32,
    /// Suggested partition key, if a likely tenant/customer-scoping FK field
    /// was found among the scanned entities (caller passes this in — the
    /// rule engine itself has no entity knowledge).
    pub partition_key_hint: Option<String>,
    pub replication_factor: u32,
}

// ── tunable constants (documented, not magic) ────────────────────────────

/// NVMe capacity assumed available per node for the primary data set.
/// Conservative — real NVMe nodes today commonly exceed this.
const DISK_GB_PER_NODE: u32 = 500;
/// Fixed WAL buffer allowance per node regardless of workload size (matches
/// the plan's own "8 GB WAL buffer" example at 32 GB RAM).
const WAL_BUFFER_GB: u32 = 8;
/// Conservative single-core write throughput assumption (txns/sec/core) for
/// a WAL-backed OLTP write path — deliberately well below what
/// `vectorvolt-engine`'s own `tests/perf_gate.rs` measures, so this sizing
/// errs toward provisioning more, not less.
const WRITE_TXNS_PER_CORE_PER_SEC: u64 = 20_000;
/// Reads parallelize better than writes (no WAL/lock contention), so a read
/// core is assumed to do proportionally more.
const READ_MULTIPLIER_VS_WRITE: u64 = 3;
/// Fixed overhead cores: one dedicated WAL/fsync thread, one for OS/metrics.
const FIXED_OVERHEAD_CORES: u32 = 2;
/// Working-set RAM as a fraction of per-node data volume (assume hot data is
/// much smaller than total data at rest).
const WORKING_SET_FRACTION: f64 = 0.25;
/// Shards provisioned per node (matches the plan's own "12 (4/node)" example).
const SHARDS_PER_NODE: u32 = 4;

/// Minimum cluster size for availability even at near-zero workload: primary
/// + `replication_factor` replicas.
fn min_nodes_for_availability(replication_factor: u32) -> u32 {
    1 + replication_factor
}

/// Nodes needed so the (replicated) data volume fits within
/// `DISK_GB_PER_NODE` per node.
fn nodes_for_capacity(estimated_data_gb: f64, replication_factor: u32) -> u32 {
    let total_storage_needed = estimated_data_gb * (1.0 + replication_factor as f64);
    (total_storage_needed / DISK_GB_PER_NODE as f64)
        .ceil()
        .max(1.0) as u32
}

pub fn compute(inputs: SizingInputs, partition_key_hint: Option<String>) -> SizingPlan {
    let node_count = min_nodes_for_availability(inputs.replication_factor).max(nodes_for_capacity(
        inputs.estimated_data_gb,
        inputs.replication_factor,
    ));

    let tps_per_node = inputs.estimated_tps.div_ceil(node_count as u64);
    let writer_cores = tps_per_node.div_ceil(WRITE_TXNS_PER_CORE_PER_SEC).max(1) as u32;
    let reader_cores = tps_per_node
        .div_ceil(WRITE_TXNS_PER_CORE_PER_SEC * READ_MULTIPLIER_VS_WRITE)
        .max(1) as u32;
    let cpu_cores_per_node = writer_cores + reader_cores + FIXED_OVERHEAD_CORES;

    let data_gb_per_node = (inputs.estimated_data_gb / node_count as f64).max(0.0);
    let working_set_gb = (data_gb_per_node * WORKING_SET_FRACTION).ceil().max(1.0) as u32;
    let ram_gb_per_node = working_set_gb + WAL_BUFFER_GB;

    let disk_gb_per_node = (data_gb_per_node.ceil() as u32).max(DISK_GB_PER_NODE);

    let shard_count = node_count * SHARDS_PER_NODE;

    SizingPlan {
        node_count,
        cpu_cores_per_node,
        writer_cores,
        reader_cores,
        ram_gb_per_node,
        working_set_gb,
        wal_buffer_gb: WAL_BUFFER_GB,
        disk_gb_per_node,
        disk_type: "NVMe",
        shard_count,
        shards_per_node: SHARDS_PER_NODE,
        partition_key_hint,
        replication_factor: inputs.replication_factor,
    }
}

pub fn render(plan: &SizingPlan) -> String {
    let mut out = String::new();
    out.push_str(&format!(
        "Cluster:   {} nodes (1 primary + {} replica(s))\n",
        plan.node_count, plan.replication_factor
    ));
    out.push_str(&format!(
        "CPU:       {} cores/node ({} writer, {} reader, 1 WAL, 1 OS)\n",
        plan.cpu_cores_per_node, plan.writer_cores, plan.reader_cores
    ));
    out.push_str(&format!(
        "RAM:       {} GB/node  ({} GB working set, {} GB WAL buffer)\n",
        plan.ram_gb_per_node, plan.working_set_gb, plan.wal_buffer_gb
    ));
    out.push_str(&format!(
        "Disk:      {} GB/node {} recommended\n",
        plan.disk_gb_per_node, plan.disk_type
    ));
    match &plan.partition_key_hint {
        Some(key) => out.push_str(&format!(
            "Shards:    {} ({}/node), partitioned by {} HASH\n",
            plan.shard_count, plan.shards_per_node, key
        )),
        None => out.push_str(&format!(
            "Shards:    {} ({}/node), partitioned by primary key HASH\n",
            plan.shard_count, plan.shards_per_node
        )),
    }
    out.push_str(&format!("Repl factor: {}\n", plan.replication_factor));
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tiny_workload_still_gets_minimum_availability_cluster() {
        let plan = compute(
            SizingInputs {
                estimated_tps: 10,
                estimated_data_gb: 1.0,
                replication_factor: 2,
            },
            None,
        );
        assert_eq!(plan.node_count, 3); // 1 primary + 2 replicas, matches plan example
    }

    #[test]
    fn large_data_volume_scales_node_count_for_capacity() {
        // 10 TB replicated x3 = 30 TB total / 500 GB per node = 60 nodes,
        // which exceeds the min-availability floor of 3.
        let plan = compute(
            SizingInputs {
                estimated_tps: 100,
                estimated_data_gb: 10_000.0,
                replication_factor: 2,
            },
            None,
        );
        assert_eq!(plan.node_count, 60);
    }

    #[test]
    fn high_tps_increases_writer_cores() {
        let low = compute(
            SizingInputs {
                estimated_tps: 1_000,
                estimated_data_gb: 1.0,
                replication_factor: 2,
            },
            None,
        );
        let high = compute(
            SizingInputs {
                estimated_tps: 1_000_000,
                estimated_data_gb: 1.0,
                replication_factor: 2,
            },
            None,
        );
        assert!(high.writer_cores > low.writer_cores);
        assert!(high.cpu_cores_per_node > low.cpu_cores_per_node);
    }

    #[test]
    fn shard_count_is_proportional_to_node_count() {
        let plan = compute(
            SizingInputs {
                estimated_tps: 100,
                estimated_data_gb: 1.0,
                replication_factor: 2,
            },
            None,
        );
        assert_eq!(plan.shard_count, plan.node_count * SHARDS_PER_NODE);
    }

    #[test]
    fn partition_key_hint_flows_through_to_plan() {
        let plan = compute(
            SizingInputs {
                estimated_tps: 100,
                estimated_data_gb: 1.0,
                replication_factor: 2,
            },
            Some("tenant_id".to_owned()),
        );
        assert_eq!(plan.partition_key_hint.as_deref(), Some("tenant_id"));
        assert!(render(&plan).contains("tenant_id HASH"));
    }

    #[test]
    fn zero_tps_still_produces_minimum_one_core_each() {
        let plan = compute(
            SizingInputs {
                estimated_tps: 0,
                estimated_data_gb: 1.0,
                replication_factor: 0,
            },
            None,
        );
        assert_eq!(plan.writer_cores, 1);
        assert_eq!(plan.reader_cores, 1);
        assert_eq!(plan.node_count, 1); // no replicas requested
    }

    #[test]
    fn render_matches_plan_example_shape() {
        let plan = compute(
            SizingInputs {
                estimated_tps: 50_000,
                estimated_data_gb: 400.0,
                replication_factor: 2,
            },
            None,
        );
        let text = render(&plan);
        assert!(text.starts_with("Cluster:"));
        assert!(text.contains("CPU:"));
        assert!(text.contains("RAM:"));
        assert!(text.contains("Disk:"));
        assert!(text.contains("Shards:"));
        assert!(text.contains("Repl factor: 2"));
    }
}
