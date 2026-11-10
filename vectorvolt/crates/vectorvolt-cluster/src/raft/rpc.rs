//! Wire-level Raft RPC messages used by [`super::node::RaftNode`] and the
//! cluster transport. The sender id travels in the transport envelope.

use super::log::LogEntry;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Rpc {
    RequestVote {
        term: u64,
        last_log_index: u64,
        last_log_term: u64,
    },
    RequestVoteReply {
        term: u64,
        granted: bool,
    },
    AppendEntries {
        term: u64,
        prev_log_index: u64,
        prev_log_term: u64,
        entries: Vec<LogEntry>,
        leader_commit: u64,
    },
    AppendEntriesReply {
        term: u64,
        success: bool,
        /// Highest index known to match the leader (valid when `success`).
        match_index: u64,
        /// Hint for the leader's next probe (valid when `!success`).
        conflict_index: u64,
    },
    InstallSnapshot {
        term: u64,
        last_included_index: u64,
        last_included_term: u64,
        data: Vec<u8>,
    },
    InstallSnapshotReply {
        term: u64,
        last_included_index: u64,
    },
}

impl Rpc {
    pub fn term(&self) -> u64 {
        match self {
            Rpc::RequestVote { term, .. }
            | Rpc::RequestVoteReply { term, .. }
            | Rpc::AppendEntries { term, .. }
            | Rpc::AppendEntriesReply { term, .. }
            | Rpc::InstallSnapshot { term, .. }
            | Rpc::InstallSnapshotReply { term, .. } => *term,
        }
    }
}
