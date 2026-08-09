//! Layer 7 control-plane integration tests.
//!
//! All tests run over a real TCP socket against a multi-table server built with
//! `Server::bind_registry`. They cover:
//!
//! - `create_table` / `drop_table` / `list_tables` lifecycle
//! - `use_table` and `table_exists` routing
//! - `describe_table` returns the correct schema
//! - Isolation between tables (inserts to one table are invisible from another)
//! - Auth scope: create/drop require Admin, list/describe/use are ReadOnly
//! - Error paths: duplicate create, drop non-existent, use non-existent

use std::sync::Arc;

use tempfile::TempDir;
use vectorvolt_engine::EngineOptions;
use vectorvolt_net::{AuthConfig, Client, Scope, Secret, Server, ServerConfig, TableRegistry};

fn bind_registry(registry: Arc<TableRegistry>) -> (Server, std::net::SocketAddr) {
    let addr: std::net::SocketAddr = "127.0.0.1:0".parse().unwrap();
    let cfg = ServerConfig::new(addr);
    let server = Server::bind_registry(registry, cfg).unwrap();
    let local = server.local_addr().unwrap();
    (server, local)
}

fn background(server: Server) {
    std::thread::spawn(move || server.serve().unwrap());
}

fn opts() -> EngineOptions {
    EngineOptions::default()
}

fn fresh_registry(tmp: &TempDir) -> Arc<TableRegistry> {
    Arc::new(TableRegistry::open(tmp.path(), opts()).unwrap())
}

// ── create / list / drop lifecycle ───────────────────────────────────────────

#[test]
fn create_table_makes_it_visible_in_list() {
    let tmp = TempDir::new().unwrap();
    let reg = fresh_registry(&tmp);
    let (server, addr) = bind_registry(reg);
    background(server);

    let mut c = Client::connect(addr).unwrap();
    c.create_table("events", vec!["id".into(), "ts".into()])
        .unwrap();

    let tables = c.list_tables().unwrap();
    assert!(
        tables.contains(&"events".to_string()),
        "events not listed: {tables:?}"
    );
}

#[test]
fn drop_table_removes_it_from_list() {
    let tmp = TempDir::new().unwrap();
    let reg = fresh_registry(&tmp);
    let (server, addr) = bind_registry(reg);
    background(server);

    let mut c = Client::connect(addr).unwrap();
    c.create_table("tmp_table", vec!["id".into()]).unwrap();
    assert!(c.list_tables().unwrap().contains(&"tmp_table".to_string()));

    c.drop_table("tmp_table").unwrap();
    assert!(
        !c.list_tables().unwrap().contains(&"tmp_table".to_string()),
        "dropped table still listed"
    );
}

#[test]
fn list_tables_returns_empty_on_fresh_registry() {
    let tmp = TempDir::new().unwrap();
    let reg = fresh_registry(&tmp);
    let (server, addr) = bind_registry(reg);
    background(server);

    let mut c = Client::connect(addr).unwrap();
    let tables = c.list_tables().unwrap();
    assert!(tables.is_empty(), "expected empty list, got: {tables:?}");
}

#[test]
fn multiple_tables_all_appear_in_list() {
    let tmp = TempDir::new().unwrap();
    let reg = fresh_registry(&tmp);
    let (server, addr) = bind_registry(reg);
    background(server);

    let mut c = Client::connect(addr).unwrap();
    for name in ["alpha", "beta", "gamma"] {
        c.create_table(name, vec!["id".into(), "val".into()])
            .unwrap();
    }

    let mut tables = c.list_tables().unwrap();
    tables.sort();
    assert_eq!(tables, vec!["alpha", "beta", "gamma"]);
}

// ── use_table / table_exists ──────────────────────────────────────────────────

#[test]
fn use_table_selects_an_existing_table() {
    let tmp = TempDir::new().unwrap();
    let reg = fresh_registry(&tmp);
    let (server, addr) = bind_registry(Arc::clone(&reg));
    background(server);

    reg.create_table("orders", vec!["id".into(), "amount".into()])
        .unwrap();

    let mut c = Client::connect(addr).unwrap();
    c.use_table("orders").unwrap();
    c.insert(1, vec![1, 500]).unwrap();
    assert_eq!(c.get(1).unwrap().unwrap()[1], 500);
}

