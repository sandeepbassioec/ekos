/// Layer 20 — replication integration tests.
///
/// Tests verify the single-leader / multi-follower replication model:
/// leader writes propagate to followers after catchup, lag and catch-up work,
/// deletions replicate, multiple followers all receive frames, and
/// disconnected followers are pruned automatically.
use std::sync::Arc;

use tempfile::TempDir;
use vectorvolt_engine::{Durability, EngineOptions, TableEngine};
use vectorvolt_replication::Leader;

fn engine(dir: &TempDir) -> Arc<TableEngine> {
    Arc::new(
        TableEngine::open_with(
            vec!["pk".into(), "val".into()],
            dir.path(),
            EngineOptions::new(1_048_576).with_durability(Durability::WriteOnly),
        )
        .unwrap(),
    )
}

// ── basic replication ─────────────────────────────────────────────────────────

#[test]
fn follower_sees_insert_after_catchup() {
    let ld = TempDir::new().unwrap();
    let rd = TempDir::new().unwrap();
    let leader = Leader::new(engine(&ld));
    let follower = leader.connect_replica(engine(&rd));

    leader.insert(1, vec![1, 42]).unwrap();
    follower.catchup().unwrap();

    let row = follower.get(1).unwrap().unwrap();
    assert_eq!(row.values[1], 42);
}

#[test]
fn follower_does_not_see_row_before_catchup() {
    let ld = TempDir::new().unwrap();
    let rd = TempDir::new().unwrap();
    let leader = Leader::new(engine(&ld));
    let follower = leader.connect_replica(engine(&rd));

    leader.insert(5, vec![5, 99]).unwrap();
    // deliberately skip catchup
    assert!(follower.get(5).unwrap().is_none(), "follower must lag");
}

#[test]
fn delete_propagates_to_follower() {
    let ld = TempDir::new().unwrap();
    let rd = TempDir::new().unwrap();
    let leader = Leader::new(engine(&ld));
    let follower = leader.connect_replica(engine(&rd));

    leader.insert(7, vec![7, 10]).unwrap();
    follower.catchup().unwrap();
    assert!(follower.get(7).unwrap().is_some());

    leader.delete(7).unwrap();
    follower.catchup().unwrap();
    assert!(follower.get(7).unwrap().is_none(), "delete must propagate");
}

// ── multiple followers ────────────────────────────────────────────────────────

#[test]
fn multiple_followers_all_receive_writes() {
    let ld = TempDir::new().unwrap();
    let rd1 = TempDir::new().unwrap();
    let rd2 = TempDir::new().unwrap();
    let rd3 = TempDir::new().unwrap();
    let leader = Leader::new(engine(&ld));
    let f1 = leader.connect_replica(engine(&rd1));
    let f2 = leader.connect_replica(engine(&rd2));
    let f3 = leader.connect_replica(engine(&rd3));

    for pk in 1u64..=5 {
        leader.insert(pk, vec![pk, pk * 10]).unwrap();
    }

    for f in [&f1, &f2, &f3] {
        f.catchup().unwrap();
        for pk in 1u64..=5 {
            let row = f.get(pk).unwrap().unwrap();
            assert_eq!(row.values[1], pk * 10, "follower must see pk={pk}");
        }
    }
}

#[test]
fn replica_count_tracks_connected_followers() {
    let ld = TempDir::new().unwrap();
    let rd1 = TempDir::new().unwrap();
    let rd2 = TempDir::new().unwrap();
    let leader = Leader::new(engine(&ld));
    assert_eq!(leader.replica_count(), 0);
    let _f1 = leader.connect_replica(engine(&rd1));
    assert_eq!(leader.replica_count(), 1);
    let _f2 = leader.connect_replica(engine(&rd2));
    assert_eq!(leader.replica_count(), 2);
}

// ── frame ordering and counting ───────────────────────────────────────────────

