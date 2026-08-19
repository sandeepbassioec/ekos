//! Mutation types and their WAL payload encoding.
//!
//! Binary layout (little-endian throughout):
//!
//! Insert:    op=0x00 | pk u64 | value_count u32 | values [u64; value_count]
//! Delete:    op=0x01 | pk u64
//! DdlCreate: op=0x10 | name_len u32 | name bytes (UTF-8)
//! DdlDrop:   op=0x11 | name_len u32 | name bytes (UTF-8)

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use std::io::{Cursor, Read};

use crate::error::{TxnError, TxnResult};

pub const OP_INSERT: u8 = 0x00;
pub const OP_DELETE: u8 = 0x01;
pub const OP_DDL_CREATE: u8 = 0x10;
pub const OP_DDL_DROP: u8 = 0x11;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Mutation {
    Insert { pk: u64, values: Vec<u64> },
    Delete { pk: u64 },
    DdlCreate { table_name: String },
    DdlDrop { table_name: String },
}

impl Mutation {
    pub fn pk(&self) -> u64 {
        match self {
            Mutation::Insert { pk, .. } | Mutation::Delete { pk } => *pk,
            Mutation::DdlCreate { .. } | Mutation::DdlDrop { .. } => 0,
        }
    }

    pub fn encode(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        match self {
            Mutation::Insert { pk, values } => {
                buf.write_u8(OP_INSERT).unwrap();
                buf.write_u64::<LittleEndian>(*pk).unwrap();
                buf.write_u32::<LittleEndian>(values.len() as u32).unwrap();
                for &v in values {
                    buf.write_u64::<LittleEndian>(v).unwrap();
                }
            }
            Mutation::Delete { pk } => {
                buf.write_u8(OP_DELETE).unwrap();
                buf.write_u64::<LittleEndian>(*pk).unwrap();
            }
            Mutation::DdlCreate { table_name } | Mutation::DdlDrop { table_name } => {
                let op = if matches!(self, Mutation::DdlCreate { .. }) {
                    OP_DDL_CREATE
                } else {
                    OP_DDL_DROP
                };
                buf.write_u8(op).unwrap();
                let bytes = table_name.as_bytes();
                buf.write_u32::<LittleEndian>(bytes.len() as u32).unwrap();
                buf.extend_from_slice(bytes);
            }
        }
        buf
    }

    pub fn decode(bytes: &[u8]) -> TxnResult<Self> {
        let mut cur = Cursor::new(bytes);
        let op = cur
            .read_u8()
            .map_err(|_| TxnError::CorruptPayload("truncated op byte".into()))?;

        match op {
            OP_INSERT => {
                let pk = cur
                    .read_u64::<LittleEndian>()
                    .map_err(|_| TxnError::CorruptPayload("truncated pk".into()))?;
                let count = cur
                    .read_u32::<LittleEndian>()
                    .map_err(|_| TxnError::CorruptPayload("truncated value_count".into()))?
                    as usize;
                let mut values = Vec::with_capacity(count);
                for i in 0..count {
                    let v = cur.read_u64::<LittleEndian>().map_err(|_| {
                        TxnError::CorruptPayload(format!("truncated value at index {i}"))
                    })?;
                    values.push(v);
                }
                Ok(Mutation::Insert { pk, values })
            }
            OP_DELETE => {
                let pk = cur
                    .read_u64::<LittleEndian>()
                    .map_err(|_| TxnError::CorruptPayload("truncated pk".into()))?;
                Ok(Mutation::Delete { pk })
            }
            OP_DDL_CREATE | OP_DDL_DROP => {
                let name_len = cur
                    .read_u32::<LittleEndian>()
                    .map_err(|_| TxnError::CorruptPayload("truncated DDL name_len".into()))?
                    as usize;
                let mut name_bytes = vec![0u8; name_len];
                cur.read_exact(&mut name_bytes)
                    .map_err(|_| TxnError::CorruptPayload("truncated DDL name".into()))?;
                let table_name = String::from_utf8(name_bytes)
                    .map_err(|_| TxnError::CorruptPayload("DDL name not valid UTF-8".into()))?;
                if op == OP_DDL_CREATE {
                    Ok(Mutation::DdlCreate { table_name })
                } else {
                    Ok(Mutation::DdlDrop { table_name })
                }
            }
            b => Err(TxnError::CorruptPayload(format!(
                "unknown op byte {b:#04x}"
            ))),
        }
    }
}

// ── tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert_round_trip() {
        let m = Mutation::Insert {
            pk: 42,
            values: vec![1, 2, 3],
        };
        assert_eq!(Mutation::decode(&m.encode()).unwrap(), m);
    }

    #[test]
    fn delete_round_trip() {
        let m = Mutation::Delete { pk: 99 };
        assert_eq!(Mutation::decode(&m.encode()).unwrap(), m);
    }

    #[test]
    fn insert_zero_values() {
        let m = Mutation::Insert {
            pk: 0,
            values: vec![],
        };
        assert_eq!(Mutation::decode(&m.encode()).unwrap(), m);
    }

    #[test]
    fn corrupt_payload_short() {
        assert!(Mutation::decode(&[]).is_err());
        assert!(Mutation::decode(&[OP_INSERT]).is_err()); // missing pk
    }

    #[test]
    fn unknown_op() {
        let mut buf = vec![0xFF];
        buf.extend_from_slice(&99u64.to_le_bytes());
        assert!(Mutation::decode(&buf).is_err());
    }
}
