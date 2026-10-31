//! SQL-over-TCP server (S35/S36, VV-NET-SQL-001..010).
//!
//! Protocol: newline-delimited JSON frames on a plain TCP *or* TLS stream.
//!
//! Request frame  (client → server):
//!   `{"sql":"SELECT * FROM t"}\n`
//!
//! Response frame (server → client):
//!   success: `{"ok":true,"type":"rows","columns":[...],"rows":[[...],...]}\n`
//!   success: `{"ok":true,"type":"affected","count":N}\n`
//!   success: `{"ok":true,"type":"ok"}\n`
//!   error:   `{"ok":false,"error":"..."}\n`
//!
//! Each TCP connection gets its own session executor (via
//! [`SqlExecutor::new_session`]), so transactions are per-connection — BEGIN
//! on one connection is invisible to another. Server-wide security state
//! (RLS policies, table tokens, read-only mode, masks, grants, views, ...)
//! is shared by all connections, so it cannot be reset by reconnecting.

use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream, ToSocketAddrs};
use std::sync::Arc;
use std::thread;

use serde_json::{Value, json};
use vectorvolt_engine::TableRegistry;

use crate::sql::executor::{QueryResult, SqlExecutor};

// ── SqlTcpServer ─────────────────────────────────────────────────────────────

/// A multi-threaded SQL-over-TCP server (plain and TLS).
///
/// Each accepted connection runs on its own OS thread with a session
/// executor derived from one shared root [`SqlExecutor`]: per-connection
/// transaction state, shared security/catalog state. The underlying
/// [`TableRegistry`] is shared too — it is already `Arc`-wrapped and
/// internally synchronised.
pub struct SqlTcpServer {
    executor: Arc<SqlExecutor>,
}

impl SqlTcpServer {
    pub fn new(registry: Arc<TableRegistry>) -> Self {
        Self::with_executor(Arc::new(SqlExecutor::new(registry)))
    }

    /// Serve connections from sessions of an existing executor, sharing its
    /// security state (e.g. one configured with a column-encryption key).
    pub fn with_executor(executor: Arc<SqlExecutor>) -> Self {
        Self { executor }
    }

    /// The shared root executor whose state every connection sees.
    pub fn executor(&self) -> &Arc<SqlExecutor> {
        &self.executor
    }

    /// Bind to `addr` and start accepting plain-TCP connections.
    ///
    /// Returns the [`TcpListener`] so the caller can read `.local_addr()`
    /// (useful for ephemeral-port tests).
    pub fn listen<A: ToSocketAddrs>(&self, addr: A) -> std::io::Result<TcpListener> {
        let listener = TcpListener::bind(addr)?;
        let root = Arc::clone(&self.executor);
        let listener_clone = listener.try_clone()?;
        thread::spawn(move || {
            for stream in listener_clone.incoming() {
                match stream {
                    Ok(s) => {
                        let root = Arc::clone(&root);
                        thread::spawn(move || handle_plain(s, root));
                    }
                    Err(_) => break,
                }
            }
        });
        Ok(listener)
    }

    /// Bind to `addr` and start accepting TLS connections using `tls_config`.
    pub fn listen_tls<A: ToSocketAddrs>(
        &self,
        addr: A,
        tls_config: Arc<rustls::ServerConfig>,
    ) -> std::io::Result<TcpListener> {
        let listener = TcpListener::bind(addr)?;
        let root = Arc::clone(&self.executor);
        let listener_clone = listener.try_clone()?;
        thread::spawn(move || {
            for stream in listener_clone.incoming() {
                match stream {
                    Ok(s) => {
                        let root = Arc::clone(&root);
                        let cfg = Arc::clone(&tls_config);
                        thread::spawn(move || handle_tls(s, root, cfg));
                    }
                    Err(_) => break,
                }
            }
        });
        Ok(listener)
    }
}

// ── Connection handlers ───────────────────────────────────────────────────────

fn handle_plain(stream: TcpStream, root: Arc<SqlExecutor>) {
    let executor = root.new_session();
    let mut reader = BufReader::new(stream);
    loop {
        let mut line = String::new();
        match reader.read_line(&mut line) {
            Ok(0) => break,
            Ok(_) => {}
            Err(_) => break,
        }
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let response = handle_request(&executor, trimmed);
        let mut out = response.to_string();
        out.push('\n');
        if reader.get_mut().write_all(out.as_bytes()).is_err() {
            break;
        }
    }
}

