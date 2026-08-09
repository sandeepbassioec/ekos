//! Layer 5 auth integration tests -- end-to-end challenge-response over a real socket.

use std::sync::Arc;

use tempfile::TempDir;
use vectorvolt_engine::TableEngine;
use vectorvolt_net::{AuthConfig, Client, Scope, Secret, Server, ServerConfig};

fn schema() -> Vec<String> {
    vec!["pk".into(), "score".into()]
}

fn start_server_no_auth() -> (TempDir, Server, std::net::SocketAddr) {
    let tmp = TempDir::new().unwrap();
    let engine = Arc::new(TableEngine::open(schema(), tmp.path(), 1_048_576).unwrap());
    let addr: std::net::SocketAddr = "127.0.0.1:0".parse().unwrap();
    let cfg = ServerConfig::new(addr).with_auth(AuthConfig::disabled());
    let server = Server::bind(engine, cfg).unwrap();
    let local = server.local_addr().unwrap();
    (tmp, server, local)
}

fn start_server_with_auth(auth: AuthConfig) -> (TempDir, Server, std::net::SocketAddr) {
    let tmp = TempDir::new().unwrap();
    let engine = Arc::new(TableEngine::open(schema(), tmp.path(), 1_048_576).unwrap());
    let addr: std::net::SocketAddr = "127.0.0.1:0".parse().unwrap();
    let cfg = ServerConfig::new(addr).with_auth(auth);
    let server = Server::bind(engine, cfg).unwrap();
    let local = server.local_addr().unwrap();
    (tmp, server, local)
}

fn background(server: Server) {
    std::thread::spawn(move || server.serve().unwrap());
}

fn rw_auth() -> AuthConfig {
    let secret = Secret::new(b"correct-horse-battery-staple!!!!!".to_vec()).unwrap();
    AuthConfig::disabled().with_scoped_key("rw", secret, Scope::ReadWrite)
}

fn ro_auth() -> AuthConfig {
    let secret = Secret::new(b"readonly-secret-readonly-secret!!".to_vec()).unwrap();
    AuthConfig::disabled().with_scoped_key("ro", secret, Scope::ReadOnly)
}

fn admin_auth() -> AuthConfig {
    let secret = Secret::new(b"admin-secret-admin-secret-admin!!".to_vec()).unwrap();
    AuthConfig::disabled().with_scoped_key("admin", secret, Scope::Admin)
}

// -- Tests: no auth (auth disabled) -------------------------------------------

#[test]
fn no_auth_server_accepts_any_connection() {
    let (_tmp, server, addr) = start_server_no_auth();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 100]).unwrap();
    assert_eq!(c.get(1).unwrap().unwrap()[1], 100);
}

#[test]
fn no_auth_ping_works_without_hello() {
    let (_tmp, server, addr) = start_server_no_auth();
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.ping().unwrap();
}

// -- Tests: successful authentication -----------------------------------------

#[test]
fn correct_secret_authenticates_and_allows_reads() {
    let (_tmp, server, addr) = start_server_with_auth(ro_auth());
    background(server);
    let mut c =
        Client::connect_authenticated(addr, "ro", b"readonly-secret-readonly-secret!!").unwrap();
    c.ping().unwrap();
    assert!(c.get(99).unwrap().is_none());
}

#[test]
fn correct_rw_secret_allows_writes() {
    let (_tmp, server, addr) = start_server_with_auth(rw_auth());
    background(server);
    let mut c =
        Client::connect_authenticated(addr, "rw", b"correct-horse-battery-staple!!!!!").unwrap();
    c.insert(1, vec![1, 42]).unwrap();
    assert_eq!(c.get(1).unwrap().unwrap()[1], 42);
    c.delete(1).unwrap();
}

#[test]
fn correct_admin_secret_allows_flush_and_compact() {
    let (_tmp, server, addr) = start_server_with_auth(admin_auth());
    background(server);
    let mut c =
        Client::connect_authenticated(addr, "admin", b"admin-secret-admin-secret-admin!!").unwrap();
    c.insert(1, vec![1, 10]).unwrap();
    c.flush().unwrap();
    c.compact().unwrap();
}

