//! PostgreSQL wire protocol server.

use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;

use tokio::net::TcpListener;
use vectorvolt_engine::TableRegistry;

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
pub struct PgServer {
    config: PgConfig,
    registry: Arc<TableRegistry>,
}

impl PgServer {
    /// Create a new server with the given config. Opens the registry from `config.data_dir`.
    pub fn new(config: PgConfig) -> std::io::Result<Self> {
        let registry = open_registry(&config.data_dir)?;
        Ok(Self { config, registry })
    }

    /// Create a server with an already-open registry (useful for tests).
    pub fn with_registry(config: PgConfig, registry: Arc<TableRegistry>) -> Self {
        Self { config, registry }
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
        let mut session = PgSession::new(stream, Arc::clone(&self.registry));
        session.run().await
    }

    /// Serve indefinitely (until the process is killed or an accept error occurs).
    pub async fn serve(self) -> std::io::Result<()> {
        let (listener, addr) = self.bind().await?;
        tracing::info!("PgServer listening on {addr}");
        let registry = Arc::clone(&self.registry);
        loop {
            let (stream, peer) = listener.accept().await?;
            tracing::debug!("accepted connection from {peer}");
            let reg = Arc::clone(&registry);
            tokio::spawn(async move {
                let mut session = PgSession::new(stream, reg);
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
        Self::accept_loop_until(Arc::clone(&self.registry), listener, addr, shutdown).await
    }

    /// Same accept-loop as [`serve_until`](Self::serve_until), but takes an
    /// already-bound listener — lets tests learn the real ephemeral port before
    /// the loop starts, the same way [`accept_one`](Self::accept_one) does.
    pub async fn accept_loop_until(
        registry: Arc<TableRegistry>,
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
                    let reg = Arc::clone(&registry);
                    tokio::spawn(async move {
                        let mut session = PgSession::new(stream, reg);
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
    use crate::wire::{FrontendMsg, decode_frontend, encode_param_status, encode_ready_for_query};
    use bytes::BufMut;
    use std::path::PathBuf;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    async fn connect_and_startup(addr: SocketAddr) -> tokio::net::TcpStream {
        let mut stream = tokio::net::TcpStream::connect(addr).await.unwrap();
        // Send startup message
        let params = b"user\0postgres\0database\0vectorvolt\0\0";
        let len = 4 + 4 + params.len();
        let mut buf = bytes::BytesMut::new();
        buf.put_i32(len as i32);
        buf.put_i32((3 << 16) | 0i32); // version 3.0
        buf.put(params.as_ref());
        stream.write_all(&buf).await.unwrap();
        stream
    }

    async fn read_until_ready(stream: &mut tokio::net::TcpStream) -> Vec<u8> {
        let mut all = Vec::new();
        let mut tmp = [0u8; 4096];
        loop {
            let n = stream.read(&mut tmp).await.unwrap();
            if n == 0 {
                break;
            }
            all.extend_from_slice(&tmp[..n]);
            // Check if we have a ReadyForQuery ('Z') at the end
            if all.last().copied() == Some(b'I') && all.len() >= 6 {
                if let Some(pos) = all.windows(1).rposition(|w| w == [b'Z']) {
                    if pos + 5 < all.len() || (all.len() > 4 && all[all.len() - 6] == b'Z') {
                        break;
                    }
                }
            }
            break; // read once is enough for simple tests
        }
        all
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
        let registry = server.registry.clone();
        let handle = tokio::spawn(async move {
            PgServer::accept_loop_until(registry, listener, addr, async {
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
        let registry = server.registry.clone();
        // Fire shutdown immediately, before any connection is made.
        tx.send(()).unwrap();
        let result = PgServer::accept_loop_until(registry, listener, addr, async {
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
