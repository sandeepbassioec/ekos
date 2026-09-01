use crate::node::NodeMeta;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("write_quorum ({write_quorum}) > replication_factor ({replication_factor})")]
    WriteQuorumExceedsReplication {
        write_quorum: usize,
        replication_factor: usize,
    },
    #[error("write_quorum must be >= 1")]
    WriteQuorumZero,
    #[error("read_quorum must be >= 1")]
    ReadQuorumZero,
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterConfig {
    pub cluster_name: String,
    pub nodes: Vec<NodeMeta>,
    pub replication_factor: usize,
    pub write_quorum: usize,
    pub read_quorum: usize,
    pub vnode_count: usize,
    pub heartbeat_interval_ms: u64,
    pub election_timeout_ms: u64,
    pub failover_timeout_ms: u64,
}

impl ClusterConfig {
    pub fn new(name: &str) -> Self {
        ClusterConfig {
            cluster_name: name.to_string(),
            nodes: Vec::new(),
            replication_factor: 3,
            write_quorum: 2,
            read_quorum: 1,
            vnode_count: 256,
            heartbeat_interval_ms: 150,
            election_timeout_ms: 300,
            failover_timeout_ms: 2000,
        }
    }

    pub fn add_node(&mut self, meta: NodeMeta) -> &mut Self {
        self.nodes.push(meta);
        self
    }

    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.write_quorum == 0 {
            return Err(ConfigError::WriteQuorumZero);
        }
        if self.read_quorum == 0 {
            return Err(ConfigError::ReadQuorumZero);
        }
        if self.write_quorum > self.replication_factor {
            return Err(ConfigError::WriteQuorumExceedsReplication {
                write_quorum: self.write_quorum,
                replication_factor: self.replication_factor,
            });
        }
        Ok(())
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap()
    }

    pub fn from_json(s: &str) -> Result<Self, ConfigError> {
        Ok(serde_json::from_str(s)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::node::{NodeId, NodeState};
    use std::net::SocketAddr;

    fn make_meta() -> NodeMeta {
        NodeMeta {
            id: NodeId::new(),
            addr: "127.0.0.1:9000".parse::<SocketAddr>().unwrap(),
            state: NodeState::Alive,
            incarnation: 0,
            last_seen_ns: 0,
        }
    }

    #[test]
    fn default_config_valid() {
        let cfg = ClusterConfig::new("test");
        assert!(cfg.validate().is_ok());
    }

    #[test]
    fn add_node_adds_to_list() {
        let mut cfg = ClusterConfig::new("test");
        cfg.add_node(make_meta());
        assert_eq!(cfg.nodes.len(), 1);
    }

    #[test]
    fn validate_replication_factor_ge_write_quorum() {
        let mut cfg = ClusterConfig::new("test");
        cfg.replication_factor = 3;
        cfg.write_quorum = 2;
        assert!(cfg.validate().is_ok());
    }

    #[test]
    fn validate_fails_write_quorum_zero() {
        let mut cfg = ClusterConfig::new("test");
        cfg.write_quorum = 0;
        assert!(matches!(cfg.validate(), Err(ConfigError::WriteQuorumZero)));
    }

    #[test]
    fn validate_fails_read_quorum_zero() {
        let mut cfg = ClusterConfig::new("test");
        cfg.read_quorum = 0;
        assert!(matches!(cfg.validate(), Err(ConfigError::ReadQuorumZero)));
    }

    #[test]
    fn to_json_from_json_roundtrip() {
        let cfg = ClusterConfig::new("mycluster");
        let json = cfg.to_json();
        let decoded = ClusterConfig::from_json(&json).unwrap();
        assert_eq!(decoded.cluster_name, "mycluster");
    }

    #[test]
    fn failover_timeout_2000ms_default() {
        let cfg = ClusterConfig::new("test");
        assert_eq!(cfg.failover_timeout_ms, 2000);
    }

    #[test]
    fn heartbeat_less_than_election_timeout() {
        let cfg = ClusterConfig::new("test");
        assert!(cfg.heartbeat_interval_ms < cfg.election_timeout_ms);
    }

    #[test]
    fn cluster_name_preserved() {
        let cfg = ClusterConfig::new("vectorvolt-prod");
        assert_eq!(cfg.cluster_name, "vectorvolt-prod");
    }

    #[test]
    fn vnode_count_256_default() {
        let cfg = ClusterConfig::new("test");
        assert_eq!(cfg.vnode_count, 256);
    }

    #[test]
    fn replication_factor_3_default() {
        let cfg = ClusterConfig::new("test");
        assert_eq!(cfg.replication_factor, 3);
    }

    #[test]
    fn add_3_nodes_validate_ok() {
        let mut cfg = ClusterConfig::new("test");
        cfg.add_node(make_meta());
        cfg.add_node(make_meta());
        cfg.add_node(make_meta());
        assert!(cfg.validate().is_ok());
    }

    #[test]
    fn serialize_preserves_all_fields() {
        let mut cfg = ClusterConfig::new("vv");
        cfg.replication_factor = 5;
        cfg.write_quorum = 3;
        cfg.read_quorum = 2;
        cfg.vnode_count = 512;
        cfg.heartbeat_interval_ms = 100;
        cfg.election_timeout_ms = 500;
        cfg.failover_timeout_ms = 1500;
        let json = cfg.to_json();
        let decoded = ClusterConfig::from_json(&json).unwrap();
        assert_eq!(decoded.replication_factor, 5);
        assert_eq!(decoded.write_quorum, 3);
        assert_eq!(decoded.read_quorum, 2);
        assert_eq!(decoded.vnode_count, 512);
        assert_eq!(decoded.heartbeat_interval_ms, 100);
        assert_eq!(decoded.election_timeout_ms, 500);
        assert_eq!(decoded.failover_timeout_ms, 1500);
    }
}
