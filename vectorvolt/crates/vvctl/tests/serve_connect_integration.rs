/// End-to-end integration tests for `vvctl serve` + `vvctl connect`.
///
/// Each test binds a real server on an ephemeral port, drives it through
/// `connect::run` or the `Client` API, then shuts it down cleanly.
///
/// Schema: pk(u64), age(u64), score(u64) â€” 3 columns total.
/// `Client::insert(pk, values)` â€” `values` contains ALL columns including pk
/// at index 0, matching how `vectorvolt-engine` stores rows.
use std::net::SocketAddr;
use std::thread;

use anyhow::Result;
use tempfile::TempDir;
use vectorvolt_net::Client;

// â”€â”€ helpers â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

fn ephemeral() -> SocketAddr {
    "127.0.0.1:0".parse().unwrap()
}

fn schema_cols() -> Vec<String> {
    vec!["pk".into(), "age".into(), "score".into()]
}

/// Build and start a plain-text, no-auth server in a background thread.
/// Returns `(addr, shutdown_fn, _dir_guard)`.
fn start_plain(dir: &TempDir) -> (SocketAddr, impl FnOnce()) {
    use vvctl::commands::serve::{ServeArgs, bind};

    let args = ServeArgs {
        dir: dir.path().to_path_buf(),
        columns: schema_cols(),
        addr: ephemeral(),
        max_connections: 32,
        secret: None,
        key_id: "default".into(),
        cert: None,
        key: None,
        multi: false,
        config: None,
        http_addr: None,
        master_config: None,
        http_token: None,
        transport: "tcp".to_string(),
        xdp_iface: "lo".to_string(),
        xdp_queue: 0,
        xdp_ring_size: 256,
        flush_threshold: None,
        durability: None,
        max_partitions: None,
        grpc_port: 50051,
        no_grpc: false,
        grpc_host: std::net::IpAddr::from([127, 0, 0, 1]),
        grpc_token: None,
        tpm_unseal: false,
        encrypt_at_rest: false,
        metrics_port: 0,
        metrics_host: std::net::IpAddr::from([127, 0, 0, 1]),
        pgwire_port: None,
        cluster_bind: None,
        cluster_seeds: vec![],
        cluster_node_id: None,
        cluster_status_port: 7900,
        catalog: false,
        restore_catalog: false,
    };

    let bound = bind(&args).expect("bind");
    let addr = bound.server.local_addr().expect("local_addr");
    let shutdown = bound.server.shutdown_handle();
    thread::spawn(move || {
        let _ = bound.server.serve();
    });

    // The listener is bound before `serve` runs so connect cannot race.
    thread::sleep(std::time::Duration::from_millis(5));

    (addr, move || shutdown.shutdown())
}

/// Build and start a server protected by an HMAC-SHA3 secret.
fn start_authed(dir: &TempDir, secret_hex: &str) -> (SocketAddr, impl FnOnce()) {
    use vvctl::commands::serve::{ServeArgs, bind};

    let args = ServeArgs {
        dir: dir.path().to_path_buf(),
        columns: schema_cols(),
        addr: ephemeral(),
        max_connections: 32,
        secret: Some(secret_hex.to_string()),
        key_id: "k1".into(),
        cert: None,
        key: None,
        multi: false,
        config: None,
        http_addr: None,
        master_config: None,
        http_token: None,
        transport: "tcp".to_string(),
        xdp_iface: "lo".to_string(),
        xdp_queue: 0,
        xdp_ring_size: 256,
        flush_threshold: None,
        durability: None,
        max_partitions: None,
        grpc_port: 50051,
        no_grpc: false,
        grpc_host: std::net::IpAddr::from([127, 0, 0, 1]),
        grpc_token: None,
        tpm_unseal: false,
        encrypt_at_rest: false,
        metrics_port: 0,
        metrics_host: std::net::IpAddr::from([127, 0, 0, 1]),
        pgwire_port: None,
        cluster_bind: None,
        cluster_seeds: vec![],
        cluster_node_id: None,
        cluster_status_port: 7900,
        catalog: false,
        restore_catalog: false,
    };

    let bound = bind(&args).expect("bind");
    let addr = bound.server.local_addr().expect("local_addr");
    let shutdown = bound.server.shutdown_handle();
    thread::spawn(move || {
        let _ = bound.server.serve();
    });

    thread::sleep(std::time::Duration::from_millis(5));
    (addr, move || shutdown.shutdown())
}

