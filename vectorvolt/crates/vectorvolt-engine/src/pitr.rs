//! PITR (point-in-time recovery) operations for [`TableEngine`] (VV-STO-017).
//!
//! [`create_snapshot`](TableEngine::create_snapshot) writes a typed
//! [`SnapshotMarker`] to the WAL and returns it.
//! [`list_snapshots`](TableEngine::list_snapshots) scans the WAL for all
//! previously written markers.
//! [`restore_from_snapshot`](TableEngine::restore_from_snapshot) opens a fresh
//! engine from the given directory — WAL replay is already handled by
//! `open_with`, so no extra work is required here. That is the *current*
//! state, not a point in time.
//!
//! [`restore_to_lsn`](TableEngine::restore_to_lsn) and its marker/timestamp
//! variants are real point-in-time restores (PP-12b): they build, in a new
//! directory, the table exactly as it stood after WAL frame `lsn`, by folding
//! WAL frames `1..=lsn` (the same fold open-time recovery uses) and writing
//! the result into a fresh table. The source table is only read.
//!
//! The WAL is the source of truth for a restore, not the snapshot's partition
//! files: those can be compacted away after the marker is written, may hold
//! rows from writes that raced the snapshot's flush, and carry no tombstones,
//! so adopting them would resurrect rows deleted before the marker. The WAL
//! is never truncated, so it holds the full history; a log whose first frame
//! is not sequence 1 is refused rather than silently restored without its
//! beginning.
//!
//! ## Authenticated markers (PP-54)
//!
//! Markers stay plaintext so `WalReader::scan_snapshot_markers` can find
//! them, but on an encrypted table each carries a `"mac"` field: a keyed
//! SHA3-256 over the marker's JSON under a key derived from the table key
//! (domain `vectorvolt-pitr-marker-mac-v1`). Readers that ignore it still
//! decode the marker. With a key, [`list_snapshots`](TableEngine::list_snapshots)
//! skips markers without a MAC (legacy, written before the table had a key)
//! and fails on a MAC that does not verify;
//! [`restore_to_marker_with`](TableEngine::restore_to_marker_with) refuses
//! either.

use std::path::Path;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use vectorvolt_crypto::KeyStore;
use vectorvolt_wal::snapshot::OP_SNAPSHOT_MARKER;
use vectorvolt_wal::{SnapshotMarker, WalConfig, WalReader};

use crate::engine::{EngineOptions, TableEngine, fold_wal, verify_wal_seal};
use crate::side_crypto::{ct_eq, derive_mac_key, from_hex, mac, to_hex};
use crate::{EngineError, EngineResult};

/// Rows written per `apply_batch` call while materialising a restore.
const RESTORE_CHUNK: usize = 4096;

const PITR_MAC_DOMAIN: &[u8] = b"vectorvolt-pitr-marker-mac-v1";
const MAC_FIELD: &str = "mac";

fn marker_mac(key: &KeyStore, marker: &SnapshotMarker) -> [u8; 32] {
    let body = serde_json::to_vec(marker).expect("SnapshotMarker serialization is infallible");
    mac(&derive_mac_key(key, PITR_MAC_DOMAIN), &body)
}

/// WAL payload for `marker`: `SnapshotMarker::encode`, plus a `"mac"` field
/// when `key` is set.
fn encode_marker(marker: &SnapshotMarker, key: Option<&KeyStore>) -> Vec<u8> {
    let Some(key) = key else {
        return marker.encode();
    };
    let mut value = serde_json::to_value(marker).expect("SnapshotMarker serialization");
    if let Some(obj) = value.as_object_mut() {
        obj.insert(
            MAC_FIELD.into(),
            serde_json::Value::String(to_hex(&marker_mac(key, marker))),
        );
    }
    let mut out = vec![OP_SNAPSHOT_MARKER];
    out.extend_from_slice(&serde_json::to_vec(&value).expect("JSON value serialization"));
    out
}