#[test]
fn authenticate_twice_on_same_connection_succeeds() {
    let (_tmp, server, addr) = start_server_with_auth(rw_auth());
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.authenticate("rw", b"correct-horse-battery-staple!!!!!")
        .unwrap();
    c.authenticate("rw", b"correct-horse-battery-staple!!!!!")
        .unwrap();
    c.ping().unwrap();
}

// -- Tests: unauthenticated rejection -----------------------------------------

#[test]
fn unauthenticated_get_is_rejected_when_auth_enabled() {
    let (_tmp, server, addr) = start_server_with_auth(rw_auth());
    background(server);
    let mut c = Client::connect(addr).unwrap();
    let err = c.get(1).unwrap_err();
    let msg = err.to_string();
    assert!(
        msg.contains("authenticate") || msg.contains("unauthenticated") || msg.contains("Auth"),
        "unexpected error: {msg}"
    );
}

#[test]
fn ping_allowed_before_auth_on_auth_enabled_server() {
    let (_tmp, server, addr) = start_server_with_auth(rw_auth());
    background(server);
    let mut c = Client::connect(addr).unwrap();
    c.ping().unwrap();
}

// -- Tests: wrong secret / unknown key ----------------------------------------

#[test]
fn wrong_secret_is_rejected() {
    let (_tmp, server, addr) = start_server_with_auth(rw_auth());
    background(server);
    let result = Client::connect_authenticated(addr, "rw", b"this-is-not-the-right-secret!!!");
    let Err(e) = result else {
        panic!("expected auth error but connect succeeded")
    };
    let msg = e.to_string();
    assert!(
        msg.contains("response")
            || msg.contains("auth")
            || msg.contains("Auth")
            || msg.contains("failed"),
        "unexpected error: {msg}"
    );
}

#[test]
fn unknown_key_id_gives_same_error_class_as_wrong_secret() {
    let (_tmp, server, addr) = start_server_with_auth(rw_auth());
    background(server);
    let result =
        Client::connect_authenticated(addr, "nosuchkey", b"correct-horse-battery-staple!!!!!");
    let Err(e) = result else {
        panic!("expected auth error but connect succeeded")
    };
    let msg = e.to_string();
    assert!(
        msg.contains("response")
            || msg.contains("auth")
            || msg.contains("Auth")
            || msg.contains("failed"),
        "unexpected error: {msg}"
    );
}

// -- Tests: scope enforcement -------------------------------------------------

#[test]
fn read_only_key_cannot_insert() {
    let (_tmp, server, addr) = start_server_with_auth(ro_auth());
    background(server);
    let mut c =
        Client::connect_authenticated(addr, "ro", b"readonly-secret-readonly-secret!!").unwrap();
    let err = c.insert(1, vec![1, 99]).unwrap_err();
    let msg = err.to_string();
    assert!(
        msg.contains("read-write") || msg.contains("scope") || msg.contains("Insufficient"),
        "expected scope error, got: {msg}"
    );
}

#[test]
fn read_only_key_cannot_delete() {
    let (_tmp, server, addr) = start_server_with_auth(ro_auth());
    background(server);
    let mut c =
        Client::connect_authenticated(addr, "ro", b"readonly-secret-readonly-secret!!").unwrap();
    let err = c.delete(1).unwrap_err();
    let msg = err.to_string();
    assert!(
        msg.contains("read-write") || msg.contains("scope") || msg.contains("Insufficient"),
        "expected scope error, got: {msg}"
    );
}

#[test]
fn read_only_key_can_scan_and_count() {
    let (_tmp, server, addr) = start_server_with_auth(ro_auth());
    background(server);
    let mut c =
        Client::connect_authenticated(addr, "ro", b"readonly-secret-readonly-secret!!").unwrap();
    let rows = c.range_scan(0, 0, u64::MAX, 0).unwrap();
    assert!(rows.is_empty());
    let count = c.count(0, 0, u64::MAX).unwrap();
    assert_eq!(count, 0);
}

#[test]
fn read_write_key_cannot_flush() {
    let (_tmp, server, addr) = start_server_with_auth(rw_auth());
    background(server);
    let mut c =
        Client::connect_authenticated(addr, "rw", b"correct-horse-battery-staple!!!!!").unwrap();
    let err = c.flush().unwrap_err();
    let msg = err.to_string();
    assert!(
        msg.contains("admin") || msg.contains("scope") || msg.contains("Insufficient"),
        "expected scope error on flush, got: {msg}"
    );
}

