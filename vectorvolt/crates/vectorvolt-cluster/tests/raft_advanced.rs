//! Raft production-hardening tests (gap F part 2): mTLS transport, chunked and
//! resumable snapshots, pre-vote, and single-server membership changes.
//!
//! `Sim` drives `RaftNode`s synchronously with a deterministic message queue
//! (no wall-clock), the async tests use real tasks over `MemNetwork`/TLS TCP.

#![allow(clippy::needless_range_loop)]

use std::collections::{HashMap, HashSet, VecDeque};
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use tempfile::TempDir;
use uuid::Uuid;
use vectorvolt_cluster::node::NodeId;
use vectorvolt_cluster::raft::node::{RaftConfig, RaftError, RaftNode, StateMachine, Status};
use vectorvolt_cluster::raft::rpc::Rpc;
use vectorvolt_cluster::raft::server::{self, RaftHandle};
use vectorvolt_cluster::raft::state::RaftRole;
use vectorvolt_cluster::transport::{
    Envelope, MemNetwork, RaftTransport, TcpConfig, TcpTransport, TlsConfig,
};

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
        seed: 7000 + i as u64,
    }
}

// ---- deterministic simulator ---------------------------------------------

struct Sim {
    ids: Vec<NodeId>,
    nodes: Vec<RaftNode<Log>>,
    applied: Vec<Applied>,
    /// Messages to or from these nodes are dropped.
    isolated: HashSet<usize>,
    q: VecDeque<(usize, usize, Rpc)>,
    /// Snapshot chunks sent to node index 2: (offset, len, done).
    chunks_to_2: Vec<(u64, usize, bool)>,
    /// Drop the n-th (0-based) snapshot chunk sent to node 2, once.
    drop_chunk: Option<usize>,
    _dirs: Vec<TempDir>,
}

impl Sim {
    fn new(n: usize, threshold: u64, chunk: usize) -> Self {
        let ids: Vec<NodeId> = (0..n).map(nid).collect();
        let dirs: Vec<TempDir> = (0..n).map(|_| tempfile::tempdir().unwrap()).collect();
        let applied: Vec<Applied> = (0..n).map(|_| Applied::default()).collect();
        let nodes = (0..n)
            .map(|i| {
                let peers = ids.iter().filter(|x| **x != ids[i]).cloned().collect();
                RaftNode::open(
                    ids[i].clone(),
                    peers,
                    RaftConfig {
                        election_ticks: 10,
                        heartbeat_ticks: 2,
                        snapshot_threshold: threshold,
                        max_entries_per_append: 64,
                        seed: 31 + i as u64,
                    },
                    dirs[i].path(),
                    Log(applied[i].clone()),
                )
                .unwrap()
                .with_snapshot_chunk_size(chunk)
            })
            .collect();
        Sim {
            ids,
            nodes,
            applied,
            isolated: HashSet::new(),
            q: VecDeque::new(),
            chunks_to_2: Vec::new(),
            drop_chunk: None,
            _dirs: dirs,
        }
    }

    fn route(&mut self, from: usize, out: Vec<(NodeId, Rpc)>) {
        for (to, rpc) in out {
            let Some(to) = self.ids.iter().position(|x| *x == to) else {
                continue;
            };
            self.q.push_back((from, to, rpc));
        }
    }

    fn run(&mut self) {
        let mut guard = 0;
        while let Some((from, to, rpc)) = self.q.pop_front() {
            guard += 1;
            assert!(guard < 100_000, "message storm");
            if self.isolated.contains(&from) || self.isolated.contains(&to) {
                continue;
            }
            if to == 2
                && let Rpc::InstallSnapshot {
                    offset, data, done, ..
                } = &rpc
            {
                let n = self.chunks_to_2.len();
                self.chunks_to_2.push((*offset, data.len(), *done));
                if self.drop_chunk == Some(n) {
                    self.drop_chunk = None;
                    continue;
                }
            }
            let src = self.ids[from].clone();
            let out = self.nodes[to].step(src, rpc).unwrap();
            self.route(to, out);
        }
    }

    fn tick_all(&mut self) {
        for i in 0..self.nodes.len() {
            let out = self.nodes[i].tick().unwrap();
            self.route(i, out);
        }
        self.run();
    }

    fn ticks(&mut self, n: usize) {
        for _ in 0..n {
            self.tick_all();
        }
    }

