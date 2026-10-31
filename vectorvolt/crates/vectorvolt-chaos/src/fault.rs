//! [`FaultInjector`] — inject delays, packet loss, corruption, and crash faults.

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::Duration;

/// A single fault specification.
#[derive(Debug, Clone)]
pub enum Fault {
    /// Delay every operation by a random duration in `[min_ms, max_ms]`.
    Delay { min_ms: u64, max_ms: u64 },
    /// Drop a packet with the given probability (0.0 = never, 1.0 = always).
    PacketLoss { probability: f64 },
    /// Corrupt a message payload with the given probability.
    Corrupt { probability: f64 },
    /// Trigger a crash after exactly `after_n_ops` operations have been ticked.
    Crash { after_n_ops: u64 },
    /// Simulate a full disk (every write fails).
    DiskFull,
    /// Slow all disk operations by `multiplier` (e.g. 10.0 = 10× slower).
    SlowDisk { multiplier: f64 },
}

/// Injects one or more faults into an operation stream.
///
/// All state is internally synchronized so `FaultInjector` can be shared
/// across threads via `Arc<FaultInjector>`.
pub struct FaultInjector {
    faults: Vec<(Fault, AtomicBool)>,
    op_counter: AtomicU64,
    rng_seed: u64,
}

impl FaultInjector {
    /// Create a new injector seeded with `seed`.
    pub fn new(seed: u64) -> Self {
        Self {
            faults: Vec::new(),
            op_counter: AtomicU64::new(0),
            rng_seed: seed,
        }
    }

    /// Register a fault.
    pub fn add_fault(&mut self, fault: Fault) -> &mut Self {
        self.faults.push((fault, AtomicBool::new(true)));
        self
    }

    /// Increment the operation counter by one.
    pub fn tick(&self) {
        self.op_counter.fetch_add(1, Ordering::Relaxed);
    }

    /// Return the current operation count.
    pub fn op_count(&self) -> u64 {
        self.op_counter.load(Ordering::Relaxed)
    }

    /// Reset the operation counter to zero.
    pub fn reset(&self) {
        self.op_counter.store(0, Ordering::Relaxed);
    }

    // -----------------------------------------------------------------------
    // Query helpers (deterministic, driven by the seeded PRNG)
    // -----------------------------------------------------------------------

    /// Return a [`Duration`] to delay the next operation, if any delay fault
    /// is registered. Uses a simple LCG seeded from `rng_seed + op_count`.
    pub fn should_delay(&self) -> Option<Duration> {
        let ops = self.op_counter.load(Ordering::Relaxed);
        for (fault, active) in &self.faults {
            if active.load(Ordering::Relaxed)
                && let Fault::Delay { min_ms, max_ms } = fault
            {
                let range = max_ms - min_ms + 1;
                let ms = min_ms + lcg(self.rng_seed, ops) % range;
                return Some(Duration::from_millis(ms));
            }
        }
        None
    }

    /// Return `true` if the current operation should be dropped (packet loss).
    pub fn should_drop(&self) -> bool {
        let ops = self.op_counter.load(Ordering::Relaxed);
        for (fault, active) in &self.faults {
            if active.load(Ordering::Relaxed)
                && let Fault::PacketLoss { probability } = fault
            {
                let r = lcg_f64(self.rng_seed.wrapping_add(1), ops);
                return r < *probability;
            }
        }
        false
    }

    /// Return `true` if the current operation should be corrupted.
    pub fn should_corrupt(&self) -> bool {
        let ops = self.op_counter.load(Ordering::Relaxed);
        for (fault, active) in &self.faults {
            if active.load(Ordering::Relaxed)
                && let Fault::Corrupt { probability } = fault
            {
                let r = lcg_f64(self.rng_seed.wrapping_add(2), ops);
                return r < *probability;
            }
        }
        false
    }

