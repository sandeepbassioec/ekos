//! PostgreSQL wire protocol session handler.

use std::sync::Arc;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use vectorvolt_engine::{EngineOptions, TableRegistry};

use subtle::ConstantTimeEq;

use crate::wire::{
    FrontendMsg, decode_frontend, decode_startup, encode_auth_cleartext_password, encode_auth_ok,
    encode_command_complete, encode_data_row, encode_error_response, encode_fatal_error,
    encode_param_status, encode_ready_for_query, encode_row_description,
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

/// A single client session over a `TcpStream`.
pub struct PgSession {
    stream: TcpStream,
    executor: SqlExecutor,
    /// Required password (cleartext-password auth); `None` = no auth.
    password: Option<String>,
    state: SessionState,
}

impl PgSession {
    /// Create a session. `executor` should be a per-connection session of the
    /// server's shared executor ([`SqlExecutor::new_session`]) so security
    /// state is server-wide while transactions stay per connection.
    pub fn new(stream: TcpStream, executor: SqlExecutor, password: Option<String>) -> Self {
        Self {
            stream,
            executor,
            password,
            state: SessionState::Startup,
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
        // SSLRequest / GSSENCRequest: encryption is not offered ('N'); the
        // client then sends the real startup message in plaintext.
        let code = i32::from_be_bytes([buf[4], buf[5], buf[6], buf[7]]);
        if buf.len() == 8 && (code == 80_877_103 || code == 80_877_104) {
            self.stream.write_all(b"N").await?;
            let Some(next) = self.read_preauth(false).await? else {
                return self.fail("08P01", "invalid startup packet").await;
            };
            buf = next;
        }
        match decode_startup(&buf) {
            Ok(FrontendMsg::Startup { major: 3, .. }) => {}
            _ => {
                return self
                    .fail("08P01", "invalid startup packet or unsupported protocol")
                    .await;
            }
        }

        // ── authentication ───────────────────────────────────────────────────
        if let Some(expected) = self.password.clone() {
            self.stream
                .write_all(&encode_auth_cleartext_password())
                .await?;
            let supplied = match self.read_preauth(true).await? {
                Some(msg) => match decode_frontend(&msg) {
                    Ok(FrontendMsg::Password(pw)) => Some(pw),
                    _ => None,
                },
                None => None,
            };
            let ok =
                supplied.is_some_and(|pw| bool::from(pw.as_bytes().ct_eq(expected.as_bytes())));
            if !ok {
                return self.fail("28P01", "password authentication failed").await;
            }
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
        self.state = SessionState::Idle;

        // ── query loop ─────────────────────────────────────────────────────────
        loop {
            let mut buf = vec![0u8; 65536];
            let n = self.stream.read(&mut buf).await?;
            if n == 0 {
                break;
            }
            let msg = match decode_frontend(&buf[..n]) {
                Ok(m) => m,
                Err(_) => break,
            };

            match msg {
                FrontendMsg::Terminate => {
                    self.state = SessionState::Done;
                    break;
                }
                FrontendMsg::Query(sql) => {
                    self.state = SessionState::Busy;
                    self.handle_query(&sql).await?;
                    self.stream.write_all(&encode_ready_for_query()).await?;
                    self.state = SessionState::Idle;
                }
                FrontendMsg::Startup { .. } | FrontendMsg::Password(_) => {
                    // Unexpected second startup / stray password message
                    break;
                }
            }
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

/// Open a `TableRegistry` from a directory (or create an empty one).
pub fn open_registry(dir: &std::path::Path) -> std::io::Result<Arc<TableRegistry>> {
    let registry = TableRegistry::open(dir, EngineOptions::default())
        .map_err(|e| std::io::Error::other(e.to_string()))?;
    Ok(Arc::new(registry))
}
