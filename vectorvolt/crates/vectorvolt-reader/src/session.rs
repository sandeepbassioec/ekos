//! PostgreSQL wire protocol session handler.

use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use std::time::Duration;

use bytes::BytesMut;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt, ReadBuf};
use tokio::net::TcpStream;
use tokio_rustls::TlsAcceptor;
use tokio_rustls::server::TlsStream;
use vectorvolt_engine::{EngineOptions, TableRegistry};

use std::collections::HashMap;

use subtle::ConstantTimeEq;

use crate::auth::{
    Credentials, SCRAM_SHA_256, ScramServer, ScramVerifier, md5_verify, random_bytes,
};
use crate::extended::{
    self, Portal, PortalState, Prepared, ProtoError, check_result_formats, is_row_returning,
    unwrap_notices,
};
use crate::wire::{
    FrontendMsg, MAX_MESSAGE_LEN, WireError, decode_frontend, decode_sasl_initial_response,
    decode_sasl_response, decode_startup, encode_auth_cleartext_password, encode_auth_md5_password,
    encode_auth_ok, encode_auth_sasl, encode_auth_sasl_continue, encode_auth_sasl_final,
    encode_bind_complete, encode_close_complete, encode_command_complete, encode_data_row,
    encode_empty_query_response, encode_error_response, encode_error_with_code, encode_fatal_error,
    encode_no_data, encode_notice_response, encode_param_status, encode_parameter_description,
    encode_parse_complete, encode_portal_suspended, encode_ready_for_query, encode_row_description,
    encode_row_description_with_formats, split_frontend_frame,
};
use vectorvolt_query::sql::executor::{QueryResult, SqlExecutor};

/// State of a session within the query-execution loop.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SessionState {
    /// Waiting for the client's startup message.
    Startup,
    /// Ready to receive queries.
    Idle,
    /// Processing a query.
    Busy,
    /// Session is done.
    Done,
}

/// Largest startup / password message accepted before authentication.
const MAX_STARTUP_LEN: usize = 10_000;

/// Protocol codes carried in the 8-byte pre-startup negotiation packets.
const SSL_REQUEST_CODE: i32 = 80_877_103;
const GSSENC_REQUEST_CODE: i32 = 80_877_104;

/// How long a client gets to complete the TLS handshake after we answer 'S'.
const TLS_HANDSHAKE_TIMEOUT: Duration = Duration::from_secs(10);

/// The client connection: plaintext, or upgraded to TLS after an SSLRequest.
pub enum PgStream {
    Plain(TcpStream),
    Tls(Box<TlsStream<TcpStream>>),
    /// Transient state while the plaintext socket is handed to the TLS
    /// acceptor; reads see EOF and writes fail.
    Closed,
}

impl PgStream {
    /// True once the connection has been upgraded to TLS.
    pub fn is_tls(&self) -> bool {
        matches!(self, PgStream::Tls(_))
    }
}

impl AsyncRead for PgStream {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        match self.get_mut() {
            PgStream::Plain(s) => Pin::new(s).poll_read(cx, buf),
            PgStream::Tls(s) => Pin::new(s.as_mut()).poll_read(cx, buf),
            PgStream::Closed => Poll::Ready(Ok(())),
        }
    }
}

impl AsyncWrite for PgStream {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<std::io::Result<usize>> {
        match self.get_mut() {
            PgStream::Plain(s) => Pin::new(s).poll_write(cx, buf),
            PgStream::Tls(s) => Pin::new(s.as_mut()).poll_write(cx, buf),
            PgStream::Closed => Poll::Ready(Err(std::io::ErrorKind::NotConnected.into())),
        }
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        match self.get_mut() {
            PgStream::Plain(s) => Pin::new(s).poll_flush(cx),
            PgStream::Tls(s) => Pin::new(s.as_mut()).poll_flush(cx),
            PgStream::Closed => Poll::Ready(Ok(())),
        }
    }

    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        match self.get_mut() {
            PgStream::Plain(s) => Pin::new(s).poll_shutdown(cx),
            PgStream::Tls(s) => Pin::new(s.as_mut()).poll_shutdown(cx),
            PgStream::Closed => Poll::Ready(Ok(())),
        }
    }
}

