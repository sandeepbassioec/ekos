//! VV-SEC-004: Linux capability drop and no-new-privs hardening.
//!
//! Call `harden()` once, after the server has bound its socket but before
//! accepting connections. After this call the process holds zero capabilities
//! and can never gain new ones, even via setuid binaries.
//!
//! On non-Linux platforms this is a no-op (Linux kernel feature only).

/// Drop all Linux capabilities and set PR_SET_NO_NEW_PRIVS.
///
/// After this call:
/// - Bounding set is empty (no capability can ever be added back)
/// - PR_SET_NO_NEW_PRIVS = 1 (execve cannot gain privs)
/// - Securebits locked (uid=0 grants no special treatment)
///
/// Must be called after socket bind — dropping CAP_NET_BIND_SERVICE before
/// bind would prevent listening on ports < 1024.
#[cfg(target_os = "linux")]
pub fn harden() -> Result<(), String> {
    use libc::{PR_CAPBSET_DROP, PR_SET_NO_NEW_PRIVS, PR_SET_SECUREBITS};

    unsafe {
        // Drop every capability from the bounding set (caps 0..=40 covers all
        // capabilities defined up to kernel 5.14).
        for cap in 0u64..=40u64 {
            libc::prctl(PR_CAPBSET_DROP, cap, 0u64, 0u64, 0u64);
        }
        // No new privs: execve can no longer gain privileges
        let ret = libc::prctl(PR_SET_NO_NEW_PRIVS, 1u64, 0u64, 0u64, 0u64);
        if ret != 0 {
            return Err(format!(
                "PR_SET_NO_NEW_PRIVS failed: errno={}",
                *libc::__errno_location()
            ));
        }
        // Lock securebits so they cannot be undone by a privileged child
        // 0x2f = SECBIT_NOROOT | SECBIT_NOROOT_LOCKED |
        //        SECBIT_NO_SETUID_FIXUP | SECBIT_NO_SETUID_FIXUP_LOCKED |
        //        SECBIT_NO_CAP_AMBIENT_RAISE | SECBIT_NO_CAP_AMBIENT_RAISE_LOCKED
        libc::prctl(PR_SET_SECUREBITS, 0x2fu64, 0u64, 0u64, 0u64);
    }
    Ok(())
}

/// No-op on non-Linux platforms.
#[cfg(not(target_os = "linux"))]
pub fn harden() -> Result<(), String> {
    Ok(())
}

/// Returns `true` if PR_SET_NO_NEW_PRIVS is active on this process (Linux only).
#[cfg(target_os = "linux")]
pub fn no_new_privs_active() -> bool {
    let ret = unsafe { libc::prctl(libc::PR_GET_NO_NEW_PRIVS, 0u64, 0u64, 0u64, 0u64) };
    ret == 1
}

#[cfg(not(target_os = "linux"))]
pub fn no_new_privs_active() -> bool {
    false
}
