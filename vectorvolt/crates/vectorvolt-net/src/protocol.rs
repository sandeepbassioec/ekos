//! The VectorVolt wire protocol (VV-NET-001/002/003).
//!
//! A binary, length-prefixed, request/response protocol. Nothing here does I/O —
//! it is a pure codec over byte slices, so every framing and parsing decision can
//! be tested against hostile input without a socket.
//!
//! ## Frame layout
//!
//! ```text
//! ┌────────────────────────────────────────────┐
//! │ magic[2]  version[1]  opcode[1]            │
//! │ flags[1]  reserved[3]                      │
//! │ request_id[8]                              │
//! │ payload_len[4]  payload_crc32[4]           │
//! ├────────────────────────────────────────────┤
//! │ payload  (payload_len bytes)               │
//! └────────────────────────────────────────────┘
//! ```
//!
//! 24-byte header, little-endian throughout to match the storage format — one
//! byte order in the project means no conversion between the wire and the disk.
//!
//! ## Three decisions worth stating
//!
//! **`payload_len` is capped before it is trusted.** A length read from a socket
//! is an allocation request from an unauthenticated peer. Four bytes asking for 4
//! GiB, repeated, is a trivial denial of service. [`MAX_PAYLOAD`] bounds it, and
//! the cap is enforced in the decoder rather than left to callers.
//!
//! **`request_id` correlates, it does not order.** Responses may return in any
//! order, so a client can pipeline without waiting. The server echoes the id it
//! was given and never invents one, which means a client can always match a
//! response to its request even if it never sees some of them.
//!
//! **The CRC detects corruption, not tampering.** It catches a flipped bit from a
//! bad NIC or a truncated frame; it is not a MAC and provides no authenticity. TLS
//! or a signed transport is the answer to a hostile network, and pretending a
//! CRC32 helps there would be worse than having none.

use crate::{NetError, NetResult};

/// Frame magic. Distinguishes a VectorVolt frame from anything else that connects
/// to the port, so a stray HTTP request fails immediately instead of being parsed
/// as a length prefix.
pub const MAGIC: [u8; 2] = *b"VV";

/// Protocol version.
pub const VERSION: u8 = 1;

/// Fixed header size.
pub const HEADER_SIZE: usize = 24;

/// Largest payload accepted, in bytes.
///
/// 16 MiB. Comfortably above any realistic batch and small enough that a
/// misbehaving peer cannot exhaust memory by announcing large frames. A legitimate
/// caller needing more should send more frames, not bigger ones.
pub const MAX_PAYLOAD: u32 = 16 * 1024 * 1024;

// ── Opcodes ───────────────────────────────────────────────────────────────────

/// What a frame carries. Requests and responses share the space so a single
/// decoder can validate a byte against everything legal.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum OpCode {
    // Requests
    Ping = 1,
    Get = 2,
    Insert = 3,
    Delete = 4,
    RangeScan = 5,
    Flush = 6,
    Compact = 7,
    Stats = 8,
    Hello = 9,
    Auth = 10,
    SecondaryLookup = 11,
    FkExists = 12,
    /// Set the active table for this connection.
    UseTable = 13,
    /// Create a new table with the given schema.
    CreateTable = 14,
    /// Deregister a table from the catalog (data directory is not deleted).
    DropTable = 15,
    /// List all registered table names.
    ListTables = 16,
    /// Insert multiple rows in a single round trip.
    BatchInsert = 17,
    /// Delete multiple rows by primary key in a single round trip.
    BatchDelete = 18,
    /// Begin a server-side transaction on this connection.
    BeginTxn = 19,
    /// Commit the active transaction, applying all buffered ops atomically.
    CommitTxn = 20,
    /// Roll back the active transaction, discarding all buffered ops.
    RollbackTxn = 21,
    /// Insert or replace a row (delete existing, then insert).
    Upsert = 22,
    /// Fetch multiple rows by primary key in one round trip.
    GetMany = 23,
    /// Update an existing row; error if the primary key is not found.
    Update = 24,
    /// Delete all rows from the selected table.
    Truncate = 25,
    /// Count rows where column `col` falls in `[lo, hi]`.
    Count = 26,
    /// Insert-or-replace multiple rows in a single round trip.
    BatchUpsert = 27,
    /// Update multiple existing rows in a single round trip.
    BatchUpdate = 28,
    /// Return the column schema for the currently selected table.
    DescribeTable = 29,
    /// Check whether a named table exists in the catalog.
    TableExists = 30,
    /// Range scan returning only primary keys (no values).
    ScanKeys = 31,
    // Responses
    Pong = 129,
    Row = 130,
    NotFound = 131,
    Ok = 132,
    Rows = 133,
    StatsReply = 134,
    Error = 135,
    Challenge = 136,
    Exists = 137,
    /// The server accepted a `UseTable` or `CreateTable` — table is now selected.
    TableSelected = 138,
    /// Response to `ListTables`.
    TableList = 139,
    /// Response to `BatchInsert` or `BatchDelete`.
    BatchResult = 140,
    /// Response to `BeginTxn` — carries the new transaction ID.
    TxnStarted = 141,
    /// Response to `GetMany` — only found rows, each with its pk.
    RowSet = 142,
    /// Response to `DescribeTable` — the column names for the selected table.
    Schema = 143,
    /// Response to `ScanKeys` — primary keys only.
    Keys = 144,
}

impl OpCode {
    /// Parse an opcode byte. Unknown values are rejected rather than ignored: a
    /// peer speaking a dialect this build does not know should be told so, not
    /// have its request silently dropped.
    pub fn from_u8(b: u8) -> NetResult<Self> {
        use OpCode::*;
        // Fully-qualified Result::Ok, because `OpCode::Ok` is in scope here and
        // shadows it — a genuinely confusing collision that the compiler catches.
        core::result::Result::Ok(match b {
            1 => Ping,
            2 => Get,
            3 => Insert,
            4 => Delete,
            5 => RangeScan,
            6 => Flush,
            7 => Compact,
            8 => Stats,
            9 => Hello,
            10 => Auth,
            11 => SecondaryLookup,
            12 => FkExists,
            13 => UseTable,
            14 => CreateTable,
            15 => DropTable,
            16 => ListTables,
            17 => BatchInsert,
            18 => BatchDelete,
            19 => BeginTxn,
            20 => CommitTxn,
            21 => RollbackTxn,
            22 => Upsert,
            23 => GetMany,
            24 => Update,
            25 => Truncate,
            26 => Count,
            27 => BatchUpsert,
            28 => BatchUpdate,
            29 => DescribeTable,
            30 => TableExists,
            31 => ScanKeys,
            129 => Pong,
            130 => Row,
            131 => NotFound,
            132 => Ok,
            133 => Rows,
            134 => StatsReply,
            135 => Error,
            136 => Challenge,
            137 => Exists,
            138 => TableSelected,
            139 => TableList,
            140 => BatchResult,
            141 => TxnStarted,
            142 => RowSet,
            143 => Schema,
            144 => Keys,
            other => return Err(NetError::UnknownOpcode(other)),
        })
    }

