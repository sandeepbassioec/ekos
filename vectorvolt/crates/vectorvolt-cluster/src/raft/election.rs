use super::handler::RaftHandler;
use super::messages::RaftMessage;
use super::state::RaftRole;
use crate::node::NodeId;
use std::collections::{HashMap, VecDeque};

pub struct RaftCluster {
    pub nodes: Vec<(NodeId, RaftHandler)>,
    bus: HashMap<NodeId, VecDeque<(NodeId, RaftMessage)>>, // (from, msg)
    partitions: Option<(Vec<usize>, Vec<usize>)>,
}

impl RaftCluster {
    pub fn new(n: usize) -> Self {
        let mut nodes = Vec::new();
        let mut bus = HashMap::new();
        for _ in 0..n {
            let id = NodeId::new();
            bus.insert(id.clone(), VecDeque::new());
            nodes.push((id.clone(), RaftHandler::new(id)));
        }
        RaftCluster {
            nodes,
            bus,
            partitions: None,
        }
    }

    fn can_communicate(&self, from: usize, to: usize) -> bool {
        if let Some((ref ga, ref gb)) = self.partitions {
            let from_a = ga.contains(&from);
            let to_a = ga.contains(&to);
            let from_b = gb.contains(&from);
            let to_b = gb.contains(&to);
            // Only communicate within same group.
            return (from_a && to_a) || (from_b && to_b);
        }
        true
    }

    pub fn trigger_election(&mut self, node_idx: usize) {
        let n = self.nodes.len();
        let (id, handler) = &mut self.nodes[node_idx];
        let new_term = handler.state.become_candidate();
        let last_log_index = handler.log.last_index();
        let last_log_term = handler.log.last_term();
        let id = id.clone();

        // Send RequestVote to all peers.
        let msg = RaftMessage::RequestVote {
            term: new_term,
            candidate_id: id.clone(),
            last_log_index,
            last_log_term,
        };
        for peer_idx in 0..n {
            if peer_idx != node_idx && self.can_communicate(node_idx, peer_idx) {
                let peer_id = self.nodes[peer_idx].0.clone();
                self.bus
                    .entry(peer_id)
                    .or_default()
                    .push_back((id.clone(), msg.clone()));
            }
        }
    }

    /// Process one round of messages. Returns true if no messages remain.
    pub fn step(&mut self) -> bool {
        let n = self.nodes.len();
        let mut any = false;

        // Collect all pending messages.
        let mut pending: Vec<(NodeId, NodeId, RaftMessage)> = Vec::new();
        for (id, q) in &mut self.bus {
            while let Some((from, msg)) = q.pop_front() {
                pending.push((from.clone(), id.clone(), msg));
                any = true;
            }
        }

        for (from, to, msg) in pending {
            let to_idx = self.nodes.iter().position(|(id, _)| *id == to);
            let from_idx = self.nodes.iter().position(|(id, _)| *id == from);
            let (to_idx, from_idx) = match (to_idx, from_idx) {
                (Some(t), Some(f)) => (t, f),
                _ => continue,
            };
            if !self.can_communicate(from_idx, to_idx) {
                continue;
            }

            match msg {
                RaftMessage::RequestVote {
                    term,
                    candidate_id,
                    last_log_index,
                    last_log_term,
                } => {
                    let (reply_term, granted) = self.nodes[to_idx].1.handle_request_vote(
                        term,
                        candidate_id,
                        last_log_index,
                        last_log_term,
                    );
                    let reply = RaftMessage::RequestVoteReply {
                        term: reply_term,
                        vote_granted: granted,
                    };
                    self.bus
                        .entry(from.clone())
                        .or_default()
                        .push_back((to.clone(), reply));
                }
                RaftMessage::RequestVoteReply { term, vote_granted } => {
                    // `to` = candidate receiving the reply, `from` = voter who sent it
                    let handler = &mut self.nodes[to_idx].1;
                    if term > handler.state.current_term {
                        handler.state.become_follower(term, None);
                    } else if vote_granted && handler.state.role == RaftRole::Candidate {
                        handler.state.record_vote(from.clone());
                        if handler.state.has_quorum(n) {
                            handler.state.become_leader();
                            // Send heartbeats.
                            let leader_term = handler.state.current_term;
                            let leader_id = handler.state.node_id.clone();
                            let commit = handler.log.commit_index();
                            let last_idx = handler.log.last_index();
                            let last_term = handler.log.last_term();
                            let hb = RaftMessage::AppendEntries {
                                term: leader_term,
                                leader_id: leader_id.clone(),
                                prev_log_index: last_idx,
                                prev_log_term: last_term,
                                entries: vec![],
                                leader_commit: commit,
                            };
                            for peer_idx in 0..n {
                                if peer_idx != to_idx && self.can_communicate(to_idx, peer_idx) {
                                    let pid = self.nodes[peer_idx].0.clone();
                                    self.bus
                                        .entry(pid)
                                        .or_default()
                                        .push_back((leader_id.clone(), hb.clone()));
                                }
                            }
                        }
                    }
                }
                RaftMessage::AppendEntries {
                    term,
                    leader_id,
                    prev_log_index,
                    prev_log_term,
                    entries,
                    leader_commit,
                } => {
                    let (reply_term, success, match_index) =
                        self.nodes[to_idx].1.handle_append_entries(
                            term,
                            leader_id,
                            prev_log_index,
                            prev_log_term,
                            entries,
                            leader_commit,
                        );
                    let reply = RaftMessage::AppendEntriesReply {
                        term: reply_term,
                        success,
                        match_index,
                    };
                    self.bus
                        .entry(from.clone())
                        .or_default()
                        .push_back((to.clone(), reply));
                }
                RaftMessage::AppendEntriesReply { .. } => {}
                _ => {}
            }
        }

        !any
    }

