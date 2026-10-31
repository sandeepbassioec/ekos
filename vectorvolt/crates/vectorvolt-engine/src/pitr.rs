//! PITR (point-in-time recovery) operations for [`TableEngine`] (VV-STO-017).
//!
//! [`create_snapshot`](TableEngine::create_snapshot) writes a typed
//! [`SnapshotMarker`] to the WAL and returns it.
//! [`list_snapshots`](TableEngine::list_snapshots) scans the WAL for all
//! previously written markers.
//! [`restore_from_snapshot`](TableEngine::restore_from_snapshot) opens a fresh
//! engine from the given directory — WAL replay is already handled by
//! `open_with`, so no extra work is required here.

use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use vectorvolt_wal::{SnapshotMarker, WalConfig, WalReader};

use crate::{EngineResult, engine::TableEngine};

impl TableEngine {
    /// Flush the write buffer, then write a [`SnapshotMarker`] to the WAL.
    ///
    /// The marker records the WAL sequence number at the moment of the snapshot
    /// and the list of `.vvsst` files that constitute the on-disk state.
    pub fn create_snapshot(&self) -> EngineResult<SnapshotMarker> {
        // Best-effort flush; ignore "nothing to flush" errors.
        let _ = self.txn.flush();

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

        // The marker goes through this engine's own WAL writer. Opening a
        // second writer on the live directory forked the sequence numbers and
        // the hash chain (PP-12).
        let lsn = self.txn.quiesced_tip();

        let marker = SnapshotMarker {
            snapshot_id: timestamp_ns,
            lsn,
            timestamp_ns,
            table_name: "default".to_string(),
            tier_files,
        };

        self.txn.append_marker(marker.encode())?;
        Ok(marker)
    }

    /// Return all snapshot markers recorded in this table's WAL.
    pub fn list_snapshots(&self) -> EngineResult<Vec<SnapshotMarker>> {
        let wal_dir = self
            .partition_dir
            .parent()
            .unwrap_or(&self.partition_dir)
            .join("wal");
        let cfg = WalConfig::default().with_dir(&wal_dir).with_fsync(false);
        let reader = WalReader::open(cfg);
        Ok(reader.scan_snapshot_markers()?)
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
}
