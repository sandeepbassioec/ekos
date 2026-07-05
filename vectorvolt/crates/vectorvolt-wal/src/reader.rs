//! WalReader — forward scan and random-access by sequence number.
//!
//! A `WalCursor` tracks position across segment boundaries.
//! `WalReader::verify()` walks the entire WAL and validates the hash chain.

use crate::config::WalConfig;
use crate::error::{WalError, WalResult};
use crate::frame::{Frame, GENESIS_HASH};
use crate::segment::{list_segment_ids, segment_path, SegmentReader, SEG_HEADER_SIZE};
use tracing::debug;

// ── WalCursor ─────────────────────────────────────────────────────────────────

/// Opaque cursor: segment_id + byte_offset within that segment.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WalCursor {
    pub segment_id: u64,
    pub offset:     u64,
}

impl WalCursor {
    pub fn beginning() -> Self {
        Self { segment_id: 0, offset: SEG_HEADER_SIZE }
    }
}

// ── WalReader ─────────────────────────────────────────────────────────────────

pub struct WalReader {
    config: WalConfig,
}

impl WalReader {
    pub fn open(config: WalConfig) -> Self {
        Self { config }
    }

    fn segment_ids(&self) -> WalResult<Vec<u64>> {
        list_segment_ids(&self.config.dir)
    }

    /// Read all frames in order starting from `cursor`, calling `f` for each.
    /// Stops when `f` returns `false` or the WAL is exhausted.
    pub fn scan<F>(&self, cursor: WalCursor, mut f: F) -> WalResult<WalCursor>
    where
        F: FnMut(Frame) -> bool,
    {
        let ids = self.segment_ids()?;
        let start_idx = ids.partition_point(|&id| id < cursor.segment_id);

        let mut cur = cursor;

        for &seg_id in &ids[start_idx..] {
            let path = segment_path(&self.config.dir, seg_id);
            let mut reader = SegmentReader::open(&path)?;

            if seg_id == cursor.segment_id {
                reader.seek_to(cursor.offset)?;
            } else {
                reader.rewind()?;
            }

            loop {
                let pos = reader.file_position()?;
                match reader.next_frame()? {
                    None => break,
                    Some(frame) => {
                        cur = WalCursor { segment_id: seg_id, offset: pos };
                        let keep_going = f(frame);
                        if !keep_going {
                            return Ok(cur);
                        }
                    }
                }
            }
            cur = WalCursor { segment_id: seg_id + 1, offset: SEG_HEADER_SIZE };
        }

        Ok(cur)
    }

    /// Scan and collect all frames into a Vec.
    pub fn read_all(&self) -> WalResult<Vec<Frame>> {
        let mut frames = Vec::new();
        self.scan(WalCursor::beginning(), |f| { frames.push(f); true })?;
        Ok(frames)
    }

    /// Read frames from sequence `from` to `to` (inclusive).
    pub fn read_range(&self, from: u64, to: u64) -> WalResult<Vec<Frame>> {
        let mut frames = Vec::new();
        self.scan(WalCursor::beginning(), |f| {
            let seq = f.sequence;
            if seq >= from && seq <= to {
                frames.push(f);
            }
            seq < to
        })?;
        Ok(frames)
    }

    /// Verify the entire hash chain from genesis. Returns the number of frames verified.
    pub fn verify(&self) -> WalResult<u64> {
        let mut prev_hash = GENESIS_HASH;
        let mut count = 0u64;

        self.scan(WalCursor::beginning(), |frame| {
            if frame.prev_hash != prev_hash {
                return false; // will surface as early return, caller checks count
            }
            let computed = frame.compute_hash();
            if computed != frame.frame_hash {
                return false;
            }
            prev_hash = frame.frame_hash;
            count += 1;
            true
        })?;

        // Re-walk to detect chain break (scan stops at false return).
        let total_frames = self.read_all()?.len() as u64;
        if count != total_frames {
            // find the break
            let mut prev = GENESIS_HASH;
            let mut broken_at = 0u64;
            for frame in self.read_all()? {
                if frame.prev_hash != prev {
                    broken_at = frame.sequence;
                    break;
                }
                prev = frame.frame_hash;
            }
            return Err(WalError::ChainBroken {
                sequence: broken_at,
                expected: hex(&prev),
                actual:   hex(&GENESIS_HASH), // placeholder; real value in the frame
            });
        }

        debug!(frames_verified = count, "hash chain verified OK");
        Ok(count)
    }

    /// WAL statistics.
    pub fn stats(&self) -> WalResult<WalStats> {
        let ids = self.segment_ids()?;
        let mut total_frames = 0u64;
        let mut total_bytes  = 0u64;
        let mut first_seq    = None;
        let mut last_seq     = None;
        let mut last_hash    = GENESIS_HASH;

        for &id in &ids {
            let path = segment_path(&self.config.dir, id);
            let mut r = SegmentReader::open(&path)?;
            let frames = r.all_frames()?;
            total_frames += frames.len() as u64;
            total_bytes  += path.metadata()?.len();
            for f in &frames {
                if first_seq.is_none() { first_seq = Some(f.sequence); }
                last_seq  = Some(f.sequence);
                last_hash = f.frame_hash;
            }
        }

        Ok(WalStats {
            segments:       ids.len() as u64,
            total_frames,
            total_bytes,
            first_sequence: first_seq.unwrap_or(0),
            last_sequence:  last_seq.unwrap_or(0),
            chain_tip:      last_hash,
        })
    }
}

#[derive(Debug)]
pub struct WalStats {
    pub segments:       u64,
    pub total_frames:   u64,
    pub total_bytes:    u64,
    pub first_sequence: u64,
    pub last_sequence:  u64,
    pub chain_tip:      [u8; 32],
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

// ── expose file_position on SegmentReader ─────────────────────────────────────

impl SegmentReader {
    pub fn file_position(&mut self) -> WalResult<u64> {
        use std::io::Seek;
        Ok(self.file.stream_position()?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::WalConfig;
    use crate::writer::WalWriter;
    use tempfile::TempDir;

    fn setup(dir: &std::path::Path) -> WalWriter {
        let cfg = WalConfig::default().with_dir(dir).with_fsync(false);
        WalWriter::open(cfg).unwrap()
    }

    #[test]
    fn read_all_frames() {
        let dir = TempDir::new().unwrap();
        let writer = setup(dir.path());
        for i in 0..10u32 {
            writer.append(i.to_le_bytes().to_vec()).unwrap();
        }
        writer.flush().unwrap();

        let reader = WalReader::open(WalConfig::default().with_dir(dir.path()).with_fsync(false));
        let frames = reader.read_all().unwrap();
        assert_eq!(frames.len(), 10);
        assert_eq!(frames[0].sequence, 1);
        assert_eq!(frames[9].sequence, 10);
    }

    #[test]
    fn verify_chain() {
        let dir = TempDir::new().unwrap();
        let writer = setup(dir.path());
        for i in 0..5u32 {
            writer.append(i.to_le_bytes().to_vec()).unwrap();
        }
        writer.flush().unwrap();

        let reader = WalReader::open(WalConfig::default().with_dir(dir.path()).with_fsync(false));
        let verified = reader.verify().unwrap();
        assert_eq!(verified, 5);
    }
}
