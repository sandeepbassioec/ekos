//! Configuration file support for VectorVolt engine (VV-ENG-017, VV-CFG-037).
//!
//! Supports both TOML (`engine.toml`) and YAML (`engine.yaml` / `engine.yml`).
//! File format is detected by extension; YAML is the preferred format.
//! When no config file is present, `load_config_with_profile()` fills in
//! [`SystemProfile`]-derived recommendations as defaults.
//!
//! ## Precedence chain (lowest → highest)
//!
//! ```text
//! SystemProfile defaults < config file < VECTORVOLT_* env vars < CLI overrides
//! ```
//!
//! ## Environment variables
//!
//! | Variable | Config field |
//! |---|---|
//! | `VECTORVOLT_FLUSH_THRESHOLD` | `flush_threshold` (integer) |
//! | `VECTORVOLT_DURABILITY` | `durability` (`fsync` / `write_only` / `buffered`) |
//! | `VECTORVOLT_MAX_PARTITIONS` | `max_partitions` (integer) |
//!
//! ## Example `engine.yaml` (preferred)
//!
//! ```yaml
//! flush_threshold: 524288
//! durability: fsync
//! max_partitions: 32
//! columns:
//!   - name: pk
//!     type: u64
//!   - name: score
//!     type: f64
//! ```
//!
//! ## Example `engine.toml` (legacy)
//!
//! ```toml
//! flush_threshold = 524288
//! durability      = "fsync"
//! max_partitions  = 32
//!
//! [[columns]]
//! name = "pk"
//! type = "u64"
//! ```

use std::path::Path;

use serde::Deserialize;

