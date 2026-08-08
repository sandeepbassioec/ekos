//! Phase 16: DescribeTable integration tests.

use std::sync::Arc;

use tempfile::TempDir;
use vectorvolt_engine::TableEngine;
use vectorvolt_net::{AuthConfig, Client, Server, ServerConfig};

fn start_server_with_schema(schema: Vec<String>) -> (TempDir, Server, std::net::SocketAddr) {
    let tmp = TempDir::new().unwrap();
    let engine = Arc::new(TableEngine::open(schema, tmp.path(), 1_048_576).unwrap());
    let addr: std::net::SocketAddr = "127.0.0.1:0".parse().unwrap();
    let cfg = ServerConfig::new(addr).with_auth(AuthConfig::disabled());
    let server = Server::bind(engine, cfg).unwrap();
    let local = server.local_addr().unwrap();
    (tmp, server, local)
}

fn background(server: Server) {
    std::thread::spawn(move || server.serve().unwrap());
}

// ── basic behaviour ───────────────────────────────────────────────────────────

#[test]
fn describe_table_returns_schema_columns() {
    let schema = vec!["pk".into(), "name".into(), "score".into()];
    let (_tmp, server, addr) = start_server_with_schema(schema.clone());
    background(server);

    let mut c = Client::connect(addr).unwrap();
    let columns = c.describe_table().unwrap();
    assert_eq!(columns, schema);
}

#[test]
fn describe_table_single_column() {
    let schema = vec!["id".into()];
    let (_tmp, server, addr) = start_server_with_schema(schema.clone());
    background(server);

    let mut c = Client::connect(addr).unwrap();
    let columns = c.describe_table().unwrap();
    assert_eq!(columns, schema);
}

#[test]
fn describe_table_many_columns() {
    let schema: Vec<String> = (0..20).map(|i| format!("col_{i}")).collect();
    let (_tmp, server, addr) = start_server_with_schema(schema.clone());
    background(server);

    let mut c = Client::connect(addr).unwrap();
    let columns = c.describe_table().unwrap();
    assert_eq!(columns, schema);
}

// ── column order ──────────────────────────────────────────────────────────────

#[test]
fn describe_table_preserves_column_order() {
    let schema = vec!["z".into(), "a".into(), "m".into(), "pk".into()];
    let (_tmp, server, addr) = start_server_with_schema(schema.clone());
    background(server);

    let mut c = Client::connect(addr).unwrap();
    let columns = c.describe_table().unwrap();
    assert_eq!(columns[0], "z");
    assert_eq!(columns[1], "a");
    assert_eq!(columns[2], "m");
    assert_eq!(columns[3], "pk");
}

// ── idempotency ───────────────────────────────────────────────────────────────

#[test]
fn describe_table_is_idempotent() {
    let schema = vec!["pk".into(), "value".into()];
    let (_tmp, server, addr) = start_server_with_schema(schema.clone());
    background(server);

    let mut c = Client::connect(addr).unwrap();
    let first = c.describe_table().unwrap();
    let second = c.describe_table().unwrap();
    assert_eq!(first, second);
    assert_eq!(first, schema);
}

// ── interop: schema matches inserted data ─────────────────────────────────────

#[test]
fn describe_table_column_count_matches_row_width() {
    let schema = vec!["pk".into(), "a".into(), "b".into()];
    let (_tmp, server, addr) = start_server_with_schema(schema);
    background(server);

    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 10, 20]).unwrap();
    let row = c.get(1).unwrap().unwrap();

    let columns = c.describe_table().unwrap();
    // get() returns all columns (pk + non-pk), so row width matches schema width
    assert_eq!(row.len(), columns.len());
}

// ── unicode names ─────────────────────────────────────────────────────────────

#[test]
fn describe_table_with_unicode_column_names() {
    let schema = vec!["pk".into(), "名前".into(), "スコア".into()];
    let (_tmp, server, addr) = start_server_with_schema(schema.clone());
    background(server);

    let mut c = Client::connect(addr).unwrap();
    let columns = c.describe_table().unwrap();
    assert_eq!(columns, schema);
}

// ── describe after data ops ───────────────────────────────────────────────────

#[test]
fn describe_table_unchanged_after_inserts_and_deletes() {
    let schema = vec!["pk".into(), "val".into()];
    let (_tmp, server, addr) = start_server_with_schema(schema.clone());
    background(server);

    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 100]).unwrap();
    c.insert(2, vec![2, 200]).unwrap();
    c.delete(1).unwrap();

    let columns = c.describe_table().unwrap();
    assert_eq!(columns, schema);
}
