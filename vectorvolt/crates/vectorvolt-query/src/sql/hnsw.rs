//! HNSW (Hierarchical Navigable Small World) approximate nearest-neighbour index (S29).
//!
//! Provides sub-linear ANN search via a multi-layer proximity graph.
//! The index is persisted as `<table_dir>/hnsw_<col>.json` next to the
//! `VectorStore` sidecar so the two stay in sync.
//!
//! ## Algorithm sketch
//!
//! * **Insert:** assign a random max-layer `ℓ` via geometric distribution with
//!   parameter 1/`M` (capped at `max_layers - 1`). Greedily descend from the
//!   entry point to `ℓ`, then for each layer from `ℓ` down to 0 run a beam
//!   search (width `ef_construction`) and connect the new node to the
//!   `M` nearest candidates.
//! * **Search:** descend from the top layer to layer 1 with greedy 1-NN,
//!   then do a full beam search (width `ef`) at layer 0 and return the
//!   top-`k` results.

use std::collections::{BinaryHeap, HashMap, HashSet};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::sql::vec_store::DistanceMetric;

/// Number of bi-directional connections per node per layer.
const M: usize = 16;
/// Beam width used when connecting new nodes during index build.
const EF_CONSTRUCTION: usize = 100;
/// Maximum number of layers.
const MAX_LAYERS: usize = 16;

// ── Serialisable graph ────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
struct NodeData {
    /// The stored vector.
    vec: Vec<f32>,
    /// Adjacency lists per layer (layer 0 = bottom, finest granularity).
    /// `layers[i]` is the list of neighbour pks at layer i.
    layers: Vec<Vec<u64>>,
}

#[derive(Debug, Serialize, Deserialize)]
struct GraphData {
    entry_pk: Option<u64>,
    entry_level: usize,
    nodes: HashMap<String, NodeData>, // pk as string → node
}

impl GraphData {
    fn empty() -> Self {
        Self {
            entry_pk: None,
            entry_level: 0,
            nodes: HashMap::new(),
        }
    }
}

// ── Public API ────────────────────────────────────────────────────────────────

/// An HNSW approximate nearest-neighbour index for one vector column.
pub struct HnswIndex {
    path: PathBuf,
    g: GraphData,
    metric: DistanceMetric,
}

impl HnswIndex {
    /// Load (or create) the HNSW index at `<base_dir>/<table>/hnsw_<col>.json`.
    pub fn load(
        base_dir: &Path,
        table: &str,
        col: &str,
        metric: DistanceMetric,
    ) -> std::io::Result<Self> {
        let path = base_dir.join(table).join(format!("hnsw_{col}.json"));
        let g = if path.exists() {
            let bytes = std::fs::read(&path)?;
            serde_json::from_slice(&bytes).unwrap_or_else(|_| GraphData::empty())
        } else {
            GraphData::empty()
        };
        Ok(Self { path, g, metric })
    }

    /// Insert a vector with the given primary key into the index.
    pub fn insert(&mut self, pk: u64, vec: Vec<f32>) {
        let level = random_level();
        let mut node = NodeData {
            vec,
            layers: vec![Vec::new(); level + 1],
        };

        if self.g.entry_pk.is_none() {
            self.g.nodes.insert(pk.to_string(), node);
            self.g.entry_pk = Some(pk);
            self.g.entry_level = level;
            return;
        }

        let entry_pk = self.g.entry_pk.unwrap();
        let entry_level = self.g.entry_level;
        let mut cur_pk = entry_pk;

        // Phase 1: greedily descend from top layer to level+1.
        let top = entry_level.max(level);
        for lc in ((level + 1)..=top).rev() {
            if lc > entry_level {
                continue;
            }
            cur_pk = self.greedy_search_layer(cur_pk, &node.vec, lc);
        }

        // Phase 2: beam search + connect at each layer from `level` down to 0.
        for lc in (0..=level.min(entry_level)).rev() {
            let candidates = self.beam_search_layer(cur_pk, &node.vec, lc, EF_CONSTRUCTION);
            let neighbours = self.select_neighbours(&candidates, M);
            // Connect new node → neighbours.
            node.layers[lc] = neighbours.iter().map(|&(npk, _)| npk).collect();
            // Connect neighbours → new node (bi-directional, pruned to M).
            for &(npk, _) in &neighbours {
                let nkey = npk.to_string();
                // Check if pruning is needed before taking a mutable borrow.
                let needs_prune = self
                    .g
                    .nodes
                    .get(&nkey)
                    .is_some_and(|nd| lc < nd.layers.len() && nd.layers[lc].len() >= M);
                // Pre-compute distances while we still have an immutable borrow.
                let dist_map: Option<(Vec<u64>, Vec<f32>)> = if needs_prune {
                    self.g.nodes.get(&nkey).map(|nd| {
                        let nvec = nd.vec.clone();
                        let metric = self.metric;
                        let nbrs = nd.layers[lc]
                            .iter()
                            .chain(std::iter::once(&pk))
                            .cloned()
                            .collect::<Vec<_>>();
                        let dists = nbrs
                            .iter()
                            .map(|&n| {
                                self.g
                                    .nodes
                                    .get(&n.to_string())
                                    .map(|x| metric.distance(&x.vec, &nvec))
                                    .unwrap_or(f32::MAX)
                            })
                            .collect::<Vec<_>>();
                        (nbrs, dists)
                    })
                } else {
                    None
                };
                // Now take the mutable borrow.
                if let Some(ndata) = self.g.nodes.get_mut(&nkey)
                    && lc < ndata.layers.len()
                {
                    ndata.layers[lc].push(pk);
                    if let Some((mut nbrs, dists)) = dist_map {
                        let mut pairs: Vec<(u64, f32)> = nbrs.drain(..).zip(dists).collect();
                        pairs.sort_by(|a, b| {
                            a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal)
                        });
                        ndata.layers[lc] = pairs.into_iter().take(M).map(|(n, _)| n).collect();
                    }
                }
            }
            // Update cur_pk to nearest found at this layer.
            if let Some(&(npk, _)) = candidates.first() {
                cur_pk = npk;
            }
        }