    fn leader(&self) -> Option<usize> {
        let ls: Vec<usize> = (0..self.nodes.len())
            .filter(|&i| self.nodes[i].role() == RaftRole::Leader && !self.isolated.contains(&i))
            .collect();
        (ls.len() == 1).then(|| ls[0])
    }

    fn elect(&mut self) -> usize {
        for _ in 0..500 {
            self.tick_all();
            if let Some(l) = self.leader() {
                self.ticks(5);
                return l;
            }
        }
        panic!("no leader");
    }

    fn propose(&mut self, l: usize, cmd: Vec<u8>) {
        let (_, out) = self.nodes[l].propose(cmd).unwrap();
        self.route(l, out);
        self.run();
    }
}

#[test]
fn prevote_partitioned_node_does_not_disrupt_stable_leader() {
    let mut s = Sim::new(3, 0, 1 << 20);
    let l = s.elect();
    s.propose(l, b"x".to_vec());
    let term = s.nodes[l].term();
    let lagger = (l + 1) % 3;

    // Partition one follower for many election timeouts: with pre-vote it
    // keeps probing but never inflates its term.
    s.isolated.insert(lagger);
    s.ticks(300);
    assert_eq!(s.nodes[lagger].term(), term, "pre-vote must not bump terms");
    assert_ne!(s.nodes[lagger].role(), RaftRole::Leader);

    // The leader makes progress without it, then it rejoins.
    s.propose(l, b"y".to_vec());
    s.isolated.clear();
    s.ticks(100);
    assert_eq!(s.leader(), Some(l), "stable leader must be undisturbed");
    assert_eq!(s.nodes[l].term(), term);
    assert_eq!(s.nodes[lagger].role(), RaftRole::Follower);
    assert_eq!(s.nodes[lagger].leader_id(), Some(&s.ids[l]));
    assert_eq!(s.applied[lagger].lock().unwrap().len(), 2);
}

#[test]
fn prevote_does_not_block_election_after_leader_loss() {
    let mut s = Sim::new(3, 0, 1 << 20);
    let l = s.elect();
    let term = s.nodes[l].term();
    s.isolated.insert(l);
    for _ in 0..500 {
        s.tick_all();
        if s.leader().is_some() {
            break;
        }
    }
    let l2 = s.leader().expect("new leader elected through pre-vote");
    assert_ne!(l2, l);
    assert!(s.nodes[l2].term() > term);
}

#[test]
fn chunked_snapshot_larger_than_one_chunk_resumes_after_loss() {
    const CHUNK: usize = 4096;
    let mut s = Sim::new(3, 5, CHUNK);
    s.isolated.insert(2);
    let l = s.elect();
    assert_ne!(l, 2);
    for i in 0..30u8 {
        let mut cmd = vec![i; 1000];
        cmd[0] = i;
        s.propose(l, cmd);
    }
    let ls = s.nodes[l].status();
    assert!(ls.snapshot_index > 0, "leader must have compacted");

    // Node 2 comes back; the 2nd chunk it is sent is lost in transit.
    s.drop_chunk = Some(1);
    s.isolated.clear();
    for _ in 0..600 {
        s.tick_all();
        if s.nodes[2].last_applied() >= s.nodes[l].commit_index() {
            break;
        }
    }
    assert_eq!(s.nodes[2].last_applied(), s.nodes[l].commit_index());
    assert!(s.nodes[2].snapshot_index() > 0, "installed via snapshot");
    assert_eq!(*s.applied[2].lock().unwrap(), *s.applied[l].lock().unwrap());

    let c = &s.chunks_to_2;
    assert!(c.len() >= 4, "snapshot must span several chunks: {c:?}");
    assert!(c.iter().all(|(_, len, _)| *len <= CHUNK), "bounded: {c:?}");
    assert!(c.iter().filter(|(_, _, d)| *d).count() >= 1);
    let offs: Vec<u64> = c.iter().map(|x| x.0).collect();
    assert!(
        offs.iter().filter(|o| **o == CHUNK as u64).count() >= 2,
        "lost chunk must be re-sent from the acknowledged offset, not from 0: {offs:?}"
    );
    assert_eq!(
        offs.iter().filter(|o| **o == 0).count(),
        1,
        "transfer resumed instead of restarting: {offs:?}"
    );
}

