//! TLS over real sockets (VV-SEC-005/006).
//!
//! The unit tests in `tls` cover configuration. These cover the thing that
//! matters: that an untrusted server is actually rejected, that rejection happens
//! **before** a credential is sent, and that a real session works end to end.

use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::time::Duration;

use tempfile::TempDir;
use vectorvolt_engine::{Durability, EngineOptions, TableEngine};
use vectorvolt_net::server::ServerStats;
use vectorvolt_net::{
    AuthConfig, Client, ClientTlsConfig, Scope, Secret, Server, ServerConfig, ServerTlsConfig,
};

const SECRET: &[u8] = b"a-sufficiently-long-shared-secret";

fn self_signed(names: Vec<String>) -> (Vec<u8>, Vec<u8>) {
    let c = rcgen::generate_simple_self_signed(names).unwrap();
    (
        c.cert.pem().into_bytes(),
        c.key_pair.serialize_pem().into_bytes(),
    )
}

struct Harness {
    addr: SocketAddr,
    stats: Arc<ServerStats>,
    shutdown: vectorvolt_net::server::ShutdownHandle,
    _dir: TempDir,
}

/// Start a server. `tls` and `auth` are independent, which is the point.
fn start(tls: Option<ServerTlsConfig>, auth: bool) -> Harness {
    let dir = TempDir::new().unwrap();
    let engine = Arc::new(
        TableEngine::open_with(
            vec!["id".into(), "price".into(), "qty".into()],
            dir.path(),
            EngineOptions::new(1_000_000).with_durability(Durability::WriteOnly),
        )
        .unwrap(),
    );
    let mut cfg = ServerConfig::new("127.0.0.1:0".parse().unwrap());
    if let Some(t) = tls {
        cfg = cfg.with_tls(t);
        assert!(cfg.is_encrypted());
    }
    if auth {
        cfg = cfg.with_auth(AuthConfig::disabled().with_scoped_key(
            "app1",
            Secret::new(SECRET.to_vec()).unwrap(),
            Scope::Admin,
        ));
    }
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

impl Drop for Harness {
    fn drop(&mut self) {
        self.shutdown.shutdown();
    }
}

/// A full session over TLS.
#[test]
fn crud_works_over_tls() {
    let (cert, key) = self_signed(vec!["localhost".into()]);
    let h = start(Some(ServerTlsConfig::from_pem(&cert, &key).unwrap()), false);
    let tls = ClientTlsConfig::pin_certificate(&cert, "localhost").unwrap();

    let mut c = Client::connect_tls(h.addr, &tls).unwrap();
    assert!(c.is_encrypted(), "the connection reports as plaintext");
    c.set_timeout(Some(Duration::from_secs(10))).unwrap();

    for i in 1..=25u64 {
        c.insert(i, vec![i, i * 100, i * 7]).unwrap();
    }
    for i in 1..=25u64 {
        assert_eq!(c.get(i).unwrap().unwrap(), vec![i, i * 100, i * 7]);
    }
    c.flush().unwrap();
    assert_eq!(c.range_scan(0, 5, 15, 0).unwrap().len(), 11);
    assert!(c.delete(3).unwrap());
    assert!(c.get(3).unwrap().is_none());
    assert_eq!(h.stats.tls_handshake_failures.load(Ordering::Relaxed), 0);
}

/// **A server presenting an untrusted certificate is rejected.**
///
/// The property that makes TLS worth anything. A client that would accept any
/// certificate has encryption without authentication, which is encryption to
/// whoever intercepted the connection.
#[test]
fn a_client_rejects_a_server_it_does_not_trust() {
    let (server_cert, server_key) = self_signed(vec!["localhost".into()]);
    // A completely different certificate — as an interceptor would present.
    let (other_cert, _) = self_signed(vec!["localhost".into()]);

    let h = start(
        Some(ServerTlsConfig::from_pem(&server_cert, &server_key).unwrap()),
        false,
    );
    let wrong = ClientTlsConfig::pin_certificate(&other_cert, "localhost").unwrap();

    let err = match Client::connect_tls(h.addr, &wrong) {
        Err(e) => e,
        Ok(_) => panic!("connected to a server with an untrusted certificate"),
    };
    let msg = err.to_string();
    assert!(
        msg.contains("TLS") || msg.to_lowercase().contains("certificate"),
        "unhelpful rejection message: {msg}"
    );

    // The correct pin still works, so the server itself is fine.
    let right = ClientTlsConfig::pin_certificate(&server_cert, "localhost").unwrap();
    let mut ok = Client::connect_tls(h.addr, &right).unwrap();
    ok.ping().unwrap();
}

/// **Verification happens before any credential is sent.**
///
/// If the certificate check were deferred to first use, `connect` would succeed
/// against an impostor and the caller could hand over its secret before anything
/// failed. This asserts the failure lands at connect, with authentication never
/// attempted.
#[test]
fn an_untrusted_server_is_rejected_before_credentials_are_sent() {
    let (server_cert, server_key) = self_signed(vec!["localhost".into()]);
    let (other_cert, _) = self_signed(vec!["localhost".into()]);
    let h = start(
        Some(ServerTlsConfig::from_pem(&server_cert, &server_key).unwrap()),
        true,
    );
    let wrong = ClientTlsConfig::pin_certificate(&other_cert, "localhost").unwrap();

    assert!(
        Client::connect_tls_authenticated(h.addr, &wrong, "app1", SECRET).is_err(),
        "authenticated against an untrusted server"
    );
    // THE assertion: no authentication attempt reached the server at all. Not
    // "it was rejected" — never attempted, because the client abandoned the
    // connection before sending anything. This is what "verified before
    // credentials are sent" means, and it holds regardless of timing.
    assert_eq!(h.stats.auth_successes.load(Ordering::Relaxed), 0);
    assert_eq!(
        h.stats.auth_failures.load(Ordering::Relaxed),
        0,
        "an authentication attempt reached the server despite an untrusted cert"
    );

    // The server-side counter is inherently racy to observe: the client rejects
    // the certificate and returns as soon as it has sent its fatal alert, so the
    // server thread may not have processed it yet. Polled rather than asserted
    // immediately — asserting straight away passes or fails on scheduling, which
    // is how a flaky test gets written.
    let mut seen = false;
    for _ in 0..100 {
        if h.stats.tls_handshake_failures.load(Ordering::Relaxed) >= 1 {
            seen = true;
            break;
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    assert!(seen, "the server never recorded the failed handshake");
}

/// The hostname is verified even when the certificate is pinned.
///
/// Pinning answers "is this the cert I expect"; the name check answers "is this
/// cert valid for the host I asked for". Both matter, and pinning must not
/// silently disable the second.
#[test]
fn the_server_name_is_verified_against_the_certificate() {
    let (cert, key) = self_signed(vec!["localhost".into()]);
    let h = start(Some(ServerTlsConfig::from_pem(&cert, &key).unwrap()), false);

    // Right certificate, wrong name.
    let mismatched = ClientTlsConfig::pin_certificate(&cert, "wrong.example.com").unwrap();
    assert!(
        Client::connect_tls(h.addr, &mismatched).is_err(),
        "a name not on the certificate was accepted"
    );

    let matched = ClientTlsConfig::pin_certificate(&cert, "localhost").unwrap();
    Client::connect_tls(h.addr, &matched)
        .unwrap()
        .ping()
        .unwrap();
}

/// A plaintext client cannot talk to a TLS server, and fails rather than hanging.
#[test]
fn a_plaintext_client_cannot_use_a_tls_server() {
    let (cert, key) = self_signed(vec!["localhost".into()]);
    let h = start(Some(ServerTlsConfig::from_pem(&cert, &key).unwrap()), false);

    let mut c = Client::connect(h.addr).unwrap();
    c.set_timeout(Some(Duration::from_secs(5))).unwrap();
    assert!(!c.is_encrypted());
    assert!(
        c.ping().is_err(),
        "a plaintext request was served by a TLS listener"
    );
}

/// A TLS client cannot talk to a plaintext server.
///
/// The reverse mismatch. It must fail cleanly — the server sends a protocol error
/// frame that is not a TLS record, and the client must treat that as a handshake
/// failure rather than misparsing it.
#[test]
fn a_tls_client_cannot_use_a_plaintext_server() {
    let (cert, _) = self_signed(vec!["localhost".into()]);
    let h = start(None, false);
    let tls = ClientTlsConfig::pin_certificate(&cert, "localhost").unwrap();
    assert!(
        Client::connect_tls(h.addr, &tls).is_err(),
        "a TLS handshake succeeded against a plaintext server"
    );
}

/// Garbage on a TLS port fails the handshake and does not harm the server.
#[test]
fn garbage_on_a_tls_port_is_survivable() {
    let (cert, key) = self_signed(vec!["localhost".into()]);
    let h = start(Some(ServerTlsConfig::from_pem(&cert, &key).unwrap()), false);

    for junk in [
        vec![0xFFu8; 512],
        vec![0u8; 8],
        b"GET / HTTP/1.1\r\n\r\n".to_vec(),
    ] {
        let mut s = TcpStream::connect(h.addr).unwrap();
        let _ = s.write_all(&junk);
        let _ = s.flush();
        let mut buf = [0u8; 64];
        let _ = s.read(&mut buf);
    }
    // Connect and close mid-handshake.
    drop(TcpStream::connect(h.addr).unwrap());

    let tls = ClientTlsConfig::pin_certificate(&cert, "localhost").unwrap();
    let mut c = Client::connect_tls(h.addr, &tls).unwrap();
    c.insert(1, vec![1, 2, 3]).unwrap();
    assert_eq!(c.get(1).unwrap().unwrap(), vec![1, 2, 3]);
}

/// TLS and authentication compose: the production configuration.
#[test]
fn tls_and_authentication_together() {
    let (cert, key) = self_signed(vec!["localhost".into()]);
    let h = start(Some(ServerTlsConfig::from_pem(&cert, &key).unwrap()), true);
    let tls = ClientTlsConfig::pin_certificate(&cert, "localhost").unwrap();

    // Encrypted but unauthenticated: refused.
    let mut anon = Client::connect_tls(h.addr, &tls).unwrap();
    assert!(
        anon.get(1).is_err(),
        "unauthenticated read over TLS allowed"
    );

    // Encrypted and authenticated: works.
    let mut c = Client::connect_tls_authenticated(h.addr, &tls, "app1", SECRET).unwrap();
    assert!(c.is_encrypted());
    c.insert(1, vec![1, 2, 3]).unwrap();
    assert_eq!(c.get(1).unwrap().unwrap(), vec![1, 2, 3]);

    // Wrong secret over TLS is still refused — encryption is not authorisation.
    let mut bad = Client::connect_tls(h.addr, &tls).unwrap();
    assert!(
        bad.authenticate("app1", b"wrong-secret-wrong-secret")
            .is_err()
    );

    assert_eq!(h.stats.auth_successes.load(Ordering::Relaxed), 1);
    assert_eq!(h.stats.auth_failures.load(Ordering::Relaxed), 1);
}

/// Several TLS clients at once, each with its own session.
#[test]
fn concurrent_tls_clients_are_served_independently() {
    let (cert, key) = self_signed(vec!["localhost".into()]);
    let h = start(Some(ServerTlsConfig::from_pem(&cert, &key).unwrap()), false);

    let handles: Vec<_> = (0..6u64)
        .map(|t| {
            let addr = h.addr;
            let cert = cert.clone();
            std::thread::spawn(move || {
                let tls = ClientTlsConfig::pin_certificate(&cert, "localhost").unwrap();
                let mut c = Client::connect_tls(addr, &tls).unwrap();
                let base = t * 1000;
                for i in 0..25u64 {
                    c.insert(base + i, vec![base + i, t, i]).unwrap();
                }
                for i in 0..25u64 {
                    let row = c.get(base + i).unwrap().unwrap();
                    assert_eq!(row[0], base + i);
                    assert_eq!(row[1], t, "a TLS session crossed with another");
                }
            })
        })
        .collect();
    for handle in handles {
        handle.join().expect("a TLS client thread panicked");
    }
    assert_eq!(h.stats.tls_handshake_failures.load(Ordering::Relaxed), 0);
}

/// A payload larger than a TLS record survives.
///
/// rustls fragments at 16 KiB. A frame bigger than that spans several records, so
/// this exercises reassembly — the case a naive single-read implementation gets
/// wrong on top of the TCP segmentation it already had to handle.
#[test]
fn large_frames_survive_tls_record_fragmentation() {
    let (cert, key) = self_signed(vec!["localhost".into()]);
    let h = start(Some(ServerTlsConfig::from_pem(&cert, &key).unwrap()), false);
    let tls = ClientTlsConfig::pin_certificate(&cert, "localhost").unwrap();
    let mut c = Client::connect_tls(h.addr, &tls).unwrap();

    for i in 0..2000u64 {
        c.insert(i, vec![i, i * 2, i * 3]).unwrap();
    }
    c.flush().unwrap();
    // ~2000 rows x 3 u64s x 8 bytes = far beyond one 16 KiB record.
    let rows = c.range_scan(0, 0, 1999, 0).unwrap();
    assert_eq!(rows.len(), 2000);
    for (i, row) in rows.iter().enumerate() {
        assert_eq!(row[0], i as u64, "row {i} corrupted across records");
        assert_eq!(row[2], i as u64 * 3);
    }
}