    /// True for opcodes a client sends. Lets the server reject a response opcode
    /// arriving on an inbound connection instead of trying to serve it.
    pub fn is_request(self) -> bool {
        (self as u8) < 128
    }
}

// ── Header ────────────────────────────────────────────────────────────────────

/// The fixed part of a frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Header {
    pub opcode: OpCode,
    pub flags: u8,
    pub request_id: u64,
    pub payload_len: u32,
    pub payload_crc: u32,
}

impl Header {
    /// Parse a header from exactly [`HEADER_SIZE`] bytes.
    ///
    /// Validates magic, version, opcode, and the payload cap **before** the caller
    /// allocates anything. That ordering is the point: the cap is useless if the
    /// buffer is reserved first.
    pub fn decode(buf: &[u8]) -> NetResult<Self> {
        if buf.len() < HEADER_SIZE {
            return Err(NetError::Truncated {
                need: HEADER_SIZE,
                got: buf.len(),
            });
        }
        if buf[0..2] != MAGIC {
            return Err(NetError::BadMagic);
        }
        if buf[2] != VERSION {
            return Err(NetError::UnsupportedVersion(buf[2]));
        }
        let opcode = OpCode::from_u8(buf[3])?;
        let flags = buf[4];
        // buf[5..8] reserved
        let request_id = u64::from_le_bytes(buf[8..16].try_into().unwrap());
        let payload_len = u32::from_le_bytes(buf[16..20].try_into().unwrap());
        let payload_crc = u32::from_le_bytes(buf[20..24].try_into().unwrap());

        if payload_len > MAX_PAYLOAD {
            return Err(NetError::PayloadTooLarge {
                len: payload_len,
                max: MAX_PAYLOAD,
            });
        }
        Ok(Header {
            opcode,
            flags,
            request_id,
            payload_len,
            payload_crc,
        })
    }

    pub fn encode(&self) -> [u8; HEADER_SIZE] {
        let mut b = [0u8; HEADER_SIZE];
        b[0..2].copy_from_slice(&MAGIC);
        b[2] = VERSION;
        b[3] = self.opcode as u8;
        b[4] = self.flags;
        b[8..16].copy_from_slice(&self.request_id.to_le_bytes());
        b[16..20].copy_from_slice(&self.payload_len.to_le_bytes());
        b[20..24].copy_from_slice(&self.payload_crc.to_le_bytes());
        b
    }
}

// ── Messages ──────────────────────────────────────────────────────────────────

/// A client request.
#[derive(Debug, Clone, PartialEq)]
pub enum Request {
    Ping,
    Get {
        pk: u64,
    },
    Insert {
        pk: u64,
        values: Vec<u64>,
    },
    Delete {
        pk: u64,
    },
    RangeScan {
        col: u32,
        lo: u64,
        hi: u64,
        limit: u32,
    },
    Flush,
    Compact,
    Stats,
    /// Ask for an authentication challenge.
    Hello,
    /// Answer a challenge. `response` is `SHA3-256(secret ‖ nonce)`.
    Auth {
        key_id: String,
        response: Vec<u8>,
    },
    /// Secondary-index lookup: all rows where column `col_idx` equals `value`.
    SecondaryLookup {
        col_idx: u32,
        value: u64,
    },
    /// Probabilistic FK existence check for a PK value.
    FkExists {
        value: u64,
    },
    /// Select the active table for this connection. All subsequent data
    /// operations (Get, Insert, Scan, …) operate on the selected table.
    UseTable {
        name: String,
    },
    /// Create a new table with the given column schema and select it.
    CreateTable {
        name: String,
        /// Column names in schema order. Column 0 is always the primary key.
        schema: Vec<String>,
    },
    /// Remove a table from the catalog. The data directory is NOT deleted.
    DropTable {
        name: String,
    },
    /// List all registered table names.
    ListTables,
    /// Insert multiple rows in a single round trip.
    ///
    /// Each entry is `(pk, values)`. Duplicate-key rows are silently skipped;
    /// the server returns how many succeeded and how many were skipped.
    BatchInsert {
        rows: Vec<(u64, Vec<u64>)>,
    },
    /// Delete multiple rows by primary key in a single round trip.
    ///
    /// Missing keys are silently skipped; the server returns how many rows
    /// were actually deleted.
    BatchDelete {
        pks: Vec<u64>,
    },
    /// Begin a server-side transaction on this connection.
    ///
    /// Subsequent `Insert` and `Delete` requests are buffered until
    /// `CommitTxn` (applied atomically) or `RollbackTxn` (discarded).
    /// Only one transaction may be active at a time per connection.
    BeginTxn,
    /// Commit the active transaction, applying all buffered ops to the engine.
    CommitTxn {
        txn_id: u64,
    },
    /// Discard all buffered ops for the active transaction.
    RollbackTxn {
        txn_id: u64,
    },
    /// Insert-or-replace: delete the existing row (if any), then insert.
    ///
    /// Never returns a duplicate-key error. Always returns `Ok { affected: 1 }`.
    Upsert {
        pk: u64,
        values: Vec<u64>,
    },
    /// Fetch multiple rows by primary key in one round trip.
    ///
    /// Missing keys are silently omitted from the response.
    GetMany {
        pks: Vec<u64>,
    },
    /// Update an existing row. Returns an error if `pk` is not found.
    Update {
        pk: u64,
        values: Vec<u64>,
    },
    /// Delete all rows from the selected table.
    ///
    /// Returns `Ok { affected }` where `affected` is the number of rows removed.
    Truncate,
    /// Count rows where column `col` value falls in `[lo, hi]` (both inclusive).
    ///
    /// Returns `Ok { affected }` where `affected` is the matching row count.
    Count {
        col: u32,
        lo: u64,
        hi: u64,
    },
    /// Insert-or-replace multiple rows in a single round trip.
    ///
    /// Each row is processed as an independent upsert (delete-then-insert).
    /// Returns `Ok { affected }` equal to the number of rows submitted.
    BatchUpsert {
        rows: Vec<(u64, Vec<u64>)>,
    },
    /// Update multiple existing rows in a single round trip.
    ///
    /// Rows whose primary key is not found are silently skipped.
    /// Returns `BatchResult { succeeded, skipped }`.
    BatchUpdate {
        rows: Vec<(u64, Vec<u64>)>,
    },
    /// Return the column names for the currently selected table.
    ///
    /// Returns `Schema { columns }`. Requires a table to be selected first.
    DescribeTable,
    /// Check whether a named table exists in the server catalog.
    ///
    /// Returns `Exists { found }`. Does not require a table to be selected.
    TableExists {
        name: String,
    },
    /// Range scan returning only primary keys (no column values).
    ///
    /// Same filter semantics as `RangeScan`: column `col`, range `[lo, hi]`,
    /// optional `limit` (0 = unlimited). Returns `Keys { pks }`.
    ScanKeys {
        col: u32,
        lo: u64,
        hi: u64,
        limit: u32,
    },
}

