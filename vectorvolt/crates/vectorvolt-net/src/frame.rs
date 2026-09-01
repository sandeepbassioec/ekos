//! `WireFrameHeader` — 64-byte, cache-line-aligned wire frame (VV-NET-001).
//!
//! Binary layout (little-endian):
//! ```text
//! offset  size  field
//!      0     4  magic       — 0x5656_4E45 "VVNE"
//!      4     1  version     — must be 1
//!      5     1  opcode      — see [`WireOpcode`]
//!      6     2  flags       — reserved, must be zero
//!      8     4  payload_len — byte count of the payload that follows
//!     12     4  crc32c      — CRC-32C of the payload bytes
//!     16     8  request_id  — client correlation id echoed in every response
//!     24    40  _pad        — zero-filled, reserved for future use
//! ```
//! Total: 64 bytes.  `align(64)` ensures the header begins on a cache-line
//! boundary, which matters for zero-copy paths.

use crate::{NetError, NetResult};

/// Frame magic: ASCII "VVNE" in little-endian as a u32.
pub const WIRE_MAGIC: u32 = 0x5656_4E45;

/// Current protocol version.
pub const WIRE_VERSION: u8 = 1;

/// Size of [`WireFrameHeader`] in bytes.
pub const WIRE_HEADER_SIZE: usize = 64;

/// Alias for [`WIRE_HEADER_SIZE`], used by `connection.rs`.
pub const WIRE_HEADER_BYTES: usize = WIRE_HEADER_SIZE;

/// Maximum payload accepted per frame: 64 MiB.
pub const WIRE_MAX_PAYLOAD: u32 = 64 * 1024 * 1024;

// ── WireOpcode ────────────────────────────────────────────────────────────────

/// Request/response opcodes carried in [`WireFrameHeader::opcode`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum WireOpcode {
    Execute = 0x01,
    Query = 0x02,
    TxnBegin = 0x10,
    TxnCommit = 0x11,
    TxnAbort = 0x12,
    Health = 0x20,
    Ping = 0x21,
    PingAck = 0x22,
}

impl WireOpcode {
    /// Parse an opcode byte.  Returns `Err` for unknown values.
    pub fn from_u8(v: u8) -> NetResult<Self> {
        match v {
            0x01 => Ok(Self::Execute),
            0x02 => Ok(Self::Query),
            0x10 => Ok(Self::TxnBegin),
            0x11 => Ok(Self::TxnCommit),
            0x12 => Ok(Self::TxnAbort),
            0x20 => Ok(Self::Health),
            0x21 => Ok(Self::Ping),
            0x22 => Ok(Self::PingAck),
            _ => Err(NetError::UnknownOpcode(v)),
        }
    }
}

// Legacy alias for code that imports `Opcode`.
pub type Opcode = WireOpcode;

// ── WireFrameHeader ───────────────────────────────────────────────────────────

/// 64-byte, cache-line-aligned wire frame header (VV-NET-001).
///
/// `#[repr(C)]` ensures a stable, predictable field layout.
/// `align(64)` pins the struct to a cache-line boundary for zero-copy I/O.
#[repr(C, align(64))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WireFrameHeader {
    /// Frame magic: must equal [`WIRE_MAGIC`].
    pub magic: u32,
    /// Protocol version: must equal [`WIRE_VERSION`].
    pub version: u8,
    /// Opcode byte — see [`WireOpcode`].
    pub opcode: u8,
    /// Flags: reserved, must be zero.
    pub flags: u16,
    /// Byte count of the payload that follows the header.
    pub payload_len: u32,
    /// CRC-32C over the payload bytes.  Validated by [`verify_payload`].
    pub crc32c: u32,
    /// Client-supplied correlation id.  Echoed verbatim in every response.
    pub request_id: u64,
    /// Zero padding to reach exactly 64 bytes.
    _pad: [u8; 40],
}

const _: () = assert!(std::mem::size_of::<WireFrameHeader>() == 64);

impl WireFrameHeader {
    /// Construct a new header.  `crc32c` is computed from `payload`.
    ///
    /// Returns `Err(PayloadTooLarge)` if `payload.len()` exceeds
    /// [`WIRE_MAX_PAYLOAD`].
    pub fn new(opcode: WireOpcode, payload: &[u8], request_id: u64) -> NetResult<Self> {
        if payload.len() as u64 > WIRE_MAX_PAYLOAD as u64 {
            return Err(NetError::PayloadTooLarge {
                len: payload.len() as u32,
                max: WIRE_MAX_PAYLOAD,
            });
        }
        Ok(Self {
            magic: WIRE_MAGIC,
            version: WIRE_VERSION,
            opcode: opcode as u8,
            flags: 0,
            payload_len: payload.len() as u32,
            crc32c: crc32fast::hash(payload),
            request_id,
            _pad: [0u8; 40],
        })
    }

