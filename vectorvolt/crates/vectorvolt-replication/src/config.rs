//! Replication configuration: mode, factor and the switchable role.

use std::str::FromStr;
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::Arc;
use std::time::Duration;

use crate::error::{ReplResult, ReplicationError};

/// When a write is considered replicated.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReplicationMode {
    /// The leader never waits for followers.
    Async,
    /// Wait until one follower has applied the write.
    SemiSync,
    /// Wait until `replication_factor - 1` followers have durably applied it.
    Sync,
}

impl FromStr for ReplicationMode {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, String> {
        match s.to_ascii_lowercase().as_str() {
            "async" => Ok(Self::Async),
            "semi-sync" | "semisync" | "semi_sync" => Ok(Self::SemiSync),
            "sync" => Ok(Self::Sync),
            other => Err(format!(
                "unknown replication mode {other:?} (async, semi-sync, sync)"
            )),
        }
    }
}

impl std::fmt::Display for ReplicationMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Async => "async",
            Self::SemiSync => "semi-sync",
            Self::Sync => "sync",
        })
    }
}

/// The node's replication role. Held in a [`RoleCell`] so a later step
/// (election / failover) can promote a follower or demote a leader at runtime.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReplicationRole {
    Standalone = 0,
    Leader = 1,
    Follower = 2,
}

impl FromStr for ReplicationRole {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, String> {
        match s.to_ascii_lowercase().as_str() {
            "leader" => Ok(Self::Leader),
            "follower" => Ok(Self::Follower),
            "standalone" | "none" => Ok(Self::Standalone),
            other => Err(format!(
                "unknown replication role {other:?} (leader, follower)"
            )),
        }
    }
}

impl std::fmt::Display for ReplicationRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Standalone => "standalone",
            Self::Leader => "leader",
            Self::Follower => "follower",
        })
    }
}

/// Shared, atomically switchable [`ReplicationRole`].
#[derive(Debug, Clone)]
pub struct RoleCell(Arc<AtomicU8>);

impl RoleCell {
    pub fn new(role: ReplicationRole) -> Self {
        Self(Arc::new(AtomicU8::new(role as u8)))
    }

    pub fn get(&self) -> ReplicationRole {
        match self.0.load(Ordering::Acquire) {
            1 => ReplicationRole::Leader,
            2 => ReplicationRole::Follower,
            _ => ReplicationRole::Standalone,
        }
    }

    /// Switch the role (e.g. promote a follower to leader).
    pub fn set(&self, role: ReplicationRole) {
        self.0.store(role as u8, Ordering::Release);
    }

    /// `Err(ReadOnlyFollower)` while this node is a follower. Call before
    /// accepting any client write.
    pub fn ensure_writable(&self) -> ReplResult<()> {
        match self.get() {
            ReplicationRole::Follower => Err(ReplicationError::ReadOnlyFollower),
            _ => Ok(()),
        }
    }
}

/// Shared, monotonically advancing "highest primary epoch seen" cell. A
/// follower's replication client drops anything from a lower epoch.
#[derive(Debug, Clone, Default)]
pub struct EpochCell(Arc<std::sync::atomic::AtomicU64>);

impl EpochCell {
    pub fn new(epoch: u64) -> Self {
        Self(Arc::new(std::sync::atomic::AtomicU64::new(epoch)))
    }

    pub fn get(&self) -> u64 {
        self.0.load(Ordering::Acquire)
    }

    /// Raise the known epoch to at least `epoch`; returns the resulting value.
    pub fn advance(&self, epoch: u64) -> u64 {
        self.0.fetch_max(epoch, Ordering::AcqRel).max(epoch)
    }
}

/// Tunables shared by leader and follower.
#[derive(Debug, Clone)]
pub struct ReplicationConfig {
    pub mode: ReplicationMode,
    /// Total copies of the data including the leader (>= 1).
    pub replication_factor: usize,
    /// How long a `Sync`/`SemiSync` writer waits for acks before erroring.
    pub ack_timeout: Duration,
    /// Leader: how often an idle stream checks the WAL for new frames.
    pub poll_interval: Duration,
    /// Leader: idle heartbeat period.
    pub heartbeat_interval: Duration,
    /// Follower: initial reconnect delay; doubles up to `backoff_max`.
    pub backoff_initial: Duration,
    pub backoff_max: Duration,
    /// Follower: drop the connection after this long without a message.
    pub read_timeout: Duration,
    /// Max frames per `Frames` message.
    pub batch_frames: usize,
    /// Max rows per snapshot message.
    pub snapshot_batch_rows: usize,
    /// Leader: when fewer in-sync followers are alive than the mode needs,
    /// accept writes anyway with fewer acks (explicitly: a `degraded` gauge, a
    /// `degraded_writes` counter and a log line) instead of rejecting them with
    /// "insufficient replicas". The configured factor itself never changes;
    /// full acknowledgement resumes as soon as enough followers are caught up.
    pub degraded_ok: bool,
}

impl Default for ReplicationConfig {
    fn default() -> Self {
        Self {
            mode: ReplicationMode::Async,
            replication_factor: 2,
            ack_timeout: Duration::from_secs(5),
            poll_interval: Duration::from_millis(10),
            heartbeat_interval: Duration::from_millis(500),
            backoff_initial: Duration::from_millis(100),
            backoff_max: Duration::from_secs(5),
            read_timeout: Duration::from_secs(10),
            batch_frames: 256,
            snapshot_batch_rows: 1024,
            degraded_ok: false,
        }
    }
}

impl ReplicationConfig {
    pub fn with_mode(mut self, mode: ReplicationMode) -> Self {
        self.mode = mode;
        self
    }

    pub fn with_replication_factor(mut self, n: usize) -> Self {
        self.replication_factor = n;
        self
    }

    pub fn with_degraded_ok(mut self, yes: bool) -> Self {
        self.degraded_ok = yes;
        self
    }

    pub fn with_ack_timeout(mut self, d: Duration) -> Self {
        self.ack_timeout = d;
        self
    }

    pub fn validate(&self) -> ReplResult<()> {
        if self.replication_factor == 0 {
            return Err(ReplicationError::Config(
                "replication factor must be >= 1".into(),
            ));
        }
        if self.batch_frames == 0 || self.snapshot_batch_rows == 0 {
            return Err(ReplicationError::Config("batch sizes must be > 0".into()));
        }
        Ok(())
    }

    /// Follower acks a write needs before it is reported replicated.
    pub fn required_acks(&self) -> usize {
        match self.mode {
            ReplicationMode::Async => 0,
            ReplicationMode::SemiSync => 1.min(self.replication_factor.saturating_sub(1)),
            ReplicationMode::Sync => self.replication_factor.saturating_sub(1),
        }
    }
}
