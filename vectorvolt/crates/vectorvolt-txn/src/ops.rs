//! Mutation types and their WAL payload encoding.
//!
//! Binary layout (little-endian throughout):
//!
//! Insert:    op=0x00 | pk u64 | value_count u32 | values [u64; value_count]
//! Delete:    op=0x01 | pk u64
//! Upsert:    op=0x02 | pk u64 | value_count u32 | values [u64; value_count]
//! DdlCreate: op=0x10 | name_len u32 | name bytes (UTF-8)
//! DdlDrop:   op=0x11 | name_len u32 | name bytes (UTF-8)
//! DdlAlter:  op=0x12 | name_len u32 | name bytes | add_count u32 | [col_len u32 | col bytes]* | drop_count u32 | [col_len u32 | col bytes]*
//! InsertBatch: op=0x03 | row_count u32 | [(pk u64 | value_count u32 | values [u64; value_count])*]

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use std::io::{Cursor, Read};

use crate::error::{TxnError, TxnResult};

pub const OP_INSERT: u8 = 0x00;
pub const OP_DELETE: u8 = 0x01;
pub const OP_UPSERT: u8 = 0x02;
pub const OP_DDL_CREATE: u8 = 0x10;
pub const OP_DDL_DROP: u8 = 0x11;
pub const OP_DDL_ALTER: u8 = 0x12;
pub const OP_INSERT_BATCH: u8 = 0x03;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Mutation {
    Insert {
        pk: u64,
        values: Vec<u64>,
    },
    Delete {
        pk: u64,
    },
    Upsert {
        pk: u64,
        values: Vec<u64>,
    },
    InsertBatch {
        rows: Vec<(u64, Vec<u64>)>,
    },
    DdlCreate {
        table_name: String,
    },
    DdlDrop {
        table_name: String,
    },
    DdlAlter {
        table_name: String,
        add_columns: Vec<String>,
        drop_columns: Vec<String>,
    },
}

