//! Raft cluster tests (gap F1/F2): in-memory transport clusters plus one real
//! TCP cluster on loopback. Timing is driven by a seeded per-node RNG and a
//! short logical tick so the whole file runs in a few seconds.

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use tempfile::TempDir;
use uuid::Uuid;
use vectorvolt_cluster::node::NodeId;
use vectorvolt_cluster::raft::node::{RaftConfig, RaftNode, StateMachine, Status};
use vectorvolt_cluster::raft::server::{self, RaftHandle};
use vectorvolt_cluster::raft::state::RaftRole;
use vectorvolt_cluster::transport::{MemNetwork, RaftTransport, TcpConfig, TcpTransport};

const TICK: Duration = Duration::from_millis(5);

type Applied = Arc<Mutex<Vec<Vec<u8>>>>;

struct Log(Applied);

impl StateMachine for Log {
    fn apply(&mut self, _index: u64, command: &[u8]) {
        self.0.lock().unwrap().push(command.to_vec());
    }
    fn snapshot(&self) -> Vec<u8> {
        serde_json::to_vec(&*self.0.lock().unwrap()).unwrap()
    }
    fn restore(&mut self, data: &[u8]) {
        *self.0.lock().unwrap() = serde_json::from_slice(data).unwrap();
    }
}

fn nid(i: usize) -> NodeId {
    NodeId(Uuid::from_u128(i as u128 + 1))
}

fn cfg(i: usize, snapshot_threshold: u64) -> RaftConfig {
    RaftConfig {
        election_ticks: 20,
        heartbeat_ticks: 2,
        snapshot_threshold,
        max_entries_per_append: 64,
        seed: 1000 + i as u64,
    }
}

struct Cluster {
    net: MemNetwork,
    ids: Vec<NodeId>,
    dirs: Vec<TempDir>,
    handles: Vec<Option<RaftHandle>>,
    applied: Vec<Applied>,
    threshold: u64,
}

impl Cluster {
    async fn new(n: usize, threshold: u64) -> Self {
        let mut c = Cluster {
            net: MemNetwork::new(),
            ids: (0..n).map(nid).collect(),
            dirs: (0..n).map(|_| tempfile::tempdir().unwrap()).collect(),
            handles: (0..n).map(|_| None).collect(),
            applied: (0..n).map(|_| Applied::default()).collect(),
            threshold,
        };
        for i in 0..n {
            c.start(i);
        }
        c
    }

    fn peers_of(&self, i: usize) -> Vec<NodeId> {
        self.ids
            .iter()
            .filter(|x| **x != self.ids[i])
            .cloned()
            .collect()
    }

    fn start(&mut self, i: usize) {
        self.applied[i] = Applied::default();
        let tr = Arc::new(self.net.transport(self.ids[i].clone()));
        let node = RaftNode::open(
            self.ids[i].clone(),
            self.peers_of(i),
            cfg(i, self.threshold),
            self.dirs[i].path(),
            Log(self.applied[i].clone()),
        )
        .unwrap();
        self.handles[i] = Some(server::spawn(node, tr, TICK));
    }

    async fn crash(&mut self, i: usize) {
        self.net.set_down(&self.ids[i], true);
        if let Some(h) = self.handles[i].take() {
            h.crash();
            h.join().await;
        }
    }

    async fn status(&self, i: usize) -> Option<Status> {
        self.handles[i].as_ref()?.status().await
    }

    async fn statuses(&self) -> Vec<(usize, Status)> {
        let mut v = Vec::new();
        for i in 0..self.ids.len() {
            if let Some(s) = self.status(i).await {
                v.push((i, s));
            }
        }
        v
    }

