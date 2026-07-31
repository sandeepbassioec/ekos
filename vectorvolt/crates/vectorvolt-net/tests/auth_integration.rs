//! Authentication over real sockets (VV-SEC-001/002/003).
//!
//! The unit tests in `auth` prove the challenge–response logic. These prove the
//! gate is actually in front of the engine — that an unauthenticated peer cannot
//! read, write, or learn anything about the data, no matter which opcode it tries.

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

const SECRET: &[u8] = b"a-sufficiently-long-shared-secret";

struct Harness {
    addr: SocketAddr,
    stats: Arc<ServerStats>,
    shutdown: vectorvolt_net::server::ShutdownHandle,
    _dir: TempDir,
}

fn start(auth: bool) -> Harness {
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
    if auth {
        // Admin explicitly. `with_key` defaults to ReadOnly (least privilege), so
        // these tests — which are about AUTHENTICATION, not authorisation — must
        // ask for full access rather than inherit it. Three of them failed when
        // the default was introduced, which is the default doing its job.
        cfg = cfg.with_auth(AuthConfig::disabled().with_scoped_key(
            "app1",
            Secret::new(SECRET.to_vec()).unwrap(),
            Scope::Admin,
        ));
        assert!(cfg.requires_auth());
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

fn client(addr: SocketAddr) -> Client {
    let c = Client::connect(addr).unwrap();
    c.set_timeout(Some(Duration::from_secs(10))).unwrap();
    c
}

/// The happy path: authenticate, then everything works.
#[test]
fn an_authenticated_client_has_full_access() {
    let h = start(true);
    let mut c = Client::connect_authenticated(h.addr, "app1", SECRET).unwrap();

    c.insert(1, vec![1, 100, 7]).unwrap();
    assert_eq!(c.get(1).unwrap().unwrap(), vec![1, 100, 7]);
    c.flush().unwrap();
    assert!(!c.range_scan(0, 0, 10, 0).unwrap().is_empty());
    assert!(c.delete(1).unwrap());
    let _ = c.stats().unwrap();

    assert_eq!(h.stats.auth_successes.load(Ordering::Relaxed), 1);
    assert_eq!(h.stats.auth_failures.load(Ordering::Relaxed), 0);
}

/// **Every data operation is refused before authentication.**
///
/// Enumerated rather than spot-checked: a gate that covers `Get` but forgets
/// `RangeScan` is worse than no gate, because it looks closed.
#[test]
fn unauthenticated_connections_can_do_nothing_to_the_data() {
    let h = start(true);
    let mut c = client(h.addr);

    let forbidden = vec![
        Request::Get { pk: 1 },
        Request::Insert {
            pk: 1,
            values: vec![1, 2, 3],
        },
        Request::Delete { pk: 1 },
        Request::RangeScan {
            col: 0,
            lo: 0,
            hi: u64::MAX,
            limit: 0,
        },
        Request::Flush,
        Request::Compact,
        // Stats leaks row counts and byte totals — information about the data,
        // even if not the data itself.
        Request::Stats,
    ];

    for req in &forbidden {
        match c.call(req).unwrap() {
            Response::Error { code, .. } => assert_eq!(
                code,
                error_code::UNAUTHENTICATED,
                "{req:?} was not refused for the right reason"
            ),
            other => panic!("{req:?} was ALLOWED without auth: {other:?}"),
        }
    }
    assert_eq!(
        h.stats.unauthenticated_rejections.load(Ordering::Relaxed),
        forbidden.len() as u64
    );

    // And nothing landed.
    let mut good = Client::connect_authenticated(h.addr, "app1", SECRET).unwrap();
    assert!(
        good.get(1).unwrap().is_none(),
        "an unauthenticated insert wrote data"
    );
}

/// `Ping` is allowed without credentials, deliberately.
///
/// A health check needs liveness without holding secrets, and a successful ping
/// reveals nothing a successful TCP connect does not.
#[test]
fn ping_is_allowed_without_authentication() {
    let h = start(true);
    let mut c = client(h.addr);
    c.ping().unwrap();
    c.ping().unwrap();
}

#[test]
fn a_wrong_secret_is_refused_and_grants_nothing() {
    let h = start(true);
    let mut c = client(h.addr);

    let err = c
        .authenticate("app1", b"the-wrong-secret-entirely")
        .unwrap_err();
    assert!(
        err.to_string()
            .contains(&error_code::AUTH_FAILED.to_string())
    );

    match c.call(&Request::Get { pk: 1 }).unwrap() {
        Response::Error { code, .. } => assert_eq!(code, error_code::UNAUTHENTICATED),
        other => panic!("access granted after a failed auth: {other:?}"),
    }
    assert_eq!(h.stats.auth_failures.load(Ordering::Relaxed), 1);
    assert_eq!(h.stats.auth_successes.load(Ordering::Relaxed), 0);
}

/// An unknown key id fails the same way a wrong secret does.
#[test]
fn an_unknown_key_id_is_refused_indistinguishably() {
    let h = start(true);
    let mut c = client(h.addr);
    let unknown = c.authenticate("nosuchkey", SECRET).unwrap_err().to_string();

    let mut c2 = client(h.addr);
    let wrong = c2
        .authenticate("app1", b"the-wrong-secret-entirely")
        .unwrap_err()
        .to_string();

    assert_eq!(
        unknown, wrong,
        "the two failures are distinguishable, which leaks which key ids exist"
    );
}

/// **A captured handshake cannot be replayed onto a new connection.**
///
/// This is the property that makes challenge–response worth the extra round trip
/// over a bearer token: an observer who records a full successful login learns
/// nothing reusable.
#[test]
fn a_captured_handshake_does_not_authenticate_a_new_connection() {
    let h = start(true);

    // Capture a real, successful exchange.
    let mut victim = client(h.addr);
    let nonce = match victim.call(&Request::Hello).unwrap() {
        Response::Challenge { nonce } => nonce,
        other => panic!("expected a challenge, got {other:?}"),
    };
    let response = vectorvolt_net::auth::client_response(SECRET, &nonce).to_vec();
    let captured = Request::Auth {
        key_id: "app1".into(),
        response,
    };
    assert!(matches!(
        victim.call(&captured).unwrap(),
        Response::Ok { .. }
    ));

    // Replay it verbatim on a fresh connection.
    let mut attacker = client(h.addr);
    let _ = attacker.call(&Request::Hello).unwrap(); // different nonce
    match attacker.call(&captured).unwrap() {
        Response::Error { code, .. } => assert_eq!(code, error_code::AUTH_FAILED),
        other => panic!("a replayed handshake authenticated: {other:?}"),
    }
    match attacker.call(&Request::Get { pk: 1 }).unwrap() {
        Response::Error { code, .. } => assert_eq!(code, error_code::UNAUTHENTICATED),
        other => panic!("replay granted access: {other:?}"),
    }
}

/// Answering without asking for a challenge fails.
#[test]
fn a_response_without_a_challenge_is_refused() {
    let h = start(true);
    let mut c = client(h.addr);
    let resp = c
        .call(&Request::Auth {
            key_id: "app1".into(),
            response: vec![0u8; 32],
        })
        .unwrap();
    match resp {
        Response::Error { code, .. } => assert_eq!(code, error_code::AUTH_FAILED),
        other => panic!("auth succeeded with no challenge: {other:?}"),
    }
}

/// Repeated failures close the connection, bounding online brute force.
#[test]
fn repeated_failures_close_the_connection() {
    let h = start(true);
    let mut c = client(h.addr);

    let mut got_error_before_close = 0;
    for _ in 0..vectorvolt_net::auth::MAX_AUTH_ATTEMPTS + 2 {
        if c.authenticate("app1", b"wrong-secret-wrong-secret").is_ok() {
            panic!("a wrong secret authenticated");
        }
        got_error_before_close += 1;
        // Once locked out the server answers and closes, so the next call fails at
        // the transport rather than returning an error response.
        if c.ping().is_err() {
            break;
        }
    }
    assert!(
        got_error_before_close <= vectorvolt_net::auth::MAX_AUTH_ATTEMPTS + 1,
        "the server allowed {got_error_before_close} attempts on one connection"
    );

    // A NEW connection still works — lock-out is per connection, not a global ban
    // that a wrong password could use to lock out everyone else.
    let mut fresh = Client::connect_authenticated(h.addr, "app1", SECRET).unwrap();
    fresh.ping().unwrap();
}

/// Authentication is per connection, not per process.
#[test]
fn authentication_does_not_leak_between_connections() {
    let h = start(true);
    let mut authed = Client::connect_authenticated(h.addr, "app1", SECRET).unwrap();
    authed.insert(1, vec![1, 2, 3]).unwrap();

    // A second connection is still anonymous.
    let mut other = client(h.addr);
    match other.call(&Request::Get { pk: 1 }).unwrap() {
        Response::Error { code, .. } => assert_eq!(code, error_code::UNAUTHENTICATED),
        other => panic!("a second connection inherited authentication: {other:?}"),
    }
}

/// With no credentials configured, everything works unauthenticated.
///
/// The embedded case. Openly open beats silently broken — and `requires_auth()`
/// lets a deployment assert which mode it is in rather than discover it.
#[test]
fn auth_disabled_leaves_the_server_fully_open() {
    let h = start(false);
    let mut c = client(h.addr);
    c.insert(1, vec![1, 2, 3]).unwrap();
    assert_eq!(c.get(1).unwrap().unwrap(), vec![1, 2, 3]);
    assert_eq!(
        h.stats.unauthenticated_rejections.load(Ordering::Relaxed),
        0
    );

    assert!(!ServerConfig::new("127.0.0.1:0".parse().unwrap()).requires_auth());
}

/// Authenticating twice on one connection is harmless.
#[test]
fn re_authenticating_is_idempotent() {
    let h = start(true);
    let mut c = Client::connect_authenticated(h.addr, "app1", SECRET).unwrap();
    c.insert(1, vec![1, 2, 3]).unwrap();
    c.authenticate("app1", SECRET).unwrap();
    assert_eq!(
        c.get(1).unwrap().unwrap(),
        vec![1, 2, 3],
        "re-authenticating disturbed the session"
    );
}

/// A malformed auth message is refused rather than crashing the handler.
#[test]
fn malformed_auth_messages_are_refused() {
    let h = start(true);
    for response in [vec![], vec![0u8; 1], vec![0u8; 31], vec![0u8; 1000]] {
        let mut c = client(h.addr);
        let _ = c.call(&Request::Hello).unwrap();
        match c
            .call(&Request::Auth {
                key_id: "app1".into(),
                response,
            })
            .unwrap()
        {
            Response::Error { code, .. } => assert_eq!(code, error_code::AUTH_FAILED),
            other => panic!("a malformed response authenticated: {other:?}"),
        }
    }
    // An empty key id, and a very long one.
    let mut c = client(h.addr);
    let _ = c.call(&Request::Hello).unwrap();
    assert!(matches!(
        c.call(&Request::Auth {
            key_id: String::new(),
            response: vec![0u8; 32],
        })
        .unwrap(),
        Response::Error { .. }
    ));

    let mut c = client(h.addr);
    let _ = c.call(&Request::Hello).unwrap();
    assert!(matches!(
        c.call(&Request::Auth {
            key_id: "x".repeat(10_000),
            response: vec![0u8; 32],
        })
        .unwrap(),
        Response::Error { .. }
    ));

    // The server is still healthy.
    let mut good = Client::connect_authenticated(h.addr, "app1", SECRET).unwrap();
    good.ping().unwrap();
}

/// Every connection gets a distinct nonce.
#[test]
fn each_connection_receives_a_different_challenge() {
    let h = start(true);
    let mut seen = std::collections::HashSet::new();
    for _ in 0..25 {
        let mut c = client(h.addr);
        match c.call(&Request::Hello).unwrap() {
            Response::Challenge { nonce } => {
                assert_eq!(nonce.len(), vectorvolt_net::auth::NONCE_LEN);
                assert!(seen.insert(nonce), "two connections got the same nonce");
            }
            other => panic!("expected a challenge, got {other:?}"),
        }
    }
}