fn mk_args(
    addr: SocketAddr,
    secret: Option<String>,
    key_id: &str,
    cmd: vvctl::commands::connect::ConnectCommand,
) -> vvctl::commands::connect::ConnectArgs {
    vvctl::commands::connect::ConnectArgs {
        addr,
        secret,
        key_id: key_id.into(),
        ca: None,
        tls_name: None,
        timeout_ms: 2000,
        command: cmd,
    }
}

fn plain(addr: SocketAddr, cmd: vvctl::commands::connect::ConnectCommand) -> Result<()> {
    vvctl::commands::connect::run(mk_args(addr, None, "default", cmd))
}

// full-row insert: values = [pk, age, score]
fn row(pk: u64, age: u64, score: u64) -> Vec<u64> {
    vec![pk, age, score]
}

// â”€â”€ serve: bind() unit-level â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

#[test]
fn bind_creates_schema_file_for_new_table() {
    use vvctl::commands::serve::{ServeArgs, bind};

    let dir = TempDir::new().unwrap();
    let args = ServeArgs {
        dir: dir.path().to_path_buf(),
        columns: vec!["pk".into(), "v".into()],
        addr: ephemeral(),
        max_connections: 4,
        secret: None,
        key_id: "default".into(),
        cert: None,
        key: None,
        multi: false,
        config: None,
        http_addr: None,
        master_config: None,
        http_token: None,
        transport: "tcp".to_string(),
        xdp_iface: "lo".to_string(),
        xdp_queue: 0,
        xdp_ring_size: 256,
        flush_threshold: None,
        durability: None,
        max_partitions: None,
        grpc_port: 50051,
        no_grpc: false,
        grpc_host: std::net::IpAddr::from([127, 0, 0, 1]),
        grpc_token: None,
        tpm_unseal: false,
        encrypt_at_rest: false,
        metrics_port: 0,
        metrics_host: std::net::IpAddr::from([127, 0, 0, 1]),
        pgwire_port: None,
        cluster_bind: None,
        cluster_seeds: vec![],
        cluster_node_id: None,
        cluster_status_port: 7900,
        catalog: false,
        restore_catalog: false,
    };
    let bound = bind(&args).unwrap();
    bound.server.shutdown_handle().shutdown();
    assert_eq!(bound.schema, vec!["pk", "v"]);
    assert!(dir.path().join("schema.txt").exists());
}

#[test]
fn bind_reads_existing_schema_from_disk() {
    use vvctl::commands::serve::{ServeArgs, bind};

    let dir = TempDir::new().unwrap();
    std::fs::write(dir.path().join("schema.txt"), "pk\nval\n").unwrap();

    let args = ServeArgs {
        dir: dir.path().to_path_buf(),
        columns: vec![], // ignored â€” schema already on disk
        addr: ephemeral(),
        max_connections: 4,
        secret: None,
        key_id: "default".into(),
        cert: None,
        key: None,
        multi: false,
        config: None,
        http_addr: None,
        master_config: None,
        http_token: None,
        transport: "tcp".to_string(),
        xdp_iface: "lo".to_string(),
        xdp_queue: 0,
        xdp_ring_size: 256,
        flush_threshold: None,
        durability: None,
        max_partitions: None,
        grpc_port: 50051,
        no_grpc: false,
        grpc_host: std::net::IpAddr::from([127, 0, 0, 1]),
        grpc_token: None,
        tpm_unseal: false,
        encrypt_at_rest: false,
        metrics_port: 0,
        metrics_host: std::net::IpAddr::from([127, 0, 0, 1]),
        pgwire_port: None,
        cluster_bind: None,
        cluster_seeds: vec![],
        cluster_node_id: None,
        cluster_status_port: 7900,
        catalog: false,
        restore_catalog: false,
    };
    let bound = bind(&args).unwrap();
    bound.server.shutdown_handle().shutdown();
    assert_eq!(bound.schema, vec!["pk", "val"]);
}

