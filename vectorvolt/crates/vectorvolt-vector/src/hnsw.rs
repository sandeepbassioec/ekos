//! HNSW (Hierarchical Navigable Small World) approximate nearest-neighbour index.
//!
//! A real multi-layer proximity graph (Malkov & Yashunin):
//!
//! * **Insert:** draw a random top layer `l` (geometric, `1/ln(M)`), greedily
//!   descend from the entry point to layer `l+1`, then at each layer from
//!   `min(l, top)` down to 0 run a beam search of width `ef_construction`,
//!   pick up to `M` neighbours with the diversity heuristic and link both ways
//!   (re-pruning neighbours that overflow `M`, or `2M` on layer 0).
//! * **Search:** greedy 1-NN descent through the upper layers, then a beam
//!   search of width `ef_search` (at least `k`) on layer 0.
//!
//! For sets no larger than the search beam the graph walk would touch every
//! node anyway, so those are answered by an exact scan.

use std::cmp::{Ordering, Reverse};
use std::collections::{BinaryHeap, HashMap, HashSet};

use crate::distance;

/// Default beam width for [`HnswIndex::search`].
pub const DEFAULT_EF_SEARCH: usize = 64;
/// Hard cap on the number of layers.
const MAX_LEVEL: usize = 16;

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

/// `f32` distance with a total order, for use in binary heaps.
#[derive(Clone, Copy, PartialEq)]
struct Dist(f32);
impl Eq for Dist {}
impl PartialOrd for Dist {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for Dist {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.total_cmp(&other.0)
    }
}

struct Node {
    vector: Vec<f32>,
    /// `links[l]` = neighbour ids on layer `l`; `links.len() == level + 1`.
    links: Vec<Vec<u64>>,
}

/// HNSW approximate nearest-neighbour index.
pub struct HnswIndex {
    m: usize,
    ef_construction: usize,
    ef_search: usize,
    nodes: HashMap<u64, Node>,
    entry_point: Option<u64>,
    max_level: usize,
    rng: u64,
}

impl HnswIndex {
    /// Create a new HNSW index.
    ///
    /// * `m` — target number of neighbours per node (layer 0 allows `2*m`)
    /// * `ef_construction` — size of the dynamic candidate list during insert
    pub fn new(m: usize, ef_construction: usize) -> Self {
        Self {
            m,
            ef_construction,
            ef_search: DEFAULT_EF_SEARCH,
            nodes: HashMap::new(),
            entry_point: None,
            max_level: 0,
            rng: 0x9E37_79B9_7F4A_7C15,
        }
    }

    /// Set the default beam width used by [`HnswIndex::search`].
    pub fn set_ef_search(&mut self, ef: usize) {
        self.ef_search = ef.max(1);
    }

    /// Default beam width used by [`HnswIndex::search`].
    pub fn ef_search(&self) -> usize {
        self.ef_search
    }

    fn m_eff(&self) -> usize {
        self.m.max(1)
    }

    fn max_links(&self, layer: usize) -> usize {
        if layer == 0 {
            self.m_eff() * 2
        } else {
            self.m_eff()
        }
    }

    fn random_level(&mut self) -> usize {
        // xorshift64
        let mut x = self.rng;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.rng = x;
        let u = ((x >> 11) as f64 + 1.0) / ((1u64 << 53) as f64 + 1.0); // (0,1)
        let ml = 1.0 / (self.m.max(2) as f64).ln();
        ((-u.ln() * ml).floor() as usize).min(MAX_LEVEL)
    }

    fn dist_to(&self, id: u64, q: &[f32]) -> f32 {
        match self.nodes.get(&id) {
            Some(n) => distance::euclidean_distance(q, &n.vector),
            None => f32::INFINITY,
        }
    }

    /// Insert a vector with the given id.
    ///
    /// A vector with a NaN or infinite component is not inserted (it is
    /// ignored); use [`HnswIndex::try_insert`] to observe the rejection.
    pub fn insert(&mut self, id: u64, vector: Vec<f32>) {
        let _ = self.try_insert(id, vector);
    }

