//! [`Manifest`] — the durable record of which partitions exist (VV-STO-025).
//!
//! ## Why this exists
//!
//! Compaction planning, retention, ZoneMap pruning, and verification are all
//! written and tested — and none of them could run, because nothing tracked what
//! partitions exist beyond their filenames. Levels, tombstone counts, creation
//! times, and key ranges lived nowhere. This is the missing input.
//!
//! ## Design: an append-only edit log, not a rewritten state file
//!
//! The obvious implementation is a state file rewritten on every change. It is
//! wrong for the same reason a database does not rewrite its data file in place:
//! a crash midway through leaves a file that is neither the old state nor the new
//! one, and the old state is gone.
//!
//! Instead this appends **edits**. Recovery replays them. A crash can only ever
//! lose or truncate the final edit, never corrupt what came before, because
//! nothing already written is ever modified. The log is periodically rewritten as
//! a compact snapshot via write-to-temp-then-rename, which is atomic on both
//! POSIX and Windows.
//!
//! ```text
//! ┌──────────────────────────────────────┐
//! │ magic[8] "VVMANIF1"                  │
//! ├──────────────────────────────────────┤
//! │ record: payload_len[4] crc32[4] body │
//! │ record: payload_len[4] crc32[4] body │
//! │ ...                                  │
//! └──────────────────────────────────────┘
//! ```
//!
//! ## The torn-write ambiguity, stated rather than hidden
//!
//! A partial write at the end of the log is normal — the process died mid-append.
//! Corruption in the middle of the log is a disk problem. **These two are not
//! distinguishable from the bytes alone**: both present as a record whose length
//! or CRC does not check out.
//!
//! Replay resolves the ambiguity by treating the first bad record as end-of-log,
//! and then *reporting how many bytes it discarded* in
//! [`ManifestRecovery::discarded_bytes`]. A torn tail discards less than one
//! record; mid-log corruption discards a lot. The caller gets the number and can
//! alarm on it. Silently truncating without saying how much is the version of this
//! that loses data quietly, and is specifically what is avoided here.

use std::collections::BTreeMap;
use std::io::Write;
use std::path::{Path, PathBuf};

use crate::lifecycle::PartitionMeta;
use crate::{StorageError, StorageResult};

/// The live partition set, keyed by `(file_id, partition_id)`.
///
/// `BTreeMap` rather than `HashMap` so iteration order is deterministic: every
/// compaction and retention plan is computed from this, and a plan that varied
/// run to run for identical input would be untestable and unreviewable.
type PartitionState = BTreeMap<(u16, u16), PartitionMeta>;

/// Magic bytes at the head of a manifest file.
pub const MANIFEST_MAGIC: &[u8; 8] = b"VVMANIF1";

/// Largest edit payload accepted during replay.
///
/// A length prefix read from a corrupt file is an allocation request. Without a
/// cap, four bad bytes ask for a 4 GiB `Vec` — an OOM abort caused by a bad file,
/// which no amount of caller care prevents. Generous relative to a real edit
/// (~100 bytes) and small enough to be harmless.
const MAX_EDIT_BYTES: u32 = 1 << 20;

/// One change to the set of partitions.
///
/// Field updates are separate variants rather than a whole-record rewrite so that
/// a tombstone count changing does not re-log a path and a key range. Compaction
/// updates tombstones far more often than anything else changes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ManifestEdit {
    Add(Box<PartitionMeta>),
    Remove {
        file_id: u16,
        partition_id: u16,
    },
    SetTombstones {
        file_id: u16,
        partition_id: u16,
        tombstones: u32,
    },
    SetLevel {
        file_id: u16,
        partition_id: u16,
        level: u8,
    },
    SetWormLock {
        file_id: u16,
        partition_id: u16,
        locked: bool,
    },
}

const TAG_ADD: u8 = 1;
const TAG_REMOVE: u8 = 2;
const TAG_TOMBSTONES: u8 = 3;
const TAG_LEVEL: u8 = 4;
const TAG_WORM: u8 = 5;

