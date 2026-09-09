//! PostgreSQL wire protocol session handler.

use std::sync::Arc;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use vectorvolt_engine::{EngineOptions, TableRegistry};

use crate::wire::{
    FrontendMsg, decode_frontend, decode_startup, encode_auth_ok, encode_command_complete,
    encode_data_row, encode_error_response, encode_param_status, encode_ready_for_query,
    encode_row_description,
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

/// A single client session over a `TcpStream`.
pub struct PgSession {
    stream: TcpStream,
    executor: SqlExecutor,
    state: SessionState,
}

impl PgSession {
    pub fn new(stream: TcpStream, registry: Arc<TableRegistry>) -> Self {
        Self {
            stream,
            executor: SqlExecutor::new(registry),
            state: SessionState::Startup,
        }
    }

    /// Drive the session to completion.
    pub async fn run(&mut self) -> std::io::Result<()> {
        // ── startup ────────────────────────────────────────────────────────────
        let mut buf = vec![0u8; 4096];
        let n = self.stream.read(&mut buf).await?;
        if n == 0 {
            return Ok(());
        }
        // Decode startup message (no type byte)
        let _ = decode_startup(&buf[..n]); // ignore parse errors for now

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
                FrontendMsg::Startup { .. } => {
                    // Unexpected second startup
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
            QueryResult::ExplainPlan(lines) => {
                let body = lines.join("\n");
                self.stream
                    .write_all(&encode_command_complete(&format!("EXPLAIN\n{body}")))
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
