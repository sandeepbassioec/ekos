//! Phase 5: multi-table catalog integration tests.
//!
//! Each test spins up a real server over loopback, connects with a real client,
//! and exercises the UseTable / CreateTable / DropTable / ListTables flow end-to-end.

use std::net::SocketAddr;
use std::sync::Arc;

use tempfile::TempDir;
use vectorvolt_engine::{EngineOptions, TableEngine, TableRegistry};
use vectorvolt_net::{Client, Server, ServerConfig};

// ── Test helpers ──────────────────────────────────────────────────────────────

fn schema() -> Vec<String> {
    vec!["id".into(), "val".into()]
}

struct TestServer {
    addr: SocketAddr,
    _shutdown: vectorvolt_net::server::ShutdownHandle,
    _tmp: TempDir,
}

impl TestServer {
    /// Spin up a server with a single `"default"` table (backward-compat path).
    fn with_default() -> Self {
        let tmp = TempDir::new().unwrap();
        let engine =
            Arc::new(TableEngine::open(schema(), tmp.path().join("default"), 1_000_000).unwrap());
        let cfg = ServerConfig::new("127.0.0.1:0".parse().unwrap());
        let server = Server::bind(engine, cfg).unwrap();
        let addr = server.local_addr().unwrap();
        let shutdown = server.shutdown_handle();
        std::thread::spawn(move || {
            let _ = server.serve();
        });
        TestServer {
            addr,
            _shutdown: shutdown,
            _tmp: tmp,
        }
    }

    /// Spin up a server with an EMPTY registry (no default table).
    fn empty_registry() -> (Self, Arc<TableRegistry>) {
        let tmp = TempDir::new().unwrap();
        let base = tmp.path().to_path_buf();
        let registry = Arc::new(TableRegistry::new(base, EngineOptions::default()));
        let cfg = ServerConfig::new("127.0.0.1:0".parse().unwrap());
        let server = Server::bind_registry(Arc::clone(&registry), cfg).unwrap();
        let addr = server.local_addr().unwrap();
        let shutdown = server.shutdown_handle();
        std::thread::spawn(move || {
            let _ = server.serve();
        });
        (
            TestServer {
                addr,
                _shutdown: shutdown,
                _tmp: tmp,
            },
            registry,
        )
    }
}

fn client(addr: SocketAddr) -> Client {
    Client::connect_timeout(addr, std::time::Duration::from_secs(5)).unwrap()
}

// ── Tests ─────────────────────────────────────────────────────────────────────

/// list_tables on a server with one default table returns ["default"].
#[test]
fn list_tables_returns_default() {
    let srv = TestServer::with_default();
    let mut c = client(srv.addr);
    let names = c.list_tables().unwrap();
    assert_eq!(names, vec!["default".to_string()]);
}

/// Backward compat: data ops work without an explicit use_table on the default server.
#[test]
fn backward_compat_insert_get_without_use_table() {
    let srv = TestServer::with_default();
    let mut c = client(srv.addr);
    c.insert(1, vec![1, 42]).unwrap();
    let row = c.get(1).unwrap().unwrap();
    assert_eq!(row, vec![1, 42]);
}

/// use_table switches the active table; subsequent ops target the new table.
#[test]
fn use_table_selects_existing_table() {
    let srv = TestServer::with_default();
    let mut c = client(srv.addr);
    c.use_table("default").unwrap();
    c.insert(99, vec![99, 7]).unwrap();
    assert_eq!(c.get(99).unwrap().unwrap(), vec![99, 7]);
}

/// use_table on a non-existent name returns an error with TABLE_NOT_FOUND code.
#[test]
fn use_table_missing_returns_error() {
    let srv = TestServer::with_default();
    let mut c = client(srv.addr);
    let err = c.use_table("ghost").unwrap_err();
    let msg = err.to_string();
    assert!(
        msg.contains("10") || msg.contains("not found"),
        "unexpected: {msg}"
    );
}