        self.g.nodes.insert(pk.to_string(), node);

        // Update global entry point if new node has a higher level.
        if level > entry_level {
            self.g.entry_pk = Some(pk);
            self.g.entry_level = level;
        }
    }

    /// Approximate top-`k` nearest neighbours to `query`.
    /// Returns `(pk, distance)` pairs sorted by distance ascending.
    pub fn search(&self, query: &[f32], k: usize) -> Vec<(u64, f32)> {
        if self.g.entry_pk.is_none() || self.g.nodes.is_empty() {
            return Vec::new();
        }
        let entry_pk = self.g.entry_pk.unwrap();
        let entry_level = self.g.entry_level;
        let mut cur_pk = entry_pk;

        // Greedy descent from top layer to layer 1.
        for lc in (1..=entry_level).rev() {
            cur_pk = self.greedy_search_layer(cur_pk, query, lc);
        }

        // Beam search at layer 0.
        let ef = k.max(EF_CONSTRUCTION);
        let mut candidates = self.beam_search_layer(cur_pk, query, 0, ef);
        candidates.truncate(k);
        candidates
    }

    /// Persist the index to disk.
    pub fn save(&self) -> std::io::Result<()> {
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let bytes = serde_json::to_vec(&self.g).map_err(std::io::Error::other)?;
        std::fs::write(&self.path, bytes)
    }

    // ── Internal helpers ──────────────────────────────────────────────────────

    fn dist(&self, pk_str: &str, query: &[f32]) -> f32 {
        self.dist_to(pk_str, query, self.metric)
    }

    fn dist_to(&self, pk_str: &str, query: &[f32], metric: DistanceMetric) -> f32 {
        self.g
            .nodes
            .get(pk_str)
            .map(|n| metric.distance(&n.vec, query))
            .unwrap_or(f32::MAX)
    }

    /// Single-NN greedy step: follow the neighbour at `layer` that minimises distance.
    fn greedy_search_layer(&self, entry: u64, query: &[f32], layer: usize) -> u64 {
        let mut best_pk = entry;
        let mut best_d = self.dist(&entry.to_string(), query);
        while let Some(node) = self.g.nodes.get(&best_pk.to_string()) {
            let nbrs: &[u64] = node.layers.get(layer).map(|v| v.as_slice()).unwrap_or(&[]);
            let mut improved = false;
            for &npk in nbrs {
                let d = self.dist(&npk.to_string(), query);
                if d < best_d {
                    best_d = d;
                    best_pk = npk;
                    improved = true;
                }
            }
            if !improved {
                break;
            }
        }
        best_pk
    }

    /// Beam search at a single layer; returns candidates sorted by distance asc.
    fn beam_search_layer(
        &self,
        entry: u64,
        query: &[f32],
        layer: usize,
        ef: usize,
    ) -> Vec<(u64, f32)> {
        // Min-heap of (distance, pk) for candidates to explore.
        // BinaryHeap is a max-heap so we negate distances.
        let mut visited: HashSet<u64> = HashSet::new();
        // candidates: min-heap (best first): store as (-d, pk)
        let mut candidates: BinaryHeap<(OrderedF32, u64)> = BinaryHeap::new();
        // result set: max-heap (worst first) capped at ef: store as (d, pk)
        let mut result: BinaryHeap<(OrderedF32, u64)> = BinaryHeap::new();

        let d0 = self.dist(&entry.to_string(), query);
        visited.insert(entry);
        candidates.push((OrderedF32(-d0), entry)); // neg so smallest d = top
        result.push((OrderedF32(d0), entry));

        while let Some((OrderedF32(neg_d), pk)) = candidates.pop() {
            let d = -neg_d;
            // If our worst result is better than this candidate, stop.
            if let Some(&(OrderedF32(worst), _)) = result.peek()
                && d > worst
                && result.len() >= ef
            {
                break;
            }
            let node = match self.g.nodes.get(&pk.to_string()) {
                Some(n) => n,
                None => continue,
            };
            let nbrs: &[u64] = node.layers.get(layer).map(|v| v.as_slice()).unwrap_or(&[]);
            for &npk in nbrs {
                if visited.contains(&npk) {
                    continue;
                }
                visited.insert(npk);
                let nd = self.dist(&npk.to_string(), query);
                let worst = result
                    .peek()
                    .map(|&(OrderedF32(w), _)| w)
                    .unwrap_or(f32::MAX);
                if nd < worst || result.len() < ef {
                    candidates.push((OrderedF32(-nd), npk));
                    result.push((OrderedF32(nd), npk));
                    if result.len() > ef {
                        result.pop(); // drop the worst
                    }
                }
            }
        }

        // Collect and sort asc by distance.
        let mut out: Vec<(u64, f32)> = result
            .into_iter()
            .map(|(OrderedF32(d), pk)| (pk, d))
            .collect();
        out.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
        out
    }

    fn select_neighbours(&self, candidates: &[(u64, f32)], m: usize) -> Vec<(u64, f32)> {
        // Simple heuristic: take the m closest.
        candidates.iter().take(m).copied().collect()
    }
}

