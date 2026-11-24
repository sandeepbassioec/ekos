//! F7 on the engine-backed shard node: the migration snapshot streams in
//! bounded chunks, and a coordinator crash at any phase is repaired on restart
//! (single owner, no lost or duplicated rows) even when the nodes themselves
//! restarted and lost their in-memory shard registry.

use std::path::Path;
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use vectorvolt_cluster::migration::{
    MapControl, MigrationConfig, MigrationError, MigrationJournal, MigrationPhase, Nodes, Recovery,
    ShardNode, bootstrap, migrate_shard, recover_migrations,
};
use vectorvolt_cluster::node::NodeId;
use vectorvolt_cluster::shard::{ShardCmd, ShardTable};
use vectorvolt_engine::{Durability, EngineOptions, TableEngine};
use vvctl::commands::serve_cluster_ext::EngineShardNode;

struct LocalControl(Mutex<ShardTable>);

#[async_trait]
impl MapControl for LocalControl {
    async fn commit(&self, cmd: ShardCmd) -> Result<(), MigrationError> {
        self.0
            .lock()
            .unwrap()
            .apply(&cmd)
            .map_err(MigrationError::Rejected)
    }
    fn table(&self) -> ShardTable {
        self.0.lock().unwrap().clone()
    }
}

fn engine(dir: &Path) -> Arc<TableEngine> {
    Arc::new(
        TableEngine::open_with(
            vec!["pk".into(), "val".into()],
            dir,
            EngineOptions::new(1_048_576).with_durability(Durability::WriteOnly),
        )
        .unwrap(),
    )
}

fn live_keys(e: &TableEngine) -> usize {
    e.consistent_snapshot_keys().1.len()
}

struct Env {
    _d: [tempfile::TempDir; 2],
    e: [Arc<TableEngine>; 2],
    ids: [NodeId; 2],
    control: Arc<LocalControl>,
}

impl Env {
    async fn new(rows: u64) -> (Env, [Arc<EngineShardNode>; 2]) {
        let d = [tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap()];
        let e = [engine(d[0].path()), engine(d[1].path())];
        let ids = [NodeId::new(), NodeId::new()];
        let nodes = [
            EngineShardNode::new(ids[0].clone(), e[0].clone()),
            EngineShardNode::new(ids[1].clone(), e[1].clone()),
        ];
        let control = Arc::new(LocalControl(Mutex::new(ShardTable::new())));
        let n = Nodes::new(vec![
            nodes[0].clone() as Arc<dyn ShardNode>,
            nodes[1].clone() as Arc<dyn ShardNode>,
        ]);
        bootstrap(control.as_ref(), &n, &ids[0]).await.unwrap();
        for pk in 0..rows {
            e[0].insert(pk * 3 + 1, vec![pk * 3 + 1, pk]).unwrap();
        }
        (
            Env {
                _d: d,
                e,
                ids,
                control,
            },
            nodes,
        )
    }

    /// Process restart: fresh shard nodes over the same engines, hosting only
    /// what the (Raft-backed) map says they own -- like `serve_cluster_ext`.
    async fn restart(&self) -> [Arc<EngineShardNode>; 2] {
        let nodes = [
            EngineShardNode::new(self.ids[0].clone(), self.e[0].clone()),
            EngineShardNode::new(self.ids[1].clone(), self.e[1].clone()),
        ];
        let t = self.control.table();
        for (i, n) in nodes.iter().enumerate() {
            n.enforce_ownership();
            for s in t.shards_of(&self.ids[i]) {
                n.host_shard(s.shard_id, s.lo, s.hi, true).await.unwrap();
            }
        }
        nodes
    }
}

fn nodes_of(n: &[Arc<EngineShardNode>; 2]) -> Nodes {
    Nodes::new(vec![
        n[0].clone() as Arc<dyn ShardNode>,
        n[1].clone() as Arc<dyn ShardNode>,
    ])
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn migration_snapshot_streams_in_bounded_chunks() {
    const ROWS: u64 = 5000;
    const CHUNK: usize = 100;
    let (env, nodes) = Env::new(ROWS).await;
    let rep = migrate_shard(
        env.control.as_ref(),
        &nodes_of(&nodes),
        1,
        &env.ids[1],
        &MigrationConfig {
            chunk: CHUNK,
            batch: 64,
            ..Default::default()
        },
    )
    .await
    .unwrap();
    assert_eq!(rep.copied_keys, ROWS);
    // The shard is 50x the chunk, but no more than one chunk of rows was ever
    // buffered by the source.
    let peak = nodes[0].peak_scan_rows();
    assert!(
        peak > 0 && peak <= CHUNK as u64,
        "peak buffered rows {peak}"
    );
    assert_eq!(live_keys(&env.e[1]), ROWS as usize);
    assert_eq!(live_keys(&env.e[0]), 0, "source copy released");
    for pk in [0u64, 1234, 4999] {
        let r = env.e[1].get(pk * 3 + 1).unwrap().unwrap();
        assert_eq!(r.values, vec![pk * 3 + 1, pk]);
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn crash_mid_migration_is_repaired_after_restart() {
    use MigrationPhase::*;
    const ROWS: u64 = 400;
    for phase in [
        Intent, Begun, Copying, Streaming, Frozen, Activated, Switched,
    ] {
        let (env, nodes) = Env::new(ROWS).await;
        let jdir = tempfile::tempdir().unwrap();
        let err = migrate_shard(
            env.control.as_ref(),
            &nodes_of(&nodes),
            1,
            &env.ids[1],
            &MigrationConfig {
                chunk: 50,
                journal: Some(Arc::new(MigrationJournal::open(jdir.path()).unwrap())),
                crash_after: Some(phase),
                ..Default::default()
            },
        )
        .await
        .unwrap_err();
        assert!(
            matches!(err, MigrationError::Crashed(p) if p == phase),
            "{err}"
        );

        // Everything restarts: nodes lose their registry, the journal is
        // reopened, recovery runs.
        drop(nodes);
        let nodes = env.restart().await;
        let journal = MigrationJournal::open(jdir.path()).unwrap();
        assert_eq!(journal.load_all().len(), 1, "{phase:?}");
        let done = recover_migrations(env.control.as_ref(), &nodes_of(&nodes), &journal).await;
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
        assert!(journal.load_all().is_empty());

        // One owner holds every row exactly once; the other holds none.
        let (own, other) = if switched { (1, 0) } else { (0, 1) };
        let t = env.control.table();
        assert_eq!(t.shard(1).unwrap().owner, env.ids[own], "{phase:?}");
        assert!(t.migrating.is_empty(), "{phase:?}");
        assert_eq!(live_keys(&env.e[own]), ROWS as usize, "{phase:?}");
        assert_eq!(live_keys(&env.e[other]), 0, "{phase:?}");
        for pk in 0..ROWS {
            let r = env.e[own].get(pk * 3 + 1).unwrap().unwrap();
            assert_eq!(r.values, vec![pk * 3 + 1, pk], "{phase:?} pk {pk}");
        }
    }
}
