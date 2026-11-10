//! Wire protocol: length-prefixed, CRC32-protected messages.
//!
//! ```text
//! magic u32 LE ("VRP1") | len u32 LE | body (len bytes: type u8 + fields) | crc32 u32 LE (of body)
//! ```
//!
//! WAL frame payloads travel exactly as stored in the leader's WAL: on an
//! encrypted table they are still AES-GCM ciphertext (bound to their sequence
//! by AAD), so replication never exposes row data in plaintext on the wire.

use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

use crate::error::{ReplResult, ReplicationError};

pub const MAGIC: u32 = 0x3150_5256; // "VRP1" little-endian
pub const PROTO_VERSION: u8 = 1;
/// Upper bound on one message body; larger lengths are treated as corruption.
pub const MAX_BODY: u32 = 64 * 1024 * 1024;

const T_HELLO: u8 = 1;
const T_WELCOME: u8 = 2;
const T_SNAP_ROWS: u8 = 3;
const T_SNAP_END: u8 = 4;
const T_FRAMES: u8 = 5;
const T_HEARTBEAT: u8 = 6;
const T_ACK: u8 = 7;
const T_ERROR: u8 = 8;

/// One WAL frame as shipped to a follower.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WireFrame {
    /// WAL sequence on the leader; this is the replication LSN.
    pub seq: u64,
    pub timestamp_ns: u64,
    /// `Some(key id)` when the source segment is encrypted.
    pub key_id: Option<[u8; 8]>,
    /// Payload as stored in the WAL (ciphertext when encrypted).
    pub payload: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Message {
    /// Follower -> leader: first message; `applied_lsn` is 0 for a new follower.
    Hello {
        version: u8,
        applied_lsn: u64,
    },
    /// Leader -> follower: stream plan. If `snapshot`, rows follow, then frames
    /// resume after `start_lsn`; otherwise frames resume after `applied_lsn`.
    Welcome {
        first_lsn: u64,
        tip: u64,
        snapshot: bool,
        start_lsn: u64,
    },
    SnapshotRows(Vec<(u64, Vec<u64>)>),
    SnapshotEnd {
        snapshot_lsn: u64,
    },
    Frames {
        tip: u64,
        frames: Vec<WireFrame>,
    },
    Heartbeat {
        tip: u64,
    },
    /// Follower -> leader.
    Ack {
        applied_lsn: u64,
        durable_lsn: u64,
    },
    Error(String),
}

// ── encoding ──────────────────────────────────────────────────────────────────

struct Wr(Vec<u8>);

impl Wr {
    fn u8(&mut self, v: u8) {
        self.0.push(v);
    }
    fn u32(&mut self, v: u32) {
        self.0.extend_from_slice(&v.to_le_bytes());
    }
    fn u64(&mut self, v: u64) {
        self.0.extend_from_slice(&v.to_le_bytes());
    }
    fn bytes(&mut self, b: &[u8]) {
        self.u32(b.len() as u32);
        self.0.extend_from_slice(b);
    }
}

struct Rd<'a> {
    buf: &'a [u8],
    pos: usize,
}

impl<'a> Rd<'a> {
    fn take(&mut self, n: usize) -> ReplResult<&'a [u8]> {
        let end = self
            .pos
            .checked_add(n)
            .filter(|&e| e <= self.buf.len())
            .ok_or_else(|| ReplicationError::Corrupt("truncated message".into()))?;
        let s = &self.buf[self.pos..end];
        self.pos = end;
        Ok(s)
    }
    fn u8(&mut self) -> ReplResult<u8> {
        Ok(self.take(1)?[0])
    }
    fn u32(&mut self) -> ReplResult<u32> {
        Ok(u32::from_le_bytes(self.take(4)?.try_into().unwrap()))
    }
    fn u64(&mut self) -> ReplResult<u64> {
        Ok(u64::from_le_bytes(self.take(8)?.try_into().unwrap()))
    }
    fn bytes(&mut self) -> ReplResult<&'a [u8]> {
        let n = self.u32()? as usize;
        self.take(n)
    }
    fn done(&self) -> ReplResult<()> {
        if self.pos == self.buf.len() {
            Ok(())
        } else {
            Err(ReplicationError::Corrupt(
                "trailing bytes in message".into(),
            ))
        }
    }
}

