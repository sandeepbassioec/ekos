//! Column type system for VectorVolt table schemas (VV-ENG-008).
//!
//! Every column has a declared [`ColumnType`]. Values are always stored as
//! `u64` on disk (the physical layer is u64-columnar); the type annotation
//! describes how those bits are interpreted and what validation the engine
//! enforces on writes.
//!
//! ## Persistence format
//!
//! `schema.txt` stores one column per line as `"name:type"`. Unknown type
//! tokens fall back to [`ColumnType::U64`] so old single-name-per-line files
//! remain readable without migration.

use crate::{EngineError, EngineResult};

// ── ColumnType ────────────────────────────────────────────────────────────────

/// Physical-storage type for a single column.
///
/// All variants store their value as a `u64` on disk; this enum records how
/// those bits should be interpreted by the query layer and what invariants the
/// engine enforces.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ColumnType {
    /// Raw unsigned 64-bit integer (default / legacy).
    U64,
    /// Signed 64-bit integer stored as two's-complement `u64`.
    I64,
    /// IEEE 754 double stored as `u64::from_bits(f64.to_bits())`.
    F64,
    /// Boolean: only `0` (false) and `1` (true) are valid.
    Bool,
    /// Unix timestamp in nanoseconds stored as `u64`.
    Timestamp,
    /// Decimal fixed-point: value × 10⁻⁶, stored as two's-complement `u64`.
    Decimal,
}

impl ColumnType {
    /// Parse the lowercase string token used in `schema.txt`.
    ///
    /// Unknown tokens map to [`ColumnType::U64`] for forward compatibility.
    pub fn from_token(s: &str) -> Self {
        match s.trim() {
            "i64" => Self::I64,
            "f64" => Self::F64,
            "bool" => Self::Bool,
            "timestamp" => Self::Timestamp,
            "decimal" => Self::Decimal,
            _ => Self::U64,
        }
    }

    /// The lowercase string token written to `schema.txt`.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::U64 => "u64",
            Self::I64 => "i64",
            Self::F64 => "f64",
            Self::Bool => "bool",
            Self::Timestamp => "timestamp",
            Self::Decimal => "decimal",
        }
    }

    /// Validate that `value` is a legal bit-pattern for this type.
    ///
    /// Most types accept any `u64` (the engine trusts the caller). Only
    /// [`ColumnType::Bool`] has a constraint: the value must be `0` or `1`.
    pub fn validate(self, col_idx: usize, value: u64) -> EngineResult<()> {
        if self == Self::Bool && value > 1 {
            return Err(EngineError::SchemaMismatch {
                col_idx,
                col_type: "bool",
                value,
            });
        }
        Ok(())
    }
}

// ── ColumnDef ─────────────────────────────────────────────────────────────────

/// Definition of a single column: name + type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ColumnDef {
    pub name: String,
    pub col_type: ColumnType,
}

impl ColumnDef {
    pub fn new(name: impl Into<String>, col_type: ColumnType) -> Self {
        Self {
            name: name.into(),
            col_type,
        }
    }

    /// Convenience: `ColumnDef` with type [`ColumnType::U64`].
    pub fn u64(name: impl Into<String>) -> Self {
        Self::new(name, ColumnType::U64)
    }

    /// Serialize to the `"name:type"` line format.
    pub fn to_line(&self) -> String {
        format!("{}:{}", self.name, self.col_type.as_str())
    }

    /// Parse from a `"name:type"` or legacy `"name"` line.
    pub fn from_line(line: &str) -> Self {
        let line = line.trim();
        match line.find(':') {
            Some(pos) => {
                let name = line[..pos].to_owned();
                let col_type = ColumnType::from_token(&line[pos + 1..]);
                Self { name, col_type }
            }
            None => Self::u64(line),
        }
    }
}

// ── TableSchema ───────────────────────────────────────────────────────────────

/// Ordered list of column definitions for a table.
///
/// Column 0 is always the primary-key column. The schema is immutable after
/// creation; DDL alterations produce a new `TableSchema`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TableSchema {
    columns: Vec<ColumnDef>,
}

impl TableSchema {
    /// Build a schema from an explicit list of [`ColumnDef`]s.
    ///
    /// Returns `Err(SchemaMismatch)` if any two columns share a name, since
    /// duplicate column names make queries ambiguous.
    pub fn new(columns: Vec<ColumnDef>) -> EngineResult<Self> {
        for i in 0..columns.len() {
            for j in (i + 1)..columns.len() {
                if columns[i].name == columns[j].name {
                    return Err(EngineError::SchemaMismatch {
                        col_idx: j,
                        col_type: "duplicate",
                        value: 0,
                    });
                }
            }
        }
        Ok(Self { columns })
    }

    /// Build a schema from column names alone; all columns are typed [`ColumnType::U64`].
    ///
    /// Used when opening tables written by older VectorVolt versions that did
    /// not persist type information.
    pub fn from_names(names: &[String]) -> Self {
        Self {
            columns: names.iter().map(ColumnDef::u64).collect(),
        }
    }

    /// Number of columns (including the PK column at index 0).
    pub fn col_count(&self) -> usize {
        self.columns.len()
    }

    /// Type of the primary-key column (always index 0).
    pub fn pk_type(&self) -> ColumnType {
        self.columns
            .first()
            .map(|c| c.col_type)
            .unwrap_or(ColumnType::U64)
    }

    /// Return the [`ColumnDef`] at `idx`, or `None` if out of range.
    pub fn get(&self, idx: usize) -> Option<&ColumnDef> {
        self.columns.get(idx)
    }

    /// Find a column by name, returning `(index, &ColumnDef)` or `None`.
    pub fn find(&self, name: &str) -> Option<(usize, &ColumnDef)> {
        self.columns
            .iter()
            .enumerate()
            .find(|(_, c)| c.name == name)
    }

    /// Validate that `values` is the right length and each value is legal for
    /// its column's type.
    pub fn validate_row(&self, values: &[u64]) -> EngineResult<()> {
        if values.len() != self.columns.len() {
            return Err(EngineError::ColOutOfRange {
                col_idx: values.len(),
                col_count: self.columns.len(),
            });
        }
        for (idx, (&v, col)) in values.iter().zip(&self.columns).enumerate() {
            col.col_type.validate(idx, v)?;
        }
        Ok(())
    }

    /// Column names in order (for APIs that still use `Vec<String>`).
    pub fn names(&self) -> Vec<String> {
        self.columns.iter().map(|c| c.name.clone()).collect()
    }

    /// Serialize to the multi-line text format stored in `schema.txt`.
    pub fn to_text(&self) -> String {
        self.columns
            .iter()
            .map(|c| c.to_line())
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Deserialize from the text stored in `schema.txt`.
    ///
    /// Handles both the new `"name:type"` format and the legacy `"name"` format.
    pub fn from_text(text: &str) -> EngineResult<Self> {
        let columns: Vec<ColumnDef> = text
            .lines()
            .map(str::trim)
            .filter(|l| !l.is_empty())
            .map(ColumnDef::from_line)
            .collect();
        Self::new(columns)
    }
}
