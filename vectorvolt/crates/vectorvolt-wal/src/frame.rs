//! WAL frame wire format.
//!
//! Each frame:
//!
//! ```text
//! ┌──────────────────────────────────────────────────────────────────┐
//! │  MAGIC       4 bytes  0x5657_414C ("VWAL")                       │
//! │  frame_len   4 bytes  total frame size (header + payload + tail) │
//! │  sequence    8 bytes  monotonic u64, 1-indexed                   │
//! │  timestamp   8 bytes  nanoseconds since Unix epoch               │
//! │  prev_hash  32 bytes  SHA3-256 of previous frame (0s at genesis) │
//! │  payload_type 1 byte  0=Data 1=Checkpoint 2=EndOfSegment         │
//! │  payload_len 4 bytes                                             │
//! │  payload     N bytes                                             │
//! │  frame_hash 32 bytes  SHA3-256(all bytes above)                  │
//! │  crc32c      4 bytes  CRC32C(all bytes above) — fast I/O guard   │
//! └──────────────────────────────────────────────────────────────────┘
//!
//! Overhead per frame: 4+4+8+8+32+1+4+32+4 = 97 bytes
//! ```

use crate::error::{WalError, WalResult};
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use sha3::{Digest, Sha3_256};
use std::io::Read;

pub const FRAME_MAGIC: u32 = 0x5657_414C; // "VWAL"
pub const FRAME_HEADER_SIZE: usize = 4 + 4 + 8 + 8 + 32 + 1 + 4; // 61 bytes
pub const FRAME_TAIL_SIZE: usize = 32 + 4; // hash + crc32 = 36 bytes
pub const FRAME_OVERHEAD: usize = FRAME_HEADER_SIZE + FRAME_TAIL_SIZE; // 97 bytes

pub const GENESIS_HASH: [u8; 32] = [0u8; 32];

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PayloadType {
    Data         = 0,
    Checkpoint   = 1,
    EndOfSegment = 2,
}