#[test]
fn membership_changes_are_one_at_a_time_and_prefix_is_reserved() {
    let mut s = Sim::new(3, 0, 1 << 20);
    let l = s.elect();
    let (_, out) = s.nodes[l].add_learner(nid(10)).unwrap();
    s.route(l, out);
    // Second change before the first commits is refused.
    let second = s.nodes[l].add_learner(nid(11));
    assert!(
        matches!(second, Err(RaftError::ConfigChange(_))),
        "{second:?}"
    );
    let reserved = s.nodes[l].propose(b"\xFFVVCFG\x01{}".to_vec());
    assert!(matches!(reserved, Err(RaftError::ReservedCommand)));
    let last = s.nodes[l].remove_server(s.ids[l].clone());
    assert!(matches!(last, Err(RaftError::ConfigChange(_))));
    // Not a member / not a learner.
    assert!(s.nodes[l].remove_server(nid(99)).is_err());
    assert!(s.nodes[l].promote(nid(99)).is_err());
}

// ---- async cluster harness (dynamic membership) ----------------------------

struct Dyn {
    net: MemNetwork,
    dirs: HashMap<usize, TempDir>,
    handles: HashMap<usize, RaftHandle>,
    applied: HashMap<usize, Applied>,
}

impl Dyn {
    fn new() -> Self {
        Dyn {
            net: MemNetwork::new(),
            dirs: HashMap::new(),
            handles: HashMap::new(),
            applied: HashMap::new(),
        }
    }

    fn start(&mut self, i: usize, peers: &[usize], joining: bool) {
        let dir = tempfile::tempdir().unwrap();
        let applied = Applied::default();
        let tr = Arc::new(self.net.transport(nid(i)));
        let peer_ids: Vec<NodeId> = peers.iter().copied().filter(|p| *p != i).map(nid).collect();
        let node = if joining {
            RaftNode::open_joining(
                nid(i),
                peer_ids,
                cfg(i, 0),
                dir.path(),
                Log(applied.clone()),
            )
        } else {
            RaftNode::open(
                nid(i),
                peer_ids,
                cfg(i, 0),
                dir.path(),
                Log(applied.clone()),
            )
        }
        .unwrap();
        self.handles.insert(i, server::spawn(node, tr, TICK));
        self.dirs.insert(i, dir);
        self.applied.insert(i, applied);
    }

    async fn status(&self, i: usize) -> Option<Status> {
        self.handles.get(&i)?.status().await
    }

    /// Wait until exactly one of `members` is leader and all of them agree on
    /// leader and term.
    async fn wait_leader(&self, members: &[usize]) -> usize {
        let start = Instant::now();
        loop {
            let mut st = Vec::new();
            for &i in members {
                if let Some(s) = self.status(i).await {
                    st.push((i, s));
                }
            }
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
                start.elapsed() < Duration::from_secs(10),
                "no leader: {st:?}"
            );
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    }

    async fn propose(&self, l: usize, cmd: &str) {
        self.handles[&l]
            .propose(cmd.as_bytes().to_vec())
            .await
            .unwrap();
    }

