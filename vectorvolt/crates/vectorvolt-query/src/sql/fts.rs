//! Full-text search: TextStore + InvertedIndex (S25-C, VV-IDX-015..018).
//!
//! Text values for TEXT columns are persisted alongside the table in
//! `<table_dir>/fts_<col_name>.json`.  The JSON is a map of `pk → text`.
//! An in-memory inverted index is built on demand by tokenising the stored texts.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Persistent text store for one column of one table.
pub struct TextStore {
    path: PathBuf,
    data: HashMap<u64, String>, // pk → text
}

impl TextStore {
    /// Load (or create) the store for `<base_dir>/<table>/fts_<col>.json`.
    pub fn load(base_dir: &Path, table: &str, col: &str) -> std::io::Result<Self> {
        let path = base_dir.join(table).join(format!("fts_{col}.json"));
        let data = if path.exists() {
            let bytes = std::fs::read(&path)?;
            serde_json::from_slice(&bytes).unwrap_or_default()
        } else {
            HashMap::new()
        };
        Ok(Self { path, data })
    }

    /// Insert or replace the text for `pk`.
    pub fn insert(&mut self, pk: u64, text: String) {
        self.data.insert(pk, text);
    }

    /// Persist to disk.
    pub fn save(&self) -> std::io::Result<()> {
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let bytes = serde_json::to_vec(&self.data).map_err(std::io::Error::other)?;
        std::fs::write(&self.path, bytes)
    }

    /// Get text for a pk.
    pub fn get(&self, pk: u64) -> Option<&str> {
        self.data.get(&pk).map(|s| s.as_str())
    }

    /// Build an in-memory inverted index: term → sorted Vec<pk>.
    pub fn build_index(&self) -> InvertedIndex {
        let mut map: HashMap<String, Vec<u64>> = HashMap::new();
        for (&pk, text) in &self.data {
            for term in tokenize(text) {
                map.entry(term).or_default().push(pk);
            }
        }
        for pks in map.values_mut() {
            pks.sort_unstable();
            pks.dedup();
        }
        InvertedIndex { map }
    }
}

/// In-memory inverted index built from a [`TextStore`].
pub struct InvertedIndex {
    map: HashMap<String, Vec<u64>>,
}

impl InvertedIndex {
    /// Return the PKs that contain `term` (case-insensitive, stemmed by lowercasing).
    pub fn search(&self, term: &str) -> &[u64] {
        let t = term.trim().to_lowercase();
        self.map.get(&t).map(|v| v.as_slice()).unwrap_or(&[])
    }
}

/// Tokenise text into lowercase words (non-alphanumeric → separator).
pub fn tokenize(text: &str) -> Vec<String> {
    text.split(|c: char| !c.is_alphanumeric())
        .filter(|t| !t.is_empty())
        .map(|t| t.to_lowercase())
        .collect()
}
