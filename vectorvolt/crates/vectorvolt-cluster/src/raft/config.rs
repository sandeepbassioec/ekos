//! Cluster membership configuration carried in the Raft log (dissertation 4.1).
//!
//! A configuration change is an ordinary log entry whose command starts with
//! [`CONFIG_PREFIX`] followed by the JSON-encoded new [`ClusterConfig`]. It takes
//! effect on the node that appends it as soon as it is in that node's log (not
//! when committed), exactly one change may be uncommitted at a time, and the
//! state machine never sees these entries.

use crate::node::NodeId;
use serde::{Deserialize, Serialize};

/// Reserved first bytes of a membership-change command. Client proposals that
/// start with it are rejected.
pub const CONFIG_PREFIX: &[u8] = b"\xFFVVCFG\x01";

/// Voting members plus non-voting learners (catching up before promotion).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClusterConfig {
    pub voters: Vec<NodeId>,
    pub learners: Vec<NodeId>,
}

pub fn is_config_command(cmd: &[u8]) -> bool {
    cmd.starts_with(CONFIG_PREFIX)
}

impl ClusterConfig {
    pub fn new(voters: Vec<NodeId>, learners: Vec<NodeId>) -> Self {
        let mut c = ClusterConfig { voters, learners };
        c.normalize();
        c
    }

    /// Sort and de-duplicate (a node is never both voter and learner).
    pub fn normalize(&mut self) {
        self.voters.sort_by_key(|n| n.0);
        self.voters.dedup();
        self.learners.sort_by_key(|n| n.0);
        self.learners.dedup();
        let v = self.voters.clone();
        self.learners.retain(|n| !v.contains(n));
    }

    pub fn is_voter(&self, id: &NodeId) -> bool {
        self.voters.contains(id)
    }

    pub fn is_learner(&self, id: &NodeId) -> bool {
        self.learners.contains(id)
    }

    pub fn contains(&self, id: &NodeId) -> bool {
        self.is_voter(id) || self.is_learner(id)
    }

    /// Log-entry command for this configuration.
    pub fn encode_entry(&self) -> Vec<u8> {
        let mut out = CONFIG_PREFIX.to_vec();
        out.extend(serde_json::to_vec(self).expect("config serializes"));
        out
    }

    /// Parse a log command; `None` if it is not a (valid) config entry.
    pub fn decode_entry(cmd: &[u8]) -> Option<ClusterConfig> {
        let body = cmd.strip_prefix(CONFIG_PREFIX)?;
        let mut c: ClusterConfig = serde_json::from_slice(body).ok()?;
        c.normalize();
        Some(c)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn entry_roundtrip_and_prefix() {
        let c = ClusterConfig::new(vec![NodeId::new(), NodeId::new()], vec![NodeId::new()]);
        let e = c.encode_entry();
        assert!(is_config_command(&e));
        assert_eq!(ClusterConfig::decode_entry(&e), Some(c));
        assert!(!is_config_command(b"hello"));
        assert!(ClusterConfig::decode_entry(b"hello").is_none());
    }
}
