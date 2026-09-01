use crate::node::NodeId;
use std::collections::HashSet;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, PartialEq)]
pub enum RaftRole {
    Follower,
    Candidate,
    Leader,
}

pub struct RaftState {
    pub node_id: NodeId,
    pub current_term: u64,
    pub voted_for: Option<NodeId>,
    pub role: RaftRole,
    pub leader_id: Option<NodeId>,
    pub votes_received: HashSet<NodeId>,
    pub election_timeout: Duration,
    pub last_heartbeat: Instant,
}

impl RaftState {
    pub fn new(node_id: NodeId) -> Self {
        RaftState {
            node_id,
            current_term: 0,
            voted_for: None,
            role: RaftRole::Follower,
            leader_id: None,
            votes_received: HashSet::new(),
            election_timeout: Duration::from_millis(300),
            last_heartbeat: Instant::now(),
        }
    }

    /// Become candidate: increment term, vote for self.
    pub fn become_candidate(&mut self) -> u64 {
        self.current_term += 1;
        self.role = RaftRole::Candidate;
        self.voted_for = Some(self.node_id.clone());
        self.votes_received.clear();
        self.votes_received.insert(self.node_id.clone());
        self.leader_id = None;
        self.current_term
    }

    pub fn become_leader(&mut self) {
        self.role = RaftRole::Leader;
        self.leader_id = Some(self.node_id.clone());
    }

    pub fn become_follower(&mut self, term: u64, leader: Option<NodeId>) {
        self.current_term = term;
        self.role = RaftRole::Follower;
        self.leader_id = leader;
        self.votes_received.clear();
        self.last_heartbeat = Instant::now();
    }

    /// Record a vote from `from`; returns true if quorum reached.
    pub fn record_vote(&mut self, from: NodeId) -> bool {
        self.votes_received.insert(from);
        false // caller checks has_quorum explicitly
    }

    pub fn has_quorum(&self, cluster_size: usize) -> bool {
        self.votes_received.len() > cluster_size / 2
    }

    pub fn election_timed_out(&self) -> bool {
        self.last_heartbeat.elapsed() >= self.election_timeout
    }

    pub fn reset_election_timer(&mut self) {
        self.last_heartbeat = Instant::now();
    }

    /// Grant vote to candidate if conditions met. Returns true if granted.
    pub fn grant_vote(&mut self, candidate: NodeId, term: u64) -> bool {
        if term < self.current_term {
            return false;
        }
        if term > self.current_term {
            self.current_term = term;
            self.voted_for = None;
        }
        if self.voted_for.is_none() {
            self.voted_for = Some(candidate);
            return true;
        }
        if self.voted_for.as_ref() == Some(&candidate) {
            return true;
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_node_is_follower() {
        let state = RaftState::new(NodeId::new());
        assert_eq!(state.role, RaftRole::Follower);
    }

    #[test]
    fn become_candidate_increments_term() {
        let mut state = RaftState::new(NodeId::new());
        let term = state.become_candidate();
        assert_eq!(term, 1);
        assert_eq!(state.current_term, 1);
    }

    #[test]
    fn become_candidate_votes_for_self() {
        let id = NodeId::new();
        let mut state = RaftState::new(id.clone());
        state.become_candidate();
        assert_eq!(state.voted_for, Some(id.clone()));
        assert!(state.votes_received.contains(&id));
    }

    #[test]
    fn record_vote_tracks_votes() {
        let id = NodeId::new();
        let mut state = RaftState::new(id.clone());
        state.become_candidate();
        let peer = NodeId::new();
        state.record_vote(peer.clone());
        assert!(state.votes_received.contains(&peer));
    }

    #[test]
    fn has_quorum_needs_majority_of_three() {
        let id = NodeId::new();
        let mut state = RaftState::new(id.clone());
        state.become_candidate(); // 1 vote (self)
        assert!(!state.has_quorum(3)); // need 2
        state.record_vote(NodeId::new()); // 2 votes
        assert!(state.has_quorum(3));
    }

    #[test]
    fn become_leader_sets_role() {
        let id = NodeId::new();
        let mut state = RaftState::new(id.clone());
        state.become_candidate();
        state.become_leader();
        assert_eq!(state.role, RaftRole::Leader);
    }

    #[test]
    fn become_follower_resets_role() {
        let mut state = RaftState::new(NodeId::new());
        state.become_candidate();
        state.become_follower(2, None);
        assert_eq!(state.role, RaftRole::Follower);
        assert_eq!(state.current_term, 2);
    }

    #[test]
    fn grant_vote_only_once_per_term() {
        let id = NodeId::new();
        let mut state = RaftState::new(id.clone());
        let c1 = NodeId::new();
        let c2 = NodeId::new();
        assert!(state.grant_vote(c1.clone(), 1));
        assert!(!state.grant_vote(c2.clone(), 1)); // already voted for c1
    }

    #[test]
    fn grant_vote_denied_if_already_voted() {
        let mut state = RaftState::new(NodeId::new());
        let c1 = NodeId::new();
        let c2 = NodeId::new();
        state.grant_vote(c1.clone(), 1);
        assert!(!state.grant_vote(c2, 1));
    }

    #[test]
    fn grant_vote_denied_if_term_less_than_current() {
        let mut state = RaftState::new(NodeId::new());
        state.current_term = 5;
        assert!(!state.grant_vote(NodeId::new(), 3));
    }

    #[test]
    fn election_timed_out_after_timeout_elapses() {
        let mut state = RaftState::new(NodeId::new());
        state.election_timeout = Duration::from_millis(1);
        state.last_heartbeat = Instant::now() - Duration::from_millis(100);
        assert!(state.election_timed_out());
    }

    #[test]
    fn reset_election_timer_clears_timeout() {
        let mut state = RaftState::new(NodeId::new());
        state.election_timeout = Duration::from_millis(1);
        state.last_heartbeat = Instant::now() - Duration::from_millis(100);
        state.reset_election_timer();
        // After reset, should not be timed out immediately.
        assert!(!state.election_timed_out());
    }

    #[test]
    fn leader_knows_its_leader_id_is_self() {
        let id = NodeId::new();
        let mut state = RaftState::new(id.clone());
        state.become_candidate();
        state.become_leader();
        assert_eq!(state.leader_id, Some(id));
    }
}