    /// Return `true` if the crash trigger has fired.
    ///
    /// The crash fires when `op_count >= after_n_ops` for any `Crash` fault.
    pub fn should_crash(&self) -> bool {
        let ops = self.op_counter.load(Ordering::Relaxed);
        for (fault, active) in &self.faults {
            if active.load(Ordering::Relaxed)
                && let Fault::Crash { after_n_ops } = fault
                && ops >= *after_n_ops
            {
                return true;
            }
        }
        false
    }
}

// ---------------------------------------------------------------------------
// Minimal seeded LCG (Knuth multiplier)
// ---------------------------------------------------------------------------

/// Map `(seed, counter)` to a pseudo-random `u64`.
fn lcg(seed: u64, counter: u64) -> u64 {
    seed.wrapping_mul(6364136223846793005)
        .wrapping_add(counter.wrapping_mul(1442695040888963407))
        .wrapping_add(1)
}

/// Map `(seed, counter)` to a pseudo-random float in `[0.0, 1.0)`.
fn lcg_f64(seed: u64, counter: u64) -> f64 {
    let v = lcg(seed, counter);
    // Use the top 53 bits to get a value in [0, 2^53).
    (v >> 11) as f64 / (1u64 << 53) as f64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_injector_no_faults() {
        let inj = FaultInjector::new(42);
        assert!(!inj.should_drop());
        assert!(!inj.should_corrupt());
        assert!(!inj.should_crash());
        assert!(inj.should_delay().is_none());
    }

    #[test]
    fn add_delay_fault() {
        let mut inj = FaultInjector::new(1);
        inj.add_fault(Fault::Delay {
            min_ms: 1,
            max_ms: 10,
        });
        let d = inj.should_delay().expect("expected Some(delay)");
        assert!(d.as_millis() >= 1 && d.as_millis() <= 10);
    }

    #[test]
    fn should_delay_returns_duration() {
        let mut inj = FaultInjector::new(7);
        inj.add_fault(Fault::Delay {
            min_ms: 5,
            max_ms: 5,
        });
        let d = inj.should_delay().unwrap();
        assert_eq!(d, Duration::from_millis(5));
    }

    #[test]
    fn packet_loss_0_never_drops() {
        let mut inj = FaultInjector::new(0);
        inj.add_fault(Fault::PacketLoss { probability: 0.0 });
        for _ in 0..100 {
            inj.tick();
            assert!(!inj.should_drop());
        }
    }

    #[test]
    fn packet_loss_1_always_drops() {
        let mut inj = FaultInjector::new(0);
        inj.add_fault(Fault::PacketLoss { probability: 1.0 });
        for _ in 0..10 {
            inj.tick();
            assert!(inj.should_drop());
        }
    }

    #[test]
    fn corrupt_respects_probability() {
        let mut inj = FaultInjector::new(99);
        inj.add_fault(Fault::Corrupt { probability: 1.0 });
        assert!(inj.should_corrupt());

        let mut inj2 = FaultInjector::new(99);
        inj2.add_fault(Fault::Corrupt { probability: 0.0 });
        assert!(!inj2.should_corrupt());
    }

    #[test]
    fn crash_fires_after_n_ops() {
        let mut inj = FaultInjector::new(0);
        inj.add_fault(Fault::Crash { after_n_ops: 5 });
        for _ in 0..5 {
            assert!(!inj.should_crash());
            inj.tick();
        }
        assert!(inj.should_crash());
    }

    #[test]
    fn tick_increments_counter() {
        let inj = FaultInjector::new(0);
        assert_eq!(inj.op_count(), 0);
        inj.tick();
        inj.tick();
        assert_eq!(inj.op_count(), 2);
    }

    #[test]
    fn reset_zeroes_counter() {
        let inj = FaultInjector::new(0);
        inj.tick();
        inj.tick();
        inj.reset();
        assert_eq!(inj.op_count(), 0);
    }

    #[test]
    fn op_count_correct() {
        let inj = FaultInjector::new(0);
        for i in 0..7 {
            assert_eq!(inj.op_count(), i);
            inj.tick();
        }
        assert_eq!(inj.op_count(), 7);
    }
}
