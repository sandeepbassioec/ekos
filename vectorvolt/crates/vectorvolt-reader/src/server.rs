//! PostgreSQL wire protocol server.

use std::net::{IpAddr, SocketAddr};
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use tokio::io::AsyncWriteExt;
use tokio::net::{TcpListener, TcpStream};
use tokio_rustls::TlsAcceptor;
use vectorvolt_engine::TableRegistry;
use vectorvolt_query::sql::executor::SqlExecutor;

use crate::auth::{AuthMethod, Credentials};
use crate::session::{PgSession, open_registry};
use crate::wire::encode_fatal_error;

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

/// Everything the accept loop needs per connection: shared executor,
/// credentials, TLS policy and the admission limits (S83 / S116).
#[derive(Clone)]
struct ConnPolicy {
    executor: Arc<SqlExecutor>,
    credentials: Credentials,
    tls: Option<TlsAcceptor>,
    require_tls: bool,
    /// Operator cap (e.g. `vvctl serve --pgwire-max-connections`).
    max_connections: Option<usize>,
    /// Sessions currently running on this listener.
    live: Arc<AtomicUsize>,
}

/// Decrements the live-session count when a session ends, however it ends.
struct LiveGuard(Arc<AtomicUsize>);

impl Drop for LiveGuard {
    fn drop(&mut self) {
        self.0.fetch_sub(1, Ordering::AcqRel);
    }
}

/// Fold an IPv4-mapped IPv6 address (`::ffff:a.b.c.d`, what a dual-stack
/// `[::]` listener reports for IPv4 peers) back to IPv4, so allowlist entries
/// written as `10.0.0.5` match either way.
fn canonical_ip(ip: IpAddr) -> IpAddr {
    match ip {
        IpAddr::V6(v6) => v6.to_ipv4_mapped().map(IpAddr::V4).unwrap_or(ip),
        v4 => v4,
    }
}

impl ConnPolicy {
    /// The executor's S116 `max_connections` (`SET MAX_CONNECTIONS n`,
    /// default 100). Shared by every session, so a `SET` over any pgwire
    /// connection applies here.
    fn executor_limit(&self) -> usize {
        usize::try_from(self.executor.max_connections()).unwrap_or(usize::MAX)
    }

    /// The effective connection limit: the tighter of the operator cap and
    /// the executor's S116 setting.
    fn connection_limit(&self) -> usize {
        self.executor_limit()
            .min(self.max_connections.unwrap_or(usize::MAX))
    }

    /// Admit `peer` or say why not, as (SQLSTATE, message).
    fn admit(&self, peer: SocketAddr) -> Result<LiveGuard, (&'static str, String)> {
        let ip = canonical_ip(peer.ip());
        // S83 — IP allowlist (empty = everyone allowed).
        if !self.executor.check_ip_allowed(&ip.to_string()) {
            return Err((
                "28000",
                format!("connection from host \"{ip}\" rejected: not in the IP allowlist"),
            ));
        }
        // S116 — max_connections. Reserve the slot first, then check, so two
        // concurrent accepts cannot both squeeze into the last slot.
        let limit = self.connection_limit();
        let prev = self.live.fetch_add(1, Ordering::AcqRel);
        let guard = LiveGuard(Arc::clone(&self.live));
        if prev >= limit {
            drop(guard);
            return Err(("53300", "sorry, too many clients already".to_owned()));
        }
        Ok(guard)
    }

    /// Build the session for an admitted connection.
    fn session(&self, stream: TcpStream) -> PgSession {
        PgSession::new(
            stream,
            self.executor.new_session(),
            self.credentials.clone(),
        )
        .with_tls(self.tls.clone(), self.require_tls)
    }