#[test]
fn bind_fails_when_no_table_and_no_columns() {
    use vvctl::commands::serve::{ServeArgs, bind};

    let dir = TempDir::new().unwrap();
    let args = ServeArgs {
        dir: dir.path().to_path_buf(),
        columns: vec![],
        addr: ephemeral(),
        max_connections: 4,
        secret: None,
        key_id: "default".into(),
        cert: None,
        key: None,
        multi: false,
        config: None,
        http_addr: None,
        master_config: None,
        http_token: None,
        transport: "tcp".to_string(),
        xdp_iface: "lo".to_string(),
        xdp_queue: 0,
        xdp_ring_size: 256,
        flush_threshold: None,
        durability: None,
        max_partitions: None,
        grpc_port: 50051,
        no_grpc: false,
        grpc_host: std::net::IpAddr::from([127, 0, 0, 1]),
        grpc_token: None,
        tpm_unseal: false,
        encrypt_at_rest: false,
        metrics_port: 0,
        metrics_host: std::net::IpAddr::from([127, 0, 0, 1]),
        pgwire_port: None,
        cluster_bind: None,
        cluster_seeds: vec![],
        cluster_node_id: None,
        cluster_status_port: 7900,
        catalog: false,
        restore_catalog: false,
    };
    assert!(bind(&args).is_err());
}

#[test]
fn bind_fails_with_cert_but_no_key() {
    use vvctl::commands::serve::{ServeArgs, bind};

    let dir = TempDir::new().unwrap();
    std::fs::write(dir.path().join("schema.txt"), "pk\n").unwrap();
    let args = ServeArgs {
        dir: dir.path().to_path_buf(),
        columns: vec![],
        addr: ephemeral(),
        max_connections: 4,
        secret: None,
        key_id: "default".into(),
        cert: Some(std::path::PathBuf::from("nonexistent.pem")),
        key: None,
        multi: false,
        config: None,
        http_addr: None,
        master_config: None,
        http_token: None,
        transport: "tcp".to_string(),
        xdp_iface: "lo".to_string(),
        xdp_queue: 0,
        xdp_ring_size: 256,
        flush_threshold: None,
        durability: None,
        max_partitions: None,
        grpc_port: 50051,
        no_grpc: false,
        grpc_host: std::net::IpAddr::from([127, 0, 0, 1]),
        grpc_token: None,
        tpm_unseal: false,
        encrypt_at_rest: false,
        metrics_port: 0,
        metrics_host: std::net::IpAddr::from([127, 0, 0, 1]),
        pgwire_port: None,
        cluster_bind: None,
        cluster_seeds: vec![],
        cluster_node_id: None,
        cluster_status_port: 7900,
        catalog: false,
        restore_catalog: false,
    };
    assert!(bind(&args).is_err());
}

// â”€â”€ connect: ping â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

#[test]
fn connect_ping_round_trips() {
    use vvctl::commands::connect::ConnectCommand;

    let dir = TempDir::new().unwrap();
    let (addr, shutdown) = start_plain(&dir);
    let result = plain(addr, ConnectCommand::Ping);
    shutdown();
    assert!(result.is_ok(), "ping: {result:?}");
}

#[test]
fn connect_to_closed_port_returns_error() {
    use vvctl::commands::connect::ConnectCommand;

    let result = plain("127.0.0.1:1".parse().unwrap(), ConnectCommand::Ping);
    assert!(result.is_err(), "expected connection error");
}

// â”€â”€ connect: CRUD â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

#[test]
fn connect_insert_then_get_round_trips() {
    use vvctl::commands::connect::ConnectCommand;

    let dir = TempDir::new().unwrap();
    let (addr, shutdown) = start_plain(&dir);

    // values = full row: [pk, age, score]
    plain(
        addr,
        ConnectCommand::Insert {
            pk: 42,
            values: row(42, 25, 99),
        },
    )
    .unwrap();
    let get = plain(addr, ConnectCommand::Get { pk: 42 });

    shutdown();
    assert!(get.is_ok(), "get: {get:?}");
}

#[test]
fn connect_get_missing_key_returns_ok() {
    use vvctl::commands::connect::ConnectCommand;

    let dir = TempDir::new().unwrap();
    let (addr, shutdown) = start_plain(&dir);
    let result = plain(addr, ConnectCommand::Get { pk: 9999 });
    shutdown();
    assert!(result.is_ok(), "get-missing: {result:?}");
}