/// A single client session over a `TcpStream` (optionally upgraded to TLS).
pub struct PgSession {
    stream: PgStream,
    executor: SqlExecutor,
    /// How clients authenticate (SCRAM / MD5 / cleartext / none).
    credentials: Credentials,
    state: SessionState,
    /// Answers SSLRequest with 'S' and upgrades when set; 'N' otherwise.
    tls: Option<TlsAcceptor>,
    /// Refuse sessions that did not upgrade to TLS.
    require_tls: bool,
    /// Extended protocol: prepared statements by name ("" = unnamed).
    statements: HashMap<String, Prepared>,
    /// Extended protocol: portals by name ("" = unnamed).
    portals: HashMap<String, Portal>,
    /// After an extended-protocol error, messages are discarded until Sync.
    skip_until_sync: bool,
    /// Pending backend messages, written on Sync / Flush / simple Query.
    out: BytesMut,
}

/// Most statements / portals one session may hold (memory bound).
const MAX_NAMED_OBJECTS: usize = 10_000;

impl PgSession {
    /// Create a session. `executor` should be a per-connection session of the
    /// server's shared executor ([`SqlExecutor::new_session`]) so security
    /// state is server-wide while transactions stay per connection.
    pub fn new(stream: TcpStream, executor: SqlExecutor, credentials: Credentials) -> Self {
        Self {
            stream: PgStream::Plain(stream),
            executor,
            credentials,
            state: SessionState::Startup,
            tls: None,
            require_tls: false,
            statements: HashMap::new(),
            portals: HashMap::new(),
            skip_until_sync: false,
            out: BytesMut::new(),
        }
    }

    /// Offer TLS: an SSLRequest is answered with 'S' and the connection is
    /// upgraded with `acceptor` (`None` keeps answering 'N'). With `require`,
    /// a client that starts up without TLS is refused (SQLSTATE 28000).
    pub fn with_tls(mut self, acceptor: Option<TlsAcceptor>, require: bool) -> Self {
        self.tls = acceptor;
        self.require_tls = require;
        self
    }

    /// True once the connection has been upgraded to TLS.
    pub fn is_tls(&self) -> bool {
        self.stream.is_tls()
    }

    /// Answer 'S' and run the TLS handshake. `Ok(false)` = the handshake
    /// failed or timed out (the connection is unusable and is dropped).
    async fn upgrade_tls(&mut self, acceptor: TlsAcceptor) -> std::io::Result<bool> {
        self.stream.write_all(b"S").await?;
        self.stream.flush().await?;
        let PgStream::Plain(plain) = std::mem::replace(&mut self.stream, PgStream::Closed) else {
            return Ok(false);
        };
        match tokio::time::timeout(TLS_HANDSHAKE_TIMEOUT, acceptor.accept(plain)).await {
            Ok(Ok(tls)) => {
                self.stream = PgStream::Tls(Box::new(tls));
                Ok(true)
            }
            Ok(Err(e)) => {
                tracing::debug!("pgwire TLS handshake failed: {e}");
                Ok(false)
            }
            Err(_) => {
                tracing::debug!("pgwire TLS handshake timed out");
                Ok(false)
            }
        }
    }

    /// Send a FATAL error and end the session (the caller drops the stream).
    /// Write errors are ignored: the peer may already have gone away.
    async fn fail(&mut self, code: &str, msg: &str) -> std::io::Result<()> {
        self.state = SessionState::Done;
        let _ = self.stream.write_all(&encode_fatal_error(code, msg)).await;
        let _ = self.stream.flush().await;
        Ok(())
    }

