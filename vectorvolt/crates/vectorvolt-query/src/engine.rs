use vectorvolt_index::RowDescriptor;

use crate::{QueryResult, catalog::TableCatalog, error::QueryError, row::QueryRow};

/// Query engine that executes PK lookups, range scans, secondary-index scans,
/// and FK existence checks against a [`TableCatalog`].
pub struct QueryEngine {
    catalog: TableCatalog,
}

impl QueryEngine {
    pub fn new(catalog: TableCatalog) -> Self {
        QueryEngine { catalog }
    }

    // ── Point lookup ──────────────────────────────────────────────────────────

    /// Return the row with the given primary key, or `None` if not found.
    ///
    /// O(1) worst-case: one CuckooMap probe → one mmap column read per column.
    pub fn pk_lookup(&self, pk: u64) -> QueryResult<Option<QueryRow>> {
        let Some(desc) = self.catalog.pk_index.get(pk) else {
            return Ok(None);
        };
        let row = self.fetch_row(pk, desc)?;
        Ok(Some(row))
    }

    // ── Range scan ────────────────────────────────────────────────────────────

    /// Return all rows where column `col_idx` falls in `[lo, hi]`.
    ///
    /// The ZoneMap prunes partitions whose min/max don't overlap the range.
    /// Only surviving partitions are scanned, making this ~10 ns per pruned partition.
    pub fn range_scan(&self, col_idx: usize, lo: u64, hi: u64) -> QueryResult<Vec<QueryRow>> {
        let mut results: Vec<QueryRow> = Vec::new();

        for (&(file_id, partition_id), reader) in &self.catalog.partitions {
            // ZoneMap prune: skip partitions whose zone doesn't overlap [lo, hi].
            if let Ok((zone_min, zone_max)) = reader.zone_min_max(col_idx)
                && (zone_min > hi || zone_max < lo)
            {
                continue;
            }

            let col_count = reader.col_count() as usize;
            if col_idx >= col_count {
                return Err(QueryError::ColIdxOutOfRange { col_idx, col_count });
            }

            // Read the scan column to find matching row offsets.
            let scan_col = reader.read_column(col_idx)?;
            let scan_vals = scan_col.to_raw_vec();

            // Read ALL columns once per partition so we can reconstruct full rows.
            let mut all_cols: Vec<Vec<u64>> = Vec::with_capacity(col_count);
            for c in 0..col_count {
                all_cols.push(reader.read_column(c)?.to_raw_vec());
            }

            for (row_offset, &val) in scan_vals.iter().enumerate() {
                if val >= lo && val <= hi {
                    // Recover the PK by looking at column 0 (convention: col 0 is the PK).
                    let pk = all_cols
                        .first()
                        .and_then(|c| c.get(row_offset))
                        .copied()
                        .unwrap_or(u64::MAX);

                    let values: Vec<u64> = all_cols.iter().map(|c| c[row_offset]).collect();

                    results.push(QueryRow { pk, values });
                }
            }

            let _ = (file_id, partition_id); // suppress unused variable lint
        }

        Ok(results)
    }

    // ── Secondary index scan ──────────────────────────────────────────────────

    /// Return all rows whose indexed column hashes to `value_hash`.
    ///
    /// The SecondaryIndex stores `(value_hash → RoaringBitmap of u32 PKs)`.
    /// Each PK in the bitmap is resolved to a full row via the CuckooMap.
    ///
    /// Hash collisions may produce false positives — callers should re-check
    /// the actual column value if exact match is required.
    pub fn secondary_scan(&self, value_hash: u32) -> QueryResult<Vec<QueryRow>> {
        let Some(iter) = self.catalog.secondary_index.get(value_hash) else {
            return Ok(Vec::new());
        };

        let mut results: Vec<QueryRow> = Vec::new();
        for pk32 in iter {
            let pk = pk32 as u64;
            if let Some(row) = self.pk_lookup(pk)? {
                results.push(row);
            }
        }
        Ok(results)
    }

    // ── FK existence check ────────────────────────────────────────────────────

    /// Return `true` if `fk_value` is probably present (CuckooFilter, ~0.78% FP rate).
    ///
    /// A `false` return is a definite miss. A `true` return may be a false positive.
    #[inline]
    pub fn fk_exists(&self, fk_value: u64) -> bool {
        self.catalog.fk_filter.contains(fk_value)
    }

    // ── Internal helpers ──────────────────────────────────────────────────────

    fn fetch_row(&self, pk: u64, desc: RowDescriptor) -> QueryResult<QueryRow> {
        let file_id = desc.file_id();
        let partition_id = desc.partition_id();
        let row_offset = desc.row_offset() as usize;

        let reader = self
            .catalog
            .partitions
            .get(&(file_id, partition_id))
            .ok_or(QueryError::PartitionNotFound {
                file_id,
                partition_id,
            })?;

        let row_count = reader.row_count() as usize;
        if row_offset >= row_count {
            return Err(QueryError::RowOffsetOutOfBounds {
                row_offset: row_offset as u32,
                row_count: row_count as u32,
            });
        }

        let col_count = reader.col_count() as usize;
        let mut values: Vec<u64> = Vec::with_capacity(col_count);
        for c in 0..col_count {
            let col = reader.read_column(c)?;
            values.push(col.get(row_offset));
        }

        Ok(QueryRow { pk, values })
    }
}

