//! F6 — epoch fencing and sub-2-second failover, three nodes in one process
//! over 127.0.0.1 (real TCP Raft control plane, real WAL-shipping replication).

use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use tempfile::TempDir;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use vectorvolt_cluster::failover::{FailoverConfig, hs, probe_repl_epoch};
use vectorvolt_cluster::node::NodeId;
use vectorvolt_cluster::node_runtime::{ClusterNode, ClusterStatusServer};
use vectorvolt_engine::{Durability, EngineOptions, TableEngine, TableRegistry};
use vectorvolt_replication::{ReplicationConfig, ReplicationMode, ReplicationRole};
use vvctl::commands::serve_failover::{FailoverNode, FailoverNodeOpts, RaftMembershipBridge};

static DEGRADED_OK: AtomicBool = AtomicBool::new(true);

const MAX_FAILOVER: Duration = Duration::from_secs(2);

/// The tests time elections, so they must not compete for CPU with each other.
static SERIAL: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

fn schema() -> Vec<String> {
    vec!["pk".into(), "val".into()]
}

fn open_engine(dir: &Path) -> Arc<TableEngine> {
    Arc::new(
        TableEngine::open_with(
            schema(),
            dir,
            EngineOptions::new(1_048_576).with_durability(Durability::WriteOnly),
        )
        .unwrap(),
    )
}

fn repl_cfg() -> ReplicationConfig {
    ReplicationConfig {
        mode: ReplicationMode::Sync,
        replication_factor: 3,
        ack_timeout: Duration::from_millis(2000),
        poll_interval: Duration::from_millis(5),
        heartbeat_interval: Duration::from_millis(100),
        backoff_initial: Duration::from_millis(20),
        backoff_max: Duration::from_millis(150),
        read_timeout: Duration::from_secs(3),
        batch_frames: 64,
        snapshot_batch_rows: 50,
        // After a failover only two of three nodes are left: keep taking sync
        // writes, explicitly degraded (the default is to refuse them).
        degraded_ok: DEGRADED_OK.load(Ordering::SeqCst),
    }
}

struct Member {
    id: NodeId,
    name: String,
    dir: PathBuf,
    engine: Arc<TableEngine>,
    node: Arc<FailoverNode>,
}

impl Member {
    fn client(&self) -> String {
        format!("client://{}", self.name)
    }
}

async fn spawn_node(
    dir: &Path,
    id: &NodeId,
    peers: Vec<NodeId>,
    engine: Arc<TableEngine>,
    name: &str,
) -> Arc<FailoverNode> {
    let mut control = FailoverConfig::new(id.clone(), dir.join("failover"), peers);
    control.client_addr = format!("client://{name}");
    FailoverNode::start(FailoverNodeOpts {
        engine,
        dir: dir.to_path_buf(),
        key: None,
        repl: repl_cfg(),
        control,
    })
    .await
    .unwrap()
}

struct Cluster {
    _tmp: TempDir,
    members: Vec<Member>,
}

impl Cluster {
    async fn start(n: usize) -> Cluster {
        let tmp = TempDir::new().unwrap();
        let ids: Vec<NodeId> = (0..n).map(|_| NodeId::new()).collect();
        let mut members = Vec::new();
        for (i, id) in ids.iter().enumerate() {
            let dir = tmp.path().join(format!("n{i}"));
            std::fs::create_dir_all(&dir).unwrap();
            let engine = open_engine(&dir);
            let peers = ids.iter().filter(|p| *p != id).cloned().collect();
            let name = format!("n{i}");
            let node = spawn_node(&dir, id, peers, engine.clone(), &name).await;
            members.push(Member {
                id: id.clone(),
                name,
                dir,
                engine,
                node,
            });
        }
        let c = Cluster { _tmp: tmp, members };
        for i in 0..n {
            c.wire(i);
        }
        c
    }

    /// Tell node `i` about everyone else, and everyone else about node `i`.
    fn wire(&self, i: usize) {
        let me = &self.members[i];
        for (j, other) in self.members.iter().enumerate() {
            if i == j {
                continue;
            }
            me.node.controller.add_peer(
                &other.id,
                other.node.controller.raft_addr(),
                other.node.controller.repl_addr(),
                other.client(),
            );
            other.node.controller.add_peer(
                &me.id,
                me.node.controller.raft_addr(),
                me.node.controller.repl_addr(),
                me.client(),
            );
        }
    }

    fn primaries(&self) -> Vec<usize> {
        (0..self.members.len())
            .filter(|&i| self.members[i].node.fence().is_primary())
            .collect()
    }

