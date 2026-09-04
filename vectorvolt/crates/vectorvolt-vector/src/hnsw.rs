//! Simplified HNSW (Hierarchical Navigable Small World) index.
//!
//! This is a single-layer implementation that maintains the correct public API
//! for VectorVolt's vector search. The search path uses brute-force comparison
//! for correctness; the graph structure is maintained for future multi-layer use.

use std::collections::HashMap;

use crate::distance;

/// HNSW approximate nearest-neighbour index.
///
/// Each inserted vector is assigned an id (`u64`) and stored in a flat map.
/// Neighbours are tracked in a graph adjacency map. Search uses brute force
/// over all stored vectors, which is exact and correct for small indexes.
pub struct HnswIndex {
    m: usize,
    ef_construction: usize,
    vectors: HashMap<u64, Vec<f32>>,
    /// Layer-0 adjacency: node_id -> neighbour_ids
    graph: HashMap<u64, Vec<u64>>,
    entry_point: Option<u64>,
}

impl HnswIndex {
    /// Create a new HNSW index.
    ///
    /// * `m` — maximum number of neighbours per node
    /// * `ef_construction` — size of the dynamic candidate list during insert
    pub fn new(m: usize, ef_construction: usize) -> Self {
        Self {
            m,
            ef_construction,
            vectors: HashMap::new(),
            graph: HashMap::new(),
            entry_point: None,
        }
    }

    /// Insert a vector with the given id.
    ///
    /// Finds the `m` nearest existing neighbours by Euclidean distance and
    /// creates bidirectional edges. Sets the entry point if the index is empty.
    pub fn insert(&mut self, id: u64, vector: Vec<f32>) {
        self.vectors.insert(id, vector.clone());
        // Find m nearest neighbours
        let mut neighbours: Vec<(u64, f32)> = self
            .vectors
            .iter()
            .filter(|&(&nid, _)| nid != id)
            .map(|(&nid, v)| (nid, distance::euclidean_distance(&vector, v)))
            .collect();
        neighbours.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
        neighbours.truncate(self.m);
        let neighbour_ids: Vec<u64> = neighbours.iter().map(|(nid, _)| *nid).collect();
        // Add reverse edges (up to m)
        for &nid in &neighbour_ids {
            let e = self.graph.entry(nid).or_default();
            if !e.contains(&id) {
                e.push(id);
                if e.len() > self.m {
                    e.truncate(self.m);
                }
            }
        }
        self.graph.insert(id, neighbour_ids);
        if self.entry_point.is_none() {
            self.entry_point = Some(id);
        }
    }

    /// Search for the `k` nearest neighbours to `query`.
    ///
    /// Returns a list of `(id, distance)` pairs sorted by ascending distance.
    pub fn search(&self, query: &[f32], k: usize) -> Vec<(u64, f32)> {
        let mut results: Vec<(u64, f32)> = self
            .vectors
            .iter()
            .map(|(&id, v)| (id, distance::euclidean_distance(query, v)))
            .collect();
        results.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
        results.truncate(k);
        results
    }

    /// Number of vectors in the index.
    pub fn len(&self) -> usize {
        self.vectors.len()
    }

    /// True if the index contains no vectors.
    pub fn is_empty(&self) -> bool {
        self.vectors.is_empty()
    }

    /// True if the index contains a vector with the given id.
    pub fn contains(&self, id: u64) -> bool {
        self.vectors.contains_key(&id)
    }

    /// Return the stored vector for the given id, or `None`.
    pub fn get_vector(&self, id: u64) -> Option<&Vec<f32>> {
        self.vectors.get(&id)
    }

    /// Return the current entry point id, or `None` if the index is empty.
    pub fn entry_point(&self) -> Option<u64> {
        self.entry_point
    }

    /// Return the neighbours of a node at layer 0.
    pub fn neighbours(&self, id: u64) -> &[u64] {
        self.graph.get(&id).map(|v| v.as_slice()).unwrap_or(&[])
    }

