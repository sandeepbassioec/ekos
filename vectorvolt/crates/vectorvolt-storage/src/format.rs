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
//! │   per column: encoding[1] + pad[7]       │
//! │     data_offset[8]  data_byte_len[8]     │
//! │     dict_offset[8]  dict_byte_len[8]     │
//! │     zone_min[8]     zone_max[8]          │
//! ├──────────────────────────────────────────┤
//! │ Data Section  (variable)                 │
//! │   Raw:  u64[row_count]  (8 bytes/row)    │
//! │   Dict: u64[dict_size] ‖ u32[row_count]  │
//! ├──────────────────────────────────────────┤
//! │ Footer CRC  4 bytes                      │
//! └──────────────────────────────────────────┘
//! ```
//!
//! All multi-byte integers are **little-endian**.
//! `data_offset` and `dict_offset` are **absolute file offsets**.

pub const MAGIC: &[u8; 8] = b"VVSST001";
pub const VERSION: u32 = 1;

/// Byte length of the fixed header.
pub const HEADER_FIXED_SIZE: usize = 32;

/// Byte length of one column-metadata entry.
pub const COL_META_SIZE: usize = 56;

/// Column data is stored as raw `u64` values.
pub const ENCODING_RAW: u8 = 0;

/// Column data is stored as a dictionary of `u64` + `u32` indices.
pub const ENCODING_DICT: u8 = 1;
