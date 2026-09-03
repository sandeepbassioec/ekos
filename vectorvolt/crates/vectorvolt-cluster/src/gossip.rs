use crate::membership::MembershipList;
use crate::node::{NodeId, NodeMeta};
use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GossipMessage {
    pub from: NodeId,
    pub epoch: u64,
    pub deltas: Vec<NodeMeta>,
}

pub struct GossipEngine {
    membership: Arc<MembershipList>,
    local_id: NodeId,
    epoch: AtomicU64,
}

impl GossipEngine {
    pub fn new(membership: Arc<MembershipList>, local_id: NodeId) -> Self {
        GossipEngine {
            membership,
            local_id,
            epoch: AtomicU64::new(0),
        }
    }

    /// Pick `fanout` random targets (excluding self), package all member deltas.
    pub fn make_gossip(&self, fanout: usize) -> Vec<GossipMessage> {
        if fanout == 0 {
            return Vec::new();
        }
        let all = self.membership.alive_members();
        let targets: Vec<&NodeMeta> = all.iter().filter(|m| m.id != self.local_id).collect();
        let count = fanout.min(targets.len());
        if count == 0 {
            return Vec::new();
        }
        let mut rng = rand::thread_rng();
        let mut chosen: Vec<&NodeMeta> = targets.clone();
        chosen.shuffle(&mut rng);
        chosen.truncate(count);

        let epoch = self.epoch.load(Ordering::SeqCst);
        let deltas: Vec<NodeMeta> = all.clone();

        chosen
            .iter()
            .map(|_target| GossipMessage {
                from: self.local_id.clone(),
                epoch,
                deltas: deltas.clone(),
            })
            .collect()
    }

    /// Apply deltas from a gossip message; returns count of applied updates.
    pub fn apply_gossip(&self, msg: GossipMessage) -> usize {
        let mut count = 0;
        for delta in msg.deltas {
            let id = delta.id.clone();
            let current = self.membership.get(&id);
            match current {
                None => {
                    self.membership.upsert(delta);
                    count += 1;
                }
                Some(existing) => {
                    if delta.incarnation > existing.incarnation {
                        self.membership.upsert(delta);
                        count += 1;
                    }
                }
            }
        }
        count
    }

    pub fn current_epoch(&self) -> u64 {
        self.epoch.load(Ordering::SeqCst)
    }

