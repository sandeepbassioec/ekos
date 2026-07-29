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
    format::{COL_META_SIZE, ENCODING_DICT, ENCODING_RAW, HEADER_FIXED_SIZE, MAGIC, VERSION},
    partition::MicroPartition,
};

/// Serialize `partition` into `writer` in `.vvsst` format.
pub fn write_partition<W: Write>(writer: &mut W, partition: &MicroPartition) -> StorageResult<()> {
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
        data_offset: u64,
        data_byte_len: u64,
        dict_offset: u64,
        dict_byte_len: u64,
        zone_min: u64,
        zone_max: u64,
    }

    let mut layouts: Vec<ColLayout> = Vec::with_capacity(partition.col_count());
    let mut cursor = data_start;
    for col in &partition.columns {
        let (zone_min, zone_max) = col.zone_min_max();
        let lo = match col {
            ColumnValues::Raw(_) => {
                let dbl = row_count as u64 * 8;
                let lo = ColLayout {
                    encoding: ENCODING_RAW,
                    data_offset: cursor,
                    data_byte_len: dbl,
                    dict_offset: 0,
                    dict_byte_len: 0,
                    zone_min,
                    zone_max,
                };
                cursor += dbl;
                lo
            }
            ColumnValues::DictEncoded { dict, .. } => {
                let dict_bytes = dict.len() as u64 * 8;
                let idx_bytes = row_count as u64 * 4;
                let total = dict_bytes + idx_bytes;
                let lo = ColLayout {
                    encoding: ENCODING_DICT,
                    data_offset: cursor,
                    data_byte_len: total,
                    dict_offset: cursor,
                    dict_byte_len: dict_bytes,
                    zone_min,
                    zone_max,
                };
                cursor += total;
                lo
            }
        };
        layouts.push(lo);
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
        buf.extend_from_slice(&[0u8; 7]); // reserved
        buf.write_u64::<LittleEndian>(lo.data_offset)?;
        buf.write_u64::<LittleEndian>(lo.data_byte_len)?;
        buf.write_u64::<LittleEndian>(lo.dict_offset)?;
        buf.write_u64::<LittleEndian>(lo.dict_byte_len)?;
        buf.write_u64::<LittleEndian>(lo.zone_min)?;
        buf.write_u64::<LittleEndian>(lo.zone_max)?;
    }

    // ── Step 6: column data ───────────────────────────────────────────────────
    for col in &partition.columns {
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
        }
    }

    // ── Step 7: file CRC footer ───────────────────────────────────────────────
    let file_crc = crc32fast::hash(&buf);
    buf.write_u32::<LittleEndian>(file_crc)?;

    writer.write_all(&buf)?;
    Ok(())
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
