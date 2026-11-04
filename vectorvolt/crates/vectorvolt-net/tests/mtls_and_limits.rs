//! Mutual TLS and the connection limit on the native wire listener (D1/D4).
//!
//! `--tls-client-ca` makes the server demand a client certificate; a client
//! without one must not get as far as a single request. `max_connections` is a
//! hard cap: connections beyond it are closed on accept, not queued.

use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::time::Duration;

use tempfile::TempDir;
use vectorvolt_engine::{Durability, EngineOptions, TableEngine};
use vectorvolt_net::server::{ServerStats, ShutdownHandle};
use vectorvolt_net::{Client, ClientTlsConfig, Server, ServerConfig, ServerTlsConfig};

struct Pki {
    server_cert: Vec<u8>,
    server_key: Vec<u8>,
    ca_cert: Vec<u8>,
    client_cert: Vec<u8>,
    client_key: Vec<u8>,
}

fn pki() -> Pki {
    let server = rcgen::generate_simple_self_signed(vec!["localhost".to_string()]).unwrap();

    let ca_key = rcgen::KeyPair::generate().unwrap();
    let mut ca_params = rcgen::CertificateParams::new(Vec::<String>::new()).unwrap();
    ca_params.is_ca = rcgen::IsCa::Ca(rcgen::BasicConstraints::Unconstrained);
    ca_params
        .distinguished_name
        .push(rcgen::DnType::CommonName, "vectorvolt test client CA");
    let ca = ca_params.self_signed(&ca_key).unwrap();

    let client_key = rcgen::KeyPair::generate().unwrap();
    let mut client_params = rcgen::CertificateParams::new(vec!["client".to_string()]).unwrap();
    client_params.extended_key_usages = vec![rcgen::ExtendedKeyUsagePurpose::ClientAuth];
    let client = client_params.signed_by(&client_key, &ca, &ca_key).unwrap();

    Pki {
        server_cert: server.cert.pem().into_bytes(),
        server_key: server.key_pair.serialize_pem().into_bytes(),
        ca_cert: ca.pem().into_bytes(),
        client_cert: client.pem().into_bytes(),
        client_key: client_key.serialize_pem().into_bytes(),
    }
}

struct Harness {
    addr: std::net::SocketAddr,
    stats: Arc<ServerStats>,
    shutdown: ShutdownHandle,
    _dir: TempDir,
}

impl Drop for Harness {
    fn drop(&mut self) {
        self.shutdown.shutdown();
    }
}

fn start(cfg: ServerConfig) -> Harness {
    let dir = TempDir::new().unwrap();
    let engine = Arc::new(
        TableEngine::open_with(
            vec!["id".into(), "v".into()],
            dir.path(),
            EngineOptions::new(1_000_000).with_durability(Durability::WriteOnly),
        )
        .unwrap(),
    );
    let server = Server::bind(engine, cfg).unwrap();
    let addr = server.local_addr().unwrap();
    let stats = server.stats();
    let shutdown = server.shutdown_handle();
    std::thread::spawn(move || {
        let _ = server.serve();
    });
    Harness {
        addr,
        stats,
        shutdown,
        _dir: dir,
    }
}

fn mtls_server(p: &Pki) -> Harness {
    let tls = ServerTlsConfig::from_pem_with_client_ca(&p.server_cert, &p.server_key, &p.ca_cert)
        .unwrap();
    assert!(tls.requires_client_cert());
    start(ServerConfig::new("127.0.0.1:0".parse().unwrap()).with_tls(tls))
}

#[test]
fn mtls_rejects_a_client_without_a_certificate() {
    let p = pki();
    let h = mtls_server(&p);
    let no_cert = ClientTlsConfig::pin_certificate(&p.server_cert, "localhost").unwrap();
    // Under TLS 1.3 the client may finish its side of the handshake before
    // the server rejects the (missing) certificate, so the failure can surface
    // on connect or on the first request — but it must surface.
    let result = Client::connect_tls(h.addr, &no_cert).and_then(|mut c| {
        c.set_timeout(Some(Duration::from_secs(5)))?;
        c.ping()
    });
    assert!(result.is_err(), "a client without a certificate was served");
    // The server attributes it to TLS.
    for _ in 0..50 {
        if h.stats.tls_handshake_failures.load(Ordering::Relaxed) > 0 {
            return;
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    panic!("the rejection was not counted as a TLS handshake failure");
}

#[test]
fn mtls_accepts_a_client_with_a_ca_issued_certificate() {
    let p = pki();
    let h = mtls_server(&p);
    let with_cert = ClientTlsConfig::pin_certificate_with_client_cert(
        &p.server_cert,
        "localhost",
        &p.client_cert,
        &p.client_key,
    )
    .unwrap();
    let mut c = Client::connect_tls(h.addr, &with_cert).unwrap();
    c.set_timeout(Some(Duration::from_secs(5))).unwrap();
    c.ping().unwrap();
    c.insert(1, vec![1, 42]).unwrap();
    assert_eq!(c.get(1).unwrap().unwrap(), vec![1, 42]);
}

#[test]
fn mtls_rejects_a_certificate_from_another_issuer() {
    let p = pki();
    let other = pki(); // a different CA entirely
    let h = mtls_server(&p);
    let wrong = ClientTlsConfig::pin_certificate_with_client_cert(
        &p.server_cert,
        "localhost",
        &other.client_cert,
        &other.client_key,
    )
    .unwrap();
    let result = Client::connect_tls(h.addr, &wrong).and_then(|mut c| {
        c.set_timeout(Some(Duration::from_secs(5)))?;
        c.ping()
    });
    assert!(
        result.is_err(),
        "a certificate from an untrusted CA was served"
    );
}

#[test]
fn connections_beyond_max_connections_are_refused() {
    let mut cfg = ServerConfig::new("127.0.0.1:0".parse().unwrap());
    cfg.max_connections = 1;
    let h = start(cfg);

    let mut first = Client::connect(h.addr).unwrap();
    first.set_timeout(Some(Duration::from_secs(5))).unwrap();
    first.ping().unwrap();

    // The first connection is live, so the second is closed on accept.
    let second = Client::connect(h.addr).and_then(|mut c| {
        c.set_timeout(Some(Duration::from_secs(5)))?;
        c.ping()
    });
    assert!(second.is_err(), "a connection beyond the limit was served");
    assert!(h.stats.connections_refused.load(Ordering::Relaxed) >= 1);

    // The live one is unaffected.
    first.ping().unwrap();

    // Once it closes, a slot frees up.
    drop(first);
    let mut ok = None;
    for _ in 0..50 {
        std::thread::sleep(Duration::from_millis(20));
        if let Ok(mut c) = Client::connect(h.addr) {
            c.set_timeout(Some(Duration::from_secs(5))).unwrap();
            if c.ping().is_ok() {
                ok = Some(c);
                break;
            }
        }
    }
    assert!(ok.is_some(), "the slot was never released");
}