    async fn wait_primary(&self, within: Duration) -> usize {
        let end = Instant::now() + within;
        loop {
            let p = self.primaries();
            if p.len() == 1 {
                return p[0];
            }
            assert!(
                Instant::now() < end,
                "no single primary within {within:?} (primaries: {p:?})"
            );
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    }
}

fn row(pk: u64) -> Vec<u64> {
    vec![pk, pk * 2]
}

/// Insert through the full write path, retrying until `within` elapses.
async fn write_until_ok(node: &Arc<FailoverNode>, pk: u64, within: Duration) -> Result<(), String> {
    let end = Instant::now() + within;
    loop {
        match node.insert(pk, row(pk)) {
            Ok(()) => return Ok(()),
            Err(e) if Instant::now() >= end => return Err(e),
            Err(_) => tokio::time::sleep(Duration::from_millis(20)).await,
        }
    }
}

async fn wait_in_sync(node: &Arc<FailoverNode>, n: usize) {
    eventually(Duration::from_secs(15), "followers in sync", || {
        node.repl_state().in_sync >= n
    })
    .await;
}

async fn eventually(within: Duration, what: &str, mut f: impl FnMut() -> bool) {
    let end = Instant::now() + within;
    while !f() {
        assert!(Instant::now() < end, "timed out waiting for: {what}");
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn primary_failover_is_fast_and_loses_no_acked_sync_write() {
    let _serial = SERIAL.lock().await;
    let c = Cluster::start(3).await;
    let p = c.wait_primary(Duration::from_secs(10)).await;
    assert_eq!(
        c.members[p].node.role(),
        ReplicationRole::Leader,
        "the elected primary runs as replication leader"
    );
    let epoch1 = c.members[p].node.fence().epoch();
    assert!(epoch1 >= 1);
    // Acknowledgements only mean "on every replica" once both followers are in
    // sync (until then the primary runs explicitly degraded).
    wait_in_sync(&c.members[p].node, 2).await;

    // A writer thread issues sync writes (acked only once every replica has
    // them) until its node dies; every Ok is an acknowledgement.
    let stop = Arc::new(AtomicBool::new(false));
    let acked: Arc<Mutex<Vec<u64>>> = Arc::new(Mutex::new(Vec::new()));
    let writer = {
        let node = c.members[p].node.clone();
        let stop = stop.clone();
        let acked = acked.clone();
        tokio::task::spawn_blocking(move || {
            let mut pk = 1u64;
            while !stop.load(Ordering::Acquire) {
                if node.insert(pk, row(pk)).is_ok() {
                    acked.lock().unwrap().push(pk);
                } else {
                    std::thread::sleep(Duration::from_millis(5));
                }
                pk += 1;
            }
        })
    };
    eventually(
        Duration::from_secs(15),
        "30 acknowledged sync writes",
        || acked.lock().unwrap().len() >= 30,
    )
    .await;

    // Kill the primary while writes are in flight.
    let t0 = Instant::now();
    c.members[p].node.kill();
    stop.store(true, Ordering::Release);

    // A new primary must accept (and sync-replicate) a write within 2 s.
    let watch: Vec<_> = (0..3)
        .filter(|&i| i != p)
        .map(|i| {
            let f = c.members[i].node.fence();
            let ctl = c.members[i].node.controller.clone();
            tokio::spawn(async move {
                let mut log = Vec::new();
                let mut last = (0, false, None);
                while t0.elapsed() < Duration::from_secs(4) {
                    let st = ctl.state();
                    let cur = (f.epoch(), f.is_primary(), st.primary.clone());
                    if cur != last {
                        log.push((t0.elapsed(), cur.clone()));
                        last = cur;
                    }
                    tokio::time::sleep(Duration::from_millis(2)).await;
                }
                log
            })
        })
        .collect();
    let survivors: Vec<usize> = (0..3).filter(|&i| i != p).collect();
    let mut fence_open = None;
    let (new_p, elapsed) = 'outer: loop {
        for &i in &survivors {
            if fence_open.is_none() && c.members[i].node.fence().is_primary() {
                fence_open = Some(t0.elapsed());
            }
            if c.members[i].node.insert(1_000_000, row(1_000_000)).is_ok() {
                break 'outer (i, t0.elapsed());
            }
        }
        assert!(
            t0.elapsed() < Duration::from_secs(10),
            "no new primary accepted a write"
        );
        tokio::time::sleep(Duration::from_millis(5)).await;
    };
    for (k, w) in watch.into_iter().enumerate() {
        println!("timeline {k}: {:?}", w.await.unwrap());
    }
    println!(
        "failover: new primary n{new_p}: fence open after {fence_open:?}, first sync-replicated write acked after {elapsed:?}"
    );
    assert!(
        elapsed < MAX_FAILOVER,
        "failover took {elapsed:?}, budget {MAX_FAILOVER:?}"
    );
    assert!(
        c.members[new_p].node.fence().epoch() > epoch1,
        "the epoch was bumped by the promotion"
    );

    // (The dying primary's writer may still be inside its ack timeout.)
    writer.await.unwrap();

    // No acknowledged write is lost: all are on the new primary...
    let acked = acked.lock().unwrap().clone();
    assert!(acked.len() >= 30);
    for pk in &acked {
        let r = c.members[new_p].engine.get(*pk).unwrap();
        assert_eq!(
            r.map(|r| r.values),
            Some(row(*pk)),
            "acknowledged write {pk} lost in failover"
        );
    }
    // ...and the other survivor converges to the same data.
    let other = survivors.iter().copied().find(|&i| i != new_p).unwrap();
    let other_engine = c.members[other].engine.clone();
    let check = acked.clone();
    eventually(
        Duration::from_secs(10),
        "second survivor resynced",
        move || {
            check
                .iter()
                .all(|pk| matches!(other_engine.get(*pk), Ok(Some(_))))
                && matches!(other_engine.get(1_000_000), Ok(Some(_)))
        },
    )
    .await;
    // Followers refuse writes while a primary is elected.
    assert_eq!(c.members[other].node.role(), ReplicationRole::Follower);
    let err = c.members[other].node.insert(7, row(7)).unwrap_err();
    assert!(
        err.contains(&c.members[new_p].client()),
        "follower names the primary: {err}"
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn returning_old_primary_is_fenced_and_rebuilt_as_follower() {
    let _serial = SERIAL.lock().await;
    let mut c = Cluster::start(3).await;
    let p = c.wait_primary(Duration::from_secs(10)).await;
    wait_in_sync(&c.members[p].node, 2).await;
    for pk in 1..=10u64 {
        write_until_ok(&c.members[p].node, pk, Duration::from_secs(10))
            .await
            .unwrap();
    }

    // The primary dies with an un-replicated tail: a row only it ever saw.
    c.members[p].node.kill();
    c.members[p].engine.insert(9001, row(9001)).unwrap();
    assert!(
        c.members[p].node.insert(11, row(11)).is_err(),
        "a killed node rejects writes"
    );

    let survivors: Vec<usize> = (0..3).filter(|&i| i != p).collect();
    let end = Instant::now() + Duration::from_secs(10);
    let new_p = loop {
        if let Some(&i) = survivors
            .iter()
            .find(|&&i| c.members[i].node.fence().is_primary())
        {
            break i;
        }
        assert!(Instant::now() < end, "no new primary");
        tokio::time::sleep(Duration::from_millis(10)).await;
    };
    for pk in 100..110u64 {
        write_until_ok(&c.members[new_p].node, pk, Duration::from_secs(10))
            .await
            .unwrap();
    }
    let new_epoch = c.members[new_p].node.fence().epoch();

    // The old primary comes back (same data dir and node id, new sockets).
    let old = &c.members[p];
    let peers = c
        .members
        .iter()
        .filter(|m| m.id != old.id)
        .map(|m| m.id.clone())
        .collect();
    let node = spawn_node(&old.dir, &old.id, peers, old.engine.clone(), &old.name).await;
    c.members[p].node = node;
    c.wire(p);

    // Right away it is not a primary: its writes are refused.
    assert!(!c.members[p].node.fence().is_primary());
    assert!(c.members[p].node.insert(12, row(12)).is_err());

    // It learns the new epoch, names the new primary, and becomes a follower.
    let revived = c.members[p].node.clone();
    let want = c.members[new_p].client();
    let fence = revived.fence();
    eventually(
        Duration::from_secs(15),
        "old primary fenced as follower",
        || {
            fence.epoch() >= new_epoch
                && revived.role() == ReplicationRole::Follower
                && revived
                    .insert(13, row(13))
                    .err()
                    .is_some_and(|e| e.contains(&want))
        },
    )
    .await;

    // Its divergent tail is gone and it holds exactly the new primary's data.
    let old_engine = c.members[p].engine.clone();
    eventually(
        Duration::from_secs(15),
        "old primary rebuilt from snapshot",
        || {
            (1..=10u64)
                .chain(100..110)
                .all(|pk| matches!(old_engine.get(pk), Ok(Some(_))))
                && matches!(old_engine.get(9001), Ok(None))
        },
    )
    .await;
    assert!(!c.members[p].node.fence().is_primary());
    assert_eq!(c.members[p].node.role(), ReplicationRole::Follower);

    // Incremental catch-up: no snapshot; exactly the one divergent key was
    // rewritten and only the new primary's 10 rows were streamed.
    // (The counters are bumped just after the rows become visible.)
    let node = c.members[p].node.clone();
    eventually(Duration::from_secs(5), "apply metrics published", move || {
        node.repl_state()
            .metrics
            .is_some_and(|m| m.rows_applied >= 10)
    })
    .await;
    let m = c.members[p].node.repl_state().metrics.unwrap();
    assert_eq!(m.snapshots_applied, 0, "{m:?}");
    assert_eq!(m.snapshot_rows, 0, "{m:?}");
    assert_eq!(m.fix_rows, 1, "{m:?}");
    assert_eq!(m.rows_applied, 10, "{m:?}");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn replication_streams_from_a_stale_epoch_are_rejected() {
    let _serial = SERIAL.lock().await;
    let c = Cluster::start(3).await;
    let p = c.wait_primary(Duration::from_secs(10)).await;
    let epoch = c.members[p].node.fence().epoch();
    let repl: SocketAddr = c.members[p].node.controller.repl_addr();

    let stale = probe_repl_epoch(repl, epoch - 1).await.unwrap();
    assert_eq!(stale.status, hs::STALE_CLIENT, "older epoch rejected");
    assert_eq!(stale.server_epoch, epoch);

    let ahead = probe_repl_epoch(repl, epoch + 1).await.unwrap();
    assert_eq!(
        ahead.status,
        hs::STALE_SERVER,
        "a follower from a newer epoch proves this node is deposed"
    );

    let ok = probe_repl_epoch(repl, epoch).await.unwrap();
    assert!(ok.accepted(), "current epoch is accepted: {ok:?}");

    // A non-primary never serves a replication stream, whatever the epoch.
    let f = (0..3).find(|&i| i != p).unwrap();
    let ffence = c.members[f].node.fence();
    eventually(Duration::from_secs(5), "follower learns the epoch", || {
        ffence.epoch() == epoch
    })
    .await;
    let r = probe_repl_epoch(c.members[f].node.controller.repl_addr(), epoch)
        .await
        .unwrap();
    assert_eq!(r.status, hs::NOT_PRIMARY);
}

async fn wait_follower_knows_primary(c: &Cluster, f: usize, p: usize) {
    let fence = c.members[f].node.fence();
    let want = c.members[p].client();
    eventually(
        Duration::from_secs(10),
        "follower learns the primary",
        || fence.primary_client_addr().as_deref() == Some(want.as_str()),
    )
    .await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn follower_rejects_http_pgwire_and_native_writes() {
    let _serial = SERIAL.lock().await;
    let c = Cluster::start(3).await;
    let p = c.wait_primary(Duration::from_secs(10)).await;
    let f = (0..3).find(|&i| i != p).unwrap();
    wait_follower_knows_primary(&c, f, p).await;
    let gate = c.members[f].node.write_gate();
    let primary_addr = c.members[p].client();
    let registry = Arc::new(TableRegistry::new_with_default(c.members[f].engine.clone()));

    // ---- HTTP ----
    let http = vectorvolt_http::HttpServer::bind(
        registry.clone(),
        vectorvolt_http::HttpServerConfig::new("127.0.0.1:0".parse().unwrap())
            .with_write_gate(gate.clone()),
    )
    .await
    .unwrap();
    let http_addr = http.local_addr().unwrap();
    tokio::spawn(http.serve());
    let http_req = |req: String| async move {
        let mut s = TcpStream::connect(http_addr).await.unwrap();
        s.write_all(req.as_bytes()).await.unwrap();
        let mut out = String::new();
        s.read_to_string(&mut out).await.unwrap();
        out
    };
    let body = r#"{"values":[5,10]}"#;
    let resp = http_req(format!(
        "POST /tables/default/rows/5 HTTP/1.1\r\nHost: x\r\nContent-Type: application/json\r\n\
         Content-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    ))
    .await;
    assert!(resp.starts_with("HTTP/1.1 503"), "HTTP write: {resp}");
    assert!(
        resp.contains("not_primary") && resp.contains(&primary_addr),
        "{resp}"
    );
    let sql = r#"{"sql":"INSERT INTO default (pk, val) VALUES (6, 12)"}"#;
    let resp = http_req(format!(
        "POST /sql HTTP/1.1\r\nHost: x\r\nContent-Type: application/json\r\n\
         Content-Length: {}\r\nConnection: close\r\n\r\n{sql}",
        sql.len()
    ))
    .await;
    assert!(resp.starts_with("HTTP/1.1 503"), "HTTP SQL write: {resp}");
    let resp = http_req(
        "GET /tables/default/rows/5 HTTP/1.1\r\nHost: x\r\nConnection: close\r\n\r\n".into(),
    )
    .await;
    assert!(
        !resp.starts_with("HTTP/1.1 503"),
        "reads are served: {resp}"
    );
    assert!(
        matches!(c.members[f].engine.get(5), Ok(None)),
        "nothing was written"
    );

    // ---- pgwire ----
    let pg = vectorvolt_reader::server::PgServer::with_registry(
        vectorvolt_reader::server::PgConfig::new(
            "127.0.0.1:0".parse().unwrap(),
            c.members[f].dir.clone(),
        ),
        registry.clone(),
    )
    .with_write_gate(gate.clone());
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let pg_addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        let _ = pg
            .serve_listener_until(listener, pg_addr, std::future::pending::<()>())
            .await;
    });
    let mut s = TcpStream::connect(pg_addr).await.unwrap();
    let mut startup = Vec::new();
    startup.extend_from_slice(&196_608u32.to_be_bytes());
    startup.extend_from_slice(b"user\0test\0database\0test\0\0");
    let mut msg = ((startup.len() + 4) as u32).to_be_bytes().to_vec();
    msg.extend_from_slice(&startup);
    s.write_all(&msg).await.unwrap();
    pg_read_until_ready(&mut s).await;
    let write = pg_simple_query(&mut s, "INSERT INTO default (pk, val) VALUES (7, 14)").await;
    assert!(
        write.contains("not the primary") && write.contains(&primary_addr),
        "pgwire write must be refused naming the primary: {write:?}"
    );
    let read = pg_simple_query(&mut s, "SELECT 1").await;
    assert!(
        !read.contains("not the primary"),
        "pgwire reads work: {read:?}"
    );
    assert!(matches!(c.members[f].engine.get(7), Ok(None)));

    // ---- native wire protocol ----
    let net = vectorvolt_net::Server::bind_registry(
        registry,
        vectorvolt_net::ServerConfig::new("127.0.0.1:0".parse().unwrap())
            .with_write_gate(vectorvolt_net::WriteGate::new(move || gate())),
    )
    .unwrap();
    let net_addr = net.local_addr().unwrap();
    std::thread::spawn(move || {
        let _ = net.serve();
    });
    let err = tokio::task::spawn_blocking(move || {
        let mut cl = vectorvolt_net::client::Client::connect(net_addr).unwrap();
        let r = cl.insert(8, vec![8, 16]).unwrap_err().to_string();
        let g = cl.get(8);
        (r, g.is_ok())
    })
    .await
    .unwrap();
    assert!(
        err.0.contains("not the primary") && err.0.contains(&primary_addr),
        "native write must be refused naming the primary: {}",
        err.0
    );
    assert!(err.1, "native reads are served");
    assert!(matches!(c.members[f].engine.get(8), Ok(None)));

    // The primary itself accepts the same write.
    write_until_ok(&c.members[p].node, 5, Duration::from_secs(10))
        .await
        .unwrap();
}

async fn pg_read_until_ready(s: &mut TcpStream) -> Vec<(u8, Vec<u8>)> {
    let mut msgs = Vec::new();
    loop {
        let mut tag = [0u8; 1];
        tokio::time::timeout(Duration::from_secs(5), s.read_exact(&mut tag))
            .await
            .expect("pgwire reply timed out")
            .unwrap();
        let mut len = [0u8; 4];
        s.read_exact(&mut len).await.unwrap();
        let n = u32::from_be_bytes(len) as usize - 4;
        let mut body = vec![0u8; n];
        s.read_exact(&mut body).await.unwrap();
        let t = tag[0];
        msgs.push((t, body));
        if t == b'Z' {
            return msgs;
        }
    }
}

/// Run a simple query; returns the concatenated text of every message.
async fn pg_simple_query(s: &mut TcpStream, sql: &str) -> String {
    let mut m = vec![b'Q'];
    m.extend_from_slice(&((sql.len() + 5) as u32).to_be_bytes());
    m.extend_from_slice(sql.as_bytes());
    m.push(0);
    s.write_all(&m).await.unwrap();
    pg_read_until_ready(s)
        .await
        .into_iter()
        .map(|(t, b)| format!("{}:{}", t as char, String::from_utf8_lossy(&b)))
        .collect::<Vec<_>>()
        .join("|")
}

// ---- F6 follow-ups -------------------------------------------------------------

/// Restart member `i` on its old data directory (new sockets) and re-wire it.
async fn revive(c: &mut Cluster, i: usize) {
    let old = &c.members[i];
    let peers = c
        .members
        .iter()
        .filter(|m| m.id != old.id)
        .map(|m| m.id.clone())
        .collect();
    let node = spawn_node(&old.dir, &old.id, peers, old.engine.clone(), &old.name).await;
    c.members[i].node = node;
    c.wire(i);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn replication_factor_is_restored_when_a_node_rejoins() {
    let _serial = SERIAL.lock().await;
    let mut c = Cluster::start(3).await;
    let p = c.wait_primary(Duration::from_secs(10)).await;
    wait_in_sync(&c.members[p].node, 2).await;
    let f = (0..3).find(|&i| i != p).unwrap();
    for pk in 1..=20u64 {
        write_until_ok(&c.members[p].node, pk, Duration::from_secs(10))
            .await
            .unwrap();
    }

    // A follower dies: the configured RF stays 3, the primary says it is degraded.
    c.members[f].node.kill();
    eventually(Duration::from_secs(10), "primary reports degraded", || {
        c.members[p].node.repl_state().degraded
    })
    .await;
    for pk in 21..=30u64 {
        write_until_ok(&c.members[p].node, pk, Duration::from_secs(10))
            .await
            .unwrap();
    }
    let degraded_writes = c.members[p]
        .node
        .repl_state()
        .metrics
        .unwrap()
        .degraded_writes;
    assert!(
        degraded_writes >= 10,
        "degraded writes counted: {degraded_writes}"
    );

    // It rejoins and catches up from its own position: no snapshot, full RF back.
    revive(&mut c, f).await;
    eventually(Duration::from_secs(15), "full RF restored", || {
        let s = c.members[p].node.repl_state();
        !s.degraded && s.in_sync == 2
    })
    .await;
    let m = c.members[f].node.repl_state().metrics.unwrap();
    assert_eq!(m.snapshots_applied, 0, "{m:?}");
    for pk in 1..=30u64 {
        assert!(
            matches!(c.members[f].engine.get(pk), Ok(Some(_))),
            "pk {pk}"
        );
    }
    let before = c.members[p]
        .node
        .repl_state()
        .metrics
        .unwrap()
        .degraded_writes;
    write_until_ok(&c.members[p].node, 99, Duration::from_secs(10))
        .await
        .unwrap();
    assert_eq!(
        c.members[p]
            .node
            .repl_state()
            .metrics
            .unwrap()
            .degraded_writes,
        before,
        "writes are fully acknowledged again"
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn sync_writes_are_refused_with_insufficient_replicas_unless_degraded_ok() {
    let _serial = SERIAL.lock().await;
    DEGRADED_OK.store(false, Ordering::SeqCst);
    let c = Cluster::start(3).await;
    DEGRADED_OK.store(true, Ordering::SeqCst);
    let p = c.wait_primary(Duration::from_secs(10)).await;
    wait_in_sync(&c.members[p].node, 2).await;
    write_until_ok(&c.members[p].node, 1, Duration::from_secs(10))
        .await
        .unwrap();
    let f = (0..3).find(|&i| i != p).unwrap();
    c.members[f].node.kill();
    let mut pk = 100u64;
    let end = Instant::now() + Duration::from_secs(10);
    loop {
        pk += 1;
        match c.members[p].node.insert(pk, row(pk)) {
            Err(e) if e.contains("insufficient replicas") => break,
            _ => {
                assert!(Instant::now() < end, "write never refused");
                tokio::time::sleep(Duration::from_millis(20)).await;
            }
        }
    }
    assert!(
        matches!(c.members[p].engine.get(pk), Ok(None)),
        "a refused write is not applied"
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn follower_rejects_grpc_writes() {
    use vectorvolt_server::grpc::proto::InsertRequest;
    use vectorvolt_server::grpc::proto::write_service_client::WriteServiceClient;
    let _serial = SERIAL.lock().await;
    let c = Cluster::start(3).await;
    let p = c.wait_primary(Duration::from_secs(10)).await;
    let f = (0..3).find(|&i| i != p).unwrap();
    wait_follower_knows_primary(&c, f, p).await;
    let gate = c.members[f].node.write_gate();
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let (_tx, rx) = tokio::sync::oneshot::channel::<()>();
    let srv = vectorvolt_server::GrpcServer::standalone(addr, c.members[f].engine.clone())
        .with_write_gate(gate);
    tokio::spawn(async move {
        let _ = srv.run_with_listener(listener, rx).await;
    });
    let mut cl = WriteServiceClient::connect(format!("http://{addr}"))
        .await
        .unwrap();
    let err = cl
        .insert(InsertRequest {
            pk: 5,
            values: vec![5, 10],
        })
        .await
        .unwrap_err();
    let msg = format!("{err:?}");
    assert!(
        msg.contains("not the primary") && msg.contains(&c.members[p].client()),
        "gRPC write must be refused naming the primary: {msg}"
    );
    assert!(matches!(c.members[f].engine.get(5), Ok(None)));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn a_node_joins_and_leaves_the_failover_group_at_runtime() {
    let _serial = SERIAL.lock().await;
    let c = Cluster::start(3).await;
    let p = c.wait_primary(Duration::from_secs(10)).await;
    write_until_ok(&c.members[p].node, 1, Duration::from_secs(10))
        .await
        .unwrap();

    // A fourth node starts as a joiner, knowing only the bootstrap members.
    let tmp = TempDir::new().unwrap();
    let id4 = NodeId::new();
    let engine4 = open_engine(tmp.path());
    let boot: Vec<NodeId> = c.members.iter().map(|m| m.id.clone()).collect();
    let mut control = FailoverConfig::new(id4.clone(), tmp.path().join("failover"), boot);
    control.client_addr = "client://n3".into();
    control.join = true;
    let n4 = FailoverNode::start(FailoverNodeOpts {
        engine: engine4.clone(),
        dir: tmp.path().to_path_buf(),
        key: None,
        repl: repl_cfg(),
        control,
    })
    .await
    .unwrap();
    for m in &c.members {
        n4.controller.add_peer(
            &m.id,
            m.node.controller.raft_addr(),
            m.node.controller.repl_addr(),
            m.client(),
        );
    }
    let seeds: Vec<SocketAddr> = c
        .members
        .iter()
        .map(|m| m.node.controller.repl_addr())
        .collect();
    n4.controller.auto_join(seeds.clone());

    let end = Instant::now() + Duration::from_secs(20);
    loop {
        let st = c.members[p].node.controller.raft_status().await.unwrap();
        if st.voters.contains(&id4) {
            break;
        }
        assert!(Instant::now() < end, "joiner never became a voter: {st:?}");
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    // Every old member learned its address through the replicated control state.
    for m in &c.members {
        eventually(Duration::from_secs(10), "members learn the joiner", || {
            m.node
                .controller
                .state()
                .nodes
                .contains_key(&id4.0.to_string())
        })
        .await;
    }
    // It follows the primary like any other node.
    let e4 = engine4.clone();
    eventually(
        Duration::from_secs(15),
        "joiner replicated the data",
        move || matches!(e4.get(1), Ok(Some(_))),
    )
    .await;
    assert_eq!(n4.role(), ReplicationRole::Follower);

    // And leaves again.
    n4.controller.leave(&seeds).await.unwrap();
    let end = Instant::now() + Duration::from_secs(20);
    loop {
        let Some(st) = c.members[p].node.controller.raft_status().await else {
            break;
        };
        if !st.voters.contains(&id4) && !st.learners.contains(&id4) {
            break;
        }
        assert!(Instant::now() < end, "member never removed: {st:?}");
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    assert!(
        c.members[p]
            .node
            .controller
            .state()
            .removed
            .contains(&id4.0.to_string())
    );
}

// ---- gossip / `vvctl cluster join|leave` -> Raft membership glue ----------

fn free_udp() -> SocketAddr {
    let s = std::net::UdpSocket::bind("127.0.0.1:0").unwrap();
    s.local_addr().unwrap()
}

async fn gossip_node(
    id: &NodeId,
    bridge: Option<Arc<FailoverNode>>,
    seeds: Vec<SocketAddr>,
) -> (Arc<ClusterNode>, SocketAddr) {
    let addr = free_udp();
    let g = ClusterNode::start(addr, vec![], Some(id.clone()));
    if let Some(n) = bridge {
        assert!(g.set_membership_hook(Arc::new(RaftMembershipBridge::new(
            n.controller.clone(),
            seeds,
        ))));
    }
    g.start_gossip(vectorvolt_cluster::swim::SwimConfig::default())
        .await
        .unwrap();
    (g, addr)
}

async fn http_post(addr: SocketAddr, path: &str) -> String {
    let mut s = TcpStream::connect(addr).await.unwrap();
    let req = format!(
        "POST {path} HTTP/1.1\r\nHost: x\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
    );
    s.write_all(req.as_bytes()).await.unwrap();
    let mut out = String::new();
    s.read_to_string(&mut out).await.unwrap();
    out
}

async fn start_joiner(c: &Cluster, tmp: &Path, id4: &NodeId, join: bool) -> Arc<FailoverNode> {
    let boot: Vec<NodeId> = c.members.iter().map(|m| m.id.clone()).collect();
    let mut control = FailoverConfig::new(id4.clone(), tmp.join("failover"), boot);
    control.client_addr = "client://n3".into();
    control.join = join;
    let n4 = FailoverNode::start(FailoverNodeOpts {
        engine: open_engine(tmp),
        dir: tmp.to_path_buf(),
        key: None,
        repl: repl_cfg(),
        control,
    })
    .await
    .unwrap();
    for m in &c.members {
        n4.controller.add_peer(
            &m.id,
            m.node.controller.raft_addr(),
            m.node.controller.repl_addr(),
            m.client(),
        );
        // The members know the joiner's addresses (as an operator-provided
        // peer list would), but it is not in the Raft group yet.
        m.node.controller.add_peer(
            id4,
            n4.controller.raft_addr(),
            n4.controller.repl_addr(),
            "client://n3".into(),
        );
    }
    n4
}

async fn wait_voter(c: &Cluster, p: usize, id: &NodeId, want: bool, what: &str) {
    let end = Instant::now() + Duration::from_secs(25);
    loop {
        // The leader may move; ask whichever member currently leads.
        for m in &c.members {
            if let Some(st) = m.node.controller.raft_status().await {
                let _ = p;
                if st.voters.contains(id) == want && (want || !st.learners.contains(id)) {
                    return;
                }
            }
        }
        assert!(Instant::now() < end, "{what}");
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn cli_cluster_join_and_leave_change_raft_membership() {
    let _serial = SERIAL.lock().await;
    let c = Cluster::start(3).await;
    let p = c.wait_primary(Duration::from_secs(10)).await;
    let id4 = NodeId::new();
    let tmp = TempDir::new().unwrap();
    let n4 = start_joiner(&c, tmp.path(), &id4, true).await;
    let seeds: Vec<SocketAddr> = c
        .members
        .iter()
        .map(|m| m.node.controller.repl_addr())
        .collect();
    let (g0, a0) = gossip_node(&c.members[0].id, None, vec![]).await;
    let (g4, _) = gossip_node(&id4, Some(n4.clone()), seeds).await;
    let ctl = free_ports(1)[0];
    let g4c = g4.clone();
    tokio::spawn(async move { ClusterStatusServer::new(g4c).serve(ctl).await });
    tokio::time::sleep(Duration::from_millis(200)).await;
    let ctl_addr: SocketAddr = format!("127.0.0.1:{ctl}").parse().unwrap();

    // `vvctl cluster join --addr <seed>` == POST /cluster/join?seed=...

    let r = http_post(ctl_addr, &format!("/cluster/join?seed={a0}")).await;
    assert!(r.starts_with("HTTP/1.1 200"), "join failed: {r}");
    wait_voter(&c, p, &id4, true, "4th node never became a Raft voter").await;

    let r = http_post(ctl_addr, "/cluster/leave").await;
    assert!(r.starts_with("HTTP/1.1 200"), "leave failed: {r}");
    wait_voter(&c, p, &id4, false, "left node never removed from Raft").await;
    drop(g0);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn gossip_join_and_leave_events_change_raft_membership() {
    let _serial = SERIAL.lock().await;
    let c = Cluster::start(3).await;
    let p = c.wait_primary(Duration::from_secs(10)).await;
    let id4 = NodeId::new();
    let tmp = TempDir::new().unwrap();
    let _n4 = start_joiner(&c, tmp.path(), &id4, true).await;

    // Members gossip with a bridge each; the 4th node only gossips.
    let mut gs = Vec::new();
    for m in &c.members {
        gs.push(gossip_node(&m.id, Some(m.node.clone()), vec![]).await);
    }
    let (g4, _) = gossip_node(&id4, None, vec![]).await;
    for (g, _) in gs.iter().skip(1) {
        g.join_seed(gs[0].1).await.unwrap();
    }
    g4.join_seed(gs[0].1).await.unwrap();
    wait_voter(&c, p, &id4, true, "gossip join never added the Raft voter").await;

    g4.leave().await.unwrap();
    wait_voter(
        &c,
        p,
        &id4,
        false,
        "gossip Left never removed the Raft member",
    )
    .await;
}

struct Procs(Vec<std::process::Child>);

impl Drop for Procs {
    fn drop(&mut self) {
        for c in &mut self.0 {
            let _ = c.kill();
            let _ = c.wait();
        }
    }
}

fn free_ports(n: usize) -> Vec<u16> {
    let ls: Vec<std::net::TcpListener> = (0..n)
        .map(|_| std::net::TcpListener::bind("127.0.0.1:0").unwrap())
        .collect();
    ls.iter().map(|l| l.local_addr().unwrap().port()).collect()
}

#[test]
fn three_real_processes_fail_over_in_under_two_seconds() {
    use vectorvolt_net::client::Client;
    let tmp = TempDir::new().unwrap();
    let ports = free_ports(9);
    let ids: Vec<NodeId> = (0..3).map(|_| NodeId::new()).collect();
    // k: 0 client, 1 raft, 2 replication
    let addr = |i: usize, k: usize| format!("127.0.0.1:{}", ports[i * 3 + k]);
    let mut procs = Procs(Vec::new());
    for i in 0..3 {
        let dir = tmp.path().join(format!("p{i}"));
        let mut cmd = std::process::Command::new(env!("CARGO_BIN_EXE_vvctl"));
        cmd.arg("serve")
            .arg(&dir)
            .args(["--columns", "pk", "val", "--no-grpc", "--metrics-port", "0"])
            .args(["--addr", &addr(i, 0)])
            .args(["--cluster-node-id", &ids[i].0.to_string()])
            .args(["--failover", "--raft-listen", &addr(i, 1)])
            .args(["--replication-listen", &addr(i, 2)])
            .args(["--replication-mode", "sync", "--replication-factor", "3"])
            .arg("--replication-degraded-ok");
        for j in (0..3).filter(|&j| j != i) {
            cmd.args([
                "--failover-peer",
                &format!("{}={},{},{}", ids[j].0, addr(j, 1), addr(j, 2), addr(j, 0)),
            ]);
        }
        let log = std::fs::File::create(tmp.path().join(format!("p{i}.log"))).unwrap();
        cmd.stdout(log.try_clone().unwrap()).stderr(log);
        procs.0.push(cmd.spawn().unwrap());
    }
    let try_insert = |i: usize, pk: u64| -> bool {
        let Ok(sa) = addr(i, 0).parse::<SocketAddr>() else {
            return false;
        };
        Client::connect(sa)
            .and_then(|mut c| c.insert(pk, vec![pk, pk * 2]))
            .is_ok()
    };
    // Find the primary: the one process that accepts a write.
    let end = Instant::now() + Duration::from_secs(30);
    let p = 'find: loop {
        for i in 0..3 {
            if try_insert(i, 1) {
                break 'find i;
            }
        }
        assert!(Instant::now() < end, "no process became primary");
        std::thread::sleep(Duration::from_millis(100));
    };
    for pk in 2..=20u64 {
        let end = Instant::now() + Duration::from_secs(10);
        while !try_insert(p, pk) {
            assert!(Instant::now() < end, "write {pk} refused");
            std::thread::sleep(Duration::from_millis(20));
        }
    }
    // Both followers hold everything before the primary dies.
    let has = |i: usize, pk: u64| -> bool {
        let sa: SocketAddr = addr(i, 0).parse().unwrap();
        Client::connect(sa)
            .and_then(|mut c| c.get(pk))
            .is_ok_and(|r| r.is_some())
    };
    let end = Instant::now() + Duration::from_secs(20);
    while !(0..3).filter(|&i| i != p).all(|i| has(i, 20)) {
        assert!(Instant::now() < end, "followers never caught up");
        std::thread::sleep(Duration::from_millis(50));
    }

    // Kill the primary process; a survivor must accept a write within 2 s.
    let t0 = Instant::now();
    procs.0[p].kill().unwrap();
    let survivors: Vec<usize> = (0..3).filter(|&i| i != p).collect();
    let (new_p, took) = 'wait: loop {
        for &i in &survivors {
            if try_insert(i, 1000) {
                break 'wait (i, t0.elapsed());
            }
        }
        assert!(t0.elapsed() < Duration::from_secs(10), "no new primary");
        std::thread::sleep(Duration::from_millis(5));
    };
    println!("process failover: n{new_p} accepted a write after {took:?}");
    assert!(took < MAX_FAILOVER, "failover took {took:?}");
    let other = survivors.into_iter().find(|&i| i != new_p).unwrap();
    let end = Instant::now() + Duration::from_secs(15);
    while !(has(other, 1000) && has(other, 5)) {
        assert!(Instant::now() < end, "second survivor never converged");
        std::thread::sleep(Duration::from_millis(50));
    }
}
