//! TOML configuration file support for VectorVolt engine (VV-ENG-017).
//!
//! An `engine.toml` file sets tunables that would otherwise require CLI flags or
//! hard-coded constants. When `vvctl serve --config <path>` is used, the file is
//! loaded, validated, and converted to [`EngineOptions`].
//!
//! ## Example `engine.toml`
//!
//! ```toml
//! flush_threshold = 524288     # rows before a WAL flush (default 1 048 576)
//! durability      = "fsync"    # "fsync" | "write_only" | "buffered"
//! max_partitions  = 32         # maximum number of storage partitions per table
//!
//! [[columns]]
//! name = "pk"
//! type = "u64"
//!
//! [[columns]]
//! name = "score"
//! type = "f64"
//!
//! [[columns]]
//! name = "active"
//! type = "bool"
//! ```

use std::path::Path;

use serde::Deserialize;

use crate::engine::{Durability, EngineOptions};
use crate::error::{EngineError, EngineResult};
use crate::schema::{ColumnDef, TableSchema};

// ── Raw TOML shape ────────────────────────────────────────────────────────────

/// Raw deserialized form of `engine.toml`.
#[derive(Debug, Deserialize, PartialEq)]
pub struct EngineConfig {
    /// Rows buffered before a WAL flush. Defaults to 1 048 576.
    pub flush_threshold: Option<usize>,

    /// Durability mode: `"fsync"` | `"write_only"` | `"buffered"`.
    /// Defaults to `"fsync"`.
    pub durability: Option<String>,

    /// Maximum number of storage partitions per table.
    /// Currently stored for informational use; passed through on open.
    pub max_partitions: Option<usize>,

    /// Column definitions. When present, used to build a [`TableSchema`].
    #[serde(default)]
    pub columns: Vec<ColumnConfig>,
}

/// A single column entry inside `[[columns]]`.
#[derive(Debug, Deserialize, PartialEq, Clone)]
pub struct ColumnConfig {
    pub name: String,
    #[serde(rename = "type")]
    pub col_type: String,
}

// ── Parsing ───────────────────────────────────────────────────────────────────

impl EngineConfig {
    /// Parse TOML text into an [`EngineConfig`].
    pub fn from_toml(text: &str) -> EngineResult<Self> {
        toml::from_str(text)
            .map_err(|e| EngineError::InvalidConfig(format!("TOML parse error: {e}")))
    }

    /// Load and parse a TOML config file from `path`.
    pub fn load(path: impl AsRef<Path>) -> EngineResult<Self> {
        let text = std::fs::read_to_string(path.as_ref()).map_err(|e| {
            EngineError::InvalidConfig(format!(
                "cannot read config {}: {e}",
                path.as_ref().display()
            ))
        })?;
        Self::from_toml(&text)
    }

    /// Convert to [`EngineOptions`].
    ///
    /// Unknown durability strings are an error rather than silently defaulting,
    /// so operators get immediate feedback if they typo a value.
    pub fn into_engine_options(self) -> EngineResult<EngineOptions> {
        let flush = self.flush_threshold.unwrap_or(1_048_576);
        if flush == 0 {
            return Err(EngineError::InvalidConfig(
                "flush_threshold must be > 0".into(),
            ));
        }

        let durability = match self.durability.as_deref().unwrap_or("fsync") {
            "fsync" => Durability::Fsync,
            "write_only" => Durability::WriteOnly,
            "buffered" => Durability::Buffered,
            other => {
                return Err(EngineError::InvalidConfig(format!(
                    "unknown durability mode {other:?}; expected \"fsync\", \"write_only\", or \"buffered\""
                )));
            }
        };

        Ok(EngineOptions::new(flush).with_durability(durability))
    }

    /// Build a [`TableSchema`] from the `[[columns]]` entries.
    ///
    /// Returns `None` when `columns` is empty (caller falls back to CLI flags).
    pub fn into_schema(self) -> EngineResult<Option<TableSchema>> {
        if self.columns.is_empty() {
            return Ok(None);
        }
        let defs: Vec<ColumnDef> = self
            .columns
            .into_iter()
            .map(|c| ColumnDef::from_line(&format!("{}:{}", c.name, c.col_type)))
            .collect();
        TableSchema::new(defs).map(Some)
    }
}

// ── Public helpers consumed by vvctl ─────────────────────────────────────────

/// Load `path`, extract `EngineOptions`, and optionally a `TableSchema`.
///
/// Returns `(EngineOptions, Option<TableSchema>)`.
pub fn load_config(path: impl AsRef<Path>) -> EngineResult<(EngineOptions, Option<TableSchema>)> {
    let cfg = EngineConfig::load(path)?;
    // Clone columns before consuming self for schema.
    let cols = cfg.columns.clone();
    let opts = EngineConfig {
        flush_threshold: cfg.flush_threshold,
        durability: cfg.durability,
        max_partitions: cfg.max_partitions,
        columns: vec![],
    }
    .into_engine_options()?;
    let schema = EngineConfig {
        flush_threshold: None,
        durability: None,
        max_partitions: None,
        columns: cols,
    }
    .into_schema()?;
    Ok((opts, schema))
}
