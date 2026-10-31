//! PostgreSQL wire protocol server.

use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;

use tokio::net::TcpListener;
use vectorvolt_engine::TableRegistry;
use vectorvolt_query::sql::executor::SqlExecutor;

use crate::session::{PgSession, open_registry};

/// Configuration for `PgServer`.
#[derive(Debug, Clone)]
pub struct PgConfig {
    /// Bind address (default 127.0.0.1:5432).
    pub bind_addr: SocketAddr,
    /// Data directory for the `TableRegistry`.
    pub data_dir: PathBuf,
}

impl Default for PgConfig {
    fn default() -> Self {
        Self {
            bind_addr: "127.0.0.1:5432".parse().unwrap(),
            data_dir: PathBuf::from("."),
        }
    }
}

impl PgConfig {
    pub fn new(bind_addr: SocketAddr, data_dir: PathBuf) -> Self {
        Self {
            bind_addr,
            data_dir,
        }
    }
}

/// PostgreSQL wire protocol server.
///
/// Listens for incoming TCP connections and spawns a `PgSession` per client.
/// Every session runs on a [`SqlExecutor::new_session`] of one shared
/// executor, so SQL security state (policies, table tokens, read-only mode,
/// masks, grants, ...) is server-wide and survives reconnects, while
/// transactions stay per connection.
pub struct PgServer {
    config: PgConfig,
    executor: Arc<SqlExecutor>,
    /// Cleartext password every client must present; `None` = no auth.
    password: Option<String>,
}

impl PgServer {
    /// Create a new server with the given config. Opens the registry from `config.data_dir`.
    pub fn new(config: PgConfig) -> std::io::Result<Self> {
        let registry = open_registry(&config.data_dir)?;
        Ok(Self::with_registry(config, registry))
    }

    /// Create a server with an already-open registry (useful for tests).
    pub fn with_registry(config: PgConfig, registry: Arc<TableRegistry>) -> Self {
        let executor = Arc::new(SqlExecutor::new(registry));
        Self {
            config,
            executor,
            password: None,
        }
    }

    /// Require clients to authenticate with this password
    /// (AuthenticationCleartextPassword, constant-time compare). `None`
    /// disables authentication.
    pub fn with_password(mut self, password: Option<String>) -> Self {
        self.password = password;
        self
    }

    /// The shared executor whose security state every session sees.
    pub fn executor(&self) -> &Arc<SqlExecutor> {
        &self.executor
    }

    /// Bind and return the listener and the actual bound address (useful when
    /// using port 0 in tests).
    pub async fn bind(&self) -> std::io::Result<(TcpListener, SocketAddr)> {
        let listener = TcpListener::bind(self.config.bind_addr).await?;
        let addr = listener.local_addr()?;
        Ok((listener, addr))
    }

    /// Accept one connection, drive its session to completion, and return.
    /// Useful for tests.
    pub async fn accept_one(&self, listener: &TcpListener) -> std::io::Result<()> {
        let (stream, _) = listener.accept().await?;
        let mut session =
            PgSession::new(stream, self.executor.new_session(), self.password.clone());
        session.run().await
    }

    /// Serve indefinitely (until the process is killed or an accept error occurs).
    pub async fn serve(self) -> std::io::Result<()> {
        let (listener, addr) = self.bind().await?;
        tracing::info!("PgServer listening on {addr}");
        loop {
            let (stream, peer) = listener.accept().await?;
            tracing::debug!("accepted connection from {peer}");
            let executor = self.executor.new_session();
            let password = self.password.clone();
            tokio::spawn(async move {
                let mut session = PgSession::new(stream, executor, password);
                if let Err(e) = session.run().await {
                    tracing::warn!("session error: {e}");
                }
            });
        }
    }

