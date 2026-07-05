//! Compaction — merges multiple SSTables into one, discarding tombstones
//! where no older version could exist.
//!
//! Strategy (Phase 2 sketch — size-tiered):
//!   - Collect all entries from N input SSTables using a merge iterator
//!   - For duplicate keys: keep the entry from the newest SSTable (highest id)
//!   - Drop tombstones whose key doesn't appear in any older level
//!
//! The merge iterator uses a min-heap keyed on (key, table_index_desc) so
//! that for equal keys the newest table wins without extra de-dup logic.

use std::{
    cmp::Reverse,
    collections::BinaryHeap,
    path::Path,
};

use bytes::Bytes;

use crate::{
    memtable::MemEntry,
    sstable::{SsTable, SsTableWriter, sstable_path},
    StoreResult,
};

// --------------------------------------------------------------------------
// Merge iterator
// --------------------------------------------------------------------------

/// A cursor over one SSTable's sorted entries.
struct TableCursor {
    entries: Vec<MemEntry>,
    pos: usize,
    table_id: u64,
}

impl TableCursor {
    fn new(sst: &SsTable, table_id: u64) -> StoreResult<Self> {
        Ok(TableCursor { entries: sst.iter()?, pos: 0, table_id })
    }

    fn peek(&self) -> Option<&MemEntry> {
        self.entries.get(self.pos)
    }

    fn advance(&mut self) {
        self.pos += 1;
    }
}

/// Heap item — wraps the current key of a cursor for comparison.
/// We want: smallest key first; for equal keys, largest table_id first
/// (newest wins), so we negate table_id with Reverse.
#[derive(Eq, PartialEq)]
struct HeapItem {
    key: Bytes,
    /// Higher table_id → newer → should win → smaller heap rank via Reverse
    table_id: Reverse<u64>,
    cursor_idx: usize,
}

impl Ord for HeapItem {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.key
            .cmp(&other.key)
            .then_with(|| self.table_id.cmp(&other.table_id))
    }
}

impl PartialOrd for HeapItem {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

/// Multi-way merge iterator over SSTables in ascending key order.
/// For duplicate keys, the cursor with the highest table_id (newest) wins.
pub struct MergeIter {
    cursors: Vec<TableCursor>,
    heap: BinaryHeap<Reverse<HeapItem>>,
    last_key: Option<Bytes>,
}

impl MergeIter {
    pub fn new(tables: &[(&SsTable, u64)]) -> StoreResult<Self> {
        let mut cursors = Vec::with_capacity(tables.len());
        let mut heap = BinaryHeap::new();

        for (i, (sst, id)) in tables.iter().enumerate() {
            let cursor = TableCursor::new(sst, *id)?;
            if let Some(entry) = cursor.peek() {
                heap.push(Reverse(HeapItem {
                    key: entry.key.clone(),
                    table_id: Reverse(*id),
                    cursor_idx: i,
                }));
            }
            cursors.push(cursor);
        }

        Ok(MergeIter { cursors, heap, last_key: None })
    }
}

impl Iterator for MergeIter {
    type Item = MemEntry;

    fn next(&mut self) -> Option<MemEntry> {
        loop {
            let Reverse(item) = self.heap.pop()?;
            let cursor = &mut self.cursors[item.cursor_idx];
            let entry = cursor.peek().cloned()?;
            cursor.advance();

            // Re-push cursor if it has more entries
            if let Some(next) = cursor.peek() {
                self.heap.push(Reverse(HeapItem {
                    key: next.key.clone(),
                    table_id: Reverse(cursor.table_id),
                    cursor_idx: item.cursor_idx,
                }));
            }

            // Skip duplicate keys (older versions) — we already emitted the newest
            if self.last_key.as_ref() == Some(&entry.key) {
                continue;
            }
            self.last_key = Some(entry.key.clone());
            return Some(entry);
        }
    }
}

// --------------------------------------------------------------------------
// Compaction runner
// --------------------------------------------------------------------------

/// Result of a compaction run.
pub struct CompactionResult {
    /// Path of the newly written output SSTable.
    pub output_path: std::path::PathBuf,
    /// Number of entries written (after dedup and tombstone removal).
    pub entries_written: u64,
    /// Number of tombstones dropped (only when `drop_tombstones=true`).
    pub tombstones_dropped: u64,
}

/// Merge N SSTables (provided in ascending age order, newest last) into a
/// single output SSTable at `output_id`.
///
/// `drop_tombstones`: set `true` only for the bottom-most level where no
/// older data can exist. For intermediate compactions keep tombstones so
/// that deletes are propagated downward.
pub fn compact(
    tables: &[(SsTable, u64)],
    output_dir: &Path,
    output_id: u64,
    drop_tombstones: bool,
) -> StoreResult<CompactionResult> {
    let refs: Vec<(&SsTable, u64)> = tables.iter().map(|(t, id)| (t, *id)).collect();
    let merge = MergeIter::new(&refs)?;

    let out_path = sstable_path(output_dir, output_id);
    let mut writer = SsTableWriter::create(&out_path)?;

    let mut entries_written = 0u64;
    let mut tombstones_dropped = 0u64;

    for entry in merge {
        if drop_tombstones && entry.value.is_none() {
            tombstones_dropped += 1;
            continue;
        }
        writer.append(&entry)?;
        entries_written += 1;
    }

    writer.finish()?;

    Ok(CompactionResult { output_path: out_path, entries_written, tombstones_dropped })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{memtable::MemEntry, sstable::SsTableWriter};
    use tempfile::TempDir;

    fn make_sst(dir: &Path, id: u64, pairs: &[(&str, Option<&str>)]) -> SsTable {
        let path = sstable_path(dir, id);
        let mut w = SsTableWriter::create(&path).unwrap();
        for (k, v) in pairs {
            w.append(&MemEntry {
                key: Bytes::from(k.to_string()),
                value: v.map(|s| Bytes::from(s.to_string())),
            })
            .unwrap();
        }
        w.finish().unwrap();
        SsTable::open(&path).unwrap()
    }

    #[test]
    fn merge_dedup_newest_wins() {
        let dir = TempDir::new().unwrap();

        // table 1 (older, id=1): a=old, b=1
        let t1 = make_sst(dir.path(), 1, &[("a", Some("old")), ("b", Some("1"))]);
        // table 2 (newer, id=2): a=new, c=3
        let t2 = make_sst(dir.path(), 2, &[("a", Some("new")), ("c", Some("3"))]);

        let result = compact(&[(t1, 1), (t2, 2)], dir.path(), 99, false).unwrap();

        let merged = SsTable::open(&result.output_path).unwrap();
        let all = merged.iter().unwrap();
        // a should have value "new" (from newer table)
        assert_eq!(all[0].value, Some(Bytes::from("new")));
        // b still present
        assert_eq!(all[1].key, Bytes::from("b"));
        assert_eq!(result.entries_written, 3);
    }

    #[test]
    fn tombstone_drop_at_bottom_level() {
        let dir = TempDir::new().unwrap();

        let t1 = make_sst(dir.path(), 1, &[("a", Some("v")), ("b", None)]);

        let result = compact(&[(t1, 1)], dir.path(), 99, true).unwrap();
        assert_eq!(result.entries_written, 1); // "b" tombstone dropped
        assert_eq!(result.tombstones_dropped, 1);
    }
}