    /// Read one length-prefixed pre-auth message (`with_type` = has a type
    /// byte). Returns `None` on EOF or an out-of-range length.
    async fn read_preauth(&mut self, with_type: bool) -> std::io::Result<Option<Vec<u8>>> {
        let hdr = if with_type { 5 } else { 4 };
        let mut buf = vec![0u8; hdr];
        if let Err(e) = self.stream.read_exact(&mut buf).await {
            return if e.kind() == std::io::ErrorKind::UnexpectedEof {
                Ok(None)
            } else {
                Err(e)
            };
        }
        let len = i32::from_be_bytes([buf[hdr - 4], buf[hdr - 3], buf[hdr - 2], buf[hdr - 1]]);
        let len = match usize::try_from(len) {
            Ok(l) if (4..=MAX_STARTUP_LEN).contains(&l) => l,
            _ => return Ok(None),
        };
        buf.resize(hdr + len - 4, 0);
        if let Err(e) = self.stream.read_exact(&mut buf[hdr..]).await {
            return if e.kind() == std::io::ErrorKind::UnexpectedEof {
                Ok(None)
            } else {
                Err(e)
            };
        }
        Ok(Some(buf))
    }

    /// Drive the session to completion.
    pub async fn run(&mut self) -> std::io::Result<()> {
        // ── startup ────────────────────────────────────────────────────────────
        // Decode startup message (no type byte). Anything unparseable, or a
        // protocol other than 3.x, is rejected and the connection closed.
        let Some(mut buf) = self.read_preauth(false).await? else {
            return self.fail("08P01", "invalid startup packet").await;
        };
        if buf.len() < 8 {
            return self.fail("08P01", "invalid startup packet").await;
        }
        // SSLRequest / GSSENCRequest negotiation. GSSAPI encryption is never
        // offered ('N'). SSL is offered ('S' + TLS handshake) when the server
        // has a certificate, otherwise refused ('N'); either way the client
        // then sends the real startup message (over TLS after an upgrade).
        // libpq may send a GSSENCRequest and then an SSLRequest, so allow a
        // couple of rounds, but at most one TLS upgrade. Nothing is read ahead
        // of the 8-byte request, so no plaintext can be smuggled into the TLS
        // session (CVE-2021-23214).
        for _ in 0..2 {
            if buf.len() != 8 {
                break;
            }
            let code = i32::from_be_bytes([buf[4], buf[5], buf[6], buf[7]]);
            if code == SSL_REQUEST_CODE && !self.stream.is_tls() {
                match self.tls.clone() {
                    Some(acceptor) => {
                        if !self.upgrade_tls(acceptor).await? {
                            self.state = SessionState::Done;
                            return Ok(());
                        }
                    }
                    None => {
                        self.stream.write_all(b"N").await?;
                        self.stream.flush().await?;
                    }
                }
            } else if code == GSSENC_REQUEST_CODE {
                self.stream.write_all(b"N").await?;
                self.stream.flush().await?;
            } else {
                break;
            }
            let Some(next) = self.read_preauth(false).await? else {
                return self.fail("08P01", "invalid startup packet").await;
            };
            buf = next;
        }
        if self.require_tls && !self.stream.is_tls() {
            return self
                .fail(
                    "28000",
                    "TLS is required: this server refuses unencrypted connections \
                     (connect with sslmode=require)",
                )
                .await;
        }
        let user = match decode_startup(&buf) {
            Ok(FrontendMsg::Startup {
                major: 3, params, ..
            }) => params
                .into_iter()
                .find(|(k, _)| k == "user")
                .map(|(_, v)| v)
                .unwrap_or_default(),
            _ => {
                return self
                    .fail("08P01", "invalid startup packet or unsupported protocol")
                    .await;
            }
        };

        // ── authentication ───────────────────────────────────────────────────
        if !matches!(self.credentials, Credentials::None) {
            if user.is_empty() {
                return self
                    .fail(
                        "28000",
                        "no PostgreSQL user name specified in startup packet",
                    )
                    .await;
            }
            if !self.authenticate(&user).await? {
                return Ok(());
            }
            // D5: per-user grants apply to the authenticated user.
            self.executor.set_authenticated_role(&user);
        }

        // Send auth ok + params + ready
        self.stream.write_all(&encode_auth_ok()).await?;
        self.stream
            .write_all(&encode_param_status("server_version", "16.0"))
            .await?;
        self.stream
            .write_all(&encode_param_status("client_encoding", "UTF8"))
            .await?;
        self.stream.write_all(&encode_ready_for_query()).await?;
        self.stream.flush().await?;
        self.state = SessionState::Idle;

        // ── query loop ─────────────────────────────────────────────────────────
        // Bytes are accumulated in `rbuf` and complete messages are split off
        // its front, so one read may carry several (pipelined) messages and
        // one message may span several reads. Every message is processed in
        // arrival order. A length outside `4..=MAX_MESSAGE_LEN` is rejected
        // as soon as its header arrives, before the body is buffered.
        let mut rbuf = BytesMut::with_capacity(8192);
        loop {
            let frame = match split_frontend_frame(&mut rbuf, MAX_MESSAGE_LEN) {
                Ok(Some(frame)) => frame,
                Ok(None) => {
                    if self.stream.read_buf(&mut rbuf).await? == 0 {
                        break; // EOF (a trailing partial message is dropped)
                    }
                    continue;
                }
                Err(WireError::InvalidLength(len)) => {
                    let msg =
                        format!("invalid message length {len} (must be 4..={MAX_MESSAGE_LEN})");
                    return self.fail("08P01", &msg).await;
                }
                Err(e) => return self.fail("08P01", &e.to_string()).await,
            };
            let msg = match decode_frontend(&frame) {
                Ok(m) => m,
                Err(e) => return self.fail("08P01", &e.to_string()).await,
            };

            // Extended protocol: after an error everything up to the next
            // Sync is discarded (a simple Query is still processed, as in
            // PostgreSQL).
            if self.skip_until_sync && msg.is_extended() {
                continue;
            }

            match msg {
                FrontendMsg::Terminate => {
                    self.state = SessionState::Done;
                    break;
                }
                FrontendMsg::Query(sql) => {
                    self.state = SessionState::Busy;
                    // A simple Query destroys the unnamed statement / portal.
                    self.statements.remove("");
                    self.portals.remove("");
                    self.flush_out().await?;
                    self.handle_query(&sql).await?;
                    self.stream.write_all(&encode_ready_for_query()).await?;
                    self.stream.flush().await?;
                    self.state = SessionState::Idle;
                }
                FrontendMsg::Sync => {
                    self.skip_until_sync = false;
                    self.out.extend_from_slice(&encode_ready_for_query());
                    self.flush_out().await?;
                    self.state = SessionState::Idle;
                }
                FrontendMsg::Flush => self.flush_out().await?,
                FrontendMsg::Parse { .. }
                | FrontendMsg::Bind { .. }
                | FrontendMsg::Describe { .. }
                | FrontendMsg::Execute { .. }
                | FrontendMsg::Close { .. } => {
                    self.state = SessionState::Busy;
                    if let Err(e) = self.handle_extended(msg) {
                        self.out
                            .extend_from_slice(&encode_error_with_code(e.code, &e.message));
                        self.skip_until_sync = true;
                    }
                    // Keep the pending output bounded for long pipelines.
                    if self.out.len() >= 64 * 1024 {
                        self.flush_out().await?;
                    }
                }
                FrontendMsg::Startup { .. } | FrontendMsg::Password(_) => {
                    // Unexpected second startup / stray password message
                    return self.fail("08P01", "unexpected message").await;
                }
            }
        }
        Ok(())
    }

