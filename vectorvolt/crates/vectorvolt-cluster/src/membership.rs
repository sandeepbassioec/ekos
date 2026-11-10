use crate::node::{NodeId, NodeMeta, NodeState};
use dashmap::DashMap;
use std::sync::Arc;

pub struct MembershipList {
    local_id: NodeId,
    members: DashMap<NodeId, NodeMeta>,
}

impl MembershipList {
    pub fn new(local: NodeMeta) -> Self {
        let members = DashMap::new();
        let local_id = local.id.clone();
        members.insert(local.id.clone(), local);
        MembershipList { local_id, members }
    }

    /// Upsert: newer incarnation wins.
    pub fn upsert(&self, meta: NodeMeta) {
        let id = meta.id.clone();
        let mut should_insert = true;
        if let Some(existing) = self.members.get(&id)
            && existing.incarnation >= meta.incarnation
        {
            should_insert = false;
        }
        if should_insert {
            self.members.insert(id, meta);
        }
    }

    pub fn mark_suspect(&self, id: &NodeId) -> bool {
        if let Some(mut entry) = self.members.get_mut(id)
            && entry.state == NodeState::Alive
        {
            entry.state = NodeState::Suspect;
            return true;
        }
        false
    }

    pub fn mark_dead(&self, id: &NodeId) -> bool {
        if let Some(mut entry) = self.members.get_mut(id)
            && entry.state != NodeState::Dead
        {
            entry.state = NodeState::Dead;
            return true;
        }
        false
    }

    pub fn alive_members(&self) -> Vec<NodeMeta> {
        self.members
            .iter()
            .filter(|e| e.state == NodeState::Alive)
            .map(|e| e.value().clone())
            .collect()
    }

    pub fn len(&self) -> usize {
        self.members.len()
    }

    pub fn is_empty(&self) -> bool {
        self.members.is_empty()
    }

    pub fn get(&self, id: &NodeId) -> Option<NodeMeta> {
        self.members.get(id).map(|e| e.value().clone())
    }

    pub fn local_id(&self) -> &NodeId {
        &self.local_id
    }

    /// Force-set state (used by SWIM prober).
    pub fn set_state(&self, id: &NodeId, state: NodeState) {
        if let Some(mut entry) = self.members.get_mut(id) {
            entry.state = state;
        }
    }

    pub fn set_incarnation(&self, id: &NodeId, inc: u64) {
        if let Some(mut entry) = self.members.get_mut(id) {
            entry.incarnation = inc;
            entry.state = NodeState::Alive;
        }
    }

    pub fn bump_incarnation(&self, id: &NodeId) -> Option<u64> {
        if let Some(mut entry) = self.members.get_mut(id) {
            entry.incarnation += 1;
            return Some(entry.incarnation);
        }
        None
    }

    /// SWIM update-precedence merge. Returns `true` if the update changed
    /// the list. Rules: a higher incarnation always wins; at an equal
    /// incarnation the "worse" state wins (Alive < Suspect < Dead < Left).
    /// Unknown nodes are inserted.
    pub fn apply_update(&self, meta: NodeMeta) -> bool {
        fn rank(s: &NodeState) -> u8 {
            match s {
                NodeState::Alive => 0,
                NodeState::Suspect => 1,
                NodeState::Dead => 2,
                NodeState::Left => 3,
            }
        }
        match self.members.entry(meta.id.clone()) {
            dashmap::mapref::entry::Entry::Vacant(v) => {
                v.insert(meta);
                true
            }
            dashmap::mapref::entry::Entry::Occupied(mut o) => {
                let cur = o.get();
                let wins = meta.incarnation > cur.incarnation
                    || (meta.incarnation == cur.incarnation
                        && rank(&meta.state) > rank(&cur.state));
                if wins {
                    o.insert(meta);
                }
                wins
            }
        }
    }

    /// All members in any state.
    pub fn all_members(&self) -> Vec<NodeMeta> {
        self.members.iter().map(|e| e.value().clone()).collect()
    }

