//! [`write_partition`] — serialize a [`MicroPartition`] to a binary `.vvsst` file.
//!
//! ## Write sequence
//!
//! 1. Build the schema section bytes (variable length).
//! 2. Compute absolute data offsets for every column.
//! 3. Write fixed header (magic + version + counts + schema_len + header_crc32).
//! 4. Write schema section.
//! 5. Write column metadata (56 bytes × col_count).
//! 6. Write column data (SoA order).
//! 7. Append file-level CRC32 footer.
//!
//! Everything is buffered in a `Vec<u8>` and flushed as one `write_all`.
//! This makes the CRC trivial to compute and avoids partial writes.

use byteorder::{LittleEndian, WriteBytesExt};
use std::io::Write;

use crate::{
    StorageError, StorageResult,
    column::ColumnValues,
    format::{
        COL_META_SIZE, COMPRESSION_LZ4, COMPRESSION_NONE, COMPRESSION_ZSTD, HEADER_FIXED_SIZE,
        MAGIC, VERSION,
    },
    partition::MicroPartition,
};

/// How a partition's column payloads are stored (VV-STO-012, VV-STO-013).
///
/// Compression is **off by default and must be asked for**. See
/// [`COMPRESSION_LZ4`] for why: compression covers the whole column payload, so
/// reading one value costs a full-column decompress, turning `read_value` from
/// O(1) into O(column). That is the right trade for cold, scan-only data and
/// the wrong one for everything else.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Compression {
    /// Store payloads verbatim. Keeps O(1) point reads.
    #[default]
    None,
    /// LZ4-compress each column payload. For cold/archive tiers only.
    Lz4,
    /// Zstd-compress each column payload. Better ratio than LZ4 at similar
    /// decode throughput. For cold/archival tiers only.
    Zstd {
        /// Zstd compression level (1–22). 3 is the Zstd default; higher
        /// levels compress more but encode more slowly. Decode speed is
        /// nearly level-independent.
        level: i32,
    },
}

impl Compression {
    fn tag(self) -> u8 {
        match self {
            Compression::None => COMPRESSION_NONE,
            Compression::Lz4 => COMPRESSION_LZ4,
            Compression::Zstd { .. } => COMPRESSION_ZSTD,
        }
    }
}

/// Options for [`write_partition_with`].
#[derive(Debug, Clone, Copy, Default)]
pub struct WriteOptions {
    pub compression: Compression,
}

impl WriteOptions {
    pub fn new() -> Self {
        Self::default()
    }

    /// Enable LZ4. Only for partitions that will be scanned or archived, not
    /// point-queried — see [`Compression`].
    pub fn with_lz4(mut self) -> Self {
        self.compression = Compression::Lz4;
        self
    }

    /// Enable Zstd at the default compression level (3).
    ///
    /// Use [`with_zstd_level`](Self::with_zstd_level) to override. Only for
    /// cold/archival partitions — see [`Compression`].
    pub fn with_zstd(mut self) -> Self {
        self.compression = Compression::Zstd { level: 3 };
        self
    }

    /// Enable Zstd at a specific compression level (1–22).
    pub fn with_zstd_level(mut self, level: i32) -> Self {
        self.compression = Compression::Zstd { level };
        self
    }
}

/// Byte length of a column's first data section, for encodings split in two.
///
/// Three of the five encodings store two arrays back to back, and the reader
/// needs to know where the second starts. That boundary is written into
/// `dict_byte_len` — a field named for the encoding that came first, kept rather
/// than renamed so v1 files stay readable.
///
/// `None` means the encoding has a single section, and `dict_offset`/
/// `dict_byte_len` are written as zero.
fn first_section_bytes(col: &ColumnValues) -> Option<usize> {
    match col {
        // Single-section encodings. `Frame`'s base is a fixed 8 bytes, so the
        // reader locates the packed words without needing a stored boundary.
        ColumnValues::Raw(_) | ColumnValues::Frame { .. } => None,
        ColumnValues::DictEncoded { dict, .. } => Some(dict.len() * 8),
        ColumnValues::Rle { values, .. } => Some(values.len() * 8),
        ColumnValues::BlockedFrame { block_bases, .. } => Some(block_bases.len() * 8),
    }
}

/// Serialize `partition` into `writer` in `.vvsst` format, uncompressed.
///
/// Equivalent to [`write_partition_with`] with default [`WriteOptions`]. Kept as
/// the primary entry point because uncompressed is the correct default for the
/// hot path.
pub fn write_partition<W: Write>(writer: &mut W, partition: &MicroPartition) -> StorageResult<()> {
    write_partition_with(writer, partition, WriteOptions::default())
}

