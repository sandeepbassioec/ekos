//! Shard split / live migration / drain / rebalance tests (gap F7).
//! In-process: a real 3-node Raft group (in-memory network) commits the shard
//! map; the data plane is `MemShardNode`.

use std::collections::HashSet;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::{Duration, Instant};

use tempfile::TempDir;
use uuid::Uuid;
use vectorvolt_cluster::migration::{
    MapControl, MemShardNode, MigrationConfig, MigrationError, MigrationJournal, MigrationPhase,
    Nodes, RaftMapControl, RaftMember, Recovery, ShardClient, ShardNode, bootstrap, drain_node,
    migrate_shard, recover_migrations, split_shard,
};
use vectorvolt_cluster::node::NodeId;
use vectorvolt_cluster::raft::node::{RaftConfig, RaftNode};
use vectorvolt_cluster::raft::server;
use vectorvolt_cluster::rebalance::{MetricsCollector, RebalanceConfig, Rebalancer};
use vectorvolt_cluster::shard::{ShardCmd, ShardMapSm};
use vectorvolt_cluster::transport::MemNetwork;

const TICK: Duration = Duration::from_millis(5);

fn nid(i: u128) -> NodeId {
    NodeId(Uuid::from_u128(i + 1))
}

struct Env {
    _dirs: Vec<TempDir>,
    control: Arc<RaftMapControl>,
    nodes: Nodes,
    mem: Vec<Arc<MemShardNode>>,
    ids: Vec<NodeId>,
}

impl Env {
    /// 3 Raft members + `n_data` data nodes (data node i uses id nid(i)); the
    /// first data node owns the bootstrap shard.
    async fn new(n_data: usize) -> Env {
        let net = MemNetwork::new();
        let raft_ids: Vec<NodeId> = (0..3).map(|i| nid(100 + i)).collect();
        let mut dirs = Vec::new();
        let mut members = Vec::new();
        for (i, id) in raft_ids.iter().enumerate() {
            let dir = tempfile::tempdir().unwrap();
            let peers: Vec<NodeId> = raft_ids.iter().filter(|x| *x != id).cloned().collect();
            let sm = ShardMapSm::new();
            let (table, results) = (sm.table.clone(), sm.results.clone());
            let node = RaftNode::open(
                id.clone(),
                peers,
                RaftConfig {
                    election_ticks: 20,
                    heartbeat_ticks: 2,
                    snapshot_threshold: 0,
                    max_entries_per_append: 64,
                    seed: 77 + i as u64,
                },
                dir.path(),
                sm,
            )
            .unwrap();
            let h = server::spawn(node, Arc::new(net.transport(id.clone())), TICK);
            members.push(RaftMember {
                handle: Arc::new(h),
                table,
                results,
            });
            dirs.push(dir);
        }
        let ids: Vec<NodeId> = (0..n_data as u128).map(nid).collect();
        let mem: Vec<Arc<MemShardNode>> = ids.iter().cloned().map(MemShardNode::new).collect();
        let nodes = Nodes::new(
            mem.iter()
                .map(|m| m.clone() as Arc<dyn ShardNode>)
                .collect(),
        );
        let control = Arc::new(RaftMapControl::new(members));
        bootstrap(control.as_ref(), &nodes, &ids[0]).await.unwrap();
        Env {
            _dirs: dirs,
            control,
            nodes,
            mem,
            ids,
        }
    }

    fn client(&self) -> ShardClient {
        ShardClient::new(self.control.clone(), self.nodes.clone())
    }

    fn total_keys(&self) -> usize {
        self.mem.iter().map(|m| m.total_keys()).sum()
    }
}

fn val(k: u64, gen_: u64) -> Vec<u8> {
    format!("{k}:{gen_}").into_bytes()
}

async fn load_keys(c: &ShardClient, n: u64) {
    for k in 0..n {
        c.put(k * 10 + 5, val(k * 10 + 5, 0)).await.unwrap();
    }
}