/// A server response.
#[derive(Debug, Clone, PartialEq)]
pub enum Response {
    Pong,
    Row {
        values: Vec<u64>,
    },
    NotFound,
    Ok {
        affected: u64,
    },
    Rows {
        rows: Vec<Vec<u64>>,
    },
    Stats {
        rows: u64,
        partitions: u64,
        bytes: u64,
    },
    Error {
        code: u16,
        message: String,
    },
    /// A per-connection nonce to authenticate against.
    Challenge {
        nonce: Vec<u8>,
    },
    /// Result of a `FkExists` request.
    Exists {
        found: bool,
    },
    /// The server accepted `UseTable` or `CreateTable`; the table is selected.
    TableSelected,
    /// Response to `ListTables`.
    TableList {
        names: Vec<String>,
    },
    /// Response to `BatchInsert` or `BatchDelete`.
    BatchResult {
        /// Rows successfully written or deleted.
        succeeded: u32,
        /// Rows skipped (duplicate key for insert, missing key for delete).
        skipped: u32,
    },
    /// Response to `BeginTxn` — the client must echo `txn_id` in CommitTxn/RollbackTxn.
    TxnStarted {
        txn_id: u64,
    },
    /// Response to `GetMany` — found rows only, each entry is `(pk, values)`.
    RowSet {
        entries: Vec<(u64, Vec<u64>)>,
    },
    /// Response to `DescribeTable` — column names in schema order.
    Schema {
        columns: Vec<String>,
    },
    /// Response to `ScanKeys` — primary keys only, in scan order.
    Keys {
        pks: Vec<u64>,
    },
}

/// A small write cursor. Every field is appended little-endian.
struct W(Vec<u8>);

impl W {
    fn new() -> Self {
        W(Vec::with_capacity(32))
    }
    fn u16(&mut self, v: u16) -> &mut Self {
        self.0.extend_from_slice(&v.to_le_bytes());
        self
    }
    fn u32(&mut self, v: u32) -> &mut Self {
        self.0.extend_from_slice(&v.to_le_bytes());
        self
    }
    fn u64(&mut self, v: u64) -> &mut Self {
        self.0.extend_from_slice(&v.to_le_bytes());
        self
    }
    fn u64s(&mut self, vs: &[u64]) -> NetResult<&mut Self> {
        let n = u32::try_from(vs.len()).map_err(|_| NetError::Malformed("slice too long"))?;
        self.u32(n);
        for v in vs {
            self.u64(*v);
        }
        Ok(self)
    }
    fn string(&mut self, s: &str) -> NetResult<&mut Self> {
        let b = s.as_bytes();
        let n = u32::try_from(b.len()).map_err(|_| NetError::Malformed("string too long"))?;
        self.u32(n);
        self.0.extend_from_slice(b);
        Ok(self)
    }
    fn strings(&mut self, ss: &[String]) -> NetResult<&mut Self> {
        let n = u32::try_from(ss.len()).map_err(|_| NetError::Malformed("too many strings"))?;
        self.u32(n);
        for s in ss {
            self.string(s)?;
        }
        Ok(self)
    }
}

/// A bounds-checked read cursor.
///
/// Every read goes through this. The payload came off a socket, so a length field
/// inside it can name anything, and `&buf[a..b]` would panic — killing the
/// connection handler, or the server, on a malformed frame.
struct R<'a> {
    b: &'a [u8],
    at: usize,
}

impl<'a> R<'a> {
    fn new(b: &'a [u8]) -> Self {
        R { b, at: 0 }
    }
    fn take(&mut self, n: usize) -> NetResult<&'a [u8]> {
        let end = self
            .at
            .checked_add(n)
            .ok_or(NetError::Malformed("length overflow"))?;
        let s = self
            .b
            .get(self.at..end)
            .ok_or(NetError::Malformed("payload truncated"))?;
        self.at = end;
        Ok(s)
    }
    fn u16(&mut self) -> NetResult<u16> {
        Ok(u16::from_le_bytes(self.take(2)?.try_into().unwrap()))
    }
    fn u32(&mut self) -> NetResult<u32> {
        Ok(u32::from_le_bytes(self.take(4)?.try_into().unwrap()))
    }
    fn u64(&mut self) -> NetResult<u64> {
        Ok(u64::from_le_bytes(self.take(8)?.try_into().unwrap()))
    }
    /// Read a length-prefixed `u64` slice.
    ///
    /// The count is validated against the bytes actually remaining before any
    /// allocation. Without that, a 4-byte count of `u32::MAX` in a 12-byte payload
    /// reserves 32 GiB — the frame cap does not help, because the lie is inside
    /// an otherwise legal frame.
    fn u64s(&mut self) -> NetResult<Vec<u64>> {
        let n = self.u32()? as usize;
        let available = (self.b.len() - self.at) / 8;
        if n > available {
            return Err(NetError::Malformed(
                "declared more values than the payload holds",
            ));
        }
        let mut out = Vec::with_capacity(n);
        for _ in 0..n {
            out.push(self.u64()?);
        }
        Ok(out)
    }
    /// Read a length-prefixed byte string.
    ///
    /// The length is validated against what remains before allocating, for the
    /// same reason as `u64s`: the count sits inside an otherwise legal frame, so
    /// the frame cap does not bound it.
    fn bytes(&mut self) -> NetResult<Vec<u8>> {
        let n = self.u32()? as usize;
        Ok(self.take(n)?.to_vec())
    }

    fn string(&mut self) -> NetResult<String> {
        let n = self.u32()? as usize;
        let bytes = self.take(n)?;
        String::from_utf8(bytes.to_vec()).map_err(|_| NetError::Malformed("string is not UTF-8"))
    }
    fn strings(&mut self) -> NetResult<Vec<String>> {
        let n = self.u32()? as usize;
        // Each string costs at least 4 bytes (length prefix).
        let max_possible = (self.b.len() - self.at) / 4;
        if n > max_possible {
            return Err(NetError::Malformed(
                "declared more strings than the payload holds",
            ));
        }
        let mut out = Vec::with_capacity(n);
        for _ in 0..n {
            out.push(self.string()?);
        }
        Ok(out)
    }
    /// Error if anything is left over.
    ///
    /// Trailing bytes mean the sender and this decoder disagree about the message,
    /// which is a version or corruption problem worth surfacing rather than a
    /// remainder to ignore.
    fn finish(&self) -> NetResult<()> {
        if self.at != self.b.len() {
            return Err(NetError::Malformed("trailing bytes after message"));
        }
        Ok(())
    }
}

