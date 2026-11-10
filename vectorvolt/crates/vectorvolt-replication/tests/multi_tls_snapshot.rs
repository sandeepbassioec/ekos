//! F5 leftovers — multi-table replication, TLS/mTLS, consistent snapshots and
//! the cheap sync wait.

use std::collections::BTreeMap;
use std::net::SocketAddr;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use rcgen::{BasicConstraints, CertificateParams, DnType, ExtendedKeyUsagePurpose, IsCa, KeyPair};
use tempfile::TempDir;
use vectorvolt_engine::{Durability, EngineOptions, TableEngine, TableRegistry};
use vectorvolt_replication::{
    load_position, save_position, Message, MultiReplicationClient, MultiReplicationServer,
    ReplicationClient, ReplicationConfig, ReplicationMode, ReplicationRole, ReplicationServer,
    ReplicationTlsClient, ReplicationTlsServer, RoleCell,
};

const T: Duration = Duration::from_secs(10);

fn opts() -> EngineOptions {
    EngineOptions::new(1_048_576).with_durability(Durability::WriteOnly)
}

fn schema() -> Vec<String> {
    vec!["pk".into(), "val".into()]
}

fn cfg(mode: ReplicationMode, rf: usize) -> ReplicationConfig {
    ReplicationConfig {
        mode,
        replication_factor: rf,
        ack_timeout: Duration::from_millis(3000),
        poll_interval: Duration::from_millis(5),
        heartbeat_interval: Duration::from_millis(100),
        backoff_initial: Duration::from_millis(20),
        backoff_max: Duration::from_millis(150),
        read_timeout: Duration::from_secs(5),
        batch_frames: 64,
        snapshot_batch_rows: 50,
        degraded_ok: false,
    }
}

fn leader_role() -> RoleCell {
    RoleCell::new(ReplicationRole::Leader)
}

fn follower_role() -> RoleCell {
    RoleCell::new(ReplicationRole::Follower)
}

fn registry(dir: &Path) -> Arc<TableRegistry> {
    Arc::new(TableRegistry::open(dir, opts()).unwrap())
}