#[test]
fn table_exists_true_for_created_table() {
    let tmp = TempDir::new().unwrap();
    let reg = fresh_registry(&tmp);
    let (server, addr) = bind_registry(reg);
    background(server);

    let mut c = Client::connect(addr).unwrap();
    c.create_table("things", vec!["id".into()]).unwrap();
    assert!(c.table_exists("things").unwrap());
}

#[test]
fn table_exists_false_for_unknown_table() {
    let tmp = TempDir::new().unwrap();
    let reg = fresh_registry(&tmp);
    let (server, addr) = bind_registry(reg);
    background(server);

    let mut c = Client::connect(addr).unwrap();
    assert!(!c.table_exists("no_such_table").unwrap());
}

// ── describe_table ────────────────────────────────────────────────────────────

#[test]
fn describe_table_returns_schema_in_order() {
    let tmp = TempDir::new().unwrap();
    let reg = fresh_registry(&tmp);
    let (server, addr) = bind_registry(reg);
    background(server);

    let mut c = Client::connect(addr).unwrap();
    c.create_table(
        "prices",
        vec!["id".into(), "ask".into(), "bid".into(), "vol".into()],
    )
    .unwrap();

    let schema = c.describe_table().unwrap();
    assert_eq!(schema, vec!["id", "ask", "bid", "vol"]);
}

// ── table isolation ───────────────────────────────────────────────────────────

#[test]
fn rows_in_one_table_are_invisible_from_another() {
    let tmp = TempDir::new().unwrap();
    let reg = fresh_registry(&tmp);
    let (server, addr) = bind_registry(reg);
    background(server);

    let mut c = Client::connect(addr).unwrap();
    c.create_table("a", vec!["id".into(), "v".into()]).unwrap();
    c.insert(42, vec![42, 999]).unwrap();

    c.create_table("b", vec!["id".into(), "v".into()]).unwrap();
    // Now selected table is b.
    assert!(
        c.get(42).unwrap().is_none(),
        "row from table a visible in table b"
    );
}

#[test]
fn two_tables_store_rows_independently() {
    let tmp = TempDir::new().unwrap();
    let reg = fresh_registry(&tmp);
    let (server, addr) = bind_registry(reg);
    background(server);

    let mut c = Client::connect(addr).unwrap();

    c.create_table("t1", vec!["id".into(), "x".into()]).unwrap();
    c.insert(1, vec![1, 100]).unwrap();

    c.create_table("t2", vec!["id".into(), "x".into()]).unwrap();
    c.insert(1, vec![1, 200]).unwrap();

    // t2: should be 200.
    assert_eq!(c.get(1).unwrap().unwrap()[1], 200);

    // Switch back to t1: should be 100.
    c.use_table("t1").unwrap();
    assert_eq!(c.get(1).unwrap().unwrap()[1], 100);
}

// ── error paths ───────────────────────────────────────────────────────────────

#[test]
fn duplicate_create_returns_error() {
    let tmp = TempDir::new().unwrap();
    let reg = fresh_registry(&tmp);
    let (server, addr) = bind_registry(reg);
    background(server);

    let mut c = Client::connect(addr).unwrap();
    c.create_table("dup", vec!["id".into()]).unwrap();
    let err = c.create_table("dup", vec!["id".into()]).unwrap_err();
    let msg = err.to_string();
    assert!(
        msg.contains("already exists") || msg.contains("TABLE_EXISTS"),
        "unexpected error: {msg}"
    );
}

#[test]
fn drop_nonexistent_table_returns_error() {
    let tmp = TempDir::new().unwrap();
    let reg = fresh_registry(&tmp);
    let (server, addr) = bind_registry(reg);
    background(server);

    let mut c = Client::connect(addr).unwrap();
    let err = c.drop_table("ghost").unwrap_err();
    let msg = err.to_string();
    assert!(
        msg.contains("not found") || msg.contains("TABLE_NOT_FOUND"),
        "unexpected error: {msg}"
    );
}