    /// Admit and run one connection to completion.
    async fn handle(&self, mut stream: TcpStream, peer: SocketAddr) -> std::io::Result<()> {
        let _guard = match self.admit(peer) {
            Ok(g) => g,
            Err((code, msg)) => {
                tracing::info!("pgwire: refusing {peer}: {msg}");
                // Sent before any negotiation: libpq reports an ErrorResponse
                // received in place of the SSLRequest / startup answer.
                let _ = stream.write_all(&encode_fatal_error(code, &msg)).await;
                let _ = stream.flush().await;
                let _ = stream.shutdown().await;
                return Ok(());
            }
        };
        self.session(stream).run().await
    }

    /// Accept connections until `shutdown` resolves.
    async fn accept_until(
        self,
        listener: TcpListener,
        addr: SocketAddr,
        shutdown: impl std::future::Future<Output = ()>,
    ) -> std::io::Result<()> {
        tracing::info!("PgServer listening on {addr}");
        tokio::pin!(shutdown);
        loop {
            tokio::select! {
                accepted = listener.accept() => {
                    let (stream, peer) = match accepted {
                        Ok(a) => a,
                        // A peer that resets mid-accept must not stop the
                        // listener for everyone else.
                        Err(e) => {
                            tracing::warn!("pgwire accept error: {e}");
                            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
                            continue;
                        }
                    };
                    tracing::debug!("accepted connection from {peer}");
                    let policy = self.clone();
                    tokio::spawn(async move {
                        if let Err(e) = policy.handle(stream, peer).await {
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

/// PostgreSQL wire protocol server.
///
/// Listens for incoming TCP connections and spawns a `PgSession` per client.
/// Every session runs on a [`SqlExecutor::new_session`] of one shared
/// executor, so SQL security state (policies, table tokens, read-only mode,
/// masks, grants, ...) is server-wide and survives reconnects, while
/// transactions stay per connection.
///
/// At accept time every connection is checked against the executor's IP
/// allowlist (S83, SQLSTATE 28000) and the connection limit (S116, SQLSTATE
/// 53300) before a session starts. With [`with_tls`](Self::with_tls) an
/// SSLRequest is answered 'S' and the connection is upgraded to TLS.
pub struct PgServer {
    config: PgConfig,
    executor: Arc<SqlExecutor>,
    /// Password every client must prove it knows; `None` = no auth.
    password: Option<String>,
    /// How the password is checked (default SCRAM-SHA-256).
    auth_method: AuthMethod,
    /// Derived from `password` + `auth_method` (for SCRAM: the verifier,
    /// computed once with a random per-server salt).
    credentials: Credentials,
    tls: Option<Arc<rustls::ServerConfig>>,
    require_tls: bool,
    max_connections: Option<usize>,
    live: Arc<AtomicUsize>,
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
            auth_method: AuthMethod::default(),
            credentials: Credentials::None,
            tls: None,
            require_tls: false,
            max_connections: None,
            live: Arc::new(AtomicUsize::new(0)),
        }
    }

    /// Require clients to authenticate with this password, using the
    /// configured [`with_auth_method`](Self::with_auth_method) (default
    /// SCRAM-SHA-256). `None` disables authentication.
    pub fn with_password(mut self, password: Option<String>) -> Self {
        self.password = password;
        self.credentials = Credentials::new(self.password.as_deref(), self.auth_method);
        self
    }

    /// Choose how the password is checked: SCRAM-SHA-256 (default), MD5 or
    /// cleartext. Has no effect without [`with_password`](Self::with_password).
    pub fn with_auth_method(mut self, method: AuthMethod) -> Self {
        self.auth_method = method;
        self.credentials = Credentials::new(self.password.as_deref(), method);
        self
    }

    /// The configured authentication method.
    pub fn auth_method(&self) -> AuthMethod {
        self.auth_method
    }

    /// Offer TLS on SSLRequest with this rustls config (`None` = answer 'N').
    /// A config with a client-certificate verifier makes it mutual TLS.
    pub fn with_tls(mut self, tls: Option<Arc<rustls::ServerConfig>>) -> Self {
        self.tls = tls;
        self
    }

    /// Refuse clients that do not upgrade to TLS (SQLSTATE 28000).
    pub fn with_require_tls(mut self, require: bool) -> Self {
        self.require_tls = require;
        self
    }

    /// Cap concurrent sessions on this listener. The executor's own S116
    /// `max_connections` (default 100) also applies; the tighter one wins.
    pub fn with_max_connections(mut self, max: Option<usize>) -> Self {
        self.max_connections = max;
        self
    }

    /// The shared executor whose security state every session sees.
    pub fn executor(&self) -> &Arc<SqlExecutor> {
        &self.executor
    }

    /// Sessions currently running on this server.
    pub fn live_connections(&self) -> usize {
        self.live.load(Ordering::Acquire)
    }

    fn policy(&self) -> ConnPolicy {
        ConnPolicy {
            executor: Arc::clone(&self.executor),
            credentials: self.credentials.clone(),
            tls: self.tls.clone().map(TlsAcceptor::from),
            require_tls: self.require_tls,
            max_connections: self.max_connections,
            live: Arc::clone(&self.live),
        }
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
        let (stream, peer) = listener.accept().await?;
        self.policy().handle(stream, peer).await
    }

    /// Serve indefinitely (until the process is killed).
    pub async fn serve(self) -> std::io::Result<()> {
        self.serve_until(std::future::pending()).await
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
        self.serve_listener_until(listener, addr, shutdown).await
    }

    /// [`serve_until`](Self::serve_until) on an already-bound listener, with
    /// this server's full policy (password, TLS, admission limits).
    pub async fn serve_listener_until(
        self,
        listener: TcpListener,
        addr: SocketAddr,
        shutdown: impl std::future::Future<Output = ()>,
    ) -> std::io::Result<()> {
        self.policy().accept_until(listener, addr, shutdown).await
    }

    /// Same accept-loop as [`serve_until`](Self::serve_until), but takes an
    /// already-bound listener — lets tests learn the real ephemeral port before
    /// the loop starts, the same way [`accept_one`](Self::accept_one) does.
    /// Plaintext only, with the executor's own admission limits; use
    /// [`serve_listener_until`](Self::serve_listener_until) for TLS.
    pub async fn accept_loop_until(
        executor: Arc<SqlExecutor>,
        password: Option<String>,
        listener: TcpListener,
        addr: SocketAddr,
        shutdown: impl std::future::Future<Output = ()>,
    ) -> std::io::Result<()> {
        ConnPolicy {
            executor,
            credentials: Credentials::new(password.as_deref(), AuthMethod::default()),
            tls: None,
            require_tls: false,
            max_connections: None,
            live: Arc::new(AtomicUsize::new(0)),
        }
        .accept_until(listener, addr, shutdown)
        .await
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
            .with_auth_method(AuthMethod::Password)
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

    // ── B13: message framing ─────────────────────────────────────────────────

    /// Read whole backend messages until `n` ReadyForQuery ('Z') messages
    /// have arrived (or the server closes). Returns the message type bytes.
    async fn read_until_ready(stream: &mut tokio::net::TcpStream, n: usize) -> Vec<u8> {
        let mut raw = Vec::new();
        let mut types = Vec::new();
        let mut pos = 0;
        let mut buf = vec![0u8; 8192];
        while types.iter().filter(|&&t| t == b'Z').count() < n {
            // Consume every complete message already buffered.
            if raw.len() >= pos + 5 {
                let len =
                    i32::from_be_bytes([raw[pos + 1], raw[pos + 2], raw[pos + 3], raw[pos + 4]])
                        as usize;
                if raw.len() >= pos + 1 + len {
                    types.push(raw[pos]);
                    pos += 1 + len;
                    continue;
                }
            }
            let got =
                tokio::time::timeout(std::time::Duration::from_secs(10), stream.read(&mut buf))
                    .await
                    .expect("timed out waiting for server reply")
                    .unwrap_or(0);
            if got == 0 {
                break;
            }
            raw.extend_from_slice(&buf[..got]);
        }
        types
    }

    /// Start a no-auth server, complete startup and drain the greeting.
    async fn ready_session(dir: &std::path::Path) -> tokio::net::TcpStream {
        let (server, listener, addr) = open_server_on_ephemeral(dir).await;
        tokio::spawn(async move {
            let _ = server.accept_one(&listener).await;
        });
        let mut stream = connect_and_startup(addr).await;
        let greet = read_until_ready(&mut stream, 1).await;
        assert_eq!(greet.last(), Some(&b'Z'), "{greet:?}");
        stream
    }

    #[tokio::test]
    async fn test_two_queries_in_one_write_get_two_responses() {
        let dir = tempfile::tempdir().unwrap();
        let mut stream = ready_session(dir.path()).await;
        let mut both = make_query_msg("CREATE TABLE pipe (pk U64)");
        both.extend(make_query_msg("SELECT * FROM pipe"));
        stream.write_all(&both).await.unwrap();
        let types = read_until_ready(&mut stream, 2).await;
        // CommandComplete + Z, then RowDescription + CommandComplete + Z.
        assert_eq!(types, vec![b'C', b'Z', b'T', b'C', b'Z'], "{types:?}");
    }

    #[tokio::test]
    async fn test_query_split_byte_by_byte_gets_one_response() {
        let dir = tempfile::tempdir().unwrap();
        let mut stream = ready_session(dir.path()).await;
        stream.set_nodelay(true).unwrap();
        for byte in make_query_msg("CREATE TABLE split (pk U64)") {
            stream.write_all(&[byte]).await.unwrap();
            stream.flush().await.unwrap();
            tokio::task::yield_now().await;
        }
        let types = read_until_ready(&mut stream, 1).await;
        assert_eq!(types, vec![b'C', b'Z'], "{types:?}");
        // The session is still in sync for the next query.
        stream
            .write_all(&make_query_msg("SELECT * FROM split"))
            .await
            .unwrap();
        let types = read_until_ready(&mut stream, 1).await;
        assert_eq!(types, vec![b'T', b'C', b'Z'], "{types:?}");
    }

    #[tokio::test]
    async fn test_200kb_query_is_handled() {
        let dir = tempfile::tempdir().unwrap();
        let mut stream = ready_session(dir.path()).await;
        // > 64 KiB (the old single-read buffer) but under MAX_MESSAGE_LEN.
        let sql = format!("CREATE TABLE big (pk U64){}", " ".repeat(200 * 1024));
        stream.write_all(&make_query_msg(&sql)).await.unwrap();
        let types = read_until_ready(&mut stream, 1).await;
        assert_eq!(types, vec![b'C', b'Z'], "{types:?}");
        stream
            .write_all(&make_query_msg("SELECT * FROM big"))
            .await
            .unwrap();
        let types = read_until_ready(&mut stream, 1).await;
        assert_eq!(types, vec![b'T', b'C', b'Z'], "{types:?}");
    }

    #[tokio::test]
    async fn test_bad_message_length_rejected_with_08p01() {
        for len in [3i32, -1, i32::MIN, crate::wire::MAX_MESSAGE_LEN as i32 + 1] {
            let dir = tempfile::tempdir().unwrap();
            let mut stream = ready_session(dir.path()).await;
            let mut msg = vec![b'Q'];
            msg.extend_from_slice(&len.to_be_bytes());
            stream.write_all(&msg).await.unwrap();
            let mut reply = Vec::new();
            let _ = tokio::time::timeout(
                std::time::Duration::from_secs(10),
                stream.read_to_end(&mut reply),
            )
            .await
            .expect("server must close the connection");
            assert_eq!(reply.first(), Some(&b'E'), "len {len}: {reply:?}");
            let text = String::from_utf8_lossy(&reply);
            assert!(text.contains("08P01"), "len {len}: {text}");
        }
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