#[test]
fn frames_applied_count_matches_mutations() {
    let ld = TempDir::new().unwrap();
    let rd = TempDir::new().unwrap();
    let leader = Leader::new(engine(&ld));
    let follower = leader.connect_replica(engine(&rd));

    for pk in 1u64..=10 {
        leader.insert(pk, vec![pk, 0]).unwrap();
    }
    leader.delete(1).unwrap();
    leader.delete(2).unwrap();

    let n = follower.catchup().unwrap();
    assert_eq!(n, 12, "10 inserts + 2 deletes = 12 frames");
    assert_eq!(follower.frames_applied(), 12);
}

#[test]
fn sequence_numbers_are_monotonically_increasing() {
    let ld = TempDir::new().unwrap();
    let rd = TempDir::new().unwrap();
    let leader = Leader::new(engine(&ld));
    let follower = leader.connect_replica(engine(&rd));

    for pk in 1u64..=5 {
        leader.insert(pk, vec![pk, 0]).unwrap();
    }
    follower.catchup().unwrap();
    // last_seq should be 4 (0-indexed frames 0..=4)
    assert_eq!(follower.last_seq(), Some(4));
}

#[test]
fn follower_handles_empty_channel_gracefully() {
    let ld = TempDir::new().unwrap();
    let rd = TempDir::new().unwrap();
    let leader = Leader::new(engine(&ld));
    let follower = leader.connect_replica(engine(&rd));

    let n = follower.apply_pending().unwrap();
    assert_eq!(n, 0);
    assert_eq!(follower.last_seq(), None);
}

// ── overwrite + lag ───────────────────────────────────────────────────────────

#[test]
fn overwrite_followed_by_catchup_shows_latest_value() {
    let ld = TempDir::new().unwrap();
    let rd = TempDir::new().unwrap();
    let leader = Leader::new(engine(&ld));
    let follower = leader.connect_replica(engine(&rd));

    leader.insert(1, vec![1, 100]).unwrap();
    leader.insert(1, vec![1, 200]).unwrap(); // overwrite same pk
    follower.catchup().unwrap();

    let row = follower.get(1).unwrap().unwrap();
    assert_eq!(row.values[1], 200, "follower must reflect overwrite");
}

#[test]
fn follower_catching_up_after_many_leader_writes() {
    let ld = TempDir::new().unwrap();
    let rd = TempDir::new().unwrap();
    let leader = Leader::new(engine(&ld));
    let follower = leader.connect_replica(engine(&rd));

    // leader writes 100 rows while follower does nothing
    for pk in 1u64..=100 {
        leader.insert(pk, vec![pk, pk * 2]).unwrap();
    }

    // catchup: all 100 frames applied
    let n = follower.catchup().unwrap();
    assert_eq!(n, 100);

    // spot-check a few
    assert_eq!(follower.get(50).unwrap().unwrap().values[1], 100);
    assert_eq!(follower.get(100).unwrap().unwrap().values[1], 200);
}

// ── leader reads ──────────────────────────────────────────────────────────────

#[test]
fn leader_can_read_its_own_writes() {
    let ld = TempDir::new().unwrap();
    let leader = Leader::new(engine(&ld));
    leader.insert(9, vec![9, 77]).unwrap();
    let row = leader.get(9).unwrap().unwrap();
    assert_eq!(row.values[1], 77);
}

// ── disconnected follower pruning ─────────────────────────────────────────────

#[test]
fn disconnected_follower_is_pruned_from_leader() {
    let ld = TempDir::new().unwrap();
    let rd = TempDir::new().unwrap();
    let leader = Leader::new(engine(&ld));
    {
        let _follower = leader.connect_replica(engine(&rd));
        assert_eq!(leader.replica_count(), 1);
        // _follower drops here, closing its receiver
    }
    // Next write triggers the leader to detect the closed channel and prune it.
    leader.insert(1, vec![1, 1]).unwrap();
    assert_eq!(
        leader.replica_count(),
        0,
        "disconnected follower must be pruned"
    );
}