impl Message {
    fn encode_body(&self) -> Vec<u8> {
        let mut w = Wr(Vec::new());
        match self {
            Message::Hello {
                version,
                applied_lsn,
            } => {
                w.u8(T_HELLO);
                w.u8(*version);
                w.u64(*applied_lsn);
            }
            Message::Welcome {
                first_lsn,
                tip,
                snapshot,
                start_lsn,
            } => {
                w.u8(T_WELCOME);
                w.u64(*first_lsn);
                w.u64(*tip);
                w.u8(*snapshot as u8);
                w.u64(*start_lsn);
            }
            Message::SnapshotRows(rows) => {
                w.u8(T_SNAP_ROWS);
                w.u32(rows.len() as u32);
                for (pk, values) in rows {
                    w.u64(*pk);
                    w.u32(values.len() as u32);
                    for v in values {
                        w.u64(*v);
                    }
                }
            }
            Message::SnapshotEnd { snapshot_lsn } => {
                w.u8(T_SNAP_END);
                w.u64(*snapshot_lsn);
            }
            Message::Frames { tip, frames } => {
                w.u8(T_FRAMES);
                w.u64(*tip);
                w.u32(frames.len() as u32);
                for f in frames {
                    w.u64(f.seq);
                    w.u64(f.timestamp_ns);
                    match f.key_id {
                        Some(id) => {
                            w.u8(1);
                            w.0.extend_from_slice(&id);
                        }
                        None => w.u8(0),
                    }
                    w.bytes(&f.payload);
                }
            }
            Message::Heartbeat { tip } => {
                w.u8(T_HEARTBEAT);
                w.u64(*tip);
            }
            Message::Ack {
                applied_lsn,
                durable_lsn,
            } => {
                w.u8(T_ACK);
                w.u64(*applied_lsn);
                w.u64(*durable_lsn);
            }
            Message::Error(s) => {
                w.u8(T_ERROR);
                w.bytes(s.as_bytes());
            }
        }
        w.0
    }

    fn decode_body(body: &[u8]) -> ReplResult<Self> {
        let mut r = Rd { buf: body, pos: 0 };
        let msg = match r.u8()? {
            T_HELLO => Message::Hello {
                version: r.u8()?,
                applied_lsn: r.u64()?,
            },
            T_WELCOME => Message::Welcome {
                first_lsn: r.u64()?,
                tip: r.u64()?,
                snapshot: r.u8()? != 0,
                start_lsn: r.u64()?,
            },
            T_SNAP_ROWS => {
                let n = r.u32()? as usize;
                let mut rows = Vec::with_capacity(n.min(65_536));
                for _ in 0..n {
                    let pk = r.u64()?;
                    let c = r.u32()? as usize;
                    let mut values = Vec::with_capacity(c.min(4096));
                    for _ in 0..c {
                        values.push(r.u64()?);
                    }
                    rows.push((pk, values));
                }
                Message::SnapshotRows(rows)
            }
            T_SNAP_END => Message::SnapshotEnd {
                snapshot_lsn: r.u64()?,
            },
            T_FRAMES => {
                let tip = r.u64()?;
                let n = r.u32()? as usize;
                let mut frames = Vec::with_capacity(n.min(4096));
                for _ in 0..n {
                    let seq = r.u64()?;
                    let timestamp_ns = r.u64()?;
                    let key_id = match r.u8()? {
                        0 => None,
                        1 => Some(r.take(8)?.try_into().unwrap()),
                        _ => return Err(ReplicationError::Corrupt("bad key flag".into())),
                    };
                    let payload = r.bytes()?.to_vec();
                    frames.push(WireFrame {
                        seq,
                        timestamp_ns,
                        key_id,
                        payload,
                    });
                }
                Message::Frames { tip, frames }
            }
            T_HEARTBEAT => Message::Heartbeat { tip: r.u64()? },
            T_ACK => Message::Ack {
                applied_lsn: r.u64()?,
                durable_lsn: r.u64()?,
            },
            T_ERROR => Message::Error(String::from_utf8_lossy(r.bytes()?).into_owned()),
            t => {
                return Err(ReplicationError::Corrupt(format!(
                    "unknown message type {t}"
                )))
            }
        };
        r.done()?;
        Ok(msg)
    }

