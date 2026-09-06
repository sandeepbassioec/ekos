//! Tests for vvctl cluster-topology (VV-CTL-005).
use vvctl::commands::cluster_topology::{
    NodeInfo, NodeRole, format_json, format_table, format_tree,
};

fn leader() -> NodeInfo {
    NodeInfo {
        id: "node-1".into(),
        addr: "127.0.0.1:50051".into(),
        role: NodeRole::Leader,
        shard_count: 100,
    }
}

fn follower(id: &str, addr: &str) -> NodeInfo {
    NodeInfo {
        id: id.into(),
        addr: addr.into(),
        role: NodeRole::Follower,
        shard_count: 50,
    }
}

fn dead_node() -> NodeInfo {
    NodeInfo {
        id: "node-dead".into(),
        addr: "127.0.0.9:50051".into(),
        role: NodeRole::Dead,
        shard_count: 0,
    }
}

#[test]
fn test_one_node_tree() {
    let nodes = vec![leader()];
    let out = format_tree(&nodes, 100);
    assert!(out.contains("node-1"));
}

#[test]
fn test_three_nodes_shows_all() {
    let nodes = vec![
        leader(),
        follower("node-2", "127.0.0.2:50051"),
        follower("node-3", "127.0.0.3:50051"),
    ];
    let out = format_tree(&nodes, 200);
    assert!(out.contains("node-1"));
    assert!(out.contains("node-2"));
    assert!(out.contains("node-3"));
}

#[test]
fn test_leader_marked() {
    let nodes = vec![leader()];
    let out = format_tree(&nodes, 100);
    assert!(out.contains("LEADER"));
}

#[test]
fn test_follower_marked() {
    let nodes = [leader(), follower("node-2", "127.0.0.2:50051")];
    let out = format_tree(&nodes, 150);
    assert!(out.contains("FOLLOWER"));
}

#[test]
fn test_shard_counts_visible() {
    let nodes = [leader(), follower("node-2", "127.0.0.2:50051")];
    let total: usize = nodes.iter().map(|n| n.shard_count).sum();
    let out = format_tree(&nodes, total);
    assert!(out.contains("150"));
}

#[test]
fn test_json_valid() {
    let nodes = vec![leader()];
    let j = format_json(&nodes);
    let v: serde_json::Value = serde_json::from_str(&j).unwrap();
    assert!(v.is_object());
}

#[test]
fn test_json_has_nodes_array() {
    let nodes = vec![leader()];
    let j = format_json(&nodes);
    let v: serde_json::Value = serde_json::from_str(&j).unwrap();
    assert!(v["nodes"].is_array());
}

#[test]
fn test_json_node_has_required_fields() {
    let nodes = vec![leader()];
    let j = format_json(&nodes);
    let v: serde_json::Value = serde_json::from_str(&j).unwrap();
    let node = &v["nodes"][0];
    assert!(node["id"].is_string());
    assert!(node["addr"].is_string());
    assert!(node["shard_count"].is_number());
}

#[test]
fn test_empty_cluster_message() {
    let out = format_tree(&[], 0);
    assert!(out.contains("empty"));
}

#[test]
fn test_dead_node_shows_dead() {
    let nodes = vec![leader(), dead_node()];
    let out = format_tree(&nodes, 100);
    assert!(out.contains("DEAD"));
}

#[test]
fn test_shard_sum_equals_total() {
    let nodes = [leader(), follower("node-2", "127.0.0.2:50051")];
    let total: usize = nodes.iter().map(|n| n.shard_count).sum();
    assert_eq!(total, 150);
}

#[test]
fn test_tree_has_box_drawing_chars() {
    let nodes = [leader(), follower("node-2", "127.0.0.2:50051")];
    let out = format_tree(&nodes, 150);
    assert!(out.contains('├') || out.contains('└'));
}

#[test]
fn test_table_format_works() {
    let nodes = vec![leader()];
    let out = format_table(&nodes);
    assert!(out.contains("node-1"));
    assert!(out.contains("LEADER"));
}