    /// Serialise the 64-byte header into a fixed-size byte array.
    ///
    /// The payload is NOT included; callers write the header then the payload
    /// separately, which avoids an allocation in the common path.
    pub fn encode(&self) -> [u8; WIRE_HEADER_SIZE] {
        let mut out = [0u8; WIRE_HEADER_SIZE];
        out[0..4].copy_from_slice(&self.magic.to_le_bytes());
        out[4] = self.version;
        out[5] = self.opcode;
        out[6..8].copy_from_slice(&self.flags.to_le_bytes());
        out[8..12].copy_from_slice(&self.payload_len.to_le_bytes());
        out[12..16].copy_from_slice(&self.crc32c.to_le_bytes());
        out[16..24].copy_from_slice(&self.request_id.to_le_bytes());
        // _pad bytes at 24..64 stay zero.
        out
    }

    /// Encode header + payload into a single `Vec<u8>`.
    ///
    /// Convenience wrapper for contexts where both parts can be concatenated
    /// without a copy concern.
    pub fn encode_with_payload(&self, payload: &[u8]) -> Vec<u8> {
        let hdr = self.encode();
        let mut out = Vec::with_capacity(WIRE_HEADER_SIZE + payload.len());
        out.extend_from_slice(&hdr);
        out.extend_from_slice(payload);
        out
    }

    /// Decode a header from exactly [`WIRE_HEADER_SIZE`] bytes.
    ///
    /// Validates magic, version, and opcode. Does NOT verify the CRC at this
    /// stage — call [`verify_payload`] once the payload bytes are available.
    pub fn decode(buf: &[u8]) -> NetResult<Self> {
        if buf.len() < WIRE_HEADER_SIZE {
            return Err(NetError::Truncated {
                need: WIRE_HEADER_SIZE,
                got: buf.len(),
            });
        }

        let magic = u32::from_le_bytes(buf[0..4].try_into().unwrap());
        if magic != WIRE_MAGIC {
            return Err(NetError::BadMagic);
        }

        let version = buf[4];
        if version != WIRE_VERSION {
            return Err(NetError::UnsupportedVersion(version));
        }

        let opcode = buf[5];
        WireOpcode::from_u8(opcode)?; // validate

        let flags = u16::from_le_bytes(buf[6..8].try_into().unwrap());
        let payload_len = u32::from_le_bytes(buf[8..12].try_into().unwrap());

        if payload_len > WIRE_MAX_PAYLOAD {
            return Err(NetError::PayloadTooLarge {
                len: payload_len,
                max: WIRE_MAX_PAYLOAD,
            });
        }

        let crc32c = u32::from_le_bytes(buf[12..16].try_into().unwrap());
        let request_id = u64::from_le_bytes(buf[16..24].try_into().unwrap());

        Ok(WireFrameHeader {
            magic,
            version,
            opcode,
            flags,
            payload_len,
            crc32c,
            request_id,
            _pad: [0u8; 40],
        })
    }

    /// Decode header + payload from a single contiguous buffer.
    ///
    /// Validates magic, version, opcode, payload length, and CRC-32C.
    /// Returns `(header, payload_slice)`.
    pub fn decode_full(buf: &[u8]) -> NetResult<(Self, &[u8])> {
        let header = Self::decode(buf)?;
        let total = WIRE_HEADER_SIZE + header.payload_len as usize;
        if buf.len() < total {
            return Err(NetError::Truncated {
                need: total,
                got: buf.len(),
            });
        }
        let payload = &buf[WIRE_HEADER_SIZE..total];
        header.verify_payload(payload)?;
        Ok((header, payload))
    }

    /// Verify that `payload` matches the CRC-32C stored in this header.
    pub fn verify_payload(&self, payload: &[u8]) -> NetResult<()> {
        let actual = crc32fast::hash(payload);
        if actual != self.crc32c {
            return Err(NetError::CrcMismatch {
                expected: self.crc32c,
                actual,
            });
        }
        Ok(())
    }

    /// Return the opcode as a [`WireOpcode`].
    pub fn wire_opcode(&self) -> NetResult<WireOpcode> {
        WireOpcode::from_u8(self.opcode)
    }
}

// ── tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn header_size_is_64() {
        assert_eq!(std::mem::size_of::<WireFrameHeader>(), 64);
    }

    #[test]
    fn header_align_is_64() {
        assert_eq!(std::mem::align_of::<WireFrameHeader>(), 64);
    }

    #[test]
    fn encode_decode_roundtrip() {
        let payload = b"hello vectorvolt";
        let h = WireFrameHeader::new(WireOpcode::Execute, payload, 1).unwrap();
        let encoded = h.encode_with_payload(payload);
        let (decoded, decoded_payload) = WireFrameHeader::decode_full(&encoded).unwrap();
        assert_eq!(decoded.magic, WIRE_MAGIC);
        assert_eq!(decoded.opcode, WireOpcode::Execute as u8);
        assert_eq!(decoded.request_id, 1);
        assert_eq!(decoded_payload, payload);
    }

    #[test]
    fn magic_mismatch_returns_error() {
        let payload = b"test";
        let h = WireFrameHeader::new(WireOpcode::Ping, payload, 0).unwrap();
        let mut encoded = h.encode_with_payload(payload);
        encoded[0] = 0xFF;
        assert!(matches!(
            WireFrameHeader::decode(&encoded),
            Err(NetError::BadMagic)
        ));
    }

    #[test]
    fn bad_crc_returns_error() {
        let payload = b"test";
        let h = WireFrameHeader::new(WireOpcode::Ping, payload, 0).unwrap();
        let hdr_bytes = h.encode();
        let mut full = Vec::new();
        full.extend_from_slice(&hdr_bytes);
        full.extend_from_slice(payload);
        // Flip a bit in the payload.
        let last = full.len() - 1;
        full[last] ^= 0xFF;
        let hdr2 = WireFrameHeader::decode(&full).unwrap();
        let payload_part = &full[WIRE_HEADER_SIZE..];
        assert!(matches!(
            hdr2.verify_payload(payload_part),
            Err(NetError::CrcMismatch { .. })
        ));
    }

    #[test]
    fn payload_length_mismatch_returns_truncated() {
        let payload = b"abc";
        let h = WireFrameHeader::new(WireOpcode::Query, payload, 99).unwrap();
        let encoded = h.encode_with_payload(payload);
        let truncated = &encoded[..encoded.len() - 1];
        assert!(matches!(
            WireFrameHeader::decode_full(truncated),
            Err(NetError::Truncated { .. })
        ));
    }

    #[test]
    fn all_opcodes_serialize_correctly() {
        let opcodes = [
            WireOpcode::Execute,
            WireOpcode::Query,
            WireOpcode::TxnBegin,
            WireOpcode::TxnCommit,
            WireOpcode::TxnAbort,
            WireOpcode::Health,
            WireOpcode::Ping,
            WireOpcode::PingAck,
        ];
        for op in opcodes {
            let h = WireFrameHeader::new(op, b"", 0).unwrap();
            let enc = h.encode();
            let dec = WireFrameHeader::decode(&enc).unwrap();
            assert_eq!(dec.opcode, op as u8);
        }
    }

    #[test]
    fn zero_payload_ping_frame() {
        let h = WireFrameHeader::new(WireOpcode::Ping, b"", 42).unwrap();
        let enc = h.encode_with_payload(b"");
        let (dec, dec_payload) = WireFrameHeader::decode_full(&enc).unwrap();
        assert_eq!(dec.opcode, WireOpcode::Ping as u8);
        assert!(dec_payload.is_empty());
    }

    #[test]
    fn large_payload_roundtrip() {
        let payload: Vec<u8> = (0..65536u32).map(|i| (i % 256) as u8).collect();
        let h = WireFrameHeader::new(WireOpcode::Execute, &payload, 7).unwrap();
        let enc = h.encode_with_payload(&payload);
        let (dec, dec_payload) = WireFrameHeader::decode_full(&enc).unwrap();
        assert_eq!(dec.request_id, 7);
        assert_eq!(dec_payload, payload.as_slice());
    }

    #[test]
    fn request_id_roundtrip() {
        let id: u64 = 0xDEAD_BEEF_CAFE_BABE;
        let h = WireFrameHeader::new(WireOpcode::TxnBegin, b"", id).unwrap();
        let enc = h.encode();
        let dec = WireFrameHeader::decode(&enc).unwrap();
        assert_eq!(dec.request_id, id);
    }

    #[test]
    fn version_check_rejects_bad_version() {
        let h = WireFrameHeader::new(WireOpcode::Ping, b"", 0).unwrap();
        let mut enc = h.encode();
        enc[4] = 42;
        assert!(matches!(
            WireFrameHeader::decode(&enc),
            Err(NetError::UnsupportedVersion(42))
        ));
    }

    #[test]
    fn concurrent_encode_decode_consistency() {
        use std::sync::Arc;
        use std::thread;

        let payload = Arc::new(b"concurrent test".to_vec());
        let mut handles = Vec::new();
        for i in 0..8u64 {
            let p = Arc::clone(&payload);
            handles.push(thread::spawn(move || {
                let h = WireFrameHeader::new(WireOpcode::Execute, &p, i).unwrap();
                let enc = h.encode_with_payload(&p);
                let (dec, _) = WireFrameHeader::decode_full(&enc).unwrap();
                assert_eq!(dec.request_id, i);
            }));
        }
        for h in handles {
            h.join().unwrap();
        }
    }

    #[test]
    fn truncated_header_returns_error() {
        let h = WireFrameHeader::new(WireOpcode::Ping, b"", 0).unwrap();
        let enc = h.encode();
        let short = &enc[..WIRE_HEADER_SIZE - 1];
        assert!(matches!(
            WireFrameHeader::decode(short),
            Err(NetError::Truncated { .. })
        ));
    }
}
