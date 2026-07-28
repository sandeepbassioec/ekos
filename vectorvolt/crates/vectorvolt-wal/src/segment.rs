//! WAL segment file management.
//!
//! Segment file format:
//!
//! ```text
//! ┌──────────────────────────────────────────────────────┐
//! │  SEG_MAGIC    8 bytes  "VWALSEG0"                    │
//! │  version      4 bytes  u32 LE, currently 1           │
//! │  segment_id   8 bytes  u64 LE, 0-indexed             │
//! │  base_seq     8 bytes  u64 LE  first sequence in seg │
//! │  created_ns   8 bytes  u64 LE  nanoseconds UTC       │
//! │  flags        4 bytes  reserved = 0                  │
//! │  reserved    24 bytes  zeroed                        │
//! └──────────────────────────────────────────────────────┘
//! Total header: 64 bytes.  Frames follow immediately.
//! ```

use crate::error::{WalError, WalResult};
use crate::frame::Frame;
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use std::fs::{File, OpenOptions};
use std::io::{BufWriter, Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};

pub const SEG_MAGIC: &[u8; 8] = b"VWALSEG0";
pub const SEG_VERSION: u32 = 1;
pub const SEG_HEADER_SIZE: u64 = 64;

/// Produces the canonical file name for a segment.
pub fn segment_path(dir: &Path, id: u64) -> PathBuf {
    dir.join(format!("{:020}.vwal", id))
}

/// Parse a segment id from a `.vwal` file name, or `None` for other files.
pub fn segment_id_from_path(path: &Path) -> Option<u64> {
    if path.extension()?.to_str()? != "vwal" {
        return None;
    }
    path.file_stem()?.to_str()?.parse().ok()
}

// ── Segment header ────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct SegmentHeader {
    pub segment_id: u64,
    pub base_seq: u64,
    pub created_ns: u64,
}

impl SegmentHeader {
    pub fn new(segment_id: u64, base_seq: u64) -> Self {
        let created_ns = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;
        Self {
            segment_id,
            base_seq,
            created_ns,
        }
    }

    pub fn write<W: Write>(&self, w: &mut W) -> WalResult<()> {
        w.write_all(SEG_MAGIC)?;
        w.write_u32::<LittleEndian>(SEG_VERSION)?;
        w.write_u64::<LittleEndian>(self.segment_id)?;
        w.write_u64::<LittleEndian>(self.base_seq)?;
        w.write_u64::<LittleEndian>(self.created_ns)?;
        w.write_u32::<LittleEndian>(0)?; // flags
        w.write_all(&[0u8; 24])?; // reserved
        Ok(())
    }

    pub fn read<R: Read>(r: &mut R, path: &str) -> WalResult<Self> {
        let mut magic = [0u8; 8];
        r.read_exact(&mut magic)?;
        if &magic != SEG_MAGIC {
            return Err(WalError::InvalidSegmentMagic {
                path: path.to_owned(),
            });
        }
        let version = r.read_u32::<LittleEndian>()?;
        if version > SEG_VERSION {
            return Err(WalError::UnsupportedVersion {
                version,
                max: SEG_VERSION,
            });
        }
        let segment_id = r.read_u64::<LittleEndian>()?;
        let base_seq = r.read_u64::<LittleEndian>()?;
        let created_ns = r.read_u64::<LittleEndian>()?;
        let mut tail = [0u8; 28]; // flags(4) + reserved(24)
        r.read_exact(&mut tail)?;
        Ok(Self {
            segment_id,
            base_seq,
            created_ns,
        })
    }
}

// ── SegmentWriter ─────────────────────────────────────────────────────────────

pub struct SegmentWriter {
    pub id: u64,
    pub path: PathBuf,
    pub byte_offset: u64,
    pub frame_count: u64,
    writer: BufWriter<File>,
    fsync: bool,
    max_bytes: u64,
}