    /// Insert a vector with the given id, rejecting non-finite vectors.
    ///
    /// Returns [`VectorError::NonFinite`] (and leaves the index unchanged) if
    /// any component is NaN or infinite. Re-inserting an existing id replaces
    /// its vector.
    pub fn try_insert(&mut self, id: u64, vector: Vec<f32>) -> Result<(), VectorError> {
        check_finite(&vector)?;
        if self.nodes.contains_key(&id) {
            self.remove_node(id);
        }
        let level = self.random_level();
        let Some(mut ep) = self.entry_point else {
            self.nodes.insert(
                id,
                Node {
                    vector,
                    links: vec![Vec::new(); level + 1],
                },
            );
            self.entry_point = Some(id);
            self.max_level = level;
            return Ok(());
        };

        // Phase 1: greedy descent through layers above the new node's level.
        for lc in ((level + 1)..=self.max_level).rev() {
            ep = self.greedy_layer(ep, &vector, lc);
        }

        // Phase 2: beam search + neighbour selection on each layer.
        let ef = self.ef_construction.max(self.m_eff());
        let mut new_links: Vec<Vec<u64>> = vec![Vec::new(); level + 1];
        let mut entries = vec![ep];
        let mut updates: Vec<(usize, Vec<(u64, f32)>)> = Vec::new();
        for lc in (0..=level.min(self.max_level)).rev() {
            let cands = self.search_layer(&vector, &entries, ef, lc, &mut 0);
            let chosen = self.select_neighbours(&cands, self.m_eff());
            new_links[lc] = chosen.iter().map(|&(n, _)| n).collect();
            entries = cands.iter().map(|&(n, _)| n).collect();
            updates.push((lc, chosen));
        }

        self.nodes.insert(
            id,
            Node {
                vector,
                links: new_links,
            },
        );

        // Back-links, pruning overflowing neighbours.
        for (lc, chosen) in updates {
            let cap = self.max_links(lc);
            for (nid, _) in chosen {
                let overflow = match self.nodes.get_mut(&nid) {
                    Some(n) if lc < n.links.len() => {
                        if !n.links[lc].contains(&id) {
                            n.links[lc].push(id);
                        }
                        n.links[lc].len() > cap
                    }
                    _ => false,
                };
                if overflow {
                    self.prune(nid, lc, cap);
                }
            }
        }

        if level > self.max_level {
            self.max_level = level;
            self.entry_point = Some(id);
        }
        Ok(())
    }

    /// Re-select the best `cap` links of `nid` on `layer` using the heuristic.
    fn prune(&mut self, nid: u64, layer: usize, cap: usize) {
        let Some(base) = self.nodes.get(&nid).map(|n| n.vector.clone()) else {
            return;
        };
        let mut cands: Vec<(u64, f32)> = self.nodes[&nid].links[layer]
            .iter()
            .map(|&o| (o, self.dist_to(o, &base)))
            .collect();
        cands.sort_by(|a, b| a.1.total_cmp(&b.1));
        let keep: Vec<u64> = self
            .select_neighbours(&cands, cap)
            .into_iter()
            .map(|(o, _)| o)
            .collect();
        if let Some(n) = self.nodes.get_mut(&nid) {
            n.links[layer] = keep;
        }
    }

    /// Diversity heuristic: keep a candidate only if it is closer to the base
    /// than to every already-kept neighbour; top up with discarded ones.
    /// `cands` must be sorted by ascending distance to the base point.
    fn select_neighbours(&self, cands: &[(u64, f32)], m: usize) -> Vec<(u64, f32)> {
        let mut kept: Vec<(u64, f32)> = Vec::with_capacity(m);
        let mut skipped: Vec<(u64, f32)> = Vec::new();
        for &(cid, cd) in cands {
            if kept.len() >= m {
                break;
            }
            let Some(cn) = self.nodes.get(&cid) else {
                continue;
            };
            let diverse = kept.iter().all(|&(kid, _)| {
                self.nodes
                    .get(&kid)
                    .is_none_or(|kn| cd < distance::euclidean_distance(&cn.vector, &kn.vector))
            });
            if diverse {
                kept.push((cid, cd));
            } else {
                skipped.push((cid, cd));
            }
        }
        for s in skipped {
            if kept.len() >= m {
                break;
            }
            kept.push(s);
        }
        kept
    }

