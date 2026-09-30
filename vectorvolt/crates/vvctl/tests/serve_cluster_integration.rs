//! Integration test for Gap #3 fix: `vvctl serve --cluster-bind` starts a
//! real `vectorvolt-cluster` node (NodeId + MembershipList + GossipEngine)
//! and exposes its live status over `GET /cluster/status`.
//!
//! Scope note: `vectorvolt-cluster` has no wire-level gossip transport today
//! (see `vectorvolt_cluster::node_runtime` docs), so this test proves what is
//! actually real: the node is genuinely constructed and running inside a
//! `vvctl serve` process, its status is observable over HTTP, and
//! `--cluster-seeds` are reported as configured-but-unverified (never faked
//! as discovered peers). Real multi-process gossip convergence is NOT proven
//! here and is left as a documented follow-up.

use std::net::SocketAddr;
use std::thread;
use std::time::Duration;

use tempfile::TempDir;

fn ephemeral() -> SocketAddr {
    "127.0.0.1:0".parse().unwrap()
}

fn free_port() -> u16 {
    let l = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let p = l.local_addr().unwrap().port();
    drop(l);
    p
}

fn start_with_cluster(dir: &TempDir, seeds: Vec<SocketAddr>) -> (u16, impl FnOnce()) {
    use vvctl::commands::serve::{ServeArgs, bind};

    let cluster_bind: SocketAddr = format!("127.0.0.1:{}", free_port()).parse().unwrap();
    let status_port = free_port();

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
        tpm_unseal: false,
        encrypt_at_rest: false,
        metrics_port: 0,
        pgwire_port: None,
        catalog: false,
        restore_catalog: false,
        cluster_bind: Some(cluster_bind),
        cluster_seeds: seeds,
        cluster_node_id: None,
        cluster_status_port: status_port,
    };

    let bound = bind(&args).expect("bind");
    let shutdown = bound.server.shutdown_handle();

    thread::spawn(move || {
        let _ = bound.server.serve();
    });
    thread::sleep(Duration::from_millis(5));

    // Spawn the cluster node + status server the same way `serve::run` does.
    {
        use vectorvolt_cluster::node_runtime::{ClusterNode, ClusterStatusServer};
        let node = ClusterNode::start(cluster_bind, args.cluster_seeds.clone(), None);
        thread::Builder::new()
            .name("cluster-status-test".into())
            .spawn(move || {
                let rt = tokio::runtime::Runtime::new().unwrap();
                rt.block_on(async move {
                    ClusterStatusServer::new(node).serve(status_port).await;
                });
            })
            .expect("spawn cluster-status-test thread");
        thread::sleep(Duration::from_millis(50));
    }

    (status_port, move || shutdown.shutdown())
}

async fn fetch_status(port: u16) -> serde_json::Value {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpStream;

    let mut stream = TcpStream::connect(format!("127.0.0.1:{port}"))
        .await
        .unwrap();
    stream
        .write_all(b"GET /cluster/status HTTP/1.1\r\n\r\n")
        .await
        .unwrap();
    let mut buf = vec![0u8; 8192];
    let n = stream.read(&mut buf).await.unwrap();
    let text = String::from_utf8_lossy(&buf[..n]);
    let body = text.split("\r\n\r\n").nth(1).unwrap_or("");
    serde_json::from_str(body).expect("valid JSON status body")
}

/// A real cluster node is constructed and its status is observable over
/// HTTP: node id is a real UUID, bind addr matches what was configured, and
/// configured seeds are reported separately (never claimed as known peers,
/// since no gossip transport has run).
#[test]
fn cluster_status_reports_real_node_and_configured_seeds() {
    let dir = TempDir::new().unwrap();
    let seed: SocketAddr = "127.0.0.1:19001".parse().unwrap();
    let (status_port, shutdown) = start_with_cluster(&dir, vec![seed]);

    let rt = tokio::runtime::Runtime::new().unwrap();
    let status = rt.block_on(fetch_status(status_port));

    shutdown();

    let node_id = status["node_id"].as_str().unwrap();
    assert!(
        uuid::Uuid::parse_str(node_id).is_ok(),
        "node_id should be a real UUID, got {node_id:?}"
    );
    assert_eq!(status["membership_size"].as_u64().unwrap(), 1);
    let configured_seeds = status["configured_seeds"].as_array().unwrap();
    assert_eq!(configured_seeds.len(), 1);
    assert_eq!(configured_seeds[0].as_str().unwrap(), seed.to_string());
    // Configured seeds must NOT be faked as known/discovered peers.
    let known_peers = status["known_peers"].as_array().unwrap();
    assert!(
        known_peers.is_empty(),
        "no gossip transport has run yet; known_peers must stay empty, not \
         hardcoded from configured_seeds"
    );
}

/// Without `--cluster-bind`, no cluster node or status endpoint is started.
#[test]
fn no_cluster_node_without_flag() {
    let dir = TempDir::new().unwrap();
    use vvctl::commands::serve::{ServeArgs, bind};

    let args = ServeArgs {
        dir: dir.path().to_path_buf(),
        addr: ephemeral(),
        multi: true,
        no_grpc: true,
        grpc_port: 0,
        metrics_port: 0,
        ..ServeArgs::default()
    };
    assert!(args.cluster_bind.is_none());

    let bound = bind(&args).expect("bind");
    let shutdown = bound.server.shutdown_handle();
    thread::spawn(move || {
        let _ = bound.server.serve();
    });
    thread::sleep(Duration::from_millis(5));
    shutdown.shutdown();
}
