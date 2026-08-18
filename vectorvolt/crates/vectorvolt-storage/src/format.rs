//! Binary format constants for the `.vvsst` (VectorVolt SSTable) file format.
//!
//! ## Layout
//!
//! ```text
//! ┌──────────────────────────────────────────┐
//! │ Fixed Header  32 bytes                   │
//! │   magic[8]  version[4]  file_id[2]       │
//! │   partition_id[2]  row_count[4]          │
//! │   col_count[4]  schema_len[4]            │
//! │   header_crc32[4]                        │
//! ├──────────────────────────────────────────┤
//! │ Schema Section  (schema_len bytes)       │
//! │   per column: name_len[2] + name_bytes   │
//! ├──────────────────────────────────────────┤
//! │ Column Metadata  (col_count × 56 bytes)  │
//! │   per column: encoding[1] bit_width[1]   │
//! │     compression[1]  pad[5]               │
//! │     data_offset[8]  data_byte_len[8]     │
//! │     uncompressed_len[8]                  │
//! │     first_section_len[8]                 │
//! │     zone_min[8]     zone_max[8]          │
//! ├──────────────────────────────────────────┤
//! │ Data Section — one payload per column,   │
//! │ optionally LZ4-compressed as a unit.     │
//! │ Offsets below are RELATIVE to payload 0. │
//! │   Raw:     u64[row_count]                │
//! │   Dict:    u64[dict_size] ‖ u32[rows]    │
//! │   Rle:     u64[runs] ‖ u32[runs]         │
//! │   Frame:   u64 base ‖ u64[packed]        │
//! │   BlkFrame:u64[blocks] ‖ u64[packed]     │
//! ├──────────────────────────────────────────┤
//! │ Footer CRC  4 bytes                      │
//! └──────────────────────────────────────────┘
//! ```
//!
//! All multi-byte integers are **little-endian**.
//!
//! `data_offset` is an absolute file offset; everything *within* a payload is
//! relative to that payload start. That distinction is what lets a payload be
//! compressed: after decompression its sections exist only in a temporary buffer
//! and have no file address, so absolute inner offsets could not be expressed.
//! `first_section_len` is the boundary between the two arrays for the encodings
//! that have two, and `uncompressed_len` is the payload size after decompression
//! (equal to `data_byte_len` when stored verbatim).

pub const MAGIC: &[u8; 8] = b"VVSST001";

/// Format version.
///
/// | Version | Change |
/// |---|---|
/// | 1 | Raw + dictionary encodings |
/// | 2 | Added RLE, frame-of-reference, blocked-frame; bit_width in column metadata byte 1 |
/// | 3 | Optional LZ4 per column; section offsets became payload-relative |
///
/// **v3 is a breaking change and `MIN_READABLE_VERSION` says so.** v1 and v2
/// stored `dict_offset` as an absolute file offset; v3 replaced that slot with
/// `uncompressed_len` and made all inner section offsets payload-relative, which
/// a compressed payload requires. Reading a v2 file under v3 rules would treat a
/// large absolute offset as a decompressed length and produce garbage rather than
/// an error — silently. Rejecting the file is the only safe behaviour, so the
/// reader accepts v3 only.
///
/// This is acceptable *today* because no v1/v2 file exists outside tests; the
/// format has never shipped. Once it has, a conversion path is required rather
/// than a version bump.
pub const VERSION: u32 = 3;

/// Oldest format version this build can read.
pub const MIN_READABLE_VERSION: u32 = 3;

/// Column payload is stored verbatim.
pub const COMPRESSION_NONE: u8 = 0;

/// Column payload is LZ4 block-compressed (VV-STO-012).
///
/// ## Why this is opt-in and off by default
///
/// LZ4 is applied to the whole column payload, so reading **one** value requires
/// decompressing **all** of it. That converts `read_value` from O(1) — two
/// unaligned reads out of the mmap — into O(column). VectorVolt's read
/// architecture rests on that O(1); enabling compression by default would trade
/// away the single property the project is built around, to save space that the
/// encodings in [`crate::column`] already recover most of.
///
/// So compression is for **cold data**: partitions that are scanned or archived
/// rather than point-queried — the L3/L4 tiers in `VECTORVOLT_STORAGE_TIERING.md`.
/// [`crate::writer::WriteOptions`] makes the choice explicit per partition, and
/// [`crate::reader::PartitionReader::is_point_read_optimal`] lets a caller detect
/// a compressed partition rather than be surprised by its latency.
///
/// Block-level compression (compress each 64 KiB chunk separately, decompress
/// only the chunk a row lands in) would soften this to O(chunk). That is the
/// right eventual design and is deliberately not built yet — it needs a chunk
/// index in the column metadata, which is a second format change best made once
/// the tiering work defines what a cold partition actually looks like.
pub const COMPRESSION_LZ4: u8 = 1;

/// Column payload is Zstd-compressed (VV-STO-013).
///
/// Zstd offers a significantly better compression ratio than LZ4 at comparable
/// decode throughput for cold/archival partitions. The same O(column) point-read
/// caveat as [`COMPRESSION_LZ4`] applies — use only on scan-heavy or archival
/// partitions. Default compression level is 3 (Zstd default); higher levels
/// yield better ratio at the cost of encode time.
pub const COMPRESSION_ZSTD: u8 = 2;

/// Byte length of the fixed header.
pub const HEADER_FIXED_SIZE: usize = 32;

/// Byte length of one column-metadata entry.
///
/// Unchanged across v1→v2: the new `bit_width` field took one of the 7 reserved
/// bytes rather than growing the entry, so data offsets are computed identically
/// and no migration is needed.
pub const COL_META_SIZE: usize = 56;

/// Column data is stored as raw `u64` values.
///
/// Data section: `u64[row_count]`.
pub const ENCODING_RAW: u8 = 0;

/// Column data is stored as a dictionary of `u64` + `u32` indices.
///
/// Data section: `u64[dict_len] ‖ u32[row_count]`.
/// `dict_offset` = data start, `dict_byte_len` = `dict_len * 8`.
pub const ENCODING_DICT: u8 = 1;

/// Run-length encoded.
///
/// Data section: `u64[run_count] ‖ u32[run_count]` — run values then **exclusive
/// cumulative run ends**.
/// `dict_offset` = data start, `dict_byte_len` = `run_count * 8`.
///
/// Ends rather than lengths so a point lookup binary-searches instead of scanning.
pub const ENCODING_RLE: u8 = 2;

/// Frame of reference: one base for the whole column.
///
/// Data section: `u64 base ‖ u64[packed_words]`.
/// `bit_width` in column-metadata byte 1. `dict_offset`/`dict_byte_len` unused (0).
pub const ENCODING_FRAME: u8 = 3;

/// Blocked frame of reference: one base per `FRAME_BLOCK` rows.
///
/// Data section: `u64[block_count] ‖ u64[packed_words]`.
/// `dict_offset` = data start, `dict_byte_len` = `block_count * 8`.
/// `bit_width` in column-metadata byte 1.
///
/// This is the encoding used for sorted and monotonic columns in place of
/// successive-delta, which would make point lookups O(n). See
/// [`crate::column::ColumnValues`].
pub const ENCODING_BLOCKED_FRAME: u8 = 4;
