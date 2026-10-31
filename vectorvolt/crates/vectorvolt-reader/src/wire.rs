//! PostgreSQL wire protocol message encoding/decoding.
//!
//! Implements a minimal subset of the PostgreSQL frontend/backend protocol
//! (protocol version 3.0) sufficient for simple query execution.

use bytes::{Buf, BufMut, Bytes, BytesMut};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum WireError {
    #[error("incomplete message")]
    Incomplete,
    #[error("unknown message type: {0}")]
    UnknownMessageType(u8),
    #[error("invalid protocol version: {0}")]
    InvalidVersion(i32),
    #[error("invalid message length: {0}")]
    InvalidLength(i32),
    #[error("utf8 error: {0}")]
    Utf8(#[from] std::str::Utf8Error),
}

/// Largest frontend message accepted after startup, as the value of its
/// Int32 length field (which counts itself but not the type byte). Larger
/// messages are a protocol violation: the session answers with a FATAL
/// ErrorResponse (08P01) and closes, so a client cannot make the server
/// buffer an unbounded amount of memory.
pub const MAX_MESSAGE_LEN: usize = 1024 * 1024;

/// Validate the Int32 length field of a typed frontend message: it counts
/// itself, so it must be at least 4, and it may not exceed `max`.
fn checked_len(raw: i32, max: usize) -> Result<usize, WireError> {
    match usize::try_from(raw) {
        Ok(l) if l >= 4 && l <= max => Ok(l),
        _ => Err(WireError::InvalidLength(raw)),
    }
}

/// Split one complete typed frontend message (`Byte1(type) Int32(len)
/// payload`) off the front of `buf`.
///
/// Returns `Ok(None)` when more bytes are needed (several messages may sit
/// in `buf`, or one message may arrive across several reads), and
/// [`WireError::InvalidLength`] as soon as the header is readable and its
/// length is outside `4..=max` — before the body is buffered.
pub fn split_frontend_frame(buf: &mut BytesMut, max: usize) -> Result<Option<BytesMut>, WireError> {
    if buf.len() < 5 {
        return Ok(None);
    }
    let raw = i32::from_be_bytes([buf[1], buf[2], buf[3], buf[4]]);
    let len = checked_len(raw, max)?;
    if buf.len() < 1 + len {
        // Make room for the rest of this message up front.
        buf.reserve(1 + len - buf.len());
        return Ok(None);
    }
    Ok(Some(buf.split_to(1 + len)))
}

// ── Frontend message types ─────────────────────────────────────────────────────

/// A message sent by the client to the server.
#[derive(Debug, Clone, PartialEq)]
pub enum FrontendMsg {
    /// Startup message (not prefixed with a type byte).
    Startup {
        major: i16,
        minor: i16,
        params: Vec<(String, String)>,
    },
    /// Simple query.
    Query(String),
    /// PasswordMessage (reply to an authentication request).
    Password(String),
    /// Terminate connection.
    Terminate,
}

/// Parse a startup message from a buffer.
/// Format: Int32(len) Int32(version) [key\0value\0]* \0
pub fn decode_startup(buf: &[u8]) -> Result<FrontendMsg, WireError> {
    if buf.len() < 8 {
        return Err(WireError::Incomplete);
    }
    let mut b = buf;
    let _len = b.get_i32();
    let version = b.get_i32();
    let major = (version >> 16) as i16;
    let minor = (version & 0xFFFF) as i16;
    let mut params = Vec::new();
    loop {
        let key = read_cstr(&mut b)?;
        if key.is_empty() {
            break;
        }
        let val = read_cstr(&mut b)?;
        params.push((key, val));
    }
    Ok(FrontendMsg::Startup {
        major,
        minor,
        params,
    })
}

/// Parse a regular (post-startup) frontend message.
/// Format: Byte1(type) Int32(len) payload
pub fn decode_frontend(buf: &[u8]) -> Result<FrontendMsg, WireError> {
    if buf.len() < 5 {
        return Err(WireError::Incomplete);
    }
    let msg_type = buf[0];
    let mut b = &buf[1..];
    let len = checked_len(b.get_i32(), MAX_MESSAGE_LEN)?;
    if buf.len() < 1 + len {
        return Err(WireError::Incomplete);
    }
    // `len >= 4`, so `5 <= 1 + len <= buf.len()`: the slice cannot panic.
    let payload = &buf[5..1 + len];
    match msg_type {
        b'Q' => {
            // Query: null-terminated string (len includes itself)
            let s = read_cstr_bytes(payload)?;
            Ok(FrontendMsg::Query(s))
        }
        b'X' => Ok(FrontendMsg::Terminate),
        b'p' => Ok(FrontendMsg::Password(read_cstr_bytes(payload)?)),
        t => Err(WireError::UnknownMessageType(t)),
    }
}

// ── Backend message encoding ───────────────────────────────────────────────────

/// Encode an AuthenticationOk message (type 'R', payload Int32(0)).
pub fn encode_auth_ok() -> Bytes {
    let mut buf = BytesMut::with_capacity(9);
    buf.put_u8(b'R');
    buf.put_i32(8); // length including self
    buf.put_i32(0); // AuthenticationOk
    buf.freeze()
}

/// Encode an AuthenticationCleartextPassword request (type 'R', Int32(3)).
pub fn encode_auth_cleartext_password() -> Bytes {
    let mut buf = BytesMut::with_capacity(9);
    buf.put_u8(b'R');
    buf.put_i32(8); // length including self
    buf.put_i32(3); // AuthenticationCleartextPassword
    buf.freeze()
}

/// Encode a FATAL ErrorResponse carrying a SQLSTATE code (sent just before
/// the connection is closed, e.g. on a bad startup or failed authentication).
/// Format: 'E' Int32(len) 'S' FATAL\0 'C' code\0 'M' msg\0 \0
pub fn encode_fatal_error(code: &str, msg: &str) -> Bytes {
    let mut payload = BytesMut::new();
    payload.put_u8(b'S');
    payload.put("FATAL".as_bytes());
    payload.put_u8(0);
    payload.put_u8(b'C');
    payload.put(code.as_bytes());
    payload.put_u8(0);
    payload.put_u8(b'M');
    payload.put(msg.as_bytes());
    payload.put_u8(0);
    payload.put_u8(0); // terminator
    let mut buf = BytesMut::with_capacity(1 + 4 + payload.len());
    buf.put_u8(b'E');
    buf.put_i32((4 + payload.len()) as i32);
    buf.put(payload);
    buf.freeze()
}

/// Encode a ParameterStatus message.
/// Format: 'S' Int32(len) key\0 value\0
pub fn encode_param_status(key: &str, value: &str) -> Bytes {
    let payload_len = key.len() + 1 + value.len() + 1;
    let mut buf = BytesMut::with_capacity(1 + 4 + payload_len);
    buf.put_u8(b'S');
    buf.put_i32((4 + payload_len) as i32);
    buf.put(key.as_bytes());
    buf.put_u8(0);
    buf.put(value.as_bytes());
    buf.put_u8(0);
    buf.freeze()
}

/// Encode a ReadyForQuery message.
/// Format: 'Z' Int32(5) Byte1(status) where status='I' (idle)
pub fn encode_ready_for_query() -> Bytes {
    let mut buf = BytesMut::with_capacity(6);
    buf.put_u8(b'Z');
    buf.put_i32(5);
    buf.put_u8(b'I'); // idle
    buf.freeze()
}

/// Encode a RowDescription message.
/// Format: 'T' Int32(len) Int16(field_count) [field]*
/// Each field: name\0 Int32(table_oid) Int16(col_attr) Int32(type_oid) Int16(type_size) Int32(type_modifier) Int16(format)
pub fn encode_row_description(columns: &[String]) -> Bytes {
    let mut payload = BytesMut::new();
    payload.put_i16(columns.len() as i16);
    for col in columns {
        payload.put(col.as_bytes());
        payload.put_u8(0);
        payload.put_i32(0); // table oid
        payload.put_i16(0); // col attr
        payload.put_i32(25); // text oid
        payload.put_i16(-1); // variable length
        payload.put_i32(-1); // no modifier
        payload.put_i16(0); // text format
    }
    let mut buf = BytesMut::with_capacity(1 + 4 + payload.len());
    buf.put_u8(b'T');
    buf.put_i32((4 + payload.len()) as i32);
    buf.put(payload);
    buf.freeze()
}

/// Encode a DataRow message.
/// Format: 'D' Int32(len) Int16(col_count) [Int32(col_len) col_data | Int32(-1)]*
pub fn encode_data_row(row: &[String]) -> Bytes {
    let mut payload = BytesMut::new();
    payload.put_i16(row.len() as i16);
    for cell in row {
        let b = cell.as_bytes();
        payload.put_i32(b.len() as i32);
        payload.put(b);
    }
    let mut buf = BytesMut::with_capacity(1 + 4 + payload.len());
    buf.put_u8(b'D');
    buf.put_i32((4 + payload.len()) as i32);
    buf.put(payload);
    buf.freeze()
}

/// Encode a CommandComplete message.
/// Format: 'C' Int32(len) tag\0
pub fn encode_command_complete(tag: &str) -> Bytes {
    let mut buf = BytesMut::with_capacity(1 + 4 + tag.len() + 1);
    buf.put_u8(b'C');
    buf.put_i32((4 + tag.len() + 1) as i32);
    buf.put(tag.as_bytes());
    buf.put_u8(0);
    buf.freeze()
}

/// Encode an ErrorResponse message.
/// Format: 'E' Int32(len) fields... \0
/// We only emit the 'S' (severity) and 'M' (message) fields.
pub fn encode_error_response(msg: &str) -> Bytes {
    let mut payload = BytesMut::new();
    payload.put_u8(b'S');
    payload.put("ERROR".as_bytes());
    payload.put_u8(0);
    payload.put_u8(b'M');
    payload.put(msg.as_bytes());
    payload.put_u8(0);
    payload.put_u8(0); // terminator
    let mut buf = BytesMut::with_capacity(1 + 4 + payload.len());
    buf.put_u8(b'E');
    buf.put_i32((4 + payload.len()) as i32);
    buf.put(payload);
    buf.freeze()
}

// ── helpers ───────────────────────────────────────────────────────────────────

fn read_cstr(buf: &mut &[u8]) -> Result<String, WireError> {
    let end = buf
        .iter()
        .position(|&b| b == 0)
        .ok_or(WireError::Incomplete)?;
    let s = std::str::from_utf8(&buf[..end])?.to_owned();
    *buf = &buf[end + 1..];
    Ok(s)
}

fn read_cstr_bytes(buf: &[u8]) -> Result<String, WireError> {
    let end = buf.iter().position(|&b| b == 0).unwrap_or(buf.len());
    Ok(std::str::from_utf8(&buf[..end])?.to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_auth_ok() {
        let b = encode_auth_ok();
        assert_eq!(b[0], b'R');
        assert_eq!(b.len(), 9);
    }

    #[test]
    fn test_encode_ready_for_query() {
        let b = encode_ready_for_query();
        assert_eq!(b[0], b'Z');
        assert_eq!(b[b.len() - 1], b'I');
    }

    #[test]
    fn test_encode_param_status() {
        let b = encode_param_status("server_version", "16.0");
        assert_eq!(b[0], b'S');
    }

    #[test]
    fn test_encode_row_description_no_cols() {
        let b = encode_row_description(&[]);
        assert_eq!(b[0], b'T');
    }

    #[test]
    fn test_encode_row_description_with_cols() {
        let cols = vec!["pk".to_owned(), "val".to_owned()];
        let b = encode_row_description(&cols);
        assert_eq!(b[0], b'T');
        // Int16 field count = 2 at bytes [5..7]
        let count = i16::from_be_bytes([b[5], b[6]]);
        assert_eq!(count, 2);
    }

    #[test]
    fn test_encode_data_row() {
        let row = vec!["1".to_owned(), "hello".to_owned()];
        let b = encode_data_row(&row);
        assert_eq!(b[0], b'D');
    }

    #[test]
    fn test_encode_command_complete() {
        let b = encode_command_complete("SELECT 1");
        assert_eq!(b[0], b'C');
    }

    #[test]
    fn test_encode_error_response() {
        let b = encode_error_response("something went wrong");
        assert_eq!(b[0], b'E');
    }

    #[test]
    fn test_decode_startup_basic() {
        // Build a startup message
        let mut buf = BytesMut::new();
        let version: i32 = 3 << 16; // 3.0
        // We'll write: len(4) version(4) "user\0alice\0\0"
        let params = b"user\0alice\0\0";
        let len = 4 + 4 + params.len();
        buf.put_i32(len as i32);
        buf.put_i32(version);
        buf.put(params.as_ref());
        let msg = decode_startup(&buf).unwrap();
        assert!(matches!(
            msg,
            FrontendMsg::Startup {
                major: 3,
                minor: 0,
                ..
            }
        ));
    }

    #[test]
    fn test_decode_frontend_query() {
        // 'Q' Int32(len) "SELECT 1\0"
        let query = b"SELECT 1\0";
        let len = 4 + query.len();
        let mut buf = BytesMut::new();
        buf.put_u8(b'Q');
        buf.put_i32(len as i32);
        buf.put(query.as_ref());
        let msg = decode_frontend(&buf).unwrap();
        assert_eq!(msg, FrontendMsg::Query("SELECT 1".into()));
    }

    #[test]
    fn test_decode_frontend_terminate() {
        let mut buf = BytesMut::new();
        buf.put_u8(b'X');
        buf.put_i32(4);
        let msg = decode_frontend(&buf).unwrap();
        assert_eq!(msg, FrontendMsg::Terminate);
    }

    #[test]
    fn test_decode_frontend_unknown() {
        let mut buf = BytesMut::new();
        buf.put_u8(b'Z'); // not a known frontend message type
        buf.put_i32(4);
        let r = decode_frontend(&buf);
        assert!(r.is_err());
    }

    #[test]
    fn test_decode_frontend_password() {
        let pw = b"hunter2\0";
        let mut buf = BytesMut::new();
        buf.put_u8(b'p');
        buf.put_i32((4 + pw.len()) as i32);
        buf.put(pw.as_ref());
        let msg = decode_frontend(&buf).unwrap();
        assert_eq!(msg, FrontendMsg::Password("hunter2".into()));
    }

    #[test]
    fn test_encode_auth_cleartext_password() {
        let b = encode_auth_cleartext_password();
        assert_eq!(b[0], b'R');
        assert_eq!(i32::from_be_bytes([b[5], b[6], b[7], b[8]]), 3);
    }

    fn header(t: u8, len: i32) -> Vec<u8> {
        let mut v = vec![t];
        v.extend_from_slice(&len.to_be_bytes());
        v
    }

    /// B12: a length field below 4 (or negative) used to panic in the slice
    /// `buf[5..1 + len]` / overflow in `1 + len`.
    #[test]
    fn test_decode_frontend_rejects_bad_lengths() {
        for len in [0, 1, 3, -1, -4, i32::MIN, i32::MAX] {
            let mut buf = header(b'Q', len);
            buf.extend_from_slice(&[0u8; 16]);
            assert!(
                matches!(decode_frontend(&buf), Err(WireError::InvalidLength(l)) if l == len),
                "len {len}"
            );
        }
        // Over the cap is rejected even if the body is present.
        let mut buf = header(b'Q', MAX_MESSAGE_LEN as i32 + 1);
        buf.resize(MAX_MESSAGE_LEN + 2, b'a');
        assert!(matches!(
            decode_frontend(&buf),
            Err(WireError::InvalidLength(_))
        ));
    }

    #[test]
    fn test_split_frontend_frame_partial_and_pipelined() {
        let q = |sql: &str| {
            let mut v = header(b'Q', (4 + sql.len() + 1) as i32);
            v.extend_from_slice(sql.as_bytes());
            v.push(0);
            v
        };
        let mut all = q("SELECT 1");
        all.extend(q("SELECT 2"));
        // Fed byte by byte, exactly two frames come out, in order.
        let mut buf = BytesMut::new();
        let mut out = Vec::new();
        for &byte in &all {
            buf.put_u8(byte);
            while let Some(f) = split_frontend_frame(&mut buf, MAX_MESSAGE_LEN).unwrap() {
                out.push(decode_frontend(&f).unwrap());
            }
        }
        assert_eq!(
            out,
            vec![
                FrontendMsg::Query("SELECT 1".into()),
                FrontendMsg::Query("SELECT 2".into())
            ]
        );
        assert!(buf.is_empty());
        // An oversized header is rejected before the body arrives.
        let mut buf = BytesMut::from(&header(b'Q', 2 * MAX_MESSAGE_LEN as i32)[..]);
        assert!(split_frontend_frame(&mut buf, MAX_MESSAGE_LEN).is_err());
        let mut buf = BytesMut::from(&header(b'Q', 3)[..]);
        assert!(split_frontend_frame(&mut buf, MAX_MESSAGE_LEN).is_err());
    }

    /// Deterministic xorshift64* generator (no extra dev-dependency).
    struct Rng(u64);
    impl Rng {
        fn next(&mut self) -> u64 {
            self.0 ^= self.0 >> 12;
            self.0 ^= self.0 << 25;
            self.0 ^= self.0 >> 27;
            self.0.wrapping_mul(0x2545_F491_4F6C_DD1D)
        }
    }

    /// B12 fuzz-style: no client byte sequence may panic the decoders or the
    /// framer. Mixes fully random bytes with known type bytes and hostile
    /// length fields.
    #[test]
    fn test_decoders_never_panic_on_hostile_input() {
        let mut rng = Rng(0x9E37_79B9_7F4A_7C15);
        let types = [b'Q', b'X', b'p', b'P', b'Z', 0, 0xFF];
        let lens = [
            0,
            1,
            3,
            4,
            5,
            8,
            -1,
            i32::MIN,
            i32::MAX,
            MAX_MESSAGE_LEN as i32,
            MAX_MESSAGE_LEN as i32 + 1,
        ];
        for i in 0..20_000 {
            let n = (rng.next() % 64) as usize;
            let mut bytes: Vec<u8> = (0..n).map(|_| rng.next() as u8).collect();
            if i % 2 == 0 {
                let mut h = vec![types[(rng.next() as usize) % types.len()]];
                let len = if i % 4 == 0 {
                    lens[(rng.next() as usize) % lens.len()]
                } else {
                    rng.next() as i32
                };
                h.extend_from_slice(&len.to_be_bytes());
                h.extend(bytes);
                bytes = h;
            }
            let _ = decode_frontend(&bytes);
            let _ = decode_startup(&bytes);
            let mut buf = BytesMut::from(&bytes[..]);
            while let Ok(Some(f)) = split_frontend_frame(&mut buf, MAX_MESSAGE_LEN) {
                let _ = decode_frontend(&f);
            }
        }
    }

    #[test]
    fn test_decode_startup_incomplete() {
        let buf = [0u8; 3];
        let r = decode_startup(&buf);
        assert!(r.is_err());
    }
}