    /// Write and flush everything queued in `out`.
    async fn flush_out(&mut self) -> std::io::Result<()> {
        if !self.out.is_empty() {
            let out = self.out.split();
            self.stream.write_all(&out).await?;
        }
        self.stream.flush().await
    }

    // ── authentication ──────────────────────────────────────────────────────

    /// Run the configured authentication exchange for `user`. `Ok(false)` =
    /// refused (a FATAL error has been sent and the session is done).
    async fn authenticate(&mut self, user: &str) -> std::io::Result<bool> {
        let failed = format!("password authentication failed for user \"{user}\"");
        match self.credentials.clone() {
            Credentials::None => Ok(true),
            Credentials::Cleartext(expected) => {
                self.stream
                    .write_all(&encode_auth_cleartext_password())
                    .await?;
                self.stream.flush().await?;
                let supplied = self.read_password().await?;
                let ok =
                    supplied.is_some_and(|pw| bool::from(pw.as_bytes().ct_eq(expected.as_bytes())));
                if !ok {
                    self.fail("28P01", &failed).await?;
                }
                Ok(ok)
            }
            Credentials::Md5(password) => {
                let salt: [u8; 4] = random_bytes();
                self.stream
                    .write_all(&encode_auth_md5_password(&salt))
                    .await?;
                self.stream.flush().await?;
                let supplied = self.read_password().await?;
                let ok = supplied.is_some_and(|pw| md5_verify(&password, user, &salt, &pw));
                if !ok {
                    self.fail("28P01", &failed).await?;
                }
                Ok(ok)
            }
            Credentials::Scram(verifier) => self.authenticate_scram(&verifier, &failed).await,
        }
    }