/// Serialize `partition` into `writer` with explicit options.
pub fn write_partition_with<W: Write>(
    writer: &mut W,
    partition: &MicroPartition,
    opts: WriteOptions,
) -> StorageResult<()> {
    if partition.col_count() == 0 {
        return Err(StorageError::EmptyPartition);
    }

    // ── Step 1: schema section bytes ─────────────────────────────────────────
    let mut schema_bytes: Vec<u8> = Vec::new();
    for name in &partition.schema {
        let nb = name.as_bytes();
        schema_bytes.write_u16::<LittleEndian>(nb.len() as u16)?;
        schema_bytes.extend_from_slice(nb);
    }
    let schema_len = schema_bytes.len() as u32;
    let col_count = partition.col_count() as u32;
    let row_count = partition.row_count() as u32;

    // ── Step 2: compute data offsets ─────────────────────────────────────────
    // data_start = HEADER_FIXED_SIZE + schema_len + col_count * COL_META_SIZE
    let data_start =
        HEADER_FIXED_SIZE as u64 + schema_len as u64 + col_count as u64 * COL_META_SIZE as u64;

    struct ColLayout {
        encoding: u8,
        bit_width: u8,
        compression: u8,
        data_offset: u64,
        data_byte_len: u64,
        /// Uncompressed payload length. Equal to `data_byte_len` when the column
        /// is stored verbatim; the decompressed size when it is not. Occupies the
        /// slot historically called `dict_offset`, which became redundant once
        /// section offsets were made relative to the payload — a compressed
        /// payload has no file offset for its inner sections at all.
        uncompressed_len: u64,
        first_section_len: u64,
        zone_min: u64,
        zone_max: u64,
    }

    // Payloads are serialized BEFORE offsets are computed, because compression
    // makes the stored length unknowable in advance. The previous version derived
    // offsets from `payload_bytes()` up front, which is only valid while every
    // column is stored verbatim.
    let mut payloads: Vec<Vec<u8>> = Vec::with_capacity(partition.col_count());
    for col in &partition.columns {
        payloads.push(serialize_payload(col)?);
    }

    let mut layouts: Vec<ColLayout> = Vec::with_capacity(partition.col_count());
    let mut stored: Vec<Vec<u8>> = Vec::with_capacity(partition.col_count());
    let mut cursor = data_start;

    for (col, payload) in partition.columns.iter().zip(payloads) {
        let (zone_min, zone_max) = col.zone_min_max();
        let uncompressed_len = payload.len() as u64;

        let bytes = match opts.compression {
            Compression::None => payload,
            Compression::Lz4 => {
                let packed = lz4_flex::block::compress(&payload);
                // Incompressible payloads exist — the `Raw` encoding is chosen
                // precisely when nothing else helps, and LZ4 on random bytes adds
                // framing overhead. Storing the larger of the two would make
                // "enable compression" able to grow a file, so fall back.
                if packed.len() < payload.len() {
                    packed
                } else {
                    payload
                }
            }
            Compression::Zstd { level } => {
                let packed = zstd::bulk::compress(&payload, level)
                    .map_err(|e| StorageError::InvalidData(format!("Zstd compress: {e}")))?;
                if packed.len() < payload.len() {
                    packed
                } else {
                    payload
                }
            }
        };
        // The tag must describe what was ACTUALLY stored, not what was requested,
        // or the reader will try to decompress verbatim bytes.
        let compression = if bytes.len() as u64 == uncompressed_len {
            COMPRESSION_NONE
        } else {
            opts.compression.tag()
        };

        let data_byte_len = bytes.len() as u64;
        layouts.push(ColLayout {
            encoding: col.encoding_byte(),
            bit_width: col.bit_width(),
            compression,
            data_offset: cursor,
            data_byte_len,
            uncompressed_len,
            first_section_len: first_section_bytes(col).unwrap_or(0) as u64,
            zone_min,
            zone_max,
        });
        cursor += data_byte_len;
        stored.push(bytes);
    }

    // ── Step 3: write into a buffer ───────────────────────────────────────────
    let total_data_bytes = cursor - data_start;
    let expected_size = data_start as usize + total_data_bytes as usize + 4; // +4 for file CRC
    let mut buf: Vec<u8> = Vec::with_capacity(expected_size);

    // Fixed header (28 bytes before the header CRC)
    buf.extend_from_slice(MAGIC);
    buf.write_u32::<LittleEndian>(VERSION)?;
    buf.write_u16::<LittleEndian>(partition.file_id)?;
    buf.write_u16::<LittleEndian>(partition.partition_id)?;
    buf.write_u32::<LittleEndian>(row_count)?;
    buf.write_u32::<LittleEndian>(col_count)?;
    buf.write_u32::<LittleEndian>(schema_len)?;
    let header_crc = crc32fast::hash(&buf); // CRC over first 28 bytes
    buf.write_u32::<LittleEndian>(header_crc)?;
    // buf.len() == 32 == HEADER_FIXED_SIZE

    // ── Step 4: schema section ────────────────────────────────────────────────
    buf.extend_from_slice(&schema_bytes);

    // ── Step 5: column metadata ───────────────────────────────────────────────
    for lo in &layouts {
        buf.write_u8(lo.encoding)?;
        buf.write_u8(lo.bit_width)?;
        buf.write_u8(lo.compression)?;
        buf.extend_from_slice(&[0u8; 5]); // reserved
        buf.write_u64::<LittleEndian>(lo.data_offset)?;
        buf.write_u64::<LittleEndian>(lo.data_byte_len)?;
        buf.write_u64::<LittleEndian>(lo.uncompressed_len)?;
        buf.write_u64::<LittleEndian>(lo.first_section_len)?;
        buf.write_u64::<LittleEndian>(lo.zone_min)?;
        buf.write_u64::<LittleEndian>(lo.zone_max)?;
    }

    // ── Step 6: column data ───────────────────────────────────────────────────
    for (lo, bytes) in layouts.iter().zip(&stored) {
        debug_assert_eq!(
            bytes.len() as u64,
            lo.data_byte_len,
            "payload length changed between layout and write"
        );
        buf.extend_from_slice(bytes);
    }

    // ── Step 7: file CRC footer ───────────────────────────────────────────────
    let file_crc = crc32fast::hash(&buf);
    buf.write_u32::<LittleEndian>(file_crc)?;

    writer.write_all(&buf)?;
    Ok(())
}

