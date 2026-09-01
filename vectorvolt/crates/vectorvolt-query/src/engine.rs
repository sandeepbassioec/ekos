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

            // SIMD-filter the scan column, then materialize only matched rows.
            //
            // The previous version decoded the scan column TWICE — once as
            // `scan_col` and again inside `all_cols` — then decoded every other
            // column in full regardless of how few rows matched.
            let offsets = reader.filter_offsets(col_idx, lo, hi)?;
            for values in reader.materialize_rows(&offsets)? {
                // Column 0 is the PK by convention.
                let pk = values.first().copied().unwrap_or(u64::MAX);
                results.push(QueryRow { pk, values });
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

        // Collect all matching PKs first, then issue a single batched lookup
        // (VV-QRY-003.5). This replaces N individual CuckooMap probes with one
        // vectorized `get_batch` call, cutting the secondary-scan probe cost
        // from O(N × CuckooMap-probe) to O(1 batch + N × read_value).
        let pks: Vec<u64> = iter.map(|pk32| pk32 as u64).collect();
        let rows = self.pk_lookup_batch(&pks)?;
        Ok(rows.into_iter().flatten().collect())
    }

    // ── FK existence check ────────────────────────────────────────────────────

    /// Return `true` if `fk_value` is definitely present in the target table.
    ///
    /// Two-phase check: the CuckooFilter is consulted first (a definite miss
    /// returns `false` without touching storage). When the filter returns `true`
    /// (which may be a false positive at ~0.78% rate) a real `pk_lookup` is
    /// performed against the partition map. Only rows confirmed present in
    /// storage return `true` (VV-QRY-004.3).
    pub fn fk_exists(&self, fk_value: u64) -> QueryResult<bool> {
        if !self.catalog.fk_filter.contains(fk_value) {
            return Ok(false);
        }
        // Filter said "maybe present" — re-check against real storage to
        // eliminate false positives.
        let confirmed = self.pk_lookup(fk_value)?.is_some();
        Ok(confirmed)
    }

    // ── SIMD analytics ───────────────────────────────────────────────────────

    /// Count rows where column `col_idx` falls in `[lo, hi]`.
    ///
    /// Uses ZoneMap pruning + `count_matching` (SIMD count_range under the hood).
    /// No offset Vec is allocated, saving a heap round-trip vs `range_scan`.
    pub fn count_scan(&self, col_idx: usize, lo: u64, hi: u64) -> QueryResult<u64> {
        if lo > hi {
            return Ok(0);
        }
        let mut total: u64 = 0;
        for (&(file_id, partition_id), reader) in &self.catalog.partitions {
            if let Ok((zone_min, zone_max)) = reader.zone_min_max(col_idx)
                && (zone_min > hi || zone_max < lo)
            {
                continue;
            }
            total += reader.count_matching(col_idx, lo, hi)? as u64;
            let _ = (file_id, partition_id);
        }
        Ok(total)
    }

    /// Sum of column `sum_col` for all rows where column `filter_col` is in `[lo, hi]`.
    ///
    /// Filters partitions with ZoneMap, then gathers `sum_col` values at matched
    /// offsets and applies `simd::sum`.
    pub fn sum_scan(
        &self,
        filter_col: usize,
        lo: u64,
        hi: u64,
        sum_col: usize,
    ) -> QueryResult<u64> {
        let mut total = 0u64;
        for (&(file_id, partition_id), reader) in &self.catalog.partitions {
            if let Ok((zone_min, zone_max)) = reader.zone_min_max(filter_col)
                && (zone_min > hi || zone_max < lo)
            {
                continue;
            }
            let offsets = reader.filter_offsets(filter_col, lo, hi)?;
            if offsets.is_empty() {
                let _ = (file_id, partition_id);
                continue;
            }
            let gathered: Vec<u64> = offsets
                .iter()
                .map(|&o| reader.read_value(sum_col, o as usize))
                .collect::<Result<_, _>>()?;
            total = total.wrapping_add(vectorvolt_hal::simd::sum(&gathered));
            let _ = (file_id, partition_id);
        }
        Ok(total)
    }

    /// Minimum value of column `col_idx` among rows where `col_idx` is in `[lo, hi]`.
    ///
    /// Returns `None` if no rows match.
    pub fn min_scan(&self, col_idx: usize, lo: u64, hi: u64) -> QueryResult<Option<u64>> {
        let mut global_min: Option<u64> = None;
        for (&(file_id, partition_id), reader) in &self.catalog.partitions {
            if let Ok((zone_min, zone_max)) = reader.zone_min_max(col_idx)
                && (zone_min > hi || zone_max < lo)
            {
                continue;
            }
            let offsets = reader.filter_offsets(col_idx, lo, hi)?;
            if offsets.is_empty() {
                let _ = (file_id, partition_id);
                continue;
            }
            let gathered: Vec<u64> = offsets
                .iter()
                .map(|&o| reader.read_value(col_idx, o as usize))
                .collect::<Result<_, _>>()?;
            if let Some((part_min, _)) = vectorvolt_hal::simd::min_max(&gathered) {
                global_min = Some(match global_min {
                    None => part_min,
                    Some(cur) => cur.min(part_min),
                });
            }
            let _ = (file_id, partition_id);
        }
        Ok(global_min)
    }

    /// Maximum value of column `col_idx` among rows where `col_idx` is in `[lo, hi]`.
    ///
    /// Returns `None` if no rows match.
    pub fn max_scan(&self, col_idx: usize, lo: u64, hi: u64) -> QueryResult<Option<u64>> {
        let mut global_max: Option<u64> = None;
        for (&(file_id, partition_id), reader) in &self.catalog.partitions {
            if let Ok((zone_min, zone_max)) = reader.zone_min_max(col_idx)
                && (zone_min > hi || zone_max < lo)
            {
                continue;
            }
            let offsets = reader.filter_offsets(col_idx, lo, hi)?;
            if offsets.is_empty() {
                let _ = (file_id, partition_id);
                continue;
            }
            let gathered: Vec<u64> = offsets
                .iter()
                .map(|&o| reader.read_value(col_idx, o as usize))
                .collect::<Result<_, _>>()?;
            if let Some((_, part_max)) = vectorvolt_hal::simd::min_max(&gathered) {
                global_max = Some(match global_max {
                    None => part_max,
                    Some(cur) => cur.max(part_max),
                });
            }
            let _ = (file_id, partition_id);
        }
        Ok(global_max)
    }

    /// Batch PK lookup using `CuckooMap::get_batch`.
    ///
    /// Returns one `Option<QueryRow>` per input PK in the same order.
    pub fn pk_lookup_batch(&self, pks: &[u64]) -> QueryResult<Vec<Option<QueryRow>>> {
        let descs = self.catalog.pk_index.get_batch(pks);
        descs
            .into_iter()
            .zip(pks.iter())
            .map(|(desc_opt, &pk)| match desc_opt {
                None => Ok(None),
                Some(desc) => self.fetch_row(pk, desc).map(Some),
            })
            .collect()
    }

    /// Batch FK existence check with false-positive recheck (VV-QRY-004.3).
    ///
    /// Returns one `bool` per input value in the same order. Values that pass
    /// the CuckooFilter are re-verified against storage; only confirmed rows
    /// return `true`.
    pub fn fk_exists_batch(&self, fk_values: &[u64]) -> QueryResult<Vec<bool>> {
        let filter_hits = self.catalog.fk_filter.contains_batch(fk_values);
        // Collect only the values the filter said "maybe present".
        let candidates: Vec<u64> = fk_values
            .iter()
            .zip(filter_hits.iter())
            .filter_map(|(&v, &hit)| if hit { Some(v) } else { None })
            .collect();
        // One batched storage lookup for all candidates.
        let confirmed_rows = self.pk_lookup_batch(&candidates)?;
        let mut confirmed_set = std::collections::HashSet::new();
        for (v, row_opt) in candidates.iter().zip(confirmed_rows.iter()) {
            if row_opt.is_some() {
                confirmed_set.insert(*v);
            }
        }
        Ok(fk_values
            .iter()
            .map(|v| confirmed_set.contains(v))
            .collect())
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

        // O(1) per column via `read_value`, which reads 8–12 bytes straight out of
        // the mmap.
        //
        // This previously called `read_column(c)` and then `.get(row_offset)` —
        // decoding and allocating *every* value in the column to return one cell,
        // once per column. That made a point lookup O(row_count × col_count)
        // instead of O(col_count), and it is the same cost profile removed from
        // `vectorvolt-engine::get()` earlier. Having both paths in the tree meant
        // whichever layer a caller happened to reach through silently decided
        // whether they got ~200 ns or ~80 µs. There is now one path.
        let col_count = reader.col_count() as usize;
        let mut values: Vec<u64> = Vec::with_capacity(col_count);
        for c in 0..col_count {
            values.push(reader.read_value(c, row_offset)?);
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
        assert!(engine.fk_exists(1).unwrap());
        assert!(engine.fk_exists(2).unwrap());
        // 99 was never inserted — very unlikely to be a false positive at this size
        assert!(!engine.fk_exists(99).unwrap());
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
