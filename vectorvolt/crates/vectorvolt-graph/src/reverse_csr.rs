use crate::csr::CsrGraph;

pub struct ReverseCsr {
    forward: CsrGraph,
    reverse: CsrGraph,
}

impl ReverseCsr {
    pub fn from_edges(edges: &[(u64, u64)]) -> Self {
        let forward = CsrGraph::from_edges(edges);
        let rev_edges: Vec<(u64, u64)> = edges.iter().map(|&(s, d)| (d, s)).collect();
        let reverse = CsrGraph::from_edges(&rev_edges);
        Self { forward, reverse }
    }

    pub fn successors(&self, node: u64) -> &[u64] {
        self.forward.neighbors(node)
    }

    pub fn predecessors(&self, node: u64) -> &[u64] {
        self.reverse.neighbors(node)
    }

    pub fn in_degree(&self, node: u64) -> usize {
        self.reverse.out_degree(node)
    }

    pub fn out_degree(&self, node: u64) -> usize {
        self.forward.out_degree(node)
    }

    pub fn node_count(&self) -> usize {
        self.forward.node_count()
    }

    pub fn edge_count(&self) -> usize {
        self.forward.edge_count()
    }

    pub fn forward(&self) -> &CsrGraph {
        &self.forward
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn successors_correct() {
        let g = ReverseCsr::from_edges(&[(1, 2), (1, 3)]);
        let s = g.successors(1);
        assert!(s.contains(&2));
        assert!(s.contains(&3));
    }

    #[test]
    fn predecessors_correct() {
        let g = ReverseCsr::from_edges(&[(1, 3), (2, 3)]);
        let p = g.predecessors(3);
        assert!(p.contains(&1));
        assert!(p.contains(&2));
    }

    #[test]
    fn in_degree_correct() {
        let g = ReverseCsr::from_edges(&[(1, 3), (2, 3), (4, 3)]);
        assert_eq!(g.in_degree(3), 3);
    }

    #[test]
    fn out_degree_correct() {
        let g = ReverseCsr::from_edges(&[(1, 2), (1, 3)]);
        assert_eq!(g.out_degree(1), 2);
    }

    #[test]
    fn node_count() {
        let g = ReverseCsr::from_edges(&[(1, 2), (2, 3)]);
        assert_eq!(g.node_count(), 3);
    }

    #[test]
    fn edge_count() {
        let g = ReverseCsr::from_edges(&[(1, 2), (2, 3), (3, 4)]);
        assert_eq!(g.edge_count(), 3);
    }

    #[test]
    fn empty_graph() {
        let g = ReverseCsr::from_edges(&[]);
        assert_eq!(g.node_count(), 0);
        assert_eq!(g.edge_count(), 0);
    }

    #[test]
    fn self_loop_predecessors() {
        let g = ReverseCsr::from_edges(&[(5, 5)]);
        assert!(g.predecessors(5).contains(&5));
    }

    #[test]
    fn no_predecessors_for_source() {
        let g = ReverseCsr::from_edges(&[(1, 2)]);
        assert_eq!(g.predecessors(1).len(), 0);
    }

    #[test]
    fn no_successors_for_sink() {
        let g = ReverseCsr::from_edges(&[(1, 2)]);
        assert_eq!(g.successors(2).len(), 0);
    }

    #[test]
    fn forward_matches_csr() {
        let edges = &[(1u64, 2u64), (2, 3)];
        let g = ReverseCsr::from_edges(edges);
        assert_eq!(g.forward().edge_count(), 2);
    }

    #[test]
    fn parallel_edges_deduplicated() {
        let g = ReverseCsr::from_edges(&[(1, 2), (1, 2)]);
        assert_eq!(g.edge_count(), 1);
    }

    #[test]
    fn large_graph() {
        let edges: Vec<(u64, u64)> = (0u64..500).map(|i| (i, i + 500)).collect();
        let g = ReverseCsr::from_edges(&edges);
        assert_eq!(g.edge_count(), 500);
        assert_eq!(g.in_degree(500), 1);
    }
}