// ── Newtype wrapper for f32 in BinaryHeap ─────────────────────────────────────

#[derive(Clone, Copy, PartialEq)]
struct OrderedF32(f32);

impl Eq for OrderedF32 {}

impl PartialOrd for OrderedF32 {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for OrderedF32 {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0
            .partial_cmp(&other.0)
            .unwrap_or(std::cmp::Ordering::Equal)
    }
}

// ── Level assignment ──────────────────────────────────────────────────────────

/// Assign a random layer level in `[0, MAX_LAYERS-1]` using a geometric
/// distribution with p = 1/M (same as the original HNSW paper).
fn random_level() -> usize {
    let ml = 1.0_f64 / (M as f64).ln();
    let rnd: f64 = random_f64();
    let level = (-rnd.ln() * ml).floor() as usize;
    level.min(MAX_LAYERS - 1)
}

/// Simple LCG pseudo-random float in `(0, 1)`. Uses thread-local state so
/// there's no dependency on `rand`.
fn random_f64() -> f64 {
    use std::cell::Cell;
    thread_local! {
        static STATE: Cell<u64> = const { Cell::new(0x4d595df4d0f33173) };
    }
    STATE.with(|s| {
        // Xorshift64
        let mut x = s.get();
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        s.set(x);
        // Map to (0,1).
        (x as f64) / (u64::MAX as f64)
    })
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_hnsw_insert_and_search() {
        let dir = TempDir::new().unwrap();
        let mut idx = HnswIndex::load(dir.path(), "t", "v", DistanceMetric::L2).unwrap();

        // Insert 20 vectors in a 2D grid.
        for i in 0u64..20 {
            idx.insert(i, vec![i as f32, i as f32]);
        }

        // Query nearest to pk=10 ([10.0, 10.0]).
        let query = vec![10.0, 10.0];
        let results = idx.search(&query, 3);
        assert!(!results.is_empty(), "search returned no results");
        // Results must be sorted ascending by distance.
        for w in results.windows(2) {
            assert!(w[0].1 <= w[1].1);
        }
        // The exact pk=10 should be in the top-3 results.
        assert!(results.iter().any(|&(pk, _)| pk == 10));
    }

    #[test]
    fn test_hnsw_save_reload() {
        let dir = TempDir::new().unwrap();
        {
            let mut idx = HnswIndex::load(dir.path(), "t", "v", DistanceMetric::L2).unwrap();
            idx.insert(1, vec![1.0, 0.0]);
            idx.insert(2, vec![0.0, 1.0]);
            idx.insert(3, vec![1.0, 1.0]);
            idx.save().unwrap();
        }
        let idx = HnswIndex::load(dir.path(), "t", "v", DistanceMetric::L2).unwrap();
        let results = idx.search(&[1.0, 0.0], 1);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].0, 1);
    }

    #[test]
    fn test_hnsw_empty_search() {
        let dir = TempDir::new().unwrap();
        let idx = HnswIndex::load(dir.path(), "t", "v", DistanceMetric::L2).unwrap();
        let results = idx.search(&[1.0, 2.0], 5);
        assert!(results.is_empty());
    }
}
