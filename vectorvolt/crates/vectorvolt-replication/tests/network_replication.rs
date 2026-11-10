//! F5 — networked WAL-shipping replication over 127.0.0.1.

use std::net::SocketAddr;
use std::path::Path;
use std::sync::Arc;
use std::time::{Duration, Instant};

use tempfile::TempDir;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpListener;
use vectorvolt_crypto::KeyStore;
use vectorvolt_engine::{Durability, EngineOptions, TableEngine};
use vectorvolt_replication::{
    load_position, Message, ReplicationClient, ReplicationConfig, ReplicationError,
    ReplicationMode, ReplicationRole, ReplicationServer, RoleCell, WireFrame,
};
use vectorvolt_txn::Mutation;
use vectorvolt_wal::{WalConfig, WalWriter};

const T: Duration = Duration::from_secs(8);

fn opts() -> EngineOptions {
    EngineOptions::new(1_048_576).with_durability(Durability::WriteOnly)
}

fn schema() -> Vec<String> {
    vec!["pk".into(), "val".into()]
}

fn engine(dir: &Path) -> Arc<TableEngine> {
    Arc::new(TableEngine::open_with(schema(), dir, opts()).unwrap())
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
    }
}

async fn leader(
    engine: Arc<TableEngine>,
    dir: &Path,
    cfg: ReplicationConfig,
) -> Arc<ReplicationServer> {
    ReplicationServer::start(
        engine,
        dir.join("wal"),
        cfg,
        RoleCell::new(ReplicationRole::Leader),
        "127.0.0.1:0".parse().unwrap(),
    )
    .await
    .unwrap()
}

fn follower(
    engine: Arc<TableEngine>,
    dir: &Path,
    key: Option<Arc<KeyStore>>,
    addr: SocketAddr,
    cfg: ReplicationConfig,
) -> Arc<ReplicationClient> {
    ReplicationClient::start(
        engine,
        dir,
        key,
        cfg,
        RoleCell::new(ReplicationRole::Follower),
        addr.to_string(),
    )
    .unwrap()
}

