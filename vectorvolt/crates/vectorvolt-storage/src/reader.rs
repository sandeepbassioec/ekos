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
use std::borrow::Cow;
use std::io::{Cursor, Read};
use std::path::Path;

use crate::{
    StorageError, StorageResult, bitpack,
    column::ColumnValues,
    column::FRAME_BLOCK,
    format::{
        COL_META_SIZE, COMPRESSION_LZ4, COMPRESSION_NONE, COMPRESSION_ZSTD, ENCODING_BLOCKED_FRAME,
        ENCODING_DICT, ENCODING_FRAME, ENCODING_RAW, ENCODING_RLE, HEADER_FIXED_SIZE, MAGIC,
        MIN_READABLE_VERSION, VERSION,
    },
};

// ── Internal metadata per column ─────────────────────────────────────────────

struct ColMeta {
    encoding: u8,
    /// Bits per packed value, for the frame encodings. Zero for the others.
    bit_width: u8,
    /// `COMPRESSION_NONE`, `COMPRESSION_LZ4`, or `COMPRESSION_ZSTD`.
    compression: u8,
    data_offset: u64,
    /// Bytes actually stored in the file (compressed length, when compressed).
    data_byte_len: u64,
    /// Payload length after decompression.
    uncompressed_len: u64,
    /// Length of the payload's first section, for the two-section encodings.
    /// **Relative to the payload**, not the file.
    first_section_len: u64,
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
    /// Cached answer to "is column 0 (the PK) non-decreasing in this file?".
    /// See [`is_pk_sorted`](Self::is_pk_sorted).
    pk_sorted: std::sync::OnceLock<bool>,
}

/// Cheap, file-size-independent zone-map peek (VV-ENG-014).
///
/// `PartitionReader::open` performs a full-file CRC32 over every byte and, on
/// Linux, prefaults every page via `mmap`'s `populate()` — cost proportional to
/// the whole partition, paid whether or not a single row is ever read from it.
/// The per-column zone stats (`zone_min`/`zone_max`) live in the column-metadata
/// section right after the header and schema, so they can be read with a small,
/// sequential, bounded read — cost proportional to column count, never to row
/// count or file size.
///
/// This only verifies the header CRC (28 bytes), not the full-file CRC, so a
/// caller that decides — based on the zone map — not to read this partition's
/// row data never pays for verifying data it will not touch. Any caller that
/// *does* go on to read row data must still call [`PartitionReader::open`],
/// which performs the full verification.
#[derive(Debug, Clone)]
pub struct ZonePeek {
    pub file_id: u16,
    pub partition_id: u16,
    pub row_count: u32,
    /// `(min, max)` per column, indexed by column position.
    pub zones: Vec<(u64, u64)>,
}

impl PartitionReader {
    /// Read just enough of `path` to answer zone-map queries — see [`ZonePeek`].
    pub fn peek_zone_maps(path: &Path) -> StorageResult<ZonePeek> {
        use std::io::BufReader;

        let file = std::fs::File::open(path)?;
        let mut r = BufReader::new(file);

        let mut header = [0u8; HEADER_FIXED_SIZE];
        r.read_exact(&mut header)?;
        if header[0..8] != MAGIC[..] {
            return Err(StorageError::InvalidMagic);
        }

        let mut cur = Cursor::new(&header[8..HEADER_FIXED_SIZE]);
        let version = cur.read_u32::<LittleEndian>()?;
        if !(MIN_READABLE_VERSION..=VERSION).contains(&version) {
            return Err(StorageError::UnsupportedVersion(version));
        }
        let file_id = cur.read_u16::<LittleEndian>()?;
        let partition_id = cur.read_u16::<LittleEndian>()?;
        let row_count = cur.read_u32::<LittleEndian>()?;
        let col_count = cur.read_u32::<LittleEndian>()?;
        let schema_len = cur.read_u32::<LittleEndian>()?;
        let stored_header_crc = cur.read_u32::<LittleEndian>()?;

        let computed_header_crc = crc32fast::hash(&header[0..28]);
        if computed_header_crc != stored_header_crc {
            return Err(StorageError::HeaderCrcMismatch {
                expected: computed_header_crc,
                stored: stored_header_crc,
            });
        }

        // Schema bytes are irrelevant to a zone-map peek — read and discard.
        let mut schema_buf = vec![0u8; schema_len as usize];
        r.read_exact(&mut schema_buf)?;

        let meta_len = (col_count as usize).saturating_mul(COL_META_SIZE);
        let mut meta_buf = vec![0u8; meta_len];
        r.read_exact(&mut meta_buf)?;
        let mut mcur = Cursor::new(&meta_buf[..]);
        let mut zones = Vec::with_capacity(col_count as usize);
        for _ in 0..col_count {
            let _encoding = mcur.read_u8()?;
            let _bit_width = mcur.read_u8()?;
            let _compression = mcur.read_u8()?;
            let mut reserved = [0u8; 5];
            mcur.read_exact(&mut reserved)?;
            let _data_offset = mcur.read_u64::<LittleEndian>()?;
            let _data_byte_len = mcur.read_u64::<LittleEndian>()?;
            let _uncompressed_len = mcur.read_u64::<LittleEndian>()?;
            let _first_section_len = mcur.read_u64::<LittleEndian>()?;
            let zone_min = mcur.read_u64::<LittleEndian>()?;
            let zone_max = mcur.read_u64::<LittleEndian>()?;
            zones.push((zone_min, zone_max));
        }

        Ok(ZonePeek {
            file_id,
            partition_id,
            row_count,
            zones,
        })
    }

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