    /// Read a PasswordMessage; `None` on EOF or any other message.
    async fn read_password(&mut self) -> std::io::Result<Option<String>> {
        Ok(match self.read_preauth(true).await? {
            Some(msg) => match decode_frontend(&msg) {
                Ok(FrontendMsg::Password(pw)) => Some(pw),
                _ => None,
            },
            None => None,
        })
    }

    /// SCRAM-SHA-256: AuthenticationSASL → SASLInitialResponse →
    /// AuthenticationSASLContinue → SASLResponse → AuthenticationSASLFinal.
    /// Channel binding (SCRAM-SHA-256-PLUS) is not offered.
    async fn authenticate_scram(
        &mut self,
        verifier: &ScramVerifier,
        failed: &str,
    ) -> std::io::Result<bool> {
        self.stream
            .write_all(&encode_auth_sasl(&[SCRAM_SHA_256]))
            .await?;
        self.stream.flush().await?;

        let Some(msg) = self.read_preauth(true).await? else {
            self.state = SessionState::Done;
            return Ok(false);
        };
        let initial = match decode_sasl_initial_response(&msg) {
            Ok(i) => i,
            Err(_) => {
                self.fail("08P01", "expected SASL response").await?;
                return Ok(false);
            }
        };
        if initial.mechanism != SCRAM_SHA_256 {
            self.fail(
                "28000",
                "client selected an invalid SASL authentication mechanism",
            )
            .await?;
            return Ok(false);
        }
        let Some(client_first) = initial.data else {
            self.fail("08P01", "SASL initial response is missing its data")
                .await?;
            return Ok(false);
        };
        let (scram, server_first) = match ScramServer::start_random(verifier, &client_first) {
            Ok(x) => x,
            Err(e) => {
                self.fail(e.sqlstate(), &e.to_string()).await?;
                return Ok(false);
            }
        };
        self.stream
            .write_all(&encode_auth_sasl_continue(server_first.as_bytes()))
            .await?;
        self.stream.flush().await?;

        let Some(msg) = self.read_preauth(true).await? else {
            self.state = SessionState::Done;
            return Ok(false);
        };
        let client_final = match decode_sasl_response(&msg) {
            Ok(d) => d,
            Err(_) => {
                self.fail("08P01", "expected SASL response").await?;
                return Ok(false);
            }
        };
        match scram.finish(verifier, &client_final) {
            Ok(server_final) => {
                self.stream
                    .write_all(&encode_auth_sasl_final(server_final.as_bytes()))
                    .await?;
                Ok(true)
            }
            Err(crate::auth::ScramError::BadProof) => {
                self.fail("28P01", failed).await?;
                Ok(false)
            }
            Err(e) => {
                self.fail(e.sqlstate(), &e.to_string()).await?;
                Ok(false)
            }
        }
    }

