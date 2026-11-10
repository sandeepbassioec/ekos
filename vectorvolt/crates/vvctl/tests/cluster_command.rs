//! Integration tests for `vvctl cluster` (VV-CLU-CLI-001..006).

use std::net::SocketAddr;
use tempfile::TempDir;
use vvctl::commands::cluster_cmd::{ClusterArgs, ClusterCommand, run};

fn tmp() -> TempDir {
    tempfile::tempdir().unwrap()
}

fn state_file(dir: &TempDir) -> std::path::PathBuf {
    dir.path().join("cluster.json")
}

fn args(dir: &TempDir, command: ClusterCommand) -> ClusterArgs {
    ClusterArgs {
        dir: dir.path().to_path_buf(),
        node: None,
        command,
    }
}

#[test]
fn init_creates_state_file() {
    let dir = tmp();
    run(args(
        &dir,
        ClusterCommand::Init {
            name: "test-cluster".into(),
            replication_factor: 3,
            vnode_count: 256,
        },
    ))
    .unwrap();
    assert!(state_file(&dir).exists());
}

#[test]
fn nodes_on_empty_cluster_does_not_panic() {
    let dir = tmp();
    run(args(
        &dir,
        ClusterCommand::Init {
            name: "c".into(),
            replication_factor: 1,
            vnode_count: 64,
        },
    ))
    .unwrap();
    run(args(&dir, ClusterCommand::Nodes)).unwrap();
}

#[test]
fn join_adds_node_and_persists() {
    let dir = tmp();
    run(args(
        &dir,
        ClusterCommand::Init {
            name: "c".into(),
            replication_factor: 1,
            vnode_count: 64,
        },
    ))
    .unwrap();
    let addr: SocketAddr = "127.0.0.1:7001".parse().unwrap();
    run(args(&dir, ClusterCommand::Join { addr })).unwrap();
    let raw = std::fs::read_to_string(state_file(&dir)).unwrap();
    assert!(raw.contains("7001"));
}

#[test]
fn join_idempotent() {
    let dir = tmp();
    run(args(
        &dir,
        ClusterCommand::Init {
            name: "c".into(),
            replication_factor: 1,
            vnode_count: 64,
        },
    ))
    .unwrap();
    let addr: SocketAddr = "127.0.0.1:7002".parse().unwrap();
    run(args(&dir, ClusterCommand::Join { addr })).unwrap();
    run(args(&dir, ClusterCommand::Join { addr })).unwrap();
    let raw = std::fs::read_to_string(state_file(&dir)).unwrap();
    // Should appear exactly once
    let count = raw.matches("7002").count();
    assert_eq!(count, 1, "expected 1 occurrence of port, got {count}");
}

#[test]
fn leave_marks_node_left() {
    let dir = tmp();
    run(args(
        &dir,
        ClusterCommand::Init {
            name: "c".into(),
            replication_factor: 1,
            vnode_count: 64,
        },
    ))
    .unwrap();
    let addr: SocketAddr = "127.0.0.1:7003".parse().unwrap();
    run(args(&dir, ClusterCommand::Join { addr })).unwrap();
    run(args(&dir, ClusterCommand::Leave { addr: Some(addr) })).unwrap();
    let raw = std::fs::read_to_string(state_file(&dir)).unwrap();
    assert!(raw.contains("Left"));
}

#[test]
fn leave_unknown_node_errors() {
    let dir = tmp();
    run(args(
        &dir,
        ClusterCommand::Init {
            name: "c".into(),
            replication_factor: 1,
            vnode_count: 64,
        },
    ))
    .unwrap();
    let addr: SocketAddr = "127.0.0.1:9999".parse().unwrap();
    assert!(run(args(&dir, ClusterCommand::Leave { addr: Some(addr) })).is_err());
}

#[test]
fn status_runs_after_join() {
    let dir = tmp();
    run(args(
        &dir,
        ClusterCommand::Init {
            name: "c".into(),
            replication_factor: 3,
            vnode_count: 256,
        },
    ))
    .unwrap();
    let addr: SocketAddr = "127.0.0.1:7004".parse().unwrap();
    run(args(&dir, ClusterCommand::Join { addr })).unwrap();
    run(args(&dir, ClusterCommand::Status)).unwrap();
}

#[test]
fn ring_lookup_returns_owner() {
    let dir = tmp();
    run(args(
        &dir,
        ClusterCommand::Init {
            name: "c".into(),
            replication_factor: 1,
            vnode_count: 256,
        },
    ))
    .unwrap();
    let addr: SocketAddr = "127.0.0.1:7005".parse().unwrap();
    run(args(&dir, ClusterCommand::Join { addr })).unwrap();
    run(args(&dir, ClusterCommand::Ring { key: Some(42) })).unwrap();
}

#[test]
fn ring_empty_cluster_does_not_crash() {
    let dir = tmp();
    run(args(
        &dir,
        ClusterCommand::Init {
            name: "c".into(),
            replication_factor: 1,
            vnode_count: 64,
        },
    ))
    .unwrap();
    run(args(&dir, ClusterCommand::Ring { key: Some(0) })).unwrap();
}