    /// Wait until exactly one live node outside `skip` is leader and all live
    /// nodes outside `skip` agree on it.
    async fn wait_leader(&self, skip: &[usize]) -> usize {
        let start = Instant::now();
        loop {
            let st: Vec<_> = self
                .statuses()
                .await
                .into_iter()
                .filter(|(i, _)| !skip.contains(i))
                .collect();
            let leaders: Vec<_> = st
                .iter()
                .filter(|(_, s)| s.role == RaftRole::Leader)
                .collect();
            if leaders.len() == 1 {
                let (li, ls) = leaders[0];
                if st
                    .iter()
                    .all(|(_, s)| s.term == ls.term && s.leader_id.as_ref() == Some(&ls.id))
                {
                    return *li;
                }
            }
            assert!(
                start.elapsed() < Duration::from_secs(8),
                "no stable leader: {st:?}"
            );
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    }

    async fn propose(&self, leader: usize, cmd: &str) -> u64 {
        self.handles[leader]
            .as_ref()
            .unwrap()
            .propose(cmd.as_bytes().to_vec())
            .await
            .unwrap()
    }

    async fn wait_applied(&self, nodes: &[usize], count: usize) {
        let start = Instant::now();
        loop {
            if nodes
                .iter()
                .all(|&i| self.applied[i].lock().unwrap().len() >= count)
            {
                return;
            }
            assert!(
                start.elapsed() < Duration::from_secs(8),
                "timeout: applied lens {:?}",
                nodes
                    .iter()
                    .map(|&i| self.applied[i].lock().unwrap().len())
                    .collect::<Vec<_>>()
            );
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    }

    fn applied_strs(&self, i: usize) -> Vec<String> {
        self.applied[i]
            .lock()
            .unwrap()
            .iter()
            .map(|c| String::from_utf8(c.clone()).unwrap())
            .collect()
    }
}

async fn elect_exactly_one(n: usize) {
    let c = Cluster::new(n, 0).await;
    let l = c.wait_leader(&[]).await;
    // Stays stable; no second leader appears in the same term.
    tokio::time::sleep(Duration::from_millis(300)).await;
    let st = c.statuses().await;
    let leaders: Vec<_> = st
        .iter()
        .filter(|(_, s)| s.role == RaftRole::Leader)
        .collect();
    assert_eq!(leaders.len(), 1);
    assert_eq!(leaders[0].0, l);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn three_node_elects_exactly_one_leader() {
    elect_exactly_one(3).await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn five_node_elects_exactly_one_leader() {
    elect_exactly_one(5).await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn replicates_and_commits_in_order() {
    let c = Cluster::new(3, 0).await;
    let l = c.wait_leader(&[]).await;
    for i in 0..10 {
        c.propose(l, &format!("cmd{i}")).await;
    }
    c.wait_applied(&[0, 1, 2], 10).await;
    let want: Vec<String> = (0..10).map(|i| format!("cmd{i}")).collect();
    for i in 0..3 {
        assert_eq!(c.applied_strs(i), want);
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn follower_rejects_proposals() {
    let c = Cluster::new(3, 0).await;
    let l = c.wait_leader(&[]).await;
    let f = (l + 1) % 3;
    let r = c.handles[f].as_ref().unwrap().propose(b"x".to_vec()).await;
    assert!(matches!(
        r,
        Err(vectorvolt_cluster::raft::node::RaftError::NotLeader(_))
    ));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn leader_crash_elects_new_leader_without_losing_committed_entries() {
    let mut c = Cluster::new(5, 0).await;
    let l = c.wait_leader(&[]).await;
    for i in 0..5 {
        c.propose(l, &format!("a{i}")).await;
    }
    c.wait_applied(&[0, 1, 2, 3, 4], 5).await;
    c.crash(l).await;
    let l2 = c.wait_leader(&[l]).await;
    assert_ne!(l, l2);
    for i in 0..3 {
        c.propose(l2, &format!("b{i}")).await;
    }
    let live: Vec<usize> = (0..5).filter(|i| *i != l).collect();
    c.wait_applied(&live, 8).await;
    let want: Vec<String> = (0..5)
        .map(|i| format!("a{i}"))
        .chain((0..3).map(|i| format!("b{i}")))
        .collect();
    for i in live {
        assert_eq!(c.applied_strs(i), want);
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn partitioned_minority_cannot_commit_and_converges_after_heal() {
    let c = Cluster::new(5, 0).await;
    let l = c.wait_leader(&[]).await;
    c.propose(l, "base").await;
    c.wait_applied(&[0, 1, 2, 3, 4], 1).await;

    let other = (l + 1) % 5;
    let majority: Vec<usize> = (0..5).filter(|i| *i != l && *i != other).collect();
    let minority_ids = [c.ids[l].clone(), c.ids[other].clone()];
    let majority_ids: Vec<NodeId> = majority.iter().map(|&i| c.ids[i].clone()).collect();
    c.net.partition(&[&minority_ids, &majority_ids]);

    // Old leader still believes it leads; its entry can never reach a majority.
    let stale_idx = c.propose(l, "stale").await;
    tokio::time::sleep(Duration::from_millis(500)).await;
    let st = c.status(l).await.unwrap();
    assert!(st.commit_index < stale_idx, "minority committed: {st:?}");
    assert_eq!(c.applied_strs(l), vec!["base"]);

    // Majority side elects its own leader and commits.
    let l2 = {
        let start = Instant::now();
        loop {
            let mut found = None;
            for &i in &majority {
                let s = c.status(i).await.unwrap();
                if s.role == RaftRole::Leader {
                    found = Some(i);
                }
            }
            if let Some(i) = found {
                break i;
            }
            assert!(start.elapsed() < Duration::from_secs(8));
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    };
    c.propose(l2, "fresh").await;
    c.wait_applied(&majority, 2).await;

    c.net.heal();
    c.wait_applied(&[0, 1, 2, 3, 4], 2).await;
    c.wait_leader(&[]).await;
    for i in 0..5 {
        assert_eq!(c.applied_strs(i), vec!["base", "fresh"], "node {i}");
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn restart_restores_term_vote_and_log_from_disk() {
    let mut c = Cluster::new(3, 0).await;
    let l = c.wait_leader(&[]).await;
    for i in 0..5 {
        c.propose(l, &format!("p{i}")).await;
    }
    c.wait_applied(&[0, 1, 2], 5).await;
    let f = (l + 1) % 3;
    let before = c.status(f).await.unwrap();
    c.crash(f).await;

    // Inspect the on-disk state directly.
    let peers = c.peers_of(f);
    let node = RaftNode::open(
        c.ids[f].clone(),
        peers,
        cfg(f, 0),
        c.dirs[f].path(),
        Log(Applied::default()),
    )
    .unwrap();
    assert!(node.term() >= before.term);
    assert!(node.voted_for().is_some(), "vote must be persisted");
    assert!(node.last_index() >= 5);
    drop(node);

    // Restart the whole cluster; committed data survives and it keeps working.
    for i in 0..3 {
        if i != f {
            c.crash(i).await;
        }
    }
    for i in 0..3 {
        c.start(i);
    }
    let l2 = c.wait_leader(&[]).await;
    c.propose(l2, "after").await;
    c.wait_applied(&[0, 1, 2], 6).await;
    let want: Vec<String> = (0..5)
        .map(|i| format!("p{i}"))
        .chain(["after".to_string()])
        .collect();
    for i in 0..3 {
        assert_eq!(c.applied_strs(i), want, "node {i}");
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn lagging_follower_catches_up_via_install_snapshot() {
    let c = Cluster::new(3, 5).await;
    let l = c.wait_leader(&[]).await;
    let lag = (l + 1) % 3;
    c.net.set_down(&c.ids[lag], true);
    for i in 0..20 {
        c.propose(l, &format!("s{i}")).await;
    }
    let up: Vec<usize> = (0..3).filter(|i| *i != lag).collect();
    c.wait_applied(&up, 20).await;
    let ls = c.status(l).await.unwrap();
    assert!(
        ls.snapshot_index > 0,
        "leader should have compacted: {ls:?}"
    );

    c.net.set_down(&c.ids[lag], false);
    c.wait_applied(&[lag], 20).await;
    let ss = c.status(lag).await.unwrap();
    assert!(
        ss.snapshot_index > 0,
        "lagging node should have installed a snapshot"
    );
    let want: Vec<String> = (0..20).map(|i| format!("s{i}")).collect();
    assert_eq!(c.applied_strs(lag), want);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn three_node_cluster_over_real_tcp() {
    let n = 3;
    let ids: Vec<NodeId> = (0..n).map(nid).collect();
    let lo: SocketAddr = "127.0.0.1:0".parse().unwrap();
    let mut transports = Vec::new();
    for id in &ids {
        transports.push(Arc::new(
            TcpTransport::bind(id.clone(), lo, HashMap::new(), TcpConfig::default())
                .await
                .unwrap(),
        ));
    }
    for t in &transports {
        for (j, id) in ids.iter().enumerate() {
            if id != t.local_id() {
                t.set_peer(id.clone(), transports[j].local_addr());
            }
        }
    }
    let dirs: Vec<TempDir> = (0..n).map(|_| tempfile::tempdir().unwrap()).collect();
    let applied: Vec<Applied> = (0..n).map(|_| Applied::default()).collect();
    let mut handles = Vec::new();
    for i in 0..n {
        let peers = ids.iter().filter(|x| **x != ids[i]).cloned().collect();
        let node = RaftNode::open(
            ids[i].clone(),
            peers,
            cfg(i, 0),
            dirs[i].path(),
            Log(applied[i].clone()),
        )
        .unwrap();
        handles.push(server::spawn(node, transports[i].clone(), TICK));
    }

    let start = Instant::now();
    let leader = loop {
        let mut found = None;
        for (i, h) in handles.iter().enumerate() {
            if h.status().await.unwrap().role == RaftRole::Leader {
                found = Some(i);
            }
        }
        if let Some(i) = found {
            break i;
        }
        assert!(
            start.elapsed() < Duration::from_secs(8),
            "no leader over TCP"
        );
        tokio::time::sleep(Duration::from_millis(10)).await;
    };
    for i in 0..5 {
        handles[leader]
            .propose(format!("t{i}").into_bytes())
            .await
            .unwrap();
    }
    let start = Instant::now();
    while !applied.iter().all(|a| a.lock().unwrap().len() >= 5) {
        assert!(
            start.elapsed() < Duration::from_secs(8),
            "tcp replication timeout"
        );
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    for a in &applied {
        assert_eq!(a.lock().unwrap().len(), 5);
    }
}
