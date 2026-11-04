//! pgwire TLS (SSLRequest negotiation, optional mTLS) and accept-time
//! admission control: the S83 IP allowlist and the S116 connection limit.

use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use rustls::pki_types::{CertificateDer, PrivateKeyDer, PrivatePkcs8KeyDer, ServerName};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio_rustls::TlsConnector;
use vectorvolt_reader::server::{PgConfig, PgServer};

const SSL_REQUEST: [u8; 8] = [0, 0, 0, 8, 0x04, 0xd2, 0x16, 0x2f]; // 80877103

fn provider() -> Arc<rustls::crypto::CryptoProvider> {
    Arc::new(rustls::crypto::ring::default_provider())
}

struct Pki {
    server_cert: CertificateDer<'static>,
    server_key: PrivateKeyDer<'static>,
    ca_cert: CertificateDer<'static>,
    client_cert: CertificateDer<'static>,
    client_key: PrivateKeyDer<'static>,
}

fn pki() -> Pki {
    let server =
        rcgen::generate_simple_self_signed(vec!["localhost".to_string(), "127.0.0.1".to_string()])
            .unwrap();
    let ca_key = rcgen::KeyPair::generate().unwrap();
    let mut ca_params = rcgen::CertificateParams::new(Vec::<String>::new()).unwrap();
    ca_params.is_ca = rcgen::IsCa::Ca(rcgen::BasicConstraints::Unconstrained);
    ca_params
        .distinguished_name
        .push(rcgen::DnType::CommonName, "pgwire test client CA");
    let ca = ca_params.self_signed(&ca_key).unwrap();
    let client_key = rcgen::KeyPair::generate().unwrap();
    let mut client_params = rcgen::CertificateParams::new(vec!["client".to_string()]).unwrap();
    client_params.extended_key_usages = vec![rcgen::ExtendedKeyUsagePurpose::ClientAuth];
    let client = client_params.signed_by(&client_key, &ca, &ca_key).unwrap();
    Pki {
        server_cert: server.cert.der().clone(),
        server_key: PrivateKeyDer::Pkcs8(PrivatePkcs8KeyDer::from(server.key_pair.serialize_der())),
        ca_cert: ca.der().clone(),
        client_cert: client.der().clone(),
        client_key: PrivateKeyDer::Pkcs8(PrivatePkcs8KeyDer::from(client_key.serialize_der())),
    }
}

fn server_tls(p: &Pki, mtls: bool) -> Arc<rustls::ServerConfig> {
    let b = rustls::ServerConfig::builder_with_provider(provider())
        .with_safe_default_protocol_versions()
        .unwrap();
    let b = if mtls {
        let mut roots = rustls::RootCertStore::empty();
        roots.add(p.ca_cert.clone()).unwrap();
        let v = rustls::server::WebPkiClientVerifier::builder_with_provider(
            Arc::new(roots),
            provider(),
        )
        .build()
        .unwrap();
        b.with_client_cert_verifier(v)
    } else {
        b.with_no_client_auth()
    };
    Arc::new(
        b.with_single_cert(vec![p.server_cert.clone()], p.server_key.clone_key())
            .unwrap(),
    )
}

fn client_tls(p: &Pki, with_cert: bool) -> TlsConnector {
    let mut roots = rustls::RootCertStore::empty();
    roots.add(p.server_cert.clone()).unwrap();
    let b = rustls::ClientConfig::builder_with_provider(provider())
        .with_safe_default_protocol_versions()
        .unwrap()
        .with_root_certificates(roots);
    let cfg = if with_cert {
        b.with_client_auth_cert(vec![p.client_cert.clone()], p.client_key.clone_key())
            .unwrap()
    } else {
        b.with_no_client_auth()
    };
    TlsConnector::from(Arc::new(cfg))
}

/// Start `server` on an ephemeral port; it runs until the test ends.
async fn start(server: PgServer) -> SocketAddr {
    let (listener, addr) = server.bind().await.unwrap();
    tokio::spawn(server.serve_listener_until(listener, addr, std::future::pending()));
    addr
}

fn new_server(dir: &tempfile::TempDir) -> PgServer {
    let config = PgConfig::new("127.0.0.1:0".parse().unwrap(), dir.path().to_path_buf());
    PgServer::new(config).unwrap()
}

fn startup_msg() -> Vec<u8> {
    let params = b"user\0postgres\0database\0vectorvolt\0\0";
    let len = (8 + params.len()) as i32;
    let mut buf = len.to_be_bytes().to_vec();
    buf.extend_from_slice(&(3i32 << 16).to_be_bytes());
    buf.extend_from_slice(params);
    buf
}

/// Read one backend message (type byte + body).
async fn read_msg<S: AsyncRead + Unpin>(s: &mut S) -> std::io::Result<(u8, Vec<u8>)> {
    let mut hdr = [0u8; 5];
    s.read_exact(&mut hdr).await?;
    let len = i32::from_be_bytes([hdr[1], hdr[2], hdr[3], hdr[4]]) as usize;
    let mut body = vec![0u8; len - 4];
    s.read_exact(&mut body).await?;
    Ok((hdr[0], body))
}

