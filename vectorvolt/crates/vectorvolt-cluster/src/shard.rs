use crate::epoch::{EpochFence, FenceError};
use crate::node::NodeId;
use std::collections::BTreeMap;
use std::sync::Arc;
use thiserror::Error;

#[derive(Debug, Clone)]
pub struct ShardRange {
    pub shard_id: u64,
    pub lo: u64,
    pub hi: u64,
    pub owner: NodeId,
    pub epoch: u64,
}

#[derive(Debug, Error)]
pub enum ShardError {
    #[error("shard {0} not found")]
    NotFound(u64),
    #[error("shards are not adjacent")]
    NotAdjacent,
    #[error("split point out of range")]
    SplitOutOfRange,
    #[error("stale epoch: {0}")]
    StaleEpoch(#[from] FenceError),
}

pub struct ShardMap {
    shards: BTreeMap<u64, ShardRange>,
    next_id: u64,
    epoch_fence: Arc<EpochFence>,
}

impl ShardMap {
    pub fn new(initial_node: NodeId, fence: Arc<EpochFence>) -> Self {
        let mut shards = BTreeMap::new();
        let shard = ShardRange {
            shard_id: 1,
            lo: 0,
            hi: u64::MAX,
            owner: initial_node,
            epoch: fence.current(),
        };
        shards.insert(0, shard);
        ShardMap {
            shards,
            next_id: 2,
            epoch_fence: fence,
        }
    }

    pub fn shard_for(&self, pk: u64) -> Option<&ShardRange> {
        // Find the shard whose lo <= pk < hi.
        self.shards
            .range(..=pk)
            .next_back()
            .map(|(_, s)| s)
            .filter(|s| pk < s.hi || s.hi == u64::MAX)
    }

    pub fn split(
        &mut self,
        shard_id: u64,
        at: u64,
        new_owner: NodeId,
    ) -> Result<[ShardRange; 2], ShardError> {
        let key = self
            .shards
            .iter()
            .find(|(_, s)| s.shard_id == shard_id)
            .map(|(k, _)| *k);
        let key = key.ok_or(ShardError::NotFound(shard_id))?;
        let original = self.shards.remove(&key).unwrap();
        if at <= original.lo || at >= original.hi {
            self.shards.insert(key, original);
            return Err(ShardError::SplitOutOfRange);
        }
        let epoch = self.epoch_fence.current();
        let lo_shard = ShardRange {
            shard_id: original.shard_id,
            lo: original.lo,
            hi: at,
            owner: original.owner.clone(),
            epoch,
        };
        let hi_shard = ShardRange {
            shard_id: self.next_id,
            lo: at,
            hi: original.hi,
            owner: new_owner,
            epoch,
        };
        self.next_id += 1;
        self.shards.insert(lo_shard.lo, lo_shard.clone());
        self.shards.insert(hi_shard.lo, hi_shard.clone());
        Ok([lo_shard, hi_shard])
    }

    pub fn merge(&mut self, shard_a: u64, shard_b: u64) -> Result<ShardRange, ShardError> {
        let key_a = self
            .shards
            .iter()
            .find(|(_, s)| s.shard_id == shard_a)
            .map(|(k, _)| *k);
        let key_b = self
            .shards
            .iter()
            .find(|(_, s)| s.shard_id == shard_b)
            .map(|(k, _)| *k);
        let (key_a, key_b) = (
            key_a.ok_or(ShardError::NotFound(shard_a))?,
            key_b.ok_or(ShardError::NotFound(shard_b))?,
        );

        let sa = self.shards[&key_a].clone();
        let sb = self.shards[&key_b].clone();

        // Must be adjacent: sa.hi == sb.lo or sb.hi == sa.lo
        let (lo_s, hi_s) = if sa.hi == sb.lo {
            (sa, sb)
        } else if sb.hi == sa.lo {
            (sb, sa)
        } else {
            return Err(ShardError::NotAdjacent);
        };

        self.shards.remove(&lo_s.lo);
        self.shards.remove(&hi_s.lo);

        let merged = ShardRange {
            shard_id: lo_s.shard_id,
            lo: lo_s.lo,
            hi: hi_s.hi,
            owner: lo_s.owner.clone(),
            epoch: self.epoch_fence.current(),
        };
        self.shards.insert(merged.lo, merged.clone());
        Ok(merged)
    }

    pub fn reassign(
        &mut self,
        shard_id: u64,
        new_owner: NodeId,
        new_epoch: u64,
    ) -> Result<(), ShardError> {
        self.epoch_fence.validate(new_epoch)?;
        let key = self
            .shards
            .iter()
            .find(|(_, s)| s.shard_id == shard_id)
            .map(|(k, _)| *k);
        let key = key.ok_or(ShardError::NotFound(shard_id))?;
        if let Some(s) = self.shards.get_mut(&key) {
            s.owner = new_owner;
            s.epoch = new_epoch;
        }
        Ok(())
    }

