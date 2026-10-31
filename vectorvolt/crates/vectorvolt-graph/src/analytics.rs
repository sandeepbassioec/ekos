use crate::csr::CsrGraph;
use std::collections::HashMap;

pub fn pagerank(graph: &CsrGraph, damping: f64, iterations: usize) -> HashMap<u64, f64> {
    let n = graph.node_count();
    if n == 0 {
        return HashMap::new();
    }
    let nodes: Vec<u64> = graph.nodes().to_vec();
    let mut rank: HashMap<u64, f64> = nodes.iter().map(|&id| (id, 1.0 / n as f64)).collect();

    for _ in 0..iterations {
        let mut new_rank: HashMap<u64, f64> = nodes
            .iter()
            .map(|&id| (id, (1.0 - damping) / n as f64))
            .collect();
        for &node in &nodes {
            let out_deg = graph.out_degree(node);
            if out_deg == 0 {
                let contrib = rank[&node] * damping / n as f64;
                for &other in &nodes {
                    *new_rank.get_mut(&other).unwrap() += contrib;
                }
            } else {
                let contrib = rank[&node] * damping / out_deg as f64;
                for &nb in graph.neighbors(node) {
                    if let Some(r) = new_rank.get_mut(&nb) {
                        *r += contrib;
                    }
                }
            }
        }
        rank = new_rank;
    }
    rank
}

pub fn degree_centrality(graph: &CsrGraph) -> HashMap<u64, f64> {
    let n = graph.node_count();
    if n <= 1 {
        return graph.nodes().iter().map(|&id| (id, 0.0)).collect();
    }
    let max_deg = (n - 1) as f64;
    graph
        .nodes()
        .iter()
        .map(|&id| (id, graph.out_degree(id) as f64 / max_deg))
        .collect()
}

pub fn clustering_coefficient(graph: &CsrGraph, node: u64) -> f64 {
    let neighbors: Vec<u64> = graph.neighbors(node).to_vec();
    let k = neighbors.len();
    if k < 2 {
        return 0.0;
    }
    let neighbor_set: std::collections::HashSet<u64> = neighbors.iter().copied().collect();
    let mut triangles = 0usize;
    for &nb in &neighbors {
        for &nb2 in graph.neighbors(nb) {
            if neighbor_set.contains(&nb2) {
                triangles += 1;
            }
        }
    }
    triangles as f64 / (k * (k - 1)) as f64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pagerank_empty() {
        let g = CsrGraph::from_edges(&[]);
        let r = pagerank(&g, 0.85, 20);
        assert!(r.is_empty());
    }

    #[test]
    fn pagerank_sums_to_one() {
        let g = CsrGraph::from_edges(&[(1, 2), (2, 3), (3, 1)]);
        let r = pagerank(&g, 0.85, 20);
        let total: f64 = r.values().sum();
        assert!((total - 1.0).abs() < 1e-6);
    }

    #[test]
    fn pagerank_uniform_cycle() {
        let g = CsrGraph::from_edges(&[(1, 2), (2, 3), (3, 1)]);
        let r = pagerank(&g, 0.85, 100);
        let v1 = r[&1];
        let v2 = r[&2];
        assert!((v1 - v2).abs() < 1e-6);
    }

    #[test]
    fn pagerank_hub_ranks_higher() {
        let g = CsrGraph::from_edges(&[(1, 3), (2, 3)]);
        let r = pagerank(&g, 0.85, 50);
        assert!(r[&3] > r[&1]);
    }

    #[test]
    fn degree_centrality_empty() {
        let g = CsrGraph::from_edges(&[]);
        let c = degree_centrality(&g);
        assert!(c.is_empty());
    }

    #[test]
    fn degree_centrality_single() {
        let g = CsrGraph::from_edges(&[(1, 2)]);
        let c = degree_centrality(&g);
        assert!((c[&1] - 1.0).abs() < 1e-9);
        assert!((c[&2]).abs() < 1e-9);
    }

    #[test]
    fn degree_centrality_range() {
        let g = CsrGraph::from_edges(&[(1, 2), (1, 3), (2, 3)]);
        let c = degree_centrality(&g);
        for &v in c.values() {
            assert!((0.0..=1.0).contains(&v));
        }
    }

    #[test]
    fn clustering_no_neighbors() {
        let g = CsrGraph::from_edges(&[(1, 2)]);
        assert_eq!(clustering_coefficient(&g, 2), 0.0);
    }

    #[test]
    fn clustering_one_neighbor() {
        let g = CsrGraph::from_edges(&[(1, 2)]);
        assert_eq!(clustering_coefficient(&g, 1), 0.0);
    }

    #[test]
    fn clustering_triangle() {
        let g = CsrGraph::from_edges(&[(1, 2), (1, 3), (2, 3)]);
        let cc = clustering_coefficient(&g, 1);
        assert!(cc > 0.0);
    }

    #[test]
    fn clustering_no_triangle() {
        let g = CsrGraph::from_edges(&[(1, 2), (1, 3)]);
        let cc = clustering_coefficient(&g, 1);
        assert_eq!(cc, 0.0);
    }

    #[test]
    fn pagerank_all_nodes_present() {
        let g = CsrGraph::from_edges(&[(1, 2), (2, 3), (3, 4)]);
        let r = pagerank(&g, 0.85, 20);
        assert_eq!(r.len(), 4);
        assert!(r.contains_key(&1));
        assert!(r.contains_key(&4));
    }

    #[test]
    fn degree_centrality_all_nodes_present() {
        let g = CsrGraph::from_edges(&[(1, 2), (2, 3)]);
        let c = degree_centrality(&g);
        assert_eq!(c.len(), 3);
    }
}