/// The SQLSTATE ('C' field) of an ErrorResponse body.
fn sqlstate(body: &[u8]) -> String {
    let mut i = 0;
    while i < body.len() && body[i] != 0 {
        let field = body[i];
        let end = i + 1 + body[i + 1..].iter().position(|&b| b == 0).unwrap();
        if field == b'C' {
            return String::from_utf8_lossy(&body[i + 1..end]).into_owned();
        }
        i = end + 1;
    }
    String::new()
}

/// Send a startup message and read until ReadyForQuery; panics on 'E'.
async fn expect_ready<S: AsyncRead + AsyncWrite + Unpin>(s: &mut S) {
    s.write_all(&startup_msg()).await.unwrap();
    s.flush().await.unwrap();
    let (t, _) = read_msg(s).await.unwrap();
    assert_eq!(t, b'R', "expected AuthenticationOk");
    loop {
        let (t, body) = read_msg(s).await.unwrap();
        assert_ne!(t, b'E', "error {}", sqlstate(&body));
        if t == b'Z' {
            return;
        }
    }
}

/// The SQLSTATE of the first message, which must be an ErrorResponse.
async fn expect_error<S: AsyncRead + Unpin>(s: &mut S) -> String {
    let (t, body) = tokio::time::timeout(Duration::from_secs(5), read_msg(s))
        .await
        .expect("timed out waiting for the server")
        .unwrap();
    assert_eq!(t, b'E', "expected an ErrorResponse, got {:?}", t as char);
    sqlstate(&body)
}

async fn ssl_request(addr: SocketAddr) -> (TcpStream, u8) {
    let mut s = TcpStream::connect(addr).await.unwrap();
    s.write_all(&SSL_REQUEST).await.unwrap();
    let mut answer = [0u8; 1];
    s.read_exact(&mut answer).await.unwrap();
    (s, answer[0])
}

// ── TLS ──────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn ssl_request_is_answered_n_without_tls_and_plaintext_continues() {
    let dir = tempfile::tempdir().unwrap();
    let addr = start(new_server(&dir)).await;
    let (mut s, answer) = ssl_request(addr).await;
    assert_eq!(answer, b'N');
    expect_ready(&mut s).await;
}

#[tokio::test]
async fn ssl_request_is_answered_s_and_the_session_runs_over_tls() {
    let dir = tempfile::tempdir().unwrap();
    let p = pki();
    let addr = start(new_server(&dir).with_tls(Some(server_tls(&p, false)))).await;
    let (s, answer) = ssl_request(addr).await;
    assert_eq!(answer, b'S');
    let name = ServerName::try_from("localhost").unwrap();
    let mut tls = client_tls(&p, false).connect(name, s).await.unwrap();
    expect_ready(&mut tls).await;

    // A real query round-trips over the encrypted session.
    let sql = b"SELECT 1\0";
    let mut q = vec![b'Q'];
    q.extend_from_slice(&((4 + sql.len()) as i32).to_be_bytes());
    q.extend_from_slice(sql);
    tls.write_all(&q).await.unwrap();
    tls.flush().await.unwrap();
    loop {
        let (t, _) = read_msg(&mut tls).await.unwrap();
        if t == b'Z' {
            break;
        }
    }
}

#[tokio::test]
async fn plaintext_startup_still_works_when_tls_is_optional() {
    let dir = tempfile::tempdir().unwrap();
    let p = pki();
    let addr = start(new_server(&dir).with_tls(Some(server_tls(&p, false)))).await;
    let mut s = TcpStream::connect(addr).await.unwrap();
    expect_ready(&mut s).await;
}

#[tokio::test]
async fn require_tls_refuses_a_plaintext_startup() {
    let dir = tempfile::tempdir().unwrap();
    let p = pki();
    let server = new_server(&dir)
        .with_tls(Some(server_tls(&p, false)))
        .with_require_tls(true);
    let addr = start(server).await;
    let mut s = TcpStream::connect(addr).await.unwrap();
    s.write_all(&startup_msg()).await.unwrap();
    assert_eq!(expect_error(&mut s).await, "28000");
}

#[tokio::test]
async fn mtls_rejects_a_client_without_a_certificate() {
    let dir = tempfile::tempdir().unwrap();
    let p = pki();
    let addr = start(new_server(&dir).with_tls(Some(server_tls(&p, true)))).await;
    let (s, answer) = ssl_request(addr).await;
    assert_eq!(answer, b'S');
    let name = ServerName::try_from("localhost").unwrap();
    // TLS 1.3: the client side may complete before the server rejects the
    // missing certificate; then the rejection surfaces on the first read.
    let served = match client_tls(&p, false).connect(name, s).await {
        Err(_) => false,
        Ok(mut tls) => {
            let _ = tls.write_all(&startup_msg()).await;
            let _ = tls.flush().await;
            matches!(
                tokio::time::timeout(Duration::from_secs(5), read_msg(&mut tls)).await,
                Ok(Ok((b'R', _)))
            )
        }
    };
    assert!(!served, "a client without a certificate got a session");
}