/// How a marker frame authenticated under the table key.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MarkerAuth {
    Valid,
    /// No MAC: written before the table had a key, or planted.
    Missing,
    /// A MAC that does not verify: forged or altered.
    Invalid,
}

/// Every snapshot marker in the WAL at `wal_dir`, with its authentication
/// status under `key` (`Valid` for every marker when there is no key).
fn scan_markers(
    wal_dir: &Path,
    key: Option<&KeyStore>,
) -> EngineResult<Vec<(SnapshotMarker, MarkerAuth, u64)>> {
    let cfg = WalConfig::default().with_dir(wal_dir).with_fsync(false);
    let mut out = Vec::new();
    WalReader::open(cfg).scan(vectorvolt_wal::WalCursor::beginning(), |frame| {
        let Ok(marker) = SnapshotMarker::decode(&frame.payload) else {
            return true;
        };
        let auth = match key {
            None => MarkerAuth::Valid,
            Some(key) => {
                let stored = serde_json::from_slice::<serde_json::Value>(&frame.payload[1..])
                    .ok()
                    .and_then(|v| v.get(MAC_FIELD)?.as_str().and_then(from_hex));
                match stored {
                    None => MarkerAuth::Missing,
                    Some(tag) if ct_eq(&tag, &marker_mac(key, &marker)) => MarkerAuth::Valid,
                    Some(_) => MarkerAuth::Invalid,
                }
            }
        };
        out.push((marker, auth, frame.sequence));
        true
    })?;
    Ok(out)
}

fn forged(seq: u64, wal_dir: &Path) -> EngineError {
    EngineError::WalDecrypt {
        seq,
        reason: format!(
            "PITR snapshot marker in {} fails its MAC under this key: forged or altered",
            wal_dir.display()
        ),
    }
}

impl TableEngine {
    /// Flush the write buffer, then write a [`SnapshotMarker`] to the WAL.
    ///
    /// The marker records the WAL sequence number at the moment of the snapshot
    /// and the list of `.vvsst` files that constitute the on-disk state.
    pub fn create_snapshot(&self) -> EngineResult<SnapshotMarker> {
        // The position comes first: every frame at or below it is fully
        // applied in memory at this instant, so the flush that follows writes
        // all of their effects out. Taking it after the flush let a write
        // that raced the flush be covered by the marker without being in any
        // listed file.
        //
        // The marker goes through this engine's own WAL writer. Opening a
        // second writer on the live directory forked the sequence numbers and
        // the hash chain (PP-12).
        let lsn = self.txn.quiesced_tip();

        // A failed flush must not produce a marker: it would describe state
        // that never reached disk (B4). An empty buffer is `Ok(None)`.
        self.flush()?;

        let tier_files: Vec<String> = std::fs::read_dir(&self.partition_dir)
            .map(|rd| {
                rd.filter_map(|e| e.ok())
                    .filter(|e| e.path().extension().and_then(|x| x.to_str()) == Some("vvsst"))
                    .filter_map(|e| e.file_name().to_str().map(|s| s.to_owned()))
                    .collect()
            })
            .unwrap_or_default();

        let timestamp_ns = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;

        let marker = SnapshotMarker {
            snapshot_id: timestamp_ns,
            lsn,
            timestamp_ns,
            table_name: "default".to_string(),
            tier_files,
        };

        let key = self.txn.wal_key().map(|k| k.as_ref());
        self.txn.append_marker(encode_marker(&marker, key))?;
        Ok(marker)
    }

    /// Return all snapshot markers recorded in this table's WAL.
    ///
    /// On an encrypted table only authenticated markers are returned; one
    /// whose MAC fails is an error, one without a MAC (written before the
    /// table had a key) is skipped.
    pub fn list_snapshots(&self) -> EngineResult<Vec<SnapshotMarker>> {
        let wal_dir = self
            .partition_dir
            .parent()
            .unwrap_or(&self.partition_dir)
            .join("wal");
        let key = self.txn.wal_key().map(|k| k.as_ref());
        let mut out = Vec::new();
        for (marker, auth, seq) in scan_markers(&wal_dir, key)? {
            match auth {
                MarkerAuth::Valid => out.push(marker),
                MarkerAuth::Missing => tracing::warn!(
                    seq,
                    snapshot_id = marker.snapshot_id,
                    "skipping unauthenticated PITR marker on an encrypted table"
                ),
                MarkerAuth::Invalid => return Err(forged(seq, &wal_dir)),
            }
        }
        Ok(out)
    }