fn handle_tls(stream: TcpStream, root: Arc<SqlExecutor>, tls_config: Arc<rustls::ServerConfig>) {
    let conn = match rustls::ServerConnection::new(tls_config) {
        Ok(c) => c,
        Err(_) => return,
    };
    let executor = root.new_session();
    let tls_stream = rustls::StreamOwned::new(conn, stream);
    let mut reader = BufReader::new(tls_stream);
    loop {
        let mut line = String::new();
        match reader.read_line(&mut line) {
            Ok(0) => break,
            Ok(_) => {}
            Err(_) => break,
        }
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let response = handle_request(&executor, trimmed);
        let mut out = response.to_string();
        out.push('\n');
        if reader.get_mut().write_all(out.as_bytes()).is_err() {
            break;
        }
    }
}

fn handle_request(executor: &SqlExecutor, raw: &str) -> Value {
    let sql = match serde_json::from_str::<Value>(raw) {
        Ok(v) => match v.get("sql").and_then(|s| s.as_str()) {
            Some(s) => s.to_owned(),
            None => {
                return json!({"ok": false, "error": "missing 'sql' field"});
            }
        },
        Err(e) => {
            return json!({"ok": false, "error": format!("json parse error: {e}")});
        }
    };

    match executor.execute(&sql) {
        Ok(result) => encode_result(result),
        Err(e) => json!({"ok": false, "error": e.to_string()}),
    }
}

fn encode_result(result: QueryResult) -> Value {
    match result {
        QueryResult::Rows { columns, rows } => {
            json!({"ok": true, "type": "rows", "columns": columns, "rows": rows})
        }
        QueryResult::Affected(n) => {
            json!({"ok": true, "type": "affected", "count": n})
        }
        QueryResult::Created => json!({"ok": true, "type": "ok", "op": "created"}),
        QueryResult::Dropped => json!({"ok": true, "type": "ok", "op": "dropped"}),
        QueryResult::ExplainPlan(lines) => {
            json!({"ok": true, "type": "explain", "plan": lines})
        }
        QueryResult::TxBegin => json!({"ok": true, "type": "ok", "op": "begin"}),
        QueryResult::TxCommit => json!({"ok": true, "type": "ok", "op": "commit"}),
        QueryResult::TxRollback => json!({"ok": true, "type": "ok", "op": "rollback"}),
        QueryResult::Error(msg) => json!({"ok": false, "error": msg}),
        QueryResult::Notice(msg) => json!({"ok": true, "notice": msg}),
        QueryResult::WithNotice { result, notice } => {
            let inner = encode_result(*result);
            json!({"ok": true, "notice": notice, "result": inner})
        }
        QueryResult::Ok(msg) => json!({"ok": true, "message": msg}),
    }
}

// ── SqlSession (plain-TCP client for tests) ───────────────────────────────────

/// Thin synchronous client for talking to a plain-TCP [`SqlTcpServer`].
pub struct SqlSession {
    writer: TcpStream,
    reader: BufReader<TcpStream>,
}

impl SqlSession {
    pub fn connect<A: ToSocketAddrs>(addr: A) -> std::io::Result<Self> {
        let stream = TcpStream::connect(addr)?;
        stream.set_read_timeout(Some(std::time::Duration::from_secs(10)))?;
        let reader = BufReader::new(stream.try_clone()?);
        Ok(Self {
            writer: stream,
            reader,
        })
    }

