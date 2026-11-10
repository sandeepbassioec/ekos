//! `vvctl cluster` against live gossip nodes (F4): init identity, join via
//! seed, members/status from the running node, graceful leave.

use std::net::{SocketAddr, UdpSocket};
use std::time::{Duration, Instant};
use vectorvolt_cluster::node_runtime::{ClusterNode, ClusterStatusServer, PeerInfo};
use vectorvolt_cluster::swim::SwimConfig;
use vvctl::commands::cluster_cmd::{
    ClusterArgs, ClusterCommand, load_identity, record_control_addr, run,
};

fn free_udp() -> SocketAddr {
    let s = UdpSocket::bind("127.0.0.1:0").unwrap();
    s.local_addr().unwrap()
}

fn free_tcp() -> u16 {
    let l = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    l.local_addr().unwrap().port()
}

fn cfg() -> SwimConfig {
    SwimConfig {
        probe_interval: Duration::from_millis(30),
        probe_timeout: Duration::from_millis(40),
        suspect_timeout: Duration::from_secs(2),
        ..Default::default()
    }
}

/// Start a node (gossip + control endpoint) on its own runtime thread.
fn start_node(seeds: Vec<SocketAddr>) -> (SocketAddr, SocketAddr) {
    let bind = free_udp();
    let port = free_tcp();
    let node = ClusterNode::start(bind, seeds, None);
    std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async move {
            node.start_gossip(cfg()).await.unwrap();
            ClusterStatusServer::new(node).serve(port).await;
        });
    });
    std::thread::sleep(Duration::from_millis(200));
    (bind, format!("127.0.0.1:{port}").parse().unwrap())
}

fn members(ctl: SocketAddr) -> Vec<PeerInfo> {
    use std::io::{Read, Write};
    let mut s = std::net::TcpStream::connect(ctl).unwrap();
    s.write_all(b"GET /cluster/members HTTP/1.1\r\n\r\n")
        .unwrap();
    let mut raw = String::new();
    s.read_to_string(&mut raw).unwrap();
    serde_json::from_str(raw.split("\r\n\r\n").nth(1).unwrap()).unwrap()
}

fn wait(limit: Duration, mut f: impl FnMut() -> bool) -> bool {
    let end = Instant::now() + limit;
    while Instant::now() < end {
        if f() {
            return true;
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    f()
}

fn args(dir: &std::path::Path, node: Option<SocketAddr>, command: ClusterCommand) -> ClusterArgs {
    ClusterArgs {
        dir: dir.to_path_buf(),
        node,
        offline: false,
        command,
    }
}

#[test]
fn init_writes_fresh_identity() {
    let d = tempfile::tempdir().unwrap();
    let init = || ClusterCommand::Init {
        name: "c".into(),
        replication_factor: 3,
        vnode_count: 16,
    };
    run(args(d.path(), None, init())).unwrap();
    let a = load_identity(d.path()).unwrap();
    run(args(d.path(), None, init())).unwrap();
    let b = load_identity(d.path()).unwrap();
    assert_ne!(a.cluster_id, b.cluster_id);
    assert_ne!(a.node_id, b.node_id);
}

#[test]
fn join_members_and_leave_talk_to_running_nodes() {
    let d = tempfile::tempdir().unwrap();
    let (seed_udp, seed_ctl) = start_node(vec![]);
    let (_b_udp, b_ctl) = start_node(vec![]);

    // `vvctl cluster join` makes the running node B join through the seed.
    run(args(
        d.path(),
        Some(b_ctl),
        ClusterCommand::Join { addr: seed_udp },
    ))
    .unwrap();
    assert!(wait(Duration::from_secs(5), || {
        members(seed_ctl)
            .iter()
            .filter(|m| m.state == "Alive")
            .count()
            == 2
    }));
    run(args(d.path(), Some(seed_ctl), ClusterCommand::Members)).unwrap();
    run(args(d.path(), Some(seed_ctl), ClusterCommand::Status)).unwrap();

    // Graceful leave on B is observed as Left by the seed.
    run(args(
        d.path(),
        Some(b_ctl),
        ClusterCommand::Leave { addr: None },
    ))
    .unwrap();
    assert!(wait(Duration::from_secs(5), || {
        members(seed_ctl).iter().any(|m| m.state == "Left")
    }));
}

#[test]
fn live_command_fails_cleanly_when_node_unreachable() {
    let d = tempfile::tempdir().unwrap();
    let dead: SocketAddr = format!("127.0.0.1:{}", free_tcp()).parse().unwrap();
    assert!(run(args(d.path(), Some(dead), ClusterCommand::Status)).is_err());
}

fn init_dir(d: &std::path::Path) {
    run(args(
        d,
        None,
        ClusterCommand::Init {
            name: "c".into(),
            replication_factor: 3,
            vnode_count: 16,
        },
    ))
    .unwrap();
}

#[test]
fn commands_default_to_the_local_node_from_cluster_identity() {
    let d = tempfile::tempdir().unwrap();
    init_dir(d.path());
    let (seed_udp, seed_ctl) = start_node(vec![]);
    let (_b_udp, b_ctl) = start_node(vec![]);

    // The identity records this node's control endpoint; no --node needed.
    record_control_addr(d.path(), b_ctl).unwrap();
    assert_eq!(load_identity(d.path()).unwrap().control_addr, Some(b_ctl));
    run(args(d.path(), None, ClusterCommand::Status)).unwrap();
    run(args(d.path(), None, ClusterCommand::Members)).unwrap();
    run(args(
        d.path(),
        None,
        ClusterCommand::Join { addr: seed_udp },
    ))
    .unwrap();
    assert!(wait(Duration::from_secs(5), || {
        members(seed_ctl)
            .iter()
            .filter(|m| m.state == "Alive")
            .count()
            == 2
    }));
    run(args(d.path(), None, ClusterCommand::Leave { addr: None })).unwrap();
    assert!(wait(Duration::from_secs(5), || {
        members(seed_ctl).iter().any(|m| m.state == "Left")
    }));
}

#[test]
fn default_does_not_silently_fall_back_to_offline_cluster_json() {
    let d = tempfile::tempdir().unwrap();
    init_dir(d.path());
    // cluster.json exists, but the recorded node is unreachable: error, not offline data.
    let dead: SocketAddr = format!("127.0.0.1:{}", free_tcp()).parse().unwrap();
    record_control_addr(d.path(), dead).unwrap();
    assert!(run(args(d.path(), None, ClusterCommand::Status)).is_err());
    assert!(run(args(d.path(), None, ClusterCommand::Members)).is_err());
    // Offline mode only with the explicit flag.
    let mut a = args(d.path(), None, ClusterCommand::Status);
    a.offline = true;
    run(a).unwrap();
}