impl Mutation {
    pub fn pk(&self) -> u64 {
        match self {
            Mutation::Insert { pk, .. } | Mutation::Delete { pk } | Mutation::Upsert { pk, .. } => {
                *pk
            }
            Mutation::InsertBatch { rows } => rows.first().map(|(pk, _)| *pk).unwrap_or(0),
            Mutation::DdlCreate { .. } | Mutation::DdlDrop { .. } | Mutation::DdlAlter { .. } => 0,
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
            Mutation::Upsert { pk, values } => {
                buf.write_u8(OP_UPSERT).unwrap();
                buf.write_u64::<LittleEndian>(*pk).unwrap();
                buf.write_u32::<LittleEndian>(values.len() as u32).unwrap();
                for &v in values {
                    buf.write_u64::<LittleEndian>(v).unwrap();
                }
            }
            Mutation::InsertBatch { rows } => {
                buf.write_u8(OP_INSERT_BATCH).unwrap();
                buf.write_u32::<LittleEndian>(rows.len() as u32).unwrap();
                for (pk, values) in rows {
                    buf.write_u64::<LittleEndian>(*pk).unwrap();
                    buf.write_u32::<LittleEndian>(values.len() as u32).unwrap();
                    for &v in values {
                        buf.write_u64::<LittleEndian>(v).unwrap();
                    }
                }
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
            Mutation::DdlAlter {
                table_name,
                add_columns,
                drop_columns,
            } => {
                buf.write_u8(OP_DDL_ALTER).unwrap();
                let name_bytes = table_name.as_bytes();
                buf.write_u32::<LittleEndian>(name_bytes.len() as u32)
                    .unwrap();
                buf.extend_from_slice(name_bytes);
                buf.write_u32::<LittleEndian>(add_columns.len() as u32)
                    .unwrap();
                for col in add_columns {
                    let cb = col.as_bytes();
                    buf.write_u32::<LittleEndian>(cb.len() as u32).unwrap();
                    buf.extend_from_slice(cb);
                }
                buf.write_u32::<LittleEndian>(drop_columns.len() as u32)
                    .unwrap();
                for col in drop_columns {
                    let cb = col.as_bytes();
                    buf.write_u32::<LittleEndian>(cb.len() as u32).unwrap();
                    buf.extend_from_slice(cb);
                }
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
            OP_UPSERT => {
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
                        TxnError::CorruptPayload(format!("truncated upsert value at index {i}"))
                    })?;
                    values.push(v);
                }
                Ok(Mutation::Upsert { pk, values })
            }
            OP_INSERT_BATCH => {
                let row_count = cur
                    .read_u32::<LittleEndian>()
                    .map_err(|_| TxnError::CorruptPayload("truncated batch row_count".into()))?
                    as usize;
                let mut rows = Vec::with_capacity(row_count.min(65536));
                for i in 0..row_count {
                    let pk = cur.read_u64::<LittleEndian>().map_err(|_| {
                        TxnError::CorruptPayload(format!("truncated batch pk at row {i}"))
                    })?;
                    let count = cur.read_u32::<LittleEndian>().map_err(|_| {
                        TxnError::CorruptPayload(format!("truncated batch value_count at row {i}"))
                    })? as usize;
                    let mut values = Vec::with_capacity(count.min(65536));
                    for j in 0..count {
                        let v = cur.read_u64::<LittleEndian>().map_err(|_| {
                            TxnError::CorruptPayload(format!(
                                "truncated batch value at row {i} col {j}"
                            ))
                        })?;
                        values.push(v);
                    }
                    rows.push((pk, values));
                }
                Ok(Mutation::InsertBatch { rows })
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
            OP_DDL_ALTER => {
                let name_len = cur
                    .read_u32::<LittleEndian>()
                    .map_err(|_| TxnError::CorruptPayload("truncated alter name_len".into()))?
                    as usize;
                let mut name_bytes = vec![0u8; name_len];
                cur.read_exact(&mut name_bytes)
                    .map_err(|_| TxnError::CorruptPayload("truncated alter name".into()))?;
                let table_name = String::from_utf8(name_bytes)
                    .map_err(|_| TxnError::CorruptPayload("alter name not valid UTF-8".into()))?;
                let add_count = cur
                    .read_u32::<LittleEndian>()
                    .map_err(|_| TxnError::CorruptPayload("truncated alter add_count".into()))?
                    as usize;
                let mut add_columns = Vec::with_capacity(add_count.min(1024));
                for _ in 0..add_count {
                    let clen = cur
                        .read_u32::<LittleEndian>()
                        .map_err(|_| TxnError::CorruptPayload("truncated add col_len".into()))?
                        as usize;
                    let mut cbytes = vec![0u8; clen];
                    cur.read_exact(&mut cbytes)
                        .map_err(|_| TxnError::CorruptPayload("truncated add col".into()))?;
                    add_columns.push(
                        String::from_utf8(cbytes)
                            .map_err(|_| TxnError::CorruptPayload("add col not UTF-8".into()))?,
                    );
                }
                let drop_count = cur
                    .read_u32::<LittleEndian>()
                    .map_err(|_| TxnError::CorruptPayload("truncated alter drop_count".into()))?
                    as usize;
                let mut drop_columns = Vec::with_capacity(drop_count.min(1024));
                for _ in 0..drop_count {
                    let clen = cur
                        .read_u32::<LittleEndian>()
                        .map_err(|_| TxnError::CorruptPayload("truncated drop col_len".into()))?
                        as usize;
                    let mut cbytes = vec![0u8; clen];
                    cur.read_exact(&mut cbytes)
                        .map_err(|_| TxnError::CorruptPayload("truncated drop col".into()))?;
                    drop_columns.push(
                        String::from_utf8(cbytes)
                            .map_err(|_| TxnError::CorruptPayload("drop col not UTF-8".into()))?,
                    );
                }
                Ok(Mutation::DdlAlter {
                    table_name,
                    add_columns,
                    drop_columns,
                })
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