    async fn wait_applied(&self, nodes: &[usize], count: usize) {
        let start = Instant::now();
        loop {
            if nodes
                .iter()
                .all(|i| self.applied[i].lock().unwrap().len() >= count)
            {
                return;
            }
            assert!(
                start.elapsed() < Duration::from_secs(10),
                "timeout: {:?}",
                nodes
                    .iter()
                    .map(|i| self.applied[i].lock().unwrap().len())
                    .collect::<Vec<_>>()
            );
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    }

    /// Wait until every node in `nodes` sees exactly `voters` as voters and no learners.
    async fn wait_voters(&self, nodes: &[usize], voters: &[usize]) {
        let want: HashSet<NodeId> = voters.iter().map(|i| nid(*i)).collect();
        let start = Instant::now();
        loop {
            let mut ok = true;
            for &i in nodes {
                match self.status(i).await {
                    Some(s) => {
                        let got: HashSet<NodeId> = s.voters.iter().cloned().collect();
                        ok &= got == want && s.learners.is_empty();
                    }
                    None => ok = false,
                }
            }
            if ok {
                return;
            }
            assert!(
                start.elapsed() < Duration::from_secs(10),
                "voters never converged to {voters:?}"
            );
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    }

    /// Retry a leader-side membership call while a previous change is in flight.
    async fn change<F, Fut>(&self, mut call: F)
    where
        F: FnMut() -> Fut,
        Fut: std::future::Future<Output = Result<u64, RaftError>>,
    {
        let start = Instant::now();
        loop {
            match call().await {
                Ok(_) => return,
                Err(RaftError::ConfigChange(m)) => {
                    assert!(start.elapsed() < Duration::from_secs(10), "stuck: {m}");
                    tokio::time::sleep(Duration::from_millis(10)).await;
                }
                Err(e) => panic!("membership change failed: {e}"),
            }
        }
    }
}

fn strs(a: &Applied) -> Vec<String> {
    a.lock()
        .unwrap()
        .iter()
        .map(|c| String::from_utf8(c.clone()).unwrap())
        .collect()
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn grow_three_to_five_then_shrink_removing_the_leader() {
    let mut d = Dyn::new();
    for i in 0..3 {
        d.start(i, &[0, 1, 2], false);
    }
    let l = d.wait_leader(&[0, 1, 2]).await;
    for i in 0..5 {
        d.propose(l, &format!("a{i}")).await;
    }

    // Grow 3 -> 5, one server at a time, while entries keep committing.
    d.start(3, &[0, 1, 2], true);
    d.change(|| d.handles[&l].add_server(nid(3))).await;
    for i in 0..5 {
        d.propose(l, &format!("b{i}")).await;
    }
    d.wait_voters(&[0, 1, 2, 3], &[0, 1, 2, 3]).await;

    d.start(4, &[0, 1, 2, 3], true);
    d.change(|| d.handles[&l].add_server(nid(4))).await;
    for i in 0..5 {
        d.propose(l, &format!("c{i}")).await;
    }
    d.wait_voters(&[0, 1, 2, 3, 4], &[0, 1, 2, 3, 4]).await;
    d.wait_applied(&[0, 1, 2, 3, 4], 15).await;
    let want: Vec<String> = ["a", "b", "c"]
        .iter()
        .flat_map(|p| (0..5).map(move |i| format!("{p}{i}")))
        .collect();
    for i in 0..5 {
        assert_eq!(strs(&d.applied[&i]), want, "node {i}");
    }

    // Shrink: remove the leader itself (5 -> 4). It must step down.
    let l = d.wait_leader(&[0, 1, 2, 3, 4]).await;
    d.change(|| d.handles[&l].remove_server(nid(l))).await;
    let start = Instant::now();
    loop {
        let s = d.status(l).await.unwrap();
        if s.role != RaftRole::Leader {
            assert!(
                !s.voters.contains(&nid(l)),
                "removed leader still lists itself"
            );
            break;
        }
        assert!(
            start.elapsed() < Duration::from_secs(10),
            "leader never stepped down"
        );
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    let rest: Vec<usize> = (0..5).filter(|i| *i != l).collect();
    let l2 = d.wait_leader(&rest).await;
    assert_ne!(l2, l);
    for i in 0..3 {
        d.propose(l2, &format!("d{i}")).await;
    }

    // Shrink again (4 -> 3): remove a follower.
    let victim = *rest.iter().find(|i| **i != l2).unwrap();
    d.change(|| d.handles[&l2].remove_server(nid(victim))).await;
    let rest3: Vec<usize> = rest.iter().copied().filter(|i| *i != victim).collect();
    d.wait_voters(&rest3, &rest3).await;
    for i in 0..3 {
        d.propose(l2, &format!("e{i}")).await;
    }
    d.wait_applied(&rest3, 21).await;
    let want: Vec<String> = want
        .into_iter()
        .chain((0..3).map(|i| format!("d{i}")))
        .chain((0..3).map(|i| format!("e{i}")))
        .collect();
    for &i in &rest3 {
        assert_eq!(strs(&d.applied[&i]), want, "node {i}");
    }

    // The removed servers must not disturb the remaining cluster.
    let t0 = d.status(l2).await.unwrap().term;
    tokio::time::sleep(Duration::from_millis(600)).await;
    let l3 = d.wait_leader(&rest3).await;
    assert_eq!(l3, l2);
    assert_eq!(d.status(l3).await.unwrap().term, t0);
}

// ---- TLS -------------------------------------------------------------------

struct Pki {
    ca_pem: Vec<u8>,
    nodes: Vec<(Vec<u8>, Vec<u8>)>,
}

fn make_ca() -> (rcgen::Certificate, rcgen::KeyPair) {
    let key = rcgen::KeyPair::generate().unwrap();
    let mut p = rcgen::CertificateParams::new(Vec::<String>::new()).unwrap();
    p.is_ca = rcgen::IsCa::Ca(rcgen::BasicConstraints::Unconstrained);
    p.distinguished_name
        .push(rcgen::DnType::CommonName, "vectorvolt test cluster CA");
    let cert = p.self_signed(&key).unwrap();
    (cert, key)
}

fn make_node_cert(ca: &rcgen::Certificate, ca_key: &rcgen::KeyPair) -> (Vec<u8>, Vec<u8>) {
    let key = rcgen::KeyPair::generate().unwrap();
    let mut p = rcgen::CertificateParams::new(Vec::<String>::new()).unwrap();
    p.subject_alt_names = vec![rcgen::SanType::IpAddress("127.0.0.1".parse().unwrap())];
    let cert = p.signed_by(&key, ca, ca_key).unwrap();
    (cert.pem().into_bytes(), key.serialize_pem().into_bytes())
}

fn make_pki(n: usize) -> Pki {
    let (ca, ca_key) = make_ca();
    Pki {
        ca_pem: ca.pem().into_bytes(),
        nodes: (0..n).map(|_| make_node_cert(&ca, &ca_key)).collect(),
    }
}

fn tls_for(pki: &Pki, i: usize) -> TlsConfig {
    TlsConfig::from_pem(&pki.ca_pem, &pki.nodes[i].0, &pki.nodes[i].1, None).unwrap()
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn three_node_cluster_over_mutual_tls() {
    let n = 3;
    let pki = make_pki(n);
    let ids: Vec<NodeId> = (0..n).map(nid).collect();
    let lo: SocketAddr = "127.0.0.1:0".parse().unwrap();
    let mut transports = Vec::new();
    for i in 0..n {
        transports.push(Arc::new(
            TcpTransport::bind_tls(
                ids[i].clone(),
                lo,
                HashMap::new(),
                TcpConfig::default(),
                tls_for(&pki, i),
            )
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
            start.elapsed() < Duration::from_secs(10),
            "no leader over TLS"
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
            start.elapsed() < Duration::from_secs(10),
            "tls replication timeout"
        );
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn tls_transport_rejects_untrusted_and_plaintext_peers() {
    let pki = make_pki(1);
    let rogue = make_pki(1); // different CA
    let lo: SocketAddr = "127.0.0.1:0".parse().unwrap();
    let (a, b, c, d) = (nid(0), nid(1), nid(2), nid(3));
    let server = TcpTransport::bind_tls(
        a.clone(),
        lo,
        HashMap::new(),
        TcpConfig::default(),
        tls_for(&pki, 0),
    )
    .await
    .unwrap();
    let probe = Rpc::RequestVote {
        term: 1,
        last_log_index: 0,
        last_log_term: 0,
    };

    // Sender with a certificate from another CA.
    let bad = TcpTransport::bind_tls(
        b.clone(),
        lo,
        HashMap::from([(a.clone(), server.local_addr())]),
        TcpConfig::default(),
        tls_for(&rogue, 0),
    )
    .await
    .unwrap();
    bad.send(&a, probe.clone()).await.unwrap();
    // Plaintext sender.
    let plain = TcpTransport::bind(
        c.clone(),
        lo,
        HashMap::from([(a.clone(), server.local_addr())]),
        TcpConfig::default(),
    )
    .await
    .unwrap();
    plain.send(&a, probe.clone()).await.unwrap();
    let got = tokio::time::timeout(Duration::from_millis(800), server.recv()).await;
    assert!(
        got.is_err(),
        "untrusted/plaintext peers must not deliver: {got:?}"
    );

    // A properly certified peer works.
    let pki2 = Pki {
        ca_pem: pki.ca_pem.clone(),
        nodes: vec![pki.nodes[0].clone()],
    };
    let good = TcpTransport::bind_tls(
        d.clone(),
        lo,
        HashMap::from([(a.clone(), server.local_addr())]),
        TcpConfig::default(),
        tls_for(&pki2, 0),
    )
    .await
    .unwrap();
    good.send(&a, probe.clone()).await.unwrap();
    let env: Envelope = tokio::time::timeout(Duration::from_secs(5), server.recv())
        .await
        .expect("trusted peer delivers")
        .unwrap();
    assert_eq!(env.from, d);
    assert_eq!(env.msg, probe);
}

#[test]
fn tls_config_rejects_garbage_pem() {
    assert!(TlsConfig::from_pem(b"nope", b"nope", b"nope", None).is_err());
    let pki = make_pki(1);
    // Key that does not match the certificate.
    let other = make_pki(1);
    assert!(TlsConfig::from_pem(&pki.ca_pem, &pki.nodes[0].0, &other.nodes[0].1, None).is_err());
}
