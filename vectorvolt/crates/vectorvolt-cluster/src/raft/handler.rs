use super::log::{LogEntry, RaftLog};
use super::state::RaftState;
use crate::node::NodeId;

pub struct RaftHandler {
    pub state: RaftState,
    pub log: RaftLog,
}

impl RaftHandler {
    pub fn new(node_id: NodeId) -> Self {
        RaftHandler {
            state: RaftState::new(node_id),
            log: RaftLog::new(),
        }
    }

    pub fn handle_request_vote(
        &mut self,
        term: u64,
        candidate_id: NodeId,
        last_log_index: u64,
        last_log_term: u64,
    ) -> (u64, bool) {
        if term < self.state.current_term {
            return (self.state.current_term, false);
        }
        if term > self.state.current_term {
            self.state.become_follower(term, None);
        }
        // Log up-to-date check.
        let my_last_term = self.log.last_term();
        let my_last_index = self.log.last_index();
        let log_ok = last_log_term > my_last_term
            || (last_log_term == my_last_term && last_log_index >= my_last_index);
        if log_ok && self.state.grant_vote(candidate_id, term) {
            self.state.reset_election_timer();
            (self.state.current_term, true)
        } else {
            (self.state.current_term, false)
        }
    }

    pub fn handle_append_entries(
        &mut self,
        term: u64,
        leader_id: NodeId,
        prev_log_index: u64,
        prev_log_term: u64,
        entries: Vec<LogEntry>,
        leader_commit: u64,
    ) -> (u64, bool, u64) {
        if term < self.state.current_term {
            return (self.state.current_term, false, 0);
        }
        self.state.become_follower(term, Some(leader_id));

        // Log consistency check.
        if prev_log_index > 0 && !self.log.contains(prev_log_index, prev_log_term) {
            return (self.state.current_term, false, 0);
        }

        // Append new entries.
        for (insert_idx, entry) in (prev_log_index + 1..).zip(entries) {
            if let Some(existing) = self.log.get(insert_idx) {
                if existing.term != entry.term {
                    self.log.truncate_from(insert_idx);
                    self.log.append(entry.term, entry.command);
                }
                // If same term, already have it.
            } else {
                self.log.append(entry.term, entry.command);
            }
        }

        // Advance commit index.
        if leader_commit > self.log.commit_index() {
            self.log.commit_up_to(leader_commit);
        }

        let match_index = self.log.last_index();
        (self.state.current_term, true, match_index)
    }

    pub fn handle_install_snapshot(
        &mut self,
        term: u64,
        _leader_id: NodeId,
        _last_included_index: u64,
        _last_included_term: u64,
        _data: Vec<u8>,
    ) -> (u64, bool) {
        if term < self.state.current_term {
            return (self.state.current_term, false);
        }
        self.state.become_follower(term, None);
        (self.state.current_term, true)
    }
}

#[cfg(test)]
mod tests {
    use super::super::messages::RaftMessage;
    use super::super::state::RaftRole;
    use super::*;

    fn handler() -> RaftHandler {
        RaftHandler::new(NodeId::new())
    }

    #[test]
    fn request_vote_granted_when_log_up_to_date() {
        let mut h = handler();
        let candidate = NodeId::new();
        let (_, granted) = h.handle_request_vote(1, candidate, 0, 0);
        assert!(granted);
    }

    #[test]
    fn request_vote_denied_when_stale_term() {
        let mut h = handler();
        h.state.current_term = 5;
        let (_, granted) = h.handle_request_vote(3, NodeId::new(), 0, 0);
        assert!(!granted);
    }

    #[test]
    fn request_vote_denied_when_already_voted() {
        let mut h = handler();
        let c1 = NodeId::new();
        let c2 = NodeId::new();
        h.handle_request_vote(1, c1, 0, 0);
        let (_, granted) = h.handle_request_vote(1, c2, 0, 0);
        assert!(!granted);
    }

    #[test]
    fn append_entries_success_on_empty_log() {
        let mut h = handler();
        let leader = NodeId::new();
        let (_, ok, _) = h.handle_append_entries(1, leader, 0, 0, vec![], 0);
        assert!(ok);
    }

    #[test]
    fn append_entries_fails_on_prev_log_mismatch() {
        let mut h = handler();
        let leader = NodeId::new();
        // prev_log_index=5 doesn't exist.
        let (_, ok, _) = h.handle_append_entries(1, leader, 5, 1, vec![], 0);
        assert!(!ok);
    }

    #[test]
    fn append_entries_appends_entries() {
        let mut h = handler();
        let leader = NodeId::new();
        let entries = vec![LogEntry {
            term: 1,
            index: 1,
            command: b"cmd1".to_vec(),
        }];
        h.handle_append_entries(1, leader, 0, 0, entries, 0);
        assert_eq!(h.log.last_index(), 1);
    }

    #[test]
    fn append_entries_advances_commit_index() {
        let mut h = handler();
        let leader = NodeId::new();
        let entries = vec![
            LogEntry {
                term: 1,
                index: 1,
                command: vec![],
            },
            LogEntry {
                term: 1,
                index: 2,
                command: vec![],
            },
        ];
        h.handle_append_entries(1, leader, 0, 0, entries, 2);
        assert_eq!(h.log.commit_index(), 2);
    }

    #[test]
    fn append_entries_higher_term_converts_to_follower() {
        let mut h = handler();
        h.state.become_candidate();
        assert_eq!(h.state.role, RaftRole::Candidate);
        let leader = NodeId::new();
        h.handle_append_entries(5, leader, 0, 0, vec![], 0);
        assert_eq!(h.state.role, RaftRole::Follower);
        assert_eq!(h.state.current_term, 5);
    }

    #[test]
    fn install_snapshot_accepted() {
        let mut h = handler();
        let leader = NodeId::new();
        let (_, ok) = h.handle_install_snapshot(1, leader, 5, 1, vec![1, 2, 3]);
        assert!(ok);
    }

    #[test]
    fn install_snapshot_denied_lower_term() {
        let mut h = handler();
        h.state.current_term = 5;
        let (_, ok) = h.handle_install_snapshot(3, NodeId::new(), 1, 1, vec![]);
        assert!(!ok);
    }

    #[test]
    fn raft_message_serializes_deserializes() {
        let msg = RaftMessage::RequestVote {
            term: 1,
            candidate_id: NodeId::new(),
            last_log_index: 0,
            last_log_term: 0,
        };
        let json = serde_json::to_string(&msg).unwrap();
        let _decoded: RaftMessage = serde_json::from_str(&json).unwrap();
    }

    #[test]
    fn heartbeat_append_entries_empty_entries_ok() {
        let mut h = handler();
        let leader = NodeId::new();
        let (_, ok, _) = h.handle_append_entries(1, leader, 0, 0, vec![], 0);
        assert!(ok);
    }

    #[test]
    fn two_append_entries_in_sequence_correct_indices() {
        let mut h = handler();
        let leader = NodeId::new();
        let e1 = vec![LogEntry {
            term: 1,
            index: 1,
            command: b"a".to_vec(),
        }];
        h.handle_append_entries(1, leader.clone(), 0, 0, e1, 0);
        let e2 = vec![LogEntry {
            term: 1,
            index: 2,
            command: b"b".to_vec(),
        }];
        h.handle_append_entries(1, leader, 1, 1, e2, 0);
        assert_eq!(h.log.last_index(), 2);
        assert_eq!(h.log.get(2).unwrap().command, b"b");
    }
}