    /// Serve until `shutdown` resolves, then stop accepting NEW connections and
    /// return `Ok(())` (Gap #6). Sessions already in flight when shutdown fires
    /// are spawned tasks that keep running independently to completion — this
    /// does not additionally wait for them to finish, only stops taking new work.
    pub async fn serve_until(
        self,
        shutdown: impl std::future::Future<Output = ()>,
    ) -> std::io::Result<()> {
        let (listener, addr) = self.bind().await?;
        Self::accept_loop_until(
            Arc::clone(&self.executor),
            self.password.clone(),
            listener,
            addr,
            shutdown,
        )
        .await
    }

    /// Same accept-loop as [`serve_until`](Self::serve_until), but takes an
    /// already-bound listener — lets tests learn the real ephemeral port before
    /// the loop starts, the same way [`accept_one`](Self::accept_one) does.
    pub async fn accept_loop_until(
        executor: Arc<SqlExecutor>,
        password: Option<String>,
        listener: TcpListener,
        addr: SocketAddr,
        shutdown: impl std::future::Future<Output = ()>,
    ) -> std::io::Result<()> {
        tracing::info!("PgServer listening on {addr}");
        tokio::pin!(shutdown);
        loop {
            tokio::select! {
                accepted = listener.accept() => {
                    let (stream, peer) = accepted?;
                    tracing::debug!("accepted connection from {peer}");
                    let session_exec = executor.new_session();
                    let password = password.clone();
                    tokio::spawn(async move {
                        let mut session = PgSession::new(stream, session_exec, password);
                        if let Err(e) = session.run().await {
                            tracing::warn!("session error: {e}");
                        }
                    });
                }
                _ = &mut shutdown => {
                    tracing::info!("PgServer on {addr} shutting down (no longer accepting new connections)");
                    return Ok(());
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use bytes::BufMut;

    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    async fn connect_and_startup(addr: SocketAddr) -> tokio::net::TcpStream {
        let mut stream = tokio::net::TcpStream::connect(addr).await.unwrap();
        // Send startup message
        let params = b"user\0postgres\0database\0vectorvolt\0\0";
        let len = 4 + 4 + params.len();
        let mut buf = bytes::BytesMut::new();
        buf.put_i32(len as i32);
        buf.put_i32(3 << 16); // version 3.0
        buf.put(params.as_ref());
        stream.write_all(&buf).await.unwrap();
        stream
    }

    fn make_query_msg(sql: &str) -> Vec<u8> {
        let payload = format!("{sql}\0");
        let len = 4 + payload.len();
        let mut buf = bytes::BytesMut::new();
        buf.put_u8(b'Q');
        buf.put_i32(len as i32);
        buf.put(payload.as_bytes());
        buf.to_vec()
    }

    async fn open_server_on_ephemeral(
        dir: &std::path::Path,
    ) -> (PgServer, TcpListener, SocketAddr) {
        let config = PgConfig::new("127.0.0.1:0".parse().unwrap(), dir.to_path_buf());
        let server = PgServer::new(config).unwrap();
        let (listener, addr) = server.bind().await.unwrap();
        (server, listener, addr)
    }

    // ── Gap #6: graceful shutdown ────────────────────────────────────────────
    #[tokio::test]
    async fn test_accept_loop_until_serves_before_shutdown() {
        let dir = tempfile::tempdir().unwrap();
        let (server, listener, addr) = open_server_on_ephemeral(dir.path()).await;
        let (tx, rx) = tokio::sync::oneshot::channel::<()>();
        let executor = server.executor.clone();
        let handle = tokio::spawn(async move {
            PgServer::accept_loop_until(executor, None, listener, addr, async {
                let _ = rx.await;
            })
            .await
        });
        // A connection made before shutdown fires must still be served.
        let mut stream = connect_and_startup(addr).await;
        let mut buf = vec![0u8; 4096];
        let n = stream.read(&mut buf).await.unwrap();
        assert!(n > 0);
        assert_eq!(buf[0], b'R'); // AuthOk

        tx.send(()).unwrap();
        handle.await.unwrap().unwrap();
    }

    #[tokio::test]
    async fn test_accept_loop_until_stops_accepting_after_shutdown() {
        let dir = tempfile::tempdir().unwrap();
        let (server, listener, addr) = open_server_on_ephemeral(dir.path()).await;
        let (tx, rx) = tokio::sync::oneshot::channel::<()>();
        let executor = server.executor.clone();
        // Fire shutdown immediately, before any connection is made.
        tx.send(()).unwrap();
        let result = PgServer::accept_loop_until(executor, None, listener, addr, async {
            let _ = rx.await;
        })
        .await;
        assert!(result.is_ok());
        // The listener has been dropped with the accept loop, so a new connect
        // attempt to the same address must fail (nothing is listening anymore).
        assert!(tokio::net::TcpStream::connect(addr).await.is_err());
    }

    #[tokio::test]
    async fn test_server_accepts_connection() {
        let dir = tempfile::tempdir().unwrap();
        let (server, listener, addr) = open_server_on_ephemeral(dir.path()).await;
        tokio::spawn(async move {
            server.accept_one(&listener).await.unwrap();
        });
        let mut stream = connect_and_startup(addr).await;
        let mut buf = vec![0u8; 4096];
        let n = stream.read(&mut buf).await.unwrap();
        assert!(n > 0);
        assert_eq!(buf[0], b'R'); // AuthOk
    }

    #[tokio::test]
    async fn test_server_sends_auth_ok() {
        let dir = tempfile::tempdir().unwrap();
        let (server, listener, addr) = open_server_on_ephemeral(dir.path()).await;
        tokio::spawn(async move {
            server.accept_one(&listener).await.unwrap();
        });
        let mut stream = connect_and_startup(addr).await;
        let mut buf = vec![0u8; 4096];
        let n = stream.read(&mut buf).await.unwrap();
        assert!(n >= 9);
        assert_eq!(buf[0], b'R');
        let payload = i32::from_be_bytes([buf[5], buf[6], buf[7], buf[8]]);
        assert_eq!(payload, 0); // AuthOk payload
    }

    #[tokio::test]
    async fn test_server_sends_ready_for_query() {
        let dir = tempfile::tempdir().unwrap();
        let (server, listener, addr) = open_server_on_ephemeral(dir.path()).await;
        tokio::spawn(async move {
            server.accept_one(&listener).await.unwrap();
        });
        let mut stream = connect_and_startup(addr).await;
        let mut buf = vec![0u8; 4096];
        let n = stream.read(&mut buf).await.unwrap();
        // find 'Z' somewhere in response
        assert!(buf[..n].contains(&b'Z'));
    }

    #[tokio::test]
    async fn test_query_create_table() {
        let dir = tempfile::tempdir().unwrap();
        let (server, listener, addr) = open_server_on_ephemeral(dir.path()).await;
        tokio::spawn(async move {
            server.accept_one(&listener).await.unwrap();
        });
        let mut stream = connect_and_startup(addr).await;
        let mut buf = vec![0u8; 4096];
        let _ = stream.read(&mut buf).await.unwrap();

        // Send CREATE TABLE
        stream
            .write_all(&make_query_msg("CREATE TABLE t (pk U64, val U64)"))
            .await
            .unwrap();
        let mut resp = vec![0u8; 4096];
        let n = stream.read(&mut resp).await.unwrap();
        assert!(n > 0);
        // Response should contain CommandComplete ('C') or ErrorResponse ('E')
        assert!(resp[..n].contains(&b'C') || resp[..n].contains(&b'E'));
    }

    #[tokio::test]
    async fn test_query_insert_select() {
        let dir = tempfile::tempdir().unwrap();
        let (server, listener, addr) = open_server_on_ephemeral(dir.path()).await;
        tokio::spawn(async move {
            server.accept_one(&listener).await.unwrap();
        });
        let mut stream = connect_and_startup(addr).await;
        let mut buf = vec![0u8; 4096];
        let _ = stream.read(&mut buf).await.unwrap();

        stream
            .write_all(&make_query_msg("CREATE TABLE t (pk U64, val U64)"))
            .await
            .unwrap();
        let _ = stream.read(&mut vec![0u8; 4096]).await.unwrap();

        stream
            .write_all(&make_query_msg("INSERT INTO t (pk, val) VALUES (1, 99)"))
            .await
            .unwrap();
        let _ = stream.read(&mut vec![0u8; 4096]).await.unwrap();

        stream
            .write_all(&make_query_msg("SELECT * FROM t"))
            .await
            .unwrap();
        let mut resp = vec![0u8; 4096];
        let n = stream.read(&mut resp).await.unwrap();
        assert!(n > 0);
        assert!(resp[..n].contains(&b'T')); // RowDescription
    }

    #[tokio::test]
    async fn test_query_error_response() {
        let dir = tempfile::tempdir().unwrap();
        let (server, listener, addr) = open_server_on_ephemeral(dir.path()).await;
        tokio::spawn(async move {
            server.accept_one(&listener).await.unwrap();
        });
        let mut stream = connect_and_startup(addr).await;
        let mut buf = vec![0u8; 4096];
        let _ = stream.read(&mut buf).await.unwrap();

        stream
            .write_all(&make_query_msg("SELECT * FROM nonexistent"))
            .await
            .unwrap();
        let mut resp = vec![0u8; 4096];
        let n = stream.read(&mut resp).await.unwrap();
        assert!(n > 0);
        assert!(resp[..n].contains(&b'E')); // ErrorResponse
    }

    #[tokio::test]
    async fn test_terminate_closes_session() {
        let dir = tempfile::tempdir().unwrap();
        let (server, listener, addr) = open_server_on_ephemeral(dir.path()).await;
        let handle = tokio::spawn(async move {
            server.accept_one(&listener).await.unwrap();
        });
        let mut stream = connect_and_startup(addr).await;
        let mut buf = vec![0u8; 4096];
        let _ = stream.read(&mut buf).await.unwrap();

        // Send Terminate
        let mut term = bytes::BytesMut::new();
        term.put_u8(b'X');
        term.put_i32(4);
        stream.write_all(&term).await.unwrap();
        handle.await.unwrap();
    }

    // ── A3: password authentication ─────────────────────────────────────────

    /// Read until the reply ends with ReadyForQuery, or the server closes.
    async fn read_reply(stream: &mut tokio::net::TcpStream) -> Vec<u8> {
        let mut out = Vec::new();
        let mut buf = vec![0u8; 4096];
        loop {
            let n = stream.read(&mut buf).await.unwrap_or(0);
            if n == 0 {
                return out;
            }
            out.extend_from_slice(&buf[..n]);
            if out.len() >= 6 && out[out.len() - 6] == b'Z' {
                return out;
            }
        }
    }

    fn password_msg(pw: &str) -> Vec<u8> {
        let mut buf = bytes::BytesMut::new();
        buf.put_u8(b'p');
        buf.put_i32((4 + pw.len() + 1) as i32);
        buf.put(pw.as_bytes());
        buf.put_u8(0);
        buf.to_vec()
    }

    async fn auth_server(dir: &std::path::Path) -> (SocketAddr, tokio::task::JoinHandle<()>) {
        let config = PgConfig::new("127.0.0.1:0".parse().unwrap(), dir.to_path_buf());
        let server = PgServer::new(config)
            .unwrap()
            .with_password(Some("s3cret".to_owned()));
        let (listener, addr) = server.bind().await.unwrap();
        let handle = tokio::spawn(async move {
            server.accept_one(&listener).await.unwrap();
        });
        (addr, handle)
    }

    /// Request code carried by the first AuthenticationXxx ('R') message.
    fn auth_code(reply: &[u8]) -> i32 {
        assert_eq!(reply[0], b'R', "{reply:?}");
        i32::from_be_bytes([reply[5], reply[6], reply[7], reply[8]])
    }

    #[tokio::test]
    async fn test_password_auth_good_password() {
        let dir = tempfile::tempdir().unwrap();
        let (addr, handle) = auth_server(dir.path()).await;
        let mut stream = connect_and_startup(addr).await;
        let mut buf = vec![0u8; 9];
        stream.read_exact(&mut buf).await.unwrap();
        assert_eq!(auth_code(&buf), 3); // AuthenticationCleartextPassword
        stream.write_all(&password_msg("s3cret")).await.unwrap();
        let reply = read_reply(&mut stream).await;
        assert_eq!(auth_code(&reply), 0); // AuthenticationOk
        assert_eq!(reply[reply.len() - 6], b'Z');
        stream
            .write_all(&make_query_msg("CREATE TABLE t (pk U64)"))
            .await
            .unwrap();
        let reply = read_reply(&mut stream).await;
        assert_eq!(reply[0], b'C');
        drop(stream);
        handle.await.unwrap();
    }

    #[tokio::test]
    async fn test_password_auth_bad_password_rejected() {
        let dir = tempfile::tempdir().unwrap();
        let (addr, handle) = auth_server(dir.path()).await;
        let mut stream = connect_and_startup(addr).await;
        let mut buf = vec![0u8; 9];
        stream.read_exact(&mut buf).await.unwrap();
        assert_eq!(auth_code(&buf), 3);
        stream.write_all(&password_msg("wrong")).await.unwrap();
        let mut reply = Vec::new();
        stream.read_to_end(&mut reply).await.unwrap(); // server closes
        assert_eq!(reply[0], b'E');
        assert!(reply.windows(5).any(|w| w == b"28P01"));
        assert!(!reply.contains(&b'Z'));
        handle.await.unwrap();
    }

    #[tokio::test]
    async fn test_password_auth_query_instead_of_password_rejected() {
        let dir = tempfile::tempdir().unwrap();
        let (addr, handle) = auth_server(dir.path()).await;
        let mut stream = connect_and_startup(addr).await;
        let mut buf = vec![0u8; 9];
        stream.read_exact(&mut buf).await.unwrap();
        stream
            .write_all(&make_query_msg("SELECT * FROM t"))
            .await
            .unwrap();
        let mut reply = Vec::new();
        stream.read_to_end(&mut reply).await.unwrap();
        assert_eq!(reply[0], b'E');
        handle.await.unwrap();
    }

    #[tokio::test]
    async fn test_malformed_startup_rejected() {
        let dir = tempfile::tempdir().unwrap();
        for bad in [
            // length shorter than the header itself
            vec![0u8, 0, 0, 3],
            // protocol 2.0
            {
                let mut b = bytes::BytesMut::new();
                b.put_i32(9);
                b.put_i32(2 << 16);
                b.put_u8(0);
                b.to_vec()
            },
            // params not NUL-terminated
            {
                let mut b = bytes::BytesMut::new();
                b.put_i32(12);
                b.put_i32(3 << 16);
                b.put(&b"user"[..]);
                b.to_vec()
            },
        ] {
            let (srv, listener, addr) = open_server_on_ephemeral(dir.path()).await;
            let handle = tokio::spawn(async move {
                srv.accept_one(&listener).await.unwrap();
            });
            let mut stream = tokio::net::TcpStream::connect(addr).await.unwrap();
            stream.write_all(&bad).await.unwrap();
            let mut reply = Vec::new();
            stream.read_to_end(&mut reply).await.unwrap();
            assert_eq!(reply.first(), Some(&b'E'), "{bad:?}");
            assert!(!reply.contains(&b'Z'), "{bad:?}");
            handle.await.unwrap();
        }
    }

    #[tokio::test]
    async fn test_ssl_request_declined_then_startup() {
        let dir = tempfile::tempdir().unwrap();
        let (server, listener, addr) = open_server_on_ephemeral(dir.path()).await;
        tokio::spawn(async move {
            server.accept_one(&listener).await.unwrap();
        });
        let mut stream = tokio::net::TcpStream::connect(addr).await.unwrap();
        let mut ssl = bytes::BytesMut::new();
        ssl.put_i32(8);
        ssl.put_i32(80_877_103);
        stream.write_all(&ssl).await.unwrap();
        let mut n = [0u8; 1];
        stream.read_exact(&mut n).await.unwrap();
        assert_eq!(n[0], b'N');
        let params = b"user\0postgres\0\0";
        let mut buf = bytes::BytesMut::new();
        buf.put_i32((8 + params.len()) as i32);
        buf.put_i32(3 << 16);
        buf.put(params.as_ref());
        stream.write_all(&buf).await.unwrap();
        let reply = read_reply(&mut stream).await;
        assert_eq!(auth_code(&reply), 0);
    }

    // ── A4: SQL security state is shared across connections ─────────────────
    #[tokio::test]
    async fn test_restriction_survives_reconnect() {
        let dir = tempfile::tempdir().unwrap();
        let (server, listener, addr) = open_server_on_ephemeral(dir.path()).await;
        let handle = tokio::spawn(async move {
            server.accept_one(&listener).await.unwrap();
            server.accept_one(&listener).await.unwrap();
        });
        let terminate = [b'X', 0, 0, 0, 4];

        let mut c1 = connect_and_startup(addr).await;
        read_reply(&mut c1).await;
        for sql in ["CREATE TABLE ro (pk U64, v U64)", "SET READ ONLY"] {
            c1.write_all(&make_query_msg(sql)).await.unwrap();
            read_reply(&mut c1).await;
        }
        c1.write_all(&terminate).await.unwrap();
        drop(c1);

        let mut c2 = connect_and_startup(addr).await;
        read_reply(&mut c2).await;
        c2.write_all(&make_query_msg("INSERT INTO ro (pk, v) VALUES (1, 1)"))
            .await
            .unwrap();
        let reply = read_reply(&mut c2).await;
        assert!(
            reply.windows(5).any(|w| w == b"25006"),
            "{}",
            String::from_utf8_lossy(&reply)
        );
        c2.write_all(&terminate).await.unwrap();
        handle.await.unwrap();
    }

    #[tokio::test]
    async fn test_pg_config_default() {
        let cfg = PgConfig::default();
        assert_eq!(cfg.bind_addr.port(), 5432);
    }

    #[tokio::test]
    async fn test_server_multiple_queries() {
        let dir = tempfile::tempdir().unwrap();
        let config = PgConfig::new("127.0.0.1:0".parse().unwrap(), dir.path().to_path_buf());
        let registry = crate::session::open_registry(dir.path()).unwrap();
        let server = PgServer::with_registry(config, registry);
        let (listener, addr) = server.bind().await.unwrap();
        tokio::spawn(async move {
            server.accept_one(&listener).await.unwrap();
        });
        let mut stream = connect_and_startup(addr).await;
        let mut buf = vec![0u8; 4096];
        let _ = stream.read(&mut buf).await.unwrap();

        stream
            .write_all(&make_query_msg("CREATE TABLE m (pk U64)"))
            .await
            .unwrap();
        let _ = stream.read(&mut vec![0u8; 4096]).await.unwrap();
        stream
            .write_all(&make_query_msg("SELECT * FROM m"))
            .await
            .unwrap();
        let mut resp = vec![0u8; 4096];
        let n = stream.read(&mut resp).await.unwrap();
        assert!(n > 0);
    }

    #[tokio::test]
    async fn test_wire_encode_decode_roundtrip() {
        use crate::wire::{encode_command_complete, encode_data_row, encode_row_description};
        let cols = vec!["pk".to_owned(), "val".to_owned()];
        let rd = encode_row_description(&cols);
        assert_eq!(rd[0], b'T');
        let row = vec!["1".to_owned(), "42".to_owned()];
        let dr = encode_data_row(&row);
        assert_eq!(dr[0], b'D');
        let cc = encode_command_complete("SELECT 1");
        assert_eq!(cc[0], b'C');
    }
}
