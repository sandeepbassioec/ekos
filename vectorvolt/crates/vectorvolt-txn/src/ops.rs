//! Mutation types and their WAL payload encoding.
//!
//! Binary layout (little-endian throughout):
//!
//! Insert:     op=0x00 | pk u64 | value_count u32 | values [u64; value_count]
//! Delete:     op=0x01 | pk u64
//! Upsert:     op=0x02 | pk u64 | value_count u32 | values [u64; value_count]
//! DdlCreate:  op=0x10 | name_len u32 | name bytes (UTF-8)
//! DdlDrop:    op=0x11 | name_len u32 | name bytes (UTF-8)
//! DdlAlter:   op=0x12 | name_len u32 | name bytes | add_count u32 | [col_len u32 | col bytes]* | drop_count u32 | [col_len u32 | col bytes]*
//! InsertBatch: op=0x03 | row_count u32 | [(pk u64 | value_count u32 | values [u64; value_count])*]
//! DeleteBatch: op=0x04 | pk_count u32 | [pk u64]*
//! UpsertBatch: op=0x05 | row_count u32 | [(pk u64 | value_count u32 | values [u64; value_count])*]
//! Checkpoint:  op=0x06 | name_len u32 | name bytes (UTF-8) | lsn u64 | timestamp u64
//! Update:      op=0x07 | pk u64 | value_count u32 | values [u64; value_count]
//! TxnBatch:    op=0x08 | op_count u32 | [(pk u64 | kind u8 (0=put, 1=delete) | [value_count u32 | values]?)*]

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
pub const OP_DELETE_BATCH: u8 = 0x04;
pub const OP_UPSERT_BATCH: u8 = 0x05;
/// Typed checkpoint: records that all rows up to `lsn` are safe in SST files.
pub const OP_CHECKPOINT: u8 = 0x06;
/// In-place update: tombstones the old physical row and writes new column values
/// under the same PK without orphaning the old storage (VV-TXN-004.6).
pub const OP_UPDATE: u8 = 0x07;
pub const OP_TXN_BATCH: u8 = 0x08;

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
    DeleteBatch {
        pks: Vec<u64>,
    },
    UpsertBatch {
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
    /// Durability checkpoint: all rows up to `lsn` are already in SST files.
    /// Replaying a Checkpoint during WAL recovery is a no-op; it exists so the
    /// WAL contains a precise record of the last safe compaction boundary.
    Checkpoint {
        table_name: String,
        lsn: u64,
        timestamp: u64,
    },
    /// Atomic update: tombstones the old physical row and writes new column
    /// values.  Replayed as delete + insert during WAL recovery so storage
    /// orphans from the old row are never visible after a restart.
    Update {
        pk: u64,
        values: Vec<u64>,
    },
    /// A committed transaction: every put (`Some(values)`) and delete
    /// (`None`) it made, as ONE frame. The frame is checksummed as a unit, so
    /// after a crash it is either entirely present or entirely absent — the
    /// all-or-nothing property a commit needs. Ops are final states, which
    /// also makes re-applying one idempotent.
    TxnBatch {
        ops: Vec<(u64, Option<Vec<u64>>)>,
    },
}

