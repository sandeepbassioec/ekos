//! D1 / D3 / D4 — `vvctl serve` TLS flags shared by every listener,
//! `--pgwire-host`, and the plaintext-exposure warning.

use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;
use std::time::Duration;

use clap::Parser;
use rustls::pki_types::{CertificateDer, ServerName};
use tempfile::TempDir;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use vectorvolt_net::{Client, ClientTlsConfig};
use vectorvolt_reader::server::PgConfig;
use vvctl::commands::serve::{
    ServeArgs, bind, build_pg_server, load_tls, pgwire_addr, plaintext_exposure_warning,
};

#[derive(Parser)]
struct Cli {
    #[command(flatten)]
    serve: ServeArgs,
}

fn parse(extra: &[&str]) -> Result<ServeArgs, clap::Error> {
    let mut argv = vec!["vvctl", "./data"];
    argv.extend_from_slice(extra);
    Cli::try_parse_from(argv).map(|c| c.serve)
}

struct Pki {
    dir: TempDir,
    server_cert_pem: String,
    server_cert_der: CertificateDer<'static>,
    client_cert_pem: String,
    client_key_pem: String,
}

impl Pki {
    fn path(&self, name: &str) -> String {
        self.dir.path().join(name).to_string_lossy().into_owned()
    }
}

/// Server cert/key, a client CA and a client certificate, written to files.
fn pki() -> Pki {
    let dir = TempDir::new().unwrap();
    let server = rcgen::generate_simple_self_signed(vec!["localhost".to_string()]).unwrap();
    let ca_key = rcgen::KeyPair::generate().unwrap();
    let mut ca_params = rcgen::CertificateParams::new(Vec::<String>::new()).unwrap();
    ca_params.is_ca = rcgen::IsCa::Ca(rcgen::BasicConstraints::Unconstrained);
    ca_params
        .distinguished_name
        .push(rcgen::DnType::CommonName, "vvctl test client CA");
    let ca = ca_params.self_signed(&ca_key).unwrap();
    let client_key = rcgen::KeyPair::generate().unwrap();
    let mut client_params = rcgen::CertificateParams::new(vec!["client".to_string()]).unwrap();
    client_params.extended_key_usages = vec![rcgen::ExtendedKeyUsagePurpose::ClientAuth];
    let client = client_params.signed_by(&client_key, &ca, &ca_key).unwrap();

    std::fs::write(dir.path().join("server.pem"), server.cert.pem()).unwrap();
    std::fs::write(
        dir.path().join("server.key"),
        server.key_pair.serialize_pem(),
    )
    .unwrap();
    std::fs::write(dir.path().join("ca.pem"), ca.pem()).unwrap();
    Pki {
        server_cert_pem: server.cert.pem(),
        server_cert_der: server.cert.der().clone(),
        client_cert_pem: client.pem(),
        client_key_pem: client_key.serialize_pem(),
        dir,
    }
}

// ── flags ────────────────────────────────────────────────────────────────────

#[test]
fn pgwire_host_defaults_to_loopback_and_parses() {
    let a = parse(&["--pgwire-port", "5433"]).unwrap();
    assert_eq!(a.listener.pgwire_host, IpAddr::from([127, 0, 0, 1]));
    assert_eq!(
        pgwire_addr(&a, 5433),
        "127.0.0.1:5433".parse::<SocketAddr>().unwrap()
    );

    let a = parse(&["--pgwire-port", "5433", "--pgwire-host", "0.0.0.0"]).unwrap();
    assert_eq!(
        pgwire_addr(&a, 5433),
        "0.0.0.0:5433".parse::<SocketAddr>().unwrap()
    );
    let a = parse(&["--pgwire-host", "::1"]).unwrap();
    assert_eq!(a.listener.pgwire_host, "::1".parse::<IpAddr>().unwrap());
    assert!(parse(&["--pgwire-host", "not-an-ip"]).is_err());
}

