use std::collections::HashMap;

/// Compressed Sparse Row adjacency list.
pub struct CsrGraph {
    row_ptr: Vec<u32>,
    col_idx: Vec<u64>,
    node_ids: Vec<u64>,
    id_to_row: HashMap<u64, u32>,
}

impl CsrGraph {
    pub fn from_edges(edges: &[(u64, u64)]) -> Self {
        let mut node_set: std::collections::BTreeSet<u64> = std::collections::BTreeSet::new();
        for &(s, d) in edges {
            node_set.insert(s);
            node_set.insert(d);
        }
        let node_ids: Vec<u64> = node_set.into_iter().collect();
        let id_to_row: HashMap<u64, u32> = node_ids
            .iter()
            .enumerate()
            .map(|(i, &id)| (id, i as u32))
            .collect();

        let n = node_ids.len();
        let mut adj: Vec<std::collections::BTreeSet<u64>> =
            vec![std::collections::BTreeSet::new(); n];
        for &(s, d) in edges {
            let row = id_to_row[&s] as usize;
            adj[row].insert(d);
        }

        let mut row_ptr = Vec::with_capacity(n + 1);
        let mut col_idx = Vec::new();
        row_ptr.push(0u32);
        for neighbors in &adj {
            for &nb in neighbors {
                col_idx.push(nb);
            }
            row_ptr.push(col_idx.len() as u32);
        }

        Self {
            row_ptr,
            col_idx,
            node_ids,
            id_to_row,
        }
    }

    pub fn neighbors(&self, node: u64) -> &[u64] {
        if let Some(&row) = self.id_to_row.get(&node) {
            let start = self.row_ptr[row as usize] as usize;
            let end = self.row_ptr[row as usize + 1] as usize;
            &self.col_idx[start..end]
        } else {
            &[]
        }
    }

    pub fn out_degree(&self, node: u64) -> usize {
        self.neighbors(node).len()
    }

    pub fn node_count(&self) -> usize {
        self.node_ids.len()
    }

    pub fn edge_count(&self) -> usize {
        self.col_idx.len()
    }

    pub fn contains_node(&self, node: u64) -> bool {
        self.id_to_row.contains_key(&node)
    }

    pub fn contains_edge(&self, src: u64, dst: u64) -> bool {
        let nbs = self.neighbors(src);
        nbs.binary_search(&dst).is_ok()
    }

    pub fn nodes(&self) -> &[u64] {
        &self.node_ids
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_edges_3_nodes() {
        let g = CsrGraph::from_edges(&[(1, 2), (2, 3)]);
        assert_eq!(g.node_count(), 3);
    }

    #[test]
    fn neighbors_correct() {
        let g = CsrGraph::from_edges(&[(1, 2), (1, 3)]);
        let nbs = g.neighbors(1);
        assert!(nbs.contains(&2));
        assert!(nbs.contains(&3));
    }

    #[test]
    fn out_degree_correct() {
        let g = CsrGraph::from_edges(&[(1, 2), (1, 3), (2, 3)]);
        assert_eq!(g.out_degree(1), 2);
        assert_eq!(g.out_degree(2), 1);
    }

    #[test]
    fn node_count() {
        let g = CsrGraph::from_edges(&[(10, 20), (20, 30), (30, 40)]);
        assert_eq!(g.node_count(), 4);
    }

    #[test]
    fn edge_count() {
        let g = CsrGraph::from_edges(&[(1, 2), (2, 3), (3, 4)]);
        assert_eq!(g.edge_count(), 3);
    }

    #[test]
    fn contains_node_true() {
        let g = CsrGraph::from_edges(&[(1, 2)]);
        assert!(g.contains_node(1));
        assert!(g.contains_node(2));
    }

    #[test]
    fn contains_node_false() {
        let g = CsrGraph::from_edges(&[(1, 2)]);
        assert!(!g.contains_node(99));
    }

    #[test]
    fn contains_edge_true() {
        let g = CsrGraph::from_edges(&[(1, 2), (2, 3)]);
        assert!(g.contains_edge(1, 2));
        assert!(g.contains_edge(2, 3));
    }

    #[test]
    fn contains_edge_false() {
        let g = CsrGraph::from_edges(&[(1, 2)]);
        assert!(!g.contains_edge(2, 1));
        assert!(!g.contains_edge(1, 99));
    }

    #[test]
    fn empty_graph() {
        let g = CsrGraph::from_edges(&[]);
        assert_eq!(g.node_count(), 0);
        assert_eq!(g.edge_count(), 0);
    }

    #[test]
    fn self_loop() {
        let g = CsrGraph::from_edges(&[(5, 5)]);
        assert!(g.contains_edge(5, 5));
    }

    #[test]
    fn parallel_edges_deduplicated() {
        let g = CsrGraph::from_edges(&[(1, 2), (1, 2)]);
        assert_eq!(g.edge_count(), 1);
    }

    #[test]
    fn large_graph_1000() {
        let edges: Vec<(u64, u64)> = (0u64..1000).map(|i| (i, i + 1000)).collect();
        let g = CsrGraph::from_edges(&edges);
        assert_eq!(g.edge_count(), 1000);
    }
}
