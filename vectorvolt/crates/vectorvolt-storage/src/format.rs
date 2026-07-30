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
//! │     pad[6]                               │
//! │     data_offset[8]  data_byte_len[8]     │
//! │     dict_offset[8]  dict_byte_len[8]     │
//! │     zone_min[8]     zone_max[8]          │
//! ├──────────────────────────────────────────┤
//! │ Data Section  (variable)                 │
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
//! `data_offset` and `dict_offset` are **absolute file offsets**.

pub const MAGIC: &[u8; 8] = b"VVSST001";

/// Format version.
///
/// | Version | Change |
/// |---|---|
/// | 1 | Raw + dictionary encodings |
/// | 2 | Added RLE, frame-of-reference, blocked-frame; `bit_width` in column metadata byte 1 |
///
/// Version 2 is backward-compatible *for readers*: byte 1 was previously part of
/// the 7 reserved bytes and always written as zero, and the three encodings that
/// existed in v1 use `bit_width = 0`. So a v1 file parses correctly under v2
/// rules. The reverse is not true — a v1 reader hits `UnknownEncoding` on a v2
/// file, which is the intended loud failure. The reader accepts both versions.
pub const VERSION: u32 = 2;

/// Oldest format version this build can read.
pub const MIN_READABLE_VERSION: u32 = 1;

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
