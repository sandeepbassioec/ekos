//! Simplified HNSW (Hierarchical Navigable Small World) index.
//!
//! This is a single-layer implementation that maintains the correct public API
//! for VectorVolt's vector search. The search path uses brute-force comparison
//! for correctness; the graph structure is maintained for future multi-layer use.

use std::collections::HashMap;

use crate::distance;

/// Errors returned by the fallible [`HnswIndex`] entry points.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum VectorError {
    /// The vector contains a NaN or infinite component at `index`.
    ///
    /// Non-finite components make distances NaN, which has no total order
    /// against real distances, so such vectors are rejected outright rather
    /// than silently poisoning the neighbour graph or the result ordering.
    #[error("vector component {index} is not finite (NaN or infinity)")]
    NonFinite { index: usize },
}

/// Reject vectors containing NaN or infinite components.
fn check_finite(vector: &[f32]) -> Result<(), VectorError> {
    match vector.iter().position(|x| !x.is_finite()) {
        Some(index) => Err(VectorError::NonFinite { index }),
        None => Ok(()),
    }
}

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
    ///
    /// A vector with a NaN or infinite component is not inserted (it is
    /// ignored); use [`HnswIndex::try_insert`] to observe the rejection.
    pub fn insert(&mut self, id: u64, vector: Vec<f32>) {
        let _ = self.try_insert(id, vector);
    }

    /// Insert a vector with the given id, rejecting non-finite vectors.
    ///
    /// Returns [`VectorError::NonFinite`] (and leaves the index unchanged) if
    /// any component is NaN or infinite.
    pub fn try_insert(&mut self, id: u64, vector: Vec<f32>) -> Result<(), VectorError> {
        check_finite(&vector)?;
        self.vectors.insert(id, vector.clone());
        // Find m nearest neighbours
        let mut neighbours: Vec<(u64, f32)> = self
            .vectors
            .iter()
            .filter(|&(&nid, _)| nid != id)
            .map(|(&nid, v)| (nid, distance::euclidean_distance(&vector, v)))
            .collect();
        neighbours.sort_by(|a, b| a.1.total_cmp(&b.1));
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
        Ok(())
    }

    /// Search for the `k` nearest neighbours to `query`.
    ///
    /// Returns a list of `(id, distance)` pairs sorted by ascending distance.
    /// A query with a NaN or infinite component matches nothing (empty
    /// result); use [`HnswIndex::try_search`] to observe the rejection.
    pub fn search(&self, query: &[f32], k: usize) -> Vec<(u64, f32)> {
        self.try_search(query, k).unwrap_or_default()
    }

    /// Search for the `k` nearest neighbours to `query`, rejecting a
    /// non-finite query with [`VectorError::NonFinite`].
    pub fn try_search(&self, query: &[f32], k: usize) -> Result<Vec<(u64, f32)>, VectorError> {
        check_finite(query)?;
        let mut results: Vec<(u64, f32)> = self
            .vectors
            .iter()
            .map(|(&id, v)| (id, distance::euclidean_distance(query, v)))
            .collect();
        // `total_cmp` never panics; distances between finite vectors can still
        // overflow to +inf, which it orders after every finite distance.
        results.sort_by(|a, b| a.1.total_cmp(&b.1));
        results.truncate(k);
        Ok(results)
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

    #[test]
    fn test_try_insert_rejects_nan_and_inf() {
        let mut idx = HnswIndex::new(4, 50);
        idx.insert(1, vec![0.0, 0.0]);
        assert_eq!(
            idx.try_insert(2, vec![1.0, f32::NAN]),
            Err(VectorError::NonFinite { index: 1 })
        );
        assert_eq!(
            idx.try_insert(3, vec![f32::INFINITY, 0.0]),
            Err(VectorError::NonFinite { index: 0 })
        );
        assert_eq!(
            idx.try_insert(4, vec![0.0, f32::NEG_INFINITY]),
            Err(VectorError::NonFinite { index: 1 })
        );
        assert_eq!(idx.len(), 1);
        // The infallible wrapper must not panic and must not insert either.
        idx.insert(5, vec![f32::NAN, f32::NAN]);
        assert!(!idx.contains(5));
        // Search over the index still works (previously panicked on NaN).
        let r = idx.search(&[0.0, 0.0], 5);
        assert_eq!(r.len(), 1);
    }

    #[test]
    fn test_try_search_rejects_nan_query() {
        let mut idx = HnswIndex::new(4, 50);
        idx.insert(1, vec![0.0, 0.0]);
        idx.insert(2, vec![1.0, 1.0]);
        assert_eq!(
            idx.try_search(&[f32::NAN, 0.0], 2),
            Err(VectorError::NonFinite { index: 0 })
        );
        assert_eq!(
            idx.try_search(&[0.0, f32::INFINITY], 2),
            Err(VectorError::NonFinite { index: 1 })
        );
        // The infallible wrapper returns nothing instead of panicking.
        assert!(idx.search(&[f32::NAN, f32::NAN], 2).is_empty());
        assert_eq!(idx.try_search(&[0.0, 0.0], 2).unwrap()[0].0, 1);
    }

    #[test]
    fn test_search_orders_overflowing_distance_last() {
        let mut idx = HnswIndex::new(4, 50);
        idx.insert(1, vec![f32::MAX, f32::MAX]);
        idx.insert(2, vec![0.0, 0.0]);
        // Distance to point 1 overflows to +inf; to point 2 it is finite.
        let r = idx.try_search(&[1.0, 1.0], 2).unwrap();
        assert_eq!(r[0].0, 2);
        assert!(r[1].1.is_infinite());
    }
}