impl Request {
    pub fn opcode(&self) -> OpCode {
        match self {
            Request::Ping => OpCode::Ping,
            Request::Get { .. } => OpCode::Get,
            Request::Insert { .. } => OpCode::Insert,
            Request::Delete { .. } => OpCode::Delete,
            Request::RangeScan { .. } => OpCode::RangeScan,
            Request::Flush => OpCode::Flush,
            Request::Compact => OpCode::Compact,
            Request::Stats => OpCode::Stats,
            Request::Hello => OpCode::Hello,
            Request::Auth { .. } => OpCode::Auth,
            Request::SecondaryLookup { .. } => OpCode::SecondaryLookup,
            Request::FkExists { .. } => OpCode::FkExists,
            Request::UseTable { .. } => OpCode::UseTable,
            Request::CreateTable { .. } => OpCode::CreateTable,
            Request::DropTable { .. } => OpCode::DropTable,
            Request::ListTables => OpCode::ListTables,
            Request::BatchInsert { .. } => OpCode::BatchInsert,
            Request::BatchDelete { .. } => OpCode::BatchDelete,
            Request::BeginTxn => OpCode::BeginTxn,
            Request::CommitTxn { .. } => OpCode::CommitTxn,
            Request::RollbackTxn { .. } => OpCode::RollbackTxn,
            Request::Upsert { .. } => OpCode::Upsert,
            Request::GetMany { .. } => OpCode::GetMany,
            Request::Update { .. } => OpCode::Update,
            Request::Truncate => OpCode::Truncate,
            Request::Count { .. } => OpCode::Count,
            Request::BatchUpsert { .. } => OpCode::BatchUpsert,
            Request::BatchUpdate { .. } => OpCode::BatchUpdate,
            Request::DescribeTable => OpCode::DescribeTable,
            Request::TableExists { .. } => OpCode::TableExists,
            Request::ScanKeys { .. } => OpCode::ScanKeys,
        }
    }

    pub fn encode_payload(&self) -> NetResult<Vec<u8>> {
        let mut w = W::new();
        match self {
            Request::Ping
            | Request::Flush
            | Request::Compact
            | Request::Stats
            | Request::Hello
            | Request::DescribeTable => {}
            Request::TableExists { name } => {
                w.string(name)?;
            }
            Request::ScanKeys { col, lo, hi, limit } => {
                w.u32(*col).u64(*lo).u64(*hi).u32(*limit);
            }
            Request::Auth { key_id, response } => {
                let k = key_id.as_bytes();
                let n =
                    u32::try_from(k.len()).map_err(|_| NetError::Malformed("key id too long"))?;
                w.u32(n);
                w.0.extend_from_slice(k);
                let n = u32::try_from(response.len())
                    .map_err(|_| NetError::Malformed("response too long"))?;
                w.u32(n);
                w.0.extend_from_slice(response);
            }
            Request::Get { pk } | Request::Delete { pk } => {
                w.u64(*pk);
            }
            Request::Insert { pk, values } => {
                w.u64(*pk);
                w.u64s(values)?;
            }
            Request::RangeScan { col, lo, hi, limit } => {
                w.u32(*col).u64(*lo).u64(*hi).u32(*limit);
            }
            Request::SecondaryLookup { col_idx, value } => {
                w.u32(*col_idx).u64(*value);
            }
            Request::FkExists { value } => {
                w.u64(*value);
            }
            Request::UseTable { name } | Request::DropTable { name } => {
                w.string(name)?;
            }
            Request::CreateTable { name, schema } => {
                w.string(name)?.strings(schema)?;
            }
            Request::ListTables => {}
            Request::BatchInsert { rows } => {
                let n = u32::try_from(rows.len())
                    .map_err(|_| NetError::Malformed("too many rows in BatchInsert"))?;
                w.u32(n);
                for (pk, values) in rows {
                    w.u64(*pk);
                    w.u64s(values)?;
                }
            }
            Request::BatchDelete { pks } => {
                let n = u32::try_from(pks.len())
                    .map_err(|_| NetError::Malformed("too many keys in BatchDelete"))?;
                w.u32(n);
                for pk in pks {
                    w.u64(*pk);
                }
            }
            Request::BeginTxn => {}
            Request::CommitTxn { txn_id } | Request::RollbackTxn { txn_id } => {
                w.u64(*txn_id);
            }
            Request::Upsert { pk, values } => {
                w.u64(*pk);
                w.u64s(values)?;
            }
            Request::GetMany { pks } => {
                let n = u32::try_from(pks.len())
                    .map_err(|_| NetError::Malformed("too many keys in GetMany"))?;
                w.u32(n);
                for pk in pks {
                    w.u64(*pk);
                }
            }
            Request::Update { pk, values } => {
                w.u64(*pk);
                w.u64s(values)?;
            }
            Request::Truncate => {}
            Request::Count { col, lo, hi } => {
                w.u32(*col).u64(*lo).u64(*hi);
            }
            Request::BatchUpsert { rows } | Request::BatchUpdate { rows } => {
                let n = u32::try_from(rows.len())
                    .map_err(|_| NetError::Malformed("too many rows in batch"))?;
                w.u32(n);
                for (pk, values) in rows {
                    w.u64(*pk);
                    w.u64s(values)?;
                }
            }
        }
        Ok(w.0)
    }