/// create_table creates a new table and auto-selects it.
#[test]
fn create_table_and_insert() {
    let (srv, _reg) = TestServer::empty_registry();
    let mut c = client(srv.addr);
    c.create_table("users", vec!["id".into(), "age".into()])
        .unwrap();
    // Should be selected immediately after creation.
    c.insert(1, vec![1, 30]).unwrap();
    assert_eq!(c.get(1).unwrap().unwrap(), vec![1, 30]);
}

/// create_table twice with the same name returns TABLE_EXISTS error.
#[test]
fn create_table_duplicate_returns_error() {
    let (srv, _reg) = TestServer::empty_registry();
    let mut c = client(srv.addr);
    c.create_table("things", vec!["id".into(), "v".into()])
        .unwrap();
    let err = c
        .create_table("things", vec!["id".into(), "v".into()])
        .unwrap_err();
    let msg = err.to_string();
    assert!(
        msg.contains("11") || msg.contains("already exists"),
        "unexpected: {msg}"
    );
}

/// list_tables returns all created tables.
#[test]
fn list_tables_shows_all_created() {
    let (srv, _reg) = TestServer::empty_registry();
    let mut c = client(srv.addr);
    c.create_table("alpha", schema()).unwrap();
    c.create_table("beta", schema()).unwrap();
    let mut names = c.list_tables().unwrap();
    names.sort();
    assert_eq!(names, vec!["alpha", "beta"]);
}

/// drop_table removes the table; subsequent list_tables no longer shows it.
#[test]
fn drop_table_removes_from_list() {
    let (srv, _reg) = TestServer::empty_registry();
    let mut c = client(srv.addr);
    c.create_table("temp", schema()).unwrap();
    let before = c.list_tables().unwrap();
    assert!(before.contains(&"temp".to_string()));

    c.drop_table("temp").unwrap();
    let after = c.list_tables().unwrap();
    assert!(!after.contains(&"temp".to_string()));
}

/// After drop_table, data ops return NOT_SELECTED (the active table was cleared).
#[test]
fn data_op_after_drop_returns_not_selected() {
    let (srv, _reg) = TestServer::empty_registry();
    let mut c = client(srv.addr);
    c.create_table("t", schema()).unwrap();
    c.insert(1, vec![1, 1]).unwrap();
    c.drop_table("t").unwrap();
    let err = c.insert(2, vec![2, 2]).unwrap_err();
    let msg = err.to_string();
    assert!(
        msg.contains("9") || msg.contains("no table selected") || msg.contains("UseTable"),
        "unexpected: {msg}"
    );
}

/// drop_table on a non-existent name returns TABLE_NOT_FOUND.
#[test]
fn drop_table_missing_returns_error() {
    let (srv, _reg) = TestServer::empty_registry();
    let mut c = client(srv.addr);
    let err = c.drop_table("nope").unwrap_err();
    let msg = err.to_string();
    assert!(
        msg.contains("10") || msg.contains("not found"),
        "unexpected: {msg}"
    );
}

/// Data ops without any prior use_table / create_table return NOT_SELECTED.
#[test]
fn data_op_without_table_selected_returns_not_selected() {
    let (srv, _reg) = TestServer::empty_registry();
    let mut c = client(srv.addr);
    let err = c.get(1).unwrap_err();
    let msg = err.to_string();
    assert!(
        msg.contains("9") || msg.contains("no table selected") || msg.contains("UseTable"),
        "unexpected: {msg}"
    );
}

/// Two connections each select a different table; they do not interfere.
#[test]
fn two_connections_independent_table_selection() {
    let (srv, _reg) = TestServer::empty_registry();
    let mut c1 = client(srv.addr);
    let mut c2 = client(srv.addr);

    c1.create_table("a", schema()).unwrap();
    c2.use_table("a").unwrap();

    c1.insert(1, vec![1, 10]).unwrap();
    c2.insert(2, vec![2, 20]).unwrap();

    // c1 still sees row 1 (it was still on "a" — auto-selected after create).
    assert_eq!(c1.get(1).unwrap().unwrap(), vec![1, 10]);
    // c2 sees row 2.
    assert_eq!(c2.get(2).unwrap().unwrap(), vec![2, 20]);
    // c1 can also read row 2 (same table).
    assert_eq!(c1.get(2).unwrap().unwrap(), vec![2, 20]);
}