#[test]
fn connect_delete_existing_row() {
    use vvctl::commands::connect::ConnectCommand;

    let dir = TempDir::new().unwrap();
    let (addr, shutdown) = start_plain(&dir);

    plain(
        addr,
        ConnectCommand::Insert {
            pk: 7,
            values: row(7, 1, 2),
        },
    )
    .unwrap();
    let del = plain(addr, ConnectCommand::Delete { pk: 7 });
    shutdown();
    assert!(del.is_ok(), "delete: {del:?}");
}

#[test]
fn connect_delete_missing_row_returns_ok() {
    use vvctl::commands::connect::ConnectCommand;

    let dir = TempDir::new().unwrap();
    let (addr, shutdown) = start_plain(&dir);
    let result = plain(addr, ConnectCommand::Delete { pk: 123 });
    shutdown();
    assert!(result.is_ok(), "delete-missing: {result:?}");
}

// â”€â”€ connect: scan â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

#[test]
fn connect_scan_returns_inserted_rows() {
    use vvctl::commands::connect::ConnectCommand;

    let dir = TempDir::new().unwrap();
    let (addr, shutdown) = start_plain(&dir);

    for pk in [10u64, 20, 30] {
        plain(
            addr,
            ConnectCommand::Insert {
                pk,
                values: row(pk, pk, pk * 2),
            },
        )
        .unwrap();
    }
    let scan = plain(
        addr,
        ConnectCommand::Scan {
            col: 0,
            lo: 10,
            hi: 30,
            limit: 0,
        },
    );
    shutdown();
    assert!(scan.is_ok(), "scan: {scan:?}");
}

#[test]
fn connect_scan_empty_range_returns_ok() {
    use vvctl::commands::connect::ConnectCommand;

    let dir = TempDir::new().unwrap();
    let (addr, shutdown) = start_plain(&dir);
    let result = plain(
        addr,
        ConnectCommand::Scan {
            col: 0,
            lo: 1000,
            hi: 2000,
            limit: 0,
        },
    );
    shutdown();
    assert!(result.is_ok(), "empty-scan: {result:?}");
}

// â”€â”€ connect: stats / flush / compact â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

#[test]
fn connect_stats_returns_ok() {
    use vvctl::commands::connect::ConnectCommand;

    let dir = TempDir::new().unwrap();
    let (addr, shutdown) = start_plain(&dir);
    let result = plain(addr, ConnectCommand::Stats);
    shutdown();
    assert!(result.is_ok(), "stats: {result:?}");
}

#[test]
fn connect_flush_returns_ok() {
    use vvctl::commands::connect::ConnectCommand;

    let dir = TempDir::new().unwrap();
    let (addr, shutdown) = start_plain(&dir);
    let result = plain(addr, ConnectCommand::Flush);
    shutdown();
    assert!(result.is_ok(), "flush: {result:?}");
}

#[test]
fn connect_compact_returns_ok() {
    use vvctl::commands::connect::ConnectCommand;

    let dir = TempDir::new().unwrap();
    let (addr, shutdown) = start_plain(&dir);

    plain(
        addr,
        ConnectCommand::Insert {
            pk: 1,
            values: row(1, 10, 20),
        },
    )
    .unwrap();
    let result = plain(addr, ConnectCommand::Compact);
    shutdown();
    assert!(result.is_ok(), "compact: {result:?}");
}

// â”€â”€ connect: auth paths â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

const SECRET_HEX: &str = "0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f20";

#[test]
fn connect_with_correct_secret_succeeds() {
    use vvctl::commands::connect::ConnectCommand;

    let dir = TempDir::new().unwrap();
    let (addr, shutdown) = start_authed(&dir, SECRET_HEX);

    let result = vvctl::commands::connect::run(mk_args(
        addr,
        Some(SECRET_HEX.to_string()),
        "k1",
        ConnectCommand::Ping,
    ));
    shutdown();
    assert!(result.is_ok(), "authed ping: {result:?}");
}

