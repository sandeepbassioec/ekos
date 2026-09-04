//! `vectorvolt-chaos` — fault injection and deterministic simulation testing.

pub mod dst;
pub mod fault;
pub mod scenario;

pub use dst::{DstEvent, DstEventKind, DstHarness};
pub use fault::{Fault, FaultInjector};
pub use scenario::{ChaosSchedule, Scenario};
