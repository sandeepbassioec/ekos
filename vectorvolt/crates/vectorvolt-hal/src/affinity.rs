//! CPU affinity — pinning threads to cores (VV-HAL-024, VV-HAL-026).
//!
//! ## Why pin at all
//!
//! An unpinned thread migrates. Every migration throws away the L1 and L2 the
//! thread had warmed — for a shard whose index was cache-resident, that is the
//! difference between a ~4 ns hit and a ~90 ns DRAM trip on the next probe.
//! Migration also invalidates TSC comparisons between samples, since two cores'
//! timestamp counters are only *usually* synchronised.
//!
//! Pinning also makes the [`CoreRoleMap`](crate::topology::CoreRoleMap)
//! enforceable rather than advisory. Detection says "writers belong on cores
//! 12 and 14"; this module is what actually puts them there.
//!
//! ## Restoring, not leaking
//!
//! [`pin_current_thread_to`] returns an [`AffinityGuard`] that restores the
//! previous mask on drop. A thread borrowed from a pool must not come back with
//! someone else's affinity, and a test must not leak a pin into the next test.
//! Deliberately drop the guard with `std::mem::forget` if a pin should outlive
//! the scope — making that explicit beats a silent permanent change.
//!
//! ## Platform limits worth knowing
//!
//! | Platform | Mechanism | Limit |
//! |---|---|---|
//! | Linux | `sched_setaffinity` | none in practice |
//! | Windows | `SetThreadAffinityMask` | **one processor group — 64 logical CPUs.** Machines above that need `SetThreadGroupAffinity`; we detect and refuse rather than silently pin to the wrong core |
//! | macOS | — | **no strict pinning API exists.** `thread_policy_set` is only a hint. We report [`AffinityError::Unsupported`] instead of pretending |
//!
//! ## What this does not do
//!
//! It pins; it does not decide *where*. That decision is
//! [`CoreRoleMap::derive`](crate::topology::CoreRoleMap::derive). Keeping them
//! apart means the policy is testable without touching a real thread, and the
//! mechanism is testable without agreeing on policy.

use crate::topology::CpuTopology;
use std::sync::OnceLock;

/// Cached topology, so a pin does not re-probe the machine.
///
/// Pinning happens at thread startup rather than on a hot path, but detection
/// reads dozens of `/sys` files on Linux and walks a variable-length buffer on
/// Windows — no reason to repeat that per thread.
pub fn cached_topology() -> &'static CpuTopology {
    static TOPO: OnceLock<CpuTopology> = OnceLock::new();
    TOPO.get_or_init(CpuTopology::detect)
}

/// Why a pin was refused.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum AffinityError {
    /// The requested core does not exist on this machine.
    ///
    /// Refused rather than clamped: silently pinning a writer to a different core
    /// than the config named would make a latency regression untraceable.
    #[error("core {requested} does not exist (machine has {available} logical cores: 0..={max})")]
    CoreOutOfRange {
        requested: usize,
        available: usize,
        max: usize,
    },

    /// An empty core set would mean "runnable nowhere".
    #[error("cannot pin to an empty core set")]
    EmptySet,

    /// Windows only: the set spans, or lies beyond, a single 64-CPU processor group.
    #[error(
        "core {core} is outside processor group 0 (Windows affinity masks cover 64 CPUs; \
         SetThreadGroupAffinity is required beyond that and is not implemented)"
    )]
    BeyondProcessorGroup { core: usize },

    /// No strict pinning API on this platform.
    #[error("thread pinning is not supported on this platform")]
    Unsupported,

    /// The OS refused. Carries `errno` on Unix, `GetLastError` on Windows.
    #[error("OS refused the affinity change (code {0})")]
    Os(i32),
}

/// Restores the previous affinity mask when dropped.
///
/// Ignores failure on drop: a `Drop` impl cannot report an error, and panicking
/// while unwinding would abort the process. The restore failing is far less bad
/// than that, so it is silent by design.
#[must_use = "dropping the guard immediately restores the previous affinity, undoing the pin"]
#[derive(Debug)]
pub struct AffinityGuard {
    #[cfg(any(target_os = "linux", target_os = "android"))]
    previous: libc::cpu_set_t,
    #[cfg(target_os = "windows")]
    previous: usize,
    #[cfg(not(any(target_os = "linux", target_os = "android", target_os = "windows")))]
    _unused: (),
}

