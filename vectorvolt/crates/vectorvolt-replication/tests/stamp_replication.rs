//! F8 — the primary's commit versionstamp travels with replicated frames and
//! lands on the follower's CDC events; the observer hook sees every stamp.

use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;

use tempfile::TempDir;
use vectorvolt_engine::{Durability, EngineOptions, TableEngine};
use vectorvolt_replication::{
    set_stamp_observer, ReplicationClient, ReplicationConfig, ReplicationMode, ReplicationRole,
    ReplicationServer, RoleCell,
};

const T: Duration = Duration::from_secs(8);

fn engine(dir: &Path) -> Arc<TableEngine> {
    let o = EngineOptions::new(1_048_576).with_durability(Durability::WriteOnly);
    Arc::new(TableEngine::open_with(vec!["pk".into(), "val".into()], dir, o).unwrap())
}

fn stamp_of(n: u64) -> [u8; 12] {
    let mut s = [0u8; 12];
    s[4..].copy_from_slice(&n.to_be_bytes());
    s
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn follower_cdc_carries_primary_stamps_and_clock_observes_them() {
    // Follower "clock": highest stamp observed (big-endian counter in bytes 4..).
    let clock = Arc::new(AtomicU64::new(0));
    let c2 = Arc::clone(&clock);
    assert!(set_stamp_observer(Arc::new(move |b: &[u8]| {
        let n = u64::from_be_bytes(b[4..12].try_into().unwrap());
        c2.fetch_max(n, Ordering::SeqCst);
    })));

    let (ld, fd) = (TempDir::new().unwrap(), TempDir::new().unwrap());
    let (le, fe) = (engine(ld.path()), engine(fd.path()));
    // Primary stamps are 1001, 1002, ...; the follower's own stamper must not
    // be used for replicated commits.
    let n = Arc::new(AtomicU64::new(1000));
    let n2 = Arc::clone(&n);
    le.set_commit_stamper(Some(Arc::new(move |_| {
        Some(stamp_of(n2.fetch_add(1, Ordering::SeqCst) + 1))
    })));
    fe.set_commit_stamper(Some(Arc::new(|_| Some(stamp_of(1)))));

    let cfg = ReplicationConfig {
        mode: ReplicationMode::Async,
        replication_factor: 2,
        poll_interval: Duration::from_millis(5),
        heartbeat_interval: Duration::from_millis(100),
        backoff_initial: Duration::from_millis(20),
        backoff_max: Duration::from_millis(150),
        ..ReplicationConfig::default()
    };
    let server = ReplicationServer::start(
        Arc::clone(&le),
        ld.path().join("wal"),
        cfg.clone(),
        RoleCell::new(ReplicationRole::Leader),
        "127.0.0.1:0".parse().unwrap(),
    )
    .await
    .unwrap();
    let client = ReplicationClient::start(
        Arc::clone(&fe),
        fd.path(),
        None,
        cfg,
        RoleCell::new(ReplicationRole::Follower),
        server.local_addr().to_string(),
    )
    .unwrap();

    // Connect first so the writes below stream as frames (a snapshot carries no stamps).
    let t0 = std::time::Instant::now();
    while server.in_sync_followers() < 1 {
        assert!(t0.elapsed() < T, "follower never connected");
        tokio::time::sleep(Duration::from_millis(5)).await;
    }
    let mut last = 0;
    for pk in 1..=5u64 {
        last = le.insert(pk, vec![pk, pk]).unwrap().sequence;
        server.notify_write();
    }
    last = le.delete(2).unwrap().sequence.max(last);
    server.notify_write();
    assert!(client.wait_for_lsn(last, T).await, "follower lagging");

    let lp: Vec<_> = le.read_log_from(0).unwrap();
    let fl: Vec<_> = fe.read_log_from(0).unwrap();
    assert_eq!(lp.len(), 6);
    assert_eq!(fl.len(), 6, "follower CDC events");
    for (a, b) in lp.iter().zip(&fl) {
        assert!(a.versionstamp.is_some());
        assert_eq!(a.versionstamp, b.versionstamp, "pk {}", a.pk);
    }
    assert!(clock.load(Ordering::SeqCst) >= 1006);
}
