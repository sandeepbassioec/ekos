//! PostgreSQL wire protocol message encoding/decoding.
//!
//! Implements a minimal subset of the PostgreSQL frontend/backend protocol
//! (protocol version 3.0): startup, cleartext / MD5 / SASL (SCRAM)
//! authentication messages, the simple query protocol and the extended
//! query protocol (Parse / Bind / Describe / Execute / Close / Sync / Flush).

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
    /// Extended query: Parse ('P').
    Parse {
        name: String,
        query: String,
        param_types: Vec<u32>,
    },
    /// Extended query: Bind ('B'). `params[i] = None` is SQL NULL.
    Bind {
        portal: String,
        statement: String,
        param_formats: Vec<i16>,
        params: Vec<Option<Vec<u8>>>,
        result_formats: Vec<i16>,
    },
    /// Extended query: Describe ('D'); `kind` is `b'S'` (statement) or
    /// `b'P'` (portal).
    Describe { kind: u8, name: String },
    /// Extended query: Execute ('E'); `max_rows == 0` = no limit.
    Execute { portal: String, max_rows: i32 },
    /// Extended query: Close ('C'); `kind` is `b'S'` or `b'P'`.
    Close { kind: u8, name: String },
    /// Extended query: Sync ('S').
    Sync,
    /// Extended query: Flush ('H').
    Flush,
}

impl FrontendMsg {
    /// True for the extended-query messages that are skipped after an error
    /// until the next Sync.
    pub fn is_extended(&self) -> bool {
        matches!(
            self,
            FrontendMsg::Parse { .. }
                | FrontendMsg::Bind { .. }
                | FrontendMsg::Describe { .. }
                | FrontendMsg::Execute { .. }
                | FrontendMsg::Close { .. }
                | FrontendMsg::Flush
        )
    }
}

/// Bounds-checked big-endian reader over a message payload: every read past
/// the end is [`WireError::Incomplete`], never a panic.
struct Rd<'a>(&'a [u8]);

impl<'a> Rd<'a> {
    fn take(&mut self, n: usize) -> Result<&'a [u8], WireError> {
        if self.0.len() < n {
            return Err(WireError::Incomplete);
        }
        let (a, b) = self.0.split_at(n);
        self.0 = b;
        Ok(a)
    }
    fn u8(&mut self) -> Result<u8, WireError> {
        Ok(self.take(1)?[0])
    }
    fn i16(&mut self) -> Result<i16, WireError> {
        let b = self.take(2)?;
        Ok(i16::from_be_bytes([b[0], b[1]]))
    }
    fn i32(&mut self) -> Result<i32, WireError> {
        let b = self.take(4)?;
        Ok(i32::from_be_bytes([b[0], b[1], b[2], b[3]]))
    }
    /// A non-negative Int16 count.
    fn count(&mut self) -> Result<usize, WireError> {
        let n = self.i16()?;
        usize::try_from(n).map_err(|_| WireError::InvalidLength(i32::from(n)))
    }
    fn cstr(&mut self) -> Result<String, WireError> {
        read_cstr(&mut self.0)
    }
}

fn decode_parse(p: &[u8]) -> Result<FrontendMsg, WireError> {
    let mut r = Rd(p);
    let name = r.cstr()?;
    let query = r.cstr()?;
    let n = r.count()?;
    let mut param_types = Vec::with_capacity(n.min(p.len() / 4));
    for _ in 0..n {
        param_types.push(r.i32()? as u32);
    }
    Ok(FrontendMsg::Parse {
        name,
        query,
        param_types,
    })
}

fn decode_bind(p: &[u8]) -> Result<FrontendMsg, WireError> {
    let mut r = Rd(p);
    let portal = r.cstr()?;
    let statement = r.cstr()?;
    let nf = r.count()?;
    let mut param_formats = Vec::with_capacity(nf.min(p.len() / 2));
    for _ in 0..nf {
        param_formats.push(r.i16()?);
    }
    let np = r.count()?;
    let mut params = Vec::with_capacity(np.min(p.len() / 4));
    for _ in 0..np {
        let len = r.i32()?;
        if len == -1 {
            params.push(None);
        } else {
            let len = usize::try_from(len).map_err(|_| WireError::InvalidLength(len))?;
            params.push(Some(r.take(len)?.to_vec()));
        }
    }
    let nr = r.count()?;
    let mut result_formats = Vec::with_capacity(nr.min(p.len() / 2));
    for _ in 0..nr {
        result_formats.push(r.i16()?);
    }
    Ok(FrontendMsg::Bind {
        portal,
        statement,
        param_formats,
        params,
        result_formats,
    })
}