    /// Send one SQL statement and return the parsed JSON response.
    pub fn query(&mut self, sql: &str) -> serde_json::Result<Value> {
        let req = format!("{}\n", json!({"sql": sql}));
        self.writer.write_all(req.as_bytes()).unwrap();
        let mut line = String::new();
        self.reader.read_line(&mut line).unwrap();
        serde_json::from_str(line.trim())
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    use tempfile::TempDir;
    use vectorvolt_engine::EngineOptions;

    // ── plain-TCP helpers (S35) ───────────────────────────────────────────────

    fn start_server() -> (SqlTcpServer, TcpListener, TempDir) {
        let dir = TempDir::new().unwrap();
        let registry = Arc::new(TableRegistry::open(dir.path(), EngineOptions::default()).unwrap());
        let server = SqlTcpServer::new(Arc::clone(&registry));
        let listener = server.listen("127.0.0.1:0").unwrap();
        (server, listener, dir)
    }

    fn session(listener: &TcpListener) -> SqlSession {
        let addr = listener.local_addr().unwrap();
        for _ in 0..20 {
            if let Ok(s) = SqlSession::connect(addr) {
                return s;
            }
            std::thread::sleep(Duration::from_millis(5));
        }
        SqlSession::connect(addr).unwrap()
    }

    #[test]
    fn test_s35_ddl_over_tcp() {
        let (_srv, listener, _dir) = start_server();
        let mut s = session(&listener);
        let r = s.query("CREATE TABLE t (pk U64, v U64)").unwrap();
        assert_eq!(r["ok"], true);
        assert_eq!(r["op"], "created");
    }

    #[test]
    fn test_s35_insert_select_over_tcp() {
        let (_srv, listener, _dir) = start_server();
        let mut s = session(&listener);
        s.query("CREATE TABLE items (pk U64, price U64)").unwrap();
        s.query("INSERT INTO items (pk, price) VALUES (1, 500)")
            .unwrap();
        s.query("INSERT INTO items (pk, price) VALUES (2, 800)")
            .unwrap();
        let r = s.query("SELECT pk, price FROM items WHERE pk = 1").unwrap();
        assert_eq!(r["ok"], true);
        assert_eq!(r["type"], "rows");
        assert_eq!(r["rows"][0][1], "500");
    }

    #[test]
    fn test_s35_txn_commit_visible() {
        let (_srv, listener, _dir) = start_server();
        let mut s = session(&listener);
        s.query("CREATE TABLE acct (pk U64, bal U64)").unwrap();
        s.query("BEGIN").unwrap();
        s.query("INSERT INTO acct (pk, bal) VALUES (1, 1000)")
            .unwrap();
        let commit_r = s.query("COMMIT").unwrap();
        assert_eq!(commit_r["op"], "commit");
        let r = s.query("SELECT pk FROM acct WHERE pk = 1").unwrap();
        assert_eq!(r["rows"].as_array().unwrap().len(), 1);
    }

    #[test]
    fn test_s35_txn_rollback_not_visible() {
        let (_srv, listener, _dir) = start_server();
        let mut s = session(&listener);
        s.query("CREATE TABLE ledger (pk U64, amount U64)").unwrap();
        s.query("BEGIN").unwrap();
        s.query("INSERT INTO ledger (pk, amount) VALUES (42, 999)")
            .unwrap();
        let rb_r = s.query("ROLLBACK").unwrap();
        assert_eq!(rb_r["op"], "rollback");
        let r = s.query("SELECT pk FROM ledger WHERE pk = 42").unwrap();
        assert_eq!(r["rows"].as_array().unwrap().len(), 0);
    }

    #[test]
    fn test_s35_two_connections_isolated() {
        let (_srv, listener, _dir) = start_server();
        let mut s1 = session(&listener);
        let mut s2 = session(&listener);
        s1.query("CREATE TABLE shared (pk U64, v U64)").unwrap();
        s1.query("BEGIN").unwrap();
        s1.query("INSERT INTO shared (pk, v) VALUES (7, 77)")
            .unwrap();
        let r2 = s2.query("SELECT pk FROM shared WHERE pk = 7").unwrap();
        assert_eq!(r2["rows"].as_array().unwrap().len(), 0);
        s1.query("COMMIT").unwrap();
        let r2_after = s2.query("SELECT pk FROM shared WHERE pk = 7").unwrap();
        assert_eq!(r2_after["rows"].as_array().unwrap().len(), 1);
    }

    // A4 — security state is server-wide: a restriction set on one
    // connection still applies to other (and later) connections.
    #[test]
    fn test_a4_read_only_survives_reconnect() {
        let (_srv, listener, _dir) = start_server();
        let mut s1 = session(&listener);
        s1.query("CREATE TABLE ro (pk U64, v U64)").unwrap();
        s1.query("SET READ ONLY").unwrap();
        drop(s1);
        let mut s2 = session(&listener);
        let r = s2.query("INSERT INTO ro (pk, v) VALUES (1, 1)").unwrap();
        assert_eq!(r["ok"], false, "{r}");
        assert!(r["error"].as_str().unwrap().contains("25006"));
    }

    #[test]
    fn test_a4_policy_applies_to_other_connections() {
        let (_srv, listener, _dir) = start_server();
        let mut s1 = session(&listener);
        s1.query("CREATE TABLE rls (pk U64, v U64)").unwrap();
        s1.query("INSERT INTO rls (pk, v) VALUES (1, 10)").unwrap();
        s1.query("INSERT INTO rls (pk, v) VALUES (2, 0)").unwrap();
        s1.query("CREATE POLICY p ON rls USING (v > 5)").unwrap();
        let mut s2 = session(&listener);
        let r = s2.query("SELECT * FROM rls").unwrap();
        assert_eq!(r["rows"].as_array().unwrap().len(), 1, "{r}");
        let mut s3 = session(&listener);
        let r = s3.query("REQUIRE TOKEN 'tk' ON rls").unwrap();
        assert_eq!(r["ok"], true, "{r}");
        let r = s2.query("SELECT * FROM rls").unwrap();
        assert_eq!(r["ok"], false, "{r}");
    }

    // ── TLS helpers (S36) ─────────────────────────────────────────────────────

    use rcgen::generate_simple_self_signed;
    use rustls::pki_types::{CertificateDer, PrivateKeyDer};

    fn make_tls_server_config() -> Arc<rustls::ServerConfig> {
        let cert = generate_simple_self_signed(vec!["localhost".to_owned()]).unwrap();
        let cert_der = CertificateDer::from(cert.cert.der().to_vec());
        let key_der = PrivateKeyDer::try_from(cert.key_pair.serialize_der()).unwrap();
        let cfg = rustls::ServerConfig::builder()
            .with_no_client_auth()
            .with_single_cert(vec![cert_der], key_der)
            .unwrap();
        Arc::new(cfg)
    }

    struct TlsSession {
        inner: BufReader<rustls::StreamOwned<rustls::ClientConnection, TcpStream>>,
    }

    impl TlsSession {
        fn connect<A: ToSocketAddrs>(addr: A) -> std::io::Result<Self> {
            let tls_cfg = Arc::new(
                rustls::ClientConfig::builder()
                    .dangerous()
                    .with_custom_certificate_verifier(Arc::new(NoVerifier))
                    .with_no_client_auth(),
            );
            let server_name = rustls::pki_types::ServerName::try_from("localhost")
                .unwrap()
                .to_owned();
            let conn = rustls::ClientConnection::new(tls_cfg, server_name).unwrap();
            let stream = TcpStream::connect(addr)?;
            stream.set_read_timeout(Some(std::time::Duration::from_secs(10)))?;
            let tls = rustls::StreamOwned::new(conn, stream);
            Ok(Self {
                inner: BufReader::new(tls),
            })
        }

        fn query(&mut self, sql: &str) -> serde_json::Result<Value> {
            let req = format!("{}\n", json!({"sql": sql}));
            self.inner.get_mut().write_all(req.as_bytes()).unwrap();
            let mut line = String::new();
            self.inner.read_line(&mut line).unwrap();
            serde_json::from_str(line.trim())
        }
    }

    // Certificate verifier that accepts everything (tests only).
    #[derive(Debug)]
    struct NoVerifier;

    impl rustls::client::danger::ServerCertVerifier for NoVerifier {
        fn verify_server_cert(
            &self,
            _end_entity: &rustls::pki_types::CertificateDer<'_>,
            _intermediates: &[rustls::pki_types::CertificateDer<'_>],
            _server_name: &rustls::pki_types::ServerName<'_>,
            _ocsp_response: &[u8],
            _now: rustls::pki_types::UnixTime,
        ) -> Result<rustls::client::danger::ServerCertVerified, rustls::Error> {
            Ok(rustls::client::danger::ServerCertVerified::assertion())
        }

        fn verify_tls12_signature(
            &self,
            _message: &[u8],
            _cert: &rustls::pki_types::CertificateDer<'_>,
            _dss: &rustls::DigitallySignedStruct,
        ) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
            Ok(rustls::client::danger::HandshakeSignatureValid::assertion())
        }

        fn verify_tls13_signature(
            &self,
            _message: &[u8],
            _cert: &rustls::pki_types::CertificateDer<'_>,
            _dss: &rustls::DigitallySignedStruct,
        ) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
            Ok(rustls::client::danger::HandshakeSignatureValid::assertion())
        }

        fn supported_verify_schemes(&self) -> Vec<rustls::SignatureScheme> {
            rustls::crypto::ring::default_provider()
                .signature_verification_algorithms
                .supported_schemes()
        }
    }

