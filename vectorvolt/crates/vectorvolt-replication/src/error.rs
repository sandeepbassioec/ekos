//! Errors for networked replication.

use std::time::Duration;

use thiserror::Error;

/// Result alias for replication operations.
pub type ReplResult<T> = Result<T, ReplicationError>;

#[derive(Debug, Error)]
pub enum ReplicationError {
    #[error("replication I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// A message failed its CRC check, or its framing was malformed.
    #[error("replication frame rejected: {0}")]
    Corrupt(String),

    #[error("replication protocol error: {0}")]
    Protocol(String),

    /// A synchronous write was not acknowledged by enough followers in time.
    /// The write itself is already committed on the leader; the caller must
    /// treat its outcome as unconfirmed rather than as a success.
    #[error(
        "replication ack timeout: lsn {lsn} acknowledged by {got} of {needed} required \
         follower(s) within {timeout:?}"
    )]
    AckTimeout {
        lsn: u64,
        needed: usize,
        got: usize,
        timeout: Duration,
    },

    /// A write was attempted on a node that is a replication follower.
    #[error("node is a read-only replication follower")]
    ReadOnlyFollower,

    #[error("node is not the replication leader")]
    NotLeader,

    #[error("WAL error: {0}")]
    Wal(String),

    #[error("WAL frame decrypt error: {0}")]
    Decrypt(String),

    #[error("engine error: {0}")]
    Engine(String),

    #[error("invalid replication config: {0}")]
    Config(String),

    #[error("replication is shutting down")]
    Shutdown,
}

impl From<vectorvolt_wal::WalError> for ReplicationError {
    fn from(e: vectorvolt_wal::WalError) -> Self {
        ReplicationError::Wal(e.to_string())
    }
}

impl From<vectorvolt_engine::EngineError> for ReplicationError {
    fn from(e: vectorvolt_engine::EngineError) -> Self {
        ReplicationError::Engine(e.to_string())
    }
}