    // ── extended query protocol ─────────────────────────────────────────────

    /// Handle one Parse / Bind / Describe / Execute / Close, queueing the
    /// responses in `out`. An `Err` is sent as ErrorResponse and starts the
    /// skip-until-Sync state.
    fn handle_extended(&mut self, msg: FrontendMsg) -> Result<(), ProtoError> {
        match msg {
            FrontendMsg::Parse {
                name,
                query,
                param_types,
            } => {
                if !name.is_empty() && self.statements.contains_key(&name) {
                    return Err(ProtoError::new(
                        "42P05",
                        format!("prepared statement \"{name}\" already exists"),
                    ));
                }
                if self.statements.len() >= MAX_NAMED_OBJECTS {
                    return Err(ProtoError::new("53400", "too many prepared statements"));
                }
                let prepared = Prepared::new(query, &param_types)?;
                self.statements.insert(name, prepared);
                self.out.extend_from_slice(&encode_parse_complete());
            }
            FrontendMsg::Bind {
                portal,
                statement,
                param_formats,
                params,
                result_formats,
            } => {
                let stmt = self.statements.get(&statement).ok_or_else(|| {
                    ProtoError::new(
                        "26000",
                        format!("prepared statement \"{statement}\" does not exist"),
                    )
                })?;
                if !portal.is_empty() && self.portals.contains_key(&portal) {
                    return Err(ProtoError::new(
                        "42P03",
                        format!("cursor \"{portal}\" already exists"),
                    ));
                }
                if self.portals.len() >= MAX_NAMED_OBJECTS {
                    return Err(ProtoError::new("53400", "too many portals"));
                }
                check_result_formats(&result_formats)?;
                let sql = extended::bind(stmt, &param_formats, &params)?;
                self.portals.insert(
                    portal,
                    Portal {
                        sql,
                        result_formats,
                        state: PortalState::Pending,
                    },
                );
                self.out.extend_from_slice(&encode_bind_complete());
            }
            FrontendMsg::Describe { kind: b'S', name } => self.describe_statement(&name)?,
            FrontendMsg::Describe { name, .. } => self.describe_portal(&name)?,
            FrontendMsg::Execute { portal, max_rows } => self.execute_portal(&portal, max_rows)?,
            FrontendMsg::Close { kind, name } => {
                // Closing a name that does not exist is not an error.
                if kind == b'S' {
                    self.statements.remove(&name);
                } else {
                    self.portals.remove(&name);
                }
                self.out.extend_from_slice(&encode_close_complete());
            }
            _ => {}
        }
        Ok(())
    }

    /// Run `sql` on the executor, mapping errors and reporting notices.
    fn run_sql(&mut self, sql: &str) -> Result<QueryResult, ProtoError> {
        let result = self
            .executor
            .execute(sql)
            .map_err(|e| ProtoError::new("XX000", e.to_string()))?;
        let mut notices = Vec::new();
        let result = unwrap_notices(result, &mut notices);
        for n in notices {
            self.out.extend_from_slice(&encode_notice_response(&n));
        }
        match result {
            QueryResult::Error(msg) => Err(ProtoError::new("XX000", msg)),
            other => Ok(other),
        }
    }

    /// Describe statement: ParameterDescription, then RowDescription (all
    /// text format) or NoData. The result columns of a row-returning
    /// statement are learned by running it with every parameter NULL.
    fn describe_statement(&mut self, name: &str) -> Result<(), ProtoError> {
        let stmt = self.statements.get(name).ok_or_else(|| {
            ProtoError::new(
                "26000",
                format!("prepared statement \"{name}\" does not exist"),
            )
        })?;
        let types = stmt.param_types.clone();
        let probe = if is_row_returning(&stmt.query) {
            let nulls = vec!["NULL".to_owned(); types.len()];
            Some(extended::substitute(&stmt.query, &nulls)?)
        } else {
            None
        };
        self.out
            .extend_from_slice(&encode_parameter_description(&types));
        let columns = match probe {
            // A probe that fails (e.g. NULL not accepted somewhere) only
            // means the shape is unknown up front.
            Some(sql) => match self.executor.execute(&sql) {
                Ok(r) => match unwrap_notices(r, &mut Vec::new()) {
                    QueryResult::Rows { columns, .. } => Some(columns),
                    _ => None,
                },
                Err(_) => None,
            },
            None => None,
        };
        match columns {
            Some(cols) => self
                .out
                .extend_from_slice(&encode_row_description_with_formats(&cols, &[])),
            None => self.out.extend_from_slice(&encode_no_data()),
        }
        Ok(())
    }