    /// Shared Arc constructor helper.
    pub fn new_arc(local: NodeMeta) -> Arc<Self> {
        Arc::new(Self::new(local))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::node::NodeState;
    use std::net::SocketAddr;

    fn make_meta(id: NodeId, inc: u64) -> NodeMeta {
        NodeMeta {
            id,
            addr: "127.0.0.1:9000".parse::<SocketAddr>().unwrap(),
            state: NodeState::Alive,
            incarnation: inc,
            last_seen_ns: 0,
        }
    }

    #[test]
    fn new_list_has_one_member_self() {
        let id = NodeId::new();
        let list = MembershipList::new(make_meta(id.clone(), 0));
        assert_eq!(list.len(), 1);
        assert!(list.get(&id).is_some());
    }

    #[test]
    fn upsert_adds_member() {
        let self_id = NodeId::new();
        let list = MembershipList::new(make_meta(self_id.clone(), 0));
        let new_id = NodeId::new();
        list.upsert(make_meta(new_id.clone(), 0));
        assert_eq!(list.len(), 2);
    }

    #[test]
    fn upsert_lower_incarnation_ignored() {
        let self_id = NodeId::new();
        let list = MembershipList::new(make_meta(self_id.clone(), 0));
        let id = NodeId::new();
        list.upsert(make_meta(id.clone(), 5));
        let mut stale = make_meta(id.clone(), 3);
        stale.last_seen_ns = 9999;
        list.upsert(stale);
        assert_eq!(list.get(&id).unwrap().incarnation, 5);
    }

    #[test]
    fn upsert_higher_incarnation_updates() {
        let self_id = NodeId::new();
        let list = MembershipList::new(make_meta(self_id.clone(), 0));
        let id = NodeId::new();
        list.upsert(make_meta(id.clone(), 1));
        let mut newer = make_meta(id.clone(), 5);
        newer.last_seen_ns = 42;
        list.upsert(newer);
        assert_eq!(list.get(&id).unwrap().incarnation, 5);
        assert_eq!(list.get(&id).unwrap().last_seen_ns, 42);
    }

    #[test]
    fn mark_suspect_changes_state() {
        let id = NodeId::new();
        let list = MembershipList::new(make_meta(id.clone(), 0));
        assert!(list.mark_suspect(&id));
        assert_eq!(list.get(&id).unwrap().state, NodeState::Suspect);
    }

    #[test]
    fn mark_dead_changes_state() {
        let id = NodeId::new();
        let list = MembershipList::new(make_meta(id.clone(), 0));
        assert!(list.mark_dead(&id));
        assert_eq!(list.get(&id).unwrap().state, NodeState::Dead);
    }

    #[test]
    fn alive_members_excludes_dead() {
        let self_id = NodeId::new();
        let list = MembershipList::new(make_meta(self_id.clone(), 0));
        let dead_id = NodeId::new();
        list.upsert(make_meta(dead_id.clone(), 0));
        list.mark_dead(&dead_id);
        let alive = list.alive_members();
        assert_eq!(alive.len(), 1);
        assert_eq!(alive[0].id, self_id);
    }

    #[test]
    fn len_correct() {
        let self_id = NodeId::new();
        let list = MembershipList::new(make_meta(self_id.clone(), 0));
        list.upsert(make_meta(NodeId::new(), 0));
        list.upsert(make_meta(NodeId::new(), 0));
        assert_eq!(list.len(), 3);
    }

    #[test]
    fn get_existing() {
        let id = NodeId::new();
        let list = MembershipList::new(make_meta(id.clone(), 0));
        assert!(list.get(&id).is_some());
    }

    #[test]
    fn get_missing_returns_none() {
        let self_id = NodeId::new();
        let list = MembershipList::new(make_meta(self_id.clone(), 0));
        assert!(list.get(&NodeId::new()).is_none());
    }

    #[test]
    fn two_nodes_alive_members_returns_two() {
        let a = NodeId::new();
        let b = NodeId::new();
        let list = MembershipList::new(make_meta(a.clone(), 0));
        list.upsert(make_meta(b.clone(), 0));
        assert_eq!(list.alive_members().len(), 2);
    }

    #[test]
    fn incarnation_counter_increment() {
        let id = NodeId::new();
        let list = MembershipList::new(make_meta(id.clone(), 3));
        let new_inc = list.bump_incarnation(&id).unwrap();
        assert_eq!(new_inc, 4);
    }

    #[test]
    fn node_id_equality() {
        let uuid = uuid::Uuid::new_v4();
        let a = NodeId(uuid);
        let b = NodeId(uuid);
        assert_eq!(a, b);
    }
}
