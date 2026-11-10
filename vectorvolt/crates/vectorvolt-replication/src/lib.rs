//! `vectorvolt-replication` — single-leader / multi-follower replication for VectorVolt.
//!
//! ## Model
//!
//! - The [`Leader`] accepts writes, applies them to its local [`TableEngine`],
//!   and broadcasts each mutation as a [`ReplicationFrame`] to every connected
//!   follower via a crossbeam unbounded channel.
//! - Each [`Follower`] holds its own [`TableEngine`] (the replica store).
//!   Frames are applied by calling [`Follower::apply_pending`] / [`Follower::catchup`].
//! - Replication is **asynchronous**: the leader never waits for followers to
//!   apply.  A follower may lag or be permanently behind (e.g. started after
//!   some writes) — log replay from before the connection point is not
//!   handled in this layer.
//! - Reads from a follower reflect only what has been applied so far.
//!
//! ## Networked WAL shipping
//!
//! [`ReplicationServer`] (leader) streams committed WAL frames over TCP to
//! [`ReplicationClient`]s (followers), CRC-framed, starting from each
//! follower's last applied LSN; a follower that is new, ahead, or older than
//! the leader's retained WAL receives a snapshot first. Followers ack applied
//! and durable LSNs; [`ReplicationMode`] decides whether a writer waits
//! (`wait_for_replication`), and a missing ack is an [`ReplicationError::AckTimeout`].
//! [`RoleCell`] holds the switchable [`ReplicationRole`] a later election step
//! can flip to promote a follower.
//!
//! ## Example (in-process)
//!
//! ```no_run
//! # use std::sync::Arc;
//! # use vectorvolt_engine::{TableEngine, EngineOptions};
//! # use vectorvolt_replication::Leader;
//! # let leader_dir = tempfile::TempDir::new().unwrap();
//! # let replica_dir = tempfile::TempDir::new().unwrap();
//! let leader_engine = Arc::new(
//!     TableEngine::open_with(vec!["pk".into(), "val".into()], leader_dir.path(),
//!         EngineOptions::new(1_048_576)).unwrap()
//! );
//! let replica_engine = Arc::new(
//!     TableEngine::open_with(vec!["pk".into(), "val".into()], replica_dir.path(),
//!         EngineOptions::new(1_048_576)).unwrap()
//! );
//!
//! let leader = Leader::new(leader_engine);
//! let follower = leader.connect_replica(replica_engine);
//!
//! leader.insert(1, vec![1, 42]).unwrap();
//! follower.catchup().unwrap();
//!
//! let row = follower.get(1).unwrap().unwrap();
//! assert_eq!(row.values[1], 42);
//! ```

mod client;
mod config;
mod error;
mod follower;
mod frame;
mod leader;
mod metrics;
mod proto;
mod server;
mod wal_tail;

pub use client::{load_position, save_position, ReplicationClient, POSITION_FILE};
pub use config::{ReplicationConfig, ReplicationMode, ReplicationRole, RoleCell};
pub use error::{ReplResult, ReplicationError};
pub use metrics::{MetricsSnapshot, ReplicationMetrics};
pub use proto::{Message, WireFrame};
pub use server::{FollowerInfo, ReplicationServer};

pub use follower::Follower;
pub use frame::{ReplicaMutation, ReplicationFrame};
pub use leader::Leader;