#[test]
fn tls_flags_parse_with_old_aliases() {
    let a = parse(&[
        "--tls-cert",
        "s.pem",
        "--tls-key",
        "s.key",
        "--tls-client-ca",
        "ca.pem",
        "--pgwire-require-tls",
        "--pgwire-max-connections",
        "7",
    ])
    .unwrap();
    assert_eq!(a.cert.as_deref(), Some(std::path::Path::new("s.pem")));
    assert_eq!(a.key.as_deref(), Some(std::path::Path::new("s.key")));
    assert_eq!(
        a.listener.tls_client_ca.as_deref(),
        Some(std::path::Path::new("ca.pem"))
    );
    assert!(a.listener.pgwire_require_tls);
    assert_eq!(a.listener.pgwire_max_connections, Some(7));

    // The pre-D1 spellings keep working.
    let a = parse(&["--cert", "s.pem", "--key", "s.key"]).unwrap();
    assert!(a.cert.is_some() && a.key.is_some());
}

#[test]
fn tls_flags_that_need_a_certificate_are_rejected_without_one() {
    assert!(parse(&["--tls-cert", "s.pem"]).is_err());
    assert!(parse(&["--tls-key", "s.key"]).is_err());
    assert!(parse(&["--tls-client-ca", "ca.pem"]).is_err());
    assert!(parse(&["--pgwire-require-tls"]).is_err());
}

#[test]
fn plaintext_exposure_is_warned_only_off_loopback_without_tls() {
    let public: SocketAddr = "0.0.0.0:5432".parse().unwrap();
    let local: SocketAddr = "127.0.0.1:5432".parse().unwrap();
    let w = plaintext_exposure_warning("pgwire", public, false).unwrap();
    assert!(w.contains("pgwire") && w.contains("--tls-cert"), "{w}");
    assert!(plaintext_exposure_warning("pgwire", public, true).is_none());
    assert!(plaintext_exposure_warning("pgwire", local, false).is_none());
}

#[test]
fn load_tls_reads_the_files_and_detects_mtls() {
    let p = pki();
    let a = parse(&[
        "--tls-cert",
        &p.path("server.pem"),
        "--tls-key",
        &p.path("server.key"),
    ])
    .unwrap();
    let t = load_tls(&a).unwrap().unwrap();
    assert!(!t.is_mutual());
    assert!(!t.net.requires_client_cert());

    let a = parse(&[
        "--tls-cert",
        &p.path("server.pem"),
        "--tls-key",
        &p.path("server.key"),
        "--tls-client-ca",
        &p.path("ca.pem"),
    ])
    .unwrap();
    let t = load_tls(&a).unwrap().unwrap();
    assert!(t.is_mutual());
    assert!(t.net.requires_client_cert());

    assert!(load_tls(&parse(&[]).unwrap()).unwrap().is_none());
}

// ── the material actually secures the listeners ──────────────────────────────

fn mtls_args(p: &Pki, data: &TempDir) -> ServeArgs {
    let mut a = parse(&[
        "--tls-cert",
        &p.path("server.pem"),
        "--tls-key",
        &p.path("server.key"),
        "--tls-client-ca",
        &p.path("ca.pem"),
        "--columns",
        "pk",
        "v",
        "--addr",
        "127.0.0.1:0",
    ])
    .unwrap();
    a.dir = data.path().to_path_buf();
    a
}

#[test]
fn the_wire_listener_requires_a_client_certificate_under_mtls() {
    let p = pki();
    let data = TempDir::new().unwrap();
    let bound = bind(&mtls_args(&p, &data)).unwrap();
    assert!(bound.tls_enabled);
    let addr = bound.server.local_addr().unwrap();
    let shutdown = bound.server.shutdown_handle();
    std::thread::spawn(move || {
        let _ = bound.server.serve();
    });

    let no_cert =
        ClientTlsConfig::pin_certificate(p.server_cert_pem.as_bytes(), "localhost").unwrap();
    let r = Client::connect_tls(addr, &no_cert).and_then(|mut c| {
        c.set_timeout(Some(Duration::from_secs(5)))?;
        c.ping()
    });
    assert!(
        r.is_err(),
        "the wire listener served a client without a cert"
    );

    let with_cert = ClientTlsConfig::pin_certificate_with_client_cert(
        p.server_cert_pem.as_bytes(),
        "localhost",
        p.client_cert_pem.as_bytes(),
        p.client_key_pem.as_bytes(),
    )
    .unwrap();
    let mut c = Client::connect_tls(addr, &with_cert).unwrap();
    c.set_timeout(Some(Duration::from_secs(5))).unwrap();
    c.ping().unwrap();
    shutdown.shutdown();
}

