//! Mutation types and their WAL payload encoding.
//!
//! Binary layout (little-endian throughout):
//!
//! Insert:  op=0x00 | pk u64 | value_count u32 | values [u64; value_count]
//! Delete:  op=0x01 | pk u64

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use std::io::Cursor;

use crate::error::{TxnError, TxnResult};

pub const OP_INSERT: u8 = 0x00;
pub const OP_DELETE: u8 = 0x01;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Mutation {
    Insert { pk: u64, values: Vec<u64> },
    Delete { pk: u64 },
}

impl Mutation {
    pub fn pk(&self) -> u64 {
        match self {
            Mutation::Insert { pk, .. } | Mutation::Delete { pk } => *pk,
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
        }
        buf
    }

    pub fn decode(bytes: &[u8]) -> TxnResult<Self> {
        let mut cur = Cursor::new(bytes);
        let op = cur
            .read_u8()
            .map_err(|_| TxnError::CorruptPayload("truncated op byte".into()))?;
        let pk = cur
            .read_u64::<LittleEndian>()
            .map_err(|_| TxnError::CorruptPayload("truncated pk".into()))?;

        match op {
            OP_INSERT => {
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
            OP_DELETE => Ok(Mutation::Delete { pk }),
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
