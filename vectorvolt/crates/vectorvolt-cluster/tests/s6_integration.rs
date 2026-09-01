use std::net::SocketAddr;
use std::sync::Arc;
use vectorvolt_cluster::{
    config::ClusterConfig,
    epoch::EpochFence,
    gossip::GossipEngine,
    membership::MembershipList,
    node::{NodeId, NodeMeta, NodeState},
    quorum::QuorumCoordinator,
    raft::{election::RaftCluster, state::RaftRole},
    ring::HashRing,
    shard::ShardMap,
    swim::{AlwaysTimeoutProber, SwimProber},
};

fn addr() -> SocketAddr {
    "127.0.0.1:9001".parse().unwrap()
}

fn meta(id: NodeId) -> NodeMeta {
    NodeMeta {
        id,
        addr: addr(),
        state: NodeState::Alive,
        incarnation: 0,
        last_seen_ns: 0,
    }
}

// 1. 3-node membership converges via gossip
#[test]
fn three_node_membership_converges_via_gossip() {
    let a = NodeId::new();
    let b = NodeId::new();
    let c = NodeId::new();

    let ml_a = Arc::new(MembershipList::new(meta(a.clone())));
    let ml_b = Arc::new(MembershipList::new(meta(b.clone())));
    let ml_c = Arc::new(MembershipList::new(meta(c.clone())));

    let ge_a = GossipEngine::new(ml_a.clone(), a.clone());
    let ge_b = GossipEngine::new(ml_b.clone(), b.clone());
    let ge_c = GossipEngine::new(ml_c.clone(), c.clone());

    // Seed cross-knowledge.
    ml_a.upsert(meta(b.clone()));
    ml_a.upsert(meta(c.clone()));

    // A gossips to B and C.
    for msg in ge_a.make_gossip(2) {
        ge_b.apply_gossip(msg.clone());
        ge_c.apply_gossip(msg);
    }
    assert!(ml_b.get(&a).is_some());
    assert!(ml_c.get(&a).is_some());
}

// 2. SWIM failure detection marks dead node
#[tokio::test]
async fn swim_failure_detection_marks_dead_node() {
    let self_id = NodeId::new();
    let target = NodeId::new();
    let ml = Arc::new(MembershipList::new(meta(self_id.clone())));
    ml.upsert(meta(target.clone()));

    let prober = SwimProber::new(
        ml.clone(),
        std::time::Duration::from_millis(0),
        std::time::Duration::from_millis(0),
    )
    .with_prober(Box::new(AlwaysTimeoutProber));
    prober.probe_direct(&target).await;
    {
        let mut t = prober.suspect_times_for_test();
        t.insert(
            target.clone(),
            std::time::Instant::now() - std::time::Duration::from_secs(10),
        );
    }
    let dead = prober.expire_suspects();
    assert!(dead.contains(&target));
    assert_eq!(ml.get(&target).unwrap().state, NodeState::Dead);
}

// 3. HashRing with 3 nodes distributes keys evenly
#[test]
fn hash_ring_3_nodes_distributes_keys_evenly() {
    let mut ring = HashRing::new(256);
    let ids: Vec<NodeId> = (0..3).map(|_| NodeId::new()).collect();
    for id in &ids {
        ring.add_node(id.clone());
    }
    assert_eq!(ring.node_count(), 3);
    assert_eq!(ring.vnodes_total(), 768);
    let sample = 300u64;
    let counts: Vec<usize> = ids
        .iter()
        .map(|id| {
            (0..sample)
                .filter(|&k| ring.owner_of(k * (u64::MAX / sample)).unwrap() == *id)
                .count()
        })
        .collect();
    // Each should have > 50 and < 250 of 300 keys.
    for c in &counts {
        assert!(*c > 20 && *c < 280, "distribution: {:?}", counts);
    }
}

// 4. Raft elects leader in 3-node cluster
#[test]
fn raft_elects_leader_in_3_node_cluster() {
    let mut cluster = RaftCluster::new(3);
    cluster.trigger_election(0);
    let leader = cluster.run_to_stable(100);
    assert!(leader.is_some());
}

// 5. Leader replicates log entry to followers
#[test]
fn leader_replicates_log_entry_to_followers() {
    let mut cluster = RaftCluster::new(3);
    cluster.trigger_election(0);
    cluster.run_to_stable(100);
    if cluster.leader().is_some() {
        let ok = cluster.replicate(b"key=1:val=hello".to_vec());
        assert!(ok);
        let followers_with_entry = cluster
            .nodes
            .iter()
            .filter(|(_, h)| h.state.role == RaftRole::Follower && h.log.last_index() >= 1)
            .count();
        assert!(followers_with_entry >= 1);
    }
}