    /// Open a new [`TableEngine`] from `base_dir`, logically restoring from
    /// the snapshot with `snapshot_id`.
    ///
    /// WAL replay is handled automatically by [`TableEngine::open`]; this
    /// function validates that the snapshot exists before opening, ensuring
    /// callers get a clear error for unknown IDs.
    pub fn restore_from_snapshot(
        schema: Vec<String>,
        base_dir: &Path,
        snapshot_id: u64,
    ) -> EngineResult<Self> {
        let wal_dir = base_dir.join("wal");
        let cfg = WalConfig::default().with_dir(&wal_dir).with_fsync(false);
        let reader = WalReader::open(cfg);
        let markers = reader.scan_snapshot_markers()?;

        if markers.iter().any(|m| m.snapshot_id == snapshot_id) {
            Self::open(schema, base_dir, 1_000_000)
        } else if snapshot_id == 0 {
            // Convenience: snapshot_id 0 means "open from scratch".
            Self::open(schema, base_dir, 1_000_000)
        } else {
            Err(crate::EngineError::Other(format!(
                "snapshot {snapshot_id} not found in WAL"
            )))
        }
    }
    /// Restore the table at `src_dir` into a new table at `dst_dir`, exactly
    /// as it stood after WAL frame `lsn` (inclusive).
    ///
    /// `dst_dir` must not exist or must be empty; `src_dir` is never written.
    /// `lsn = 0`, or any sequence before the first write, gives an empty
    /// table. A sequence past the end of the source WAL is an error.
    pub fn restore_to_lsn(
        schema: Vec<String>,
        src_dir: &Path,
        dst_dir: &Path,
        lsn: u64,
    ) -> EngineResult<Self> {
        Self::restore_to_lsn_with(schema, src_dir, dst_dir, lsn, None)
    }

    /// [`restore_to_lsn`](Self::restore_to_lsn) for an encrypted table:
    /// `key` decrypts the source WAL and encrypts the restored table.
    pub fn restore_to_lsn_with(
        schema: Vec<String>,
        src_dir: &Path,
        dst_dir: &Path,
        lsn: u64,
        key: Option<Arc<vectorvolt_crypto::KeyStore>>,
    ) -> EngineResult<Self> {
        let src_wal = src_dir.join("wal");
        if !src_wal.is_dir() {
            return Err(EngineError::Other(format!(
                "restore: no WAL at {}",
                src_wal.display()
            )));
        }
        if let (Ok(a), Ok(b)) = (src_dir.canonicalize(), dst_dir.canonicalize())
            && a == b
        {
            return Err(EngineError::Other(
                "restore: destination is the source table; restores never run in place".into(),
            ));
        }
        if dst_dir.exists() && std::fs::read_dir(dst_dir)?.next().is_some() {
            return Err(EngineError::Other(format!(
                "restore: destination {} is not empty",
                dst_dir.display()
            )));
        }

        verify_wal_seal(&src_wal, key.as_ref())?;
        let fold = fold_wal(&src_wal, 0, lsn, key.as_ref())?;
        if let Some(first) = fold.first_seq
            && first > 1
        {
            return Err(EngineError::Other(format!(
                "restore: WAL history before sequence {first} is missing; \
                 cannot reconstruct a point in time"
            )));
        }
        if lsn > fold.last_seq {
            let tip = fold_wal(&src_wal, u64::MAX, u64::MAX, key.as_ref())?.last_seq;
            if lsn > tip {
                return Err(EngineError::Other(format!(
                    "restore: sequence {lsn} is past the end of the WAL (tip {tip})"
                )));
            }
        }

        let options = EngineOptions::new(1_000_000);
        let engine = match key {
            Some(k) => Self::open_with_encryption(schema, dst_dir, options, k)?,
            None => Self::open_with(schema, dst_dir, options)?,
        };
        let width = engine.schema.col_count();
        let mut rows: Vec<(u64, Vec<u64>)> = fold
            .state
            .into_iter()
            .filter_map(|(pk, v)| v.map(|v| (pk, v)))
            // A row logged before an ALTER changed the column count cannot be
            // represented in this schema; open-time recovery skips it too.
            .filter(|(_, v)| v.len() == width)
            .collect();
        rows.sort_unstable_by_key(|(pk, _)| *pk);
        for chunk in rows.chunks(RESTORE_CHUNK) {
            engine.apply_batch(chunk.iter().map(|(pk, v)| (*pk, Some(v.clone()))).collect())?;
        }
        engine.flush()?;
        Ok(engine)
    }

