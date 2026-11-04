//! WalReader — forward scan and random-access by sequence number.
//!
//! A `WalCursor` tracks position across segment boundaries.
//! `WalReader::verify()` walks the entire WAL and validates the hash chain.

use crate::config::WalConfig;
use crate::error::{WalError, WalResult};
use crate::frame::{Frame, GENESIS_HASH};
use crate::segment::{
    SEG_HEADER_SIZE, SegmentHeader, SegmentReader, list_segment_ids, segment_path,
};
use tracing::debug;

// ── WalCursor ─────────────────────────────────────────────────────────────────

/// Opaque cursor: segment_id + byte_offset within that segment.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WalCursor {
    pub segment_id: u64,
    pub offset: u64,
}

impl WalCursor {
    pub fn beginning() -> Self {
        Self {
            segment_id: 0,
            offset: SEG_HEADER_SIZE,
        }
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
        self.scan_with_header(cursor, |frame, _| f(frame))
    }

    /// The header of every segment, in segment order.
    pub fn segment_headers(&self) -> WalResult<Vec<SegmentHeader>> {
        self.segment_ids()?
            .into_iter()
            .map(|id| Ok(SegmentReader::open(&segment_path(&self.config.dir, id))?.header))
            .collect()
    }

    /// Like [`scan`](Self::scan), also passing the header of the segment each
    /// frame was read from (so a caller can tell whether that segment is
    /// encrypted, and under which key id).
    pub fn scan_with_header<F>(&self, cursor: WalCursor, mut f: F) -> WalResult<WalCursor>
    where
        F: FnMut(Frame, &SegmentHeader) -> bool,
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
                        cur = WalCursor {
                            segment_id: seg_id,
                            offset: pos,
                        };
                        let keep_going = f(frame, &reader.header);
                        if !keep_going {
                            return Ok(cur);
                        }
                    }
                }
            }
            cur = WalCursor {
                segment_id: seg_id + 1,
                offset: SEG_HEADER_SIZE,
            };
        }

        Ok(cur)
    }

    /// Scan and collect all frames into a Vec.
    pub fn read_all(&self) -> WalResult<Vec<Frame>> {
        let mut frames = Vec::new();
        self.scan(WalCursor::beginning(), |f| {
            frames.push(f);
            true
        })?;
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

    /// Verify the hash chain from genesis. Returns the number of frames verified.
    ///
    /// ## What this checks that `Frame::decode` does not
    ///
    /// [`Frame::decode`](crate::Frame::decode) already validates each frame *in
    /// isolation*: its CRC32, and that its stored `frame_hash` matches a hash
    /// recomputed over its own bytes. A frame failing either never reaches this
    /// function — [`scan`](Self::scan) propagates the error instead.
    ///
    /// What `decode` cannot know is whether frame N's `prev_hash` equals frame
    /// N−1's `frame_hash`, because that is a *relationship between* frames rather
    /// than a property of one. Checking that link is this function's entire job,
    /// and it is what makes the log tamper-**evident** rather than merely
    /// checksummed: an attacker can rewrite one frame and recompute both its CRC
    /// and its self-hash (each computable from the frame alone), but the rewrite
    /// changes that frame's `frame_hash`, so the chain visibly breaks at the *next*
    /// frame — and cannot be repaired without rewriting every frame after it.
    ///
    /// ## Errors
    ///
    /// - [`WalError::ChainBroken`] — a frame does not link to its predecessor.
    ///   Reported at the frame that fails to link, so the tampering is at or
    ///   before `sequence - 1`.
    /// - [`WalError::Crc32Mismatch`] / [`WalError::CorruptFrame`] — surfaced from
    ///   `decode` for accidental corruption, before chain checking begins.
    pub fn verify(&self) -> WalResult<u64> {
        let mut prev_hash = GENESIS_HASH;
        let mut count = 0u64;
        // Captured inside the closure so the failure detail is recorded at the
        // moment of detection. The previous implementation returned `false` and
        // then re-walked the log twice to work out where and why — three full
        // reads of the WAL, and it reported a hardcoded placeholder as the
        // offending value.
        let mut failure: Option<WalError> = None;

        self.scan(WalCursor::beginning(), |frame| {
            if frame.prev_hash != prev_hash {
                failure = Some(WalError::ChainBroken {
                    sequence: frame.sequence,
                    expected: hex(&prev_hash),
                    actual: hex(&frame.prev_hash),
                });
                return false;
            }

            // Defence in depth. `decode` has already verified this, so it should
            // be unreachable — kept so that if `decode` is ever changed to skip
            // hash validation for speed, `verify` still catches it rather than
            // silently degrading into a chain-link-only check.
            let computed = frame.compute_hash();
            if computed != frame.frame_hash {
                failure = Some(WalError::FrameHashMismatch {
                    sequence: frame.sequence,
                    expected: hex(&computed),
                    actual: hex(&frame.frame_hash),
                });
                return false;
            }

            prev_hash = frame.frame_hash;
            count += 1;
            true
        })?;

        if let Some(err) = failure {
            return Err(err);
        }

        debug!(frames_verified = count, "hash chain verified OK");
        Ok(count)
    }

    /// WAL statistics.
    pub fn stats(&self) -> WalResult<WalStats> {
        let ids = self.segment_ids()?;
        let mut total_frames = 0u64;
        let mut total_bytes = 0u64;
        let mut first_seq = None;
        let mut last_seq = None;
        let mut last_hash = GENESIS_HASH;

        for &id in &ids {
            let path = segment_path(&self.config.dir, id);
            let mut r = SegmentReader::open(&path)?;
            let frames = r.all_frames()?;
            total_frames += frames.len() as u64;
            total_bytes += path.metadata()?.len();
            for f in &frames {
                if first_seq.is_none() {
                    first_seq = Some(f.sequence);
                }
                last_seq = Some(f.sequence);
                last_hash = f.frame_hash;
            }
        }

        Ok(WalStats {
            segments: ids.len() as u64,
            total_frames,
            total_bytes,
            first_sequence: first_seq.unwrap_or(0),
            last_sequence: last_seq.unwrap_or(0),
            chain_tip: last_hash,
        })
    }
}

#[derive(Debug)]
pub struct WalStats {
    pub segments: u64,
    pub total_frames: u64,
    pub total_bytes: u64,
    pub first_sequence: u64,
    pub last_sequence: u64,
    pub chain_tip: [u8; 32],
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
