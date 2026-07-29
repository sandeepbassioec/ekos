//! [`MicroPartition`] — an in-memory micro-partition: a set of columns with a
//! shared `(file_id, partition_id)` address.
//!
//! A micro-partition is the unit of I/O in VectorVolt. A single SSTable file
//! contains exactly one micro-partition.  Each column stores its data in
//! column-oriented (SoA) layout for cache-efficient scans.

use crate::{StorageError, StorageResult, column::ColumnValues};

/// An in-memory micro-partition ready for serialization.
pub struct MicroPartition {
    /// SSTable file number (matches the `file_id` in [`RowDescriptor`]).
    pub file_id: u16,
    /// Micro-partition index within the file.
    pub partition_id: u16,
    /// Column names, parallel to `columns`.
    pub schema: Vec<String>,
    /// Column data in SoA order.
    pub columns: Vec<ColumnValues>,
}

impl MicroPartition {
    /// Create an empty partition.
    pub fn new(file_id: u16, partition_id: u16) -> Self {
        MicroPartition {
            file_id,
            partition_id,
            schema: Vec::new(),
            columns: Vec::new(),
        }
    }

    /// Append a column.  All columns in a partition must have the same row count.
    pub fn push_column(
        &mut self,
        name: impl Into<String>,
        values: ColumnValues,
    ) -> StorageResult<()> {
        if let Some(first) = self.columns.first() {
            let expected = first.row_count();
            if values.row_count() != expected {
                return Err(StorageError::RowCountMismatch {
                    expected,
                    got: values.row_count(),
                });
            }
        }
        self.schema.push(name.into());
        self.columns.push(values);
        Ok(())
    }

    /// Number of rows across all columns.
    pub fn row_count(&self) -> usize {
        self.columns.first().map_or(0, |c| c.row_count())
    }

    /// Number of columns.
    pub fn col_count(&self) -> usize {
        self.columns.len()
    }
}

// ── tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::column::ColumnValues;

    #[test]
    fn basic_push() {
        let mut p = MicroPartition::new(1, 0);
        p.push_column("age", ColumnValues::Raw(vec![25, 30, 42]))
            .unwrap();
        assert_eq!(p.row_count(), 3);
        assert_eq!(p.col_count(), 1);
    }

    #[test]
    fn mismatched_row_count_returns_error() {
        let mut p = MicroPartition::new(0, 0);
        p.push_column("a", ColumnValues::Raw(vec![1, 2, 3]))
            .unwrap();
        let result = p.push_column("b", ColumnValues::Raw(vec![1, 2]));
        assert!(matches!(result, Err(StorageError::RowCountMismatch { .. })));
    }

    #[test]
    fn empty_partition() {
        let p = MicroPartition::new(0, 0);
        assert_eq!(p.row_count(), 0);
        assert_eq!(p.col_count(), 0);
    }
}