#[tokio::test]
async fn split_preserves_keys_and_routing() {
    let env = Env::new(2).await;
    let c = env.client();
    load_keys(&c, 200).await;
    let v0 = env.control.table().version;

    let (lo, hi) = split_shard(env.control.as_ref(), &env.nodes, 1, None)
        .await
        .unwrap();
    assert_eq!(lo.hi, hi.lo);
    assert!(env.control.table().version > v0);
    // Median split: both halves non-trivial.
    let lo_keys = env.mem[0].dump().iter().filter(|(k, _)| *k < hi.lo).count();
    assert!(lo_keys > 50 && lo_keys < 150, "lo_keys={lo_keys}");

    assert_eq!(env.total_keys(), 200);
    for k in 0..200u64 {
        let key = k * 10 + 5;
        assert_eq!(c.get(key).await.unwrap(), Some(val(key, 0)), "key {key}");
    }
    // Explicit split point too, and every node's Raft table agrees.
    split_shard(env.control.as_ref(), &env.nodes, lo.shard_id, Some(500))
        .await
        .unwrap();
    for k in 0..200u64 {
        let key = k * 10 + 5;
        assert_eq!(c.get(key).await.unwrap(), Some(val(key, 0)));
    }
    let t = env.control.table();
    assert_eq!(t.shards.len(), 3);
    assert_eq!(t.shard_for(499).unwrap().hi, 500);
    // Out-of-range split is rejected by the replicated state machine.
    assert!(matches!(
        split_shard(env.control.as_ref(), &env.nodes, 1, Some(u64::MAX - 1)).await,
        Err(MigrationError::Rejected(_))
    ));
}

#[tokio::test]
async fn migration_under_concurrent_writes_loses_nothing() {
    let env = Env::new(2).await;
    let c = Arc::new(env.client());
    load_keys(&c, 2000).await;
    // Pre-split so the moved shard is the upper half only partly overlapping.
    split_shard(env.control.as_ref(), &env.nodes, 1, Some(10_000))
        .await
        .unwrap();

    let stop = Arc::new(AtomicBool::new(false));
    let acked: Arc<Vec<AtomicU64>> = Arc::new((0..4).map(|_| AtomicU64::new(0)).collect());
    // Each writer owns distinct keys (in the migrating range [10000, ..)) and
    // overwrites them with an increasing generation.
    let mut writers = Vec::new();
    for w in 0..4u64 {
        let (c, stop, acked) = (c.clone(), stop.clone(), acked.clone());
        writers.push(tokio::spawn(async move {
            let mut gen_ = 1u64;
            while !stop.load(Ordering::SeqCst) {
                for j in 0..50u64 {
                    let key = 10_000 + w * 1000 + j;
                    c.put(key, val(key, gen_)).await.unwrap();
                }
                acked[w as usize].store(gen_, Ordering::SeqCst);
                gen_ += 1;
                tokio::task::yield_now().await;
            }
            gen_ - 1
        }));
    }
    tokio::time::sleep(Duration::from_millis(30)).await;
    let rep = migrate_shard(
        env.control.as_ref(),
        &env.nodes,
        2,
        &env.ids[1],
        &MigrationConfig {
            chunk: 100,
            batch: 64,
            ..Default::default()
        },
    )
    .await
    .unwrap();
    tokio::time::sleep(Duration::from_millis(30)).await;
    stop.store(true, Ordering::SeqCst);
    let mut finals = Vec::new();
    for w in writers {
        finals.push(w.await.unwrap());
    }

    assert!(
        rep.cutover < Duration::from_millis(500),
        "{:?}",
        rep.cutover
    );
    assert!(rep.copied_keys > 0);
    let t = env.control.table();
    assert_eq!(t.shard(2).unwrap().owner, env.ids[1]);
    assert!(t.migrating.is_empty());

    // Nothing lost, nothing duplicated: source no longer holds shard 2.
    assert_eq!(env.mem[0].hosted_ids(), vec![1]);
    assert_eq!(env.mem[1].hosted_ids(), vec![2]);
    let dump_src: HashSet<u64> = env.mem[0].dump().into_iter().map(|(k, _)| k).collect();
    let dump_dst: HashSet<u64> = env.mem[1].dump().into_iter().map(|(k, _)| k).collect();
    assert!(dump_src.is_disjoint(&dump_dst));
    for w in 0..4u64 {
        for j in 0..50u64 {
            let key = 10_000 + w * 1000 + j;
            let got = c.get(key).await.unwrap().expect("writer key lost");
            assert_eq!(got, val(key, finals[w as usize]), "key {key} stale/lost");
        }
    }
    // Original data all present exactly once (writer keys end in ...5 only
    // when j*... ; check untouched seed keys).
    let mut untouched = 0;
    for k in 0..2000u64 {
        let key = k * 10 + 5;
        let writer_key = (10_000..10_000 + 4 * 1000).contains(&key) && (key - 10_000) % 1000 < 50;
        if writer_key {
            continue;
        }
        assert_eq!(c.get(key).await.unwrap(), Some(val(key, 0)), "key {key}");
        untouched += 1;
    }
    assert!(untouched > 1500);
    let distinct: HashSet<u64> = dump_src.union(&dump_dst).copied().collect();
    assert_eq!(distinct.len(), dump_src.len() + dump_dst.len());
    assert_eq!(env.total_keys(), distinct.len());
}