impl ManifestEdit {
    fn encode(&self) -> StorageResult<Vec<u8>> {
        let mut b = Vec::with_capacity(64);
        match self {
            ManifestEdit::Add(p) => {
                b.push(TAG_ADD);
                b.extend_from_slice(&p.file_id.to_le_bytes());
                b.extend_from_slice(&p.partition_id.to_le_bytes());
                b.push(p.level);
                b.push(u8::from(p.worm_locked));
                b.extend_from_slice(&p.size_bytes.to_le_bytes());
                b.extend_from_slice(&p.row_count.to_le_bytes());
                b.extend_from_slice(&p.created_at_millis.to_le_bytes());
                b.extend_from_slice(&p.min_key.to_le_bytes());
                b.extend_from_slice(&p.max_key.to_le_bytes());
                b.extend_from_slice(&p.tombstones.to_le_bytes());

                // Paths are stored as UTF-8. A non-UTF-8 path (possible on both
                // Linux and Windows) is rejected at WRITE time rather than
                // silently lossily converted, so the manifest can never name a
                // file that does not exist.
                let path = p.path.to_str().ok_or_else(|| {
                    StorageError::InvalidData(format!(
                        "partition path is not valid UTF-8 and cannot be recorded: {:?}",
                        p.path
                    ))
                })?;
                let bytes = path.as_bytes();
                let len = u16::try_from(bytes.len()).map_err(|_| {
                    StorageError::InvalidData(format!("partition path exceeds 65535 bytes: {path}"))
                })?;
                b.extend_from_slice(&len.to_le_bytes());
                b.extend_from_slice(bytes);
            }
            ManifestEdit::Remove {
                file_id,
                partition_id,
            } => {
                b.push(TAG_REMOVE);
                b.extend_from_slice(&file_id.to_le_bytes());
                b.extend_from_slice(&partition_id.to_le_bytes());
            }
            ManifestEdit::SetTombstones {
                file_id,
                partition_id,
                tombstones,
            } => {
                b.push(TAG_TOMBSTONES);
                b.extend_from_slice(&file_id.to_le_bytes());
                b.extend_from_slice(&partition_id.to_le_bytes());
                b.extend_from_slice(&tombstones.to_le_bytes());
            }
            ManifestEdit::SetLevel {
                file_id,
                partition_id,
                level,
            } => {
                b.push(TAG_LEVEL);
                b.extend_from_slice(&file_id.to_le_bytes());
                b.extend_from_slice(&partition_id.to_le_bytes());
                b.push(*level);
            }
            ManifestEdit::SetWormLock {
                file_id,
                partition_id,
                locked,
            } => {
                b.push(TAG_WORM);
                b.extend_from_slice(&file_id.to_le_bytes());
                b.extend_from_slice(&partition_id.to_le_bytes());
                b.push(u8::from(*locked));
            }
        }
        Ok(b)
    }

