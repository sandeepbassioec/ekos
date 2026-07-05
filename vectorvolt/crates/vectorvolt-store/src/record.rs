//! LogRecord — the unit written into the WAL for every KV mutation.
//!
//! Wire format (big-endian):
//!   op_tag   : u8       (0 = Put, 1 = Delete)
//!   key_len  : u32
//!   key      : [u8; key_len]
//!   val_len  : u32      (0 for Delete)
//!   val      : [u8; val_len]

use bytes::{Buf, BufMut, Bytes, BytesMut};

use crate::StoreError;

const OP_PUT: u8 = 0;
const OP_DELETE: u8 = 1;

/// A single WAL entry representing one KV mutation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LogRecord {
    Put { key: Bytes, value: Bytes },
    Delete { key: Bytes },
}

impl LogRecord {
    pub fn key(&self) -> &Bytes {
        match self {
            LogRecord::Put { key, .. } => key,
            LogRecord::Delete { key } => key,
        }
    }

    pub fn encode(&self) -> Bytes {
        let (op, key, val): (u8, &Bytes, &[u8]) = match self {
            LogRecord::Put { key, value } => (OP_PUT, key, value),
            LogRecord::Delete { key } => (OP_DELETE, key, b""),
        };

        let len = 1 + 4 + key.len() + 4 + val.len();
        let mut buf = BytesMut::with_capacity(len);
        buf.put_u8(op);
        buf.put_u32(key.len() as u32);
        buf.put_slice(key);
        buf.put_u32(val.len() as u32);
        buf.put_slice(val);
        buf.freeze()
    }

    pub fn decode(mut src: Bytes) -> Result<Self, StoreError> {
        if src.len() < 9 {
            return Err(StoreError::Corrupt("log record too short".into()));
        }

        let op = src.get_u8();
        let key_len = src.get_u32() as usize;

        if src.remaining() < key_len + 4 {
            return Err(StoreError::Corrupt("log record key truncated".into()));
        }
        let key = src.copy_to_bytes(key_len);
        let val_len = src.get_u32() as usize;

        if src.remaining() < val_len {
            return Err(StoreError::Corrupt("log record value truncated".into()));
        }
        let value = src.copy_to_bytes(val_len);

        match op {
            OP_PUT => Ok(LogRecord::Put { key, value }),
            OP_DELETE => Ok(LogRecord::Delete { key }),
            _ => Err(StoreError::Corrupt(format!("unknown op tag {op}"))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::Bytes;

    #[test]
    fn roundtrip_put() {
        let r = LogRecord::Put {
            key: Bytes::from("hello"),
            value: Bytes::from("world"),
        };
        assert_eq!(LogRecord::decode(r.encode()).unwrap(), r);
    }

    #[test]
    fn roundtrip_delete() {
        let r = LogRecord::Delete { key: Bytes::from("gone") };
        assert_eq!(LogRecord::decode(r.encode()).unwrap(), r);
    }
}