    /// Describe portal: RowDescription (with the Bind's result formats) or
    /// NoData. A row-returning portal is executed now and its rows buffered
    /// for the following Execute.
    fn describe_portal(&mut self, name: &str) -> Result<(), ProtoError> {
        let portal = self
            .portals
            .get(name)
            .ok_or_else(|| ProtoError::new("34000", format!("portal \"{name}\" does not exist")))?;
        if matches!(portal.state, PortalState::Pending) && is_row_returning(&portal.sql) {
            let sql = portal.sql.clone();
            let result = self.run_sql(&sql)?;
            let portal = self.portals.get_mut(name).expect("portal checked above");
            portal.state = match result {
                QueryResult::Rows { columns, rows } => PortalState::Rows {
                    columns,
                    rows: rows.into(),
                    sent: 0,
                },
                other => PortalState::Executed(other),
            };
        }
        let portal = &self.portals[name];
        match &portal.state {
            PortalState::Rows { columns, .. } => {
                self.out
                    .extend_from_slice(&encode_row_description_with_formats(
                        columns,
                        &portal.result_formats,
                    ))
            }
            _ => self.out.extend_from_slice(&encode_no_data()),
        }
        Ok(())
    }

    /// Execute: DataRows (at most `max_rows` when > 0) then CommandComplete,
    /// or PortalSuspended when rows remain.
    fn execute_portal(&mut self, name: &str, max_rows: i32) -> Result<(), ProtoError> {
        let portal = self
            .portals
            .get(name)
            .ok_or_else(|| ProtoError::new("34000", format!("portal \"{name}\" does not exist")))?;
        if matches!(portal.state, PortalState::Pending) {
            let sql = portal.sql.clone();
            if sql.trim().is_empty() {
                self.out.extend_from_slice(&encode_empty_query_response());
                return Ok(());
            }
            let result = self.run_sql(&sql)?;
            let portal = self.portals.get_mut(name).expect("portal checked above");
            portal.state = match result {
                QueryResult::Rows { columns, rows } => PortalState::Rows {
                    columns,
                    rows: rows.into(),
                    sent: 0,
                },
                other => PortalState::Executed(other),
            };
        }
        let limit = usize::try_from(max_rows)
            .ok()
            .filter(|&n| n > 0)
            .unwrap_or(usize::MAX);
        let portal = self.portals.get_mut(name).expect("portal checked above");
        let next = match &mut portal.state {
            PortalState::Rows { rows, sent, .. } => {
                let mut n = 0;
                while n < limit {
                    let Some(row) = rows.pop_front() else { break };
                    self.out.extend_from_slice(&encode_data_row(&row));
                    n += 1;
                }
                *sent += n;
                if rows.is_empty() {
                    let tag = format!("SELECT {sent}");
                    self.out.extend_from_slice(&encode_command_complete(&tag));
                    Some(PortalState::Done(tag))
                } else {
                    self.out.extend_from_slice(&encode_portal_suspended());
                    None
                }
            }
            PortalState::Executed(result) => {
                let tag = command_tag(result);
                self.out.extend_from_slice(&encode_command_complete(&tag));
                Some(PortalState::Done(tag))
            }
            PortalState::Done(tag) => {
                self.out.extend_from_slice(&encode_command_complete(tag));
                None
            }
            PortalState::Pending => None,
        };
        if let Some(state) = next {
            portal.state = state;
        }
        Ok(())
    }