impl TryFrom<u8> for PayloadType {
    type Error = WalError;
    fn try_from(v: u8) -> WalResult<Self> {
        match v {
            0 => Ok(Self::Data),
            1 => Ok(Self::Checkpoint),
            2 => Ok(Self::EndOfSegment),
            _ => Err(WalError::CorruptFrame {
                segment: 0, offset: 0, reason: "unknown payload type",
            }),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Frame {
    pub sequence:     u64,
    pub timestamp_ns: u64,
    pub prev_hash:    [u8; 32],
    pub payload_type: PayloadType,
    pub payload:      Vec<u8>,
    /// SHA3-256 of everything before the tail; computed on write, verified on read.
    pub frame_hash:   [u8; 32],
}

impl Frame {
    /// Compute the frame_hash from scratch (does not use self.frame_hash).
    pub fn compute_hash(&self) -> [u8; 32] {
        let mut h = Sha3_256::new();
        h.update(FRAME_MAGIC.to_le_bytes());
        let frame_len = (FRAME_OVERHEAD + self.payload.len()) as u32;
        h.update(frame_len.to_le_bytes());
        h.update(self.sequence.to_le_bytes());
        h.update(self.timestamp_ns.to_le_bytes());
        h.update(self.prev_hash);
        h.update([self.payload_type as u8]);
        h.update((self.payload.len() as u32).to_le_bytes());
        h.update(&self.payload);
        h.finalize().into()
    }

    /// Serialize to bytes, computing hash and crc32c.
    pub fn encode(&mut self) -> Vec<u8> {
        let frame_len = (FRAME_OVERHEAD + self.payload.len()) as u32;
        let mut buf = Vec::with_capacity(frame_len as usize);

        buf.write_u32::<LittleEndian>(FRAME_MAGIC).unwrap();
        buf.write_u32::<LittleEndian>(frame_len).unwrap();
        buf.write_u64::<LittleEndian>(self.sequence).unwrap();
        buf.write_u64::<LittleEndian>(self.timestamp_ns).unwrap();
        buf.extend_from_slice(&self.prev_hash);
        buf.write_u8(self.payload_type as u8).unwrap();
        buf.write_u32::<LittleEndian>(self.payload.len() as u32).unwrap();
        buf.extend_from_slice(&self.payload);

        // compute hash over everything written so far
        let hash: [u8; 32] = {
            let mut h = Sha3_256::new();
            h.update(&buf);
            h.finalize().into()
        };
        self.frame_hash = hash;
        buf.extend_from_slice(&hash);

        let crc = crc32fast::hash(&buf);
        buf.write_u32::<LittleEndian>(crc).unwrap();

        buf
    }

    /// Deserialize from a reader; verifies CRC32 and frame_hash.
    pub fn decode<R: Read>(r: &mut R, segment_id: u64, offset: u64) -> WalResult<Self> {
        let magic = r.read_u32::<LittleEndian>()?;
        if magic != FRAME_MAGIC {
            return Err(WalError::CorruptFrame { segment: segment_id, offset, reason: "bad magic" });
        }
        let frame_len = r.read_u32::<LittleEndian>()? as usize;
        if frame_len < FRAME_OVERHEAD {
            return Err(WalError::CorruptFrame { segment: segment_id, offset, reason: "frame_len too small" });
        }
        let payload_len = frame_len - FRAME_OVERHEAD;

        let sequence     = r.read_u64::<LittleEndian>()?;
        let timestamp_ns = r.read_u64::<LittleEndian>()?;
        let mut prev_hash = [0u8; 32];
        r.read_exact(&mut prev_hash)?;
        let payload_type = PayloadType::try_from(r.read_u8()?)?;
        let declared_payload_len = r.read_u32::<LittleEndian>()? as usize;
        if declared_payload_len != payload_len {
            return Err(WalError::CorruptFrame { segment: segment_id, offset, reason: "payload_len mismatch" });
        }
        let mut payload = vec![0u8; payload_len];
        r.read_exact(&mut payload)?;
        let mut frame_hash = [0u8; 32];
        r.read_exact(&mut frame_hash)?;
        let stored_crc = r.read_u32::<LittleEndian>()?;

        // re-encode header + payload + hash to verify CRC
        let mut check_buf = Vec::with_capacity(frame_len);
        check_buf.write_u32::<LittleEndian>(FRAME_MAGIC).unwrap();
        check_buf.write_u32::<LittleEndian>(frame_len as u32).unwrap();
        check_buf.write_u64::<LittleEndian>(sequence).unwrap();
        check_buf.write_u64::<LittleEndian>(timestamp_ns).unwrap();
        check_buf.extend_from_slice(&prev_hash);
        check_buf.write_u8(payload_type as u8).unwrap();
        check_buf.write_u32::<LittleEndian>(payload_len as u32).unwrap();
        check_buf.extend_from_slice(&payload);
        check_buf.extend_from_slice(&frame_hash);

        let computed_crc = crc32fast::hash(&check_buf);
        if computed_crc != stored_crc {
            return Err(WalError::Crc32Mismatch { sequence });
        }

        // verify frame_hash
        let expected_hash: [u8; 32] = {
            let mut h = Sha3_256::new();
            // hash covers everything except the hash + crc tail
            h.update(&check_buf[..check_buf.len() - 32]);
            h.finalize().into()
        };
        if expected_hash != frame_hash {
            return Err(WalError::CorruptFrame { segment: segment_id, offset, reason: "frame_hash mismatch" });
        }

        Ok(Frame { sequence, timestamp_ns, prev_hash, payload_type, payload, frame_hash })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn roundtrip() {
        let mut f = Frame {
            sequence:     42,
            timestamp_ns: 123_456_789,
            prev_hash:    GENESIS_HASH,
            payload_type: PayloadType::Data,
            payload:      b"hello vectorvolt".to_vec(),
            frame_hash:   [0u8; 32],
        };
        let encoded = f.encode();
        let mut cur = Cursor::new(&encoded);
        let decoded = Frame::decode(&mut cur, 0, 0).unwrap();
        assert_eq!(decoded.sequence, 42);
        assert_eq!(decoded.payload, b"hello vectorvolt");
        assert_eq!(decoded.frame_hash, f.frame_hash);
    }

    #[test]
    fn detects_bit_flip() {
        let mut f = Frame {
            sequence: 1, timestamp_ns: 0, prev_hash: GENESIS_HASH,
            payload_type: PayloadType::Data, payload: vec![0u8; 64], frame_hash: [0u8; 32],
        };
        let mut encoded = f.encode();
        // flip a bit in the payload area
        encoded[FRAME_HEADER_SIZE + 10] ^= 0xFF;
        let mut cur = Cursor::new(&encoded);
        assert!(Frame::decode(&mut cur, 0, 0).is_err());
    }
}
