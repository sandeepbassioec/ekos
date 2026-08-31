//! Integration tests for Layer 180: vvctl topology command (VV-CTL-180).

use vvctl::commands::topology::{TopologyArgs, run};

fn args(cores: bool, numa: bool) -> TopologyArgs {
    TopologyArgs { cores, numa }
}

#[test]
fn topology_default_ok() {
    assert!(run(args(false, false)).is_ok());
}

#[test]
fn topology_cores_ok() {
    assert!(run(args(true, false)).is_ok());
}

#[test]
fn topology_numa_ok() {
    assert!(run(args(false, true)).is_ok());
}

#[test]
fn topology_cores_and_numa_ok() {
    assert!(run(args(true, true)).is_ok());
}

#[test]
fn topology_repeated_calls_ok() {
    assert!(run(args(false, false)).is_ok());
    assert!(run(args(false, false)).is_ok());
}

#[test]
fn topology_cores_false_ok() {
    assert!(run(args(false, false)).is_ok());
}

#[test]
fn topology_numa_false_ok() {
    assert!(run(args(false, false)).is_ok());
}

#[test]
fn topology_both_false_ok() {
    assert!(run(args(false, false)).is_ok());
}

#[test]
fn topology_probe_succeeds() {
    let topo = vectorvolt_hal::CpuTopology::detect();
    assert!(
        !topo.logical.is_empty(),
        "must have at least 1 logical core"
    );
}

#[test]
fn topology_logical_ge_1() {
    let topo = vectorvolt_hal::CpuTopology::detect();
    assert!(topo.logical.len() >= 1);
}

#[test]
fn topology_numa_ge_1() {
    let topo = vectorvolt_hal::CpuTopology::detect();
    assert!(!topo.numa_nodes.is_empty());
}

#[test]
fn topology_run_is_fast() {
    let start = std::time::Instant::now();
    run(args(true, true)).unwrap();
    assert!(start.elapsed().as_secs() < 5);
}

#[test]
fn topology_cores_list_consistent() {
    let topo = vectorvolt_hal::CpuTopology::detect();
    for lc in &topo.logical {
        assert!(lc.id < 1024, "core id suspiciously large: {}", lc.id);
    }
}