    async fn handle_query(&mut self, sql: &str) -> std::io::Result<()> {
        match self.executor.execute(sql) {
            Ok(result) => self.send_result(result).await,
            Err(e) => {
                self.stream
                    .write_all(&encode_error_response(&e.to_string()))
                    .await
            }
        }
    }

    async fn send_result(&mut self, result: QueryResult) -> std::io::Result<()> {
        match result {
            QueryResult::Rows { columns, rows } => {
                let n = rows.len();
                self.stream
                    .write_all(&encode_row_description(&columns))
                    .await?;
                for row in &rows {
                    self.stream.write_all(&encode_data_row(row)).await?;
                }
                let tag = format!("SELECT {n}");
                self.stream
                    .write_all(&encode_command_complete(&tag))
                    .await?;
            }
            QueryResult::Affected(n) => {
                let tag = format!("INSERT 0 {n}");
                self.stream
                    .write_all(&encode_command_complete(&tag))
                    .await?;
            }
            QueryResult::Created => {
                self.stream
                    .write_all(&encode_command_complete("CREATE TABLE"))
                    .await?;
            }
            QueryResult::Dropped => {
                self.stream
                    .write_all(&encode_command_complete("DROP TABLE"))
                    .await?;
            }
            QueryResult::TxBegin => {
                self.stream
                    .write_all(&encode_command_complete("BEGIN"))
                    .await?;
            }
            QueryResult::TxCommit => {
                self.stream
                    .write_all(&encode_command_complete("COMMIT"))
                    .await?;
            }
            QueryResult::TxRollback => {
                self.stream
                    .write_all(&encode_command_complete("ROLLBACK"))
                    .await?;
            }
            QueryResult::ExplainPlan(lines) => {
                let body = lines.join("\n");
                self.stream
                    .write_all(&encode_command_complete(&format!("EXPLAIN\n{body}")))
                    .await?;
            }
            QueryResult::Error(msg) => {
                self.stream
                    .write_all(&encode_command_complete(&format!("ERROR: {msg}")))
                    .await?;
            }
            QueryResult::Notice(msg) => {
                self.stream
                    .write_all(&encode_command_complete(&format!("NOTICE: {msg}")))
                    .await?;
            }
            QueryResult::WithNotice { result, notice } => {
                self.stream
                    .write_all(&encode_command_complete(&format!("NOTICE: {notice}")))
                    .await?;
                Box::pin(self.send_result(*result)).await?;
            }
            QueryResult::Ok(msg) => {
                self.stream
                    .write_all(&encode_command_complete(&msg))
                    .await?;
            }
        }
        Ok(())
    }
}

/// CommandComplete tag for a result reported without rows (same tags as the
/// simple query protocol).
fn command_tag(result: &QueryResult) -> String {
    match result {
        QueryResult::Rows { rows, .. } => format!("SELECT {}", rows.len()),
        QueryResult::Affected(n) => format!("INSERT 0 {n}"),
        QueryResult::Created => "CREATE TABLE".to_owned(),
        QueryResult::Dropped => "DROP TABLE".to_owned(),
        QueryResult::TxBegin => "BEGIN".to_owned(),
        QueryResult::TxCommit => "COMMIT".to_owned(),
        QueryResult::TxRollback => "ROLLBACK".to_owned(),
        QueryResult::ExplainPlan(lines) => format!("EXPLAIN\n{}", lines.join("\n")),
        QueryResult::Error(msg) => format!("ERROR: {msg}"),
        QueryResult::Notice(msg) => format!("NOTICE: {msg}"),
        QueryResult::WithNotice { result, .. } => command_tag(result),
        QueryResult::Ok(msg) => msg.clone(),
    }
}

/// Open a `TableRegistry` from a directory (or create an empty one).
pub fn open_registry(dir: &std::path::Path) -> std::io::Result<Arc<TableRegistry>> {
    let registry = TableRegistry::open(dir, EngineOptions::default())
        .map_err(|e| std::io::Error::other(e.to_string()))?;
    Ok(Arc::new(registry))
}
