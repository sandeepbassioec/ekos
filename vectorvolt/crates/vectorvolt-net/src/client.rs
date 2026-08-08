//! [`Client`] — a blocking client for the VectorVolt wire protocol (VV-NET-006).

use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream, ToSocketAddrs};
use std::time::Duration;

use crate::protocol::{HEADER_SIZE, Header, Request, Response, encode_request, verify_payload};
use crate::tls::{ClientConn, ClientTlsConfig};
use crate::{NetError, NetResult};

/// A connection to a VectorVolt server.
///
/// Blocking and single-threaded by design: one request, one response. The
/// protocol supports pipelining via `request_id`, but a client that pipelines has
/// to manage outstanding requests, and that belongs in the SDK rather than here.
/// This is the smallest thing that can talk to the server correctly.
pub struct Client {
    stream: ClientConn,
    next_id: u64,
}

impl Client {
    pub fn connect(addr: impl ToSocketAddrs) -> NetResult<Self> {
        Self::connect_with_tls(addr, None)
    }

    /// Connect over TLS.
    ///
    /// The certificate is verified during `connect`, so a server this client does
    /// not trust fails HERE — before any credential is sent. Deferring
    /// verification to first use would open a window in which a caller has already
    /// handed its secret to an unverified peer.
    pub fn connect_tls(addr: impl ToSocketAddrs, tls: &ClientTlsConfig) -> NetResult<Self> {
        Self::connect_with_tls(addr, Some(tls))
    }

    fn connect_with_tls(
        addr: impl ToSocketAddrs,
        tls: Option<&ClientTlsConfig>,
    ) -> NetResult<Self> {
        let tcp = TcpStream::connect(addr)?;
        // Nagle off for the same reason the server disables it: a request/response
        // protocol has nothing to coalesce, and the delay lands on every call.
        let _ = tcp.set_nodelay(true);
        Ok(Client {
            stream: ClientConn::connect(tcp, tls)?,
            next_id: 1,
        })
    }

    /// True if this connection is encrypted.
    pub fn is_encrypted(&self) -> bool {
        self.stream.is_tls()
    }

    /// Connect with a timeout, so an unreachable host fails fast instead of
    /// waiting out the OS default (which can be over a minute).
    pub fn connect_timeout(addr: SocketAddr, timeout: Duration) -> NetResult<Self> {
        let tcp = TcpStream::connect_timeout(&addr, timeout)?;
        let _ = tcp.set_nodelay(true);
        Ok(Client {
            stream: ClientConn::connect(tcp, None)?,
            next_id: 1,
        })
    }

    /// Bound how long a single call may block.
    pub fn set_timeout(&self, timeout: Option<Duration>) -> NetResult<()> {
        self.stream.socket().set_read_timeout(timeout)?;
        self.stream.socket().set_write_timeout(timeout)?;
        Ok(())
    }

    pub fn local_addr(&self) -> NetResult<SocketAddr> {
        Ok(self.stream.socket().local_addr()?)
    }

    /// Send a request and wait for its response.
    ///
    /// The returned frame's `request_id` is checked against the one sent. On a
    /// strictly sequential connection a mismatch should be impossible — which is
    /// exactly why it is worth checking: if it ever happens, the stream has
    /// desynchronised and every subsequent response would be attributed to the
    /// wrong call. Failing loudly beats returning another request's answer.
    pub fn call(&mut self, req: &Request) -> NetResult<Response> {
        let id = self.next_id;
        // Wrapping rather than overflowing. Reaching 2^64 requests on one
        // connection is not going to happen, but a panic in a client library is a
        // worse answer than reusing an id that has long since been answered.
        self.next_id = self.next_id.wrapping_add(1).max(1);

        let frame = encode_request(id, req)?;
        self.stream.write_all(&frame)?;
        self.stream.flush()?;

        let mut header_buf = [0u8; HEADER_SIZE];
        self.stream.read_exact(&mut header_buf)?;
        let header = Header::decode(&header_buf)?;

        let mut payload = vec![0u8; header.payload_len as usize];
        if header.payload_len > 0 {
            self.stream.read_exact(&mut payload)?;
        }
        verify_payload(&header, &payload)?;

        if header.request_id != id {
            return Err(NetError::Malformed(
                "response request_id does not match the request; the stream has desynchronised",
            ));
        }
        Response::decode(header.opcode, &payload)
    }

