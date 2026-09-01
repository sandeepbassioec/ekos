//! PITR snapshot markers for VectorVolt (VV-WAL-013).
//!
//! A [`SnapshotMarker`] is written to the WAL to record the LSN and set of
//! tier files that constitute a consistent point-in-time snapshot. On restore
//! the engine replays only WAL frames that follow the snapshot's LSN.

use crate::{
    WalCursor,
    error::{WalError, WalResult},
    frame::Frame,
    reader::WalReader,
    writer::{WalTip, WalWriter},
};

pub const OP_SNAPSHOT_MARKER: u8 = 0x20;

/// A typed snapshot marker written as a WAL frame payload.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SnapshotMarker {
    pub snapshot_id: u64,
    pub lsn: u64,
    pub timestamp_ns: u64,
    pub table_name: String,
    pub tier_files: Vec<String>,
}

impl SnapshotMarker {
    /// Encode into a WAL payload: one-byte opcode + JSON body.
    pub fn encode(&self) -> Vec<u8> {
        let mut bytes = vec![OP_SNAPSHOT_MARKER];
        let json = serde_json::to_vec(self).expect("SnapshotMarker serialization is infallible");
        bytes.extend_from_slice(&json);
        bytes
    }

    /// Decode from a WAL payload. Fails if the opcode byte is absent or wrong.
    pub fn decode(bytes: &[u8]) -> WalResult<Self> {
        if bytes.is_empty() || bytes[0] != OP_SNAPSHOT_MARKER {
            return Err(WalError::SnapshotDecodeError(
                "missing or incorrect snapshot opcode byte".into(),
            ));
        }
        serde_json::from_slice(&bytes[1..])
            .map_err(|e| WalError::SnapshotDecodeError(e.to_string()))
    }
}

// ── WalWriter extension ───────────────────────────────────────────────────────

impl WalWriter {
    /// Write a snapshot marker frame to the WAL.
    pub fn write_snapshot_marker(&self, marker: &SnapshotMarker) -> WalResult<WalTip> {
        self.append(marker.encode())
    }
}

// ── WalReader extension ───────────────────────────────────────────────────────

impl WalReader {
    /// Collect all snapshot markers in the WAL in order.
    pub fn scan_snapshot_markers(&self) -> WalResult<Vec<SnapshotMarker>> {
        let mut markers = Vec::new();
        self.scan(WalCursor::beginning(), |frame: Frame| {
            if let Ok(m) = SnapshotMarker::decode(&frame.payload) {
                markers.push(m);
            }
            true
        })?;
        Ok(markers)
    }

    /// Return the most recent snapshot marker, if any.
    pub fn latest_snapshot(&self) -> WalResult<Option<SnapshotMarker>> {
        let mut markers = self.scan_snapshot_markers()?;
        Ok(markers.pop())
    }

    /// Return all Data frames whose sequence is strictly greater than the LSN
    /// recorded in the snapshot identified by `snapshot_id`.
    pub fn replay_from_snapshot(&self, snapshot_id: u64) -> WalResult<Vec<Frame>> {
        // Find the snapshot marker with the matching id.
        let markers = self.scan_snapshot_markers()?;
        let marker = markers
            .iter()
            .find(|m| m.snapshot_id == snapshot_id)
            .ok_or_else(|| {
                WalError::SnapshotDecodeError(format!("snapshot {snapshot_id} not found"))
            })?;

        let after_lsn = marker.lsn;

        let mut frames = Vec::new();
        self.scan(WalCursor::beginning(), |frame: Frame| {
            if frame.sequence > after_lsn {
                // Include only Data frames (skip snapshot markers themselves).
                if frame.payload.first().copied() != Some(OP_SNAPSHOT_MARKER) {
                    frames.push(frame);
                }
            }
            true
        })?;
        Ok(frames)
    }
}

// ── tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{WalConfig, WalReader, WalWriter};
    use tempfile::TempDir;

    fn writer(dir: &TempDir) -> WalWriter {
        let cfg = WalConfig::default().with_dir(dir.path()).with_fsync(false);
        WalWriter::open(cfg).unwrap()
    }

    fn reader(dir: &TempDir) -> WalReader {
        let cfg = WalConfig::default().with_dir(dir.path()).with_fsync(false);
        WalReader::open(cfg)
    }

    fn sample_marker(id: u64, lsn: u64) -> SnapshotMarker {
        SnapshotMarker {
            snapshot_id: id,
            lsn,
            timestamp_ns: 123456789,
            table_name: "test_table".into(),
            tier_files: vec!["00000_l0.vvsst".into()],
        }
    }

    #[test]
    fn encode_decode_roundtrip() {
        let m = sample_marker(1, 42);
        let encoded = m.encode();
        let decoded = SnapshotMarker::decode(&encoded).unwrap();
        assert_eq!(decoded.snapshot_id, 1);
        assert_eq!(decoded.lsn, 42);
        assert_eq!(decoded.table_name, "test_table");
    }

    #[test]
    fn decode_empty_bytes_errors() {
        assert!(SnapshotMarker::decode(&[]).is_err());
    }

    #[test]
    fn decode_wrong_opcode_errors() {
        let bytes = [0x00, b'{', b'}'];
        assert!(SnapshotMarker::decode(&bytes).is_err());
    }

    #[test]
    fn decode_invalid_json_errors() {
        let bytes = [OP_SNAPSHOT_MARKER, 0xFF, 0xFE];
        assert!(SnapshotMarker::decode(&bytes).is_err());
    }

    #[test]
    fn write_snapshot_marker_appends_to_wal() {
        let dir = TempDir::new().unwrap();
        let w = writer(&dir);
        let marker = sample_marker(1, 10);
        let tip = w.write_snapshot_marker(&marker).unwrap();
        w.flush().unwrap();
        assert!(tip.sequence >= 1);
    }

    #[test]
    fn scan_snapshot_markers_empty_wal() {
        let dir = TempDir::new().unwrap();
        let _w = writer(&dir);
        let r = reader(&dir);
        let markers = r.scan_snapshot_markers().unwrap();
        assert!(markers.is_empty());
    }

    #[test]
    fn scan_finds_written_marker() {
        let dir = TempDir::new().unwrap();
        let w = writer(&dir);
        w.write_snapshot_marker(&sample_marker(1, 1)).unwrap();
        w.flush().unwrap();
        drop(w);
        let markers = reader(&dir).scan_snapshot_markers().unwrap();
        assert_eq!(markers.len(), 1);
        assert_eq!(markers[0].snapshot_id, 1);
    }

    #[test]
    fn latest_snapshot_returns_last() {
        let dir = TempDir::new().unwrap();
        let w = writer(&dir);
        w.write_snapshot_marker(&sample_marker(1, 1)).unwrap();
        w.write_snapshot_marker(&sample_marker(2, 5)).unwrap();
        w.flush().unwrap();
        drop(w);
        let latest = reader(&dir).latest_snapshot().unwrap().unwrap();
        assert_eq!(latest.snapshot_id, 2);
    }

    #[test]
    fn latest_snapshot_none_on_empty_wal() {
        let dir = TempDir::new().unwrap();
        let _w = writer(&dir);
        let result = reader(&dir).latest_snapshot().unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn replay_from_snapshot_returns_frames_after_lsn() {
        let dir = TempDir::new().unwrap();
        let w = writer(&dir);
        // Write some data frames before snapshot
        w.append(b"before".to_vec()).unwrap();
        w.append(b"before2".to_vec()).unwrap();
        let tip = w.write_snapshot_marker(&sample_marker(99, 2)).unwrap();
        // Write data frames after snapshot
        w.append(b"after1".to_vec()).unwrap();
        w.append(b"after2".to_vec()).unwrap();
        w.flush().unwrap();
        drop(w);

        let frames = reader(&dir).replay_from_snapshot(99).unwrap();
        assert_eq!(frames.len(), 2);
        assert_eq!(frames[0].payload, b"after1");
        assert_eq!(frames[1].payload, b"after2");
        let _ = tip;
    }

    #[test]
    fn replay_missing_snapshot_id_errors() {
        let dir = TempDir::new().unwrap();
        let w = writer(&dir);
        w.write_snapshot_marker(&sample_marker(1, 1)).unwrap();
        w.flush().unwrap();
        drop(w);
        let result = reader(&dir).replay_from_snapshot(999);
        assert!(result.is_err());
    }

    #[test]
    fn tier_files_preserved_roundtrip() {
        let m = SnapshotMarker {
            snapshot_id: 7,
            lsn: 100,
            timestamp_ns: 0,
            table_name: "t".into(),
            tier_files: vec!["a.vvsst".into(), "b.vvsst".into()],
        };
        let decoded = SnapshotMarker::decode(&m.encode()).unwrap();
        assert_eq!(decoded.tier_files, vec!["a.vvsst", "b.vvsst"]);
    }
}