#[tokio::test]
async fn mtls_accepts_a_client_with_a_ca_issued_certificate() {
    let dir = tempfile::tempdir().unwrap();
    let p = pki();
    let addr = start(new_server(&dir).with_tls(Some(server_tls(&p, true)))).await;
    let (s, answer) = ssl_request(addr).await;
    assert_eq!(answer, b'S');
    let name = ServerName::try_from("localhost").unwrap();
    let mut tls = client_tls(&p, true).connect(name, s).await.unwrap();
    expect_ready(&mut tls).await;
}

// ── Admission (S83 / S116) ───────────────────────────────────────────────────

#[tokio::test]
async fn a_client_ip_outside_the_allowlist_is_rejected_with_28000() {
    let dir = tempfile::tempdir().unwrap();
    let server = new_server(&dir);
    server
        .executor()
        .execute("ADD ALLOWED IP '10.9.8.7'")
        .unwrap();
    let addr = start(server).await;
    let mut s = TcpStream::connect(addr).await.unwrap();
    s.write_all(&startup_msg()).await.unwrap();
    assert_eq!(expect_error(&mut s).await, "28000");
}

#[tokio::test]
async fn an_allowlisted_client_ip_is_admitted() {
    let dir = tempfile::tempdir().unwrap();
    let server = new_server(&dir);
    server
        .executor()
        .execute("ADD ALLOWED IP '127.0.0.1'")
        .unwrap();
    let addr = start(server).await;
    let mut s = TcpStream::connect(addr).await.unwrap();
    expect_ready(&mut s).await;
}

#[tokio::test]
async fn the_allowlist_set_over_a_pgwire_session_applies_to_new_connections() {
    let dir = tempfile::tempdir().unwrap();
    let addr = start(new_server(&dir)).await;
    let mut admin = TcpStream::connect(addr).await.unwrap();
    expect_ready(&mut admin).await;
    let sql = b"ADD ALLOWED IP '10.9.8.7'\0";
    let mut q = vec![b'Q'];
    q.extend_from_slice(&((4 + sql.len()) as i32).to_be_bytes());
    q.extend_from_slice(sql);
    admin.write_all(&q).await.unwrap();
    loop {
        let (t, _) = read_msg(&mut admin).await.unwrap();
        if t == b'Z' {
            break;
        }
    }
    let mut s = TcpStream::connect(addr).await.unwrap();
    s.write_all(&startup_msg()).await.unwrap();
    assert_eq!(expect_error(&mut s).await, "28000");
}

#[tokio::test]
async fn connections_over_the_configured_limit_are_rejected_with_53300() {
    let dir = tempfile::tempdir().unwrap();
    let server = new_server(&dir).with_max_connections(Some(1));
    let addr = start(server).await;
    let mut first = TcpStream::connect(addr).await.unwrap();
    expect_ready(&mut first).await;

    let mut second = TcpStream::connect(addr).await.unwrap();
    second.write_all(&startup_msg()).await.unwrap();
    assert_eq!(expect_error(&mut second).await, "53300");

    // Closing the first frees the slot.
    drop(first);
    let mut admitted = false;
    for _ in 0..50 {
        tokio::time::sleep(Duration::from_millis(20)).await;
        let mut s = TcpStream::connect(addr).await.unwrap();
        s.write_all(&startup_msg()).await.unwrap();
        if let Ok((b'R', _)) = read_msg(&mut s).await {
            admitted = true;
            break;
        }
    }
    assert!(admitted, "the slot was never released");
}

#[tokio::test]
async fn the_executor_max_connections_setting_is_enforced() {
    let dir = tempfile::tempdir().unwrap();
    let server = new_server(&dir);
    server.executor().execute("SET MAX_CONNECTIONS 1").unwrap();
    let addr = start(server).await;
    let mut first = TcpStream::connect(addr).await.unwrap();
    expect_ready(&mut first).await;
    let mut second = TcpStream::connect(addr).await.unwrap();
    second.write_all(&startup_msg()).await.unwrap();
    assert_eq!(expect_error(&mut second).await, "53300");
}

#[tokio::test]
async fn the_limit_rejection_is_readable_in_reply_to_an_ssl_request() {
    // libpq sends SSLRequest first; the rejection must arrive as an
    // ErrorResponse it can report, not a bare close.
    let dir = tempfile::tempdir().unwrap();
    let p = pki();
    let server = new_server(&dir)
        .with_tls(Some(server_tls(&p, false)))
        .with_max_connections(Some(0));
    let addr = start(server).await;
    let mut s = TcpStream::connect(addr).await.unwrap();
    s.write_all(&SSL_REQUEST).await.unwrap();
    assert_eq!(expect_error(&mut s).await, "53300");
}
