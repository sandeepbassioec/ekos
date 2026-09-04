use tempfile::tempdir;
use vectorvolt_graph::{
    analytics::{degree_centrality, pagerank},
    cascade::CascadeDelete,
    csr::CsrGraph,
    reverse_csr::ReverseCsr,
    simd_join::{common_neighbors, intersect_sorted, jaccard_similarity},
    store::GraphStore,
    traversal::{bfs, connected_components, dfs, reachable, shortest_path},
};

// ── L1+L2: CSR + ReverseCsr ───────────────────────────────────────────────────

#[test]
fn csr_and_reverse_agree_on_edge_count() {
    let edges = &[(1u64, 2), (2, 3), (3, 4), (1, 3)];
    let csr = CsrGraph::from_edges(edges);
    let rev = ReverseCsr::from_edges(edges);
    assert_eq!(csr.edge_count(), rev.edge_count());
    assert_eq!(csr.node_count(), rev.node_count());
}

#[test]
fn reverse_csr_predecessors_match_manual() {
    let rev = ReverseCsr::from_edges(&[(1, 3), (2, 3), (4, 3)]);
    let preds = rev.predecessors(3);
    assert_eq!(preds.len(), 3);
    assert!(preds.contains(&1));
    assert!(preds.contains(&2));
    assert!(preds.contains(&4));
}

// ── L3: traversal ─────────────────────────────────────────────────────────────

#[test]
fn bfs_and_dfs_visit_same_nodes_on_tree() {
    let g = CsrGraph::from_edges(&[(1, 2), (1, 3), (2, 4), (3, 5)]);
    let bfs_nodes: std::collections::HashSet<u64> =
        bfs(&g, 1, 10).into_iter().map(|(n, _)| n).collect();
    let dfs_nodes: std::collections::HashSet<u64> =
        dfs(&g, 1, 10).into_iter().map(|(n, _)| n).collect();
    assert_eq!(bfs_nodes, dfs_nodes);
    assert_eq!(bfs_nodes.len(), 5);
}

#[test]
fn reachable_excludes_unreachable_component() {
    let g = CsrGraph::from_edges(&[(1, 2), (3, 4)]);
    let r = reachable(&g, 1);
    assert!(r.contains(&1));
    assert!(r.contains(&2));
    assert!(!r.contains(&3));
    assert!(!r.contains(&4));
}

#[test]
fn shortest_path_chain_length() {
    let g = CsrGraph::from_edges(&[(1, 2), (2, 3), (3, 4), (4, 5)]);
    let path = shortest_path(&g, 1, 5).unwrap();
    assert_eq!(path.len(), 5);
    assert_eq!(path[0], 1);
    assert_eq!(path[4], 5);
}

#[test]
fn connected_components_count() {
    let g = CsrGraph::from_edges(&[(1, 2), (3, 4), (5, 6)]);
    let comps = connected_components(&g);
    assert_eq!(comps.len(), 3);
}

// ── L4: cascade ───────────────────────────────────────────────────────────────

#[test]
fn cascade_deletion_order_respects_dependencies() {
    // 1->2, 2->3, 1->3: delete 3, predecessors are {1,2}
    let edges = &[(1u64, 3), (2, 3)];
    let rev = ReverseCsr::from_edges(edges);
    let c = CascadeDelete::new(rev);
    let order = c.deletion_order(3);
    let pos_3 = order.iter().position(|&x| x == 3).unwrap();
    // predecessors must appear before the root
    for &pred in &[1u64, 2u64] {
        let pos = order.iter().position(|&x| x == pred).unwrap();
        assert!(pos < pos_3, "predecessor {pred} should come before 3");
    }
}

#[test]
fn cascade_has_cycle_detects_correctly() {
    let c_cycle = CascadeDelete::new(ReverseCsr::from_edges(&[(1, 2), (2, 3), (3, 1)]));
    let c_dag = CascadeDelete::new(ReverseCsr::from_edges(&[(1, 2), (2, 3)]));
    assert!(c_cycle.has_cycle());
    assert!(!c_dag.has_cycle());
}

// ── L5: analytics ─────────────────────────────────────────────────────────────

#[test]
fn pagerank_sums_to_one_on_larger_graph() {
    let edges: Vec<(u64, u64)> = (0u64..5)
        .flat_map(|i| (i + 1..5).map(move |j| (i, j)))
        .collect();
    let g = CsrGraph::from_edges(&edges);
    let r = pagerank(&g, 0.85, 30);
    let total: f64 = r.values().sum();
    assert!((total - 1.0).abs() < 1e-5, "total={total}");
}

#[test]
fn degree_centrality_hub_is_highest() {
    // node 0 points to all others
    let edges: Vec<(u64, u64)> = (1u64..5).map(|j| (0, j)).collect();
    let g = CsrGraph::from_edges(&edges);
    let c = degree_centrality(&g);
    let hub = c[&0];
    for j in 1u64..5 {
        assert!(hub >= c[&j]);
    }
}

// ── L6: SIMD join ─────────────────────────────────────────────────────────────

#[test]
fn intersect_sorted_large() {
    let a: Vec<u64> = (0u64..100).filter(|x| x % 2 == 0).collect();
    let b: Vec<u64> = (0u64..100).filter(|x| x % 3 == 0).collect();
    let result = intersect_sorted(&a, &b);
    // multiples of 6 up to 99: 0,6,12,...,96 = 17 values
    assert_eq!(result.len(), 17);
}

#[test]
fn common_neighbors_and_jaccard_consistent() {
    let g = CsrGraph::from_edges(&[(1, 3), (1, 4), (2, 3), (2, 4), (2, 5)]);
    let cn = common_neighbors(&g, 1, 2);
    let j = jaccard_similarity(&g, 1, 2);
    // common = {3,4}, union = {3,4,5} => jaccard = 2/3
    assert_eq!(cn.len(), 2);
    assert!((j - 2.0 / 3.0).abs() < 1e-9);
}

// ── L7: store ─────────────────────────────────────────────────────────────────

#[test]
fn store_full_lifecycle_persist_and_reload() {
    let dir = tempdir().unwrap();
    // write
    {
        let s = GraphStore::open(dir.path()).unwrap();
        for i in 0u64..5 {
            s.add_edge(i, i + 1).unwrap();
        }
        s.save().unwrap();
    }
    // reload
    let s2 = GraphStore::open(dir.path()).unwrap();
    let stats = s2.stats();
    assert_eq!(stats.edge_count, 5);
    assert_eq!(stats.node_count, 6);
    // BFS from 0 should reach all 6 nodes
    let snap = s2.snapshot();
    let visited = bfs(snap.forward(), 0, 10);
    assert_eq!(visited.len(), 6);
}

#[test]
fn store_cascade_remove_cleans_dependents() {
    let dir = tempdir().unwrap();
    let s = GraphStore::open(dir.path()).unwrap();
    // 1->3, 2->3, 3->4
    s.add_edge(1, 3).unwrap();
    s.add_edge(2, 3).unwrap();
    s.add_edge(3, 4).unwrap();
    // removing 3: predecessors = {1,2}
    let deleted = s.remove_node(3).unwrap();
    assert!(deleted.contains(&3));
    assert!(deleted.contains(&1));
    assert!(deleted.contains(&2));
    let snap = s.snapshot();
    // all edges involving 1,2,3 should be gone
    assert!(!snap.forward().contains_edge(1, 3));
    assert!(!snap.forward().contains_edge(2, 3));
    assert!(!snap.forward().contains_edge(3, 4));
}