use crate::system_profile::SystemProfile;

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

    /// Parse YAML text into an [`EngineConfig`].
    pub fn from_yaml(text: &str) -> EngineResult<Self> {
        serde_yaml::from_str(text)
            .map_err(|e| EngineError::InvalidConfig(format!("YAML parse error: {e}")))
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

    /// Load a config file, auto-detecting format by extension.
    ///
    /// `.yaml` / `.yml` → YAML; anything else → TOML.
    pub fn load_auto(path: impl AsRef<Path>) -> EngineResult<Self> {
        let path = path.as_ref();
        let text = std::fs::read_to_string(path).map_err(|e| {
            EngineError::InvalidConfig(format!("cannot read config {}: {e}", path.display()))
        })?;
        let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");
        if ext == "yaml" || ext == "yml" {
            Self::from_yaml(&text)
        } else {
            Self::from_toml(&text)
        }
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

/// Load a TOML config at `path` and return `(EngineOptions, Option<TableSchema>)`.
pub fn load_config(path: impl AsRef<Path>) -> EngineResult<(EngineOptions, Option<TableSchema>)> {
    split_config(EngineConfig::load(path)?)
}

/// Load any config file (YAML or TOML) at `path`, auto-detected by extension.
///
/// Returns `(EngineOptions, Option<TableSchema>)`.
pub fn load_config_auto(
    path: impl AsRef<Path>,
) -> EngineResult<(EngineOptions, Option<TableSchema>)> {
    split_config(EngineConfig::load_auto(path)?)
}

/// Load config with [`SystemProfile`] recommendations as fallback defaults.
///
/// If `path` is `None` or the file does not exist, the profile's
/// `recommended_flush_threshold` and `recommended_partition_count` fill in
/// for any unset fields before conversion.
pub fn load_config_with_profile(
    path: Option<&Path>,
    profile: &SystemProfile,
) -> EngineResult<(EngineOptions, Option<TableSchema>)> {
    let mut cfg = match path {
        Some(p) if p.exists() => EngineConfig::load_auto(p)?,
        _ => EngineConfig {
            flush_threshold: None,
            durability: None,
            max_partitions: None,
            columns: vec![],
        },
    };
    // Fill in profile recommendations where the config leaves fields unset.
    if cfg.flush_threshold.is_none() {
        cfg.flush_threshold = Some(profile.recommended_flush_threshold);
    }
    if cfg.max_partitions.is_none() {
        cfg.max_partitions = Some(profile.recommended_partition_count);
    }
    split_config(cfg)
}

/// Optional CLI-level overrides applied after env vars.
///
/// `None` for any field means "not overridden at this layer".
#[derive(Debug, Default, Clone)]
pub struct CliOverrides {
    pub flush_threshold: Option<usize>,
    pub durability: Option<String>,
    pub max_partitions: Option<usize>,
}

impl CliOverrides {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_flush_threshold(mut self, v: usize) -> Self {
        self.flush_threshold = Some(v);
        self
    }

    pub fn with_durability(mut self, v: impl Into<String>) -> Self {
        self.durability = Some(v.into());
        self
    }

    pub fn with_max_partitions(mut self, v: usize) -> Self {
        self.max_partitions = Some(v);
        self
    }
}

/// Apply `VECTORVOLT_*` environment variables over a mutable [`EngineConfig`].
///
/// Each env var only overrides the corresponding field when the variable is set
/// and non-empty. Parse errors return [`EngineError::InvalidConfig`].
pub fn apply_env_overrides(cfg: &mut EngineConfig) -> EngineResult<()> {
    apply_env_overrides_with(cfg, |key| std::env::var(key).ok())
}

/// Like [`apply_env_overrides`] but uses a custom lookup instead of the real env.
///
/// Useful for deterministic testing without process-level env var mutation.
pub fn apply_env_overrides_with(
    cfg: &mut EngineConfig,
    lookup: impl Fn(&str) -> Option<String>,
) -> EngineResult<()> {
    if let Some(val) = lookup("VECTORVOLT_FLUSH_THRESHOLD") {
        let val = val.trim().to_owned();
        if !val.is_empty() {
            cfg.flush_threshold = Some(val.parse::<usize>().map_err(|_| {
                EngineError::InvalidConfig(format!(
                    "VECTORVOLT_FLUSH_THRESHOLD={val:?} is not a valid integer"
                ))
            })?);
        }
    }
    if let Some(val) = lookup("VECTORVOLT_DURABILITY") {
        let val = val.trim().to_owned();
        if !val.is_empty() {
            cfg.durability = Some(val);
        }
    }
    if let Some(val) = lookup("VECTORVOLT_MAX_PARTITIONS") {
        let val = val.trim().to_owned();
        if !val.is_empty() {
            cfg.max_partitions = Some(val.parse::<usize>().map_err(|_| {
                EngineError::InvalidConfig(format!(
                    "VECTORVOLT_MAX_PARTITIONS={val:?} is not a valid integer"
                ))
            })?);
        }
    }
    Ok(())
}

/// Apply [`CliOverrides`] over a mutable [`EngineConfig`].
///
/// CLI values win over everything because they are the highest-priority layer.
pub fn apply_cli_overrides(cfg: &mut EngineConfig, cli: &CliOverrides) {
    if let Some(v) = cli.flush_threshold {
        cfg.flush_threshold = Some(v);
    }
    if let Some(ref v) = cli.durability {
        cfg.durability = Some(v.clone());
    }
    if let Some(v) = cli.max_partitions {
        cfg.max_partitions = Some(v);
    }
}

/// Full precedence chain: profile < file < env < CLI.
///
/// - `path`: optional config file (YAML or TOML, auto-detected).
/// - `profile`: [`SystemProfile`] used as baseline when file is absent / fields unset.
/// - `cli`: explicit CLI overrides applied last.
pub fn load_config_full(
    path: Option<&Path>,
    profile: &SystemProfile,
    cli: &CliOverrides,
) -> EngineResult<(EngineOptions, Option<TableSchema>)> {
    // Layer 1: file or empty config
    let mut cfg = match path {
        Some(p) if p.exists() => EngineConfig::load_auto(p)?,
        _ => EngineConfig {
            flush_threshold: None,
            durability: None,
            max_partitions: None,
            columns: vec![],
        },
    };
    // Layer 2: profile recommendations for unset fields
    if cfg.flush_threshold.is_none() {
        cfg.flush_threshold = Some(profile.recommended_flush_threshold);
    }
    if cfg.max_partitions.is_none() {
        cfg.max_partitions = Some(profile.recommended_partition_count);
    }
    // Layer 3: env vars
    apply_env_overrides(&mut cfg)?;
    // Layer 4: CLI flags
    apply_cli_overrides(&mut cfg, cli);
    split_config(cfg)
}

// ── Internal ──────────────────────────────────────────────────────────────────

fn split_config(cfg: EngineConfig) -> EngineResult<(EngineOptions, Option<TableSchema>)> {
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