#[tokio::test]
async fn cutover_pause_is_bounded() {
    let env = Env::new(2).await;
    let c = Arc::new(env.client());
    load_keys(&c, 3000).await;
    let stop = Arc::new(AtomicBool::new(false));
    let max_pause = Arc::new(AtomicU64::new(0));
    let w = {
        let (c, stop, max_pause) = (c.clone(), stop.clone(), max_pause.clone());
        tokio::spawn(async move {
            let mut i = 0u64;
            while !stop.load(Ordering::SeqCst) {
                let t = Instant::now();
                c.put(5 + (i % 3000) * 10, val(i, 1)).await.unwrap();
                max_pause.fetch_max(t.elapsed().as_millis() as u64, Ordering::SeqCst);
                i += 1;
                tokio::task::yield_now().await;
            }
        })
    };
    tokio::time::sleep(Duration::from_millis(20)).await;
    let rep = migrate_shard(
        env.control.as_ref(),
        &env.nodes,
        1,
        &env.ids[1],
        &MigrationConfig::default(),
    )
    .await
    .unwrap();
    tokio::time::sleep(Duration::from_millis(20)).await;
    stop.store(true, Ordering::SeqCst);
    w.await.unwrap();
    assert!(
        rep.cutover < Duration::from_millis(500),
        "{:?}",
        rep.cutover
    );
    assert!(
        max_pause.load(Ordering::SeqCst) < 500,
        "writer pause {}ms",
        max_pause.load(Ordering::SeqCst)
    );
    assert_eq!(env.mem[0].total_keys(), 0);
    assert_eq!(env.mem[1].total_keys(), 3000);
}

#[tokio::test]
async fn drain_empties_a_node() {
    let env = Env::new(3).await;
    let c = env.client();
    load_keys(&c, 300).await;
    split_shard(env.control.as_ref(), &env.nodes, 1, None)
        .await
        .unwrap();
    split_shard(env.control.as_ref(), &env.nodes, 1, None)
        .await
        .unwrap();
    assert_eq!(env.control.table().shards_of(&env.ids[0]).len(), 3);

    let rep = drain_node(
        env.control.as_ref(),
        &env.nodes,
        &env.ids[0],
        &MigrationConfig::default(),
    )
    .await
    .unwrap();
    assert_eq!(rep.moved.len(), 3);
    let t = env.control.table();
    assert!(t.shards_of(&env.ids[0]).is_empty());
    assert!(t.drained.contains(&env.ids[0]));
    assert!(!t.draining.contains(&env.ids[0]));
    assert_eq!(env.mem[0].total_keys(), 0);
    assert!(env.mem[0].hosted_ids().is_empty());
    assert_eq!(env.total_keys(), 300);
    // Shards spread over the survivors, not piled on one.
    assert!(!t.shards_of(&env.ids[1]).is_empty());
    assert!(!t.shards_of(&env.ids[2]).is_empty());
    for k in 0..300u64 {
        assert_eq!(c.get(k * 10 + 5).await.unwrap(), Some(val(k * 10 + 5, 0)));
    }
    // Nothing may be moved onto a drained node.
    assert!(
        migrate_shard(
            env.control.as_ref(),
            &env.nodes,
            t.shards_of(&env.ids[1])[0].shard_id,
            &env.ids[0],
            &MigrationConfig::default()
        )
        .await
        .is_err()
    );
}