#[test]
fn use_nonexistent_table_returns_error() {
    let tmp = TempDir::new().unwrap();
    let reg = fresh_registry(&tmp);
    let (server, addr) = bind_registry(reg);
    background(server);

    let mut c = Client::connect(addr).unwrap();
    let err = c.use_table("no_such").unwrap_err();
    let msg = err.to_string();
    assert!(
        msg.contains("not found") || msg.contains("TABLE_NOT_FOUND"),
        "unexpected error: {msg}"
    );
}

// ── registry persistence across connections ───────────────────────────────────

#[test]
fn tables_created_on_one_connection_visible_to_another() {
    let tmp = TempDir::new().unwrap();
    let reg = fresh_registry(&tmp);
    let (server, addr) = bind_registry(reg);
    background(server);

    let mut c1 = Client::connect(addr).unwrap();
    c1.create_table("shared", vec!["id".into(), "v".into()])
        .unwrap();
    c1.insert(7, vec![7, 77]).unwrap();
    drop(c1);

    let mut c2 = Client::connect(addr).unwrap();
    c2.use_table("shared").unwrap();
    assert_eq!(c2.get(7).unwrap().unwrap()[1], 77);
}

// ── auth scope enforcement for control plane ─────────────────────────────────

#[test]
fn read_only_key_cannot_create_table() {
    let tmp = TempDir::new().unwrap();
    let reg = fresh_registry(&tmp);
    let secret = Secret::new(b"readonly-secret-readonly-secret!!".to_vec()).unwrap();
    let auth = AuthConfig::disabled().with_scoped_key("ro", secret, Scope::ReadOnly);
    let addr: std::net::SocketAddr = "127.0.0.1:0".parse().unwrap();
    let server = Server::bind_registry(reg, ServerConfig::new(addr).with_auth(auth)).unwrap();
    let local = server.local_addr().unwrap();
    background(server);

    let mut c =
        Client::connect_authenticated(local, "ro", b"readonly-secret-readonly-secret!!").unwrap();
    let err = c.create_table("x", vec!["id".into()]).unwrap_err();
    let msg = err.to_string();
    assert!(
        msg.contains("admin") || msg.contains("scope") || msg.contains("Insufficient"),
        "expected scope error, got: {msg}"
    );
}

#[test]
fn read_only_key_cannot_drop_table() {
    let tmp = TempDir::new().unwrap();
    let reg = Arc::new(TableRegistry::open(tmp.path(), opts()).unwrap());
    reg.create_table("victim", vec!["id".into()]).unwrap();

    let secret = Secret::new(b"readonly-secret-readonly-secret!!".to_vec()).unwrap();
    let auth = AuthConfig::disabled().with_scoped_key("ro", secret, Scope::ReadOnly);
    let addr: std::net::SocketAddr = "127.0.0.1:0".parse().unwrap();
    let server = Server::bind_registry(reg, ServerConfig::new(addr).with_auth(auth)).unwrap();
    let local = server.local_addr().unwrap();
    background(server);

    let mut c =
        Client::connect_authenticated(local, "ro", b"readonly-secret-readonly-secret!!").unwrap();
    let err = c.drop_table("victim").unwrap_err();
    let msg = err.to_string();
    assert!(
        msg.contains("admin") || msg.contains("scope") || msg.contains("Insufficient"),
        "expected scope error, got: {msg}"
    );
}

#[test]
fn admin_key_can_create_and_drop_table() {
    let tmp = TempDir::new().unwrap();
    let reg = fresh_registry(&tmp);
    let secret = Secret::new(b"admin-secret-admin-secret-admin!!".to_vec()).unwrap();
    let auth = AuthConfig::disabled().with_scoped_key("admin", secret, Scope::Admin);
    let addr: std::net::SocketAddr = "127.0.0.1:0".parse().unwrap();
    let server = Server::bind_registry(reg, ServerConfig::new(addr).with_auth(auth)).unwrap();
    let local = server.local_addr().unwrap();
    background(server);

    let mut c = Client::connect_authenticated(local, "admin", b"admin-secret-admin-secret-admin!!")
        .unwrap();

    c.create_table("managed", vec!["id".into(), "v".into()])
        .unwrap();
    assert!(c.table_exists("managed").unwrap());

    c.drop_table("managed").unwrap();
    assert!(!c.table_exists("managed").unwrap());
}
