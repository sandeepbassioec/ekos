//! [`PartitionReader`] — memory-mapped reader for `.vvsst` partition files.
//!
//! ## Why mmap?
//!
//! `mmap` maps file pages directly into the process address space.  The OS
//! fetches only the pages actually accessed (demand paging), meaning a
//! point-lookup that touches one column of one partition reads exactly the
//! pages it needs without a `read()` syscall.
//!
//! On **Linux**, `MmapOptions::populate()` pre-faults all pages into the TLB
//! at open time — useful when the full partition will be scanned.  On Windows,
//! `populate()` is a no-op; demand-paging still applies.
//!
//! ## Verification
//!
//! On `open()` the reader verifies both the header CRC32 and the full-file
//! CRC32 before returning.  A single flipped bit is detected immediately.

use byteorder::{LittleEndian, ReadBytesExt};
use std::io::{Cursor, Read};
use std::path::Path;

use crate::{
    StorageError, StorageResult,
    column::ColumnValues,
    format::{COL_META_SIZE, ENCODING_DICT, ENCODING_RAW, HEADER_FIXED_SIZE, MAGIC, VERSION},
};

// ── Internal metadata per column ─────────────────────────────────────────────

struct ColMeta {
    encoding: u8,
    data_offset: u64,
    #[allow(dead_code)]
    data_byte_len: u64,
    dict_offset: u64,
    dict_byte_len: u64,
    pub zone_min: u64,
    pub zone_max: u64,
}

// ── PartitionReader ───────────────────────────────────────────────────────────

/// Memory-mapped reader for a single `.vvsst` partition file.
pub struct PartitionReader {
    // Keep the file open so the mapping stays valid on all platforms.
    _file: std::fs::File,
    mmap: memmap2::Mmap,
    // Cached header fields (avoids re-parsing on every call).
    file_id: u16,
    partition_id: u16,
    row_count: u32,
    col_count: u32,
    schema: Vec<String>,
    col_meta: Vec<ColMeta>,
}

impl PartitionReader {
    /// Open a `.vvsst` file, verify both CRCs, and parse metadata.
    ///
    /// Returns `Err` immediately on magic mismatch, version mismatch,
    /// or any CRC failure.
    pub fn open(path: &Path) -> StorageResult<Self> {
        let file = std::fs::File::open(path)?;
        // SAFETY: file content must not be modified while the Mmap is live.
        // VectorVolt SSTables are immutable once written — this invariant holds.
        let mmap = unsafe { memmap2::MmapOptions::new().populate().map(&file)? };
        Self::parse(file, mmap)
    }

    fn parse(file: std::fs::File, mmap: memmap2::Mmap) -> StorageResult<Self> {
        let data: &[u8] = &mmap;

        // ── Magic + minimum size check ────────────────────────────────────
        if data.len() < HEADER_FIXED_SIZE + 4 {
            return Err(StorageError::InvalidMagic);
        }
        if &data[0..8] != MAGIC.as_ref() {
            return Err(StorageError::InvalidMagic);
        }

        // ── File CRC (verify whole file first — cheapest corruption gate) ─
        let file_len = data.len();
        let stored_file_crc = u32::from_le_bytes(data[file_len - 4..].try_into().unwrap());
        let computed_file_crc = crc32fast::hash(&data[..file_len - 4]);
        if computed_file_crc != stored_file_crc {
            return Err(StorageError::FileCrcMismatch {
                expected: computed_file_crc,
                stored: stored_file_crc,
            });
        }

        // ── Fixed header ──────────────────────────────────────────────────
        let mut cur = Cursor::new(data);
        cur.set_position(8); // skip magic (already verified)
        let version = cur.read_u32::<LittleEndian>()?;
        if version != VERSION {
            return Err(StorageError::UnsupportedVersion(version));
        }
        let file_id = cur.read_u16::<LittleEndian>()?;
        let partition_id = cur.read_u16::<LittleEndian>()?;
        let row_count = cur.read_u32::<LittleEndian>()?;
        let col_count = cur.read_u32::<LittleEndian>()?;
        let schema_len = cur.read_u32::<LittleEndian>()?;
        let stored_header_crc = cur.read_u32::<LittleEndian>()?;
        // cursor is now at position 32

        // ── Header CRC ────────────────────────────────────────────────────
        let computed_header_crc = crc32fast::hash(&data[0..28]);
        if computed_header_crc != stored_header_crc {
            return Err(StorageError::HeaderCrcMismatch {
                expected: computed_header_crc,
                stored: stored_header_crc,
            });
        }

        // ── Schema ────────────────────────────────────────────────────────
        let schema_start = HEADER_FIXED_SIZE;
        let schema_end = schema_start + schema_len as usize;
        let mut scur = Cursor::new(&data[schema_start..schema_end]);
        let mut schema: Vec<String> = Vec::with_capacity(col_count as usize);
        for _ in 0..col_count {
            let name_len = scur.read_u16::<LittleEndian>()? as usize;
            let mut name_bytes = vec![0u8; name_len];
            scur.read_exact(&mut name_bytes)?;
            let name = String::from_utf8(name_bytes)
                .map_err(|_| StorageError::InvalidData("column name is not valid UTF-8".into()))?;
            schema.push(name);
        }

        // ── Column metadata ───────────────────────────────────────────────
        let meta_start = schema_end;
        let meta_end = meta_start + col_count as usize * COL_META_SIZE;
        let mut mcur = Cursor::new(&data[meta_start..meta_end]);
        let mut col_meta: Vec<ColMeta> = Vec::with_capacity(col_count as usize);
        for _ in 0..col_count {
            let encoding = mcur.read_u8()?;
            let mut reserved = [0u8; 7];
            mcur.read_exact(&mut reserved)?;
            let data_offset = mcur.read_u64::<LittleEndian>()?;
            let data_byte_len = mcur.read_u64::<LittleEndian>()?;
            let dict_offset = mcur.read_u64::<LittleEndian>()?;
            let dict_byte_len = mcur.read_u64::<LittleEndian>()?;
            let zone_min = mcur.read_u64::<LittleEndian>()?;
            let zone_max = mcur.read_u64::<LittleEndian>()?;
            col_meta.push(ColMeta {
                encoding,
                data_offset,
                data_byte_len,
                dict_offset,
                dict_byte_len,
                zone_min,
                zone_max,
            });
        }

        Ok(PartitionReader {
            _file: file,
            mmap,
            file_id,
            partition_id,
            row_count,
            col_count,
            schema,
            col_meta,
        })
    }

