//! WAL segment file management.
//!
//! Segment file format:
//!
//! ```text
//! ┌──────────────────────────────────────────────────────┐
//! │  SEG_MAGIC    8 bytes  "VWALSEG0"                    │
//! │  version      4 bytes  u32 LE, 1 plain / 2 encrypted │
//! │  segment_id   8 bytes  u64 LE, 0-indexed             │
//! │  base_seq     8 bytes  u64 LE  first sequence in seg │
//! │  created_ns   8 bytes  u64 LE  nanoseconds UTC       │
//! │  flags        4 bytes  bit 0 = frames are encrypted  │
//! │  key_id       8 bytes  key fingerprint (bit 0 set)   │
//! │  reserved    16 bytes  zeroed                        │
//! └──────────────────────────────────────────────────────┘
//! Total header: 64 bytes.  Frames follow immediately.
//! ```
//!
//! ## Encryption metadata (PP-52)
//!
//! A segment whose `flags` has [`SEG_FLAG_ENCRYPTED`] set holds mutation
//! payloads encrypted under the key whose non-secret fingerprint
//! ([`KeyId`]) is in `key_id`. Such segments are written as version 2, so an
//! older binary refuses them loudly instead of misreading ciphertext as
//! plaintext. Segments written before this existed have `flags = 0` (and
//! zeroed reserved bytes) and read back as plaintext with no key id.

use crate::error::{WalError, WalResult};
use crate::frame::Frame;
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use std::fs::{File, OpenOptions};
use std::io::{BufWriter, Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};

pub const SEG_MAGIC: &[u8; 8] = b"VWALSEG0";
/// Highest segment version this build reads (and the one written for
/// encrypted segments). Plaintext segments are still written as version 1.
pub const SEG_VERSION: u32 = 2;
const SEG_VERSION_PLAIN: u32 = 1;
pub const SEG_HEADER_SIZE: u64 = 64;
/// `flags` bit 0: frames in this segment are encrypted under `key_id`.
pub const SEG_FLAG_ENCRYPTED: u32 = 1;

/// Non-secret 8-byte fingerprint of an encryption key, stored in segment
/// headers so a wrong or missing key is caught before replay. It is
/// `vectorvolt_crypto::KeyStore::key_id()` (first 8 bytes of
/// SHA3-256("vectorvolt-key-id-v1" || key)); this crate only stores it.
pub type KeyId = [u8; 8];

/// Lower-case hex of a key id, for error messages.
pub fn key_id_hex(id: &KeyId) -> String {
    id.iter().map(|b| format!("{b:02x}")).collect()
}

/// Additional authenticated data binding an encrypted frame payload to its
/// position: a domain tag, the key id and the frame's sequence number (u64
/// LE). Sequence numbers are unique within a WAL, so a ciphertext copied over
/// another frame no longer authenticates.
pub fn frame_aad(key_id: &KeyId, sequence: u64) -> [u8; 24] {
    let mut aad = [0u8; 24];
    aad[..8].copy_from_slice(b"VVWAAD01");
    aad[8..16].copy_from_slice(key_id);
    aad[16..].copy_from_slice(&sequence.to_le_bytes());
    aad
}

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
    /// `Some` when the segment's frames are encrypted (flags bit 0), holding
    /// the fingerprint of the key they were written with.
    pub key_id: Option<KeyId>,
}

impl SegmentHeader {
    pub fn new(segment_id: u64, base_seq: u64) -> Self {
        Self::with_key_id(segment_id, base_seq, None)
    }

    /// A header for a segment whose frames are encrypted under `key_id`
    /// (`None` = plaintext).
    pub fn with_key_id(segment_id: u64, base_seq: u64, key_id: Option<KeyId>) -> Self {
        let created_ns = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;
        Self {
            segment_id,
            base_seq,
            created_ns,
            key_id,
        }
    }

    /// Whether this segment's frames are encrypted.
    pub fn is_encrypted(&self) -> bool {
        self.key_id.is_some()
    }

    pub fn write<W: Write>(&self, w: &mut W) -> WalResult<()> {
        let (version, flags, id) = match self.key_id {
            Some(id) => (SEG_VERSION, SEG_FLAG_ENCRYPTED, id),
            None => (SEG_VERSION_PLAIN, 0, [0u8; 8]),
        };
        w.write_all(SEG_MAGIC)?;
        w.write_u32::<LittleEndian>(version)?;
        w.write_u64::<LittleEndian>(self.segment_id)?;
        w.write_u64::<LittleEndian>(self.base_seq)?;
        w.write_u64::<LittleEndian>(self.created_ns)?;
        w.write_u32::<LittleEndian>(flags)?;
        w.write_all(&id)?; // key id (zero when plaintext)
        w.write_all(&[0u8; 16])?; // reserved
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
        let flags = r.read_u32::<LittleEndian>()?;
        let mut tail = [0u8; 24]; // key_id(8) + reserved(16)
        r.read_exact(&mut tail)?;
        let key_id = if flags & SEG_FLAG_ENCRYPTED != 0 {
            let mut id = [0u8; 8];
            id.copy_from_slice(&tail[..8]);
            Some(id)
        } else {
            None
        };
        Ok(Self {
            segment_id,
            base_seq,
            created_ns,
            key_id,
        })
    }
}

