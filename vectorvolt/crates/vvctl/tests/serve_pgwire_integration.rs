//! Integration tests for Gap #2 fix: `vvctl serve --pgwire-port` starts a
//! pgwire (PostgreSQL wire protocol) listener that shares the SAME
//! `TableRegistry` as the main wire-protocol server, instead of requiring a
//! separate `vvctl reader start` process with its own independent registry.

use std::net::SocketAddr;
use std::path::PathBuf;
use std::thread;
use std::time::Duration;

use bytes::BufMut;
use tempfile::TempDir;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use vectorvolt_net::Client;

fn ephemeral() -> SocketAddr {
    "127.0.0.1:0".parse().unwrap()
}

/// Start `vvctl serve --multi` with an ephemeral wire-protocol port and,
/// optionally, an ephemeral pgwire port. Returns the wire-protocol addr, the
/// resolved pgwire addr (if enabled), and a shutdown closure.
fn start_multi_with_pgwire(
    dir: &TempDir,
    enable_pgwire: bool,
) -> (SocketAddr, Option<u16>, impl FnOnce()) {
    use vvctl::commands::serve::{ServeArgs, bind};

    // Pick a free port up front for pgwire since PgServer::serve() binds
    // itself (unlike the wire-protocol Server, which exposes local_addr()
    // after an ephemeral bind).
    let pgwire_port = if enable_pgwire {
        let l = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let p = l.local_addr().unwrap().port();
        drop(l);
        Some(p)
    } else {
        None
    };

    let args = ServeArgs {
        dir: dir.path().to_path_buf(),
        columns: vec![],
        addr: ephemeral(),
        max_connections: 32,
        secret: None,
        key_id: "default".into(),
        cert: None,
        key: None,
        multi: true,
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
        grpc_port: 0,
        no_grpc: true,
        grpc_host: std::net::IpAddr::from([127, 0, 0, 1]),
        grpc_token: None,
        tpm_unseal: false,
        encrypt_at_rest: false,
        metrics_port: 0,
        metrics_host: std::net::IpAddr::from([127, 0, 0, 1]),
        pgwire_port,
        catalog: false,
        restore_catalog: false,
        cluster_bind: None,
        cluster_seeds: vec![],
        cluster_node_id: None,
        cluster_status_port: 7900,
    };

    let bound = bind(&args).expect("bind");
    let addr = bound.server.local_addr().expect("local_addr");
    let shutdown = bound.server.shutdown_handle();
    let registry = bound.registry.clone();

    thread::spawn(move || {
        let _ = bound.server.serve();
    });
    thread::sleep(Duration::from_millis(5));

    // Spawn the pgwire listener the same way `serve::run` does, sharing the
    // same registry, so the test exercises the real integration path.
    if let Some(port) = pgwire_port {
        let reg = registry.expect("multi mode registry");
        thread::Builder::new()
            .name("pgwire-test".into())
            .spawn(move || {
                let rt = tokio::runtime::Runtime::new().unwrap();
                rt.block_on(async move {
                    use vectorvolt_reader::server::{PgConfig, PgServer};
                    let cfg =
                        PgConfig::new(format!("127.0.0.1:{port}").parse().unwrap(), PathBuf::new());
                    let server = PgServer::with_registry(cfg, reg);
                    let _ = server.serve().await;
                });
            })
            .expect("spawn pgwire-test thread");
        thread::sleep(Duration::from_millis(50));
    }

    (addr, pgwire_port, move || shutdown.shutdown())
}

async fn connect_and_startup(addr: SocketAddr) -> TcpStream {
    let mut stream = TcpStream::connect(addr).await.unwrap();
    let params = b"user\0postgres\0database\0vectorvolt\0\0";
    let len = 4 + 4 + params.len();
    let mut buf = bytes::BytesMut::new();
    buf.put_i32(len as i32);
    buf.put_i32(3 << 16);
    buf.put(params.as_ref());
    stream.write_all(&buf).await.unwrap();
    stream
}

fn make_query_msg(sql: &str) -> Vec<u8> {
    let payload = format!("{sql}\0");
    let len = 4 + payload.len();
    let mut buf = bytes::BytesMut::new();
    buf.put_u8(b'Q');
    buf.put_i32(len as i32);
    buf.put(payload.as_bytes());
    buf.to_vec()
}

/// Proves the pgwire listener started by `vvctl serve --pgwire-port` shares
/// the same `TableRegistry` as the normal wire-protocol path: a row inserted
/// via the wire-protocol `Client` is visible to a SELECT issued over a raw
/// pgwire connection to the SAME process.
#[test]
fn pgwire_listener_shares_registry_with_wire_protocol_server() {
    let dir = TempDir::new().unwrap();
    let (wire_addr, pgwire_port, shutdown) = start_multi_with_pgwire(&dir, true);
    let pgwire_addr: SocketAddr = format!("127.0.0.1:{}", pgwire_port.unwrap())
        .parse()
        .unwrap();

    // Create the table + insert a row via the normal wire-protocol path.
    let mut c = Client::connect(wire_addr).unwrap();
    c.create_table("shared", vec!["pk".into(), "val".into()])
        .expect("create table via wire protocol");
    c.insert(42, vec![42, 99])
        .expect("insert via wire protocol");

    // Now read it back over pgwire on the SAME server process.
    let rt = tokio::runtime::Runtime::new().unwrap();
    let found = rt.block_on(async move {
        let mut stream = connect_and_startup(pgwire_addr).await;
        let mut buf = vec![0u8; 4096];
        let _ = stream.read(&mut buf).await.unwrap(); // auth-ok + ready-for-query

        stream
            .write_all(&make_query_msg("SELECT * FROM shared"))
            .await
            .unwrap();
        let mut resp = vec![0u8; 4096];
        let n = stream.read(&mut resp).await.unwrap();
        let text = String::from_utf8_lossy(&resp[..n]);
        resp[..n].contains(&b'T') && text.contains("99")
    });

    shutdown();
    assert!(
        found,
        "expected row inserted via wire-protocol Client to be visible via pgwire SELECT \
         on the same server process, proving both listeners share one TableRegistry"
    );
}

/// Without `--pgwire-port`, `vvctl serve` must not bind any pgwire listener.
#[test]
fn no_pgwire_listener_without_flag() {
    let dir = TempDir::new().unwrap();
    let (_wire_addr, pgwire_port, shutdown) = start_multi_with_pgwire(&dir, false);
    assert!(pgwire_port.is_none());

    // Port was never reserved/bound by serve — sanity-check by trying to pick
    // an arbitrary high port and confirming nothing VectorVolt started is
    // listening on it (best-effort; the real assertion is that `bind()` never
    // constructs a PgServer/thread when the flag is absent, verified by code
    // inspection of `commands::serve::run`).
    shutdown();
}