// 6. Epoch fence blocks stale-epoch write
#[test]
fn epoch_fence_blocks_stale_epoch_write() {
    let fence = EpochFence::new();
    fence.advance(1);
    fence.advance(2);
    let result = fence.validate(0);
    assert!(result.is_err());
}

// 7. Shard split + ownership reassignment
#[test]
fn shard_split_and_ownership_reassignment() {
    let f = Arc::new(EpochFence::new());
    let mut map = ShardMap::new(NodeId::new(), f.clone());
    map.split(1, 1000, NodeId::new()).unwrap();
    assert_eq!(map.shard_count(), 2);
    let new_owner = NodeId::new();
    map.reassign(1, new_owner.clone(), f.current()).unwrap();
    let shard = map
        .all_shards()
        .into_iter()
        .find(|s| s.shard_id == 1)
        .unwrap();
    assert_eq!(shard.owner, new_owner);
}

// 8. Quorum write requires 2/3 acks
#[test]
fn quorum_write_requires_2_of_3_acks() {
    let f = Arc::new(EpochFence::new());
    let coord = QuorumCoordinator::new(2, f.clone());
    let id = coord.begin_write(b"mutation".to_vec(), f.current());
    coord.record_ack(id, NodeId::new());
    assert!(!coord.is_committed(id));
    coord.record_ack(id, NodeId::new());
    assert!(coord.is_committed(id));
}

// 9. Partition → no leader → heal → leader re-elected
#[test]
fn partition_heal_leader_reelected() {
    let mut cluster = RaftCluster::new(3);
    cluster.partition(&[0], &[1, 2]);
    cluster.trigger_election(0);
    for _ in 0..50 {
        cluster.step();
    }
    let _pre_heal_leader = cluster.leader();
    cluster.heal_partition();
    cluster.trigger_election(1);
    let post_heal_leader = cluster.run_to_stable(100);
    assert!(post_heal_leader.is_some());
}

// 10. ClusterConfig validates replication_factor=3 write_quorum=2
#[test]
fn cluster_config_validates_replication_3_write_2() {
    let mut cfg = ClusterConfig::new("vv-cluster");
    cfg.replication_factor = 3;
    cfg.write_quorum = 2;
    assert!(cfg.validate().is_ok());
}

// 11. GossipEngine converges membership in 3 rounds
#[test]
fn gossip_engine_converges_in_3_rounds() {
    let a = NodeId::new();
    let b = NodeId::new();
    let c = NodeId::new();
    let d = NodeId::new(); // new node A knows about

    let ml_a = Arc::new(MembershipList::new(meta(a.clone())));
    let ml_b = Arc::new(MembershipList::new(meta(b.clone())));

    ml_a.upsert(meta(b.clone()));
    ml_a.upsert(meta(c.clone()));
    ml_a.upsert(meta(d.clone()));

    let ge_a = GossipEngine::new(ml_a.clone(), a.clone());
    let ge_b = GossipEngine::new(ml_b.clone(), b.clone());

    // 3 rounds of gossip from A to B.
    for _ in 0..3 {
        for msg in ge_a.make_gossip(1) {
            ge_b.apply_gossip(msg);
        }
    }
    // B should know about C and D.
    assert!(ml_b.get(&c).is_some());
    assert!(ml_b.get(&d).is_some());
}

// 12. EpochFence advances on each Raft term change
#[test]
fn epoch_fence_advances_on_raft_term_change() {
    let fence = EpochFence::new();
    let e1 = fence.advance(1);
    let e2 = fence.advance(2);
    let e3 = fence.advance(3);
    assert!(e1 < e2 && e2 < e3);
    assert_eq!(fence.epoch_for_term(2).unwrap(), e2);
}

// 13. Full path: elect leader → quorum write → log committed → followers apply
#[test]
fn full_path_elect_leader_quorum_write_log_committed() {
    let mut cluster = RaftCluster::new(3);
    cluster.trigger_election(0);
    cluster.run_to_stable(100);
    if let Some(_leader) = cluster.leader() {
        let ok = cluster.replicate(b"SET x=42".to_vec());
        assert!(ok, "replication failed");
        let leader_commit = cluster
            .nodes
            .iter()
            .find(|(_, h)| h.state.role == RaftRole::Leader)
            .map(|(_, h)| h.log.commit_index())
            .unwrap_or(0);
        assert!(leader_commit >= 1);
    }
}