    pub fn decode(opcode: OpCode, payload: &[u8]) -> NetResult<Self> {
        let mut r = R::new(payload);
        let msg = match opcode {
            OpCode::Ping => Request::Ping,
            OpCode::Flush => Request::Flush,
            OpCode::Compact => Request::Compact,
            OpCode::Stats => Request::Stats,
            OpCode::Hello => Request::Hello,
            OpCode::Auth => Request::Auth {
                key_id: r.string()?,
                response: r.bytes()?,
            },
            OpCode::Get => Request::Get { pk: r.u64()? },
            OpCode::Delete => Request::Delete { pk: r.u64()? },
            OpCode::Insert => Request::Insert {
                pk: r.u64()?,
                values: r.u64s()?,
            },
            OpCode::RangeScan => Request::RangeScan {
                col: r.u32()?,
                lo: r.u64()?,
                hi: r.u64()?,
                limit: r.u32()?,
            },
            OpCode::SecondaryLookup => Request::SecondaryLookup {
                col_idx: r.u32()?,
                value: r.u64()?,
            },
            OpCode::FkExists => Request::FkExists { value: r.u64()? },
            OpCode::UseTable => Request::UseTable { name: r.string()? },
            OpCode::CreateTable => Request::CreateTable {
                name: r.string()?,
                schema: r.strings()?,
            },
            OpCode::DropTable => Request::DropTable { name: r.string()? },
            OpCode::ListTables => Request::ListTables,
            OpCode::BatchInsert => {
                let n = r.u32()? as usize;
                let max_possible = (r.b.len() - r.at) / 8;
                if n > max_possible {
                    return Err(NetError::Malformed("declared more rows than payload holds"));
                }
                let mut rows = Vec::with_capacity(n);
                for _ in 0..n {
                    let pk = r.u64()?;
                    let values = r.u64s()?;
                    rows.push((pk, values));
                }
                Request::BatchInsert { rows }
            }
            OpCode::BatchDelete => {
                let n = r.u32()? as usize;
                let max_possible = (r.b.len() - r.at) / 8;
                if n > max_possible {
                    return Err(NetError::Malformed("declared more keys than payload holds"));
                }
                let mut pks = Vec::with_capacity(n);
                for _ in 0..n {
                    pks.push(r.u64()?);
                }
                Request::BatchDelete { pks }
            }
            OpCode::BeginTxn => Request::BeginTxn,
            OpCode::CommitTxn => Request::CommitTxn { txn_id: r.u64()? },
            OpCode::RollbackTxn => Request::RollbackTxn { txn_id: r.u64()? },
            OpCode::Upsert => Request::Upsert {
                pk: r.u64()?,
                values: r.u64s()?,
            },
            OpCode::GetMany => {
                let n = r.u32()? as usize;
                let max_possible = (r.b.len() - r.at) / 8;
                if n > max_possible {
                    return Err(NetError::Malformed("declared more keys than payload holds"));
                }
                let mut pks = Vec::with_capacity(n);
                for _ in 0..n {
                    pks.push(r.u64()?);
                }
                Request::GetMany { pks }
            }
            OpCode::Update => Request::Update {
                pk: r.u64()?,
                values: r.u64s()?,
            },
            OpCode::Truncate => Request::Truncate,
            OpCode::Count => Request::Count {
                col: r.u32()?,
                lo: r.u64()?,
                hi: r.u64()?,
            },
            OpCode::BatchUpsert | OpCode::BatchUpdate => {
                let n = r.u32()? as usize;
                let max_possible = (r.b.len() - r.at) / 8;
                if n > max_possible {
                    return Err(NetError::Malformed("declared more rows than payload holds"));
                }
                let mut rows = Vec::with_capacity(n);
                for _ in 0..n {
                    let pk = r.u64()?;
                    let values = r.u64s()?;
                    rows.push((pk, values));
                }
                if opcode == OpCode::BatchUpsert {
                    Request::BatchUpsert { rows }
                } else {
                    Request::BatchUpdate { rows }
                }
            }
            OpCode::DescribeTable => Request::DescribeTable,
            OpCode::TableExists => Request::TableExists { name: r.string()? },
            OpCode::ScanKeys => Request::ScanKeys {
                col: r.u32()?,
                lo: r.u64()?,
                hi: r.u64()?,
                limit: r.u32()?,
            },
            other => return Err(NetError::NotARequest(other as u8)),
        };
        r.finish()?;
        Ok(msg)
    }
}

impl Response {
    pub fn opcode(&self) -> OpCode {
        match self {
            Response::Pong => OpCode::Pong,
            Response::Row { .. } => OpCode::Row,
            Response::NotFound => OpCode::NotFound,
            Response::Ok { .. } => OpCode::Ok,
            Response::Rows { .. } => OpCode::Rows,
            Response::Stats { .. } => OpCode::StatsReply,
            Response::Challenge { .. } => OpCode::Challenge,
            Response::Error { .. } => OpCode::Error,
            Response::Exists { .. } => OpCode::Exists,
            Response::TableSelected => OpCode::TableSelected,
            Response::TableList { .. } => OpCode::TableList,
            Response::BatchResult { .. } => OpCode::BatchResult,
            Response::TxnStarted { .. } => OpCode::TxnStarted,
            Response::RowSet { .. } => OpCode::RowSet,
            Response::Schema { .. } => OpCode::Schema,
            Response::Keys { .. } => OpCode::Keys,
        }
    }

    pub fn encode_payload(&self) -> NetResult<Vec<u8>> {
        let mut w = W::new();
        match self {
            Response::Pong | Response::NotFound => {}
            Response::Row { values } => {
                w.u64s(values)?;
            }
            Response::Ok { affected } => {
                w.u64(*affected);
            }
            Response::Rows { rows } => {
                let n =
                    u32::try_from(rows.len()).map_err(|_| NetError::Malformed("too many rows"))?;
                w.u32(n);
                for row in rows {
                    w.u64s(row)?;
                }
            }
            Response::Stats {
                rows,
                partitions,
                bytes,
            } => {
                w.u64(*rows).u64(*partitions).u64(*bytes);
            }
            Response::Challenge { nonce } => {
                let n = u32::try_from(nonce.len())
                    .map_err(|_| NetError::Malformed("nonce too long"))?;
                w.u32(n);
                w.0.extend_from_slice(nonce);
            }
            Response::Error { code, message } => {
                w.u16(*code);
                let b = message.as_bytes();
                let n =
                    u32::try_from(b.len()).map_err(|_| NetError::Malformed("message too long"))?;
                w.u32(n);
                w.0.extend_from_slice(b);
            }
            Response::Exists { found } => {
                w.0.push(if *found { 1 } else { 0 });
            }
            Response::TableSelected => {}
            Response::TableList { names } => {
                w.strings(names)?;
            }
            Response::BatchResult { succeeded, skipped } => {
                w.u32(*succeeded).u32(*skipped);
            }
            Response::TxnStarted { txn_id } => {
                w.u64(*txn_id);
            }
            Response::RowSet { entries } => {
                let n = u32::try_from(entries.len())
                    .map_err(|_| NetError::Malformed("too many entries in RowSet"))?;
                w.u32(n);
                for (pk, values) in entries {
                    w.u64(*pk);
                    w.u64s(values)?;
                }
            }
            Response::Schema { columns } => {
                w.strings(columns)?;
            }
            Response::Keys { pks } => {
                let n =
                    u32::try_from(pks.len()).map_err(|_| NetError::Malformed("too many keys"))?;
                w.u32(n);
                for pk in pks {
                    w.u64(*pk);
                }
            }
        }
        Ok(w.0)
    }