impl Drop for AffinityGuard {
    fn drop(&mut self) {
        #[cfg(any(target_os = "linux", target_os = "android"))]
        {
            // SAFETY: `previous` was produced by a successful `sched_getaffinity`
            // and is a valid, correctly sized `cpu_set_t`.
            unsafe {
                libc::sched_setaffinity(0, std::mem::size_of::<libc::cpu_set_t>(), &self.previous);
            }
        }
        #[cfg(target_os = "windows")]
        {
            if self.previous != 0 {
                // SAFETY: pseudo-handle from GetCurrentThread is always valid;
                // `previous` is the mask this thread had before we changed it.
                unsafe {
                    windows_sys::Win32::System::Threading::SetThreadAffinityMask(
                        windows_sys::Win32::System::Threading::GetCurrentThread(),
                        self.previous,
                    );
                }
            }
        }
    }
}

/// Pin the calling thread to exactly one logical core.
///
/// The core id is the OS-visible id, matching
/// [`LogicalCore::id`](crate::topology::LogicalCore::id) and the values in a
/// [`CoreRoleMap`](crate::topology::CoreRoleMap).
///
/// # Errors
/// [`AffinityError::CoreOutOfRange`] if the core does not exist; platform errors
/// otherwise. Never clamps.
pub fn pin_current_thread_to(core: usize) -> Result<AffinityGuard, AffinityError> {
    pin_current_thread_to_set(&[core])
}

/// Pin the calling thread to a set of logical cores.
///
/// Use for roles that may share, such as background work across every E-core.
/// For latency-critical threads prefer a single core — a set still permits
/// migration within it.
pub fn pin_current_thread_to_set(cores: &[usize]) -> Result<AffinityGuard, AffinityError> {
    if cores.is_empty() {
        return Err(AffinityError::EmptySet);
    }

    // Validate against the real machine before touching the OS, so the error
    // names the problem rather than surfacing a bare EINVAL.
    let topo = cached_topology();
    let available = topo.logical_count();
    for &c in cores {
        if topo.core(c).is_none() {
            return Err(AffinityError::CoreOutOfRange {
                requested: c,
                available,
                max: available.saturating_sub(1),
            });
        }
    }

    platform::pin(cores)
}

/// Logical core the calling thread is executing on right now, if the platform
/// can say.
///
/// Useful for *verifying* a pin took effect. Note the value is a snapshot: on an
/// unpinned thread it may already be stale by the time you read it.
pub fn current_core() -> Option<usize> {
    platform::current_core()
}

/// True if this platform supports strict pinning.
pub fn is_supported() -> bool {
    cfg!(any(
        target_os = "linux",
        target_os = "android",
        target_os = "windows"
    ))
}

// ── Linux ─────────────────────────────────────────────────────────────────────

#[cfg(any(target_os = "linux", target_os = "android"))]
mod platform {
    use super::{AffinityError, AffinityGuard};

    pub(super) fn pin(cores: &[usize]) -> Result<AffinityGuard, AffinityError> {
        // SAFETY: `sched_getaffinity` fills a zeroed cpu_set_t of the size we
        // declare. Checked for failure before the value is used.
        let previous = unsafe {
            let mut set: libc::cpu_set_t = std::mem::zeroed();
            if libc::sched_getaffinity(0, std::mem::size_of::<libc::cpu_set_t>(), &mut set) != 0 {
                return Err(AffinityError::Os(errno()));
            }
            set
        };

        // SAFETY: CPU_ZERO/CPU_SET operate on a correctly sized, owned cpu_set_t.
        // Core ids were bounds-checked against the detected topology by the caller.
        let rc = unsafe {
            let mut set: libc::cpu_set_t = std::mem::zeroed();
            libc::CPU_ZERO(&mut set);
            for &c in cores {
                libc::CPU_SET(c, &mut set);
            }
            libc::sched_setaffinity(0, std::mem::size_of::<libc::cpu_set_t>(), &set)
        };

        if rc != 0 {
            return Err(AffinityError::Os(errno()));
        }
        Ok(AffinityGuard { previous })
    }