fn decode_target(p: &[u8]) -> Result<(u8, String), WireError> {
    let mut r = Rd(p);
    let kind = r.u8()?;
    if kind != b'S' && kind != b'P' {
        return Err(WireError::UnknownMessageType(kind));
    }
    Ok((kind, r.cstr()?))
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
        b'P' => decode_parse(payload),
        b'B' => decode_bind(payload),
        b'D' => {
            let (kind, name) = decode_target(payload)?;
            Ok(FrontendMsg::Describe { kind, name })
        }
        b'C' => {
            let (kind, name) = decode_target(payload)?;
            Ok(FrontendMsg::Close { kind, name })
        }
        b'E' => {
            let mut r = Rd(payload);
            let portal = r.cstr()?;
            let max_rows = r.i32()?;
            Ok(FrontendMsg::Execute { portal, max_rows })
        }
        b'S' => Ok(FrontendMsg::Sync),
        b'H' => Ok(FrontendMsg::Flush),
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

/// Encode an `AuthenticationXxx` message: 'R' Int32(len) Int32(code) data.
fn encode_auth(code: i32, data: &[u8]) -> Bytes {
    let mut buf = BytesMut::with_capacity(9 + data.len());
    buf.put_u8(b'R');
    buf.put_i32((8 + data.len()) as i32);
    buf.put_i32(code);
    buf.put(data);
    buf.freeze()
}

/// AuthenticationMD5Password (code 5) with its 4-byte salt.
pub fn encode_auth_md5_password(salt: &[u8; 4]) -> Bytes {
    encode_auth(5, salt)
}

/// AuthenticationSASL (code 10): the offered mechanisms, each NUL-terminated,
/// then an empty terminator.
pub fn encode_auth_sasl(mechanisms: &[&str]) -> Bytes {
    let mut data = Vec::new();
    for m in mechanisms {
        data.extend_from_slice(m.as_bytes());
        data.push(0);
    }
    data.push(0);
    encode_auth(10, &data)
}

/// AuthenticationSASLContinue (code 11) carrying the server-first-message.
pub fn encode_auth_sasl_continue(data: &[u8]) -> Bytes {
    encode_auth(11, data)
}

/// AuthenticationSASLFinal (code 12) carrying the server-final-message.
pub fn encode_auth_sasl_final(data: &[u8]) -> Bytes {
    encode_auth(12, data)
}

/// A SASLInitialResponse ('p' during SASL): the chosen mechanism and the
/// initial client data (`None` when its length is -1).
#[derive(Debug, Clone, PartialEq)]
pub struct SaslInitialResponse {
    pub mechanism: String,
    pub data: Option<Vec<u8>>,
}

/// Decode a SASLInitialResponse from a full 'p' message (type byte + length).
pub fn decode_sasl_initial_response(buf: &[u8]) -> Result<SaslInitialResponse, WireError> {
    let payload = typed_payload(buf, b'p')?;
    let mut r = Rd(payload);
    let mechanism = r.cstr()?;
    let len = r.i32()?;
    let data = if len == -1 {
        None
    } else {
        let len = usize::try_from(len).map_err(|_| WireError::InvalidLength(len))?;
        Some(r.take(len)?.to_vec())
    };
    Ok(SaslInitialResponse { mechanism, data })
}

/// The raw data of a SASLResponse ('p' during SASL) message.
pub fn decode_sasl_response(buf: &[u8]) -> Result<Vec<u8>, WireError> {
    Ok(typed_payload(buf, b'p')?.to_vec())
}

/// The payload of a typed message of type `expected`.
fn typed_payload(buf: &[u8], expected: u8) -> Result<&[u8], WireError> {
    if buf.len() < 5 {
        return Err(WireError::Incomplete);
    }
    if buf[0] != expected {
        return Err(WireError::UnknownMessageType(buf[0]));
    }
    let len = checked_len(
        i32::from_be_bytes([buf[1], buf[2], buf[3], buf[4]]),
        MAX_MESSAGE_LEN,
    )?;
    if buf.len() < 1 + len {
        return Err(WireError::Incomplete);
    }
    Ok(&buf[5..1 + len])
}

/// A body-less backend message: type byte + Int32(4).
fn encode_empty(t: u8) -> Bytes {
    let mut buf = BytesMut::with_capacity(5);
    buf.put_u8(t);
    buf.put_i32(4);
    buf.freeze()
}

/// ParseComplete ('1').
pub fn encode_parse_complete() -> Bytes {
    encode_empty(b'1')
}

/// BindComplete ('2').
pub fn encode_bind_complete() -> Bytes {
    encode_empty(b'2')
}

/// CloseComplete ('3').
pub fn encode_close_complete() -> Bytes {
    encode_empty(b'3')
}

/// NoData ('n').
pub fn encode_no_data() -> Bytes {
    encode_empty(b'n')
}

/// PortalSuspended ('s'): Execute hit its row limit.
pub fn encode_portal_suspended() -> Bytes {
    encode_empty(b's')
}

/// EmptyQueryResponse ('I').
pub fn encode_empty_query_response() -> Bytes {
    encode_empty(b'I')
}

/// ParameterDescription ('t'): Int16(count) then one Int32 type OID each.
pub fn encode_parameter_description(oids: &[u32]) -> Bytes {
    let mut buf = BytesMut::with_capacity(7 + 4 * oids.len());
    buf.put_u8(b't');
    buf.put_i32((6 + 4 * oids.len()) as i32);
    buf.put_i16(oids.len() as i16);
    for &o in oids {
        buf.put_u32(o);
    }
    buf.freeze()
}

/// RowDescription with per-column format codes (`formats` follows the Bind
/// convention: empty = all text, one entry = applies to all, else one each).
/// Every column is reported as `text` (OID 25), whose binary representation
/// is the same UTF-8 bytes as its text form.
pub fn encode_row_description_with_formats(columns: &[String], formats: &[i16]) -> Bytes {
    let mut payload = BytesMut::new();
    payload.put_i16(columns.len() as i16);
    for (i, col) in columns.iter().enumerate() {
        let fmt = match formats {
            [] => 0,
            [f] => *f,
            fs => fs.get(i).copied().unwrap_or(0),
        };
        payload.put(col.as_bytes());
        payload.put_u8(0);
        payload.put_i32(0); // table oid
        payload.put_i16(0); // col attr
        payload.put_i32(25); // text oid
        payload.put_i16(-1); // variable length
        payload.put_i32(-1); // no modifier
        payload.put_i16(fmt);
    }
    let mut buf = BytesMut::with_capacity(1 + 4 + payload.len());
    buf.put_u8(b'T');
    buf.put_i32((4 + payload.len()) as i32);
    buf.put(payload);
    buf.freeze()
}

/// ErrorResponse (severity ERROR) with a SQLSTATE code.
pub fn encode_error_with_code(code: &str, msg: &str) -> Bytes {
    encode_notice_like(b'E', "ERROR", code, msg)
}

/// NoticeResponse ('N', severity NOTICE, SQLSTATE 00000).
pub fn encode_notice_response(msg: &str) -> Bytes {
    encode_notice_like(b'N', "NOTICE", "00000", msg)
}

fn encode_notice_like(t: u8, severity: &str, code: &str, msg: &str) -> Bytes {
    let mut payload = BytesMut::new();
    for (f, v) in [
        (b'S', severity),
        (b'V', severity),
        (b'C', code),
        (b'M', msg),
    ] {
        payload.put_u8(f);
        payload.put(v.as_bytes());
        payload.put_u8(0);
    }
    payload.put_u8(0); // terminator
    let mut buf = BytesMut::with_capacity(1 + 4 + payload.len());
    buf.put_u8(t);
    buf.put_i32((4 + payload.len()) as i32);
    buf.put(payload);
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
/// Emits severity ('S' / 'V'), SQLSTATE 'C' = XX000 (drivers such as
/// tokio-postgres require a code) and the message.
pub fn encode_error_response(msg: &str) -> Bytes {
    encode_error_with_code("XX000", msg)
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
        let types = [
            b'Q', b'X', b'p', b'P', b'B', b'D', b'E', b'C', b'S', b'H', b'Z', 0, 0xFF,
        ];
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
            let _ = decode_sasl_initial_response(&bytes);
            let _ = decode_sasl_response(&bytes);
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