async fn eventually(what: &str, mut f: impl FnMut() -> bool) {
    let deadline = Instant::now() + T;
    while !f() {
        assert!(Instant::now() < deadline, "timed out waiting for {what}");
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
}

/// Every live row of `e`, by pk (scan can repeat pks; `get` is the authority).
fn rows(e: &TableEngine) -> BTreeMap<u64, Vec<u64>> {
    let mut out = BTreeMap::new();
    for r in e.scan(0, 0, u64::MAX).unwrap() {
        if let Some(cur) = e.get(r.pk).unwrap() {
            out.insert(cur.pk, cur.values);
        }
    }
    out
}

fn assert_tables_equal(l: &TableRegistry, f: &TableRegistry, name: &str) {
    let a = rows(&l.get(name).unwrap());
    let b = rows(&f.get(name).unwrap());
    assert_eq!(a, b, "table {name} diverged");
}

// ── multi-table ───────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn multi_table_converges_including_tables_created_and_dropped_mid_stream() {
    let (ld, fd) = (TempDir::new().unwrap(), TempDir::new().unwrap());
    let lreg = registry(ld.path());
    let freg = registry(fd.path());
    let a = lreg.create_table("alpha", schema()).unwrap();
    let b = lreg.create_table("beta", schema()).unwrap();
    for pk in 1..=40u64 {
        a.insert(pk, vec![pk, pk * 2]).unwrap();
    }
    for pk in 1..=7u64 {
        b.insert(pk, vec![pk, pk * 3]).unwrap();
    }

    let server = MultiReplicationServer::start(
        Arc::clone(&lreg),
        cfg(ReplicationMode::Async, 2),
        leader_role(),
        "127.0.0.1:0".parse().unwrap(),
    )
    .await
    .unwrap();
    let client = MultiReplicationClient::start(
        Arc::clone(&freg),
        cfg(ReplicationMode::Async, 2),
        follower_role(),
        server.local_addr().to_string(),
    )
    .unwrap();

    let last_a = a.insert(41, vec![41, 0]).unwrap().sequence;
    let last_b = b.insert(8, vec![8, 24]).unwrap().sequence;
    assert!(client.wait_for_lsn("alpha", last_a, T).await);
    assert!(client.wait_for_lsn("beta", last_b, T).await);
    assert_tables_equal(&lreg, &freg, "alpha");
    assert_tables_equal(&lreg, &freg, "beta");

    // A table created after the follower connected.
    let c = lreg.create_table("gamma", schema()).unwrap();
    let mut last_c = 0;
    for pk in 1..=25u64 {
        last_c = c.insert(pk, vec![pk, pk + 1000]).unwrap().sequence;
    }
    assert!(client.wait_for_lsn("gamma", last_c, T).await);
    assert_tables_equal(&lreg, &freg, "gamma");

    // Each table has its own persisted position.
    let pos = |t: &str| load_position(&fd.path().join(t));
    eventually("positions persisted", || {
        pos("alpha") >= last_a && pos("beta") >= last_b && pos("gamma") >= last_c
    })
    .await;
    assert_ne!(pos("alpha"), pos("beta"));

    // Mutations keep flowing to all tables.
    a.delete(3).unwrap();
    let last_a = a.insert(4, vec![4, 444]).unwrap().sequence;
    assert!(client.wait_for_lsn("alpha", last_a, T).await);
    assert_tables_equal(&lreg, &freg, "alpha");

    // Drop propagates.
    lreg.drop_table("beta").unwrap();
    eventually("beta dropped on the follower", || !freg.contains("beta")).await;
    assert!(client.applied_lsn("beta").is_none());
    assert!(freg.contains("alpha") && freg.contains("gamma"));

    // Re-creating the dropped name (the leader reuses the old directory, so
    // it recovers the old rows too) converges to exactly the leader's state.
    let b2 = lreg.create_table("beta", schema()).unwrap();
    let last_b2 = b2.insert(100, vec![100, 1]).unwrap().sequence;
    eventually("beta re-created on the follower", || freg.contains("beta")).await;
    assert!(client.wait_for_lsn("beta", last_b2, T).await);
    assert_tables_equal(&lreg, &freg, "beta");

    drop(client);
    drop(server);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn multi_table_follower_restart_resumes_each_table() {
    let (ld, fd) = (TempDir::new().unwrap(), TempDir::new().unwrap());
    let lreg = registry(ld.path());
    let a = lreg.create_table("alpha", schema()).unwrap();
    let b = lreg.create_table("beta", schema()).unwrap();
    let server = MultiReplicationServer::start(
        Arc::clone(&lreg),
        cfg(ReplicationMode::Async, 2),
        leader_role(),
        "127.0.0.1:0".parse().unwrap(),
    )
    .await
    .unwrap();
    let addr = server.local_addr().to_string();
    let (mut la, mut lb) = (0, 0);
    for pk in 1..=20u64 {
        la = a.insert(pk, vec![pk, pk]).unwrap().sequence;
        lb = b.insert(pk, vec![pk, pk + 1]).unwrap().sequence;
    }
    {
        let freg = registry(fd.path());
        let client = MultiReplicationClient::start(
            Arc::clone(&freg),
            cfg(ReplicationMode::Async, 2),
            follower_role(),
            addr.clone(),
        )
        .unwrap();
        assert!(client.wait_for_lsn("alpha", la, T).await);
        assert!(client.wait_for_lsn("beta", lb, T).await);
        client.shutdown();
    }
    for pk in 21..=30u64 {
        la = a.insert(pk, vec![pk, pk]).unwrap().sequence;
    }
    let freg = registry(fd.path());
    assert!(freg.contains("alpha") && freg.contains("beta"));
    let client = MultiReplicationClient::start(
        Arc::clone(&freg),
        cfg(ReplicationMode::Async, 2),
        follower_role(),
        addr,
    )
    .unwrap();
    assert!(client.wait_for_lsn("alpha", la, T).await);
    assert_tables_equal(&lreg, &freg, "alpha");
    assert_tables_equal(&lreg, &freg, "beta");
    for t in ["alpha", "beta"] {
        let m = client.table_metrics(t).unwrap().snapshot();
        assert_eq!(m.snapshots_applied, 0, "{t}: resume must not snapshot");
    }
    // Beta saw no new writes: at most a trailing housekeeping frame is
    // shipped, never its 20 rows again.
    let beta_frames = client
        .table_metrics("beta")
        .unwrap()
        .snapshot()
        .frames_applied;
    assert!(beta_frames <= 2, "beta re-shipped {beta_frames} frames");
}

// ── TLS / mTLS ────────────────────────────────────────────────────────────────

struct Pki {
    ca_pem: String,
    server_cert: String,
    server_key: String,
    client_cert: String,
    client_key: String,
}

fn pki() -> Pki {
    let ca_key = KeyPair::generate().unwrap();
    let mut ca = CertificateParams::new(vec![]).unwrap();
    ca.is_ca = IsCa::Ca(BasicConstraints::Unconstrained);
    ca.distinguished_name.push(DnType::CommonName, "vv test ca");
    let ca_cert = ca.self_signed(&ca_key).unwrap();

    let leaf = |name: &str, eku: ExtendedKeyUsagePurpose| {
        let key = KeyPair::generate().unwrap();
        let mut p = CertificateParams::new(vec![name.to_owned()]).unwrap();
        p.extended_key_usages = vec![eku];
        let cert = p.signed_by(&key, &ca_cert, &ca_key).unwrap();
        (cert.pem(), key.serialize_pem())
    };
    let (server_cert, server_key) = leaf("localhost", ExtendedKeyUsagePurpose::ServerAuth);
    let (client_cert, client_key) = leaf("follower", ExtendedKeyUsagePurpose::ClientAuth);
    Pki {
        ca_pem: ca_cert.pem(),
        server_cert,
        server_key,
        client_cert,
        client_key,
    }
}

fn engine(dir: &Path) -> Arc<TableEngine> {
    Arc::new(TableEngine::open_with(schema(), dir, opts()).unwrap())
}

async fn tls_leader(
    le: &Arc<TableEngine>,
    dir: &Path,
    tls: Option<ReplicationTlsServer>,
) -> Arc<ReplicationServer> {
    ReplicationServer::start_tls(
        Arc::clone(le),
        dir.join("wal"),
        cfg(ReplicationMode::Async, 2),
        leader_role(),
        "127.0.0.1:0".parse().unwrap(),
        tls,
    )
    .await
    .unwrap()
}

fn tls_follower(
    fe: &Arc<TableEngine>,
    dir: &Path,
    addr: SocketAddr,
    tls: Option<ReplicationTlsClient>,
) -> Arc<ReplicationClient> {
    ReplicationClient::start_tls(
        Arc::clone(fe),
        dir,
        None,
        cfg(ReplicationMode::Async, 2),
        follower_role(),
        addr.to_string(),
        tls,
    )
    .unwrap()
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn tls_replication_converges() {
    let p = pki();
    let (ld, fd) = (TempDir::new().unwrap(), TempDir::new().unwrap());
    let (le, fe) = (engine(ld.path()), engine(fd.path()));
    let server = tls_leader(
        &le,
        ld.path(),
        Some(
            ReplicationTlsServer::from_pem(p.server_cert.as_bytes(), p.server_key.as_bytes(), None)
                .unwrap(),
        ),
    )
    .await;
    let mut last = 0;
    for pk in 1..=80u64 {
        last = le.insert(pk, vec![pk, pk * 7]).unwrap().sequence;
    }
    let client = tls_follower(
        &fe,
        fd.path(),
        server.local_addr(),
        Some(
            ReplicationTlsClient::from_pem(p.ca_pem.as_bytes(), None, Some("localhost".into()))
                .unwrap(),
        ),
    );
    assert!(
        client.wait_for_lsn(last, T).await,
        "{:?}",
        client.last_error()
    );
    assert_eq!(rows(&le), rows(&fe));

    // A plaintext follower cannot talk to a TLS leader.
    let (pd, pe) = (
        TempDir::new().unwrap(),
        engine(&TempDir::new().unwrap().keep()),
    );
    let plain = tls_follower(&pe, pd.path(), server.local_addr(), None);
    tokio::time::sleep(Duration::from_millis(500)).await;
    assert_eq!(plain.applied_lsn(), 0);
    assert!(rows(&pe).is_empty());

    // A follower that does not trust the leader's CA is refused too.
    let other = pki();
    let (bd, be) = (
        TempDir::new().unwrap(),
        engine(&TempDir::new().unwrap().keep()),
    );
    let untrusting = tls_follower(
        &be,
        bd.path(),
        server.local_addr(),
        Some(
            ReplicationTlsClient::from_pem(other.ca_pem.as_bytes(), None, Some("localhost".into()))
                .unwrap(),
        ),
    );
    tokio::time::sleep(Duration::from_millis(500)).await;
    assert_eq!(untrusting.applied_lsn(), 0);
    assert!(untrusting.last_error().is_some());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn mtls_requires_a_client_certificate() {
    let p = pki();
    let (ld, fd) = (TempDir::new().unwrap(), TempDir::new().unwrap());
    let (le, fe) = (engine(ld.path()), engine(fd.path()));
    let server_tls = ReplicationTlsServer::from_pem(
        p.server_cert.as_bytes(),
        p.server_key.as_bytes(),
        Some(p.ca_pem.as_bytes()),
    )
    .unwrap();
    assert!(server_tls.requires_client_cert());
    let server = tls_leader(&le, ld.path(), Some(server_tls)).await;
    let mut last = 0;
    for pk in 1..=30u64 {
        last = le.insert(pk, vec![pk, pk]).unwrap().sequence;
    }

    // No client certificate: the leader rejects the handshake.
    let (nd, ne) = (
        TempDir::new().unwrap(),
        engine(&TempDir::new().unwrap().keep()),
    );
    let anon = tls_follower(
        &ne,
        nd.path(),
        server.local_addr(),
        Some(
            ReplicationTlsClient::from_pem(p.ca_pem.as_bytes(), None, Some("localhost".into()))
                .unwrap(),
        ),
    );
    tokio::time::sleep(Duration::from_millis(600)).await;
    assert_eq!(anon.applied_lsn(), 0);
    assert!(rows(&ne).is_empty(), "anonymous follower received data");
    anon.shutdown();

    // With a certificate signed by the client CA it replicates.
    let client = tls_follower(
        &fe,
        fd.path(),
        server.local_addr(),
        Some(
            ReplicationTlsClient::from_pem(
                p.ca_pem.as_bytes(),
                Some((p.client_cert.as_bytes(), p.client_key.as_bytes())),
                Some("localhost".into()),
            )
            .unwrap(),
        ),
    );
    assert!(
        client.wait_for_lsn(last, T).await,
        "{:?}",
        client.last_error()
    );
    assert_eq!(rows(&le), rows(&fe));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn multi_table_replication_over_mtls() {
    let p = pki();
    let (ld, fd) = (TempDir::new().unwrap(), TempDir::new().unwrap());
    let lreg = registry(ld.path());
    let freg = registry(fd.path());
    let t = lreg.create_table("orders", schema()).unwrap();
    let server = MultiReplicationServer::start_tls(
        Arc::clone(&lreg),
        cfg(ReplicationMode::Async, 2),
        leader_role(),
        "127.0.0.1:0".parse().unwrap(),
        Some(
            ReplicationTlsServer::from_pem(
                p.server_cert.as_bytes(),
                p.server_key.as_bytes(),
                Some(p.ca_pem.as_bytes()),
            )
            .unwrap(),
        ),
    )
    .await
    .unwrap();
    let client = MultiReplicationClient::start_tls(
        Arc::clone(&freg),
        cfg(ReplicationMode::Async, 2),
        follower_role(),
        server.local_addr().to_string(),
        Some(
            ReplicationTlsClient::from_pem(
                p.ca_pem.as_bytes(),
                Some((p.client_cert.as_bytes(), p.client_key.as_bytes())),
                Some("localhost".into()),
            )
            .unwrap(),
        ),
    )
    .unwrap();
    let mut last = 0;
    for pk in 1..=20u64 {
        last = t.insert(pk, vec![pk, pk]).unwrap().sequence;
    }
    assert!(client.wait_for_lsn("orders", last, T).await);
    assert_tables_equal(&lreg, &freg, "orders");
}

// ── snapshot consistency ──────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn snapshot_under_concurrent_writes_yields_an_exact_replica() {
    let (ld, fd) = (TempDir::new().unwrap(), TempDir::new().unwrap());
    let (le, fe) = (engine(ld.path()), engine(fd.path()));
    for pk in 0..2000u64 {
        le.insert(pk, vec![pk, pk]).unwrap();
    }
    // A stale row the snapshot must wipe, and a position that forces a
    // snapshot (it is ahead of the leader).
    fe.insert(999_999, vec![999_999, 1]).unwrap();
    save_position(fd.path(), 10_000_000).unwrap();

    let server = tls_leader(&le, ld.path(), None).await;
    let stop = Arc::new(AtomicBool::new(false));
    // Hammer the same keys with inserts, overwrites and deletes while the
    // follower connects, snapshots and streams.
    let writer = {
        let (le, stop) = (Arc::clone(&le), Arc::clone(&stop));
        tokio::task::spawn_blocking(move || {
            let mut i = 0u64;
            let mut last = 0;
            while !stop.load(Ordering::Relaxed) || i < 3000 {
                let pk = (i * 7) % 2500;
                // (A delete of an absent pk reports sequence 0.)
                let seq = if i % 4 == 3 {
                    le.delete(pk).unwrap().sequence
                } else {
                    le.insert(pk, vec![pk, i]).unwrap().sequence
                };
                last = last.max(seq);
                i += 1;
                if i.is_multiple_of(64) {
                    std::thread::sleep(Duration::from_micros(200));
                }
            }
            last
        })
    };
    tokio::time::sleep(Duration::from_millis(30)).await;
    let client = tls_follower(&fe, fd.path(), server.local_addr(), None);
    tokio::time::sleep(Duration::from_millis(300)).await;
    stop.store(true, Ordering::Relaxed);
    let last = writer.await.unwrap();
    server.notify_write();
    let m = client.metrics();
    eventually("snapshot installed", || m.snapshot().snapshots_applied >= 1).await;
    assert!(
        client.wait_for_lsn(last, T).await,
        "{:?}",
        client.last_error()
    );
    assert_eq!(m.snapshot().snapshots_applied, 1);
    assert_eq!(rows(&le), rows(&fe), "replica is not an exact copy");
    assert!(fe.get(999_999).unwrap().is_none());
}

// ── sync wait ─────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn async_mode_wait_returns_immediately() {
    let ld = TempDir::new().unwrap();
    let le = engine(ld.path());
    let server = tls_leader(&le, ld.path(), None).await;
    let start = Instant::now();
    for lsn in 0..200_000u64 {
        server.wait_for_replication(lsn).await.unwrap();
    }
    let per_call = start.elapsed() / 200_000;
    assert!(
        per_call < Duration::from_micros(5),
        "async wait costs {per_call:?} per call"
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn sync_wait_wakes_on_ack_and_acked_lsns_are_free() {
    let ld = TempDir::new().unwrap();
    let le = engine(ld.path());
    let mut c = cfg(ReplicationMode::Sync, 2);
    c.ack_timeout = Duration::from_secs(10);
    let server = ReplicationServer::start(
        Arc::clone(&le),
        ld.path().join("wal"),
        c,
        leader_role(),
        "127.0.0.1:0".parse().unwrap(),
    )
    .await
    .unwrap();

    // A scripted follower, so the test controls exactly when the ack lands.
    let conn = tokio::net::TcpStream::connect(server.local_addr())
        .await
        .unwrap();
    let (mut rd, mut wr) = conn.into_split();
    send_msg(
        &mut wr,
        &Message::Hello {
            version: vectorvolt_replication::PROTO_VERSION,
            applied_lsn: 0,
            epoch: 0,
            history: vec![],
        },
    )
    .await;
    let welcome = recv_msg(&mut rd).await;
    assert!(matches!(welcome, Message::Welcome { .. }));

    let lsn = le.insert(1, vec![1, 1]).unwrap().sequence;
    let waiter = {
        let server = Arc::clone(&server);
        tokio::spawn(async move {
            let t = Instant::now();
            server.wait_for_replication(lsn).await.unwrap();
            t.elapsed()
        })
    };
    tokio::time::sleep(Duration::from_millis(300)).await;
    assert!(!waiter.is_finished(), "returned before any follower acked");

    // Drain the frame, then ack durably.
    loop {
        if let Message::Frames { .. } = recv_msg(&mut rd).await {
            break;
        }
    }
    let acked_at = Instant::now();
    send_msg(
        &mut wr,
        &Message::Ack {
            epoch: 0,
            applied_lsn: lsn,
            durable_lsn: lsn,
        },
    )
    .await;
    let waited = tokio::time::timeout(Duration::from_secs(2), waiter)
        .await
        .expect("waiter never woke")
        .unwrap();
    assert!(waited >= Duration::from_millis(250));
    assert!(
        acked_at.elapsed() < Duration::from_millis(500),
        "wake took {:?} after the ack",
        acked_at.elapsed()
    );

    // Already-acked LSNs cost one atomic load.
    let start = Instant::now();
    for _ in 0..100_000 {
        server.wait_for_replication(lsn).await.unwrap();
    }
    assert!(
        start.elapsed() / 100_000 < Duration::from_micros(5),
        "acked wait costs {:?}",
        start.elapsed() / 100_000
    );
}

async fn send_msg(w: &mut tokio::net::tcp::OwnedWriteHalf, m: &Message) {
    use tokio::io::AsyncWriteExt;
    w.write_all(&m.encode()).await.unwrap();
}

async fn recv_msg(r: &mut tokio::net::tcp::OwnedReadHalf) -> Message {
    use tokio::io::AsyncReadExt;
    let mut head = [0u8; 8];
    r.read_exact(&mut head).await.unwrap();
    let len = u32::from_le_bytes(head[4..8].try_into().unwrap()) as usize;
    let mut buf = head.to_vec();
    buf.resize(8 + len + 4, 0);
    r.read_exact(&mut buf[8..]).await.unwrap();
    Message::decode(&buf).unwrap()
}