    /// Remove a node and every link pointing at it (used for re-insertion).
    fn remove_node(&mut self, id: u64) {
        self.nodes.remove(&id);
        for n in self.nodes.values_mut() {
            for l in n.links.iter_mut() {
                l.retain(|&o| o != id);
            }
        }
        if self.entry_point == Some(id) {
            let best = self
                .nodes
                .iter()
                .max_by_key(|(nid, n)| (n.links.len(), Reverse(**nid)))
                .map(|(&nid, n)| (nid, n.links.len() - 1));
            match best {
                Some((nid, lvl)) => {
                    self.entry_point = Some(nid);
                    self.max_level = lvl;
                }
                None => {
                    self.entry_point = None;
                    self.max_level = 0;
                }
            }
        }
    }

    /// Greedy 1-NN walk on `layer`.
    fn greedy_layer(&self, entry: u64, q: &[f32], layer: usize) -> u64 {
        let mut best = entry;
        let mut best_d = self.dist_to(entry, q);
        loop {
            let mut improved = false;
            if let Some(n) = self.nodes.get(&best)
                && let Some(links) = n.links.get(layer)
            {
                for &o in links {
                    let d = self.dist_to(o, q);
                    if d < best_d {
                        best_d = d;
                        best = o;
                        improved = true;
                    }
                }
            }
            if !improved {
                return best;
            }
        }
    }

    /// Beam search on one layer; returns up to `ef` `(id, dist)` ascending.
    /// `visited_out` is incremented by the number of distance evaluations.
    fn search_layer(
        &self,
        q: &[f32],
        entries: &[u64],
        ef: usize,
        layer: usize,
        visited_out: &mut usize,
    ) -> Vec<(u64, f32)> {
        let mut visited: HashSet<u64> = HashSet::new();
        // min-heap on distance of nodes still to expand
        let mut frontier: BinaryHeap<Reverse<(Dist, u64)>> = BinaryHeap::new();
        // max-heap of the current best `ef`
        let mut best: BinaryHeap<(Dist, u64)> = BinaryHeap::new();
        for &e in entries {
            if self.nodes.contains_key(&e) && visited.insert(e) {
                let d = Dist(self.dist_to(e, q));
                frontier.push(Reverse((d, e)));
                best.push((d, e));
                if best.len() > ef {
                    best.pop();
                }
            }
        }
        while let Some(Reverse((d, c))) = frontier.pop() {
            if best.len() >= ef && best.peek().is_some_and(|&(w, _)| d > w) {
                break;
            }
            let Some(links) = self.nodes.get(&c).and_then(|n| n.links.get(layer)) else {
                continue;
            };
            for &o in links {
                if !visited.insert(o) {
                    continue;
                }
                let od = Dist(self.dist_to(o, q));
                if best.len() < ef || best.peek().is_some_and(|&(w, _)| od < w) {
                    frontier.push(Reverse((od, o)));
                    best.push((od, o));
                    if best.len() > ef {
                        best.pop();
                    }
                }
            }
        }
        *visited_out += visited.len();
        let mut out: Vec<(u64, f32)> = best.into_iter().map(|(d, id)| (id, d.0)).collect();
        out.sort_by(|a, b| a.1.total_cmp(&b.1).then(a.0.cmp(&b.0)));
        out
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
        Ok(self.try_search_ef(query, k, self.ef_search)?.0)
    }