    fn decode(b: &[u8]) -> StorageResult<Self> {
        // Every read goes through a cursor that bounds-checks, because these bytes
        // came off disk and a length field can name anything.
        struct Cur<'a> {
            b: &'a [u8],
            at: usize,
        }
        impl Cur<'_> {
            fn take(&mut self, n: usize) -> StorageResult<&[u8]> {
                let end = self.at.checked_add(n).ok_or_else(bad)?;
                let s = self.b.get(self.at..end).ok_or_else(bad)?;
                self.at = end;
                Ok(s)
            }
            fn u8(&mut self) -> StorageResult<u8> {
                Ok(self.take(1)?[0])
            }
            fn u16(&mut self) -> StorageResult<u16> {
                Ok(u16::from_le_bytes(self.take(2)?.try_into().unwrap()))
            }
            fn u32(&mut self) -> StorageResult<u32> {
                Ok(u32::from_le_bytes(self.take(4)?.try_into().unwrap()))
            }
            fn u64(&mut self) -> StorageResult<u64> {
                Ok(u64::from_le_bytes(self.take(8)?.try_into().unwrap()))
            }
        }
        fn bad() -> StorageError {
            StorageError::InvalidData("manifest edit is truncated".into())
        }

        let mut c = Cur { b, at: 0 };
        let tag = c.u8()?;
        let file_id = c.u16()?;
        let partition_id = c.u16()?;
        match tag {
            TAG_ADD => {
                let level = c.u8()?;
                let worm_locked = c.u8()? != 0;
                let size_bytes = c.u64()?;
                let row_count = c.u32()?;
                let created_at_millis = c.u64()?;
                let min_key = c.u64()?;
                let max_key = c.u64()?;
                let tombstones = c.u32()?;
                let plen = c.u16()? as usize;
                let pbytes = c.take(plen)?;
                let path = std::str::from_utf8(pbytes).map_err(|_| {
                    StorageError::InvalidData("manifest path is not valid UTF-8".into())
                })?;
                Ok(ManifestEdit::Add(Box::new(PartitionMeta {
                    file_id,
                    partition_id,
                    path: PathBuf::from(path),
                    level,
                    size_bytes,
                    row_count,
                    created_at_millis,
                    min_key,
                    max_key,
                    tombstones,
                    worm_locked,
                })))
            }
            TAG_REMOVE => Ok(ManifestEdit::Remove {
                file_id,
                partition_id,
            }),
            TAG_TOMBSTONES => Ok(ManifestEdit::SetTombstones {
                file_id,
                partition_id,
                tombstones: c.u32()?,
            }),
            TAG_LEVEL => Ok(ManifestEdit::SetLevel {
                file_id,
                partition_id,
                level: c.u8()?,
            }),
            TAG_WORM => Ok(ManifestEdit::SetWormLock {
                file_id,
                partition_id,
                locked: c.u8()? != 0,
            }),
            other => Err(StorageError::InvalidData(format!(
                "unknown manifest edit tag {other}"
            ))),
        }
    }
}

/// What replay found. Returned alongside the manifest so recovery is visible.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ManifestRecovery {
    /// Edits successfully replayed.
    pub edits_applied: usize,
    /// Trailing bytes that did not form a valid record and were discarded.
    ///
    /// **Zero is the only fully clean value.** A small number is a torn tail — the
    /// process died mid-append, which is normal and expected. A large number means
    /// mid-log corruption and the manifest may be missing real partitions. See the
    /// module docs: the bytes cannot distinguish these, so the count is reported
    /// rather than judged.
    pub discarded_bytes: u64,
}

impl ManifestRecovery {
    /// True if nothing at all had to be discarded.
    pub fn is_clean(&self) -> bool {
        self.discarded_bytes == 0
    }
}

/// The durable set of live partitions.
pub struct Manifest {
    path: PathBuf,
    /// Keyed by `(file_id, partition_id)` and ordered, so iteration and therefore
    /// every downstream plan is deterministic. A `HashMap` here would make
    /// compaction plans vary run to run for identical input.
    partitions: PartitionState,
    file: std::fs::File,
    /// Edits appended since the last rewrite, for the compaction heuristic.
    edits_since_rewrite: usize,
}

impl Manifest {
    /// Edits appended before the log is rewritten as a compact snapshot.
    ///
    /// Bounds replay time: without a rewrite, a long-running table replays every
    /// edit ever made at startup. The value trades startup time against rewrite
    /// frequency and is not tuned against a measurement.
    pub const REWRITE_AFTER_EDITS: usize = 1000;

    /// Open an existing manifest or create an empty one.
    pub fn open(path: impl AsRef<Path>) -> StorageResult<(Self, ManifestRecovery)> {
        let path = path.as_ref().to_path_buf();
        let (partitions, recovery, valid_len) = if path.exists() {
            Self::replay(&path)?
        } else {
            (BTreeMap::new(), ManifestRecovery::default(), 0)
        };

        // Truncate the torn tail before appending. Otherwise the next append lands
        // after the garbage, and the next replay stops at that garbage and loses
        // everything written afterwards — a silent, permanent data loss that only
        // shows up on the SECOND restart.
        let file = if path.exists() {
            let f = std::fs::OpenOptions::new()
                .read(true)
                .write(true)
                .open(&path)?;
            if recovery.discarded_bytes > 0 {
                f.set_len(valid_len)?;
            }
            let mut f = f;
            use std::io::Seek;
            f.seek(std::io::SeekFrom::End(0))?;
            f
        } else {
            let mut f = std::fs::OpenOptions::new()
                .create_new(true)
                .read(true)
                .write(true)
                .open(&path)?;
            f.write_all(MANIFEST_MAGIC)?;
            f.sync_all()?;
            f
        };

        Ok((
            Manifest {
                path,
                partitions,
                file,
                edits_since_rewrite: recovery.edits_applied,
            },
            recovery,
        ))
    }

