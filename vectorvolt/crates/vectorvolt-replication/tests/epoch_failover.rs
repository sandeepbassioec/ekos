//! F6 follow-ups — per-frame epochs, incremental catch-up after a primary
//! change, and explicit degraded replication, over 127.0.0.1.

use std::net::SocketAddr;
use std::path::Path;
use std::sync::Arc;
use std::time::{Duration, Instant};

use parking_lot::Mutex;
use tempfile::TempDir;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use vectorvolt_engine::{Durability, EngineOptions, TableEngine};
use vectorvolt_replication::{
    save_position_full, EpochCell, Lineage, LineageStore, Message, Position, ReplicationClient,
    ReplicationConfig, ReplicationMode, ReplicationRole, ReplicationServer, RoleCell, WireFrame,
    PROTO_VERSION,
};
use vectorvolt_txn::Mutation;

const T: Duration = Duration::from_secs(10);

fn engine(dir: &Path) -> Arc<TableEngine> {
    let opts = EngineOptions::new(1_048_576).with_durability(Durability::WriteOnly);
    Arc::new(TableEngine::open_with(vec!["pk".into(), "val".into()], dir, opts).unwrap())
}

fn cfg(mode: ReplicationMode, rf: usize) -> ReplicationConfig {
    ReplicationConfig {
        mode,
        replication_factor: rf,
        ack_timeout: Duration::from_millis(1500),
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

async fn wait_for(what: &str, mut f: impl FnMut() -> bool) {
    let end = Instant::now() + T;
    while !f() {
        assert!(Instant::now() < end, "timed out waiting for {what}");
        tokio::time::sleep(Duration::from_millis(5)).await;
    }
}

async fn read_msg(s: &mut TcpStream) -> Message {
    let mut head = [0u8; 8];
    s.read_exact(&mut head).await.unwrap();
    let len = u32::from_le_bytes(head[4..8].try_into().unwrap()) as usize;
    let mut rest = vec![0u8; len + 4];
    s.read_exact(&mut rest).await.unwrap();
    let mut all = head.to_vec();
    all.extend_from_slice(&rest);
    Message::decode(&all).unwrap()
}

fn insert_frame(epoch: u64, seq: u64, pk: u64) -> Message {
    Message::Frames {
        epoch,
        tip: seq,
        frames: vec![WireFrame {
            seq,
            timestamp_ns: 0,
            key_id: None,
            payload: Mutation::Insert {
                pk,
                values: vec![pk, pk * 10],
            }
            .encode(),
        }],
    }
}

fn welcome(epoch: u64, tip: u64) -> Message {
    Message::Welcome {
        first_lsn: 1,
        tip,
        snapshot: false,
        start_lsn: 0,
        epoch,
        truncate: false,
        common_epoch: 0,
        common_lsn: 0,
    }
}

// ---- per-frame epoch ---------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn stale_epoch_frame_mid_stream_is_dropped() {
    let fd = TempDir::new().unwrap();
    let fe = engine(fd.path());
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let fake = tokio::spawn(async move {
        let (mut s, _) = listener.accept().await.unwrap();
        assert!(matches!(read_msg(&mut s).await, Message::Hello { .. }));
        s.write_all(&welcome(5, 2).encode()).await.unwrap();
        s.write_all(&insert_frame(5, 1, 7).encode()).await.unwrap();
        tokio::time::sleep(Duration::from_millis(300)).await;
        // An old primary's frame lands in the middle of the stream.
        s.write_all(&insert_frame(4, 2, 8).encode()).await.unwrap();
        tokio::time::sleep(Duration::from_secs(3)).await;
    });
    let known = EpochCell::new(0);
    let lineage: Arc<LineageStore> = Arc::new(Mutex::new(Lineage::open(fd.path()).unwrap()));
    let client = ReplicationClient::start_epoch(
        Arc::clone(&fe),
        fd.path(),
        None,
        cfg(ReplicationMode::Async, 2),
        RoleCell::new(ReplicationRole::Follower),
        addr.to_string(),
        known.clone(),
        lineage,
    )
    .unwrap();
    wait_for("the stale frame to be dropped", || {
        client.metrics().snapshot().stale_epoch_frames >= 1
    })
    .await;
    assert!(fe.get(7).unwrap().is_some(), "the in-epoch frame applied");
    assert!(fe.get(8).unwrap().is_none(), "the stale frame was applied");
    assert_eq!(client.applied_lsn(), 1);
    assert_eq!(known.get(), 5, "the follower learned the stream's epoch");
    fake.abort();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn a_primary_at_a_lower_epoch_than_known_is_refused_at_the_welcome() {
    let fd = TempDir::new().unwrap();
    let fe = engine(fd.path());
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let fake = tokio::spawn(async move {
        loop {
            let (mut s, _) = listener.accept().await.unwrap();
            let _ = read_msg(&mut s).await;
            let _ = s.write_all(&welcome(3, 1).encode()).await;
            let _ = s.write_all(&insert_frame(3, 1, 7).encode()).await;
            tokio::time::sleep(Duration::from_millis(200)).await;
        }
    });
    let client = ReplicationClient::start_epoch(
        Arc::clone(&fe),
        fd.path(),
        None,
        cfg(ReplicationMode::Async, 2),
        RoleCell::new(ReplicationRole::Follower),
        addr.to_string(),
        EpochCell::new(5),
        Arc::new(Mutex::new(Lineage::open(fd.path()).unwrap())),
    )
    .unwrap();
    wait_for("the stale welcome to be refused", || {
        client.metrics().snapshot().stale_epoch_frames >= 1
    })
    .await;
    assert!(fe.get(7).unwrap().is_none());
    fake.abort();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn primary_drops_acks_from_another_epoch() {
    let ld = TempDir::new().unwrap();
    let le = engine(ld.path());
    let lineage: Arc<LineageStore> = Arc::new(Mutex::new(Lineage::open(ld.path()).unwrap()));
    let server = ReplicationServer::start_epoch(
        Arc::clone(&le),
        ld.path().join("wal"),
        cfg(ReplicationMode::Async, 2),
        RoleCell::new(ReplicationRole::Leader),
        "127.0.0.1:0".parse().unwrap(),
        7,
        lineage,
    )
    .await
    .unwrap();
    for pk in 1..=3u64 {
        le.insert(pk, vec![pk, pk]).unwrap();
    }
    let mut s = TcpStream::connect(server.local_addr()).await.unwrap();
    let hello = Message::Hello {
        version: PROTO_VERSION,
        applied_lsn: 0,
        epoch: 7,
        history: vec![(7, 0)],
    };
    s.write_all(&hello.encode()).await.unwrap();
    assert!(matches!(
        read_msg(&mut s).await,
        Message::Welcome { epoch: 7, .. }
    ));
    let ack = |epoch| Message::Ack {
        epoch,
        applied_lsn: 3,
        durable_lsn: 3,
    };
    s.write_all(&ack(6).encode()).await.unwrap();
    wait_for("the stale ack to be counted", || {
        server.metrics().snapshot().stale_acks == 1
    })
    .await;
    assert_eq!(
        server.followers()[0].applied_lsn,
        0,
        "a stale-epoch ack must not advance the follower"
    );
    s.write_all(&ack(7).encode()).await.unwrap();
    wait_for("the in-epoch ack to count", || {
        server.followers()[0].applied_lsn == 3
    })
    .await;
    assert_eq!(server.metrics().snapshot().stale_acks, 1);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn a_v1_hello_gets_a_clear_error() {
    let ld = TempDir::new().unwrap();
    let le = engine(ld.path());
    let server = ReplicationServer::start(
        le,
        ld.path().join("wal"),
        cfg(ReplicationMode::Async, 2),
        RoleCell::new(ReplicationRole::Leader),
        "127.0.0.1:0".parse().unwrap(),
    )
    .await
    .unwrap();
    let mut s = TcpStream::connect(server.local_addr()).await.unwrap();
    let v1 = Message::Hello {
        version: 1,
        applied_lsn: 0,
        epoch: 0,
        history: vec![],
    };
    s.write_all(&v1.encode()).await.unwrap();
    match read_msg(&mut s).await {
        Message::Error(e) => assert!(e.contains("v1") && e.contains("upgrade"), "{e}"),
        m => panic!("expected an error, got {m:?}"),
    }
}

// ---- incremental catch-up ----------------------------------------------------

struct Node {
    dir: TempDir,
    engine: Arc<TableEngine>,
    lineage: Arc<LineageStore>,
    role: RoleCell,
}

fn node() -> Node {
    let dir = TempDir::new().unwrap();
    let engine = engine(dir.path());
    let lineage = Arc::new(Mutex::new(Lineage::open(dir.path()).unwrap()));
    Node {
        dir,
        engine,
        lineage,
        role: RoleCell::new(ReplicationRole::Follower),
    }
}

impl Node {
    async fn lead(&self, epoch: u64) -> Arc<ReplicationServer> {
        self.lineage
            .lock()
            .start_primary(epoch, self.engine.wal_tip_sequence())
            .unwrap();
        self.role.set(ReplicationRole::Leader);
        ReplicationServer::start_epoch(
            Arc::clone(&self.engine),
            self.dir.path().join("wal"),
            cfg(ReplicationMode::Async, 3),
            self.role.clone(),
            "127.0.0.1:0".parse().unwrap(),
            epoch,
            Arc::clone(&self.lineage),
        )
        .await
        .unwrap()
    }

    fn follow(&self, addr: SocketAddr, known: &EpochCell) -> Arc<ReplicationClient> {
        self.role.set(ReplicationRole::Follower);
        ReplicationClient::start_epoch(
            Arc::clone(&self.engine),
            self.dir.path(),
            None,
            cfg(ReplicationMode::Async, 3),
            self.role.clone(),
            addr.to_string(),
            known.clone(),
            Arc::clone(&self.lineage),
        )
        .unwrap()
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn after_a_primary_change_only_the_tail_is_transferred() {
    let (a, b, c) = (node(), node(), node());
    let sa = a.lead(1).await;
    let (kb, kc) = (EpochCell::new(0), EpochCell::new(0));
    let cb = b.follow(sa.local_addr(), &kb);
    let cc = c.follow(sa.local_addr(), &kc);
    for pk in 1..=300u64 {
        a.engine.insert(pk, vec![pk, pk * 2]).unwrap();
    }
    sa.notify_write();
    let tip_a = a.engine.wal_tip_sequence();
    assert!(cb.wait_for_lsn(tip_a, T).await && cc.wait_for_lsn(tip_a, T).await);

    // A crashes holding one write nobody else ever saw.
    sa.shutdown();
    cb.shutdown();
    cc.shutdown();
    a.engine.insert(9001, vec![9001, 1]).unwrap();
    a.lineage
        .lock()
        .end_primary(1, a.engine.wal_tip_sequence())
        .unwrap();
    save_position_full(
        a.dir.path(),
        &Position {
            epoch: 1,
            lsn: a.engine.wal_tip_sequence(),
            primary: false,
        },
    )
    .unwrap();

    // B is promoted at epoch 2 and takes 20 more writes.
    kb.advance(2);
    let sb = b.lead(2).await;
    for pk in 1000..1020u64 {
        b.engine.insert(pk, vec![pk, pk * 2]).unwrap();
    }

    // C follows the new primary: no snapshot, only B's 20 new rows.
    let cc2 = c.follow(sb.local_addr(), &kc);
    let tip_b = b.engine.wal_tip_sequence();
    sb.notify_write();
    wait_for("C to stream the tail", || {
        cc2.metrics().snapshot().rows_applied >= 20
    })
    .await;
    assert_eq!(cc2.applied_lsn(), tip_b);
    let m = cc2.metrics().snapshot();
    assert_eq!(m.snapshots_applied, 0, "no snapshot: {m:?}");
    assert_eq!(m.snapshot_rows, 0);
    assert_eq!(m.fix_rows, 0, "C had no divergent tail");
    assert_eq!(m.rows_applied, 20, "only the tail was streamed: {m:?}");
    assert_eq!(m.incremental_resumes, 1);
    for pk in (1..=300u64).chain(1000..1020) {
        assert_eq!(
            c.engine.get(pk).unwrap().map(|r| r.values),
            b.engine.get(pk).unwrap().map(|r| r.values),
            "pk {pk}"
        );
    }

    // The old primary returns with its un-acked tail: that tail (and only
    // that) is rewritten, the rest is streamed.
    let ka = EpochCell::new(1);
    let ca = a.follow(sb.local_addr(), &ka);
    wait_for("A to stream the tail", || {
        ca.metrics().snapshot().rows_applied >= 20
    })
    .await;
    assert_eq!(ca.applied_lsn(), tip_b);
    let m = ca.metrics().snapshot();
    assert_eq!(m.snapshots_applied, 0, "no snapshot: {m:?}");
    assert_eq!(m.snapshot_rows, 0);
    assert_eq!(m.fix_rows, 1, "exactly the one divergent key: {m:?}");
    assert_eq!(m.rows_applied, 20, "{m:?}");
    assert!(a.engine.get(9001).unwrap().is_none(), "tail truncated");
    for pk in (1..=300u64).chain(1000..1020) {
        assert_eq!(
            a.engine.get(pk).unwrap().map(|r| r.values),
            b.engine.get(pk).unwrap().map(|r| r.values),
            "pk {pk}"
        );
    }
    assert_eq!(ka.get(), 2);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn without_history_the_follower_falls_back_to_a_snapshot() {
    let (b, c) = (node(), node());
    // C holds rows from some unrelated past; B has no lineage entry for it.
    for pk in 1..=50u64 {
        c.engine.insert(pk, vec![pk, 1]).unwrap();
        b.engine.insert(pk, vec![pk, 2]).unwrap();
    }
    save_position_full(
        c.dir.path(),
        &Position {
            epoch: 1,
            lsn: 50,
            primary: false,
        },
    )
    .unwrap();
    let sb = b.lead(2).await;
    let cc = c.follow(sb.local_addr(), &EpochCell::new(1));
    wait_for("snapshot", || {
        cc.metrics().snapshot().snapshots_applied == 1
    })
    .await;
    for pk in 1..=50u64 {
        assert_eq!(c.engine.get(pk).unwrap().unwrap().values[1], 2);
    }
}

// ---- replication factor ------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn degraded_mode_is_explicit_and_full_rf_returns_when_followers_catch_up() {
    let a = node();
    let mut c = cfg(ReplicationMode::Sync, 3);
    c.degraded_ok = true;
    a.role.set(ReplicationRole::Leader);
    let sa = ReplicationServer::start_epoch(
        Arc::clone(&a.engine),
        a.dir.path().join("wal"),
        c,
        a.role.clone(),
        "127.0.0.1:0".parse().unwrap(),
        1,
        Arc::clone(&a.lineage),
    )
    .await
    .unwrap();
    assert!(sa.is_degraded(), "no followers: below the configured RF");

    // Writes are accepted, visibly degraded.
    sa.insert_replicated(1, vec![1, 1]).await.unwrap();
    assert!(sa.metrics().snapshot().degraded_writes >= 1);

    let (b, d) = (node(), node());
    let (kb, kd) = (EpochCell::new(0), EpochCell::new(0));
    let _cb = b.follow(sa.local_addr(), &kb);
    wait_for("one follower in sync", || sa.in_sync_followers() == 1).await;
    assert!(
        sa.is_degraded(),
        "RF 3 needs two followers; one is not enough"
    );
    sa.insert_replicated(2, vec![2, 2]).await.unwrap();

    let _cd = d.follow(sa.local_addr(), &kd);
    wait_for("full RF", || !sa.is_degraded()).await;
    assert_eq!(sa.in_sync_followers(), 2);
    let before = sa.metrics().snapshot().degraded_writes;
    let lsn = sa.insert_replicated(3, vec![3, 3]).await.unwrap();
    assert_eq!(sa.metrics().snapshot().degraded_writes, before);
    assert!(b.engine.get(3).unwrap().is_some() && d.engine.get(3).unwrap().is_some());
    assert!(lsn > 0);

    // One follower leaves: degraded again, and it is reported.
    _cb.shutdown();
    wait_for("degraded again", || sa.is_degraded()).await;
    sa.insert_replicated(4, vec![4, 4]).await.unwrap();
    assert!(sa.metrics().snapshot().degraded);
}
