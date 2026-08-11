//! [`ReplicationFrame`] — the unit of replication shipped from leader to follower.

/// A single mutation event broadcast from the leader.
#[derive(Debug, Clone)]
pub struct ReplicationFrame {
    /// Monotonically increasing sequence number assigned by the leader.
    pub seq: u64,
    /// The mutation to apply on the follower.
    pub mutation: ReplicaMutation,
}

/// A mutation that can be replicated.
#[derive(Debug, Clone)]
pub enum ReplicaMutation {
    Insert { pk: u64, values: Vec<u64> },
    Delete { pk: u64 },
}