    pub fn bump_epoch(&self) -> u64 {
        self.epoch.fetch_add(1, Ordering::SeqCst) + 1
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::node::NodeState;
    use std::net::SocketAddr;

    fn addr() -> SocketAddr {
        "127.0.0.1:9000".parse().unwrap()
    }

    fn make_meta(id: NodeId, inc: u64) -> NodeMeta {
        NodeMeta {
            id,
            addr: addr(),
            state: NodeState::Alive,
            incarnation: inc,
            last_seen_ns: 0,
        }
    }

    fn engine(self_id: NodeId) -> (Arc<MembershipList>, GossipEngine) {
        let ml = Arc::new(MembershipList::new(make_meta(self_id.clone(), 0)));
        let ge = GossipEngine::new(ml.clone(), self_id);
        (ml, ge)
    }

    #[test]
    fn make_gossip_produces_fanout_messages() {
        let self_id = NodeId::new();
        let (ml, ge) = engine(self_id.clone());
        for _ in 0..3 {
            ml.upsert(make_meta(NodeId::new(), 0));
        }
        let msgs = ge.make_gossip(2);
        assert_eq!(msgs.len(), 2);
    }

    #[test]
    fn each_message_from_local_id() {
        let self_id = NodeId::new();
        let (ml, ge) = engine(self_id.clone());
        ml.upsert(make_meta(NodeId::new(), 0));
        let msgs = ge.make_gossip(1);
        assert_eq!(msgs[0].from, self_id);
    }

    #[test]
    fn apply_gossip_adds_new_member() {
        let self_id = NodeId::new();
        let (ml, ge) = engine(self_id.clone());
        let new_id = NodeId::new();
        let msg = GossipMessage {
            from: NodeId::new(),
            epoch: 0,
            deltas: vec![make_meta(new_id.clone(), 1)],
        };
        let applied = ge.apply_gossip(msg);
        assert_eq!(applied, 1);
        assert!(ml.get(&new_id).is_some());
    }

    #[test]
    fn apply_gossip_updates_with_higher_incarnation() {
        let self_id = NodeId::new();
        let (ml, ge) = engine(self_id.clone());
        let id = NodeId::new();
        ml.upsert(make_meta(id.clone(), 1));
        let msg = GossipMessage {
            from: NodeId::new(),
            epoch: 0,
            deltas: vec![make_meta(id.clone(), 5)],
        };
        ge.apply_gossip(msg);
        assert_eq!(ml.get(&id).unwrap().incarnation, 5);
    }

    #[test]
    fn apply_gossip_ignores_stale_incarnation() {
        let self_id = NodeId::new();
        let (ml, ge) = engine(self_id.clone());
        let id = NodeId::new();
        ml.upsert(make_meta(id.clone(), 5));
        let msg = GossipMessage {
            from: NodeId::new(),
            epoch: 0,
            deltas: vec![make_meta(id.clone(), 2)],
        };
        let applied = ge.apply_gossip(msg);
        assert_eq!(applied, 0);
        assert_eq!(ml.get(&id).unwrap().incarnation, 5);
    }

    #[test]
    fn epoch_increments_on_bump() {
        let self_id = NodeId::new();
        let (_, ge) = engine(self_id);
        assert_eq!(ge.current_epoch(), 0);
        assert_eq!(ge.bump_epoch(), 1);
        assert_eq!(ge.current_epoch(), 1);
    }

    #[test]
    fn two_nodes_converge_after_gossip_exchange() {
        let a_id = NodeId::new();
        let b_id = NodeId::new();
        let (ml_a, ge_a) = engine(a_id.clone());
        let (ml_b, ge_b) = engine(b_id.clone());

        // A knows about a new node C.
        let c_id = NodeId::new();
        ml_a.upsert(make_meta(c_id.clone(), 1));

        // B doesn't appear in A's alive list (not upserted), so add B to A's membership first.
        ml_a.upsert(make_meta(b_id.clone(), 0));
        let msgs = ge_a.make_gossip(1);
        for msg in msgs {
            ge_b.apply_gossip(msg);
        }
        assert!(ml_b.get(&c_id).is_some() || ml_b.get(&a_id).is_some());
    }

    #[test]
    fn empty_membership_make_gossip_returns_empty() {
        let self_id = NodeId::new();
        let (_, ge) = engine(self_id);
        // Only self in membership — no targets.
        let msgs = ge.make_gossip(3);
        assert!(msgs.is_empty());
    }

    #[test]
    fn gossip_message_serializes_deserializes() {
        let msg = GossipMessage {
            from: NodeId::new(),
            epoch: 42,
            deltas: vec![],
        };
        let json = serde_json::to_string(&msg).unwrap();
        let decoded: GossipMessage = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.epoch, 42);
        assert_eq!(decoded.from, msg.from);
    }

    #[test]
    fn apply_zero_deltas_returns_zero() {
        let self_id = NodeId::new();
        let (_, ge) = engine(self_id.clone());
        let msg = GossipMessage {
            from: NodeId::new(),
            epoch: 0,
            deltas: vec![],
        };
        assert_eq!(ge.apply_gossip(msg), 0);
    }

    #[test]
    fn apply_same_delta_twice_is_idempotent() {
        let self_id = NodeId::new();
        let (ml, ge) = engine(self_id.clone());
        let id = NodeId::new();
        let meta = make_meta(id.clone(), 3);
        let msg1 = GossipMessage {
            from: NodeId::new(),
            epoch: 0,
            deltas: vec![meta.clone()],
        };
        let msg2 = GossipMessage {
            from: NodeId::new(),
            epoch: 0,
            deltas: vec![meta.clone()],
        };
        ge.apply_gossip(msg1);
        ge.apply_gossip(msg2);
        assert_eq!(ml.get(&id).unwrap().incarnation, 3);
    }

    #[test]
    fn gossip_epoch_monotonic() {
        let self_id = NodeId::new();
        let (_, ge) = engine(self_id);
        let e1 = ge.bump_epoch();
        let e2 = ge.bump_epoch();
        let e3 = ge.bump_epoch();
        assert!(e1 < e2 && e2 < e3);
    }

    #[test]
    fn fanout_zero_returns_empty() {
        let self_id = NodeId::new();
        let (ml, ge) = engine(self_id.clone());
        ml.upsert(make_meta(NodeId::new(), 0));
        assert!(ge.make_gossip(0).is_empty());
    }
}