    /// Full wire bytes: magic, length, body, CRC.
    pub fn encode(&self) -> Vec<u8> {
        let body = self.encode_body();
        let mut out = Vec::with_capacity(body.len() + 12);
        out.extend_from_slice(&MAGIC.to_le_bytes());
        out.extend_from_slice(&(body.len() as u32).to_le_bytes());
        out.extend_from_slice(&body);
        out.extend_from_slice(&crc32fast::hash(&body).to_le_bytes());
        out
    }

    /// Decode one complete message from `bytes` (as produced by [`encode`](Self::encode)).
    pub fn decode(bytes: &[u8]) -> ReplResult<Self> {
        if bytes.len() < 12 {
            return Err(ReplicationError::Corrupt("short message".into()));
        }
        if u32::from_le_bytes(bytes[0..4].try_into().unwrap()) != MAGIC {
            return Err(ReplicationError::Corrupt("bad magic".into()));
        }
        let len = u32::from_le_bytes(bytes[4..8].try_into().unwrap()) as usize;
        if bytes.len() != 12 + len {
            return Err(ReplicationError::Corrupt("length mismatch".into()));
        }
        let body = &bytes[8..8 + len];
        let crc = u32::from_le_bytes(bytes[8 + len..].try_into().unwrap());
        if crc32fast::hash(body) != crc {
            return Err(ReplicationError::Corrupt("crc mismatch".into()));
        }
        Self::decode_body(body)
    }
}

/// Write one message; returns bytes written.
pub async fn write_message<W: AsyncWrite + Unpin>(w: &mut W, msg: &Message) -> ReplResult<usize> {
    let bytes = msg.encode();
    w.write_all(&bytes).await?;
    w.flush().await?;
    Ok(bytes.len())
}

/// Read and verify one message; returns it with the bytes consumed.
pub async fn read_message<R: AsyncRead + Unpin>(r: &mut R) -> ReplResult<(Message, usize)> {
    let mut head = [0u8; 8];
    r.read_exact(&mut head).await?;
    if u32::from_le_bytes(head[0..4].try_into().unwrap()) != MAGIC {
        return Err(ReplicationError::Corrupt("bad magic".into()));
    }
    let len = u32::from_le_bytes(head[4..8].try_into().unwrap());
    if len == 0 || len > MAX_BODY {
        return Err(ReplicationError::Corrupt(format!("bad length {len}")));
    }
    let mut body = vec![0u8; len as usize];
    r.read_exact(&mut body).await?;
    let mut crc = [0u8; 4];
    r.read_exact(&mut crc).await?;
    if crc32fast::hash(&body) != u32::from_le_bytes(crc) {
        return Err(ReplicationError::Corrupt("crc mismatch".into()));
    }
    Ok((Message::decode_body(&body)?, 12 + len as usize))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_and_detects_bit_flips() {
        let msg = Message::Frames {
            tip: 9,
            frames: vec![WireFrame {
                seq: 9,
                timestamp_ns: 5,
                key_id: Some([1; 8]),
                payload: vec![1, 2, 3],
            }],
        };
        let bytes = msg.encode();
        assert_eq!(Message::decode(&bytes).unwrap(), msg);
        for i in 0..bytes.len() {
            let mut bad = bytes.clone();
            bad[i] ^= 0x40;
            assert!(Message::decode(&bad).is_err(), "flip at {i} accepted");
        }
    }
}