#[test]
fn read_write_key_cannot_compact() {
    let (_tmp, server, addr) = start_server_with_auth(rw_auth());
    background(server);
    let mut c =
        Client::connect_authenticated(addr, "rw", b"correct-horse-battery-staple!!!!!").unwrap();
    let err = c.compact().unwrap_err();
    let msg = err.to_string();
    assert!(
        msg.contains("admin") || msg.contains("scope") || msg.contains("Insufficient"),
        "expected scope error on compact, got: {msg}"
    );
}

#[test]
fn read_write_key_cannot_truncate() {
    let (_tmp, server, addr) = start_server_with_auth(rw_auth());
    background(server);
    let mut c =
        Client::connect_authenticated(addr, "rw", b"correct-horse-battery-staple!!!!!").unwrap();
    let err = c.truncate().unwrap_err();
    let msg = err.to_string();
    assert!(
        msg.contains("admin") || msg.contains("scope") || msg.contains("Insufficient"),
        "expected scope error on truncate, got: {msg}"
    );
}

#[test]
fn admin_key_can_read_write_and_admin_ops() {
    let (_tmp, server, addr) = start_server_with_auth(admin_auth());
    background(server);
    let mut c =
        Client::connect_authenticated(addr, "admin", b"admin-secret-admin-secret-admin!!").unwrap();
    c.insert(1, vec![1, 77]).unwrap();
    assert_eq!(c.get(1).unwrap().unwrap()[1], 77);
    c.flush().unwrap();
    c.compact().unwrap();
    c.truncate().unwrap();
    assert!(c.get(1).unwrap().is_none());
}

// -- Tests: multi-key config with different scopes ----------------------------

#[test]
fn multiple_keys_with_different_scopes_enforced_independently() {
    let ro_secret = Secret::new(b"readonly-secret-readonly-secret!!".to_vec()).unwrap();
    let rw_secret = Secret::new(b"correct-horse-battery-staple!!!!!".to_vec()).unwrap();
    let admin_secret = Secret::new(b"admin-secret-admin-secret-admin!!".to_vec()).unwrap();
    let auth = AuthConfig::disabled()
        .with_scoped_key("ro", ro_secret, Scope::ReadOnly)
        .with_scoped_key("rw", rw_secret, Scope::ReadWrite)
        .with_scoped_key("admin", admin_secret, Scope::Admin);

    let tmp = TempDir::new().unwrap();
    let engine = Arc::new(TableEngine::open(schema(), tmp.path(), 1_048_576).unwrap());
    let addr: std::net::SocketAddr = "127.0.0.1:0".parse().unwrap();
    let server = Server::bind(engine, ServerConfig::new(addr).with_auth(auth)).unwrap();
    let local = server.local_addr().unwrap();
    background(server);

    let mut ro =
        Client::connect_authenticated(local, "ro", b"readonly-secret-readonly-secret!!").unwrap();
    assert!(ro.get(1).unwrap().is_none());
    assert!(ro.insert(1, vec![1, 1]).is_err());

    let mut rw =
        Client::connect_authenticated(local, "rw", b"correct-horse-battery-staple!!!!!").unwrap();
    rw.insert(1, vec![1, 10]).unwrap();
    assert!(rw.flush().is_err());

    let mut admin =
        Client::connect_authenticated(local, "admin", b"admin-secret-admin-secret-admin!!")
            .unwrap();
    admin.flush().unwrap();
}

// -- Tests: lockout after repeated failures -----------------------------------

#[test]
fn three_bad_attempts_lock_out_the_connection() {
    use vectorvolt_net::auth::MAX_AUTH_ATTEMPTS;

    let (_tmp, server, addr) = start_server_with_auth(rw_auth());
    background(server);

    let mut c = Client::connect(addr).unwrap();
    for _ in 0..MAX_AUTH_ATTEMPTS {
        let _ = c.authenticate("rw", b"wrong-password-wrong-password!!!!");
    }
    // After MAX_AUTH_ATTEMPTS failures the server closes the connection.
    // Any further operation must return an error (I/O or auth).
    let data_result = c.get(1);
    assert!(
        data_result.is_err(),
        "expected error after lockout but got ok"
    );
}
