//! Wire-level Raft RPC messages used by [`super::node::RaftNode`] and the
//! cluster transport. The sender id travels in the transport envelope.

use super::config::ClusterConfig;
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
    /// One chunk of a snapshot (Raft fig. 13): `data` covers bytes
    /// `offset..offset+data.len()` of the snapshot; `done` marks the last chunk.
    InstallSnapshot {
        term: u64,
        last_included_index: u64,
        last_included_term: u64,
        offset: u64,
        data: Vec<u8>,
        done: bool,
        /// Membership as of `last_included_index`.
        config: Option<ClusterConfig>,
    },
    InstallSnapshotReply {
        term: u64,
        /// Non-zero once the snapshot (or newer state) is fully installed.
        last_included_index: u64,
        /// Next byte the follower expects (resume point) while still receiving.
        next_offset: u64,
    },
    /// Pre-vote probe (dissertation 9.6): `term` is the term the sender
    /// *would* use; receivers do not adopt it or change any state.
    PreVote {
        term: u64,
        last_log_index: u64,
        last_log_term: u64,
    },
    PreVoteReply {
        /// Responder's current term.
        term: u64,
        proposed_term: u64,
        granted: bool,
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
            | Rpc::InstallSnapshotReply { term, .. }
            | Rpc::PreVote { term, .. }
            | Rpc::PreVoteReply { term, .. } => *term,
        }
    }
}
