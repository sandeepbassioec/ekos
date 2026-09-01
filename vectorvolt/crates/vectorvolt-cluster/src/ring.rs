use crate::node::NodeId;
use std::collections::hash_map::DefaultHasher;
use std::collections::{BTreeMap, HashMap};
use std::hash::{Hash, Hasher};

fn vnode_hash(id: &NodeId, vnode_idx: usize) -> u64 {
    let mut h = DefaultHasher::new();
    id.0.hash(&mut h);
    vnode_idx.hash(&mut h);
    h.finish()
}

pub struct HashRing {
    vnodes: BTreeMap<u64, NodeId>,
    vnode_count: usize,
}

impl HashRing {
    pub fn new(vnode_count: usize) -> Self {
        HashRing {
            vnodes: BTreeMap::new(),
            vnode_count,
        }
    }

    pub fn add_node(&mut self, id: NodeId) {
        // idempotent: remove first if exists, then re-add
        self.remove_node(&id);
        for i in 0..self.vnode_count {
            let h = vnode_hash(&id, i);
            self.vnodes.insert(h, id.clone());
        }
    }

    pub fn remove_node(&mut self, id: &NodeId) {
        self.vnodes.retain(|_, v| v != id);
    }

    pub fn owner_of(&self, key: u64) -> Option<NodeId> {
        if self.vnodes.is_empty() {
            return None;
        }
        // Find first vnode >= key (wrap around).
        let entry = self
            .vnodes
            .range(key..)
            .next()
            .or_else(|| self.vnodes.iter().next());
        entry.map(|(_, id)| id.clone())
    }

    pub fn replicas_of(&self, key: u64, n: usize) -> Vec<NodeId> {
        if self.vnodes.is_empty() {
            return Vec::new();
        }
        let mut seen = std::collections::HashSet::new();
        let mut result = Vec::new();
        // Walk ring starting from key.
        let after: Vec<NodeId> = self
            .vnodes
            .range(key..)
            .chain(self.vnodes.iter())
            .map(|(_, id)| id.clone())
            .collect();
        for id in after {
            if seen.insert(id.clone()) {
                result.push(id);
                if result.len() == n {
                    break;
                }
            }
        }
        result
    }

    pub fn vnode_count_for(&self, id: &NodeId) -> usize {
        self.vnodes.values().filter(|v| *v == id).count()
    }

    pub fn vnodes_total(&self) -> usize {
        self.vnodes.len()
    }

    pub fn node_count(&self) -> usize {
        let mut set = std::collections::HashSet::new();
        for id in self.vnodes.values() {
            set.insert(id.clone());
        }
        set.len()
    }

    pub fn rebalance_delta(&self, leaving: &NodeId) -> HashMap<u64, NodeId> {
        // For each vnode owned by `leaving`, find the next owner.
        let mut result = HashMap::new();
        for (pos, owner) in &self.vnodes {
            if owner == leaving {
                // Find next node after this pos that isn't `leaving`.
                let next = self
                    .vnodes
                    .range((*pos + 1)..)
                    .chain(self.vnodes.iter())
                    .find(|(_, v)| *v != leaving);
                if let Some((_, new_owner)) = next {
                    result.insert(*pos, new_owner.clone());
                }
            }
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_one_node_owns_all_keys() {
        let mut ring = HashRing::new(256);
        let id = NodeId::new();
        ring.add_node(id.clone());
        assert_eq!(ring.owner_of(0).unwrap(), id);
        assert_eq!(ring.owner_of(u64::MAX / 2).unwrap(), id);
    }

    #[test]
    fn add_two_nodes_splits_roughly_50_50() {
        let mut ring = HashRing::new(256);
        let a = NodeId::new();
        let b = NodeId::new();
        ring.add_node(a.clone());
        ring.add_node(b.clone());
        let total = 1000u64;
        let a_count = (0..total)
            .filter(|&k| ring.owner_of(k * (u64::MAX / total)).unwrap() == a)
            .count();
        // Should be roughly 500 each; allow wide tolerance.
        assert!(a_count > 200 && a_count < 800, "split: {a_count}/1000");
    }

    #[test]
    fn remove_node_redistributes() {
        let mut ring = HashRing::new(256);
        let a = NodeId::new();
        let b = NodeId::new();
        ring.add_node(a.clone());
        ring.add_node(b.clone());
        ring.remove_node(&a);
        assert_eq!(ring.owner_of(0).unwrap(), b);
    }

    #[test]
    fn owner_of_consistent_for_same_key() {
        let mut ring = HashRing::new(256);
        let a = NodeId::new();
        ring.add_node(a.clone());
        let key = 12345678u64;
        assert_eq!(ring.owner_of(key), ring.owner_of(key));
    }

    #[test]
    fn replicas_of_returns_n_distinct_nodes() {
        let mut ring = HashRing::new(256);
        for _ in 0..4 {
            ring.add_node(NodeId::new());
        }
        let replicas = ring.replicas_of(42, 3);
        assert_eq!(replicas.len(), 3);
        // All distinct
        let unique: std::collections::HashSet<_> = replicas.iter().collect();
        assert_eq!(unique.len(), 3);
    }

    #[test]
    fn vnode_count_for_correct() {
        let mut ring = HashRing::new(256);
        let id = NodeId::new();
        ring.add_node(id.clone());
        assert_eq!(ring.vnode_count_for(&id), 256);
    }

    #[test]
    fn node_count_correct() {
        let mut ring = HashRing::new(256);
        ring.add_node(NodeId::new());
        ring.add_node(NodeId::new());
        assert_eq!(ring.node_count(), 2);
    }

    #[test]
    fn sixteen_nodes_256_vnodes_4096_entries() {
        let mut ring = HashRing::new(256);
        for _ in 0..16 {
            ring.add_node(NodeId::new());
        }
        assert_eq!(ring.vnodes.len(), 4096);
    }

    #[test]
    fn key_zero_has_owner() {
        let mut ring = HashRing::new(256);
        ring.add_node(NodeId::new());
        assert!(ring.owner_of(0).is_some());
    }

    #[test]
    fn key_max_has_owner() {
        let mut ring = HashRing::new(256);
        ring.add_node(NodeId::new());
        assert!(ring.owner_of(u64::MAX).is_some());
    }

    #[test]
    fn rebalance_delta_non_empty_after_remove() {
        let mut ring = HashRing::new(4);
        let a = NodeId::new();
        let b = NodeId::new();
        ring.add_node(a.clone());
        ring.add_node(b.clone());
        let delta = ring.rebalance_delta(&a);
        assert!(!delta.is_empty());
    }

    #[test]
    fn add_same_node_twice_is_idempotent() {
        let mut ring = HashRing::new(256);
        let id = NodeId::new();
        ring.add_node(id.clone());
        ring.add_node(id.clone());
        assert_eq!(ring.vnode_count_for(&id), 256);
        assert_eq!(ring.node_count(), 1);
    }

    #[test]
    fn empty_ring_owner_of_returns_none() {
        let ring = HashRing::new(256);
        assert!(ring.owner_of(0).is_none());
    }
}