    pub fn decode(opcode: OpCode, payload: &[u8]) -> NetResult<Self> {
        let mut r = R::new(payload);
        let msg = match opcode {
            OpCode::Pong => Response::Pong,
            OpCode::NotFound => Response::NotFound,
            OpCode::Row => Response::Row { values: r.u64s()? },
            OpCode::Ok => Response::Ok { affected: r.u64()? },
            OpCode::Rows => {
                let n = r.u32()? as usize;
                // Each row costs at least 4 bytes (its own count), so a row count
                // above that bound is a lie regardless of what follows.
                let max_possible = (r.b.len() - r.at) / 4;
                if n > max_possible {
                    return Err(NetError::Malformed(
                        "declared more rows than the payload holds",
                    ));
                }
                let mut rows = Vec::with_capacity(n);
                for _ in 0..n {
                    rows.push(r.u64s()?);
                }
                Response::Rows { rows }
            }
            OpCode::StatsReply => Response::Stats {
                rows: r.u64()?,
                partitions: r.u64()?,
                bytes: r.u64()?,
            },
            OpCode::Challenge => Response::Challenge { nonce: r.bytes()? },
            OpCode::Error => Response::Error {
                code: r.u16()?,
                message: r.string()?,
            },
            OpCode::Exists => {
                let byte = r.take(1)?[0];
                Response::Exists { found: byte != 0 }
            }
            OpCode::TableSelected => Response::TableSelected,
            OpCode::TableList => Response::TableList {
                names: r.strings()?,
            },
            OpCode::BatchResult => Response::BatchResult {
                succeeded: r.u32()?,
                skipped: r.u32()?,
            },
            OpCode::TxnStarted => Response::TxnStarted { txn_id: r.u64()? },
            OpCode::RowSet => {
                let n = r.u32()? as usize;
                let max_possible = (r.b.len() - r.at) / 4;
                if n > max_possible {
                    return Err(NetError::Malformed(
                        "declared more entries than the payload holds",
                    ));
                }
                let mut entries = Vec::with_capacity(n);
                for _ in 0..n {
                    let pk = r.u64()?;
                    let values = r.u64s()?;
                    entries.push((pk, values));
                }
                Response::RowSet { entries }
            }
            OpCode::Schema => Response::Schema {
                columns: r.strings()?,
            },
            OpCode::Keys => {
                let n = r.u32()? as usize;
                let max_possible = (r.b.len() - r.at) / 8;
                if n > max_possible {
                    return Err(NetError::Malformed("declared more keys than payload holds"));
                }
                let mut pks = Vec::with_capacity(n);
                for _ in 0..n {
                    pks.push(r.u64()?);
                }
                Response::Keys { pks }
            }
            other => return Err(NetError::NotAResponse(other as u8)),
        };
        r.finish()?;
        Ok(msg)
    }
}

// ── Whole-frame helpers ───────────────────────────────────────────────────────

/// Serialize a full frame: header plus payload.
pub fn encode_frame(request_id: u64, opcode: OpCode, payload: &[u8]) -> NetResult<Vec<u8>> {
    let payload_len =
        u32::try_from(payload.len()).map_err(|_| NetError::Malformed("payload too long"))?;
    if payload_len > MAX_PAYLOAD {
        return Err(NetError::PayloadTooLarge {
            len: payload_len,
            max: MAX_PAYLOAD,
        });
    }
    let header = Header {
        opcode,
        flags: 0,
        request_id,
        payload_len,
        payload_crc: crc32fast::hash(payload),
    };
    let mut out = Vec::with_capacity(HEADER_SIZE + payload.len());
    out.extend_from_slice(&header.encode());
    out.extend_from_slice(payload);
    Ok(out)
}

pub fn encode_request(request_id: u64, req: &Request) -> NetResult<Vec<u8>> {
    encode_frame(request_id, req.opcode(), &req.encode_payload()?)
}

pub fn encode_response(request_id: u64, resp: &Response) -> NetResult<Vec<u8>> {
    encode_frame(request_id, resp.opcode(), &resp.encode_payload()?)
}

/// Verify a payload against the CRC its header declared.
pub fn verify_payload(header: &Header, payload: &[u8]) -> NetResult<()> {
    if payload.len() != header.payload_len as usize {
        return Err(NetError::Truncated {
            need: header.payload_len as usize,
            got: payload.len(),
        });
    }
    let actual = crc32fast::hash(payload);
    if actual != header.payload_crc {
        return Err(NetError::CrcMismatch {
            expected: header.payload_crc,
            actual,
        });
    }
    Ok(())
}

// ── tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn requests() -> Vec<Request> {
        vec![
            Request::Ping,
            Request::Flush,
            Request::Compact,
            Request::Stats,
            Request::Get { pk: 0 },
            Request::Get { pk: u64::MAX },
            Request::Delete { pk: 42 },
            Request::Insert {
                pk: 7,
                values: vec![],
            },
            Request::Insert {
                pk: u64::MAX,
                values: vec![0, u64::MAX, 1],
            },
            Request::Insert {
                pk: 1,
                values: (0..1000).collect(),
            },
            Request::RangeScan {
                col: 0,
                lo: 0,
                hi: u64::MAX,
                limit: 0,
            },
            Request::RangeScan {
                col: u32::MAX,
                lo: 5,
                hi: 5,
                limit: u32::MAX,
            },
            Request::SecondaryLookup {
                col_idx: 0,
                value: 0,
            },
            Request::SecondaryLookup {
                col_idx: u32::MAX,
                value: u64::MAX,
            },
            Request::FkExists { value: 0 },
            Request::FkExists { value: u64::MAX },
            Request::UseTable {
                name: "default".into(),
            },
            Request::CreateTable {
                name: "orders".into(),
                schema: vec!["pk".into(), "amount".into(), "customer_id".into()],
            },
            Request::CreateTable {
                name: "t".into(),
                schema: vec![],
            },
            Request::DropTable {
                name: "orders".into(),
            },
            Request::ListTables,
            Request::BatchInsert { rows: vec![] },
            Request::BatchInsert {
                rows: vec![(1, vec![1, 10]), (2, vec![2, 20]), (u64::MAX, vec![])],
            },
            Request::BatchDelete { pks: vec![] },
            Request::BatchDelete {
                pks: vec![1, 2, u64::MAX],
            },
            Request::BeginTxn,
            Request::CommitTxn { txn_id: 1 },
            Request::CommitTxn { txn_id: u64::MAX },
            Request::RollbackTxn { txn_id: 1 },
            Request::RollbackTxn { txn_id: u64::MAX },
            Request::Upsert {
                pk: 5,
                values: vec![],
            },
            Request::Upsert {
                pk: u64::MAX,
                values: vec![0, 1, u64::MAX],
            },
            Request::GetMany { pks: vec![] },
            Request::GetMany {
                pks: vec![1, 2, u64::MAX],
            },
            Request::Update {
                pk: 1,
                values: vec![1, 42],
            },
            Request::Update {
                pk: u64::MAX,
                values: vec![],
            },
            Request::Truncate,
            Request::Count {
                col: 0,
                lo: 0,
                hi: u64::MAX,
            },
            Request::Count {
                col: 3,
                lo: 100,
                hi: 200,
            },
            Request::BatchUpsert { rows: vec![] },
            Request::BatchUpsert {
                rows: vec![(1, vec![1, 10]), (2, vec![2, 20]), (u64::MAX, vec![])],
            },
            Request::BatchUpdate { rows: vec![] },
            Request::BatchUpdate {
                rows: vec![(1, vec![1, 99]), (u64::MAX, vec![0])],
            },
            Request::DescribeTable,
            Request::TableExists {
                name: "default".into(),
            },
            Request::TableExists {
                name: String::new(),
            },
            Request::ScanKeys {
                col: 0,
                lo: 0,
                hi: u64::MAX,
                limit: 0,
            },
            Request::ScanKeys {
                col: 2,
                lo: 100,
                hi: 200,
                limit: 50,
            },
        ]
    }

    fn responses() -> Vec<Response> {
        vec![
            Response::Pong,
            Response::NotFound,
            Response::Ok { affected: 0 },
            Response::Ok { affected: u64::MAX },
            Response::Row { values: vec![] },
            Response::Row {
                values: vec![1, 2, u64::MAX],
            },
            Response::Rows { rows: vec![] },
            Response::Rows {
                rows: vec![vec![], vec![1], vec![1, 2, 3]],
            },
            Response::Rows {
                rows: (0..100).map(|i| vec![i, i * 2, i * 3]).collect(),
            },
            Response::Stats {
                rows: 1,
                partitions: 2,
                bytes: 3,
            },
            Response::Error {
                code: 0,
                message: String::new(),
            },
            Response::Error {
                code: u16::MAX,
                message: "something went wrong — with unicode: 日本語".into(),
            },
            Response::Exists { found: false },
            Response::Exists { found: true },
            Response::TableSelected,
            Response::TableList { names: vec![] },
            Response::TableList {
                names: vec!["default".into(), "orders".into(), "products".into()],
            },
            Response::BatchResult {
                succeeded: 0,
                skipped: 0,
            },
            Response::BatchResult {
                succeeded: 1000,
                skipped: 3,
            },
            Response::TxnStarted { txn_id: 0 },
            Response::TxnStarted { txn_id: u64::MAX },
            Response::RowSet { entries: vec![] },
            Response::RowSet {
                entries: vec![(1, vec![1, 10]), (2, vec![2, 20])],
            },
            Response::Schema { columns: vec![] },
            Response::Schema {
                columns: vec!["pk".into(), "name".into(), "score".into()],
            },
            Response::Keys { pks: vec![] },
            Response::Keys {
                pks: vec![1, 2, u64::MAX],
            },
        ]
    }

    #[test]
    fn every_request_round_trips() {
        for (i, req) in requests().iter().enumerate() {
            let frame = encode_request(i as u64, req).unwrap();
            let header = Header::decode(&frame).unwrap();
            assert_eq!(header.request_id, i as u64);
            assert_eq!(header.opcode, req.opcode());
            assert!(header.opcode.is_request());

            let payload = &frame[HEADER_SIZE..];
            verify_payload(&header, payload).unwrap();
            assert_eq!(&Request::decode(header.opcode, payload).unwrap(), req);
        }
    }

    #[test]
    fn every_response_round_trips() {
        for (i, resp) in responses().iter().enumerate() {
            let frame = encode_response(i as u64, resp).unwrap();
            let header = Header::decode(&frame).unwrap();
            assert!(!header.opcode.is_request());
            let payload = &frame[HEADER_SIZE..];
            verify_payload(&header, payload).unwrap();
            assert_eq!(&Response::decode(header.opcode, payload).unwrap(), resp);
        }
    }

    /// The request id is echoed verbatim, including values a naive
    /// implementation might normalise.
    #[test]
    fn request_ids_survive_unchanged() {
        for id in [0u64, 1, u64::MAX, u64::MAX / 2, 1 << 63] {
            let frame = encode_request(id, &Request::Ping).unwrap();
            assert_eq!(Header::decode(&frame).unwrap().request_id, id);
        }
    }

    #[test]
    fn bad_magic_version_and_opcode_are_rejected() {
        let good = encode_request(1, &Request::Ping).unwrap();

        let mut bad = good.clone();
        bad[0] = b'X';
        assert!(matches!(Header::decode(&bad), Err(NetError::BadMagic)));

        let mut bad = good.clone();
        bad[2] = 99;
        assert!(matches!(
            Header::decode(&bad),
            Err(NetError::UnsupportedVersion(99))
        ));

        let mut bad = good.clone();
        bad[3] = 200;
        assert!(matches!(
            Header::decode(&bad),
            Err(NetError::UnknownOpcode(200))
        ));
    }

    /// **A declared payload length above the cap is rejected before allocating.**
    ///
    /// This is the denial-of-service guard. A peer that can make the server
    /// reserve an arbitrary buffer per connection needs no other attack.
    #[test]
    fn oversized_payload_length_is_rejected() {
        let mut frame = encode_request(1, &Request::Ping).unwrap();
        for len in [MAX_PAYLOAD + 1, u32::MAX, 1 << 30] {
            frame[16..20].copy_from_slice(&len.to_le_bytes());
            match Header::decode(&frame) {
                Err(NetError::PayloadTooLarge { len: l, .. }) => assert_eq!(l, len),
                other => panic!("length {len} was accepted: {other:?}"),
            }
        }
        // The boundary itself is legal.
        frame[16..20].copy_from_slice(&MAX_PAYLOAD.to_le_bytes());
        assert!(Header::decode(&frame).is_ok());
    }

    /// **A count inside the payload must be checked against the bytes present.**
    ///
    /// The frame cap does not help here: the lie fits inside an otherwise legal
    /// frame. A 4-byte count of `u32::MAX` in a 12-byte payload would reserve
    /// 32 GiB if taken at face value.
    #[test]
    fn absurd_inner_counts_are_rejected_not_allocated() {
        // Insert: pk(8) + count(4) with no values following.
        let mut payload = Vec::new();
        payload.extend_from_slice(&1u64.to_le_bytes());
        payload.extend_from_slice(&u32::MAX.to_le_bytes());
        assert!(matches!(
            Request::decode(OpCode::Insert, &payload),
            Err(NetError::Malformed(_))
        ));

        // Rows: a row count far beyond what the payload could hold.
        let mut payload = Vec::new();
        payload.extend_from_slice(&u32::MAX.to_le_bytes());
        assert!(matches!(
            Response::decode(OpCode::Rows, &payload),
            Err(NetError::Malformed(_))
        ));

        // A string length beyond the payload.
        let mut payload = Vec::new();
        payload.extend_from_slice(&1u16.to_le_bytes());
        payload.extend_from_slice(&u32::MAX.to_le_bytes());
        assert!(matches!(
            Response::decode(OpCode::Error, &payload),
            Err(NetError::Malformed(_))
        ));
    }

    #[test]
    fn corrupt_payload_fails_its_crc() {
        let frame = encode_request(
            1,
            &Request::Insert {
                pk: 5,
                values: vec![1, 2, 3],
            },
        )
        .unwrap();
        let header = Header::decode(&frame).unwrap();
        let mut payload = frame[HEADER_SIZE..].to_vec();
        payload[0] ^= 0xFF;
        assert!(matches!(
            verify_payload(&header, &payload),
            Err(NetError::CrcMismatch { .. })
        ));
    }

    #[test]
    fn a_short_payload_is_reported_as_truncated() {
        let frame = encode_request(1, &Request::Get { pk: 9 }).unwrap();
        let header = Header::decode(&frame).unwrap();
        let short = &frame[HEADER_SIZE..frame.len() - 1];
        assert!(matches!(
            verify_payload(&header, short),
            Err(NetError::Truncated { .. })
        ));
        assert!(matches!(
            Header::decode(&frame[..HEADER_SIZE - 1]),
            Err(NetError::Truncated { .. })
        ));
    }

    /// Trailing bytes are an error, not a remainder to ignore.
    ///
    /// They mean the sender and this decoder disagree about the message shape — a
    /// version skew or corruption — and silently accepting the prefix would let
    /// two builds appear to interoperate while dropping fields.
    #[test]
    fn trailing_bytes_are_rejected() {
        let mut payload = Request::Get { pk: 1 }.encode_payload().unwrap();
        payload.push(0);
        assert!(matches!(
            Request::decode(OpCode::Get, &payload),
            Err(NetError::Malformed(_))
        ));

        let mut payload = Response::Pong.encode_payload().unwrap();
        payload.push(0);
        assert!(matches!(
            Response::decode(OpCode::Pong, &payload),
            Err(NetError::Malformed(_))
        ));
    }

    /// A response opcode on a request decode is rejected, and vice versa.
    ///
    /// Without this a client could drive the server through its response path, or
    /// a hostile server could hand a client a request to interpret.
    #[test]
    fn opcodes_cannot_cross_direction() {
        assert!(matches!(
            Request::decode(OpCode::Pong, &[]),
            Err(NetError::NotARequest(_))
        ));
        assert!(matches!(
            Response::decode(OpCode::Ping, &[]),
            Err(NetError::NotAResponse(_))
        ));
        assert!(OpCode::Ping.is_request());
        assert!(!OpCode::Pong.is_request());
    }

    /// **Truncate every frame at every offset: never panic.**
    ///
    /// A socket delivers bytes in arbitrary chunks, and a peer can close mid-frame
    /// at any point. Every prefix must produce an error, not an index panic that
    /// takes down the connection handler.
    #[test]
    fn any_truncation_produces_an_error_not_a_panic() {
        for req in requests() {
            let frame = encode_request(7, &req).unwrap();
            for cut in 0..frame.len() {
                let part = &frame[..cut];
                match Header::decode(part) {
                    Err(_) => {}
                    Ok(h) => {
                        // Header parsed; the payload must then be short.
                        let payload = &part[HEADER_SIZE..];
                        assert!(
                            verify_payload(&h, payload).is_err(),
                            "cut {cut}: a truncated frame verified"
                        );
                    }
                }
            }
        }
    }

    /// Random bytes must never panic the decoder.
    ///
    /// Anything can connect to a TCP port. The decoder is the first thing an
    /// unauthenticated peer reaches, so it has to be total.
    #[test]
    fn arbitrary_bytes_never_panic() {
        let mut x: u64 = 0x243F_6A88_85A3_08D3;
        let mut next = || {
            x ^= x << 13;
            x ^= x >> 7;
            x ^= x << 17;
            x
        };
        for len in [0usize, 1, 7, 24, 25, 64, 200] {
            for _ in 0..200 {
                let buf: Vec<u8> = (0..len).map(|_| (next() & 0xFF) as u8).collect();
                let _ = Header::decode(&buf);
                for op in 1u8..=31 {
                    if let Ok(o) = OpCode::from_u8(op) {
                        let _ = Request::decode(o, &buf);
                    }
                }
                for op in 129u8..=144 {
                    if let Ok(o) = OpCode::from_u8(op) {
                        let _ = Response::decode(o, &buf);
                    }
                }
            }
        }
    }

    /// A payload at the cap encodes and decodes.
    #[test]
    fn a_maximum_size_frame_is_legal() {
        // 16 MiB of u64s, minus the pk and count overhead.
        let n = (MAX_PAYLOAD as usize - 12) / 8;
        let req = Request::Insert {
            pk: 1,
            values: vec![0xAB; n],
        };
        let frame = encode_request(1, &req).unwrap();
        let header = Header::decode(&frame).unwrap();
        let payload = &frame[HEADER_SIZE..];
        verify_payload(&header, payload).unwrap();
        assert_eq!(Request::decode(header.opcode, payload).unwrap(), req);
    }

    /// Encoding something past the cap fails rather than emitting a frame no peer
    /// will accept.
    #[test]
    fn encoding_past_the_cap_is_refused() {
        let n = MAX_PAYLOAD as usize / 8 + 16;
        let req = Request::Insert {
            pk: 1,
            values: vec![0; n],
        };
        assert!(matches!(
            encode_request(1, &req),
            Err(NetError::PayloadTooLarge { .. })
        ));
    }
}