// ── SegmentWriter ─────────────────────────────────────────────────────────────

pub struct SegmentWriter {
    pub id: u64,
    pub path: PathBuf,
    pub byte_offset: u64,
    pub frame_count: u64,
    /// Key id recorded in this segment's header (`None` = plaintext).
    pub key_id: Option<KeyId>,
    writer: BufWriter<File>,
    fsync: bool,
    max_bytes: u64,
}

impl SegmentWriter {
    /// Create a new segment file. `key_id` marks its frames as encrypted
    /// under that key (`None` = plaintext); see the module docs.
    pub fn create(
        dir: &Path,
        segment_id: u64,
        base_seq: u64,
        max_bytes: u64,
        fsync: bool,
        key_id: Option<KeyId>,
    ) -> WalResult<Self> {
        let path = segment_path(dir, segment_id);
        let file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)?;
        let mut writer = BufWriter::new(file);
        SegmentHeader::with_key_id(segment_id, base_seq, key_id).write(&mut writer)?;
        writer.flush()?;
        Ok(Self {
            id: segment_id,
            path,
            byte_offset: SEG_HEADER_SIZE,
            frame_count: 0,
            key_id,
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
    /// `key_id` is the one its header records.
    #[allow(clippy::too_many_arguments)]
    pub fn reopen(
        id: u64,
        path: PathBuf,
        file: File,
        byte_offset: u64,
        frame_count: u64,
        max_bytes: u64,
        fsync: bool,
        key_id: Option<KeyId>,
    ) -> Self {
        Self {
            id,
            path,
            byte_offset,
            frame_count,
            key_id,
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

    /// Read every intact frame, tolerating an incomplete final frame.
    ///
    /// Returns the frames, the byte offset just past the last intact frame, and
    /// whether the bytes after it look like a torn write. A torn write is a
    /// final frame that ends early (`UnexpectedEof`) or a tail of zero bytes —
    /// the two shapes an interrupted append leaves behind. Any other decode
    /// failure is real corruption and is returned as an error: truncating it
    /// would silently discard frames that may already have been acknowledged.
    pub fn intact_prefix(&mut self) -> WalResult<(Vec<Frame>, u64, bool)> {
        self.rewind()?;
        let file_len = self.file.metadata()?.len();
        let mut frames = Vec::new();
        loop {
            let pos = self.file.stream_position()?;
            if pos >= file_len {
                return Ok((frames, pos, false));
            }
            match self.next_frame() {
                Ok(Some(f)) => frames.push(f),
                Ok(None) => return Ok((frames, pos, false)),
                Err(WalError::Io(e)) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                    return Ok((frames, pos, true));
                }
                Err(e) => {
                    if self.rest_is_zero(pos)? {
                        return Ok((frames, pos, true));
                    }
                    return Err(e);
                }
            }
        }
    }

    fn rest_is_zero(&mut self, from: u64) -> WalResult<bool> {
        self.file.seek(SeekFrom::Start(from))?;
        let mut buf = [0u8; 4096];
        loop {
            let n = std::io::Read::read(&mut self.file, &mut buf)?;
            if n == 0 {
                return Ok(true);
            }
            if buf[..n].iter().any(|&b| b != 0) {
                return Ok(false);
            }
        }
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
        let mut writer =
            SegmentWriter::create(dir.path(), 0, 1, 64 * 1024 * 1024, false, None).unwrap();

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

    #[test]
    fn header_records_encryption_and_key_id() {
        let dir = TempDir::new().unwrap();
        let id: KeyId = *b"\x01\x02\x03\x04\x05\x06\x07\x08";
        SegmentWriter::create(dir.path(), 3, 10, 1 << 20, false, Some(id)).unwrap();
        let path = segment_path(dir.path(), 3);
        let r = SegmentReader::open(&path).unwrap();
        assert_eq!(r.header.key_id, Some(id));
        assert_eq!(r.header.base_seq, 10);
        assert_eq!(std::fs::metadata(&path).unwrap().len(), SEG_HEADER_SIZE);
    }

    #[test]
    fn legacy_header_reads_as_plaintext() {
        // A version-1 header with flags = 0, exactly as older builds wrote it.
        let dir = TempDir::new().unwrap();
        let mut bytes = Vec::new();
        bytes.extend_from_slice(SEG_MAGIC);
        bytes.extend_from_slice(&1u32.to_le_bytes());
        bytes.extend_from_slice(&0u64.to_le_bytes());
        bytes.extend_from_slice(&1u64.to_le_bytes());
        bytes.extend_from_slice(&0u64.to_le_bytes());
        bytes.extend_from_slice(&[0u8; 28]);
        assert_eq!(bytes.len() as u64, SEG_HEADER_SIZE);
        let path = segment_path(dir.path(), 0);
        std::fs::write(&path, bytes).unwrap();
        let r = SegmentReader::open(&path).unwrap();
        assert!(!r.header.is_encrypted());
    }

    #[test]
    fn aad_binds_sequence_and_key_id() {
        let a: KeyId = [1u8; 8];
        let b: KeyId = [2u8; 8];
        assert_ne!(frame_aad(&a, 1), frame_aad(&a, 2));
        assert_ne!(frame_aad(&a, 1), frame_aad(&b, 1));
    }
}