fn assert_same(l: &TableEngine, f: &TableEngine, pks: std::ops::RangeInclusive<u64>) {
    for pk in pks {
        let a = l.get(pk).unwrap().map(|r| r.values);
        let b = f.get(pk).unwrap().map(|r| r.values);
        assert_eq!(a, b, "pk {pk} diverged");
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn follower_converges_to_leader() {
    let (ld, fd) = (TempDir::new().unwrap(), TempDir::new().unwrap());
    let (le, fe) = (engine(ld.path()), engine(fd.path()));
    let server = leader(Arc::clone(&le), ld.path(), cfg(ReplicationMode::Async, 2)).await;
    // Writes made before the follower exists are caught up, later ones streamed.
    for pk in 1..=60u64 {
        le.insert(pk, vec![pk, pk * 10]).unwrap();
    }
    let mut last;
    let client = follower(
        Arc::clone(&fe),
        fd.path(),
        None,
        server.local_addr(),
        cfg(ReplicationMode::Async, 2),
    );
    for pk in 61..=100u64 {
        le.insert(pk, vec![pk, pk * 10]).unwrap();
    }
    last = le.insert(5, vec![5, 555]).unwrap().sequence; // update
    for pk in 10..=19u64 {
        last = le.delete(pk).unwrap().sequence;
    }
    server.notify_write();
    assert!(
        client.wait_for_lsn(last, T).await,
        "follower did not catch up"
    );
    assert_same(&le, &fe, 1..=100);
    assert!(fe.get(12).unwrap().is_none());
    assert_eq!(fe.get(5).unwrap().unwrap().values[1], 555);

    // Lag metrics and leader-side follower view.
    let m = client.metrics().snapshot();
    assert_eq!(m.applied_lsn, last);
    assert_eq!(m.lag_lsn, 0);
    assert_eq!(m.lag_ms, 0);
    assert!(m.connected);
    assert_eq!(m.snapshots_applied, 0);
    assert_eq!(load_position(fd.path()), last);
    let deadline = Instant::now() + T;
    while server.followers().first().map(|f| f.durable_lsn) != Some(last) {
        assert!(
            Instant::now() < deadline,
            "leader never saw the durable ack"
        );
        tokio::time::sleep(Duration::from_millis(5)).await;
    }
    assert_eq!(server.followers()[0].lag_lsn, 0);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn follower_restart_resumes_from_its_lsn() {
    let (ld, fd) = (TempDir::new().unwrap(), TempDir::new().unwrap());
    let le = engine(ld.path());
    let server = leader(Arc::clone(&le), ld.path(), cfg(ReplicationMode::Async, 2)).await;
    let mut last = 0;
    for pk in 1..=30u64 {
        last = le.insert(pk, vec![pk, pk]).unwrap().sequence;
    }
    let first_lsn;
    {
        let fe = engine(fd.path());
        let client = follower(
            fe,
            fd.path(),
            None,
            server.local_addr(),
            cfg(ReplicationMode::Async, 2),
        );
        assert!(client.wait_for_lsn(last, T).await);
        first_lsn = client.applied_lsn();
        client.shutdown();
    }
    let deadline = Instant::now() + T;
    while server.follower_count() != 0 {
        assert!(Instant::now() < deadline, "leader kept a dead follower");
        tokio::time::sleep(Duration::from_millis(5)).await;
    }
    let sent_before = server.metrics().snapshot().frames_sent;
    assert_eq!(sent_before, 30);

    // Leader moves on while the follower is down.
    for pk in 31..=50u64 {
        last = le.insert(pk, vec![pk, pk]).unwrap().sequence;
    }
    // Reopen the follower on the same directory: it must resume, not restart.
    let fe = engine(fd.path());
    assert_eq!(load_position(fd.path()), first_lsn);
    let client = follower(
        Arc::clone(&fe),
        fd.path(),
        None,
        server.local_addr(),
        cfg(ReplicationMode::Async, 2),
    );
    assert!(client.wait_for_lsn(last, T).await);
    assert_same(&le, &fe, 1..=50);
    let m = client.metrics().snapshot();
    assert_eq!(m.snapshots_applied, 0, "resume must not need a snapshot");
    assert_eq!(m.frames_applied, 20, "only the missing frames are shipped");
    assert_eq!(server.metrics().snapshot().frames_sent - sent_before, 20);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn follower_behind_the_retained_wal_gets_a_snapshot() {
    let (ld, fd) = (TempDir::new().unwrap(), TempDir::new().unwrap());
    // Seed the leader WAL across many tiny segments, as a long-lived leader would have.
    let wal_dir = ld.path().join("wal");
    std::fs::create_dir_all(&wal_dir).unwrap();
    {
        let w = WalWriter::open(
            WalConfig::default()
                .with_dir(&wal_dir)
                .with_fsync(false)
                .with_segment_max_bytes(600),
        )
        .unwrap();
        for pk in 1..=200u64 {
            w.append(
                Mutation::Insert {
                    pk,
                    values: vec![pk, pk * 3],
                }
                .encode(),
            )
            .unwrap();
        }
    }
    let le = engine(ld.path()); // replays all 200 rows
    assert_eq!(le.get(200).unwrap().unwrap().values[1], 600);
    // The old history is gone (retention): drop every segment but the newest two.
    let mut ids = vectorvolt_wal::list_segment_ids(&wal_dir).unwrap();
    assert!(ids.len() > 4, "seed should span several segments");
    ids.truncate(ids.len() - 2);
    for id in ids {
        std::fs::remove_file(vectorvolt_wal::segment_path(&wal_dir, id)).unwrap();
    }

    let server = leader(Arc::clone(&le), ld.path(), cfg(ReplicationMode::Async, 2)).await;
    // The follower holds stale state that the snapshot must replace.
    let fe = engine(fd.path());
    fe.insert(999, vec![999, 1]).unwrap();
    let client = follower(
        Arc::clone(&fe),
        fd.path(),
        None,
        server.local_addr(),
        cfg(ReplicationMode::Async, 2),
    );
    let tip = le.insert(201, vec![201, 1]).unwrap().sequence;
    server.notify_write();
    assert!(
        client.wait_for_lsn(tip, T).await,
        "{:?}",
        client.last_error()
    );
    assert_same(&le, &fe, 1..=201);
    assert!(
        fe.get(999).unwrap().is_none(),
        "stale follower row survived the snapshot"
    );
    assert_eq!(client.metrics().snapshot().snapshots_applied, 1);
    assert_eq!(server.metrics().snapshot().snapshots_sent, 1);
    // ...and streaming continues afterwards.
    let tip = le.delete(7).unwrap().sequence;
    server.notify_write();
    assert!(client.wait_for_lsn(tip, T).await);
    assert!(fe.get(7).unwrap().is_none());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn sync_mode_waits_for_ack_and_times_out_without_a_follower() {
    let (ld, fd) = (TempDir::new().unwrap(), TempDir::new().unwrap());
    let le = engine(ld.path());
    let server = leader(Arc::clone(&le), ld.path(), cfg(ReplicationMode::Sync, 2)).await;

    // Follower down: the write is committed locally but must not report success.
    let started = Instant::now();
    let err = server.insert_replicated(1, vec![1, 1]).await.unwrap_err();
    assert!(
        matches!(
            err,
            ReplicationError::AckTimeout {
                needed: 1,
                got: 0,
                ..
            }
        ),
        "{err:?}"
    );
    assert!(
        started.elapsed() >= Duration::from_millis(500),
        "returned before the timeout"
    );
    assert_eq!(server.metrics().snapshot().ack_timeouts, 1);

    // Follower up: the write returns only once it is durably applied there.
    let fe = engine(fd.path());
    let client = follower(
        Arc::clone(&fe),
        fd.path(),
        None,
        server.local_addr(),
        cfg(ReplicationMode::Sync, 2),
    );
    let lsn = server.insert_replicated(2, vec![2, 22]).await.unwrap();
    assert!(client.applied_lsn() >= lsn);
    assert_eq!(fe.get(2).unwrap().unwrap().values[1], 22);
    assert_eq!(fe.get(1).unwrap().unwrap().values[1], 1); // earlier write caught up too

    // Replication factor 3 needs two followers; one is not enough.
    let (ld3, fd3) = (TempDir::new().unwrap(), TempDir::new().unwrap());
    let le3 = engine(ld3.path());
    let server3 = leader(Arc::clone(&le3), ld3.path(), cfg(ReplicationMode::Sync, 3)).await;
    let _c = follower(
        engine(fd3.path()),
        fd3.path(),
        None,
        server3.local_addr(),
        cfg(ReplicationMode::Sync, 3),
    );
    let err = server3.insert_replicated(1, vec![1, 1]).await.unwrap_err();
    assert!(
        matches!(
            err,
            ReplicationError::AckTimeout {
                needed: 2,
                got: 1,
                ..
            }
        ),
        "{err:?}"
    );

    // Semi-sync needs just one follower even at RF 3.
    let (lds, fds) = (TempDir::new().unwrap(), TempDir::new().unwrap());
    let les = engine(lds.path());
    let servers = leader(
        Arc::clone(&les),
        lds.path(),
        cfg(ReplicationMode::SemiSync, 3),
    )
    .await;
    let _c = follower(
        engine(fds.path()),
        fds.path(),
        None,
        servers.local_addr(),
        cfg(ReplicationMode::SemiSync, 3),
    );
    servers.insert_replicated(1, vec![1, 1]).await.unwrap();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn async_mode_never_blocks_on_followers() {
    let ld = TempDir::new().unwrap();
    let le = engine(ld.path());
    let mut c = cfg(ReplicationMode::Async, 3);
    c.ack_timeout = Duration::from_secs(30);
    let server = leader(Arc::clone(&le), ld.path(), c).await;
    let started = Instant::now();
    for pk in 1..=50u64 {
        server.insert_replicated(pk, vec![pk, pk]).await.unwrap();
    }
    assert!(
        started.elapsed() < Duration::from_secs(3),
        "async writes waited for a follower"
    );
    assert_eq!(server.metrics().snapshot().ack_timeouts, 0);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn corrupted_frame_is_rejected_and_not_applied() {
    let fd = TempDir::new().unwrap();
    let fe = engine(fd.path());
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let good = Message::Frames {
        tip: 1,
        frames: vec![WireFrame {
            seq: 1,
            timestamp_ns: 0,
            key_id: None,
            payload: Mutation::Insert {
                pk: 7,
                values: vec![7, 70],
            }
            .encode(),
        }],
    };
    let mut bad = good.encode();
    let mid = bad.len() / 2;
    bad[mid] ^= 0x01;
    let (release_tx, release_rx) = tokio::sync::oneshot::channel::<()>();
    let fake = tokio::spawn(async move {
        let welcome = Message::Welcome {
            first_lsn: 1,
            tip: 1,
            snapshot: false,
            start_lsn: 0,
        };
        // Connection 1: valid welcome, then a bit-flipped frame message.
        let (mut s, _) = listener.accept().await.unwrap();
        let _ = vectorvolt_replication_read(&mut s).await;
        s.write_all(&welcome.encode()).await.unwrap();
        s.write_all(&bad).await.unwrap();
        let _ = release_rx.await;
        drop(s);
        // Connection 2 (after the follower reconnects): the intact frame.
        let (mut s, _) = listener.accept().await.unwrap();
        let _ = vectorvolt_replication_read(&mut s).await;
        s.write_all(&welcome.encode()).await.unwrap();
        s.write_all(&good.encode()).await.unwrap();
        tokio::time::sleep(Duration::from_secs(2)).await;
    });
    let client = follower(
        Arc::clone(&fe),
        fd.path(),
        None,
        addr,
        cfg(ReplicationMode::Async, 2),
    );
    let deadline = Instant::now() + T;
    while client.metrics().snapshot().rejected_frames == 0 {
        assert!(Instant::now() < deadline, "corruption was not detected");
        tokio::time::sleep(Duration::from_millis(5)).await;
    }
    assert!(fe.get(7).unwrap().is_none(), "corrupted frame was applied");
    assert_eq!(client.applied_lsn(), 0);
    assert!(client.last_error().unwrap().contains("rejected"));
    release_tx.send(()).unwrap();
    assert!(
        client.wait_for_lsn(1, T).await,
        "follower did not recover on reconnect"
    );
    assert_eq!(fe.get(7).unwrap().unwrap().values[1], 70);
    fake.abort();
}

/// Read and discard the follower's Hello (raw bytes: 12 header/crc + 10 body).
async fn vectorvolt_replication_read(s: &mut tokio::net::TcpStream) -> std::io::Result<()> {
    use tokio::io::AsyncReadExt;
    let mut buf = [0u8; 8 + 10 + 4];
    s.read_exact(&mut buf).await.map(|_| ())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn follower_role_is_read_only_until_promoted() {
    let (ld, fd) = (TempDir::new().unwrap(), TempDir::new().unwrap());
    let le = engine(ld.path());
    let server = leader(Arc::clone(&le), ld.path(), cfg(ReplicationMode::Async, 2)).await;
    let fe = engine(fd.path());
    let client = follower(
        Arc::clone(&fe),
        fd.path(),
        None,
        server.local_addr(),
        cfg(ReplicationMode::Async, 2),
    );
    assert!(matches!(
        client.role().ensure_writable(),
        Err(ReplicationError::ReadOnlyFollower)
    ));
    // A server whose role is Follower refuses replicated writes.
    let dd = TempDir::new().unwrap();
    let demoted = ReplicationServer::start(
        engine(dd.path()),
        dd.path().join("wal"),
        cfg(ReplicationMode::Sync, 2),
        RoleCell::new(ReplicationRole::Follower),
        "127.0.0.1:0".parse().unwrap(),
    )
    .await
    .unwrap();
    assert!(matches!(
        demoted.insert_replicated(1, vec![1, 1]).await,
        Err(ReplicationError::NotLeader)
    ));
    // Promotion flips the role cell; the client stops pulling.
    let lsn = le.insert(1, vec![1, 1]).unwrap().sequence;
    server.notify_write();
    assert!(client.wait_for_lsn(lsn, T).await);
    client.role().set(ReplicationRole::Leader);
    assert!(client.role().ensure_writable().is_ok());
    let lsn2 = le.insert(2, vec![2, 2]).unwrap().sequence;
    server.notify_write();
    assert!(!client.wait_for_lsn(lsn2, Duration::from_millis(300)).await);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn encrypted_wal_replicates_with_the_table_key() {
    let (ld, fd) = (TempDir::new().unwrap(), TempDir::new().unwrap());
    let key = Arc::new(KeyStore::generate().unwrap());
    let le = Arc::new(
        TableEngine::open_with_encryption(schema(), ld.path(), opts(), Arc::clone(&key)).unwrap(),
    );
    let server = leader(Arc::clone(&le), ld.path(), cfg(ReplicationMode::Async, 2)).await;
    let mut last = 0;
    for pk in 1..=25u64 {
        last = le.insert(pk, vec![pk, pk + 1000]).unwrap().sequence;
    }

    // Without the key the follower cannot apply (and says why) rather than skipping rows.
    let nokey_dir = TempDir::new().unwrap();
    let nokey_engine = engine(nokey_dir.path());
    let nokey = follower(
        Arc::clone(&nokey_engine),
        nokey_dir.path(),
        None,
        server.local_addr(),
        cfg(ReplicationMode::Async, 2),
    );
    let deadline = Instant::now() + T;
    while nokey.last_error().is_none() {
        assert!(Instant::now() < deadline);
        tokio::time::sleep(Duration::from_millis(5)).await;
    }
    assert!(
        nokey.last_error().unwrap().contains("no key"),
        "{:?}",
        nokey.last_error()
    );
    assert!(nokey_engine.get(1).unwrap().is_none());
    nokey.shutdown();

    let fe = Arc::new(
        TableEngine::open_with_encryption(schema(), fd.path(), opts(), Arc::clone(&key)).unwrap(),
    );
    let client = follower(
        Arc::clone(&fe),
        fd.path(),
        Some(Arc::clone(&key)),
        server.local_addr(),
        cfg(ReplicationMode::Async, 2),
    );
    assert!(
        client.wait_for_lsn(last, T).await,
        "{:?}",
        client.last_error()
    );
    assert_same(&le, &fe, 1..=25);
}