    // ── Convenience wrappers ──────────────────────────────────────────────────

    /// Authenticate with `key_id` and `secret`.
    ///
    /// Two round trips: ask for a challenge, answer it. The secret never leaves
    /// this process — only `SHA3-256(secret ‖ nonce)` goes on the wire, and the
    /// nonce differs per connection, so a captured exchange is not replayable.
    ///
    /// **This does not encrypt the session.** Everything after the handshake is
    /// plaintext. See the `auth` module: authentication answers "who is this", not
    /// "is anyone listening".
    pub fn authenticate(&mut self, key_id: &str, secret: &[u8]) -> NetResult<()> {
        let nonce = match self.call(&Request::Hello)? {
            Response::Challenge { nonce } => nonce,
            other => return Err(unexpected(other)),
        };
        if nonce.len() != crate::auth::NONCE_LEN {
            return Err(NetError::Malformed("challenge nonce has the wrong length"));
        }
        let response = crate::auth::client_response(secret, &nonce);
        match self.call(&Request::Auth {
            key_id: key_id.to_string(),
            response: response.to_vec(),
        })? {
            Response::Ok { .. } => Ok(()),
            other => Err(unexpected(other)),
        }
    }

    /// Connect and authenticate in one step.
    pub fn connect_authenticated(
        addr: impl ToSocketAddrs,
        key_id: &str,
        secret: &[u8],
    ) -> NetResult<Self> {
        let mut c = Self::connect(addr)?;
        c.authenticate(key_id, secret)?;
        Ok(c)
    }

    /// Connect over TLS and authenticate. The production path.
    pub fn connect_tls_authenticated(
        addr: impl ToSocketAddrs,
        tls: &ClientTlsConfig,
        key_id: &str,
        secret: &[u8],
    ) -> NetResult<Self> {
        let mut c = Self::connect_tls(addr, tls)?;
        c.authenticate(key_id, secret)?;
        Ok(c)
    }

    pub fn ping(&mut self) -> NetResult<()> {
        match self.call(&Request::Ping)? {
            Response::Pong => Ok(()),
            other => Err(unexpected(other)),
        }
    }

    /// `Ok(None)` when the key does not exist — a normal outcome, not an error.
    pub fn get(&mut self, pk: u64) -> NetResult<Option<Vec<u64>>> {
        match self.call(&Request::Get { pk })? {
            Response::Row { values } => Ok(Some(values)),
            Response::NotFound => Ok(None),
            other => Err(unexpected(other)),
        }
    }

    pub fn insert(&mut self, pk: u64, values: Vec<u64>) -> NetResult<()> {
        match self.call(&Request::Insert { pk, values })? {
            Response::Ok { .. } => Ok(()),
            other => Err(unexpected(other)),
        }
    }

    /// `Ok(false)` when there was nothing to delete.
    pub fn delete(&mut self, pk: u64) -> NetResult<bool> {
        match self.call(&Request::Delete { pk })? {
            Response::Ok { affected } => Ok(affected > 0),
            other => Err(unexpected(other)),
        }
    }

    /// `limit` of 0 means unlimited.
    pub fn range_scan(
        &mut self,
        col: u32,
        lo: u64,
        hi: u64,
        limit: u32,
    ) -> NetResult<Vec<Vec<u64>>> {
        match self.call(&Request::RangeScan { col, lo, hi, limit })? {
            Response::Rows { rows } => Ok(rows),
            other => Err(unexpected(other)),
        }
    }

    pub fn flush(&mut self) -> NetResult<()> {
        match self.call(&Request::Flush)? {
            Response::Ok { .. } => Ok(()),
            other => Err(unexpected(other)),
        }
    }

    /// Rows kept by the compaction, or 0 if there was nothing to do.
    pub fn compact(&mut self) -> NetResult<u64> {
        match self.call(&Request::Compact)? {
            Response::Ok { affected } => Ok(affected),
            other => Err(unexpected(other)),
        }
    }