#[tokio::test]
async fn aborted_migration_rolls_back_to_source() {
    let env = Env::new(2).await;
    let c = env.client();
    load_keys(&c, 400).await;
    // Target dies after its second apply (mid-copy: 400 keys / chunk 100).
    env.mem[1].fail_after_applies(2);
    let r = migrate_shard(
        env.control.as_ref(),
        &env.nodes,
        1,
        &env.ids[1],
        &MigrationConfig {
            chunk: 100,
            ..Default::default()
        },
    )
    .await;
    assert!(matches!(r, Err(MigrationError::Aborted(_))), "{r:?}");
    let t = env.control.table();
    assert_eq!(t.shard(1).unwrap().owner, env.ids[0]);
    assert!(t.migrating.is_empty(), "AbortMove must clear the marker");
    assert!(!env.mem[0].is_frozen(1));
    assert!(!env.mem[0].is_capturing(1));
    // Source still serves reads and writes, no data lost.
    for k in 0..400u64 {
        assert_eq!(c.get(k * 10 + 5).await.unwrap(), Some(val(k * 10 + 5, 0)));
    }
    c.put(7, b"after-abort".to_vec()).await.unwrap();
    assert_eq!(c.get(7).await.unwrap(), Some(b"after-abort".to_vec()));
    assert_eq!(env.mem[0].total_keys(), 401);
    // And a retry to a healthy target succeeds afterwards.
    env.mem[1].set_down(false);
    env.mem[1].fail_after_applies(-1);
    migrate_shard(
        env.control.as_ref(),
        &env.nodes,
        1,
        &env.ids[1],
        &MigrationConfig::default(),
    )
    .await
    .unwrap();
    assert_eq!(env.mem[1].total_keys(), 401);
}

#[tokio::test]
async fn rebalancer_converges_and_respects_rate_limit() {
    let env = Env::new(3).await;
    let c = env.client();
    load_keys(&c, 900).await; // everything on node 0: heavily skewed
    let mcfg = MigrationConfig::default();

    // Off by default.
    let off = Rebalancer::new(RebalanceConfig::default());
    assert!(!RebalanceConfig::default().enabled);
    let collector = MetricsCollector::default();
    let m = collector.collect(&env.control.table(), &env.nodes).await;
    let rep = off
        .run_round(env.control.clone(), env.nodes.clone(), m, mcfg.clone())
        .await;
    assert!(rep.disabled && rep.splits == 0 && rep.moves.is_empty());
    assert_eq!(env.mem[0].total_keys(), 900);

    let rb = Rebalancer::new(RebalanceConfig {
        enabled: true,
        imbalance_threshold: 0.2,
        max_actions_per_round: 2,
        ..Default::default()
    });
    let mut rounds = 0;
    let mut total_moves = 0;
    loop {
        let t = env.control.table();
        let m = collector.collect(&t, &env.nodes).await;
        let rep = rb
            .run_round(env.control.clone(), env.nodes.clone(), m, mcfg.clone())
            .await;
        assert!(rep.failures.is_empty(), "{:?}", rep.failures);
        assert!(rep.moves.len() <= 2);
        total_moves += rep.moves.len();
        rounds += 1;
        if rep.moves.is_empty() && rep.splits == 0 {
            break;
        }
        assert!(rounds < 40, "did not converge");
    }
    assert!(total_moves >= 2);
    let loads: Vec<usize> = env.mem.iter().map(|m| m.total_keys()).collect();
    let avg = 900.0 / 3.0;
    for l in &loads {
        assert!(
            (*l as f64) <= avg * 1.2 + 1.0,
            "node load {l} over threshold: {loads:?}"
        );
    }
    assert_eq!(loads.iter().sum::<usize>(), 900);
    // One migration at a time per node, even with two parallel actions.
    assert_eq!(rb.peak_migrations_per_node(), 1);
    for k in 0..900u64 {
        assert_eq!(c.get(k * 10 + 5).await.unwrap(), Some(val(k * 10 + 5, 0)));
    }
}