#[tokio::test]
async fn the_pgwire_listener_upgrades_with_the_serve_tls_material() {
    let p = pki();
    let data = TempDir::new().unwrap();
    let a = parse(&[
        "--tls-cert",
        &p.path("server.pem"),
        "--tls-key",
        &p.path("server.key"),
    ])
    .unwrap();
    let tls = load_tls(&a).unwrap().unwrap();
    let registry = Arc::new(
        vectorvolt_engine::TableRegistry::open(
            data.path(),
            vectorvolt_engine::EngineOptions::default(),
        )
        .unwrap(),
    );
    let cfg = PgConfig::new(pgwire_addr(&a, 0), data.path().to_path_buf());
    let server = build_pg_server(cfg, registry, None, Default::default(), None)
        .with_tls(Some(tls.net.rustls_config()))
        .with_require_tls(true);
    let (listener, addr) = server.bind().await.unwrap();
    tokio::spawn(server.serve_listener_until(listener, addr, std::future::pending()));

    let mut s = tokio::net::TcpStream::connect(addr).await.unwrap();
    s.write_all(&[0, 0, 0, 8, 0x04, 0xd2, 0x16, 0x2f])
        .await
        .unwrap();
    let mut answer = [0u8; 1];
    s.read_exact(&mut answer).await.unwrap();
    assert_eq!(answer[0], b'S');

    let mut roots = rustls::RootCertStore::empty();
    roots.add(p.server_cert_der.clone()).unwrap();
    let cfg = rustls::ClientConfig::builder_with_provider(Arc::new(
        rustls::crypto::ring::default_provider(),
    ))
    .with_safe_default_protocol_versions()
    .unwrap()
    .with_root_certificates(roots)
    .with_no_client_auth();
    let mut tls_stream = tokio_rustls::TlsConnector::from(Arc::new(cfg))
        .connect(ServerName::try_from("localhost").unwrap(), s)
        .await
        .unwrap();
    let params = b"user\0postgres\0database\0vectorvolt\0\0";
    let mut startup = ((8 + params.len()) as i32).to_be_bytes().to_vec();
    startup.extend_from_slice(&(3i32 << 16).to_be_bytes());
    startup.extend_from_slice(params);
    tls_stream.write_all(&startup).await.unwrap();
    tls_stream.flush().await.unwrap();
    let mut first = [0u8; 1];
    tls_stream.read_exact(&mut first).await.unwrap();
    assert_eq!(first[0], b'R', "no AuthenticationOk over TLS");
}

/// D2 — `--pgwire-auth` selects SCRAM (default), MD5 or cleartext, and
/// `build_pg_server` applies it.
#[test]
fn pgwire_auth_flag_parses_and_plumbs() {
    use vectorvolt_reader::auth::AuthMethod;
    assert_eq!(
        parse(&[]).unwrap().listener.pgwire_auth,
        AuthMethod::ScramSha256
    );
    for (flag, want) in [
        ("scram", AuthMethod::ScramSha256),
        ("md5", AuthMethod::Md5),
        ("password", AuthMethod::Password),
    ] {
        let args = parse(&["--pgwire-auth", flag]).unwrap();
        assert_eq!(args.listener.pgwire_auth, want, "{flag}");
    }
    assert!(parse(&["--pgwire-auth", "trust"]).is_err());

    let dir = TempDir::new().unwrap();
    let registry =
        Arc::new(vectorvolt_engine::TableRegistry::open(dir.path(), Default::default()).unwrap());
    let cfg = PgConfig::new("127.0.0.1:0".parse().unwrap(), dir.path().to_path_buf());
    let server = build_pg_server(cfg, registry, Some("pw".into()), AuthMethod::Md5, None);
    assert_eq!(server.auth_method(), AuthMethod::Md5);
}
