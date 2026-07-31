//! Authorisation scopes over real sockets (VV-SEC-004).
//!
//! Authentication answers "who is this". These cover the next question: "and what
//! may they do". Every scope is checked against every operation, because an
//! authorisation table that is right for six of seven opcodes is not right.

use std::net::SocketAddr;
use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::time::Duration;

use tempfile::TempDir;
use vectorvolt_engine::{Durability, EngineOptions, TableEngine};
use vectorvolt_net::server::ServerStats;
use vectorvolt_net::{
    AuthConfig, Client, Request, Response, Scope, Secret, Server, ServerConfig, error_code,
};

const RO: &[u8] = b"read-only-key-secret-value-long";
const RW: &[u8] = b"read-write-key-secret-value-long";
const AD: &[u8] = b"admin-key-secret-value-long-here";

struct Harness {
    addr: SocketAddr,
    stats: Arc<ServerStats>,
    shutdown: vectorvolt_net::server::ShutdownHandle,
    _dir: TempDir,
}

fn start() -> Harness {
    let dir = TempDir::new().unwrap();
    let engine = Arc::new(
        TableEngine::open_with(
            vec!["id".into(), "price".into(), "qty".into()],
            dir.path(),
            EngineOptions::new(1_000_000).with_durability(Durability::WriteOnly),
        )
        .unwrap(),
    );
    let auth = AuthConfig::disabled()
        .with_scoped_key("ro", Secret::new(RO.to_vec()).unwrap(), Scope::ReadOnly)
        .with_scoped_key("rw", Secret::new(RW.to_vec()).unwrap(), Scope::ReadWrite)
        .with_scoped_key("admin", Secret::new(AD.to_vec()).unwrap(), Scope::Admin);
    let cfg = ServerConfig::new("127.0.0.1:0".parse().unwrap()).with_auth(auth);
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

fn conn(addr: SocketAddr, key: &str, secret: &[u8]) -> Client {
    let mut c = Client::connect(addr).unwrap();
    c.set_timeout(Some(Duration::from_secs(10))).unwrap();
    c.authenticate(key, secret).unwrap();
    c
}

/// Is this request allowed for this connection?
fn allowed(c: &mut Client, req: &Request) -> bool {
    match c.call(req).unwrap() {
        Response::Error { code, .. } if code == error_code::FORBIDDEN => false,
        // Any other error is an engine-level outcome (missing key, bad schema),
        // which means the request got THROUGH authorisation. That distinction is
        // the whole point of a separate FORBIDDEN code.
        _ => true,
    }
}

fn all_ops() -> Vec<(&'static str, Request, Scope)> {
    vec![
        ("Get", Request::Get { pk: 1 }, Scope::ReadOnly),
        (
            "RangeScan",
            Request::RangeScan {
                col: 0,
                lo: 0,
                hi: 10,
                limit: 0,
            },
            Scope::ReadOnly,
        ),
        ("Stats", Request::Stats, Scope::ReadOnly),
        (
            "Insert",
            Request::Insert {
                pk: 1,
                values: vec![1, 2, 3],
            },
            Scope::ReadWrite,
        ),
        ("Delete", Request::Delete { pk: 1 }, Scope::ReadWrite),
        ("Flush", Request::Flush, Scope::Admin),
        ("Compact", Request::Compact, Scope::Admin),
    ]
}

/// **The full authorisation matrix**: every scope against every operation.
///
/// Spot-checking would leave the interesting cases untested. This asserts both
/// directions — that each scope is permitted everything at or below it, and
/// refused everything above.
#[test]
fn every_scope_against_every_operation() {
    let h = start();
    for (key, secret, held) in [
        ("ro", RO, Scope::ReadOnly),
        ("rw", RW, Scope::ReadWrite),
        ("admin", AD, Scope::Admin),
    ] {
        let mut c = conn(h.addr, key, secret);
        for (name, req, required) in all_ops() {
            let want = held >= required;
            let got = allowed(&mut c, &req);
            assert_eq!(
                got,
                want,
                "{key} ({}) on {name} (needs {}): expected allowed={want}",
                held.as_str(),
                required.as_str()
            );
        }
    }
}

/// A read-only key cannot change anything, and nothing it tried landed.
#[test]
fn a_read_only_key_cannot_mutate() {
    let h = start();
    let mut writer = conn(h.addr, "rw", RW);
    writer.insert(1, vec![1, 100, 7]).unwrap();

    let mut ro = conn(h.addr, "ro", RO);
    assert_eq!(ro.get(1).unwrap().unwrap(), vec![1, 100, 7], "reads work");

    // Every mutation refused.
    for req in [
        Request::Insert {
            pk: 2,
            values: vec![2, 2, 2],
        },
        Request::Delete { pk: 1 },
        Request::Flush,
        Request::Compact,
    ] {
        match ro.call(&req).unwrap() {
            Response::Error { code, .. } => assert_eq!(code, error_code::FORBIDDEN, "{req:?}"),
            other => panic!("{req:?} was allowed for a read-only key: {other:?}"),
        }
    }

    // And the data is untouched — the check happened BEFORE the engine, not after.
    let mut check = conn(h.addr, "admin", AD);
    assert_eq!(check.get(1).unwrap().unwrap(), vec![1, 100, 7]);
    assert!(check.get(2).unwrap().is_none(), "a forbidden insert landed");
}

/// A read-write key can change rows but cannot rewrite files.
///
/// The line that matters most: `Compact` rewrites every partition in a job and
/// deletes the originals, so one call from a misconfigured writer can rewrite a
/// whole table.
#[test]
fn a_read_write_key_cannot_compact() {
    let h = start();
    let mut rw = conn(h.addr, "rw", RW);
    rw.insert(1, vec![1, 2, 3]).unwrap();
    assert!(rw.delete(1).unwrap());

    for req in [Request::Flush, Request::Compact] {
        match rw.call(&req).unwrap() {
            Response::Error { code, .. } => assert_eq!(code, error_code::FORBIDDEN, "{req:?}"),
            other => panic!("{req:?} was allowed for a read-write key: {other:?}"),
        }
    }
    let mut admin = conn(h.addr, "admin", AD);
    admin.flush().unwrap();
}

/// FORBIDDEN and UNAUTHENTICATED are different codes.
///
/// "Log in" and "you cannot do this" need different responses from a client, and
/// conflating them sends operators hunting for a credential problem that does not
/// exist.
#[test]
fn forbidden_is_distinguishable_from_unauthenticated() {
    let h = start();

    let mut anon = Client::connect(h.addr).unwrap();
    match anon.call(&Request::Get { pk: 1 }).unwrap() {
        Response::Error { code, .. } => assert_eq!(code, error_code::UNAUTHENTICATED),
        other => panic!("anonymous read allowed: {other:?}"),
    }

    let mut ro = conn(h.addr, "ro", RO);
    match ro.call(&Request::Compact).unwrap() {
        Response::Error { code, message } => {
            assert_eq!(code, error_code::FORBIDDEN);
            // The message names both sides — an authz failure is nearly always a
            // misconfiguration, and a bare "forbidden" turns a one-minute fix into
            // an afternoon.
            assert!(
                message.contains("admin"),
                "message lacks required: {message}"
            );
            assert!(
                message.contains("read-only"),
                "message lacks held scope: {message}"
            );
        }
        other => panic!("compact allowed for read-only: {other:?}"),
    }

    assert_eq!(h.stats.forbidden_rejections.load(Ordering::Relaxed), 1);
    assert_eq!(
        h.stats.unauthenticated_rejections.load(Ordering::Relaxed),
        1
    );
}

/// Scope is bound to the key, not the connection order.
#[test]
fn scope_follows_the_key_across_connections() {
    let h = start();
    // Two connections, different keys, interleaved.
    let mut ro = conn(h.addr, "ro", RO);
    let mut admin = conn(h.addr, "admin", AD);

    admin.insert(1, vec![1, 2, 3]).unwrap();
    assert!(!allowed(&mut ro, &Request::Compact));
    assert!(allowed(&mut admin, &Request::Compact));
    assert!(!allowed(
        &mut ro,
        &Request::Insert {
            pk: 9,
            values: vec![9, 9, 9]
        }
    ));
    assert!(allowed(&mut admin, &Request::Flush));
}

/// A key added without an explicit scope is read-only.
///
/// Least privilege by default: forgetting to specify a scope must under-grant, not
/// over-grant.
#[test]
fn keys_default_to_read_only() {
    let dir = TempDir::new().unwrap();
    let engine = Arc::new(
        TableEngine::open_with(
            vec!["id".into(), "price".into(), "qty".into()],
            dir.path(),
            EngineOptions::new(1_000_000).with_durability(Durability::WriteOnly),
        )
        .unwrap(),
    );
    let auth = AuthConfig::disabled().with_key("plain", Secret::new(RO.to_vec()).unwrap());
    let server = Server::bind(
        engine,
        ServerConfig::new("127.0.0.1:0".parse().unwrap()).with_auth(auth),
    )
    .unwrap();
    let addr = server.local_addr().unwrap();
    let sd = server.shutdown_handle();
    std::thread::spawn(move || {
        let _ = server.serve();
    });

    let mut c = conn(addr, "plain", RO);
    assert!(allowed(&mut c, &Request::Get { pk: 1 }));
    assert!(!allowed(
        &mut c,
        &Request::Insert {
            pk: 1,
            values: vec![1, 2, 3]
        }
    ));
    sd.shutdown();
}

/// With authentication disabled, scopes do not apply.
///
/// There is nobody to distinguish, so enforcing a scope would break the embedded
/// case while providing no security.
#[test]
fn scopes_are_inert_when_auth_is_disabled() {
    let dir = TempDir::new().unwrap();
    let engine = Arc::new(
        TableEngine::open_with(
            vec!["id".into(), "price".into(), "qty".into()],
            dir.path(),
            EngineOptions::new(1_000_000).with_durability(Durability::WriteOnly),
        )
        .unwrap(),
    );
    let server = Server::bind(engine, ServerConfig::new("127.0.0.1:0".parse().unwrap())).unwrap();
    let addr = server.local_addr().unwrap();
    let sd = server.shutdown_handle();
    std::thread::spawn(move || {
        let _ = server.serve();
    });

    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 2, 3]).unwrap();
    c.flush().unwrap();
    let _ = c.compact().unwrap();
    assert!(c.delete(1).unwrap());
    sd.shutdown();
}

/// The scope ordering is a real total order.
#[test]
fn scope_ordering_is_total_and_inclusive() {
    assert!(Scope::Admin.allows(Scope::ReadWrite));
    assert!(Scope::Admin.allows(Scope::ReadOnly));
    assert!(Scope::ReadWrite.allows(Scope::ReadOnly));
    assert!(Scope::ReadOnly.allows(Scope::ReadOnly));

    assert!(!Scope::ReadOnly.allows(Scope::ReadWrite));
    assert!(!Scope::ReadOnly.allows(Scope::Admin));
    assert!(!Scope::ReadWrite.allows(Scope::Admin));

    // Default is the least privileged.
    assert_eq!(Scope::default(), Scope::ReadOnly);
}