    /// Restore to the WAL position recorded by snapshot marker `snapshot_id`
    /// (see [`create_snapshot`](Self::create_snapshot)).
    pub fn restore_to_marker(
        schema: Vec<String>,
        src_dir: &Path,
        dst_dir: &Path,
        snapshot_id: u64,
    ) -> EngineResult<Self> {
        let cfg = WalConfig::default()
            .with_dir(src_dir.join("wal"))
            .with_fsync(false);
        let marker = WalReader::open(cfg)
            .scan_snapshot_markers()?
            .into_iter()
            .find(|m| m.snapshot_id == snapshot_id)
            .ok_or_else(|| {
                EngineError::Other(format!("snapshot {snapshot_id} not found in WAL"))
            })?;
        Self::restore_to_lsn(schema, src_dir, dst_dir, marker.lsn)
    }

    /// [`restore_to_marker`](Self::restore_to_marker) for an encrypted table.
    /// The marker must carry a MAC that verifies under `key`: an
    /// unauthenticated or forged marker could point the restore anywhere.
    pub fn restore_to_marker_with(
        schema: Vec<String>,
        src_dir: &Path,
        dst_dir: &Path,
        snapshot_id: u64,
        key: Arc<KeyStore>,
    ) -> EngineResult<Self> {
        let wal_dir = src_dir.join("wal");
        let (marker, auth, seq) = scan_markers(&wal_dir, Some(&key))?
            .into_iter()
            .find(|(m, _, _)| m.snapshot_id == snapshot_id)
            .ok_or_else(|| {
                EngineError::Other(format!("snapshot {snapshot_id} not found in WAL"))
            })?;
        match auth {
            MarkerAuth::Valid => {}
            MarkerAuth::Missing => {
                return Err(EngineError::WalDecrypt {
                    seq,
                    reason: format!(
                        "PITR snapshot marker {snapshot_id} in {} has no MAC; refusing to \
                         restore an encrypted table to an unauthenticated position",
                        wal_dir.display()
                    ),
                });
            }
            MarkerAuth::Invalid => return Err(forged(seq, &wal_dir)),
        }
        Self::restore_to_lsn_with(schema, src_dir, dst_dir, marker.lsn, Some(key))
    }

    /// Restore to the last WAL frame written at or before `timestamp_ns`
    /// (nanoseconds since the Unix epoch, as stamped on each frame). A time
    /// before the first frame gives an empty table.
    pub fn restore_to_timestamp(
        schema: Vec<String>,
        src_dir: &Path,
        dst_dir: &Path,
        timestamp_ns: u64,
    ) -> EngineResult<Self> {
        let cfg = WalConfig::default()
            .with_dir(src_dir.join("wal"))
            .with_fsync(false);
        let mut lsn = 0;
        WalReader::open(cfg).scan(vectorvolt_wal::WalCursor::beginning(), |frame| {
            if frame.timestamp_ns > timestamp_ns {
                return false;
            }
            lsn = frame.sequence;
            true
        })?;
        Self::restore_to_lsn(schema, src_dir, dst_dir, lsn)
    }
}