    fn start_tls_server() -> (
        SqlTcpServer,
        TcpListener,
        TempDir,
        Arc<rustls::ServerConfig>,
    ) {
        let dir = TempDir::new().unwrap();
        let registry = Arc::new(TableRegistry::open(dir.path(), EngineOptions::default()).unwrap());
        let server = SqlTcpServer::new(Arc::clone(&registry));
        let tls_cfg = make_tls_server_config();
        let listener = server
            .listen_tls("127.0.0.1:0", Arc::clone(&tls_cfg))
            .unwrap();
        (server, listener, dir, tls_cfg)
    }

    fn tls_session(listener: &TcpListener) -> TlsSession {
        let addr = listener.local_addr().unwrap();
        for _ in 0..20 {
            if let Ok(s) = TlsSession::connect(addr) {
                return s;
            }
            std::thread::sleep(Duration::from_millis(10));
        }
        TlsSession::connect(addr).unwrap()
    }

    #[test]
    fn test_s36_tls_ddl() {
        let (_srv, listener, _dir, _cfg) = start_tls_server();
        let mut s = tls_session(&listener);
        let r = s.query("CREATE TABLE tls_t (pk U64, v U64)").unwrap();
        assert_eq!(r["ok"], true);
        assert_eq!(r["op"], "created");
    }