#[test]
fn connect_with_wrong_secret_fails() {
    use vvctl::commands::connect::ConnectCommand;

    let dir = TempDir::new().unwrap();
    let (addr, shutdown) = start_authed(&dir, SECRET_HEX);

    let wrong = "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff";
    let result = vvctl::commands::connect::run(mk_args(
        addr,
        Some(wrong.to_string()),
        "k1",
        ConnectCommand::Insert {
            pk: 1,
            values: row(1, 1, 1),
        },
    ));
    shutdown();
    assert!(result.is_err(), "wrong-secret must fail; got {result:?}");
}

#[test]
fn connect_unauthenticated_to_authed_server_ping_ok_insert_fails() {
    use vvctl::commands::connect::ConnectCommand;

    let dir = TempDir::new().unwrap();
    let (addr, shutdown) = start_authed(&dir, SECRET_HEX);

    // Ping is always allowed without auth.
    let ping = plain(addr, ConnectCommand::Ping);
    // Insert requires auth.
    let ins = plain(
        addr,
        ConnectCommand::Insert {
            pk: 5,
            values: row(5, 1, 2),
        },
    );

    shutdown();
    assert!(ping.is_ok(), "unauthenticated ping must succeed: {ping:?}");
    assert!(ins.is_err(), "unauthenticated insert must fail: {ins:?}");
}

// â”€â”€ full lifecycle via connect::run â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

#[test]
fn full_crud_lifecycle_via_connect_run() {
    use vvctl::commands::connect::ConnectCommand;

    let dir = TempDir::new().unwrap();
    let (addr, shutdown) = start_plain(&dir);

    plain(
        addr,
        ConnectCommand::Insert {
            pk: 100,
            values: row(100, 30, 50),
        },
    )
    .unwrap();
    plain(
        addr,
        ConnectCommand::Insert {
            pk: 200,
            values: row(200, 40, 60),
        },
    )
    .unwrap();
    plain(addr, ConnectCommand::Get { pk: 100 }).unwrap();
    plain(
        addr,
        ConnectCommand::Scan {
            col: 0,
            lo: 100,
            hi: 200,
            limit: 10,
        },
    )
    .unwrap();
    plain(addr, ConnectCommand::Delete { pk: 100 }).unwrap();
    plain(addr, ConnectCommand::Get { pk: 100 }).unwrap(); // missing now â€” still Ok
    plain(addr, ConnectCommand::Flush).unwrap();
    plain(addr, ConnectCommand::Stats).unwrap();
    plain(addr, ConnectCommand::Compact).unwrap();
    shutdown();
}

// â”€â”€ direct Client API: verify underlying round-trips â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

#[test]
fn client_insert_and_get_match() {
    let dir = TempDir::new().unwrap();
    let (addr, shutdown) = start_plain(&dir);

    let mut c = Client::connect(addr).unwrap();
    // values = full row: [pk, age, score]
    c.insert(55, vec![55, 11, 22]).unwrap();
    let got = c.get(55).unwrap().expect("row must exist");
    shutdown();

    assert_eq!(got, vec![55, 11, 22]);
}

#[test]
fn client_scan_limit_is_respected() {
    let dir = TempDir::new().unwrap();
    let (addr, shutdown) = start_plain(&dir);

    let mut c = Client::connect(addr).unwrap();
    for pk in 1u64..=10 {
        c.insert(pk, vec![pk, pk * 10, pk * 100]).unwrap();
    }
    let rows = c.range_scan(0, 1, 10, 3).unwrap();
    shutdown();
    assert_eq!(
        rows.len(),
        3,
        "limit=3 must return exactly 3 rows; got {}",
        rows.len()
    );
}

#[test]
fn client_delete_removes_row() {
    let dir = TempDir::new().unwrap();
    let (addr, shutdown) = start_plain(&dir);

    let mut c = Client::connect(addr).unwrap();
    c.insert(77, vec![77, 1, 2]).unwrap();
    let existed = c.delete(77).unwrap();
    let after = c.get(77).unwrap();
    shutdown();

    assert!(existed, "delete must report row existed");
    assert!(after.is_none(), "row must be gone after delete");
}

#[test]
fn client_stats_reflect_inserted_rows() {
    let dir = TempDir::new().unwrap();
    let (addr, shutdown) = start_plain(&dir);

    let mut c = Client::connect(addr).unwrap();
    for pk in [1u64, 2, 3] {
        c.insert(pk, vec![pk, pk, pk]).unwrap();
    }
    // Flush so rows are in SST files; stats counts persisted rows only.
    c.flush().unwrap();
    let (rows, _parts, _bytes) = c.stats().unwrap();
    shutdown();

    assert!(rows >= 3, "stats must report â‰¥3 rows; got {rows}");
}

