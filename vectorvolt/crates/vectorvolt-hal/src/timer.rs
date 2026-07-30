//! Cycle-accurate timing.
//!
//! ## Why `Instant` is not enough
//!
//! VectorVolt's read target is under 100 ns. `std::time::Instant::now()` costs
//! roughly 20–30 ns per call on Windows and Linux, because it goes through a
//! vDSO or QPC call. Measuring a 100 ns operation with a 25 ns ruler — twice,
//! once at each end — means up to half your measurement is the measurement. That
//! is why `vvctl bench` currently reports a `min` of `0 ns` for GET: the timer
//! granularity swallowed the operation.
//!
//! `RDTSC` reads the CPU's timestamp counter in ~6–8 cycles (about 2 ns) with no
//! kernel transition. It is the only honest way to measure something this fast.
//!
//! ## Invariant TSC
//!
//! On every CPU since Nehalem/Bulldozer the TSC is *invariant*: it ticks at a
//! fixed rate tied to the base clock, independent of turbo, frequency scaling,
//! and C-states. That is what makes a single [`CycleTimer::calibrate`] valid for
//! the process lifetime. On pre-invariant hardware these numbers would drift —
//! VectorVolt does not target such CPUs.
//!
//! ## Caveats you must respect when using this
//!
//! - **RDTSC is not a serializing instruction.** The CPU may reorder it against
//!   surrounding work. For measuring a block of tens of nanoseconds this is
//!   noise, not bias; for single-instruction measurement use `RDTSCP` or a fence.
//! - **Cross-core comparison is unsound** unless threads are pinned. Two cores'
//!   TSCs are synchronized on modern platforms but not guaranteed to be. Pin
//!   before you compare (VV-HAL-024).
//!
//! ## Usage
//!
//! ```
//! use vectorvolt_hal::timer::{CycleTimer, cycles};
//!
//! let timer = CycleTimer::calibrate();
//! let start = cycles();
//! let mut acc = 0u64;
//! for i in 0..1000u64 { acc = acc.wrapping_add(i); }
//! let elapsed_ns = timer.cycles_to_ns(cycles().saturating_sub(start));
//! assert!(acc > 0);
//! # let _ = elapsed_ns;
//! ```

use std::time::Instant;

/// Read the CPU timestamp counter.
///
/// On x86-64 this is a single `RDTSC` (~2 ns). On other architectures it falls
/// back to a monotonic nanosecond clock, so the unit is *nanoseconds, not
/// cycles* there — use [`CycleTimer`] rather than interpreting this directly.
#[inline(always)]
pub fn cycles() -> u64 {
    #[cfg(target_arch = "x86_64")]
    {
        // SAFETY: RDTSC is unprivileged and available on every x86-64 CPU. It
        // reads a counter register and has no side effects.
        unsafe { core::arch::x86_64::_rdtsc() }
    }
    #[cfg(target_arch = "aarch64")]
    {
        let value: u64;
        // SAFETY: CNTVCT_EL0 is the unprivileged virtual counter, readable from
        // EL0 on every aarch64 implementation we target.
        unsafe {
            core::arch::asm!("mrs {}, cntvct_el0", out(reg) value, options(nomem, nostack));
        }
        value
    }
    #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
    {
        monotonic_nanos()
    }
}

/// Nanoseconds since an arbitrary process-local epoch. Portable fallback, only
/// compiled on architectures without a readable cycle counter.
#[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
#[inline]
fn monotonic_nanos() -> u64 {
    use std::sync::OnceLock;
    static EPOCH: OnceLock<Instant> = OnceLock::new();
    EPOCH.get_or_init(Instant::now).elapsed().as_nanos() as u64
}

/// Converts [`cycles`] deltas into nanoseconds.
///
/// Calibrate once at startup and reuse; calibration busy-waits for a few
/// milliseconds and is far too expensive for a hot path.
#[derive(Debug, Clone, Copy)]
pub struct CycleTimer {
    cycles_per_ns: f64,
}

impl CycleTimer {
    /// Default busy-wait duration for calibration.
    const CALIBRATION_NANOS: u128 = 2_000_000; // 2 ms

    /// Measure the TSC rate against the OS monotonic clock.
    ///
    /// Busy-waits ~2 ms. Uses a spin rather than `sleep` so the CPU stays in a
    /// high-frequency P-state — sleeping would let it drop to idle and, on
    /// non-invariant hardware, skew the ratio.
    pub fn calibrate() -> Self {
        Self::calibrate_for(Self::CALIBRATION_NANOS)
    }

    /// [`calibrate`](Self::calibrate) with an explicit busy-wait budget in
    /// nanoseconds. Shorter windows are faster but noisier.
    pub fn calibrate_for(busy_nanos: u128) -> Self {
        let wall_start = Instant::now();
        let tsc_start = cycles();

        // Spin. `black_box` keeps the loop from being optimized away.
        while wall_start.elapsed().as_nanos() < busy_nanos {
            std::hint::black_box(cycles());
        }

        let elapsed_ns = wall_start.elapsed().as_nanos() as f64;
        let elapsed_cycles = cycles().saturating_sub(tsc_start) as f64;

        let cycles_per_ns = if elapsed_ns > 0.0 && elapsed_cycles > 0.0 {
            elapsed_cycles / elapsed_ns
        } else {
            1.0 // Degenerate clock; treat cycles as nanoseconds.
        };

        CycleTimer { cycles_per_ns }
    }