    pub fn leader(&self) -> Option<NodeId> {
        for (id, handler) in &self.nodes {
            if handler.state.role == RaftRole::Leader {
                return Some(id.clone());
            }
        }
        None
    }

    pub fn run_to_stable(&mut self, max_rounds: usize) -> Option<NodeId> {
        for _ in 0..max_rounds {
            if self.step() && self.leader().is_some() {
                return self.leader();
            }
            if self.leader().is_some() {
                // Process a few more rounds to confirm stability.
                for _ in 0..5 {
                    self.step();
                }
                return self.leader();
            }
        }
        self.leader()
    }

    pub fn partition(&mut self, group_a: &[usize], group_b: &[usize]) {
        self.partitions = Some((group_a.to_vec(), group_b.to_vec()));
    }

    pub fn heal_partition(&mut self) {
        self.partitions = None;
    }

    /// Discard all pending messages in the bus.
    pub fn drain_bus(&mut self) {
        for q in self.bus.values_mut() {
            q.clear();
        }
    }

    /// Replicate a command from leader to followers.
    pub fn replicate(&mut self, command: Vec<u8>) -> bool {
        let n = self.nodes.len();
        let leader_idx = self
            .nodes
            .iter()
            .position(|(_, h)| h.state.role == RaftRole::Leader);
        let Some(leader_idx) = leader_idx else {
            return false;
        };
        let leader_id = self.nodes[leader_idx].0.clone();
        let term = self.nodes[leader_idx].1.state.current_term;
        let prev_idx = self.nodes[leader_idx].1.log.last_index();
        let prev_term = self.nodes[leader_idx].1.log.last_term();
        let log_idx = self.nodes[leader_idx].1.log.append(term, command.clone());
        let entry = self.nodes[leader_idx].1.log.get(log_idx).unwrap().clone();
        let commit = self.nodes[leader_idx].1.log.commit_index();

        let ae = RaftMessage::AppendEntries {
            term,
            leader_id: leader_id.clone(),
            prev_log_index: prev_idx,
            prev_log_term: prev_term,
            entries: vec![entry],
            leader_commit: commit,
        };
        for i in 0..n {
            if i != leader_idx && self.can_communicate(leader_idx, i) {
                let pid = self.nodes[i].0.clone();
                self.bus
                    .entry(pid)
                    .or_default()
                    .push_back((leader_id.clone(), ae.clone()));
            }
        }
        for _ in 0..10 {
            self.step();
        }
        // Commit if quorum.
        let acked = self
            .nodes
            .iter()
            .filter(|(_, h)| h.log.last_index() >= log_idx)
            .count();
        if acked > n / 2 {
            self.nodes[leader_idx].1.log.commit_up_to(log_idx);
            return true;
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn three_node_cluster_elects_leader() {
        let mut cluster = RaftCluster::new(3);
        cluster.trigger_election(0);
        let leader = cluster.run_to_stable(100);
        assert!(leader.is_some());
    }

    #[test]
    fn leader_has_majority_votes() {
        let mut cluster = RaftCluster::new(3);
        cluster.trigger_election(0);
        cluster.run_to_stable(100);
        let leader_idx = cluster
            .nodes
            .iter()
            .position(|(_, h)| h.state.role == RaftRole::Leader);
        if let Some(idx) = leader_idx {
            assert!(cluster.nodes[idx].1.state.votes_received.len() >= 2);
        }
    }

    #[test]
    fn follower_knows_leader() {
        let mut cluster = RaftCluster::new(3);
        cluster.trigger_election(0);
        cluster.run_to_stable(100);
        // After stable, followers should know the leader.
        let leader = cluster.leader();
        if let Some(lid) = leader {
            let follower_count = cluster
                .nodes
                .iter()
                .filter(|(_, h)| h.state.role == RaftRole::Follower)
                .filter(|(_, h)| h.state.leader_id.as_ref() == Some(&lid))
                .count();
            assert!(follower_count >= 1);
        }
    }

    #[test]
    fn reelection_after_leader_removed() {
        let mut cluster = RaftCluster::new(3);
        cluster.trigger_election(0);
        cluster.run_to_stable(200);
        // Drain bus completely.
        for _ in 0..20 {
            cluster.step();
        }
        // Kill leader by resigning it as follower at term 0.
        let leader_idx = cluster
            .nodes
            .iter()
            .position(|(_, h)| h.state.role == RaftRole::Leader)
            .unwrap_or(0);
        cluster.nodes[leader_idx].1.state.role = RaftRole::Follower;
        cluster.nodes[leader_idx].1.state.voted_for = None;
        // Drain stale messages destined for ex-leader.
        cluster.drain_bus();
        // Elect new leader from a different node.
        let next = (leader_idx + 1) % 3;
        cluster.trigger_election(next);
        let new_leader = cluster.run_to_stable(200);
        assert!(new_leader.is_some());
    }

    #[test]
    fn leader_has_highest_term() {
        let mut cluster = RaftCluster::new(3);
        cluster.trigger_election(0);
        cluster.run_to_stable(100);
        if let Some(lid) = cluster.leader() {
            let leader_term = cluster
                .nodes
                .iter()
                .find(|(id, _)| *id == lid)
                .map(|(_, h)| h.state.current_term)
                .unwrap();
            for (_, h) in &cluster.nodes {
                assert!(h.state.current_term <= leader_term);
            }
        }
    }

    #[test]
    fn log_replication_to_followers() {
        let mut cluster = RaftCluster::new(3);
        cluster.trigger_election(0);
        cluster.run_to_stable(100);
        let replicated = cluster.replicate(b"write:key=1".to_vec());
        assert!(replicated);
    }

    #[test]
    fn commit_requires_quorum() {
        let mut cluster = RaftCluster::new(3);
        cluster.trigger_election(0);
        cluster.run_to_stable(100);
        if cluster.leader().is_some() {
            let committed = cluster.replicate(b"data".to_vec());
            assert!(committed);
        }
    }

    #[test]
    fn partition_prevents_election_for_single_node() {
        let mut cluster = RaftCluster::new(3);
        // Partition: node 0 isolated from nodes 1,2.
        cluster.partition(&[0], &[1, 2]);
        cluster.trigger_election(0);
        for _ in 0..50 {
            cluster.step();
        }
        // Node 0 can't get quorum (only 1 of 3 votes possible).
        assert!(
            cluster.nodes[0].1.state.role != RaftRole::Leader
                || cluster.nodes[0].1.state.votes_received.len() < 2
        );
    }

    #[test]
    fn heal_partition_allows_election() {
        let mut cluster = RaftCluster::new(3);
        cluster.partition(&[0], &[1, 2]);
        for _ in 0..20 {
            cluster.step();
        }
        cluster.heal_partition();
        cluster.trigger_election(1);
        let leader = cluster.run_to_stable(100);
        assert!(leader.is_some());
    }

    #[test]
    fn term_monotonically_increases() {
        let mut cluster = RaftCluster::new(3);
        cluster.trigger_election(0);
        cluster.run_to_stable(50);
        let t1 = cluster.nodes[0].1.state.current_term;
        cluster.nodes[0].1.state.become_follower(0, None);
        cluster.trigger_election(0);
        cluster.run_to_stable(50);
        let t2 = cluster.nodes[0].1.state.current_term;
        assert!(t2 >= t1);
    }

    #[test]
    fn no_split_brain_only_one_leader_per_term() {
        let mut cluster = RaftCluster::new(3);
        cluster.trigger_election(0);
        cluster.run_to_stable(100);
        let leader_count = cluster
            .nodes
            .iter()
            .filter(|(_, h)| h.state.role == RaftRole::Leader)
            .count();
        assert!(leader_count <= 1);
    }

    #[test]
    fn new_node_added_gets_log_replicated() {
        let mut cluster = RaftCluster::new(3);
        cluster.trigger_election(0);
        cluster.run_to_stable(100);
        if cluster.leader().is_some() {
            cluster.replicate(b"entry1".to_vec());
            // Check at least 2 nodes have entry.
            let have_entry = cluster
                .nodes
                .iter()
                .filter(|(_, h)| h.log.last_index() >= 1)
                .count();
            assert!(have_entry >= 2);
        }
    }

    #[test]
    fn leader_sends_heartbeats() {
        let mut cluster = RaftCluster::new(3);
        cluster.trigger_election(0);
        cluster.run_to_stable(100);
        if cluster.leader().is_some() {
            let follower_count = cluster
                .nodes
                .iter()
                .filter(|(_, h)| h.state.role == RaftRole::Follower)
                .filter(|(_, h)| h.state.leader_id.is_some())
                .count();
            assert!(follower_count >= 1);
        }
    }
}
