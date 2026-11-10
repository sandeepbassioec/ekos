//! SWIM membership: convergence, failure detection, refutation, leave —
//! over an in-memory network, plus one run over real UDP on loopback.

use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};
use vectorvolt_cluster::gossip_net::{MemNetwork, Transport, UdpTransport};
use vectorvolt_cluster::membership::MembershipList;
use vectorvolt_cluster::node::{NodeId, NodeMeta, NodeState};
use vectorvolt_cluster::swim::{SwimConfig, SwimNode};

fn cfg() -> SwimConfig {
    SwimConfig {
        probe_interval: Duration::from_millis(20),
        probe_timeout: Duration::from_millis(15),
        suspect_timeout: Duration::from_millis(500),
        indirect_probes: 3,
        max_piggyback: 8,
    }
}

fn spawn_node(transport: Arc<dyn Transport>) -> Arc<SwimNode> {
    let meta = NodeMeta {
        id: NodeId::new(),
        addr: transport.local_addr(),
        state: NodeState::Alive,
        incarnation: 0,
        last_seen_ns: 0,
    };
    let node = SwimNode::new(Arc::new(MembershipList::new(meta)), transport, cfg());
    node.start();
    node
}

fn mem_cluster(n: usize) -> (Arc<MemNetwork>, Vec<Arc<SwimNode>>) {
    let net = MemNetwork::new();
    let nodes = (0..n)
        .map(|i| {
            let addr: SocketAddr = format!("10.0.0.{}:7000", i + 1).parse().unwrap();
            spawn_node(net.endpoint(addr))
        })
        .collect();
    (net, nodes)
}

fn state_of(observer: &SwimNode, target: &SwimNode) -> Option<NodeState> {
    let id = target.membership().local_id().clone();
    observer.membership().get(&id).map(|m| m.state)
}

async fn until(limit: Duration, mut f: impl FnMut() -> bool) -> bool {
    let end = Instant::now() + limit;
    while Instant::now() < end {
        if f() {
            return true;
        }
        tokio::time::sleep(Duration::from_millis(5)).await;
    }
    f()
}

fn all_see(nodes: &[Arc<SwimNode>], target: &Arc<SwimNode>, want: NodeState) -> bool {
    nodes
        .iter()
        .filter(|o| !Arc::ptr_eq(o, target))
        .all(|o| state_of(o, target) == Some(want.clone()))
}

async fn join_all(nodes: &[Arc<SwimNode>]) {
    let seed = nodes[0].local_addr();
    for n in &nodes[1..] {
        assert!(n.join(&[seed]).await, "join via seed failed");
    }
}

async fn converge(nodes: &[Arc<SwimNode>]) {
    let ok = until(Duration::from_secs(5), || {
        nodes.iter().all(|o| {
            nodes
                .iter()
                .all(|t| state_of(o, t) == Some(NodeState::Alive))
        })
    })
    .await;
    assert!(ok, "cluster did not converge to all-Alive");
}

#[tokio::test]
async fn five_nodes_converge_to_all_alive() {
    let (_net, nodes) = mem_cluster(5);
    join_all(&nodes).await;
    converge(&nodes).await;
}

#[tokio::test]
async fn killed_node_goes_suspect_then_dead() {
    let (net, nodes) = mem_cluster(5);
    join_all(&nodes).await;
    converge(&nodes).await;

    let victim = nodes[4].clone();
    net.set_down(victim.local_addr(), true);
    victim.shutdown();

    let mut saw_suspect = false;
    let ok = until(Duration::from_secs(5), || {
        saw_suspect |= nodes[..4]
            .iter()
            .any(|o| state_of(o, &victim) == Some(NodeState::Suspect));
        all_see(&nodes[..4], &victim, NodeState::Dead)
    })
    .await;
    assert!(ok, "victim not declared Dead everywhere in time");
    assert!(saw_suspect, "victim must pass through Suspect before Dead");
}

#[tokio::test]
async fn false_suspicion_is_refuted_by_higher_incarnation() {
    let (net, nodes) = mem_cluster(5);
    join_all(&nodes).await;
    converge(&nodes).await;

    let flaky = nodes[3].clone();
    net.set_down(flaky.local_addr(), true);
    let suspected = until(Duration::from_secs(3), || {
        nodes
            .iter()
            .filter(|o| !Arc::ptr_eq(o, &flaky))
            .any(|o| state_of(o, &flaky) == Some(NodeState::Suspect))
    })
    .await;
    assert!(suspected, "flaky node was never suspected");
    // Heal well before the suspicion timeout.
    net.set_down(flaky.local_addr(), false);

    let ok = until(Duration::from_secs(5), || {
        nodes
            .iter()
            .all(|o| state_of(o, &flaky) == Some(NodeState::Alive))
    })
    .await;
    assert!(ok, "suspicion was not refuted");
    let inc = flaky
        .membership()
        .get(flaky.membership().local_id())
        .unwrap()
        .incarnation;
    assert!(inc >= 1, "refutation must bump incarnation, got {inc}");
}

#[tokio::test]
async fn leave_propagates_as_left() {
    let (_net, nodes) = mem_cluster(5);
    join_all(&nodes).await;
    converge(&nodes).await;

    nodes[2].leave().await;
    let ok = until(Duration::from_secs(5), || {
        all_see(&nodes, &nodes[2], NodeState::Left)
    })
    .await;
    assert!(ok, "Left did not propagate");
}

#[tokio::test]
async fn real_udp_cluster_converges_and_detects_failure() {
    let lo: SocketAddr = "127.0.0.1:0".parse().unwrap();
    let mut nodes = Vec::new();
    for _ in 0..3 {
        nodes.push(spawn_node(UdpTransport::bind(lo).await.unwrap()));
    }
    join_all(&nodes).await;
    converge(&nodes).await;

    nodes[2].shutdown();
    let ok = until(Duration::from_secs(6), || {
        all_see(&nodes[..2], &nodes[2], NodeState::Dead)
    })
    .await;
    assert!(ok, "UDP: killed node not declared Dead");
}
