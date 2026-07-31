//! [`execute_job`] — turn a [`CompactionJob`] into a merged partition
//! (VV-STO-026).
//!
//! The other half of VV-STO-015. `CompactionStrategy::plan` decides *what* to
//! merge; this does it. Kept separate so the policy stays a pure function that can
//! be tested exhaustively without touching a byte of data.
//!
//! ## Crash safety is an ordering argument, not a lock
//!
//! Compaction replaces N partitions with one. There is no way to make a file
//! write, a manifest update, and N deletions simultaneous, so instead the order is
//! chosen such that **every interruption point leaves a recoverable state**:
//!
//! | Crash after | State on disk | Consequence |
//! |---|---|---|
//! | writing the output | output exists, manifest unchanged | orphan file, leaks disk |
//! | the manifest `Replace` | both old and new exist, manifest names only the new | orphan files, leak disk |
//! | deleting the inputs | clean | none |
//!
//! Every failure mode is a **disk leak**, never data loss and never duplication.
//! That asymmetry is deliberate: an orphan file is found by a directory scan and
//! deleted, whereas a lost partition is unrecoverable and a duplicated one
//! silently doubles rows in every scan.
//!
//! The manifest swap is one [`ManifestEdit::Replace`] record precisely so it
//! cannot be interrupted halfway — a partial record fails its CRC and is
//! discarded, leaving the pre-compaction state intact.
//!
//! ## Liveness is injected, not inferred
//!
//! `.vvsst` files have no per-row delete marker: a delete removes the row from the
//! PK index and leaves the bytes in place. So a merge cannot tell a live row from
//! a deleted one by looking at the partition — it has to ask the index. Rather
//! than make this crate depend on `vectorvolt-index`, the caller supplies a
//! [`Liveness`] predicate. That also makes the merge testable with a hand-written
//! predicate instead of a whole index.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use crate::lifecycle::{CompactionJob, PartitionMeta};
use crate::{
    ColumnValues, MicroPartition, PartitionReader, StorageError, StorageResult, WriteOptions,
    write_partition_with,
};

/// Where a row physically lives, for a liveness decision.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RowLocation {
    pub file_id: u16,
    pub partition_id: u16,
    pub row_offset: u32,
}

/// Decides whether a row found during a merge should survive it.
///
/// Implemented by the caller against its PK index. A row is dead when the index
/// no longer points at that exact location — either because it was deleted, or
/// because a later write superseded it.
pub trait Liveness {
    fn is_live(&self, pk: u64, at: RowLocation) -> bool;
}

/// Keeps every row. The correct choice when no index is available, and the
/// behaviour a merge falls back to: **duplicates are still resolved** by
/// last-writer-wins, so this loses nothing, it merely reclaims no deleted space.
pub struct KeepAll;

impl Liveness for KeepAll {
    fn is_live(&self, _pk: u64, _at: RowLocation) -> bool {
        true
    }
}

impl<F> Liveness for F
where
    F: Fn(u64, RowLocation) -> bool,
{
    fn is_live(&self, pk: u64, at: RowLocation) -> bool {
        self(pk, at)
    }
}

/// What a compaction actually did.
#[derive(Debug, Clone, PartialEq)]
pub struct CompactionOutcome {
    /// The merged partition, ready to be recorded in the manifest.
    pub output: PartitionMeta,
    /// Input partitions to remove, as `(file_id, partition_id)`.
    pub removed: Vec<(u16, u16)>,
    /// Paths of the input files, for deletion after the manifest is updated.
    pub input_paths: Vec<PathBuf>,
    /// Primary keys of the merged output, **in written row order**.
    ///
    /// The caller must have this. Compaction moves every surviving row to a new
    /// `(file_id, partition_id, row_offset)`, so a PK index still pointing at the
    /// old coordinates addresses rows in a file that no longer exists. Returning
    /// the order means the index can be rebuilt exactly, with `row_offset` being
    /// the position in this vector — no second pass over the output file.
    pub output_pks: Vec<u64>,
    pub rows_in: u64,
    pub rows_out: u64,
    pub bytes_in: u64,
    pub bytes_out: u64,
    /// Input files the caller tried to delete and could not.
    ///
    /// Populated by the caller, not the executor. Not an error — the manifest no
    /// longer names them, so they are invisible to reads — but they are leaked
    /// disk, and a leak nobody is told about is one nobody cleans up.
    ///
    /// This is routine on **Windows**, where a live memory mapping holds the file
    /// open and `remove_file` fails with OS error 1224 until every reader is
    /// dropped. A caller holding `PartitionReader`s must release them before
    /// compacting, or every compaction leaks its inputs.
    pub undeleted: Vec<PathBuf>,
}

