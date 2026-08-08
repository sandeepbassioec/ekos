//! Phase 17: TableExists integration tests.

use std::net::SocketAddr;
use std::sync::Arc;

use tempfile::TempDir;
use vectorvolt_engine::{EngineOptions, TableEngine, TableRegistry};
use vectorvolt_net::{AuthConfig, Client, Server, ServerConfig};

fn schema() -> Vec<String> {
    vec!["pk".into(), "val".into()]
}

struct TestServer {
    pub addr: SocketAddr,
    _tmp: TempDir,
}

fn start_registry_server() -> (TestServer, Arc<TableRegistry>) {
    let tmp = TempDir::new().unwrap();
    let registry = Arc::new(TableRegistry::new(
        tmp.path().to_path_buf(),
        EngineOptions::default(),
    ));
    let cfg = ServerConfig::new("127.0.0.1:0".parse().unwrap()).with_auth(AuthConfig::disabled());
    let server = Server::bind_registry(Arc::clone(&registry), cfg).unwrap();
    let addr = server.local_addr().unwrap();
    std::thread::spawn(move || {
        let _ = server.serve();
    });
    (TestServer { addr, _tmp: tmp }, registry)
}

fn start_default_server() -> TestServer {
    let tmp = TempDir::new().unwrap();
    let engine =
        Arc::new(TableEngine::open(schema(), tmp.path().join("default"), 1_048_576).unwrap());
    let cfg = ServerConfig::new("127.0.0.1:0".parse().unwrap()).with_auth(AuthConfig::disabled());
    let server = Server::bind(engine, cfg).unwrap();
    let addr = server.local_addr().unwrap();
    std::thread::spawn(move || {
        let _ = server.serve();
    });
    TestServer { addr, _tmp: tmp }
}

fn c(addr: SocketAddr) -> Client {
    Client::connect(addr).unwrap()
}

// ── basic behaviour ───────────────────────────────────────────────────────────

#[test]
fn table_exists_returns_true_for_default_table() {
    let srv = start_default_server();
    let mut cl = c(srv.addr);
    assert!(cl.table_exists("default").unwrap());
}

#[test]
fn table_exists_returns_false_for_unknown_table() {
    let srv = start_default_server();
    let mut cl = c(srv.addr);
    assert!(!cl.table_exists("nonexistent").unwrap());
}

// ── registry-backed server ────────────────────────────────────────────────────

#[test]
fn table_exists_false_before_create() {
    let (srv, _registry) = start_registry_server();
    let mut cl = c(srv.addr);
    assert!(!cl.table_exists("orders").unwrap());
}

#[test]
fn table_exists_true_after_create_table() {
    let (srv, _registry) = start_registry_server();
    let mut cl = c(srv.addr);
    cl.create_table("orders", schema()).unwrap();
    assert!(cl.table_exists("orders").unwrap());
}

#[test]
fn table_exists_false_after_drop_table() {
    let (srv, _registry) = start_registry_server();
    let mut cl = c(srv.addr);
    cl.create_table("products", schema()).unwrap();
    assert!(cl.table_exists("products").unwrap());
    cl.drop_table("products").unwrap();
    assert!(!cl.table_exists("products").unwrap());
}

// ── does not require a selected table ────────────────────────────────────────

#[test]
fn table_exists_works_without_use_table() {
    let (srv, _registry) = start_registry_server();
    let mut cl = c(srv.addr);
    // No use_table called — table_exists must still work.
    let found = cl.table_exists("anything").unwrap();
    assert!(!found);
}

// ── multiple tables ───────────────────────────────────────────────────────────

#[test]
fn table_exists_distinguishes_multiple_tables() {
    let (srv, _registry) = start_registry_server();
    let mut cl = c(srv.addr);
    cl.create_table("a", schema()).unwrap();
    cl.create_table("b", schema()).unwrap();
    assert!(cl.table_exists("a").unwrap());
    assert!(cl.table_exists("b").unwrap());
    assert!(!cl.table_exists("c").unwrap());
}

// ── idempotency ───────────────────────────────────────────────────────────────

#[test]
fn table_exists_is_idempotent() {
    let srv = start_default_server();
    let mut cl = c(srv.addr);
    let first = cl.table_exists("default").unwrap();
    let second = cl.table_exists("default").unwrap();
    assert_eq!(first, second);
    assert!(first);
}