    /// Replay a manifest file. Returns the state, what recovery did, and the byte
    /// offset of the end of the last valid record.
    fn replay(path: &Path) -> StorageResult<(PartitionState, ManifestRecovery, u64)> {
        let data = std::fs::read(path)?;
        if data.len() < MANIFEST_MAGIC.len() || &data[..8] != MANIFEST_MAGIC.as_slice() {
            return Err(StorageError::InvalidData(
                "manifest magic missing or wrong".into(),
            ));
        }

        let mut state: PartitionState = BTreeMap::new();
        let mut recovery = ManifestRecovery::default();
        let mut at = MANIFEST_MAGIC.len();

        // Header is 8 bytes: length + CRC. Anything shorter is a torn tail.
        while let Some(header) = data.get(at..at + 8) {
            let len = u32::from_le_bytes(header[0..4].try_into().unwrap());
            let crc = u32::from_le_bytes(header[4..8].try_into().unwrap());

            if len == 0 || len > MAX_EDIT_BYTES {
                break;
            }
            let body_start = at + 8;
            let Some(body) = data.get(body_start..body_start + len as usize) else {
                break; // record announced more bytes than the file holds
            };
            if crc32fast::hash(body) != crc {
                break; // torn write, or corruption — see module docs
            }
            let Ok(edit) = ManifestEdit::decode(body) else {
                break;
            };

            Self::apply_to(&mut state, edit);
            recovery.edits_applied += 1;
            at = body_start + len as usize;
        }

        recovery.discarded_bytes = (data.len() - at) as u64;
        Ok((state, recovery, at as u64))
    }

    fn apply_to(state: &mut PartitionState, edit: ManifestEdit) {
        match edit {
            ManifestEdit::Add(p) => {
                state.insert((p.file_id, p.partition_id), *p);
            }
            ManifestEdit::Remove {
                file_id,
                partition_id,
            } => {
                state.remove(&(file_id, partition_id));
            }
            // A field update for a partition that is not present is IGNORED, not
            // an error. Replay must tolerate `SetTombstones` after `Remove` —
            // ordinary if a compaction finished while a stats update was in
            // flight — and erroring there would make a recoverable log unopenable.
            ManifestEdit::SetTombstones {
                file_id,
                partition_id,
                tombstones,
            } => {
                if let Some(p) = state.get_mut(&(file_id, partition_id)) {
                    p.tombstones = tombstones;
                }
            }
            ManifestEdit::SetLevel {
                file_id,
                partition_id,
                level,
            } => {
                if let Some(p) = state.get_mut(&(file_id, partition_id)) {
                    p.level = level;
                }
            }
            ManifestEdit::SetWormLock {
                file_id,
                partition_id,
                locked,
            } => {
                if let Some(p) = state.get_mut(&(file_id, partition_id)) {
                    p.worm_locked = locked;
                }
            }
        }
    }