    /// `ef_construction` value this index was created with.
    pub fn ef_construction(&self) -> usize {
        self.ef_construction
    }

    /// `m` value this index was created with.
    pub fn m(&self) -> usize {
        self.m
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_index() {
        let idx = HnswIndex::new(16, 200);
        assert!(idx.is_empty());
        assert_eq!(idx.len(), 0);
        assert!(idx.entry_point().is_none());
    }

    #[test]
    fn test_insert_single() {
        let mut idx = HnswIndex::new(16, 200);
        idx.insert(1, vec![1.0, 0.0, 0.0]);
        assert_eq!(idx.len(), 1);
        assert!(idx.contains(1));
        assert_eq!(idx.entry_point(), Some(1));
    }

    #[test]
    fn test_insert_multiple() {
        let mut idx = HnswIndex::new(4, 50);
        for i in 0..10u64 {
            idx.insert(i, vec![i as f32, 0.0]);
        }
        assert_eq!(idx.len(), 10);
    }

    #[test]
    fn test_search_nearest() {
        let mut idx = HnswIndex::new(4, 50);
        idx.insert(1, vec![0.0, 0.0]);
        idx.insert(2, vec![1.0, 0.0]);
        idx.insert(3, vec![10.0, 0.0]);
        let results = idx.search(&[0.1, 0.0], 1);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].0, 1); // closest to [0.0, 0.0]
    }

    #[test]
    fn test_search_k_results() {
        let mut idx = HnswIndex::new(4, 50);
        for i in 0..20u64 {
            idx.insert(i, vec![i as f32]);
        }
        let results = idx.search(&[10.0], 5);
        assert_eq!(results.len(), 5);
    }

    #[test]
    fn test_search_sorted_by_distance() {
        let mut idx = HnswIndex::new(4, 50);
        idx.insert(1, vec![0.0]);
        idx.insert(2, vec![2.0]);
        idx.insert(3, vec![5.0]);
        let results = idx.search(&[1.0], 3);
        assert!(results[0].1 <= results[1].1);
        assert!(results[1].1 <= results[2].1);
    }

    #[test]
    fn test_get_vector() {
        let mut idx = HnswIndex::new(4, 50);
        idx.insert(42, vec![1.0, 2.0, 3.0]);
        let v = idx.get_vector(42).unwrap();
        assert_eq!(v, &vec![1.0f32, 2.0, 3.0]);
    }

    #[test]
    fn test_get_vector_missing() {
        let idx = HnswIndex::new(4, 50);
        assert!(idx.get_vector(99).is_none());
    }

    #[test]
    fn test_contains() {
        let mut idx = HnswIndex::new(4, 50);
        idx.insert(7, vec![1.0]);
        assert!(idx.contains(7));
        assert!(!idx.contains(8));
    }

    #[test]
    fn test_graph_has_neighbours() {
        let mut idx = HnswIndex::new(4, 50);
        idx.insert(1, vec![0.0]);
        idx.insert(2, vec![1.0]);
        // Node 2 should have node 1 as a neighbour (and vice versa)
        assert!(!idx.neighbours(2).is_empty() || !idx.neighbours(1).is_empty());
    }

    #[test]
    fn test_m_cap() {
        let m = 2;
        let mut idx = HnswIndex::new(m, 50);
        for i in 0..10u64 {
            idx.insert(i, vec![i as f32]);
        }
        for i in 0..10u64 {
            assert!(idx.neighbours(i).len() <= m * 2); // forward + reverse edges
        }
    }

    #[test]
    fn test_search_empty_returns_empty() {
        let idx = HnswIndex::new(4, 50);
        let results = idx.search(&[1.0, 2.0], 5);
        assert!(results.is_empty());
    }

    #[test]
    fn test_ef_construction_accessor() {
        let idx = HnswIndex::new(16, 200);
        assert_eq!(idx.ef_construction(), 200);
        assert_eq!(idx.m(), 16);
    }
}
