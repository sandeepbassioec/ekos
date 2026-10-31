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
use tracing::{debug, info, warn};

// ── State shared between WalWriter and GroupCommitBatcher ─────────────────────

#[derive(Debug, Clone, Default)]
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
            // A crash mid-append leaves a partial final frame. Opening must
            // survive that, so the torn bytes are cut back to the last intact
            // frame; anything else that fails to decode is real corruption and
            // still refuses to open (see `SegmentReader::intact_prefix`).
            let (frames, intact_end, torn) = reader.intact_prefix()?;
            drop(reader);
            if torn {
                let f = std::fs::OpenOptions::new().write(true).open(&last_path)?;
                let old_len = f.metadata()?.len();
                f.set_len(intact_end)?;
                f.sync_all()?;
                warn!(
                    segment = last_id,
                    discarded_bytes = old_len - intact_end,
                    "truncated torn WAL tail"
                );
            }
            let reader = SegmentReader::open(&last_path)?;

            let (next_seq, chain_hash) = if let Some(last) = frames.last() {
                (last.sequence + 1, last.frame_hash)
            } else {
                // An empty last segment (crash right after rotation) must still
                // chain from the previous segment, or `verify` reports a break
                // at the first frame written after reopening.
                (
                    reader.header.base_seq,
                    previous_segment_hash(&config.dir, &ids)?,
                )
            };

            info!(last_segment = last_id, next_seq, "recovered WAL");

            // Re-open the last segment for appending.
            let file = std::fs::OpenOptions::new().append(true).open(&last_path)?;
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

/// Hash of the last frame in the newest non-empty segment before the last one,
/// or `GENESIS_HASH` when there is none.
fn previous_segment_hash(dir: &std::path::Path, ids: &[u64]) -> WalResult<[u8; 32]> {
    for &id in ids.iter().rev().skip(1) {
        let mut reader = SegmentReader::open(&segment_path(dir, id))?;
        if let Some(last) = reader.all_frames()?.last() {
            return Ok(last.frame_hash);
        }
    }
    Ok(GENESIS_HASH)
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

    fn last_segment(dir: &std::path::Path) -> std::path::PathBuf {
        let ids = list_segment_ids(dir).unwrap();
        segment_path(dir, *ids.last().unwrap())
    }

    #[test]
    fn torn_tail_is_truncated_on_open() {
        let dir = TempDir::new().unwrap();
        let cfg = WalConfig::default().with_dir(dir.path()).with_fsync(false);
        let writer = WalWriter::open(cfg.clone()).unwrap();
        writer.append(b"one".to_vec()).unwrap();
        writer.append(b"two".to_vec()).unwrap();
        writer.flush().unwrap();
        drop(writer);

        // Simulate a crash mid-append: half of a third frame reaches disk.
        let path = last_segment(dir.path());
        let mut frame = Frame {
            sequence: 3,
            timestamp_ns: 0,
            prev_hash: GENESIS_HASH,
            payload_type: PayloadType::Data,
            payload: b"three-is-torn".to_vec(),
            frame_hash: [0u8; 32],
        };
        let bytes = frame.encode();
        let mut f = std::fs::OpenOptions::new()
            .append(true)
            .open(&path)
            .unwrap();
        std::io::Write::write_all(&mut f, &bytes[..bytes.len() / 2]).unwrap();
        drop(f);

        let writer = WalWriter::open(cfg.clone()).unwrap();
        assert_eq!(writer.tip().sequence, 2);
        let tip = writer.append(b"three".to_vec()).unwrap();
        assert_eq!(tip.sequence, 3);
        writer.flush().unwrap();
        drop(writer);
        assert_eq!(crate::WalReader::open(cfg).verify().unwrap(), 3);
    }

    #[test]
    fn zero_filled_tail_is_truncated_on_open() {
        let dir = TempDir::new().unwrap();
        let cfg = WalConfig::default().with_dir(dir.path()).with_fsync(false);
        let writer = WalWriter::open(cfg.clone()).unwrap();
        writer.append(b"one".to_vec()).unwrap();
        writer.flush().unwrap();
        drop(writer);

        let path = last_segment(dir.path());
        let mut f = std::fs::OpenOptions::new()
            .append(true)
            .open(&path)
            .unwrap();
        std::io::Write::write_all(&mut f, &[0u8; 300]).unwrap();
        drop(f);

        let writer = WalWriter::open(cfg).unwrap();
        assert_eq!(writer.tip().sequence, 1);
    }

    #[test]
    fn corruption_before_the_tail_still_refuses_to_open() {
        let dir = TempDir::new().unwrap();
        let cfg = WalConfig::default().with_dir(dir.path()).with_fsync(false);
        let writer = WalWriter::open(cfg.clone()).unwrap();
        writer.append(b"aaaaaaaaaaaaaaaa".to_vec()).unwrap();
        writer.append(b"bbbbbbbbbbbbbbbb".to_vec()).unwrap();
        writer.flush().unwrap();
        drop(writer);

        // Flip a payload byte in the first frame: an intact-length frame with a
        // bad checksum, followed by a valid one. That is not a torn write.
        let path = last_segment(dir.path());
        let mut bytes = std::fs::read(&path).unwrap();
        let pos = crate::segment::SEG_HEADER_SIZE as usize + 70;
        bytes[pos] ^= 0xFF;
        std::fs::write(&path, &bytes).unwrap();

        assert!(WalWriter::open(cfg).is_err());
    }

    #[test]
    fn empty_last_segment_chains_from_previous_segment() {
        let dir = TempDir::new().unwrap();
        let cfg = WalConfig::default()
            .with_dir(dir.path())
            .with_fsync(false)
            .with_segment_max_bytes(512);
        let writer = WalWriter::open(cfg.clone()).unwrap();
        for i in 0..6u64 {
            writer.append(format!("payload-{i}").into_bytes()).unwrap();
        }
        writer.flush().unwrap();
        let next = writer.tip().sequence + 1;
        drop(writer);

        // Crash right after rotation: a new segment with a header and no frames.
        let ids = list_segment_ids(dir.path()).unwrap();
        let mut seg = crate::segment::SegmentWriter::create(
            dir.path(),
            ids.last().unwrap() + 1,
            next,
            512,
            false,
        )
        .unwrap();
        seg.flush().unwrap();
        drop(seg);

        let writer = WalWriter::open(cfg.clone()).unwrap();
        writer.append(b"after".to_vec()).unwrap();
        writer.flush().unwrap();
        drop(writer);
        assert_eq!(crate::WalReader::open(cfg).verify().unwrap(), next);
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
