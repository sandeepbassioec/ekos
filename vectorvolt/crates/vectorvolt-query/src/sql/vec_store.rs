//! Vector sidecar store + brute-force similarity search (S26-A, VV-IDX-019..022).
//!
//! Vectors for `ColumnType::Vector` columns are persisted alongside the table in
//! `<table_dir>/vec_<col_name>.json`. The JSON is a map of `pk (as string) → Vec<f32>`.
//! Distance is computed on-the-fly; HNSW approximate index is S27.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Persistent vector store for one column of one table.
pub struct VectorStore {
    path: PathBuf,
    data: HashMap<u64, Vec<f32>>, // pk → vector
}

impl VectorStore {
    /// Load (or create) the store for `<base_dir>/<table>/vec_<col>.json`.
    pub fn load(base_dir: &Path, table: &str, col: &str) -> std::io::Result<Self> {
        let path = base_dir.join(table).join(format!("vec_{col}.json"));
        let data: HashMap<u64, Vec<f32>> = if path.exists() {
            let bytes = std::fs::read(&path)?;
            // JSON keys are strings; deserialize as HashMap<String, Vec<f32>> then convert.
            let raw: HashMap<String, Vec<f32>> = serde_json::from_slice(&bytes).unwrap_or_default();
            raw.into_iter()
                .filter_map(|(k, v)| k.parse::<u64>().ok().map(|pk| (pk, v)))
                .collect()
        } else {
            HashMap::new()
        };
        Ok(Self { path, data })
    }

    /// Insert or replace the vector for `pk`.
    pub fn insert(&mut self, pk: u64, vec: Vec<f32>) {
        self.data.insert(pk, vec);
    }

    /// Persist to disk (keys serialized as strings for JSON compatibility).
    pub fn save(&self) -> std::io::Result<()> {
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let raw: HashMap<String, &Vec<f32>> =
            self.data.iter().map(|(k, v)| (k.to_string(), v)).collect();
        let bytes = serde_json::to_vec(&raw).map_err(std::io::Error::other)?;
        std::fs::write(&self.path, bytes)
    }

    /// Return all (pk, distance) pairs sorted by `metric` ascending.
    pub fn ranked(&self, query: &[f32], metric: DistanceMetric) -> Vec<(u64, f32)> {
        let mut pairs: Vec<(u64, f32)> = self
            .data
            .iter()
            .map(|(&pk, vec)| (pk, metric.distance(vec, query)))
            .collect();
        pairs.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
        pairs
    }
}

/// Distance metric for vector similarity queries.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DistanceMetric {
    /// Euclidean (L2) distance — smaller = more similar.
    L2,
    /// Cosine distance = 1 − cosine_similarity — smaller = more similar.
    Cosine,
}

impl DistanceMetric {
    pub fn distance(self, a: &[f32], b: &[f32]) -> f32 {
        match self {
            Self::L2 => l2(a, b),
            Self::Cosine => cosine_dist(a, b),
        }
    }
}

fn l2(a: &[f32], b: &[f32]) -> f32 {
    a.iter()
        .zip(b.iter())
        .map(|(x, y)| (x - y).powi(2))
        .sum::<f32>()
        .sqrt()
}

fn cosine_dist(a: &[f32], b: &[f32]) -> f32 {
    let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let na: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let nb: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    if na == 0.0 || nb == 0.0 {
        1.0
    } else {
        1.0 - (dot / (na * nb))
    }
}

/// Parse a bracket-delimited float vector literal: `[1.0, 2.0, 3.0]`.
pub fn parse_vec_literal(s: &str) -> Option<Vec<f32>> {
    let s = s.trim();
    let inner = s.strip_prefix('[')?.strip_suffix(']')?;
    inner
        .split(',')
        .map(|t| t.trim().parse::<f32>().ok())
        .collect()
}
