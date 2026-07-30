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
    StorageError, StorageResult, bitpack,
    column::ColumnValues,
    column::FRAME_BLOCK,
    format::{
        COL_META_SIZE, ENCODING_BLOCKED_FRAME, ENCODING_DICT, ENCODING_FRAME, ENCODING_RAW,
        ENCODING_RLE, HEADER_FIXED_SIZE, MAGIC, MIN_READABLE_VERSION, VERSION,
    },
};

// ── Internal metadata per column ─────────────────────────────────────────────

struct ColMeta {
    encoding: u8,
    /// Bits per packed value, for the frame encodings. Zero for the others.
    bit_width: u8,
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
            let mut reserved = [0u8; 6];
            mcur.read_exact(&mut reserved)?;
            let data_offset = mcur.read_u64::<LittleEndian>()?;
            let data_byte_len = mcur.read_u64::<LittleEndian>()?;
            let dict_offset = mcur.read_u64::<LittleEndian>()?;
            let dict_byte_len = mcur.read_u64::<LittleEndian>()?;
            let zone_min = mcur.read_u64::<LittleEndian>()?;
            let zone_max = mcur.read_u64::<LittleEndian>()?;
            col_meta.push(ColMeta {
                encoding,
                bit_width,
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

    /// Read `N` bytes at `start`, or an error if that range is outside the file.
    ///
    /// Every offset in this reader is computed from *file contents* — column
    /// metadata, dictionary indices — so none of it may be trusted. Direct slice
    /// indexing (`data[a..b]`) **panics** on an out-of-range range, which would turn
    /// a corrupt partition into a process abort. `.get()` turns it into an error.
    #[inline]
    fn bytes_at<const N: usize>(data: &[u8], start: usize) -> StorageResult<[u8; N]> {
        let end = start
            .checked_add(N)
            .ok_or(StorageError::OffsetOutsideFile {
                start,
                end: usize::MAX,
                len: data.len(),
            })?;
        let slice = data
            .get(start..end)
            .ok_or(StorageError::OffsetOutsideFile {
                start,
                end,
                len: data.len(),
            })?;
        // Infallible: `get` already guaranteed exactly N bytes.
        Ok(slice.try_into().expect("slice length checked by get()"))
    }

    /// Read a single value from column `col_idx` at `row_offset`.
    ///
    /// O(1) for both Raw and Dict encodings — reads exactly 8 or 12 bytes from the
    /// mmap without allocating or deserializing the full column. This is the path
    /// point lookups should use; [`read_column`](Self::read_column) is for scans.
    ///
    /// # Errors
    ///
    /// Never panics, for any `col_idx`, `row_offset`, or file content:
    /// - [`StorageError::ColumnOutOfRange`] — no such column
    /// - [`StorageError::RowOffsetOutOfBounds`] — offset past the last row, which a
    ///   stale `RowDescriptor` (e.g. cached across a compaction) can produce
    /// - [`StorageError::DictIndexOutOfBounds`] — corrupt dictionary index in the file
    /// - [`StorageError::OffsetOutsideFile`] — corrupt column metadata
    pub fn read_value(&self, col_idx: usize, row_offset: usize) -> StorageResult<u64> {
        let m = self
            .col_meta
            .get(col_idx)
            .ok_or(StorageError::ColumnOutOfRange(col_idx, self.col_count))?;

        // Check the caller's offset against the partition before computing any
        // byte position, so the error names the real problem rather than a
        // derived byte range the caller never supplied.
        if row_offset >= self.row_count as usize {
            return Err(StorageError::RowOffsetOutOfBounds {
                row_offset,
                row_count: self.row_count,
            });
        }

        let data: &[u8] = &self.mmap;

        match m.encoding {
            ENCODING_RAW => {
                let offset = m.data_offset as usize + row_offset * 8;
                Ok(u64::from_le_bytes(Self::bytes_at::<8>(data, offset)?))
            }
            ENCODING_DICT => {
                // Index array starts immediately after the dictionary values.
                let idx_offset = m.dict_offset as usize + m.dict_byte_len as usize + row_offset * 4;
                let dict_idx = u32::from_le_bytes(Self::bytes_at::<4>(data, idx_offset)?) as usize;

                // `dict_idx` came out of the file, so validate it against the
                // dictionary's own declared size rather than trusting it.
                let dict_entries = m.dict_byte_len as usize / 8;
                if dict_idx >= dict_entries {
                    return Err(StorageError::DictIndexOutOfBounds {
                        index: dict_idx,
                        size: dict_entries,
                    });
                }

                let val_offset = m.dict_offset as usize + dict_idx * 8;
                Ok(u64::from_le_bytes(Self::bytes_at::<8>(data, val_offset)?))
            }
            ENCODING_RLE => {
                // Binary search the exclusive run-end array for the first run
                // ending past `row_offset`. O(log r) — the one encoding whose
                // point lookup is not O(1); see `ColumnValues` for why that trade
                // is only accepted when RLE at least halves the column.
                let run_count = m.dict_byte_len as usize / 8;
                if run_count == 0 {
                    return Err(StorageError::InvalidData("RLE column has no runs".into()));
                }
                let ends_start = m.dict_offset as usize + m.dict_byte_len as usize;

                let mut lo = 0usize;
                let mut hi = run_count; // first k with run_ends[k] > row_offset
                while lo < hi {
                    let mid = lo + (hi - lo) / 2;
                    let end = u32::from_le_bytes(Self::bytes_at::<4>(data, ends_start + mid * 4)?)
                        as usize;
                    if end <= row_offset {
                        lo = mid + 1;
                    } else {
                        hi = mid;
                    }
                }
                if lo >= run_count {
                    // Only reachable if the final run end is <= row_offset, i.e.
                    // the runs do not cover row_count rows. The header said they
                    // do, so the file is inconsistent.
                    return Err(StorageError::InvalidData(
                        "RLE runs do not cover row_count".into(),
                    ));
                }
                let val_offset = m.dict_offset as usize + lo * 8;
                Ok(u64::from_le_bytes(Self::bytes_at::<8>(data, val_offset)?))
            }
            ENCODING_FRAME => {
                let base = u64::from_le_bytes(Self::bytes_at::<8>(data, m.data_offset as usize)?);
                let words_start = m.data_offset as usize + 8;
                let offset = Self::unpack_at(data, words_start, m.bit_width, row_offset)?;
                // The file supplies both base and offset, so a corrupt file could
                // make these sum past u64::MAX. Wrapping would fabricate a small
                // value from two large ones and report it as real data.
                base.checked_add(offset).ok_or(StorageError::InvalidData(
                    "frame base + offset overflows u64".into(),
                ))
            }
            ENCODING_BLOCKED_FRAME => {
                let block = row_offset / FRAME_BLOCK;
                let block_count = m.dict_byte_len as usize / 8;
                if block >= block_count {
                    return Err(StorageError::InvalidData(
                        "blocked-frame row is past the last block base".into(),
                    ));
                }
                let base = u64::from_le_bytes(Self::bytes_at::<8>(
                    data,
                    m.dict_offset as usize + block * 8,
                )?);
                let words_start = m.dict_offset as usize + m.dict_byte_len as usize;
                let offset = Self::unpack_at(data, words_start, m.bit_width, row_offset)?;
                base.checked_add(offset).ok_or(StorageError::InvalidData(
                    "blocked-frame base + offset overflows u64".into(),
                ))
            }
            b => Err(StorageError::UnknownEncoding(b)),
        }
    }

    /// Unpack one bit-packed value straight from the mmap, without materializing
    /// the word array.
    ///
    /// Reads at most two `u64`s. The packed words are not `u64`-aligned in the
    /// file (`data_offset` is arbitrary), so they are read via
    /// [`bytes_at`](Self::bytes_at) rather than reinterpreted — an unaligned
    /// `*const u64` deref is undefined behaviour in Rust even on x86, where the
    /// hardware would tolerate it.
    #[inline]
    fn unpack_at(data: &[u8], words_start: usize, bit_width: u8, i: usize) -> StorageResult<u64> {
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

        let lo = u64::from_le_bytes(Self::bytes_at::<8>(data, words_start + word * 8)?);
        let mut v = lo >> shift;
        if shift + w > 64 {
            let hi = u64::from_le_bytes(Self::bytes_at::<8>(data, words_start + (word + 1) * 8)?);
            v |= hi << (64 - shift);
        }
        let mask = if w >= 64 { u64::MAX } else { (1u64 << w) - 1 };
        Ok(v & mask)
    }

    /// Byte address of one cell, for prefetching. `None` if out of range or the
    /// column is dictionary-encoded (where the value lives behind an indirection,
    /// so a single address is not meaningful).
    #[inline]
    fn raw_cell_ptr(&self, col_idx: usize, row_offset: usize) -> Option<*const u8> {
        let m = self.col_meta.get(col_idx)?;
        if m.encoding != ENCODING_RAW || row_offset >= self.row_count as usize {
            return None;
        }
        let off = m.data_offset as usize + row_offset * 8;
        let data: &[u8] = &self.mmap;
        if off + 8 > data.len() {
            return None;
        }
        Some(unsafe { data.as_ptr().add(off) })
    }

    /// Row offsets where column `col_idx` falls in `[lo, hi]`, inclusive.
    ///
    /// Uses the SIMD range filter (AVX2 where available, scalar otherwise), which
    /// measured 2.5–2.6× the scalar loop on cache-resident data. The comparison is
    /// unsigned, so primary keys above `i64::MAX` behave correctly — see the
    /// range-shift note in `vectorvolt-hal`'s AVX2 module.
    pub fn filter_offsets(&self, col_idx: usize, lo: u64, hi: u64) -> StorageResult<Vec<u32>> {
        // An inverted range can never match; skip the decode entirely.
        if lo > hi {
            return Ok(Vec::new());
        }
        let values = self.read_column(col_idx)?.to_raw_vec();
        let mut out = Vec::new();
        vectorvolt_hal::simd::filter_range(&values, lo, hi, &mut out);
        Ok(out)
    }

    /// Every column's value for one row, via the O(1) [`read_value`](Self::read_value) path.
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
    /// precisely so the crossover can be re-derived per platform later (the S1a
    /// cache measurements showed thresholds do not transfer between machines).
    pub const LATE_MATERIALIZATION_MAX_SELECTIVITY: f64 = 0.15;

    /// Build full rows for `offsets`, choosing the cheaper strategy.
    ///
    /// ## Why adaptive rather than always-late
    ///
    /// Late materialization — read only the cells belonging to matched rows — is a
    /// large win on a selective query: at 1% selectivity over 3 columns it touches
    /// ~3% of the cells a full decode would. But it is a *loss* on an unselective
    /// one, where per-cell call overhead is paid for nearly every cell anyway and a
    /// bulk decode amortises better.
    ///
    /// The filter has already run by this point, so the exact match count is known
    /// — the choice needs no estimate or statistics, which is consistent with
    /// `QUERY_EXECUTION_MODEL.md`'s stance against cost-based planning.
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

        if selectivity > Self::LATE_MATERIALIZATION_MAX_SELECTIVITY {
            // Bulk path: one decode per column, then gather.
            let n = self.col_count as usize;
            let cols: Vec<Vec<u64>> = (0..n)
                .map(|c| self.read_column(c).map(|cv| cv.to_raw_vec()))
                .collect::<StorageResult<_>>()?;
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
        // adjacent in memory and the hardware prefetcher sees no stride — this is
        // exactly the indexed-gather case explicit prefetch exists for
        // (VV-HAL-018/023 integration).
        const PREFETCH_ROWS_AHEAD: usize = 4;
        let n = self.col_count as usize;
        let mut rows_out = Vec::with_capacity(offsets.len());

        for (i, &off) in offsets.iter().enumerate() {
            if let Some(&future) = offsets.get(i + PREFETCH_ROWS_AHEAD) {
                for c in 0..n {
                    if let Some(p) = self.raw_cell_ptr(c, future as usize) {
                        vectorvolt_hal::prefetch::prefetch(p, vectorvolt_hal::Locality::L1);
                    }
                }
            }
            rows_out.push(self.read_row(off as usize)?);
        }
        Ok(rows_out)
    }

    /// Decode and return the values for column `col_idx`.
    ///
    /// Deserializes the full column. Use [`read_value`] for point lookups.
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
                let mut cur = Cursor::new(Self::section(data, start, end, "raw column")?);
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
                let mut dcur = Cursor::new(Self::section(data, ds, de, "dictionary")?);
                let mut dict = Vec::with_capacity(dict_size);
                for _ in 0..dict_size {
                    dict.push(dcur.read_u64::<LittleEndian>()?);
                }

                let is = de; // indices start right after the dict
                let ie = is + row_count * 4;
                let mut icur = Cursor::new(Self::section(data, is, ie, "dictionary indices")?);
                let mut indices = Vec::with_capacity(row_count);
                for _ in 0..row_count {
                    indices.push(icur.read_u32::<LittleEndian>()?);
                }

                Ok(ColumnValues::DictEncoded { dict, indices })
            }
            ENCODING_RLE => {
                let run_count = m.dict_byte_len as usize / 8;
                let mut values = Vec::with_capacity(run_count);
                for k in 0..run_count {
                    values.push(u64::from_le_bytes(Self::bytes_at::<8>(
                        data,
                        m.dict_offset as usize + k * 8,
                    )?));
                }
                let ends_start = m.dict_offset as usize + m.dict_byte_len as usize;
                let mut run_ends = Vec::with_capacity(run_count);
                for k in 0..run_count {
                    run_ends.push(u32::from_le_bytes(Self::bytes_at::<4>(
                        data,
                        ends_start + k * 4,
                    )?));
                }
                // `run_ends` must be strictly increasing and end exactly at
                // row_count, or `to_raw_vec` would produce the wrong length (or
                // panic on a reversed subtraction). Validated here rather than
                // trusted, because these bytes came off disk.
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
                let base = u64::from_le_bytes(Self::bytes_at::<8>(data, m.data_offset as usize)?);
                let words_start = m.data_offset as usize + 8;
                let packed = Self::read_packed_words(data, words_start, m.bit_width, row_count)?;
                Ok(ColumnValues::Frame {
                    base,
                    bit_width: m.bit_width,
                    packed,
                    row_count,
                })
            }
            ENCODING_BLOCKED_FRAME => {
                let block_count = m.dict_byte_len as usize / 8;
                if block_count != row_count.div_ceil(FRAME_BLOCK) {
                    return Err(StorageError::InvalidData(
                        "blocked-frame base count does not match row_count".into(),
                    ));
                }
                let mut block_bases = Vec::with_capacity(block_count);
                for k in 0..block_count {
                    block_bases.push(u64::from_le_bytes(Self::bytes_at::<8>(
                        data,
                        m.dict_offset as usize + k * 8,
                    )?));
                }
                let words_start = m.dict_offset as usize + m.dict_byte_len as usize;
                let packed = Self::read_packed_words(data, words_start, m.bit_width, row_count)?;
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
        data: &[u8],
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
                data,
                words_start + k * 8,
            )?));
        }
        Ok(packed)
    }
}
