//! WalWriter — the single authoritative write path.
//!
//! One WalWriter per WAL directory. All appends go through here.
//! Segment rotation is automatic when the current segment exceeds `max_bytes`.
//! The hash chain is maintained in-memory; the previous frame's hash is stamped
//! onto every new frame before encoding.

use crate::config::WalConfig;
use crate::error::{WalError, WalResult};
use crate::frame::{Frame, GENESIS_HASH, PayloadType};
use crate::segment::{SegmentReader, SegmentWriter, list_segment_ids, segment_path};
use parking_lot::Mutex;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::{debug, info};

// ── State shared between WalWriter and GroupCommitBatcher ─────────────────────

#[derive(Debug, Clone)]
pub struct WalTip {
    pub sequence: u64,
    pub frame_hash: [u8; 32],
    pub segment_id: u64,
}

struct WriterInner {
    config: WalConfig,
    segment: SegmentWriter,
    next_seq: u64,
    chain_hash: [u8; 32],
}

impl WriterInner {
    fn now_ns() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64
    }

    /// Rotate to a new segment file.
    fn rotate(&mut self) -> WalResult<()> {
        self.segment.flush()?;
        let new_id = self.segment.id + 1;
        info!(
            old_segment = self.segment.id,
            new_segment = new_id,
            "rotating WAL segment"
        );
        self.segment = SegmentWriter::create(
            &self.config.dir,
            new_id,
            self.next_seq,
            self.config.segment_max_bytes,
            self.config.fsync,
        )?;
        Ok(())
    }

    fn append_frame(&mut self, payload: Vec<u8>, payload_type: PayloadType) -> WalResult<WalTip> {
        if payload.len() > self.config.max_payload_bytes {
            return Err(WalError::PayloadTooLarge {
                size: payload.len(),
                max: self.config.max_payload_bytes,
            });
        }

        let mut frame = Frame {
            sequence: self.next_seq,
            timestamp_ns: Self::now_ns(),
            prev_hash: self.chain_hash,
            payload_type,
            payload,
            frame_hash: [0u8; 32],
        };
        let encoded = frame.encode();

        if self.segment.would_exceed(encoded.len()) {
            self.rotate()?;
        }

        self.segment.append_bytes(&encoded)?;
        self.chain_hash = frame.frame_hash;
        let tip = WalTip {
            sequence: self.next_seq,
            frame_hash: frame.frame_hash,
            segment_id: self.segment.id,
        };
        self.next_seq += 1;
        debug!(
            sequence = tip.sequence,
            segment = tip.segment_id,
            "frame appended"
        );
        Ok(tip)
    }
}

// ── WalWriter ─────────────────────────────────────────────────────────────────

/// The primary write interface. Cheap to clone (Arc inside).
#[derive(Clone)]
pub struct WalWriter {
    inner: Arc<Mutex<WriterInner>>,
}

impl WalWriter {
    /// Open or create a WAL at `config.dir`.
    /// Recovers the chain hash and next sequence from the last segment.
    pub fn open(config: WalConfig) -> WalResult<Self> {
        std::fs::create_dir_all(&config.dir)?;

        let ids = list_segment_ids(&config.dir)?;

        let (segment, next_seq, chain_hash) = if ids.is_empty() {
            // Fresh WAL.
            let seg =
                SegmentWriter::create(&config.dir, 0, 1, config.segment_max_bytes, config.fsync)?;
            info!(dir = %config.dir.display(), "created new WAL");
            (seg, 1u64, GENESIS_HASH)
        } else {
            // Recover from the last segment.
            let last_id = *ids.last().unwrap();
            let last_path = segment_path(&config.dir, last_id);
            let mut reader = SegmentReader::open(&last_path)?;
            let frames = reader.all_frames()?;

            let (next_seq, chain_hash) = if let Some(last) = frames.last() {
                (last.sequence + 1, last.frame_hash)
            } else {
                (reader.header.base_seq, GENESIS_HASH)
            };

            info!(last_segment = last_id, next_seq, "recovered WAL");

            // Re-open the last segment for appending.
            let file = std::fs::OpenOptions::new()
                .write(true)
                .append(true)
                .open(&last_path)?;
            let byte_offset = file.metadata()?.len();
            let seg = SegmentWriter::reopen(
                last_id,
                last_path,
                file,
                byte_offset,
                frames.len() as u64,
                config.segment_max_bytes,
                config.fsync,
            );
            (seg, next_seq, chain_hash)
        };

        Ok(Self {
            inner: Arc::new(Mutex::new(WriterInner {
                config,
                segment,
                next_seq,
                chain_hash,
            })),
        })
    }

    /// Append a data payload. Returns the WAL tip after the write.
    /// Does NOT fsync — call `flush()` or use the GroupCommitBatcher.
    pub fn append(&self, payload: Vec<u8>) -> WalResult<WalTip> {
        self.inner.lock().append_frame(payload, PayloadType::Data)
    }

    /// Flush OS write buffers + optional fsync for everything written so far.
    pub fn flush(&self) -> WalResult<()> {
        self.inner.lock().segment.flush()
    }

    /// Current chain tip (sequence + hash).
    pub fn tip(&self) -> WalTip {
        let g = self.inner.lock();
        WalTip {
            sequence: g.next_seq.saturating_sub(1),
            frame_hash: g.chain_hash,
            segment_id: g.segment.id,
        }
    }
}

// ── SegmentWriter fields need pub for recovery above ─────────────────────────

// The fields used in recovery are re-exported via the pub struct in segment.rs.
// This is fine because WalWriter owns the only path to SegmentWriter.

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::WalConfig;
    use tempfile::TempDir;

    #[test]
    fn append_and_recover() {
        let dir = TempDir::new().unwrap();
        let cfg = WalConfig::default().with_dir(dir.path()).with_fsync(false);

        let writer = WalWriter::open(cfg.clone()).unwrap();
        let tip1 = writer.append(b"first".to_vec()).unwrap();
        let tip2 = writer.append(b"second".to_vec()).unwrap();
        writer.flush().unwrap();
        assert_eq!(tip1.sequence, 1);
        assert_eq!(tip2.sequence, 2);

        // reopen and verify recovery
        drop(writer);
        let writer2 = WalWriter::open(cfg).unwrap();
        let tip = writer2.tip();
        assert_eq!(tip.sequence, 2);
    }

    #[test]
    fn segment_rotation() {
        let dir = TempDir::new().unwrap();
        // tiny segment: only holds ~2 small frames
        let cfg = WalConfig::default()
            .with_dir(dir.path())
            .with_fsync(false)
            .with_segment_max_bytes(512);

        let writer = WalWriter::open(cfg).unwrap();
        for i in 0..20u64 {
            writer
                .append(format!("payload-{}", i).into_bytes())
                .unwrap();
        }
        writer.flush().unwrap();

        let ids = list_segment_ids(dir.path()).unwrap();
        assert!(ids.len() > 1, "expected multiple segments after rotation");
    }
}