impl CompactionOutcome {
    /// Rows discarded as dead or superseded.
    pub fn rows_reclaimed(&self) -> u64 {
        self.rows_in.saturating_sub(self.rows_out)
    }

    /// Output bytes as a fraction of input bytes. Above 1.0 means the merge grew
    /// the data, which is possible and not necessarily wrong — see
    /// [`execute_job`].
    pub fn size_ratio(&self) -> f64 {
        if self.bytes_in == 0 {
            return 1.0;
        }
        self.bytes_out as f64 / self.bytes_in as f64
    }
}

/// Options for [`execute_job`].
#[derive(Debug, Clone)]
pub struct CompactionOptions {
    /// Directory for the merged file.
    pub output_dir: PathBuf,
    /// `file_id` for the output.
    pub output_file_id: u16,
    /// `partition_id` for the output. Must not collide with a live partition.
    pub output_partition_id: u16,
    /// Storage options for the merged file. Compaction is the natural place to
    /// enable LZ4, since a compacted partition is usually colder than a fresh one.
    pub write_options: WriteOptions,
    /// Milliseconds since the epoch, stamped on the output. Passed in rather than
    /// read from the clock so a merge is deterministic and testable.
    pub now_millis: u64,
}

/// Merge a job's inputs into one partition.
///
/// Writes the output file and returns everything needed to update the manifest.
/// **Does not touch the manifest and does not delete the inputs** — those are the
/// caller's, in that order, for the reasons in the module docs.
///
/// ## Merge semantics
///
/// - Rows are keyed by column 0, the primary key by convention.
/// - When a key appears in several inputs, the one from the **highest**
///   `(file_id, partition_id)` wins. Partition ids are allocated monotonically, so
///   that is last-writer-wins.
/// - A row the [`Liveness`] predicate rejects is dropped.
/// - Output is **sorted by primary key**. Not incidental: a sorted key column is
///   exactly the shape `BlockedFrame` compresses best (9x on sequential keys), and
///   it makes the output's zone map tight, so a later range scan can prune it.
///
/// ## The output can be larger than the inputs
///
/// Merging partitions with disjoint key ranges produces a wider range, which can
/// force a wider bit width than any input needed. With few dead rows to reclaim,
/// the result may exceed the inputs in bytes. That is a real outcome, reported via
/// [`CompactionOutcome::size_ratio`] rather than treated as an error — the merge is
/// still correct, and it still reduced the partition count, which is what read
/// amplification actually cares about.
pub fn execute_job<L: Liveness>(
    job: &CompactionJob,
    schema: &[String],
    liveness: &L,
    opts: &CompactionOptions,
) -> StorageResult<CompactionOutcome> {
    if job.inputs.is_empty() {
        return Err(StorageError::InvalidData(
            "compaction job has no inputs".into(),
        ));
    }
    if schema.is_empty() {
        return Err(StorageError::InvalidData(
            "compaction needs a non-empty schema".into(),
        ));
    }

    // Ascending, so a later partition's row overwrites an earlier one's on
    // insert. Relying on `BTreeMap::insert` returning the last write rather than
    // comparing ids per row keeps the inner loop free of branches.
    let mut inputs = job.inputs.clone();
    inputs.sort_by_key(|p| (p.file_id, p.partition_id));

    let col_count = schema.len();
    // BTreeMap, so the output is PK-sorted for free. A HashMap plus a sort at the
    // end would be marginally faster but would also make the output ordering
    // depend on hash iteration for equal keys.
    let mut live: BTreeMap<u64, Vec<u64>> = BTreeMap::new();
    let mut rows_in: u64 = 0;
    let mut bytes_in: u64 = 0;
    let mut scratch: Vec<u64> = Vec::new();

    for part in &inputs {
        let reader = PartitionReader::open(Path::new(&part.path))?;
        let n = reader.row_count() as usize;
        bytes_in += part.size_bytes;
        rows_in += n as u64;

        if reader.col_count() as usize != col_count {
            return Err(StorageError::InvalidData(format!(
                "partition {} has {} columns but the schema has {}",
                part.path.display(),
                reader.col_count(),
                col_count
            )));
        }

        // Decode column-wise and transpose, rather than calling read_row per row.
        // A row read touches every column separately; decoding each column once
        // and transposing does the same work with one pass per column.
        let mut cols: Vec<Vec<u64>> = Vec::with_capacity(col_count);
        for c in 0..col_count {
            reader.decode_column_into(c, &mut scratch)?;
            cols.push(std::mem::take(&mut scratch));
        }

        for row in 0..n {
            let pk = cols[0][row];
            let at = RowLocation {
                file_id: part.file_id,
                partition_id: part.partition_id,
                row_offset: row as u32,
            };
            if !liveness.is_live(pk, at) {
                continue;
            }
            live.insert(pk, cols.iter().map(|c| c[row]).collect());
        }
        // Reuse the largest buffer for the next partition.
        scratch = cols
            .into_iter()
            .max_by_key(|c| c.capacity())
            .unwrap_or_default();
    }

    // An empty result is legal — every row in the job was deleted. The caller
    // still needs the removals applied, so this is not an error; it produces an
    // outcome with no output partition rows.
    let rows_out = live.len() as u64;

    let filename = format!(
        "{:05}_{:05}.vvsst",
        opts.output_file_id, opts.output_partition_id
    );
    let out_path = opts.output_dir.join(&filename);
    std::fs::create_dir_all(&opts.output_dir)?;

    let (min_key, max_key) = match (live.keys().next(), live.keys().next_back()) {
        (Some(a), Some(b)) => (*a, *b),
        _ => (0, 0),
    };

    let mut mp = MicroPartition::new(opts.output_file_id, opts.output_partition_id);
    for (c, name) in schema.iter().enumerate() {
        let col: Vec<u64> = live.values().map(|r| r[c]).collect();
        // Column 0 stays Raw for the same reason the flush path keeps it Raw: it
        // is the point-lookup column, and an encoded column costs ~5x on
        // read_value. Everything else is auto-encoded.
        let encoded = if c == 0 {
            ColumnValues::Raw(col)
        } else {
            ColumnValues::auto_encode(col)
        };
        mp.push_column(name.clone(), encoded)?;
    }

    // Written to a temp name and renamed, so a crash mid-write cannot leave a
    // truncated file under the real name. A truncated .vvsst would be rejected by
    // its CRC on open, but only after the manifest already pointed at it.
    let tmp_path = opts.output_dir.join(format!("{filename}.tmp"));
    {
        let mut f = std::fs::File::create(&tmp_path)?;
        write_partition_with(&mut f, &mp, opts.write_options)?;
        use std::io::Write;
        f.flush()?;
        f.sync_all()?;
    }
    std::fs::rename(&tmp_path, &out_path)?;

    let bytes_out = std::fs::metadata(&out_path)?.len();

    Ok(CompactionOutcome {
        output: PartitionMeta {
            file_id: opts.output_file_id,
            partition_id: opts.output_partition_id,
            path: out_path,
            level: job.output_level,
            size_bytes: bytes_out,
            row_count: u32::try_from(rows_out).map_err(|_| {
                StorageError::InvalidData("merged partition exceeds u32 rows".into())
            })?,
            created_at_millis: opts.now_millis,
            min_key,
            max_key,
            // The merge just dropped every dead row it could see, so by
            // construction the output has none.
            tombstones: 0,
            // A WORM lock is a property of retained DATA, and compaction rewrites
            // it. Carrying the flag forward would be wrong (the bytes are new) and
            // dropping it silently would be worse. See `execute_job`'s caller
            // contract: a job whose inputs include a WORM-locked partition should
            // not be scheduled at all.
            worm_locked: false,
        },
        removed: inputs.iter().map(|p| (p.file_id, p.partition_id)).collect(),
        input_paths: inputs.iter().map(|p| p.path.clone()).collect(),
        output_pks: live.keys().copied().collect(),
        rows_in,
        rows_out,
        bytes_in,
        bytes_out,
        undeleted: Vec::new(),
    })
}

// ── tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lifecycle::CompactionReason;
    use tempfile::TempDir;

    fn schema() -> Vec<String> {
        vec!["pk".into(), "a".into(), "b".into()]
    }

    /// Write a partition with the given rows and return its metadata.
    fn write_part(dir: &Path, file_id: u16, pid: u16, rows: &[(u64, u64, u64)]) -> PartitionMeta {
        let mut mp = MicroPartition::new(file_id, pid);
        mp.push_column("pk", ColumnValues::Raw(rows.iter().map(|r| r.0).collect()))
            .unwrap();
        mp.push_column(
            "a",
            ColumnValues::auto_encode(rows.iter().map(|r| r.1).collect()),
        )
        .unwrap();
        mp.push_column(
            "b",
            ColumnValues::auto_encode(rows.iter().map(|r| r.2).collect()),
        )
        .unwrap();
        std::fs::create_dir_all(dir).unwrap();
        let path = dir.join(format!("{file_id:05}_{pid:05}.vvsst"));
        let mut f = std::fs::File::create(&path).unwrap();
        crate::write_partition(&mut f, &mp).unwrap();
        let size = std::fs::metadata(&path).unwrap().len();
        PartitionMeta {
            file_id,
            partition_id: pid,
            path,
            level: 0,
            size_bytes: size,
            row_count: rows.len() as u32,
            created_at_millis: 1000 + pid as u64,
            min_key: rows.iter().map(|r| r.0).min().unwrap_or(0),
            max_key: rows.iter().map(|r| r.0).max().unwrap_or(0),
            tombstones: 0,
            worm_locked: false,
        }
    }

    fn job(inputs: Vec<PartitionMeta>) -> CompactionJob {
        CompactionJob {
            inputs,
            output_level: 1,
            reason: CompactionReason::LevelOverflow,
        }
    }

    fn opts(dir: &Path) -> CompactionOptions {
        CompactionOptions {
            output_dir: dir.to_path_buf(),
            output_file_id: 9,
            output_partition_id: 900,
            write_options: WriteOptions::default(),
            now_millis: 5_000,
        }
    }

    fn read_back(p: &PartitionMeta) -> Vec<(u64, u64, u64)> {
        let r = PartitionReader::open(&p.path).unwrap();
        let mut out = Vec::new();
        for row in 0..r.row_count() as usize {
            out.push((
                r.read_value(0, row).unwrap(),
                r.read_value(1, row).unwrap(),
                r.read_value(2, row).unwrap(),
            ));
        }
        out
    }

    #[test]
    fn merges_disjoint_partitions_in_key_order() {
        let d = TempDir::new().unwrap();
        let a = write_part(d.path(), 0, 1, &[(30, 3, 33), (10, 1, 11)]);
        let b = write_part(d.path(), 0, 2, &[(20, 2, 22), (40, 4, 44)]);
        let out = execute_job(&job(vec![a, b]), &schema(), &KeepAll, &opts(d.path())).unwrap();

        assert_eq!(out.rows_in, 4);
        assert_eq!(out.rows_out, 4);
        assert_eq!(out.rows_reclaimed(), 0);
        // Sorted, regardless of input order — the property the encodings and the
        // zone map both depend on.
        assert_eq!(
            read_back(&out.output),
            vec![(10, 1, 11), (20, 2, 22), (30, 3, 33), (40, 4, 44)]
        );
        assert_eq!(out.output.min_key, 10);
        assert_eq!(out.output.max_key, 40);
        assert_eq!(out.output.level, 1);
        assert_eq!(out.output.row_count, 4);
    }

    /// A key in several inputs resolves to the HIGHEST partition id.
    ///
    /// Partition ids are allocated monotonically, so this is last-writer-wins.
    /// Getting it backwards would resurrect old values — a silent, permanent
    /// corruption that no CRC or bounds check catches.
    #[test]
    fn duplicate_keys_resolve_to_the_newest_partition() {
        let d = TempDir::new().unwrap();
        let old = write_part(d.path(), 0, 1, &[(5, 100, 100), (6, 100, 100)]);
        let mid = write_part(d.path(), 0, 2, &[(5, 200, 200)]);
        let new = write_part(d.path(), 0, 7, &[(5, 300, 300), (6, 300, 300)]);

        // Deliberately passed out of order, to prove the sort is doing the work.
        let out = execute_job(
            &job(vec![new, old, mid]),
            &schema(),
            &KeepAll,
            &opts(d.path()),
        )
        .unwrap();

        assert_eq!(out.rows_in, 5);
        assert_eq!(out.rows_out, 2, "duplicates should collapse");
        assert_eq!(read_back(&out.output), vec![(5, 300, 300), (6, 300, 300)]);
    }

    #[test]
    fn dead_rows_are_dropped() {
        let d = TempDir::new().unwrap();
        let a = write_part(
            d.path(),
            0,
            1,
            &[(1, 1, 1), (2, 2, 2), (3, 3, 3), (4, 4, 4)],
        );
        // Only even keys survive.
        let live = |pk: u64, _at: RowLocation| pk.is_multiple_of(2);
        let out = execute_job(&job(vec![a]), &schema(), &live, &opts(d.path())).unwrap();

        assert_eq!(out.rows_in, 4);
        assert_eq!(out.rows_out, 2);
        assert_eq!(out.rows_reclaimed(), 2);
        assert_eq!(read_back(&out.output), vec![(2, 2, 2), (4, 4, 4)]);
    }

    /// Liveness sees the row's real location, so it can distinguish two copies of
    /// the same key in different partitions.
    ///
    /// A predicate keyed only on the PK could not express "the copy in partition 1
    /// is stale" — which is exactly what an index-backed predicate needs to say.
    #[test]
    fn liveness_receives_the_true_row_location() {
        let d = TempDir::new().unwrap();
        let p1 = write_part(d.path(), 0, 1, &[(5, 111, 111)]);
        let p2 = write_part(d.path(), 0, 2, &[(5, 222, 222)]);

        // Accept only the copy living in partition 1 — the OPPOSITE of
        // last-writer-wins, so if locations were ignored the result would be 222.
        let live = |_pk: u64, at: RowLocation| at.partition_id == 1;
        let out = execute_job(&job(vec![p1, p2]), &schema(), &live, &opts(d.path())).unwrap();

        assert_eq!(out.rows_out, 1);
        assert_eq!(
            read_back(&out.output),
            vec![(5, 111, 111)],
            "the liveness predicate was not given the real location"
        );
    }

    /// Every row dead is legal, not an error.
    ///
    /// A partition whose rows were all deleted is precisely the case garbage
    /// collection exists for; erroring would leave the wasted space forever.
    #[test]
    fn a_fully_dead_job_produces_an_empty_partition() {
        let d = TempDir::new().unwrap();
        let a = write_part(d.path(), 0, 1, &[(1, 1, 1), (2, 2, 2)]);
        let dead = |_pk: u64, _at: RowLocation| false;
        let out = execute_job(&job(vec![a.clone()]), &schema(), &dead, &opts(d.path())).unwrap();

        assert_eq!(out.rows_out, 0);
        assert_eq!(out.rows_reclaimed(), 2);
        assert_eq!(out.output.row_count, 0);
        assert_eq!((out.output.min_key, out.output.max_key), (0, 0));
        assert_eq!(out.removed, vec![(0, 1)], "the input must still be removed");
        // And the file is readable rather than malformed.
        let r = PartitionReader::open(&out.output.path).unwrap();
        assert_eq!(r.row_count(), 0);
    }

    #[test]
    fn output_carries_no_tombstones_and_no_worm_lock() {
        let d = TempDir::new().unwrap();
        let mut a = write_part(d.path(), 0, 1, &[(1, 1, 1)]);
        a.tombstones = 500;
        a.worm_locked = true;
        let out = execute_job(&job(vec![a]), &schema(), &KeepAll, &opts(d.path())).unwrap();
        assert_eq!(out.output.tombstones, 0, "merged output has no dead rows");
        assert!(
            !out.output.worm_locked,
            "a WORM lock must not silently transfer to rewritten bytes"
        );
    }

    #[test]
    fn empty_job_and_empty_schema_are_rejected() {
        let d = TempDir::new().unwrap();
        assert!(execute_job(&job(vec![]), &schema(), &KeepAll, &opts(d.path())).is_err());
        let a = write_part(d.path(), 0, 1, &[(1, 1, 1)]);
        assert!(execute_job(&job(vec![a]), &[], &KeepAll, &opts(d.path())).is_err());
    }

    #[test]
    fn schema_width_mismatch_is_an_error_not_a_panic() {
        let d = TempDir::new().unwrap();
        let a = write_part(d.path(), 0, 1, &[(1, 1, 1)]);
        let wide = vec!["pk".into(), "a".into(), "b".into(), "c".into()];
        let err = execute_job(&job(vec![a]), &wide, &KeepAll, &opts(d.path()));
        assert!(
            err.is_err(),
            "a wider schema than the file must be rejected"
        );
    }

    /// No `.tmp` file is left behind, and the output is complete.
    ///
    /// The output is written to a temp name and renamed so a crash mid-write
    /// cannot leave a truncated file under the real name — the manifest would then
    /// point at something that fails its CRC.
    #[test]
    fn output_is_renamed_into_place_leaving_no_temp_file() {
        let d = TempDir::new().unwrap();
        let a = write_part(d.path(), 0, 1, &[(1, 1, 1), (2, 2, 2)]);
        let out = execute_job(&job(vec![a]), &schema(), &KeepAll, &opts(d.path())).unwrap();

        assert!(out.output.path.exists());
        let temps: Vec<_> = std::fs::read_dir(d.path())
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().to_string_lossy().ends_with(".tmp"))
            .collect();
        assert!(temps.is_empty(), "a .tmp file was left behind");
        // Recorded size matches reality — retention budgets depend on it.
        assert_eq!(
            out.output.size_bytes,
            std::fs::metadata(&out.output.path).unwrap().len()
        );
        assert_eq!(out.bytes_out, out.output.size_bytes);
    }

    /// Inputs are NOT deleted by the executor.
    ///
    /// Deletion must happen after the manifest swap, or a crash between the two
    /// loses data. Asserting it here keeps that ordering from being "optimised"
    /// into the executor later.
    #[test]
    fn inputs_are_left_on_disk_for_the_caller_to_remove() {
        let d = TempDir::new().unwrap();
        let a = write_part(d.path(), 0, 1, &[(1, 1, 1)]);
        let b = write_part(d.path(), 0, 2, &[(2, 2, 2)]);
        let paths = vec![a.path.clone(), b.path.clone()];
        let out = execute_job(&job(vec![a, b]), &schema(), &KeepAll, &opts(d.path())).unwrap();

        for p in &paths {
            assert!(p.exists(), "the executor deleted an input: {}", p.display());
        }
        assert_eq!(out.input_paths, paths);
        assert_eq!(out.removed, vec![(0, 1), (0, 2)]);
    }

    /// Compaction reclaims real space on a realistic shape.
    #[test]
    fn compaction_shrinks_a_duplicate_heavy_set() {
        let d = TempDir::new().unwrap();
        // Four partitions that all rewrite the same 500 keys.
        let inputs: Vec<PartitionMeta> = (1..=4u16)
            .map(|generation| {
                let rows: Vec<(u64, u64, u64)> = (0..500u64)
                    .map(|k| (k, generation as u64 * 1000 + k, k * 2))
                    .collect();
                write_part(d.path(), 0, generation, &rows)
            })
            .collect();
        let bytes_before: u64 = inputs.iter().map(|p| p.size_bytes).sum();

        let out = execute_job(&job(inputs), &schema(), &KeepAll, &opts(d.path())).unwrap();
        assert_eq!(out.rows_in, 2000);
        assert_eq!(out.rows_out, 500, "3 of every 4 rows are superseded");
        assert_eq!(out.bytes_in, bytes_before);
        assert!(
            out.size_ratio() < 0.5,
            "expected a large shrink, got ratio {:.3}",
            out.size_ratio()
        );
        // The surviving values are all from the newest generation.
        let rows = read_back(&out.output);
        assert_eq!(rows.len(), 500);
        assert_eq!(rows[0], (0, 4000, 0));
        assert_eq!(rows[499], (499, 4499, 998));
    }

    /// LZ4 can be enabled for the merged output.
    #[test]
    fn compaction_can_compress_its_output() {
        let d = TempDir::new().unwrap();
        let rows: Vec<(u64, u64, u64)> = (0..2000u64).map(|k| (k, k % 7, k * 3)).collect();
        let a = write_part(d.path(), 0, 1, &rows);

        let mut o = opts(d.path());
        o.write_options = WriteOptions::new().with_lz4();
        let out = execute_job(&job(vec![a.clone()]), &schema(), &KeepAll, &o).unwrap();

        assert!(
            out.bytes_out < a.size_bytes,
            "compressed output {} not smaller than input {}",
            out.bytes_out,
            a.size_bytes
        );
        assert_eq!(read_back(&out.output), rows, "compression changed the data");
    }
}