impl SegmentWriter {
    /// Create a new segment file.
    pub fn create(
        dir: &Path,
        segment_id: u64,
        base_seq: u64,
        max_bytes: u64,
        fsync: bool,
    ) -> WalResult<Self> {
        let path = segment_path(dir, segment_id);
        let file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)?;
        let mut writer = BufWriter::new(file);
        SegmentHeader::new(segment_id, base_seq).write(&mut writer)?;
        writer.flush()?;
        Ok(Self {
            id: segment_id,
            path,
            byte_offset: SEG_HEADER_SIZE,
            frame_count: 0,
            writer,
            fsync,
            max_bytes,
        })
    }

    /// Append pre-encoded frame bytes (from Frame::encode).
    pub fn append_bytes(&mut self, bytes: &[u8]) -> WalResult<()> {
        self.writer.write_all(bytes)?;
        self.byte_offset += bytes.len() as u64;
        self.frame_count += 1;
        Ok(())
    }

    /// Flush OS buffers and optionally fsync.
    pub fn flush(&mut self) -> WalResult<()> {
        self.writer.flush()?;
        if self.fsync {
            self.writer.get_ref().sync_data()?;
        }
        Ok(())
    }

    /// Re-open an existing segment for appending (used during WAL recovery).
    pub fn reopen(
        id: u64,
        path: PathBuf,
        file: File,
        byte_offset: u64,
        frame_count: u64,
        max_bytes: u64,
        fsync: bool,
    ) -> Self {
        Self {
            id,
            path,
            byte_offset,
            frame_count,
            writer: BufWriter::new(file),
            fsync,
            max_bytes,
        }
    }

    /// True if appending `extra_bytes` would exceed the segment size limit.
    pub fn would_exceed(&self, extra_bytes: usize) -> bool {
        self.byte_offset + extra_bytes as u64 > self.max_bytes
    }
}

// ── SegmentReader ─────────────────────────────────────────────────────────────

pub struct SegmentReader {
    pub header: SegmentHeader,
    pub file: File,
    pub path: PathBuf,
}

impl SegmentReader {
    pub fn open(path: &Path) -> WalResult<Self> {
        let mut file = File::open(path)?;
        let header = SegmentHeader::read(&mut file, &path.to_string_lossy())?;
        Ok(Self {
            header,
            file,
            path: path.to_owned(),
        })
    }

    /// Read the next frame at the current file position.
    pub fn next_frame(&mut self) -> WalResult<Option<Frame>> {
        let pos = self.file.stream_position()?;
        // peek: try reading 1 byte to detect EOF cleanly
        let mut peek = [0u8; 1];
        match std::io::Read::read(&mut self.file, &mut peek) {
            Ok(0) => return Ok(None),
            Ok(_) => {
                // seek back to re-read properly in Frame::decode
                self.file.seek(SeekFrom::Start(pos))?;
            }
            Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => return Ok(None),
            Err(e) => return Err(e.into()),
        }
        let frame = Frame::decode(&mut self.file, self.header.segment_id, pos)?;
        Ok(Some(frame))
    }

    /// Seek to a byte offset within the segment (relative to file start, not header).
    pub fn seek_to(&mut self, offset: u64) -> WalResult<()> {
        self.file.seek(SeekFrom::Start(offset))?;
        Ok(())
    }

    /// Seek to just after the segment header (first frame position).
    pub fn rewind(&mut self) -> WalResult<()> {
        self.seek_to(SEG_HEADER_SIZE)
    }

    /// Return all frames in this segment (used by verify + inspect).
    pub fn all_frames(&mut self) -> WalResult<Vec<Frame>> {
        self.rewind()?;
        let mut frames = Vec::new();
        while let Some(f) = self.next_frame()? {
            frames.push(f);
        }
        Ok(frames)
    }
}

// ── Segment discovery ─────────────────────────────────────────────────────────

/// Return segment ids sorted ascending from a WAL directory.
pub fn list_segment_ids(dir: &Path) -> WalResult<Vec<u64>> {
    if !dir.exists() {
        return Ok(vec![]);
    }
    let mut ids: Vec<u64> = std::fs::read_dir(dir)?
        .filter_map(|e| e.ok())
        .filter_map(|e| segment_id_from_path(&e.path()))
        .collect();
    ids.sort_unstable();
    Ok(ids)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frame::{GENESIS_HASH, PayloadType};
    use tempfile::TempDir;

    #[test]
    fn create_and_read_segment() {
        let dir = TempDir::new().unwrap();
        let mut writer = SegmentWriter::create(dir.path(), 0, 1, 64 * 1024 * 1024, false).unwrap();

        let mut frame = Frame {
            sequence: 1,
            timestamp_ns: 42,
            prev_hash: GENESIS_HASH,
            payload_type: PayloadType::Data,
            payload: b"test payload".to_vec(),
            frame_hash: [0u8; 32],
        };
        let encoded = frame.encode();
        writer.append_bytes(&encoded).unwrap();
        writer.flush().unwrap();

        let path = segment_path(dir.path(), 0);
        let mut reader = SegmentReader::open(&path).unwrap();
        reader.rewind().unwrap();
        let f = reader.next_frame().unwrap().unwrap();
        assert_eq!(f.sequence, 1);
        assert_eq!(f.payload, b"test payload");
    }
}
