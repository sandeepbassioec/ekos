//! Reads committed frames out of a live WAL directory for shipping.

use std::path::{Path, PathBuf};

use vectorvolt_wal::segment::{SegmentHeader, SEG_HEADER_SIZE};
use vectorvolt_wal::{list_segment_ids, segment_path, SegmentReader, WalError};

use crate::error::{ReplResult, ReplicationError};
use crate::proto::WireFrame;

/// First and last sequence the WAL directory can serve.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct WalBounds {
    /// Lowest sequence still on disk (`base_seq` of the oldest segment);
    /// `None` when there are no segments.
    pub first_seq: Option<u64>,
    /// Highest sequence on disk (0 when the WAL is empty).
    pub tip: u64,
}

fn is_eof(e: &WalError) -> bool {
    matches!(e, WalError::Io(io) if io.kind() == std::io::ErrorKind::UnexpectedEof)
}

pub(crate) fn wal_bounds(dir: &Path) -> ReplResult<WalBounds> {
    let ids = list_segment_ids(dir)?;
    let Some(&first) = ids.first() else {
        return Ok(WalBounds {
            first_seq: None,
            tip: 0,
        });
    };
    let first_seq = SegmentReader::open(&segment_path(dir, first))?
        .header
        .base_seq;
    // Highest frame: newest segment that has any intact frame.
    let mut tip = 0;
    for &id in ids.iter().rev() {
        let mut r = SegmentReader::open(&segment_path(dir, id))?;
        let (frames, _, _) = r.intact_prefix()?;
        if let Some(f) = frames.last() {
            tip = f.sequence;
            break;
        }
    }
    Ok(WalBounds {
        first_seq: Some(first_seq),
        tip,
    })
}

/// Cursor over the WAL that yields frames with `sequence > after`.
pub(crate) struct WalTail {
    dir: PathBuf,
    seg_id: u64,
    offset: u64,
    after: u64,
    /// Highest sequence returned (or `after` if none yet).
    pub last_seq: u64,
}

impl WalTail {
    /// Position at the segment that holds `after + 1`.
    pub fn open(dir: &Path, after: u64) -> ReplResult<Self> {
        let ids = list_segment_ids(dir)?;
        let mut seg_id = ids.first().copied().unwrap_or(0);
        for &id in &ids {
            let base = SegmentReader::open(&segment_path(dir, id))?.header.base_seq;
            if base <= after + 1 {
                seg_id = id;
            } else {
                break;
            }
        }
        Ok(Self {
            dir: dir.to_owned(),
            seg_id,
            offset: SEG_HEADER_SIZE,
            after,
            last_seq: after,
        })
    }

    /// Read up to `max` frames appended since the last call. An incomplete
    /// frame at the end of the active segment is left for the next call.
    pub fn read_batch(&mut self, max: usize) -> ReplResult<Vec<WireFrame>> {
        let mut out = Vec::new();
        loop {
            let ids = list_segment_ids(&self.dir)?;
            if ids.is_empty() {
                return Ok(out);
            }
            if !ids.contains(&self.seg_id) {
                if ids[0] > self.seg_id {
                    return Err(ReplicationError::Wal(format!(
                        "WAL segment {} was removed while streaming",
                        self.seg_id
                    )));
                }
                return Ok(out);
            }
            let mut reader = match SegmentReader::open(&segment_path(&self.dir, self.seg_id)) {
                Ok(r) => r,
                // Header of a segment still being created.
                Err(e) if is_eof(&e) => return Ok(out),
                Err(e) => return Err(e.into()),
            };
            reader.seek_to(self.offset)?;
            let header: SegmentHeader = reader.header.clone();
            loop {
                let pos = reader.file_position()?;
                match reader.next_frame() {
                    Ok(None) => break,
                    Ok(Some(f)) => {
                        self.offset = reader.file_position()?;
                        if f.sequence <= self.after {
                            continue;
                        }
                        self.last_seq = f.sequence;
                        out.push(WireFrame {
                            seq: f.sequence,
                            timestamp_ns: f.timestamp_ns,
                            key_id: header.key_id,
                            payload: f.payload,
                        });
                        if out.len() >= max {
                            return Ok(out);
                        }
                    }
                    Err(e) if is_eof(&e) => {
                        self.offset = pos;
                        return Ok(out);
                    }
                    Err(e) => return Err(e.into()),
                }
            }
            // Segment fully read: advance only if a newer one exists.
            match ids.iter().find(|&&id| id > self.seg_id) {
                Some(&next) => {
                    self.seg_id = next;
                    self.offset = SEG_HEADER_SIZE;
                }
                None => return Ok(out),
            }
        }
    }
}