// ── unit tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use vectorvolt_index::{CuckooFilter, CuckooMap, RowDescriptor, SecondaryIndex, ZoneMap};
    use vectorvolt_storage::{ColumnValues, MicroPartition, PartitionReader, write_partition};

    use super::*;
    use crate::catalog::TableCatalog;

    fn build_partition(
        file_id: u16,
        partition_id: u16,
        pks: &[u64],
        ages: &[u64],
    ) -> PartitionReader {
        let mut p = MicroPartition::new(file_id, partition_id);
        p.push_column("pk", ColumnValues::Raw(pks.to_vec()))
            .unwrap();
        p.push_column("age", ColumnValues::auto_encode(ages.to_vec()))
            .unwrap();

        let mut buf: Vec<u8> = Vec::new();
        write_partition(&mut buf, &p).unwrap();

        let tmp = tempfile::NamedTempFile::new().unwrap();
        std::io::Write::write_all(&mut tmp.as_file(), &buf).unwrap();
        PartitionReader::open(tmp.path()).unwrap()
    }

    fn simple_engine() -> QueryEngine {
        // Single partition: pk=[1,2,3], age=[25,30,42]
        let pks = [1u64, 2, 3];
        let ages = [25u64, 30, 42];
        let reader = build_partition(0, 0, &pks, &ages);

        let mut pk_index = CuckooMap::with_capacity(8);
        for (offset, &pk) in pks.iter().enumerate() {
            pk_index
                .insert(pk, RowDescriptor::new(0, 0, offset as u32))
                .unwrap();
        }

        let mut secondary_index = SecondaryIndex::new();
        for (offset, &age) in ages.iter().enumerate() {
            let hash = age as u32;
            secondary_index.insert(hash, offset as u32 + 1); // row_id = pk (1-indexed here)
        }

        let mut zone_map = ZoneMap::new();
        zone_map.set(0, 25, 42);

        let mut fk_filter = CuckooFilter::with_capacity(16);
        for &pk in &pks {
            fk_filter.insert(pk).unwrap();
        }

        let mut partitions = HashMap::new();
        partitions.insert((0u16, 0u16), reader);

        let catalog = TableCatalog::new(pk_index, secondary_index, zone_map, fk_filter, partitions);
        QueryEngine::new(catalog)
    }

    #[test]
    fn pk_lookup_found() {
        let engine = simple_engine();
        let row = engine.pk_lookup(2).unwrap().expect("pk=2 must exist");
        assert_eq!(row.pk, 2);
        assert_eq!(row.values[1], 30); // age
    }

    #[test]
    fn pk_lookup_not_found() {
        let engine = simple_engine();
        assert!(engine.pk_lookup(99).unwrap().is_none());
    }

    #[test]
    fn fk_exists_hit_and_miss() {
        let engine = simple_engine();
        assert!(engine.fk_exists(1));
        assert!(engine.fk_exists(2));
        // 99 was never inserted — very unlikely to be a false positive at this size
        assert!(!engine.fk_exists(99));
    }

    #[test]
    fn range_scan_returns_correct_rows() {
        let engine = simple_engine();
        // age BETWEEN 25 AND 30 → rows with pk=1 (age=25) and pk=2 (age=30)
        let mut rows = engine.range_scan(1, 25, 30).unwrap();
        rows.sort_by_key(|r| r.pk);
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].pk, 1);
        assert_eq!(rows[0].values[1], 25);
        assert_eq!(rows[1].pk, 2);
        assert_eq!(rows[1].values[1], 30);
    }

    #[test]
    fn range_scan_zone_prune_skips_partition() {
        // Build an engine with two partitions. Query range falls only in partition 1.
        let pks0 = [10u64, 11, 12];
        let ages0 = [20u64, 21, 22]; // zone [20,22]
        let pks1 = [20u64, 21, 22];
        let ages1 = [50u64, 51, 52]; // zone [50,52]

        let r0 = build_partition(0, 0, &pks0, &ages0);
        let r1 = build_partition(0, 1, &pks1, &ages1);

        let mut pk_index = CuckooMap::with_capacity(16);
        for (offset, &pk) in pks0.iter().enumerate() {
            pk_index
                .insert(pk, RowDescriptor::new(0, 0, offset as u32))
                .unwrap();
        }
        for (offset, &pk) in pks1.iter().enumerate() {
            pk_index
                .insert(pk, RowDescriptor::new(0, 1, offset as u32))
                .unwrap();
        }

        let mut partitions = HashMap::new();
        partitions.insert((0u16, 0u16), r0);
        partitions.insert((0u16, 1u16), r1);

        let catalog = TableCatalog::new(
            pk_index,
            SecondaryIndex::new(),
            ZoneMap::new(),
            CuckooFilter::with_capacity(16),
            partitions,
        );
        let engine = QueryEngine::new(catalog);

        // age BETWEEN 50 AND 52 — only partition 1 survives zone pruning
        let rows = engine.range_scan(1, 50, 52).unwrap();
        assert_eq!(rows.len(), 3);
        for r in &rows {
            assert!(r.values[1] >= 50 && r.values[1] <= 52);
        }
    }
}