    /// A file section, or an error if it does not lie inside the mapping.
    ///
    /// Every offset in a `.vvsst` file is read *from* that file, so a truncated
    /// or hostile file can name a range past the end of the mapping. Slicing with
    /// `&data[a..b]` turns that into a panic — which for a database means the
    /// process dies on a partially-written file instead of reporting it and
    /// carrying on. `what` names the section so the error says which one.
    fn section<'a>(
        data: &'a [u8],
        start: usize,
        end: usize,
        what: &'static str,
    ) -> StorageResult<&'a [u8]> {
        data.get(start..end).ok_or_else(|| {
            StorageError::InvalidData(format!(
                "{what} section [{start}, {end}) lies outside a {}-byte file",
                data.len()
            ))
        })
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
        // A range, not equality: v1 files remain readable because v2 only claimed
        // one previously-reserved metadata byte and added encoding tags. Rejecting
        // them would make the encoding work a breaking change for no reason.
        if !(MIN_READABLE_VERSION..=VERSION).contains(&version) {
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
        //
        // `schema_len` and `col_count` come out of the file, so every slice
        // derived from them is a potential panic on a truncated or corrupt file.
        // `&data[a..b]` panics; `.get(a..b)` does not. This is the same defect
        // class as the `read_value` bounds bug — see `section` below.
        let schema_start = HEADER_FIXED_SIZE;
        let schema_end = schema_start + schema_len as usize;
        let mut scur = Cursor::new(Self::section(data, schema_start, schema_end, "schema")?);
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
        let meta_end = meta_start
            .checked_add((col_count as usize).saturating_mul(COL_META_SIZE))
            .ok_or(StorageError::InvalidData(
                "column metadata length overflows".into(),
            ))?;
        let mut mcur = Cursor::new(Self::section(
            data,
            meta_start,
            meta_end,
            "column metadata",
        )?);
        let mut col_meta: Vec<ColMeta> = Vec::with_capacity(col_count as usize);
        for _ in 0..col_count {
            let encoding = mcur.read_u8()?;
            let bit_width = mcur.read_u8()?;
            let compression = mcur.read_u8()?;
            let mut reserved = [0u8; 5];
            mcur.read_exact(&mut reserved)?;
            let data_offset = mcur.read_u64::<LittleEndian>()?;
            let data_byte_len = mcur.read_u64::<LittleEndian>()?;
            let uncompressed_len = mcur.read_u64::<LittleEndian>()?;
            let first_section_len = mcur.read_u64::<LittleEndian>()?;
            let zone_min = mcur.read_u64::<LittleEndian>()?;
            let zone_max = mcur.read_u64::<LittleEndian>()?;
            col_meta.push(ColMeta {
                encoding,
                bit_width,
                compression,
                data_offset,
                data_byte_len,
                uncompressed_len,
                first_section_len,
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
            pk_sorted: std::sync::OnceLock::new(),
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

    /// Read `N` bytes at `start`, or an error if that range is outside the file.
    ///
    /// Every offset in this reader is computed from *file contents* — column
    /// metadata, dictionary indices — so none of it may be trusted. Direct slice
    /// indexing (`data[a..b]`) **panics** on an out-of-range range, which would turn
    /// a corrupt partition into a process abort. `.get()` turns it into an error.
    #[inline]
    #[allow(
        clippy::unnecessary_lazy_evaluations,
        reason = "measured: eager construction costs 2x on the point-read path"
    )]
    fn bytes_at<const N: usize>(data: &[u8], start: usize) -> StorageResult<[u8; N]> {
        // `ok_or_else` throughout. This is the hottest function in the crate —
        // several calls per decoded value — and `ok_or` builds its error
        // unconditionally, on the success path too.
        let end = start
            .checked_add(N)
            .ok_or_else(|| StorageError::OffsetOutsideFile {
                start,
                end: usize::MAX,
                len: data.len(),
            })?;
        let slice = data
            .get(start..end)
            .ok_or_else(|| StorageError::OffsetOutsideFile {
                start,
                end,
                len: data.len(),
            })?;
        // Infallible: `get` already guaranteed exactly N bytes.
        Ok(slice.try_into().expect("slice length checked by get()"))
    }

    /// The column's payload bytes, decompressing if needed.
    ///
    /// Returns a borrowed slice of the mmap for a verbatim column and an owned
    /// buffer for a compressed one. **That difference is the entire cost model of
    /// compression**: the borrowed case is free and O(1)-indexable, the owned case
    /// allocates and decompresses the whole column to answer for a single value.
    /// See [`COMPRESSION_LZ4`].
    fn payload(&self, m: &ColMeta) -> StorageResult<Cow<'_, [u8]>> {
        let data: &[u8] = &self.mmap;
        let start = m.data_offset as usize;
        let end = start
            .checked_add(m.data_byte_len as usize)
            .ok_or_else(|| StorageError::InvalidData("column payload length overflows".into()))?;
        let raw = Self::section(data, start, end, "column payload")?;

        match m.compression {
            COMPRESSION_NONE => Ok(Cow::Borrowed(raw)),
            COMPRESSION_LZ4 => {
                // `uncompressed_len` comes from the file, so it is a
                // corruption-controlled allocation size. Capped rather than
                // trusted: without this, 8 bad bytes request an 18-exabyte Vec.
                const MAX_PAYLOAD: u64 = 1 << 34; // 16 GiB
                if m.uncompressed_len > MAX_PAYLOAD {
                    return Err(StorageError::InvalidData(format!(
                        "declared uncompressed length {} exceeds the {MAX_PAYLOAD}-byte cap",
                        m.uncompressed_len
                    )));
                }
                let out = lz4_flex::block::decompress(raw, m.uncompressed_len as usize)
                    .map_err(|e| StorageError::InvalidData(format!("LZ4 decompress: {e}")))?;
                Ok(Cow::Owned(out))
            }
            COMPRESSION_ZSTD => {
                const MAX_PAYLOAD: u64 = 1 << 34; // 16 GiB
                if m.uncompressed_len > MAX_PAYLOAD {
                    return Err(StorageError::InvalidData(format!(
                        "declared uncompressed length {} exceeds the {MAX_PAYLOAD}-byte cap",
                        m.uncompressed_len
                    )));
                }
                let out = zstd::bulk::decompress(raw, m.uncompressed_len as usize)
                    .map_err(|e| StorageError::InvalidData(format!("Zstd decompress: {e}")))?;
                Ok(Cow::Owned(out))
            }
            other => Err(StorageError::InvalidData(format!(
                "unknown compression tag {other}"
            ))),
        }
    }

    /// True if point reads on this partition are O(1).
    ///
    /// False when any column is compressed, because reading one value then costs a
    /// full-column decompress. Exposed so a caller choosing between a point lookup
    /// and a scan can tell, rather than discovering it as unexplained latency.
    pub fn is_point_read_optimal(&self) -> bool {
        self.col_meta
            .iter()
            .all(|m| m.compression == COMPRESSION_NONE)
    }

    /// True if `col_idx` is stored compressed.
    pub fn is_column_compressed(&self, col_idx: usize) -> bool {
        self.col_meta
            .get(col_idx)
            .is_some_and(|m| m.compression != COMPRESSION_NONE)
    }

    /// Read a single value from column `col_idx` at `row_offset`.
    ///
    /// **O(1) for every encoding** on an uncompressed column — at most a few
    /// unaligned reads out of the mmap, no allocation, no full-column decode. The
    /// two exceptions are stated rather than hidden:
    /// - `Rle` is O(log r): a binary search over the run-end array.
    /// - A **compressed** column is O(column): the payload must be decompressed in
    ///   full first. That is why compression is opt-in — see [`COMPRESSION_LZ4`].
    ///
    /// # Errors
    ///
    /// Never panics, for any `col_idx`, `row_offset`, or file content.
    pub fn read_value(&self, col_idx: usize, row_offset: usize) -> StorageResult<u64> {
        let m = self
            .col_meta
            .get(col_idx)
            .ok_or(StorageError::ColumnOutOfRange(col_idx, self.col_count))?;

        // Check the caller's offset against the partition before computing any
        // byte position, so the error names the real problem rather than a derived
        // byte range the caller never supplied.
        if row_offset >= self.row_count as usize {
            return Err(StorageError::RowOffsetOutOfBounds {
                row_offset,
                row_count: self.row_count,
            });
        }

        let payload = self.payload(m)?;
        Self::value_at(&payload, m, row_offset)
    }

    /// Decode one value out of a column payload.
    ///
    /// All offsets are relative to the payload start, so this is identical for
    /// compressed and verbatim columns — which is the reason the format stores
    /// relative section lengths rather than absolute file offsets. A compressed
    /// payload has no file offset for its inner sections at all.
    fn value_at(p: &[u8], m: &ColMeta, row: usize) -> StorageResult<u64> {
        let first_len = m.first_section_len as usize;
        match m.encoding {
            ENCODING_RAW => Ok(u64::from_le_bytes(Self::bytes_at::<8>(p, row * 8)?)),
            ENCODING_DICT => {
                let idx_off = first_len + row * 4;
                let dict_idx = u32::from_le_bytes(Self::bytes_at::<4>(p, idx_off)?) as usize;
                // Came out of the file, so validate against the dictionary's own
                // declared size rather than trusting it.
                let entries = first_len / 8;
                if dict_idx >= entries {
                    return Err(StorageError::DictIndexOutOfBounds {
                        index: dict_idx,
                        size: entries,
                    });
                }
                Ok(u64::from_le_bytes(Self::bytes_at::<8>(p, dict_idx * 8)?))
            }
            ENCODING_RLE => {
                // Binary search the exclusive run-end array for the first run
                // ending past `row`. O(log r) — the one encoding whose point lookup
                // is not O(1); see `ColumnValues` for why that trade is only taken
                // when RLE at least halves the column.
                let run_count = first_len / 8;
                if run_count == 0 {
                    return Err(StorageError::InvalidData("RLE column has no runs".into()));
                }
                let ends = first_len;
                let (mut lo, mut hi) = (0usize, run_count);
                while lo < hi {
                    let mid = lo + (hi - lo) / 2;
                    let end = u32::from_le_bytes(Self::bytes_at::<4>(p, ends + mid * 4)?) as usize;
                    if end <= row {
                        lo = mid + 1;
                    } else {
                        hi = mid;
                    }
                }
                if lo >= run_count {
                    return Err(StorageError::InvalidData(
                        "RLE runs do not cover row_count".into(),
                    ));
                }
                Ok(u64::from_le_bytes(Self::bytes_at::<8>(p, lo * 8)?))
            }
            ENCODING_FRAME => {
                let base = u64::from_le_bytes(Self::bytes_at::<8>(p, 0)?);
                let offset = Self::unpack_at(p, 8, m.bit_width, row)?;
                // Both operands come from the file, so a corrupt file could make
                // these sum past u64::MAX. Wrapping would fabricate a small value
                // out of two large ones and report it as real data.
                base.checked_add(offset).ok_or_else(|| {
                    StorageError::InvalidData("frame base + offset overflows u64".into())
                })
            }
            ENCODING_BLOCKED_FRAME => {
                let block = row / FRAME_BLOCK;
                if block >= first_len / 8 {
                    return Err(StorageError::InvalidData(
                        "blocked-frame row is past the last block base".into(),
                    ));
                }
                let base = u64::from_le_bytes(Self::bytes_at::<8>(p, block * 8)?);
                let offset = Self::unpack_at(p, first_len, m.bit_width, row)?;
                base.checked_add(offset).ok_or_else(|| {
                    StorageError::InvalidData("blocked-frame base + offset overflows u64".into())
                })
            }
            b => Err(StorageError::UnknownEncoding(b)),
        }
    }

    /// Unpack one bit-packed value from a payload, without materializing the word
    /// array.
    ///
    /// Reads at most two `u64`s. The packed words are not `u64`-aligned (a payload
    /// starts at an arbitrary file offset), so they are read via
    /// [`bytes_at`](Self::bytes_at) rather than reinterpreted — an unaligned
    /// `*const u64` deref is undefined behaviour in Rust even on x86, where the
    /// hardware itself would tolerate it.
    #[inline]
    fn unpack_at(p: &[u8], words_start: usize, bit_width: u8, i: usize) -> StorageResult<u64> {
        if bit_width == 0 {
            return Ok(0);
        }
        if bit_width > 64 {
            return Err(StorageError::InvalidData(
                "column bit_width exceeds 64".into(),
            ));
        }
        let w = bit_width as usize;
        let pos = i * w;
        let word = pos / 64;
        let shift = pos % 64;

        let lo = u64::from_le_bytes(Self::bytes_at::<8>(p, words_start + word * 8)?);
        let mut v = lo >> shift;
        if shift + w > 64 {
            let hi = u64::from_le_bytes(Self::bytes_at::<8>(p, words_start + (word + 1) * 8)?);
            v |= hi << (64 - shift);
        }
        let mask = if w >= 64 { u64::MAX } else { (1u64 << w) - 1 };
        Ok(v & mask)
    }

    /// Byte address of one cell, for prefetching.
    ///
    /// `None` unless the column is verbatim `Raw`, where a value has a single
    /// stable file address. Every other encoding reaches its value through an
    /// indirection or a bit offset, and a compressed column has no file address for
    /// its values at all.
    #[inline]
    fn raw_cell_ptr(&self, col_idx: usize, row_offset: usize) -> Option<*const u8> {
        let m = self.col_meta.get(col_idx)?;
        if m.encoding != ENCODING_RAW
            || m.compression != COMPRESSION_NONE
            || row_offset >= self.row_count as usize
        {
            return None;
        }
        let off = m.data_offset as usize + row_offset * 8;
        let data: &[u8] = &self.mmap;
        if off + 8 > data.len() {
            return None;
        }
        Some(unsafe { data.as_ptr().add(off) })
    }

    /// Decode a whole column into `out`, reusing its allocation (VV-STO-013/014).
    ///
    /// The scan primitive. Three things it does that the previous
    /// `read_column().to_raw_vec()` did not:
    ///
    /// 1. **One allocation, reusable across partitions.** The old path allocated a
    ///    `ColumnValues` (holding a dictionary or a packed word array), then
    ///    allocated a second `Vec<u64>` to flatten it, then dropped the first. For
    ///    a 4096-row column that is two allocations and a 32 KiB copy, per column,
    ///    per partition scanned.
    /// 2. **Decodes straight from the mmap.** No intermediate `ColumnValues`.
    /// 3. **Prefetches ahead of the read cursor.** This is the smaller win: a
    ///    sequential walk is the one access pattern the hardware prefetcher already
    ///    handles well, so the hint mostly helps at the start of a column and after
    ///    a page boundary. The large win is (1).
    pub fn decode_column_into(&self, col_idx: usize, out: &mut Vec<u64>) -> StorageResult<()> {
        let m = self
            .col_meta
            .get(col_idx)
            .ok_or(StorageError::ColumnOutOfRange(col_idx, self.col_count))?;
        let n = self.row_count as usize;
        let payload = self.payload(m)?;
        let p: &[u8] = &payload;

        out.clear();
        out.reserve(n);

        // Prefetch distance in cache lines, chosen to cover roughly L2 latency at
        // the rate a decode loop consumes bytes.
        const PREFETCH_LINES: usize = 8;
        let prefetch = |byte_off: usize| {
            let target = byte_off + PREFETCH_LINES * vectorvolt_hal::cache::CACHE_LINE;
            if target < p.len() {
                // SAFETY: `target` is in bounds, and prefetch never dereferences.
                let ptr = unsafe { p.as_ptr().add(target) };
                vectorvolt_hal::prefetch::prefetch(ptr, vectorvolt_hal::Locality::L1);
            }
        };

        let first_len = m.first_section_len as usize;
        match m.encoding {
            ENCODING_RAW => {
                for i in 0..n {
                    prefetch(i * 8);
                    out.push(u64::from_le_bytes(Self::bytes_at::<8>(p, i * 8)?));
                }
            }
            ENCODING_DICT => {
                let entries = first_len / 8;
                let mut dict = Vec::with_capacity(entries);
                for k in 0..entries {
                    dict.push(u64::from_le_bytes(Self::bytes_at::<8>(p, k * 8)?));
                }
                for i in 0..n {
                    prefetch(first_len + i * 4);
                    let idx =
                        u32::from_le_bytes(Self::bytes_at::<4>(p, first_len + i * 4)?) as usize;
                    let v = *dict.get(idx).ok_or(StorageError::DictIndexOutOfBounds {
                        index: idx,
                        size: entries,
                    })?;
                    out.push(v);
                }
            }
            ENCODING_RLE => {
                // Expanded run by run rather than row by row: one binary search
                // avoided per row, and `extend` fills contiguously.
                let run_count = first_len / 8;
                let mut start = 0usize;
                for k in 0..run_count {
                    let v = u64::from_le_bytes(Self::bytes_at::<8>(p, k * 8)?);
                    let end =
                        u32::from_le_bytes(Self::bytes_at::<4>(p, first_len + k * 4)?) as usize;
                    if end > n || end <= start {
                        return Err(StorageError::InvalidData(
                            "RLE run ends are not strictly increasing within row_count".into(),
                        ));
                    }
                    out.extend(std::iter::repeat_n(v, end - start));
                    start = end;
                }
                if start != n {
                    return Err(StorageError::InvalidData(
                        "RLE runs do not cover row_count".into(),
                    ));
                }
            }
            ENCODING_FRAME => {
                let base = u64::from_le_bytes(Self::bytes_at::<8>(p, 0)?);
                let bw = m.bit_width;
                for i in 0..n {
                    prefetch(8 + (i * bw as usize) / 8);
                    let off = Self::unpack_at(p, 8, bw, i)?;
                    // `ok_or_else`, not `ok_or`. `ok_or` evaluates its argument
                    // eagerly, and `InvalidData(&str.into())` ALLOCATES A STRING —
                    // so the error path was being constructed once per row for an
                    // error that never fires. See the perf note on this function.
                    out.push(base.checked_add(off).ok_or_else(|| {
                        StorageError::InvalidData("frame base + offset overflows u64".into())
                    })?);
                }
            }
            ENCODING_BLOCKED_FRAME => {
                let blocks = first_len / 8;
                if blocks != n.div_ceil(FRAME_BLOCK) {
                    return Err(StorageError::InvalidData(
                        "blocked-frame base count does not match row_count".into(),
                    ));
                }
                let bw = m.bit_width;
                // The base is re-read once per block rather than once per row —
                // 1/64th the reads of the naive `block_bases[i / FRAME_BLOCK]`.
                let mut base = 0u64;
                for i in 0..n {
                    if i % FRAME_BLOCK == 0 {
                        base = u64::from_le_bytes(Self::bytes_at::<8>(p, (i / FRAME_BLOCK) * 8)?);
                    }
                    prefetch(first_len + (i * bw as usize) / 8);
                    let off = Self::unpack_at(p, first_len, bw, i)?;
                    out.push(base.checked_add(off).ok_or_else(|| {
                        StorageError::InvalidData(
                            "blocked-frame base + offset overflows u64".into(),
                        )
                    })?);
                }
            }
            b => return Err(StorageError::UnknownEncoding(b)),
        }
        Ok(())
    }

    /// Row offsets where column `col_idx` falls in `[lo, hi]`, inclusive.
    ///
    /// Uses the SIMD range filter (AVX2 where available, scalar otherwise). The
    /// comparison is unsigned, so primary keys above `i64::MAX` behave correctly —
    /// see the range-shift note in `vectorvolt-hal`'s AVX2 module.
    pub fn filter_offsets(&self, col_idx: usize, lo: u64, hi: u64) -> StorageResult<Vec<u32>> {
        let mut scratch = Vec::new();
        self.filter_offsets_into(col_idx, lo, hi, &mut scratch)
    }

    /// [`filter_offsets`](Self::filter_offsets) with a caller-owned decode buffer.
    ///
    /// A scan across N partitions can hand the same buffer to every one and pay a
    /// single allocation instead of N.
    pub fn filter_offsets_into(
        &self,
        col_idx: usize,
        lo: u64,
        hi: u64,
        scratch: &mut Vec<u64>,
    ) -> StorageResult<Vec<u32>> {
        // An inverted range can never match; skip the decode entirely.
        if lo > hi {
            return Ok(Vec::new());
        }
        self.decode_column_into(col_idx, scratch)?;
        let mut out = Vec::new();
        vectorvolt_hal::simd::filter_range(scratch, lo, hi, &mut out);
        Ok(out)
    }

    /// True if column 0 (the primary key, by convention) is stored
    /// non-decreasing in this file (VV-QRY-002.6).
    ///
    /// Compaction output is always PK-sorted — `compaction::execute_job` builds
    /// it from a `BTreeMap<u64, _>` on purpose, for exactly this reason. A file
    /// fresh out of a memtable flush is **not**: `TxnManager::flush` groups
    /// buffered rows by partition hash and writes each group in drain order,
    /// with no sort step. The two cases are indistinguishable from the file
    /// alone — there is no on-disk flag recording which one produced a given
    /// `.vvsst` — so this checks directly, using the O(1)-per-row point-read
    /// path (`read_value`), and caches the answer for the lifetime of this
    /// reader. The first range query on a given partition pays an O(n)
    /// sequential check; every later one on the same reader is free.
    pub fn is_pk_sorted(&self) -> StorageResult<bool> {
        if let Some(&sorted) = self.pk_sorted.get() {
            return Ok(sorted);
        }
        let n = self.row_count as usize;
        let mut sorted = true;
        if n > 1 {
            let mut prev = self.read_value(0, 0)?;
            for i in 1..n {
                let v = self.read_value(0, i)?;
                if v < prev {
                    sorted = false;
                    break;
                }
                prev = v;
            }
        }
        // A concurrent caller may race and compute the same answer; whichever
        // `set` loses is a harmless no-op, both computed the same value.
        let _ = self.pk_sorted.set(sorted);
        Ok(sorted)
    }

    /// First row offset `i` in `0..row_count` with `read_value(0, i) >= key`
    /// (Rust's `partition_point` semantics), assuming column 0 is sorted.
    fn pk_lower_bound(&self, key: u64) -> StorageResult<usize> {
        let mut lo = 0usize;
        let mut hi = self.row_count as usize;
        while lo < hi {
            let mid = lo + (hi - lo) / 2;
            if self.read_value(0, mid)? < key {
                lo = mid + 1;
            } else {
                hi = mid;
            }
        }
        Ok(lo)
    }

    /// First row offset `i` in `0..row_count` with `read_value(0, i) > key`,
    /// assuming column 0 is sorted.
    fn pk_upper_bound(&self, key: u64) -> StorageResult<usize> {
        let mut lo = 0usize;
        let mut hi = self.row_count as usize;
        while lo < hi {
            let mid = lo + (hi - lo) / 2;
            if self.read_value(0, mid)? <= key {
                lo = mid + 1;
            } else {
                hi = mid;
            }
        }
        Ok(lo)
    }

    /// Row offsets where the primary key (column 0) falls in `[lo, hi]`.
    ///
    /// This is [`filter_offsets`](Self::filter_offsets) specialised for
    /// VV-QRY-002.6: when this partition's PK column is sorted (see
    /// [`is_pk_sorted`](Self::is_pk_sorted)), the matching rows form one
    /// contiguous run, found via two binary searches — O(log n) comparisons,
    /// each an O(1) mmap point-read, no column decode at all — instead of
    /// decoding and predicate-testing every row. When the PK column is not
    /// sorted (an un-compacted flush), this falls back to the same SIMD
    /// linear scan `filter_offsets` uses, so the result is identical either
    /// way; only the cost differs.
    pub fn filter_offsets_pk(&self, lo: u64, hi: u64) -> StorageResult<Vec<u32>> {
        if lo > hi {
            return Ok(Vec::new());
        }
        if !self.is_pk_sorted()? {
            return self.filter_offsets(0, lo, hi);
        }
        let n = self.row_count as usize;
        if n == 0 {
            return Ok(Vec::new());
        }
        let start = self.pk_lower_bound(lo)?;
        let end = self.pk_upper_bound(hi)?;
        Ok((start as u32..end as u32).collect())
    }

    /// Count rows where column `col_idx` falls in `[lo, hi]`, without allocating a match Vec.
    ///
    /// Equivalent to `filter_offsets(col_idx, lo, hi)?.len()` but uses
    /// `simd::count_range` which avoids writing the matching indices. Prefer this
    /// over `filter_offsets` when you only need the count (e.g. index cardinality
    /// estimates, selectivity probes).
    pub fn count_matching(&self, col_idx: usize, lo: u64, hi: u64) -> StorageResult<usize> {
        if lo > hi {
            return Ok(0);
        }
        let mut scratch = Vec::new();
        self.decode_column_into(col_idx, &mut scratch)?;
        Ok(vectorvolt_hal::simd::count_range(&scratch, lo, hi))
    }

    /// SIMD wrapping-sum of every value in column `col_idx`.
    ///
    /// Useful for aggregate queries (`SUM(col)`). The sum wraps on overflow, matching
    /// the semantics of `simd::sum`.
    pub fn column_sum(&self, col_idx: usize) -> StorageResult<u64> {
        let mut scratch = Vec::new();
        self.decode_column_into(col_idx, &mut scratch)?;
        Ok(vectorvolt_hal::simd::sum(&scratch))
    }

    /// SIMD `(min, max)` of every value in column `col_idx`, or `None` if the
    /// partition is empty.
    ///
    /// This decodes the full column and runs `simd::min_max` on the result.
    /// For the stored per-column zone statistics use [`zone_min_max`](Self::zone_min_max),
    /// which is cheaper (no decode). This method is correct even for columns that
    /// were written before zone stats existed.
    pub fn column_min_max(&self, col_idx: usize) -> StorageResult<Option<(u64, u64)>> {
        let mut scratch = Vec::new();
        self.decode_column_into(col_idx, &mut scratch)?;
        Ok(vectorvolt_hal::simd::min_max(&scratch))
    }

    /// Every column's value for one row, via the [`read_value`](Self::read_value) path.
    pub fn read_row(&self, row_offset: usize) -> StorageResult<Vec<u64>> {
        let n = self.col_count as usize;
        let mut values = Vec::with_capacity(n);
        for c in 0..n {
            values.push(self.read_value(c, row_offset)?);
        }
        Ok(values)
    }

    /// Selectivity at or below which late materialization is used.
    ///
    /// Below this, fetching individual cells for matched rows beats decoding whole
    /// columns. Above it, the per-cell call overhead exceeds what skipping
    /// non-matching rows saves, and a bulk decode wins.
    ///
    /// 0.15 is a starting estimate, not a measured optimum. It is a named constant
    /// precisely so the crossover can be re-derived per platform later.
    pub const LATE_MATERIALIZATION_MAX_SELECTIVITY: f64 = 0.15;

    /// Build full rows for `offsets`, choosing the cheaper strategy.
    ///
    /// Late materialization — read only the cells belonging to matched rows — is a
    /// large win on a selective query and a loss on an unselective one, where
    /// per-cell overhead is paid for nearly every cell anyway. The filter has
    /// already run by this point, so the exact match count is known and the choice
    /// needs no estimate or statistics.
    ///
    /// A **compressed** column forces the bulk path regardless of selectivity:
    /// per-cell reads would decompress the whole column once per cell, so late
    /// materialization there is not merely worse, it is quadratic.
    pub fn materialize_rows(&self, offsets: &[u32]) -> StorageResult<Vec<Vec<u64>>> {
        if offsets.is_empty() {
            return Ok(Vec::new());
        }
        let rows = self.row_count as usize;
        let selectivity = if rows == 0 {
            1.0
        } else {
            offsets.len() as f64 / rows as f64
        };
        let any_compressed = !self.is_point_read_optimal();

        if selectivity > Self::LATE_MATERIALIZATION_MAX_SELECTIVITY || any_compressed {
            let n = self.col_count as usize;
            let mut buf = Vec::new();
            let mut cols: Vec<Vec<u64>> = Vec::with_capacity(n);
            for c in 0..n {
                self.decode_column_into(c, &mut buf)?;
                cols.push(std::mem::take(&mut buf));
            }
            return offsets
                .iter()
                .map(|&o| {
                    let o = o as usize;
                    cols.iter()
                        .map(|c| {
                            c.get(o).copied().ok_or(StorageError::RowOffsetOutOfBounds {
                                row_offset: o,
                                row_count: self.row_count,
                            })
                        })
                        .collect::<StorageResult<Vec<u64>>>()
                })
                .collect();
        }

        // Late path: touch only the matched rows' cells.
        //
        // Prefetch a few rows ahead. In a struct-of-arrays layout the cells of one
        // row are scattered across columns, so consecutive matched rows are not
        // adjacent in memory and the hardware prefetcher sees no stride — exactly
        // the indexed-gather case explicit prefetch exists for.
        const PREFETCH_ROWS_AHEAD: usize = 4;
        let n = self.col_count as usize;
        let mut rows_out = Vec::with_capacity(offsets.len());
        for (i, &off) in offsets.iter().enumerate() {
            if let Some(&future) = offsets.get(i + PREFETCH_ROWS_AHEAD) {
                for c in 0..n {
                    if let Some(ptr) = self.raw_cell_ptr(c, future as usize) {
                        vectorvolt_hal::prefetch::prefetch(ptr, vectorvolt_hal::Locality::L1);
                    }
                }
            }
            rows_out.push(self.read_row(off as usize)?);
        }
        Ok(rows_out)
    }

    /// Decode a column back into a [`ColumnValues`].
    ///
    /// Reconstructs the encoded form. Most callers want
    /// [`decode_column_into`](Self::decode_column_into), which skips this
    /// intermediate entirely; this exists for callers that need to inspect or
    /// re-serialize the encoding itself.
    pub fn read_column(&self, col_idx: usize) -> StorageResult<ColumnValues> {
        let m = self
            .col_meta
            .get(col_idx)
            .ok_or(StorageError::ColumnOutOfRange(col_idx, self.col_count))?;
        let row_count = self.row_count as usize;
        let payload = self.payload(m)?;
        let p: &[u8] = &payload;
        let first_len = m.first_section_len as usize;

        match m.encoding {
            ENCODING_RAW => {
                let mut vals = Vec::with_capacity(row_count);
                for i in 0..row_count {
                    vals.push(u64::from_le_bytes(Self::bytes_at::<8>(p, i * 8)?));
                }
                Ok(ColumnValues::Raw(vals))
            }
            ENCODING_DICT => {
                let entries = first_len / 8;
                let mut dict = Vec::with_capacity(entries);
                for k in 0..entries {
                    dict.push(u64::from_le_bytes(Self::bytes_at::<8>(p, k * 8)?));
                }
                let mut indices = Vec::with_capacity(row_count);
                for i in 0..row_count {
                    indices.push(u32::from_le_bytes(Self::bytes_at::<4>(
                        p,
                        first_len + i * 4,
                    )?));
                }
                Ok(ColumnValues::DictEncoded { dict, indices })
            }
            ENCODING_RLE => {
                let run_count = first_len / 8;
                let mut values = Vec::with_capacity(run_count);
                for k in 0..run_count {
                    values.push(u64::from_le_bytes(Self::bytes_at::<8>(p, k * 8)?));
                }
                let mut run_ends = Vec::with_capacity(run_count);
                for k in 0..run_count {
                    run_ends.push(u32::from_le_bytes(Self::bytes_at::<4>(
                        p,
                        first_len + k * 4,
                    )?));
                }
                // Must be strictly increasing and end exactly at row_count, or
                // `to_raw_vec` produces the wrong length (or panics on a reversed
                // subtraction). Validated because these bytes came off disk.
                let mut prev = 0u32;
                for (k, &e) in run_ends.iter().enumerate() {
                    if e <= prev && !(k == 0 && e > 0) {
                        return Err(StorageError::InvalidData(
                            "RLE run ends are not strictly increasing".into(),
                        ));
                    }
                    prev = e;
                }
                if prev as usize != row_count {
                    return Err(StorageError::InvalidData(
                        "RLE run ends do not sum to row_count".into(),
                    ));
                }
                Ok(ColumnValues::Rle { values, run_ends })
            }
            ENCODING_FRAME => {
                let base = u64::from_le_bytes(Self::bytes_at::<8>(p, 0)?);
                let packed = Self::read_packed_words(p, 8, m.bit_width, row_count)?;
                Ok(ColumnValues::Frame {
                    base,
                    bit_width: m.bit_width,
                    packed,
                    row_count,
                })
            }
            ENCODING_BLOCKED_FRAME => {
                let blocks = first_len / 8;
                if blocks != row_count.div_ceil(FRAME_BLOCK) {
                    return Err(StorageError::InvalidData(
                        "blocked-frame base count does not match row_count".into(),
                    ));
                }
                let mut block_bases = Vec::with_capacity(blocks);
                for k in 0..blocks {
                    block_bases.push(u64::from_le_bytes(Self::bytes_at::<8>(p, k * 8)?));
                }
                let packed = Self::read_packed_words(p, first_len, m.bit_width, row_count)?;
                Ok(ColumnValues::BlockedFrame {
                    block_bases,
                    bit_width: m.bit_width,
                    packed,
                    row_count,
                })
            }
            b => Err(StorageError::UnknownEncoding(b)),
        }
    }

    /// Read the packed word array for a bit-packed column.
    fn read_packed_words(
        p: &[u8],
        words_start: usize,
        bit_width: u8,
        row_count: usize,
    ) -> StorageResult<Vec<u64>> {
        if bit_width > 64 {
            return Err(StorageError::InvalidData(
                "column bit_width exceeds 64".into(),
            ));
        }
        let n_words = bitpack::packed_words(row_count, bit_width);
        let mut packed = Vec::with_capacity(n_words);
        for k in 0..n_words {
            packed.push(u64::from_le_bytes(Self::bytes_at::<8>(
                p,
                words_start + k * 8,
            )?));
        }
        Ok(packed)
    }
}