    /// `(rows, partitions, bytes)`.
    pub fn stats(&mut self) -> NetResult<(u64, u64, u64)> {
        match self.call(&Request::Stats)? {
            Response::Stats {
                rows,
                partitions,
                bytes,
            } => Ok((rows, partitions, bytes)),
            other => Err(unexpected(other)),
        }
    }

    /// All rows where column `col_idx` equals `value` (secondary-index lookup).
    ///
    /// Returns `Vec<Vec<u64>>` where each inner vec is a full row (all columns).
    /// Deleted rows are silently excluded server-side.
    pub fn secondary_scan(&mut self, col_idx: u32, value: u64) -> NetResult<Vec<Vec<u64>>> {
        match self.call(&Request::SecondaryLookup { col_idx, value })? {
            Response::Rows { rows } => Ok(rows),
            other => Err(unexpected(other)),
        }
    }

    /// Probabilistic FK existence check. `true` if `value` was ever inserted as
    /// a PK. False positive rate ≈ 0.78%; false negatives are impossible.
    pub fn fk_exists(&mut self, value: u64) -> NetResult<bool> {
        match self.call(&Request::FkExists { value })? {
            Response::Exists { found } => Ok(found),
            other => Err(unexpected(other)),
        }
    }

    /// Select the active table for this connection.
    pub fn use_table(&mut self, name: impl Into<String>) -> NetResult<()> {
        match self.call(&Request::UseTable { name: name.into() })? {
            Response::TableSelected => Ok(()),
            other => Err(unexpected(other)),
        }
    }

    /// Create a new table and select it as the active table.
    pub fn create_table(&mut self, name: impl Into<String>, schema: Vec<String>) -> NetResult<()> {
        match self.call(&Request::CreateTable {
            name: name.into(),
            schema,
        })? {
            Response::TableSelected => Ok(()),
            other => Err(unexpected(other)),
        }
    }

    /// Drop a named table. The active table is cleared after this call
    /// regardless of which table was selected.
    pub fn drop_table(&mut self, name: impl Into<String>) -> NetResult<()> {
        match self.call(&Request::DropTable { name: name.into() })? {
            Response::Ok { .. } => Ok(()),
            other => Err(unexpected(other)),
        }
    }

    /// List all table names known to the server.
    pub fn list_tables(&mut self) -> NetResult<Vec<String>> {
        match self.call(&Request::ListTables)? {
            Response::TableList { names } => Ok(names),
            other => Err(unexpected(other)),
        }
    }

    /// Insert multiple rows in one round-trip.
    ///
    /// Returns `(succeeded, skipped)` where `skipped` counts rows the server
    /// could not insert (e.g. an engine error on that row).
    pub fn batch_insert(&mut self, rows: Vec<(u64, Vec<u64>)>) -> NetResult<(u32, u32)> {
        match self.call(&Request::BatchInsert { rows })? {
            Response::BatchResult { succeeded, skipped } => Ok((succeeded, skipped)),
            other => Err(unexpected(other)),
        }
    }

    /// Delete multiple rows by primary key in one round-trip.
    ///
    /// Returns `(succeeded, skipped)` where `skipped` counts keys not found.
    pub fn batch_delete(&mut self, pks: Vec<u64>) -> NetResult<(u32, u32)> {
        match self.call(&Request::BatchDelete { pks })? {
            Response::BatchResult { succeeded, skipped } => Ok((succeeded, skipped)),
            other => Err(unexpected(other)),
        }
    }

    /// Begin a server-side transaction on this connection.
    ///
    /// Returns the `txn_id` that must be passed to [`commit_txn`](Self::commit_txn)
    /// or [`rollback_txn`](Self::rollback_txn).
    pub fn begin_txn(&mut self) -> NetResult<u64> {
        match self.call(&Request::BeginTxn)? {
            Response::TxnStarted { txn_id } => Ok(txn_id),
            other => Err(unexpected(other)),
        }
    }

    /// Commit the active transaction.
    ///
    /// Returns the number of operations applied.
    pub fn commit_txn(&mut self, txn_id: u64) -> NetResult<u64> {
        match self.call(&Request::CommitTxn { txn_id })? {
            Response::Ok { affected } => Ok(affected),
            other => Err(unexpected(other)),
        }
    }