    /// A timer that treats cycles as nanoseconds 1:1. For tests and for
    /// architectures where [`cycles`] already returns nanoseconds.
    pub const fn identity() -> Self {
        CycleTimer { cycles_per_ns: 1.0 }
    }

    /// Measured TSC ticks per nanosecond — numerically equal to the base clock
    /// in GHz on x86-64.
    #[inline]
    pub fn cycles_per_ns(&self) -> f64 {
        self.cycles_per_ns
    }

    /// Approximate base clock in GHz.
    #[inline]
    pub fn ghz(&self) -> f64 {
        self.cycles_per_ns
    }

    /// Convert a cycle delta to nanoseconds.
    #[inline(always)]
    pub fn cycles_to_ns(&self, cycles: u64) -> u64 {
        (cycles as f64 / self.cycles_per_ns) as u64
    }

    /// Convert a cycle delta to nanoseconds without truncating.
    #[inline(always)]
    pub fn cycles_to_ns_f64(&self, cycles: u64) -> f64 {
        cycles as f64 / self.cycles_per_ns
    }

    /// Convert nanoseconds to an equivalent cycle count.
    #[inline(always)]
    pub fn ns_to_cycles(&self, nanos: u64) -> u64 {
        (nanos as f64 * self.cycles_per_ns) as u64
    }
}

impl Default for CycleTimer {
    fn default() -> Self {
        Self::calibrate()
    }
}

/// RAII cycle counter — records [`cycles`] on construction.
///
/// ```
/// use vectorvolt_hal::timer::{Stopwatch, CycleTimer};
/// let timer = CycleTimer::calibrate();
/// let sw = Stopwatch::start();
/// std::hint::black_box((0..100).sum::<u64>());
/// let _ns = sw.elapsed_ns(&timer);
/// ```
#[derive(Debug, Clone, Copy)]
pub struct Stopwatch {
    start: u64,
}

impl Stopwatch {
    #[inline(always)]
    pub fn start() -> Self {
        Stopwatch { start: cycles() }
    }

    /// Cycles since [`start`](Self::start). Saturating, so a TSC that appears to
    /// go backwards (migration between unsynchronized cores) yields 0 rather
    /// than a nonsense huge value.
    #[inline(always)]
    pub fn elapsed_cycles(&self) -> u64 {
        cycles().saturating_sub(self.start)
    }

    #[inline(always)]
    pub fn elapsed_ns(&self, timer: &CycleTimer) -> u64 {
        timer.cycles_to_ns(self.elapsed_cycles())
    }

    /// Reset the origin to now.
    #[inline(always)]
    pub fn reset(&mut self) {
        self.start = cycles();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cycles_advance_monotonically() {
        let a = cycles();
        let b = cycles();
        assert!(b >= a, "timestamp counter went backwards: {a} then {b}");
    }

    #[test]
    fn calibration_yields_a_plausible_clock() {
        let t = CycleTimer::calibrate_for(1_000_000); // 1 ms
        let ghz = t.ghz();
        // Any CPU this code will run on is between 0.5 and 10 GHz. On a
        // non-x86 fallback the ratio is ~1.0, which is inside this window.
        assert!(
            ghz > 0.3 && ghz < 12.0,
            "implausible calibrated clock: {ghz} GHz"
        );
    }

    #[test]
    fn conversions_round_trip() {
        let t = CycleTimer::identity();
        assert_eq!(t.cycles_to_ns(1000), 1000);
        assert_eq!(t.ns_to_cycles(1000), 1000);
    }

    #[test]
    fn stopwatch_measures_a_real_interval() {
        let t = CycleTimer::calibrate_for(500_000);
        let sw = Stopwatch::start();
        // Enough work to exceed timer noise.
        let mut acc = 0u64;
        for i in 0..200_000u64 {
            acc = acc.wrapping_add(i);
        }
        std::hint::black_box(acc);
        let ns = sw.elapsed_ns(&t);
        assert!(ns > 0, "200k iterations measured as 0 ns");
        assert!(ns < 1_000_000_000, "200k iterations took over 1 s: {ns} ns");
    }

    #[test]
    fn stopwatch_reset_moves_the_origin() {
        let mut sw = Stopwatch::start();
        for i in 0..10_000u64 {
            std::hint::black_box(i);
        }
        let first = sw.elapsed_cycles();
        sw.reset();
        let after_reset = sw.elapsed_cycles();
        assert!(
            after_reset <= first,
            "reset did not move origin forward: {first} then {after_reset}"
        );
    }

    #[test]
    fn identity_timer_needs_no_calibration() {
        // Must be const-constructible for use in static contexts.
        const T: CycleTimer = CycleTimer::identity();
        assert_eq!(T.cycles_to_ns(42), 42);
    }
}
