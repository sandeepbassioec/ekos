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
    /// UTF-8 text. Stored as a u64 hash in the columnar engine; full strings live
    /// in a per-table sidecar managed by the SQL executor's FTS layer.
    Text,
    /// Floating-point vector of `usize` dimensions.
    /// Vectors are stored in a sidecar HNSW index, not in the columnar layout.
    Vector(usize),
}

impl ColumnType {
    /// Parse the lowercase string token used in `schema.txt`.
    ///
    /// Unknown tokens map to [`ColumnType::U64`] for forward compatibility.
    pub fn from_token(s: &str) -> Self {
        let s = s.trim();
        if let Some(rest) = s.strip_prefix("vector(").and_then(|r| r.strip_suffix(')'))
            && let Ok(n) = rest.parse::<usize>()
        {
            return Self::Vector(n);
        }
        if s == "vector" {
            return Self::Vector(0);
        }
        match s {
            "i64" => Self::I64,
            "f64" => Self::F64,
            "bool" => Self::Bool,
            "timestamp" => Self::Timestamp,
            "decimal" => Self::Decimal,
            "text" => Self::Text,
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
            Self::Text => "text",
            Self::Vector(_) => "vector",
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
        // Vector columns are stored as 0 in the columnar layout (data lives in
        // the HNSW sidecar). Any u64 value is technically valid here.
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

    /// Return all column definitions in declaration order.
    pub fn columns(&self) -> &[ColumnDef] {
        &self.columns
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

    /// Return a new schema with `new_cols` appended.
    ///
    /// Errors if any added column name duplicates an existing one.
    pub fn with_added(&self, new_cols: &[ColumnDef]) -> EngineResult<Self> {
        let mut columns = self.columns.clone();
        for col in new_cols {
            if columns.iter().any(|c| c.name == col.name) {
                return Err(crate::EngineError::InvalidTableName(format!(
                    "column '{}' already exists",
                    col.name
                )));
            }
            columns.push(col.clone());
        }
        Self::new(columns)
    }

    /// Return a new schema with the named columns removed.
    ///
    /// Silently ignores names that don't match any column (idempotent).
    /// Errors if the result would have zero columns.
    pub fn with_dropped(&self, names: &[&str]) -> EngineResult<Self> {
        let columns: Vec<ColumnDef> = self
            .columns
            .iter()
            .filter(|c| !names.contains(&c.name.as_str()))
            .cloned()
            .collect();
        if columns.is_empty() {
            return Err(crate::EngineError::InvalidTableName(
                "ALTER TABLE DROP would remove all columns".into(),
            ));
        }
        Self::new(columns)
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

// ── TypedValue ────────────────────────────────────────────────────────────────

/// A runtime value produced by interpreting a raw `u64` bit-pattern through a
/// [`ColumnType`] lens.
///
/// VectorVolt stores every column as `u64` on disk.  `TypedValue` is the
/// presentation layer: it carries the decoded value so callers can display or
/// compare it without knowing the storage representation.
#[derive(Debug, Clone, PartialEq)]
pub enum TypedValue {
    /// Raw unsigned 64-bit integer.
    U64(u64),
    /// Signed 64-bit integer (two's-complement reinterpret).
    I64(i64),
    /// IEEE 754 double (bit-cast from `u64`).
    F64(f64),
    /// Boolean (`false` = 0, `true` = 1).
    Bool(bool),
    /// Unix timestamp in nanoseconds (displayed as raw `u64`).
    Timestamp(u64),
    /// Fixed-point decimal: `raw_bits as i64` / 1_000_000 (6 decimal places).
    Decimal(i64),
    /// Floating-point vector. Not stored in the columnar u64 layout; the u64
    /// representation is always 0. Data lives in the HNSW sidecar index.
    Vector(Vec<f32>),
}

impl TypedValue {
    /// Decode a raw `u64` storage value according to the given [`ColumnType`].
    pub fn from_bits(col_type: ColumnType, bits: u64) -> Self {
        match col_type {
            ColumnType::U64 => Self::U64(bits),
            ColumnType::I64 => Self::I64(bits as i64),
            ColumnType::F64 => Self::F64(f64::from_bits(bits)),
            ColumnType::Bool => Self::Bool(bits != 0),
            ColumnType::Timestamp => Self::Timestamp(bits),
            ColumnType::Decimal => Self::Decimal(bits as i64),
            // Text and Vector live in sidecars; the columnar slot stores a hash/0.
            ColumnType::Text => Self::U64(bits),
            ColumnType::Vector(_) => Self::U64(bits),
        }
    }

    /// Re-encode this value back to its raw `u64` storage representation.
    pub fn to_bits(&self) -> u64 {
        match self {
            Self::U64(v) => *v,
            Self::I64(v) => *v as u64,
            Self::F64(v) => v.to_bits(),
            Self::Bool(v) => *v as u64,
            Self::Timestamp(v) => *v,
            Self::Decimal(v) => *v as u64,
            Self::Vector(_) => 0,
        }
    }

    /// Human-readable string for CLI display.
    ///
    /// - `U64` / `Timestamp` → decimal integer
    /// - `I64` → signed decimal (negative values show a minus sign)
    /// - `F64` → up to 6 significant decimal digits
    /// - `Bool` → `"true"` / `"false"`
    /// - `Decimal` → `"N.NNNNNN"` (6 decimal places, signed)
    pub fn display_str(&self) -> String {
        match self {
            Self::U64(v) => v.to_string(),
            Self::I64(v) => v.to_string(),
            Self::F64(v) => format!("{v:.6}"),
            Self::Bool(v) => v.to_string(),
            Self::Timestamp(v) => v.to_string(),
            Self::Decimal(v) => {
                let whole = v / 1_000_000;
                let frac = (v % 1_000_000).abs();
                format!("{whole}.{frac:06}")
            }
            Self::Vector(v) => {
                let inner: Vec<String> = v.iter().map(|f| format!("{f}")).collect();
                format!("[{}]", inner.join(", "))
            }
        }
    }

    /// Parse a human-readable string into a `TypedValue` of the given type.
    ///
    /// Accepts the same formats produced by [`TypedValue::display_str`] plus
    /// common shorthands (`"1"` / `"0"` for booleans, plain integers for
    /// decimals).
    pub fn parse(col_type: ColumnType, s: &str) -> EngineResult<Self> {
        let s = s.trim();
        match col_type {
            ColumnType::U64 => s
                .parse::<u64>()
                .map(Self::U64)
                .map_err(|_| EngineError::InvalidConfig(format!("expected u64, got '{s}'"))),
            ColumnType::I64 => s
                .parse::<i64>()
                .map(Self::I64)
                .map_err(|_| EngineError::InvalidConfig(format!("expected i64, got '{s}'"))),
            ColumnType::F64 => s
                .parse::<f64>()
                .map(Self::F64)
                .map_err(|_| EngineError::InvalidConfig(format!("expected f64, got '{s}'"))),
            ColumnType::Bool => match s {
                "true" | "1" => Ok(Self::Bool(true)),
                "false" | "0" => Ok(Self::Bool(false)),
                _ => Err(EngineError::InvalidConfig(format!(
                    "expected bool (true/false/1/0), got '{s}'"
                ))),
            },
            ColumnType::Timestamp => s.parse::<u64>().map(Self::Timestamp).map_err(|_| {
                EngineError::InvalidConfig(format!("expected timestamp (u64 ns), got '{s}'"))
            }),
            ColumnType::Decimal => {
                if let Some(dot) = s.find('.') {
                    let whole: i64 = s[..dot].parse().map_err(|_| {
                        EngineError::InvalidConfig(format!("expected decimal, got '{s}'"))
                    })?;
                    let frac_str = &s[dot + 1..];
                    let frac_str_padded = format!("{frac_str:0<6}");
                    let frac: i64 = frac_str_padded[..6].parse().map_err(|_| {
                        EngineError::InvalidConfig(format!("expected decimal, got '{s}'"))
                    })?;
                    let sign: i64 = if whole < 0 { -1 } else { 1 };
                    Ok(Self::Decimal(whole * 1_000_000 + sign * frac))
                } else {
                    s.parse::<i64>()
                        .map(|v| Self::Decimal(v * 1_000_000))
                        .map_err(|_| {
                            EngineError::InvalidConfig(format!("expected decimal, got '{s}'"))
                        })
                }
            }
            // Text and Vector live in sidecars; parse stores a hash/0 placeholder.
            ColumnType::Text => {
                let hash = s.bytes().fold(0u64, |h, b| h.wrapping_mul(31).wrapping_add(b as u64));
                Ok(Self::U64(hash))
            }
            ColumnType::Vector(_) => Ok(Self::U64(0)),
        }
    }
}
