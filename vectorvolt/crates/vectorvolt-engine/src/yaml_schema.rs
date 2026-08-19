//! YAML entity schema loader (VV-ENG-020).
//!
//! Loads a table schema from a YAML file with the following format:
//!
//! ```yaml
//! table: orders
//! columns:
//!   - name: id
//!     type: u64
//!   - name: amount
//!     type: f64
//!   - name: ts
//!     type: timestamp
//! ```
//!
//! The `type` field accepts the same tokens as `schema.txt` (`u64`, `i64`,
//! `f64`, `bool`, `timestamp`, `decimal`). Unknown tokens default to `u64`.

use std::path::Path;

use serde::Deserialize;

use crate::error::{EngineError, EngineResult};
use crate::schema::{ColumnDef, ColumnType, TableSchema};

// ── YAML data model ───────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct YamlTableDef {
    pub table: String,
    pub columns: Vec<YamlColumn>,
}

#[derive(Debug, Deserialize)]
pub struct YamlColumn {
    pub name: String,
    #[serde(rename = "type", default = "default_type")]
    pub col_type: String,
}

fn default_type() -> String {
    "u64".into()
}

// ── loader ────────────────────────────────────────────────────────────────────

/// Parse a [`YamlTableDef`] from a YAML string.
pub fn parse_yaml_schema(yaml: &str) -> EngineResult<YamlTableDef> {
    serde_yaml::from_str::<YamlTableDef>(yaml)
        .map_err(|e| EngineError::InvalidYamlSchema(e.to_string()))
}

/// Load a [`YamlTableDef`] from a file on disk.
pub fn load_yaml_schema(path: &Path) -> EngineResult<YamlTableDef> {
    let raw = std::fs::read_to_string(path)?;
    parse_yaml_schema(&raw)
}

/// Convert a [`YamlTableDef`] to a [`TableSchema`].
pub fn yaml_def_to_schema(def: &YamlTableDef) -> EngineResult<TableSchema> {
    let cols: Vec<ColumnDef> = def
        .columns
        .iter()
        .map(|c| ColumnDef::new(c.name.clone(), ColumnType::from_token(&c.col_type)))
        .collect();
    TableSchema::new(cols).map_err(|e| EngineError::InvalidYamlSchema(e.to_string()))
}