    /// Apply an edit and make it durable before returning.
    ///
    /// `sync_all` on every edit. The manifest is the index of what exists; losing
    /// an edit means either a partition file nobody knows about (leaked disk) or a
    /// manifest entry for a file that was deleted (a lookup into nothing). Edits
    /// happen per flush and per compaction, not per row, so the fsync is
    /// affordable here in a way it is not on the write path.
    pub fn apply(&mut self, edit: ManifestEdit) -> StorageResult<()> {
        let body = edit.encode()?;
        let len = u32::try_from(body.len())
            .map_err(|_| StorageError::InvalidData("manifest edit too large".into()))?;

        // One `write_all` for the whole record. A record split across two writes
        // could interleave with another process appending to the same file.
        let mut record = Vec::with_capacity(8 + body.len());
        record.extend_from_slice(&len.to_le_bytes());
        record.extend_from_slice(&crc32fast::hash(&body).to_le_bytes());
        record.extend_from_slice(&body);
        self.file.write_all(&record)?;
        self.file.sync_all()?;

        Self::apply_to(&mut self.partitions, edit);
        self.edits_since_rewrite += 1;
        if self.edits_since_rewrite >= Self::REWRITE_AFTER_EDITS {
            self.rewrite()?;
        }
        Ok(())
    }

    /// Rewrite the log as one `Add` per live partition.
    ///
    /// Write-to-temp-then-rename, which is atomic on POSIX and on Windows via
    /// `ReplaceFile` semantics in `std::fs::rename`. A crash leaves either the old
    /// complete log or the new complete log, never a half-written one — the same
    /// reason the log is append-only in the first place.
    pub fn rewrite(&mut self) -> StorageResult<()> {
        let tmp = self.path.with_extension("manifest.tmp");
        {
            let mut f = std::fs::File::create(&tmp)?;
            f.write_all(MANIFEST_MAGIC)?;
            for p in self.partitions.values() {
                let body = ManifestEdit::Add(Box::new(p.clone())).encode()?;
                let len = u32::try_from(body.len())
                    .map_err(|_| StorageError::InvalidData("manifest edit too large".into()))?;
                f.write_all(&len.to_le_bytes())?;
                f.write_all(&crc32fast::hash(&body).to_le_bytes())?;
                f.write_all(&body)?;
            }
            // The temp file must be durable BEFORE the rename. Renaming a file
            // whose contents are still in the page cache can, after a power loss,
            // leave the manifest name pointing at an empty file — the old log
            // already unlinked. This ordering is the whole point of the dance.
            f.sync_all()?;
        }
        std::fs::rename(&tmp, &self.path)?;

        let mut f = std::fs::OpenOptions::new()
            .read(true)
            .append(true)
            .open(&self.path)?;
        use std::io::Seek;
        f.seek(std::io::SeekFrom::End(0))?;
        self.file = f;
        self.edits_since_rewrite = 0;
        Ok(())
    }

    /// Every live partition, ordered by `(file_id, partition_id)`.
    ///
    /// This is the input to [`crate::CompactionStrategy::plan`],
    /// [`crate::RetentionPolicy::plan`], and [`crate::verify_partitions`] — the
    /// three layers that had no way to run before this existed.
    pub fn partitions(&self) -> Vec<PartitionMeta> {
        self.partitions.values().cloned().collect()
    }

    pub fn get(&self, file_id: u16, partition_id: u16) -> Option<&PartitionMeta> {
        self.partitions.get(&(file_id, partition_id))
    }

    pub fn len(&self) -> usize {
        self.partitions.len()
    }

    pub fn is_empty(&self) -> bool {
        self.partitions.is_empty()
    }