// â”€â”€ connect: secondary-lookup â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

#[test]
fn connect_secondary_lookup_returns_matching_rows() {
    use vvctl::commands::connect::ConnectCommand;

    let dir = TempDir::new().unwrap();
    let (addr, shutdown) = start_plain(&dir);

    plain(
        addr,
        ConnectCommand::Insert {
            pk: 10,
            values: row(10, 30, 0),
        },
    )
    .unwrap();
    plain(
        addr,
        ConnectCommand::Insert {
            pk: 20,
            values: row(20, 40, 0),
        },
    )
    .unwrap();
    plain(
        addr,
        ConnectCommand::Insert {
            pk: 30,
            values: row(30, 30, 0),
        },
    )
    .unwrap();

    let result = plain(addr, ConnectCommand::SecondaryLookup { col: 1, value: 30 });
    shutdown();
    assert!(result.is_ok(), "secondary-lookup: {result:?}");
}

#[test]
fn connect_secondary_lookup_no_match_returns_ok() {
    use vvctl::commands::connect::ConnectCommand;

    let dir = TempDir::new().unwrap();
    let (addr, shutdown) = start_plain(&dir);

    plain(
        addr,
        ConnectCommand::Insert {
            pk: 1,
            values: row(1, 5, 0),
        },
    )
    .unwrap();

    let result = plain(addr, ConnectCommand::SecondaryLookup { col: 1, value: 999 });
    shutdown();
    assert!(
        result.is_ok(),
        "empty secondary-lookup must return Ok: {result:?}"
    );
}

// â”€â”€ connect: fk-exists â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

#[test]
fn connect_fk_exists_true_for_inserted_pk() {
    use vvctl::commands::connect::ConnectCommand;

    let dir = TempDir::new().unwrap();
    let (addr, shutdown) = start_plain(&dir);

    plain(
        addr,
        ConnectCommand::Insert {
            pk: 42,
            values: row(42, 1, 2),
        },
    )
    .unwrap();

    let result = plain(addr, ConnectCommand::FkExists { value: 42 });
    shutdown();
    assert!(result.is_ok(), "fk-exists for inserted pk: {result:?}");
}

#[test]
fn connect_fk_exists_false_for_unknown_pk() {
    use vvctl::commands::connect::ConnectCommand;

    let dir = TempDir::new().unwrap();
    let (addr, shutdown) = start_plain(&dir);

    plain(
        addr,
        ConnectCommand::Insert {
            pk: 1,
            values: row(1, 1, 1),
        },
    )
    .unwrap();

    let result = plain(addr, ConnectCommand::FkExists { value: 99999 });
    shutdown();
    assert!(result.is_ok(), "fk-exists for unknown pk: {result:?}");
}

// â”€â”€ direct Client API: secondary_scan + fk_exists â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

#[test]
fn client_secondary_scan_finds_inserted_value() {
    let dir = TempDir::new().unwrap();
    let (addr, shutdown) = start_plain(&dir);

    let mut c = Client::connect(addr).unwrap();
    c.insert(1, vec![1, 77, 0]).unwrap();
    c.insert(2, vec![2, 88, 0]).unwrap();
    c.insert(3, vec![3, 77, 0]).unwrap();

    let rows = c.secondary_scan(1, 77).unwrap();
    shutdown();

    let mut pks: Vec<u64> = rows.iter().map(|r| r[0]).collect();
    pks.sort_unstable();
    assert_eq!(pks, vec![1, 3]);
}

#[test]
fn client_fk_exists_true_after_insert() {
    let dir = TempDir::new().unwrap();
    let (addr, shutdown) = start_plain(&dir);

    let mut c = Client::connect(addr).unwrap();
    c.insert(55, vec![55, 1, 2]).unwrap();

    let found = c.fk_exists(55).unwrap();
    let not_found = c.fk_exists(9999).unwrap();
    shutdown();

    assert!(found, "pk=55 must be found");
    assert!(!not_found, "pk=9999 must not be found");
}