    /// Roll back the active transaction, discarding all buffered ops.
    pub fn rollback_txn(&mut self, txn_id: u64) -> NetResult<()> {
        match self.call(&Request::RollbackTxn { txn_id })? {
            Response::Ok { .. } => Ok(()),
            other => Err(unexpected(other)),
        }
    }

    /// Insert-or-replace: if a row with `pk` exists it is deleted first, then
    /// the new row is inserted. Never returns a duplicate-key error.
    pub fn upsert(&mut self, pk: u64, values: Vec<u64>) -> NetResult<()> {
        match self.call(&Request::Upsert { pk, values })? {
            Response::Ok { .. } => Ok(()),
            other => Err(unexpected(other)),
        }
    }

    /// Fetch multiple rows by primary key in one round trip.
    ///
    /// Returns only found rows as `(pk, values)` pairs. Missing keys are
    /// silently omitted — the caller can diff against the input list.
    pub fn get_many(&mut self, pks: Vec<u64>) -> NetResult<Vec<(u64, Vec<u64>)>> {
        match self.call(&Request::GetMany { pks })? {
            Response::RowSet { entries } => Ok(entries),
            other => Err(unexpected(other)),
        }
    }

    /// Update an existing row. Returns an error if `pk` does not exist.
    pub fn update(&mut self, pk: u64, values: Vec<u64>) -> NetResult<()> {
        match self.call(&Request::Update { pk, values })? {
            Response::Ok { .. } => Ok(()),
            other => Err(unexpected(other)),
        }
    }

    /// Delete all rows from the selected table.
    ///
    /// Returns the number of rows that were removed.
    pub fn truncate(&mut self) -> NetResult<u64> {
        match self.call(&Request::Truncate)? {
            Response::Ok { affected } => Ok(affected),
            other => Err(unexpected(other)),
        }
    }

    /// Count rows where column `col` value falls in `[lo, hi]` (both inclusive).
    ///
    /// `col = 0` counts by primary key range; higher indices count by non-pk columns.
    pub fn count(&mut self, col: u32, lo: u64, hi: u64) -> NetResult<u64> {
        match self.call(&Request::Count { col, lo, hi })? {
            Response::Ok { affected } => Ok(affected),
            other => Err(unexpected(other)),
        }
    }

    /// Insert-or-replace multiple rows in one round trip.
    ///
    /// Returns the number of rows submitted (all are applied — there is no
    /// per-row skip count because upsert never fails on a duplicate key).
    pub fn batch_upsert(&mut self, rows: Vec<(u64, Vec<u64>)>) -> NetResult<u64> {
        match self.call(&Request::BatchUpsert { rows })? {
            Response::Ok { affected } => Ok(affected),
            other => Err(unexpected(other)),
        }
    }

    /// Update multiple existing rows in one round trip.
    ///
    /// Returns `(succeeded, skipped)` where `skipped` counts keys not found.
    pub fn batch_update(&mut self, rows: Vec<(u64, Vec<u64>)>) -> NetResult<(u32, u32)> {
        match self.call(&Request::BatchUpdate { rows })? {
            Response::BatchResult { succeeded, skipped } => Ok((succeeded, skipped)),
            other => Err(unexpected(other)),
        }
    }

    /// Check whether a named table exists in the server catalog.
    ///
    /// Does not require a table to be selected first.
    pub fn table_exists(&mut self, name: impl Into<String>) -> NetResult<bool> {
        match self.call(&Request::TableExists { name: name.into() })? {
            Response::Exists { found } => Ok(found),
            other => Err(unexpected(other)),
        }
    }

    /// Return the column names for the currently selected table.
    pub fn describe_table(&mut self) -> NetResult<Vec<String>> {
        match self.call(&Request::DescribeTable)? {
            Response::Schema { columns } => Ok(columns),
            other => Err(unexpected(other)),
        }
    }
}

/// Turn an unexpected response into an error.
///
/// A server `Error` response keeps its message, because that is the whole reason
/// the server sent one. Anything else means the two sides disagree about the
/// protocol, which is worth saying differently.
fn unexpected(r: Response) -> NetError {
    match r {
        Response::Error { code, message } => NetError::Engine(format!("[{code}] {message}")),
        other => NetError::Engine(format!("unexpected response: {other:?}")),
    }
}