    /// Total bytes across all live partitions.
    pub fn total_bytes(&self) -> u64 {
        self.partitions.values().map(|p| p.size_bytes).sum()
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Edits appended since the last rewrite.
    pub fn edits_since_rewrite(&self) -> usize {
        self.edits_since_rewrite
    }
}

// ── tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn meta(id: u16) -> PartitionMeta {
        PartitionMeta {
            file_id: 1,
            partition_id: id,
            path: PathBuf::from(format!("data/{id:05}.vvsst")),
            level: (id % 3) as u8,
            size_bytes: 1000 + id as u64,
            row_count: 500 + id as u32,
            created_at_millis: 1_700_000_000_000 + id as u64,
            min_key: id as u64 * 100,
            max_key: id as u64 * 100 + 99,
            tombstones: id as u32,
            worm_locked: id.is_multiple_of(5),
        }
    }

    #[test]
    fn every_edit_round_trips_through_encoding() {
        let edits = vec![
            ManifestEdit::Add(Box::new(meta(0))),
            ManifestEdit::Add(Box::new(meta(65535))),
            ManifestEdit::Remove {
                file_id: 7,
                partition_id: 9,
            },
            ManifestEdit::SetTombstones {
                file_id: 1,
                partition_id: 2,
                tombstones: u32::MAX,
            },
            ManifestEdit::SetLevel {
                file_id: 1,
                partition_id: 2,
                level: 255,
            },
            ManifestEdit::SetWormLock {
                file_id: 1,
                partition_id: 2,
                locked: true,
            },
            ManifestEdit::SetWormLock {
                file_id: 1,
                partition_id: 2,
                locked: false,
            },
        ];
        for e in edits {
            let bytes = e.encode().unwrap();
            assert_eq!(ManifestEdit::decode(&bytes).unwrap(), e, "round-trip");
        }
    }

    #[test]
    fn state_survives_reopen() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("MANIFEST");

        {
            let (mut m, rec) = Manifest::open(&path).unwrap();
            assert!(rec.is_clean());
            assert!(m.is_empty());
            for i in 0..5 {
                m.apply(ManifestEdit::Add(Box::new(meta(i)))).unwrap();
            }
            m.apply(ManifestEdit::SetTombstones {
                file_id: 1,
                partition_id: 3,
                tombstones: 999,
            })
            .unwrap();
            m.apply(ManifestEdit::Remove {
                file_id: 1,
                partition_id: 1,
            })
            .unwrap();
        }

        let (m, rec) = Manifest::open(&path).unwrap();
        assert!(rec.is_clean(), "clean shutdown must discard nothing");
        assert_eq!(rec.edits_applied, 7);
        assert_eq!(m.len(), 4, "5 added, 1 removed");
        assert!(m.get(1, 1).is_none(), "removed partition came back");
        assert_eq!(m.get(1, 3).unwrap().tombstones, 999, "field update lost");
        // Ordering is deterministic, which every downstream plan depends on.
        let ids: Vec<u16> = m.partitions().iter().map(|p| p.partition_id).collect();
        assert_eq!(ids, vec![0, 2, 3, 4]);
    }

    /// **Truncate at every byte offset; replay must never panic and must always
    /// yield a valid prefix of the history.**
    ///
    /// This is the crash test. A process killed mid-append leaves the file cut at
    /// an arbitrary offset — not a record boundary — so every offset is a real
    /// scenario, including partway through a length prefix, a CRC, or a path.
    #[test]
    fn truncation_at_any_offset_recovers_a_valid_prefix() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("MANIFEST");
        {
            let (mut m, _) = Manifest::open(&path).unwrap();
            for i in 0..8 {
                m.apply(ManifestEdit::Add(Box::new(meta(i)))).unwrap();
            }
        }
        let full = std::fs::read(&path).unwrap();

        for cut in 0..=full.len() {
            let d = TempDir::new().unwrap();
            let p = d.path().join("MANIFEST");
            std::fs::write(&p, &full[..cut]).unwrap();

            match Manifest::open(&p) {
                Err(_) => {
                    // Only acceptable when the magic itself is incomplete.
                    assert!(
                        cut < MANIFEST_MAGIC.len(),
                        "cut {cut} rejected a valid file"
                    );
                }
                Ok((m, rec)) => {
                    // Whatever survived must be a PREFIX of the real history: the
                    // partitions present must be exactly 0..n for some n.
                    let ids: Vec<u16> = m.partitions().iter().map(|p| p.partition_id).collect();
                    let expected: Vec<u16> = (0..ids.len() as u16).collect();
                    assert_eq!(
                        ids, expected,
                        "cut {cut}: recovered a non-prefix state {ids:?}"
                    );
                    assert_eq!(rec.edits_applied, ids.len());
                    // And every recovered partition is intact, not half-decoded.
                    for p in m.partitions() {
                        assert_eq!(p, meta(p.partition_id), "cut {cut}: corrupted record");
                    }
                }
            }
        }
    }

    /// A torn tail must be truncated on open, or the NEXT append lands after the
    /// garbage and the append after that is lost forever.
    ///
    /// This is the subtle one: without the truncation, everything looks fine on the
    /// first restart — replay stops at the garbage and the state is right — but the
    /// new edits are written past it, so the SECOND restart stops at the same
    /// garbage and silently loses all of them.
    #[test]
    fn torn_tail_is_truncated_so_later_appends_survive() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("MANIFEST");
        {
            let (mut m, _) = Manifest::open(&path).unwrap();
            for i in 0..3 {
                m.apply(ManifestEdit::Add(Box::new(meta(i)))).unwrap();
            }
        }
        // Simulate a partial append: a plausible header plus a truncated body.
        {
            use std::io::Write as _;
            let mut f = std::fs::OpenOptions::new()
                .append(true)
                .open(&path)
                .unwrap();
            f.write_all(&40u32.to_le_bytes()).unwrap();
            f.write_all(&0xDEAD_BEEFu32.to_le_bytes()).unwrap();
            f.write_all(&[0xAB; 11]).unwrap();
        }

        // First reopen: torn bytes reported and discarded.
        {
            let (mut m, rec) = Manifest::open(&path).unwrap();
            assert_eq!(rec.edits_applied, 3);
            assert_eq!(rec.discarded_bytes, 19, "torn tail size should be reported");
            assert!(!rec.is_clean());
            m.apply(ManifestEdit::Add(Box::new(meta(99)))).unwrap();
        }

        // Second reopen: the post-recovery append must still be there.
        let (m, rec) = Manifest::open(&path).unwrap();
        assert!(rec.is_clean(), "the tail was not truncated: {rec:?}");
        assert_eq!(m.len(), 4);
        assert!(
            m.get(1, 99).is_some(),
            "an edit written after recovery was lost on the next restart"
        );
    }

    #[test]
    fn rewrite_compacts_the_log_without_changing_state() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("MANIFEST");
        let (mut m, _) = Manifest::open(&path).unwrap();

        // Lots of churn: 50 adds, then 40 removes, leaving 10.
        for i in 0..50 {
            m.apply(ManifestEdit::Add(Box::new(meta(i)))).unwrap();
        }
        for i in 0..40 {
            m.apply(ManifestEdit::Remove {
                file_id: 1,
                partition_id: i,
            })
            .unwrap();
        }
        let before = m.partitions();
        let big = std::fs::metadata(&path).unwrap().len();

        m.rewrite().unwrap();
        let after_len = std::fs::metadata(&path).unwrap().len();
        assert!(
            after_len < big,
            "rewrite did not shrink the log: {big} -> {after_len}"
        );
        assert_eq!(m.edits_since_rewrite(), 0);
        assert_eq!(m.partitions(), before, "rewrite changed the state");

        // And a further append after the rewrite survives a reopen — the rewrite
        // must leave the file handle positioned at the end, not at the old offset.
        m.apply(ManifestEdit::Add(Box::new(meta(500)))).unwrap();
        drop(m);
        let (m2, rec) = Manifest::open(&path).unwrap();
        assert!(rec.is_clean(), "{rec:?}");
        assert_eq!(m2.len(), before.len() + 1);
        assert!(m2.get(1, 500).is_some(), "post-rewrite append lost");
    }

    #[test]
    fn automatic_rewrite_fires_and_preserves_state() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("MANIFEST");
        let (mut m, _) = Manifest::open(&path).unwrap();

        // Churn one partition well past the threshold.
        m.apply(ManifestEdit::Add(Box::new(meta(1)))).unwrap();
        for i in 0..Manifest::REWRITE_AFTER_EDITS as u32 + 10 {
            m.apply(ManifestEdit::SetTombstones {
                file_id: 1,
                partition_id: 1,
                tombstones: i,
            })
            .unwrap();
        }
        assert!(
            m.edits_since_rewrite() < Manifest::REWRITE_AFTER_EDITS,
            "automatic rewrite never fired"
        );
        drop(m);

        let (m, rec) = Manifest::open(&path).unwrap();
        assert!(rec.is_clean(), "{rec:?}");
        assert_eq!(m.len(), 1);
        assert_eq!(
            m.get(1, 1).unwrap().tombstones,
            Manifest::REWRITE_AFTER_EDITS as u32 + 9,
            "the last update before the rewrite boundary was lost"
        );
    }

    /// A field update for a partition that no longer exists is ignored, not fatal.
    ///
    /// Ordinary during recovery: a compaction removed a partition while a stats
    /// update for it was in flight. Erroring would make a recoverable log
    /// permanently unopenable.
    #[test]
    fn updates_to_removed_partitions_are_ignored() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("MANIFEST");
        {
            let (mut m, _) = Manifest::open(&path).unwrap();
            m.apply(ManifestEdit::Add(Box::new(meta(1)))).unwrap();
            m.apply(ManifestEdit::Remove {
                file_id: 1,
                partition_id: 1,
            })
            .unwrap();
            m.apply(ManifestEdit::SetTombstones {
                file_id: 1,
                partition_id: 1,
                tombstones: 42,
            })
            .unwrap();
            m.apply(ManifestEdit::SetLevel {
                file_id: 1,
                partition_id: 1,
                level: 9,
            })
            .unwrap();
        }
        let (m, rec) = Manifest::open(&path).unwrap();
        assert!(rec.is_clean());
        assert_eq!(rec.edits_applied, 4, "all edits must replay, none error");
        assert!(m.is_empty(), "a removed partition was resurrected");
    }

    /// A corrupt length prefix must not become a huge allocation.
    ///
    /// Four bytes off disk used as a `Vec` capacity is an OOM waiting to happen.
    #[test]
    fn absurd_length_prefix_is_capped_not_allocated() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("MANIFEST");
        {
            let (mut m, _) = Manifest::open(&path).unwrap();
            m.apply(ManifestEdit::Add(Box::new(meta(1)))).unwrap();
        }
        {
            use std::io::Write as _;
            let mut f = std::fs::OpenOptions::new()
                .append(true)
                .open(&path)
                .unwrap();
            f.write_all(&u32::MAX.to_le_bytes()).unwrap();
            f.write_all(&0u32.to_le_bytes()).unwrap();
            f.write_all(&[0u8; 16]).unwrap();
        }
        let (m, rec) = Manifest::open(&path).unwrap();
        assert_eq!(m.len(), 1, "the valid record before the garbage was lost");
        assert_eq!(rec.discarded_bytes, 24);
    }

    #[test]
    fn wrong_magic_is_rejected() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("MANIFEST");
        std::fs::write(&path, b"NOTAMANIFEST").unwrap();
        assert!(Manifest::open(&path).is_err());

        // An empty file is also not a manifest — an earlier version of this would
        // have created one over the top and silently started from scratch.
        std::fs::write(&path, b"").unwrap();
        assert!(Manifest::open(&path).is_err());
    }

    /// The manifest feeds the three planning layers that previously had no input.
    ///
    /// The point of the whole module, asserted end to end.
    #[test]
    fn manifest_drives_compaction_retention_and_verify() {
        use crate::{CompactionStrategy, RetentionPolicy, RetentionRule, verify_partitions};

        let dir = TempDir::new().unwrap();
        let path = dir.path().join("MANIFEST");
        let (mut m, _) = Manifest::open(&path).unwrap();
        for i in 0..6 {
            let mut p = meta(i);
            p.level = 0; // all L0, to trigger a merge
            m.apply(ManifestEdit::Add(Box::new(p))).unwrap();
        }

        let parts = m.partitions();
        assert_eq!(parts.len(), 6);

        let jobs = CompactionStrategy::default().plan(&parts);
        assert!(
            !jobs.is_empty(),
            "6 L0 partitions should trigger compaction"
        );

        let plan = RetentionPolicy::new()
            .with_rule(RetentionRule::MaxPartitions(2))
            .plan(&parts, 1_800_000_000_000);
        assert!(!plan.delete.is_empty(), "retention should select something");

        // Files do not exist, so verify reports them — the point is that it RUNS
        // on manifest output at all, which was impossible before.
        let report = verify_partitions(&parts).unwrap();
        assert_eq!(report.checked, 6);
    }
}