    #[test]
    fn test_s36_tls_insert_select() {
        let (_srv, listener, _dir, _cfg) = start_tls_server();
        let mut s = tls_session(&listener);
        s.query("CREATE TABLE tls_items (pk U64, price U64)")
            .unwrap();
        s.query("INSERT INTO tls_items (pk, price) VALUES (1, 999)")
            .unwrap();
        let r = s
            .query("SELECT pk, price FROM tls_items WHERE pk = 1")
            .unwrap();
        assert_eq!(r["ok"], true);
        assert_eq!(r["rows"][0][1], "999");
    }

    #[test]
    fn test_s36_tls_txn_commit() {
        let (_srv, listener, _dir, _cfg) = start_tls_server();
        let mut s = tls_session(&listener);
        s.query("CREATE TABLE tls_acct (pk U64, bal U64)").unwrap();
        s.query("BEGIN").unwrap();
        s.query("INSERT INTO tls_acct (pk, bal) VALUES (5, 2000)")
            .unwrap();
        let r = s.query("COMMIT").unwrap();
        assert_eq!(r["op"], "commit");
        let r2 = s.query("SELECT pk FROM tls_acct WHERE pk = 5").unwrap();
        assert_eq!(r2["rows"].as_array().unwrap().len(), 1);
    }

    #[test]
    fn test_s36_tls_txn_rollback() {
        let (_srv, listener, _dir, _cfg) = start_tls_server();
        let mut s = tls_session(&listener);
        s.query("CREATE TABLE tls_rb (pk U64, v U64)").unwrap();
        s.query("BEGIN").unwrap();
        s.query("INSERT INTO tls_rb (pk, v) VALUES (99, 1)")
            .unwrap();
        let r = s.query("ROLLBACK").unwrap();
        assert_eq!(r["op"], "rollback");
        let r2 = s.query("SELECT pk FROM tls_rb WHERE pk = 99").unwrap();
        assert_eq!(r2["rows"].as_array().unwrap().len(), 0);
    }

    #[test]
    fn test_s36_tls_error_response() {
        let (_srv, listener, _dir, _cfg) = start_tls_server();
        let mut s = tls_session(&listener);
        let r = s.query("SELECT * FROM nonexistent_table").unwrap();
        assert_eq!(r["ok"], false);
        assert!(r["error"].as_str().is_some());
    }
}