/// Serialize one column's payload, with all section offsets **relative to the
/// payload start**.
///
/// Relative rather than absolute because a compressed payload has no file offset
/// for its inner sections — they only exist after decompression. Making both
/// cases relative means the reader has exactly one layout to understand instead
/// of one per compression mode.
fn serialize_payload(col: &ColumnValues) -> StorageResult<Vec<u8>> {
    let mut buf: Vec<u8> = Vec::with_capacity(col.payload_bytes());
    match col {
        ColumnValues::Raw(vals) => {
            for &v in vals {
                buf.write_u64::<LittleEndian>(v)?;
            }
        }
        ColumnValues::DictEncoded { dict, indices } => {
            for &v in dict {
                buf.write_u64::<LittleEndian>(v)?;
            }
            for &i in indices {
                buf.write_u32::<LittleEndian>(i)?;
            }
        }
        ColumnValues::Rle { values, run_ends } => {
            for &v in values {
                buf.write_u64::<LittleEndian>(v)?;
            }
            for &e in run_ends {
                buf.write_u32::<LittleEndian>(e)?;
            }
        }
        ColumnValues::Frame { base, packed, .. } => {
            buf.write_u64::<LittleEndian>(*base)?;
            for &w in packed {
                buf.write_u64::<LittleEndian>(w)?;
            }
        }
        ColumnValues::BlockedFrame {
            block_bases,
            packed,
            ..
        } => {
            for &b in block_bases {
                buf.write_u64::<LittleEndian>(b)?;
            }
            for &w in packed {
                buf.write_u64::<LittleEndian>(w)?;
            }
        }
    }
    // Offsets in column metadata are computed from `payload_bytes()`, so a
    // mismatch here misaligns every later column. Cheap to check.
    debug_assert_eq!(
        buf.len(),
        col.payload_bytes(),
        "encoding {} serialized {} bytes but payload_bytes() promised {}",
        col.encoding_byte(),
        buf.len(),
        col.payload_bytes()
    );
    Ok(buf)
}

// ── tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::partition::MicroPartition;

    fn simple_partition() -> MicroPartition {
        let mut p = MicroPartition::new(2, 7);
        p.push_column("pk", ColumnValues::Raw(vec![1, 2, 3, 4, 5]))
            .unwrap();
        p.push_column("status", ColumnValues::auto_encode(vec![0, 1, 0, 2, 1]))
            .unwrap();
        p
    }

    #[test]
    fn write_produces_valid_magic() {
        let p = simple_partition();
        let mut buf: Vec<u8> = Vec::new();
        write_partition(&mut buf, &p).unwrap();
        assert_eq!(&buf[0..8], b"VVSST001");
    }

    #[test]
    fn write_empty_partition_errors() {
        let p = MicroPartition::new(0, 0);
        let mut buf: Vec<u8> = Vec::new();
        let result = write_partition(&mut buf, &p);
        assert!(matches!(result, Err(StorageError::EmptyPartition)));
    }

    #[test]
    fn header_crc_is_correct() {
        let p = simple_partition();
        let mut buf: Vec<u8> = Vec::new();
        write_partition(&mut buf, &p).unwrap();
        let stored = u32::from_le_bytes(buf[28..32].try_into().unwrap());
        let computed = crc32fast::hash(&buf[0..28]);
        assert_eq!(stored, computed);
    }

    #[test]
    fn file_crc_is_correct() {
        let p = simple_partition();
        let mut buf: Vec<u8> = Vec::new();
        write_partition(&mut buf, &p).unwrap();
        let len = buf.len();
        let stored = u32::from_le_bytes(buf[len - 4..].try_into().unwrap());
        let computed = crc32fast::hash(&buf[..len - 4]);
        assert_eq!(stored, computed);
    }
}