    pub(super) fn current_core() -> Option<usize> {
        // SAFETY: no arguments, no side effects; returns -1 on failure.
        let c = unsafe { libc::sched_getcpu() };
        if c < 0 { None } else { Some(c as usize) }
    }

    fn errno() -> i32 {
        std::io::Error::last_os_error().raw_os_error().unwrap_or(-1)
    }
}

// ── Windows ───────────────────────────────────────────────────────────────────

#[cfg(target_os = "windows")]
mod platform {
    use super::{AffinityError, AffinityGuard};
    use windows_sys::Win32::System::Threading::{
        GetCurrentProcessorNumber, GetCurrentThread, SetThreadAffinityMask,
    };

    /// `SetThreadAffinityMask` takes a `usize` mask covering one processor group.
    const GROUP_CAPACITY: usize = usize::BITS as usize;

    pub(super) fn pin(cores: &[usize]) -> Result<AffinityGuard, AffinityError> {
        let mut mask: usize = 0;
        for &c in cores {
            // A machine with >64 logical CPUs splits into processor groups, and
            // this API cannot address beyond group 0. Refuse rather than wrap the
            // bit around and pin to an unrelated core.
            if c >= GROUP_CAPACITY {
                return Err(AffinityError::BeyondProcessorGroup { core: c });
            }
            mask |= 1usize << c;
        }

        // SAFETY: the pseudo-handle from GetCurrentThread is always valid and needs
        // no close. Returns the PREVIOUS mask, or 0 on failure.
        let previous = unsafe { SetThreadAffinityMask(GetCurrentThread(), mask) };
        if previous == 0 {
            return Err(AffinityError::Os(
                std::io::Error::last_os_error().raw_os_error().unwrap_or(-1),
            ));
        }
        Ok(AffinityGuard { previous })
    }

    pub(super) fn current_core() -> Option<usize> {
        // SAFETY: no arguments, no side effects.
        Some(unsafe { GetCurrentProcessorNumber() } as usize)
    }
}

// ── Everything else ───────────────────────────────────────────────────────────

#[cfg(not(any(target_os = "linux", target_os = "android", target_os = "windows")))]
mod platform {
    use super::{AffinityError, AffinityGuard};

    pub(super) fn pin(_cores: &[usize]) -> Result<AffinityGuard, AffinityError> {
        // macOS has no strict pinning API — `thread_policy_set` is a hint the
        // scheduler may ignore. Reporting Unsupported is honest; returning a guard
        // that did nothing would make every downstream latency claim a lie.
        Err(AffinityError::Unsupported)
    }

    pub(super) fn current_core() -> Option<usize> {
        None
    }
}

// ── tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::topology::CoreRoleMap;

    /// Yield until the thread is observed on an allowed core, or give up.
    ///
    /// An affinity change takes effect at the next scheduling decision, not at the
    /// call, so reading the current core immediately can still show the old one.
    /// Bounded retries keep this from being either flaky or an infinite loop.
    fn settles_on(allowed: &[usize]) -> Option<usize> {
        for _ in 0..2_000 {
            if let Some(c) = current_core()
                && allowed.contains(&c)
            {
                return Some(c);
            }
            std::thread::yield_now();
        }
        current_core()
    }

    #[test]
    fn out_of_range_core_is_refused_not_clamped() {
        let n = cached_topology().logical_count();
        let err = pin_current_thread_to(n + 1_000).expect_err("should refuse a nonexistent core");
        match err {
            AffinityError::CoreOutOfRange {
                requested,
                available,
                ..
            } => {
                assert_eq!(requested, n + 1_000);
                assert_eq!(available, n);
            }
            other => panic!("expected CoreOutOfRange, got {other}"),
        }
        // And the message must name both numbers, so an operator can act on it.
        let msg = pin_current_thread_to(n + 1_000).unwrap_err().to_string();
        assert!(
            msg.contains(&(n + 1_000).to_string()),
            "message lacks the request: {msg}"
        );
        assert!(
            msg.contains(&n.to_string()),
            "message lacks the machine size: {msg}"
        );
    }

    #[test]
    fn empty_set_is_refused() {
        assert_eq!(
            pin_current_thread_to_set(&[]).unwrap_err(),
            AffinityError::EmptySet
        );
    }

    #[test]
    fn current_core_is_within_the_machine() {
        if let Some(c) = current_core() {
            let n = cached_topology().logical_count();
            assert!(c < n, "current core {c} outside 0..{n}");
        }
    }

    #[test]
    fn pinning_actually_moves_the_thread() {
        if !is_supported() {
            eprintln!("skipped: pinning unsupported on this platform");
            return;
        }
        let topo = cached_topology();
        if topo.logical_count() < 2 {
            eprintln!("skipped: needs at least 2 cores");
            return;
        }

        // Run in a child thread so a stuck pin cannot affect the test harness.
        let n = topo.logical_count().min(64); // Windows group-0 limit
        std::thread::spawn(move || {
            for target in [n - 1, 0] {
                let guard = match pin_current_thread_to(target) {
                    Ok(g) => g,
                    Err(AffinityError::Unsupported) => return,
                    Err(e) => panic!("pin to {target} failed: {e}"),
                };
                let observed = settles_on(&[target]);
                assert_eq!(
                    observed,
                    Some(target),
                    "thread did not land on core {target}; observed {observed:?}"
                );
                drop(guard);
            }
        })
        .join()
        .expect("pinning thread panicked");
    }

    #[test]
    fn guard_restores_previous_affinity() {
        if !is_supported() {
            return;
        }
        let topo = cached_topology();
        if topo.logical_count() < 4 {
            return;
        }
        let n = topo.logical_count().min(64);

        std::thread::spawn(move || {
            // Confine to a two-core set, then pin to one of them inside that.
            let wide = vec![0usize, n - 1];
            let outer = match pin_current_thread_to_set(&wide) {
                Ok(g) => g,
                Err(_) => return,
            };
            assert!(settles_on(&wide).is_some());

            {
                let _inner = pin_current_thread_to(n - 1).expect("inner pin");
                assert_eq!(settles_on(&[n - 1]), Some(n - 1), "inner pin did not take");
            } // inner guard drops -> restores the two-core set

            // Must be able to reach core 0 again, which the inner pin forbade.
            let back = settles_on(&[0]);
            assert_eq!(
                back,
                Some(0),
                "guard did not restore the wider mask; thread stuck on {back:?}"
            );
            drop(outer);
        })
        .join()
        .expect("guard thread panicked");
    }

    #[test]
    fn role_map_cores_are_all_pinnable() {
        // The point of the whole module: every core the role map recommends must
        // be a core we can actually pin to.
        if !is_supported() {
            return;
        }
        let topo = cached_topology();
        let roles = CoreRoleMap::derive(topo);

        let mut checked = 0usize;
        for (role, cores) in [
            ("writers", &roles.writers),
            ("readers", &roles.readers),
            ("network", &roles.network),
            ("background", &roles.background),
        ] {
            for &c in cores.iter().take(3) {
                if c >= 64 {
                    continue; // Windows group-0 limit, not a role-map problem
                }
                let handle = std::thread::spawn(move || pin_current_thread_to(c).map(drop));
                let res = handle.join().expect("pin thread panicked");
                assert!(
                    res.is_ok(),
                    "role {role} recommends core {c}, which failed to pin: {res:?}"
                );
                checked += 1;
            }
        }
        assert!(
            checked > 0,
            "no role cores were exercised — role map was empty"
        );
    }

    #[test]
    fn pinning_two_threads_to_distinct_cores_keeps_them_apart() {
        // Shard-per-core depends on this: two pinned threads must not end up
        // sharing a core.
        if !is_supported() {
            return;
        }
        let topo = cached_topology();
        let reps = topo.physical_representatives();
        if reps.len() < 2 || reps[1] >= 64 {
            return;
        }
        let (a, b) = (reps[0], reps[1]);

        let ha = std::thread::spawn(move || {
            let _g = pin_current_thread_to(a).ok()?;
            settles_on(&[a])
        });
        let hb = std::thread::spawn(move || {
            let _g = pin_current_thread_to(b).ok()?;
            settles_on(&[b])
        });

        let (ra, rb) = (ha.join().unwrap(), hb.join().unwrap());
        if let (Some(ra), Some(rb)) = (ra, rb) {
            assert_eq!(ra, a);
            assert_eq!(rb, b);
            assert_ne!(
                ra, rb,
                "two threads pinned to different cores landed on the same one"
            );
        }
    }
}