    // ── Accessors ─────────────────────────────────────────────────────────────

    pub fn file_id(&self) -> u16 {
        self.file_id
    }
    pub fn partition_id(&self) -> u16 {
        self.partition_id
    }
    pub fn row_count(&self) -> u32 {
        self.row_count
    }
    pub fn col_count(&self) -> u32 {
        self.col_count
    }

    /// Column name by index.
    pub fn col_name(&self, col_idx: usize) -> StorageResult<&str> {
        self.schema
            .get(col_idx)
            .map(String::as_str)
            .ok_or(StorageError::ColumnOutOfRange(col_idx, self.col_count))
    }

    /// `(min, max)` zone statistics for column `col_idx`.
    pub fn zone_min_max(&self, col_idx: usize) -> StorageResult<(u64, u64)> {
        let m = self
            .col_meta
            .get(col_idx)
            .ok_or(StorageError::ColumnOutOfRange(col_idx, self.col_count))?;
        Ok((m.zone_min, m.zone_max))
    }

    /// Decode and return the values for column `col_idx`.
    pub fn read_column(&self, col_idx: usize) -> StorageResult<ColumnValues> {
        let m = self
            .col_meta
            .get(col_idx)
            .ok_or(StorageError::ColumnOutOfRange(col_idx, self.col_count))?;
        let data: &[u8] = &self.mmap;
        let row_count = self.row_count as usize;

        match m.encoding {
            ENCODING_RAW => {
                let start = m.data_offset as usize;
                let end = start + row_count * 8;
                let mut cur = Cursor::new(&data[start..end]);
                let mut vals = Vec::with_capacity(row_count);
                for _ in 0..row_count {
                    vals.push(cur.read_u64::<LittleEndian>()?);
                }
                Ok(ColumnValues::Raw(vals))
            }
            ENCODING_DICT => {
                let dict_size = m.dict_byte_len as usize / 8;
                let ds = m.dict_offset as usize;
                let de = ds + m.dict_byte_len as usize;
                let mut dcur = Cursor::new(&data[ds..de]);
                let mut dict = Vec::with_capacity(dict_size);
                for _ in 0..dict_size {
                    dict.push(dcur.read_u64::<LittleEndian>()?);
                }

                let is = de; // indices start right after the dict
                let ie = is + row_count * 4;
                let mut icur = Cursor::new(&data[is..ie]);
                let mut indices = Vec::with_capacity(row_count);
                for _ in 0..row_count {
                    indices.push(icur.read_u32::<LittleEndian>()?);
                }

                Ok(ColumnValues::DictEncoded { dict, indices })
            }
            b => Err(StorageError::UnknownEncoding(b)),
        }
    }
}