    pub fn shard_count(&self) -> usize {
        self.shards.len()
    }

    pub fn all_shards(&self) -> Vec<&ShardRange> {
        self.shards.values().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    fn fence() -> Arc<EpochFence> {
        Arc::new(EpochFence::new())
    }

    #[test]
    fn new_map_has_one_shard_covering_full_range() {
        let f = fence();
        let map = ShardMap::new(NodeId::new(), f);
        assert_eq!(map.shard_count(), 1);
        let shards = map.all_shards();
        assert_eq!(shards[0].lo, 0);
        assert_eq!(shards[0].hi, u64::MAX);
    }

    #[test]
    fn shard_for_pk_zero_returns_shard() {
        let f = fence();
        let map = ShardMap::new(NodeId::new(), f);
        assert!(map.shard_for(0).is_some());
    }

    #[test]
    fn shard_for_pk_max_returns_shard() {
        let f = fence();
        let map = ShardMap::new(NodeId::new(), f);
        assert!(map.shard_for(u64::MAX - 1).is_some());
    }

    #[test]
    fn split_creates_two_shards() {
        let f = fence();
        let mut map = ShardMap::new(NodeId::new(), f);
        map.split(1, 500, NodeId::new()).unwrap();
        assert_eq!(map.shard_count(), 2);
    }

    #[test]
    fn split_lo_shard_keys_lo_to_at() {
        let f = fence();
        let mut map = ShardMap::new(NodeId::new(), f);
        let [lo, _hi] = map.split(1, 500, NodeId::new()).unwrap();
        assert_eq!(lo.lo, 0);
        assert_eq!(lo.hi, 500);
    }

    #[test]
    fn split_hi_shard_keys_at_to_hi() {
        let f = fence();
        let mut map = ShardMap::new(NodeId::new(), f);
        let [_lo, hi] = map.split(1, 500, NodeId::new()).unwrap();
        assert_eq!(hi.lo, 500);
        assert_eq!(hi.hi, u64::MAX);
    }

    #[test]
    fn merge_two_adjacent_shards() {
        let f = fence();
        let mut map = ShardMap::new(NodeId::new(), f);
        let [lo, hi] = map.split(1, 500, NodeId::new()).unwrap();
        map.merge(lo.shard_id, hi.shard_id).unwrap();
        assert_eq!(map.shard_count(), 1);
    }

    #[test]
    fn merge_non_adjacent_returns_err() {
        let f = fence();
        let node = NodeId::new();
        let mut map = ShardMap::new(node.clone(), f);
        // Split twice to create 3 shards.
        let [s1, _s2] = map.split(1, 300, NodeId::new()).unwrap();
        // Try to merge s1 (0..300) with s3 (600..MAX) — non-adjacent.
        // We need a third split first.
        let s2_id = map
            .all_shards()
            .iter()
            .find(|s| s.lo == 300)
            .map(|s| s.shard_id)
            .unwrap();
        let [_, s3] = map.split(s2_id, 600, NodeId::new()).unwrap();
        let result = map.merge(s1.shard_id, s3.shard_id);
        assert!(matches!(result, Err(ShardError::NotAdjacent)));
    }

    #[test]
    fn reassign_changes_owner() {
        let f = fence();
        let f_clone = f.clone();
        let mut map = ShardMap::new(NodeId::new(), f);
        let new_owner = NodeId::new();
        let current_epoch = f_clone.current();
        map.reassign(1, new_owner.clone(), current_epoch).unwrap();
        assert_eq!(map.all_shards()[0].owner, new_owner);
    }

    #[test]
    fn shard_count_correct_after_split() {
        let f = fence();
        let mut map = ShardMap::new(NodeId::new(), f);
        map.split(1, 100, NodeId::new()).unwrap();
        map.split(2, 200, NodeId::new()).unwrap();
        assert_eq!(map.shard_count(), 3);
    }

    #[test]
    fn shard_count_decreases_after_merge() {
        let f = fence();
        let mut map = ShardMap::new(NodeId::new(), f);
        let [a, b] = map.split(1, 500, NodeId::new()).unwrap();
        assert_eq!(map.shard_count(), 2);
        map.merge(a.shard_id, b.shard_id).unwrap();
        assert_eq!(map.shard_count(), 1);
    }

    #[test]
    fn stale_epoch_reassign_rejected() {
        let f = fence();
        f.advance(1); // current = 1
        f.advance(2); // current = 2
        let mut map = ShardMap::new(NodeId::new(), f);
        let result = map.reassign(1, NodeId::new(), 0); // epoch 0 < current 2
        assert!(result.is_err());
    }

    #[test]
    fn split_at_boundary_ok() {
        let f = fence();
        let mut map = ShardMap::new(NodeId::new(), f);
        // Split at 1 (lo+1, valid boundary).
        let result = map.split(1, 1, NodeId::new());
        assert!(result.is_ok());
    }
}
