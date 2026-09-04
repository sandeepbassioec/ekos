//! [`ChaosSchedule`] — schedule named scenarios at specific operation counts.

/// A high-level scenario that can be injected into a running cluster.
#[derive(Debug, Clone)]
pub enum Scenario {
    /// Partition the network for `duration_ms` milliseconds.
    NetworkPartition { duration_ms: u64 },
    /// Crash the current Raft leader.
    LeaderCrash,
    /// Introduce a `delay_ms`-millisecond extra latency on `node_id`.
    SlowFollower { node_id: u64, delay_ms: u64 },
    /// Fill the disk on `node_id`.
    DiskFull { node_id: u64 },
    /// Skew the clock of `node_id` by `skew_ms` milliseconds (may be negative).
    ClockSkew { node_id: u64, skew_ms: i64 },
}

/// A ordered list of `(at_op, Scenario)` pairs.
///
/// Use [`ChaosSchedule::at`] to build the schedule, then call
/// [`ChaosSchedule::scenario_at`] to query what happens at a given operation
/// counter.
pub struct ChaosSchedule {
    scenarios: Vec<(u64, Scenario)>,
}

impl ChaosSchedule {
    /// Create an empty schedule.
    pub fn new() -> Self {
        Self { scenarios: Vec::new() }
    }

    /// Append a scenario that fires when the operation counter reaches `op`.
    pub fn at(mut self, op: u64, scenario: Scenario) -> Self {
        self.scenarios.push((op, scenario));
        self
    }

    /// Return the first scenario scheduled at exactly `op`, if any.
    pub fn scenario_at(&self, op: u64) -> Option<&Scenario> {
        self.scenarios.iter().find(|(o, _)| *o == op).map(|(_, s)| s)
    }
}

impl Default for ChaosSchedule {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chaos_schedule_at_op_10() {
        let sched = ChaosSchedule::new().at(10, Scenario::LeaderCrash);
        assert!(sched.scenario_at(10).is_some());
    }

    #[test]
    fn scenario_at_10_returns_scenario() {
        let sched = ChaosSchedule::new().at(10, Scenario::NetworkPartition { duration_ms: 100 });
        let s = sched.scenario_at(10).unwrap();
        matches!(s, Scenario::NetworkPartition { .. });
    }

    #[test]
    fn scenario_at_9_returns_none() {
        let sched = ChaosSchedule::new().at(10, Scenario::LeaderCrash);
        assert!(sched.scenario_at(9).is_none());
    }

    #[test]
    fn multiple_scenarios_in_schedule() {
        let sched = ChaosSchedule::new()
            .at(1, Scenario::LeaderCrash)
            .at(5, Scenario::DiskFull { node_id: 2 })
            .at(10, Scenario::ClockSkew { node_id: 1, skew_ms: -50 });
        assert!(sched.scenario_at(1).is_some());
        assert!(sched.scenario_at(5).is_some());
        assert!(sched.scenario_at(10).is_some());
        assert!(sched.scenario_at(3).is_none());
    }
}