    /// Like [`HnswIndex::try_search`] with an explicit beam width `ef`, also
    /// returning the number of nodes whose distance was evaluated.
    pub fn try_search_ef(
        &self,
        query: &[f32],
        k: usize,
        ef: usize,
    ) -> Result<(Vec<(u64, f32)>, usize), VectorError> {
        check_finite(query)?;
        let Some(mut ep) = self.entry_point else {
            return Ok((Vec::new(), 0));
        };
        let ef = ef.max(k).max(1);
        if self.nodes.len() <= ef {
            // Exact scan: the beam would visit everything anyway.
            let mut all: Vec<(u64, f32)> = self
                .nodes
                .iter()
                .map(|(&id, n)| (id, distance::euclidean_distance(query, &n.vector)))
                .collect();
            all.sort_by(|a, b| a.1.total_cmp(&b.1).then(a.0.cmp(&b.0)));
            all.truncate(k);
            return Ok((all, self.nodes.len()));
        }
        let mut visited = 0usize;
        for lc in (1..=self.max_level).rev() {
            ep = self.greedy_layer(ep, query, lc);
            visited += 1;
        }
        let mut res = self.search_layer(query, &[ep], ef, 0, &mut visited);
        res.truncate(k);
        Ok((res, visited))
    }

    /// Number of vectors in the index.
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    /// True if the index contains no vectors.
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    /// True if the index contains a vector with the given id.
    pub fn contains(&self, id: u64) -> bool {
        self.nodes.contains_key(&id)
    }

    /// Return the stored vector for the given id, or `None`.
    pub fn get_vector(&self, id: u64) -> Option<&Vec<f32>> {
        self.nodes.get(&id).map(|n| &n.vector)
    }

    /// Return the current entry point id, or `None` if the index is empty.
    pub fn entry_point(&self) -> Option<u64> {
        self.entry_point
    }

    /// Number of layers currently in the graph.
    pub fn num_layers(&self) -> usize {
        if self.entry_point.is_some() {
            self.max_level + 1
        } else {
            0
        }
    }

    /// Return the neighbours of a node at layer 0.
    pub fn neighbours(&self, id: u64) -> &[u64] {
        self.nodes
            .get(&id)
            .and_then(|n| n.links.first())
            .map(|v| v.as_slice())
            .unwrap_or(&[])
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

    fn rand_vecs(n: usize, dim: usize, seed: u64) -> Vec<Vec<f32>> {
        let mut x = seed | 1;
        (0..n)
            .map(|_| {
                (0..dim)
                    .map(|_| {
                        x ^= x << 13;
                        x ^= x >> 7;
                        x ^= x << 17;
                        (x >> 40) as f32 / (1u64 << 24) as f32
                    })
                    .collect()
            })
            .collect()
    }

    #[test]
    fn test_recall_at_10_vs_brute_force_and_sublinear_visits() {
        use std::collections::HashSet;
        let n = 3000;
        let data = rand_vecs(n, 16, 42);
        let mut idx = HnswIndex::new(16, 200);
        for (i, v) in data.iter().enumerate() {
            idx.insert(i as u64, v.clone());
        }
        assert!(idx.num_layers() > 1, "expected a multi-layer graph");
        let queries = rand_vecs(100, 16, 7);
        let (mut hits, mut visited_total) = (0usize, 0usize);
        for q in &queries {
            let mut exact: Vec<(u64, f32)> = data
                .iter()
                .enumerate()
                .map(|(i, v)| (i as u64, distance::euclidean_distance(q, v)))
                .collect();
            exact.sort_by(|a, b| a.1.total_cmp(&b.1));
            let truth: HashSet<u64> = exact.iter().take(10).map(|e| e.0).collect();
            let (got, visited) = idx.try_search_ef(q, 10, 64).unwrap();
            visited_total += visited;
            hits += got.iter().filter(|g| truth.contains(&g.0)).count();
        }
        let recall = hits as f64 / (queries.len() * 10) as f64;
        let avg_visited = visited_total / queries.len();
        assert!(recall >= 0.9, "recall@10 = {recall}");
        assert!(avg_visited < n / 3, "avg visited {avg_visited} of {n}");
    }

    #[test]
    fn test_reinsert_replaces_vector() {
        let mut idx = HnswIndex::new(4, 50);
        for i in 0..200u64 {
            idx.insert(i, vec![i as f32, 0.0]);
        }
        idx.insert(5, vec![1000.0, 0.0]);
        assert_eq!(idx.len(), 200);
        assert_eq!(idx.search(&[1000.0, 0.0], 1)[0].0, 5);
    }
}