impl Mutation {
    pub fn pk(&self) -> u64 {
        match self {
            Mutation::Insert { pk, .. } | Mutation::Delete { pk } | Mutation::Upsert { pk, .. } => {
                *pk
            }
            Mutation::InsertBatch { rows } => rows.first().map(|(pk, _)| *pk).unwrap_or(0),
            Mutation::DeleteBatch { pks } => pks.first().copied().unwrap_or(0),
            Mutation::UpsertBatch { rows } => rows.first().map(|(pk, _)| *pk).unwrap_or(0),
            Mutation::DdlCreate { .. }
            | Mutation::DdlDrop { .. }
            | Mutation::DdlAlter { .. }
            | Mutation::Checkpoint { .. } => 0,
            Mutation::Update { pk, .. } => *pk,
            Mutation::TxnBatch { ops } => ops.first().map(|(pk, _)| *pk).unwrap_or(0),
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
            Mutation::DeleteBatch { pks } => {
                buf.write_u8(OP_DELETE_BATCH).unwrap();
                buf.write_u32::<LittleEndian>(pks.len() as u32).unwrap();
                for &pk in pks {
                    buf.write_u64::<LittleEndian>(pk).unwrap();
                }
            }
            Mutation::UpsertBatch { rows } => {
                buf.write_u8(OP_UPSERT_BATCH).unwrap();
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
            Mutation::Checkpoint {
                table_name,
                lsn,
                timestamp,
            } => {
                buf.write_u8(OP_CHECKPOINT).unwrap();
                let name_bytes = table_name.as_bytes();
                buf.write_u32::<LittleEndian>(name_bytes.len() as u32)
                    .unwrap();
                buf.extend_from_slice(name_bytes);
                buf.write_u64::<LittleEndian>(*lsn).unwrap();
                buf.write_u64::<LittleEndian>(*timestamp).unwrap();
            }
            Mutation::Update { pk, values } => {
                buf.write_u8(OP_UPDATE).unwrap();
                buf.write_u64::<LittleEndian>(*pk).unwrap();
                buf.write_u32::<LittleEndian>(values.len() as u32).unwrap();
                for &v in values {
                    buf.write_u64::<LittleEndian>(v).unwrap();
                }
            }
            Mutation::TxnBatch { ops } => {
                buf.write_u8(OP_TXN_BATCH).unwrap();
                buf.write_u32::<LittleEndian>(ops.len() as u32).unwrap();
                for (pk, op) in ops {
                    buf.write_u64::<LittleEndian>(*pk).unwrap();
                    match op {
                        Some(values) => {
                            buf.write_u8(0).unwrap();
                            buf.write_u32::<LittleEndian>(values.len() as u32).unwrap();
                            for &v in values {
                                buf.write_u64::<LittleEndian>(v).unwrap();
                            }
                        }
                        None => buf.write_u8(1).unwrap(),
                    }
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
            OP_UPSERT_BATCH => {
                let row_count = cur.read_u32::<LittleEndian>().map_err(|_| {
                    TxnError::CorruptPayload("truncated upsert_batch row_count".into())
                })? as usize;
                let mut rows = Vec::with_capacity(row_count.min(65536));
                for i in 0..row_count {
                    let pk = cur.read_u64::<LittleEndian>().map_err(|_| {
                        TxnError::CorruptPayload(format!("truncated upsert_batch pk at row {i}"))
                    })?;
                    let count = cur.read_u32::<LittleEndian>().map_err(|_| {
                        TxnError::CorruptPayload(format!(
                            "truncated upsert_batch value_count at row {i}"
                        ))
                    })? as usize;
                    let mut values = Vec::with_capacity(count.min(65536));
                    for j in 0..count {
                        let v = cur.read_u64::<LittleEndian>().map_err(|_| {
                            TxnError::CorruptPayload(format!(
                                "truncated upsert_batch value at row {i} col {j}"
                            ))
                        })?;
                        values.push(v);
                    }
                    rows.push((pk, values));
                }
                Ok(Mutation::UpsertBatch { rows })
            }
            OP_DELETE_BATCH => {
                let pk_count = cur.read_u32::<LittleEndian>().map_err(|_| {
                    TxnError::CorruptPayload("truncated delete_batch pk_count".into())
                })? as usize;
                let mut pks = Vec::with_capacity(pk_count.min(65536));
                for i in 0..pk_count {
                    let pk = cur.read_u64::<LittleEndian>().map_err(|_| {
                        TxnError::CorruptPayload(format!("truncated delete_batch pk at index {i}"))
                    })?;
                    pks.push(pk);
                }
                Ok(Mutation::DeleteBatch { pks })
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
            OP_CHECKPOINT => {
                let name_len = cur
                    .read_u32::<LittleEndian>()
                    .map_err(|_| TxnError::CorruptPayload("truncated checkpoint name_len".into()))?
                    as usize;
                let mut name_bytes = vec![0u8; name_len];
                cur.read_exact(&mut name_bytes)
                    .map_err(|_| TxnError::CorruptPayload("truncated checkpoint name".into()))?;
                let table_name = String::from_utf8(name_bytes).map_err(|_| {
                    TxnError::CorruptPayload("checkpoint name not valid UTF-8".into())
                })?;
                let lsn = cur
                    .read_u64::<LittleEndian>()
                    .map_err(|_| TxnError::CorruptPayload("truncated checkpoint lsn".into()))?;
                let timestamp = cur.read_u64::<LittleEndian>().map_err(|_| {
                    TxnError::CorruptPayload("truncated checkpoint timestamp".into())
                })?;
                Ok(Mutation::Checkpoint {
                    table_name,
                    lsn,
                    timestamp,
                })
            }
            OP_UPDATE => {
                let pk = cur
                    .read_u64::<LittleEndian>()
                    .map_err(|_| TxnError::CorruptPayload("truncated update pk".into()))?;
                let count = cur
                    .read_u32::<LittleEndian>()
                    .map_err(|_| TxnError::CorruptPayload("truncated update value_count".into()))?
                    as usize;
                let mut values = Vec::with_capacity(count);
                for i in 0..count {
                    let v = cur.read_u64::<LittleEndian>().map_err(|_| {
                        TxnError::CorruptPayload(format!("truncated update value at index {i}"))
                    })?;
                    values.push(v);
                }
                Ok(Mutation::Update { pk, values })
            }
            OP_TXN_BATCH => {
                let corrupt =
                    |what: &str| TxnError::CorruptPayload(format!("truncated txn batch {what}"));
                let count = cur
                    .read_u32::<LittleEndian>()
                    .map_err(|_| corrupt("op_count"))? as usize;
                let mut ops = Vec::with_capacity(count.min(1 << 16));
                for _ in 0..count {
                    let pk = cur.read_u64::<LittleEndian>().map_err(|_| corrupt("pk"))?;
                    let kind = cur.read_u8().map_err(|_| corrupt("kind"))?;
                    let op = match kind {
                        0 => {
                            let n = cur
                                .read_u32::<LittleEndian>()
                                .map_err(|_| corrupt("value_count"))?
                                as usize;
                            let mut values = Vec::with_capacity(n.min(1 << 16));
                            for _ in 0..n {
                                values.push(
                                    cur.read_u64::<LittleEndian>()
                                        .map_err(|_| corrupt("value"))?,
                                );
                            }
                            Some(values)
                        }
                        1 => None,
                        k => {
                            return Err(TxnError::CorruptPayload(format!(
                                "txn batch op kind {k} is not put/delete"
                            )));
                        }
                    };
                    ops.push((pk, op));
                }
                Ok(Mutation::TxnBatch { ops })
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
    fn txn_batch_round_trips() {
        let m = Mutation::TxnBatch {
            ops: vec![(1, Some(vec![1, 2, 3])), (2, None), (3, Some(vec![]))],
        };
        assert_eq!(Mutation::decode(&m.encode()).unwrap(), m);
    }

    #[test]
    fn truncated_txn_batch_is_rejected() {
        let m = Mutation::TxnBatch {
            ops: vec![(1, Some(vec![1, 2, 3])), (2, None)],
        };
        let bytes = m.encode();
        assert!(Mutation::decode(&bytes[..bytes.len() - 3]).is_err());
    }

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

    #[test]
    fn checkpoint_round_trip() {
        let m = Mutation::Checkpoint {
            table_name: "orders".to_string(),
            lsn: 4242,
            timestamp: 1_700_000_000_000,
        };
        assert_eq!(Mutation::decode(&m.encode()).unwrap(), m);
    }

    #[test]
    fn checkpoint_pk_is_zero() {
        let m = Mutation::Checkpoint {
            table_name: "t".to_string(),
            lsn: 1,
            timestamp: 2,
        };
        assert_eq!(m.pk(), 0);
    }

    #[test]
    fn update_round_trip() {
        let m = Mutation::Update {
            pk: 7,
            values: vec![10, 20, 30],
        };
        assert_eq!(Mutation::decode(&m.encode()).unwrap(), m);
    }

    #[test]
    fn update_zero_values() {
        let m = Mutation::Update {
            pk: 0,
            values: vec![],
        };
        assert_eq!(Mutation::decode(&m.encode()).unwrap(), m);
    }

    #[test]
    fn update_pk_preserved() {
        let m = Mutation::Update {
            pk: 999,
            values: vec![1],
        };
        assert_eq!(m.pk(), 999);
    }

    #[test]
    fn checkpoint_empty_name() {
        let m = Mutation::Checkpoint {
            table_name: String::new(),
            lsn: 0,
            timestamp: 0,
        };
        assert_eq!(Mutation::decode(&m.encode()).unwrap(), m);
    }
}