#[tokio::test]
async fn rebalancer_skips_busy_nodes() {
    use vectorvolt_cluster::rebalance::{Action, ShardMetric, plan};
    let env = Env::new(3).await;
    let c = env.client();
    load_keys(&c, 100).await;
    split_shard(env.control.as_ref(), &env.nodes, 1, None)
        .await
        .unwrap();
    let t = env.control.table();
    let cfg = RebalanceConfig {
        enabled: true,
        ..Default::default()
    };
    let metrics: Vec<ShardMetric> = t
        .shards
        .values()
        .map(|s| ShardMetric {
            shard_id: s.shard_id,
            owner: s.owner.clone(),
            keys: 50,
            ops_per_sec: 0.0,
        })
        .collect();
    let free = plan(&cfg, &t, &metrics, &env.nodes.ids(), &HashSet::new());
    assert!(matches!(free[0], Action::Move { .. }));
    let busy: HashSet<_> = [env.ids[0].clone()].into();
    assert!(plan(&cfg, &t, &metrics, &env.nodes.ids(), &busy).is_empty());
    // Raft rejected-command path is visible to callers.
    let r = env
        .control
        .commit(ShardCmd::MarkDrained {
            node: env.ids[0].clone(),
        })
        .await;
    assert!(matches!(r, Err(MigrationError::Rejected(_))));
}

/// Crash the coordinator right after each migration phase, "restart" it
/// (fresh journal handle, recovery pass) and check the invariants: exactly one
/// owner, every row present exactly once, no leftover state.
#[tokio::test]
async fn coordinator_crash_at_every_phase_is_repaired() {
    use MigrationPhase::*;
    for phase in [
        Intent, Begun, Copying, Streaming, Frozen, Activated, Switched,
    ] {
        let env = Env::new(2).await;
        let c = env.client();
        load_keys(&c, 300).await;
        let jdir = tempfile::tempdir().unwrap();
        let journal = Arc::new(MigrationJournal::open(jdir.path()).unwrap());
        let err = migrate_shard(
            env.control.as_ref(),
            &env.nodes,
            1,
            &env.ids[1],
            &MigrationConfig {
                chunk: 50,
                journal: Some(journal.clone()),
                crash_after: Some(phase),
                ..Default::default()
            },
        )
        .await
        .unwrap_err();
        assert!(
            matches!(err, MigrationError::Crashed(p) if p == phase),
            "{phase:?}: {err}"
        );
        // The crash left a durable record behind.
        assert_eq!(journal.load_all().len(), 1, "{phase:?}");

        // Restart: a new coordinator opens the journal and repairs.
        drop(journal);
        let journal = MigrationJournal::open(jdir.path()).unwrap();
        let rec = journal.load_all();
        assert_eq!(rec[0].phase, phase);
        let done = recover_migrations(env.control.as_ref(), &env.nodes, &journal).await;
        let switched = phase == Switched;
        assert_eq!(
            done,
            vec![if switched {
                Recovery::Completed(1)
            } else {
                Recovery::RolledBack(1)
            }],
            "{phase:?}"
        );
        assert!(journal.load_all().is_empty(), "{phase:?}");

        let t = env.control.table();
        assert!(t.migrating.is_empty(), "{phase:?}");
        let owner = if switched { 1 } else { 0 };
        assert_eq!(t.shard(1).unwrap().owner, env.ids[owner], "{phase:?}");
        // Single host, nothing duplicated or lost.
        assert_eq!(env.mem[owner].hosted_ids(), vec![1], "{phase:?}");
        assert!(env.mem[1 - owner].hosted_ids().is_empty(), "{phase:?}");
        assert_eq!(env.total_keys(), 300, "{phase:?}");
        assert!(!env.mem[owner].is_frozen(1), "{phase:?}");
        for k in 0..300u64 {
            let key = k * 10 + 5;
            assert_eq!(
                c.get(key).await.unwrap(),
                Some(val(key, 0)),
                "{phase:?} {key}"
            );
        }
        // Writes work again, and a retry of the same migration succeeds.
        c.put(7, val(7, 1)).await.unwrap();
        if !switched {
            migrate_shard(
                env.control.as_ref(),
                &env.nodes,
                1,
                &env.ids[1],
                &MigrationConfig::default(),
            )
            .await
            .unwrap();
            assert_eq!(env.total_keys(), 301, "{phase:?}");
        }
    }
}
