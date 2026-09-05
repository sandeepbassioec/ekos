//! VV-SEC-003: DAC (Discretionary Access Control) hardening helpers.
//!
//! Sets data directory ownership to the `vectorvolt` system user and
//! restricts permissions so only that user can read/write/execute.
//!
//! On Linux: `chmod 700` on directories, `chmod 600` on files, plus
//!   `chown vectorvolt:vectorvolt` if the process is running as root.
//!
//! On non-Linux: prints a warning; full DAC hardening requires Linux.

use std::path::Path;

#[derive(Debug)]
pub struct DacReport {
    pub dirs_hardened: usize,
    pub files_hardened: usize,
    pub chown_skipped: bool, // true when not running as root
}

/// Apply DAC permissions to `data_dir` and all files within it.
///
/// * Directories → mode 0o700
/// * Files → mode 0o600
/// * If running as root on Linux, `chown vectorvolt:vectorvolt` is attempted.
pub fn harden_data_dir(data_dir: &Path) -> std::io::Result<DacReport> {
    #[cfg(target_os = "linux")]
    {
        let mut report = DacReport {
            dirs_hardened: 0,
            files_hardened: 0,
            chown_skipped: true,
        };
        set_perms_recursive(data_dir, &mut report)?;
        let uid = unsafe { libc::getuid() };
        if uid == 0 {
            chown_recursive(data_dir)?;
            report.chown_skipped = false;
        }
        return Ok(report);
    }

    #[cfg(not(target_os = "linux"))]
    {
        eprintln!(
            "warning: DAC hardening (chmod/chown) is only supported on Linux. Skipped for {}",
            data_dir.display()
        );
        Ok(DacReport {
            dirs_hardened: 0,
            files_hardened: 0,
            chown_skipped: true,
        })
    }
}

#[cfg(target_os = "linux")]
fn set_perms_recursive(path: &Path, report: &mut DacReport) -> std::io::Result<()> {
    use std::os::unix::fs::PermissionsExt;

    let meta = std::fs::metadata(path)?;
    if meta.is_dir() {
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o700))?;
        report.dirs_hardened += 1;
        for entry in std::fs::read_dir(path)? {
            let entry = entry?;
            set_perms_recursive(&entry.path(), report)?;
        }
    } else if meta.is_file() {
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600))?;
        report.files_hardened += 1;
    }
    Ok(())
}

#[cfg(target_os = "linux")]
fn chown_recursive(path: &Path) -> std::io::Result<()> {
    // Resolve the vectorvolt user and group IDs via getpwnam
    let name = b"vectorvolt\0";
    let pw = unsafe { libc::getpwnam(name.as_ptr() as *const libc::c_char) };
    if pw.is_null() {
        // vectorvolt user doesn't exist yet — skip chown silently
        return Ok(());
    }
    let (uid, gid) = unsafe { ((*pw).pw_uid, (*pw).pw_gid) };
    chown_path(path, uid, gid)?;
    if path.is_dir() {
        for entry in std::fs::read_dir(path)? {
            let entry = entry?;
            chown_recursive(&entry.path())?;
        }
    }
    Ok(())
}

#[cfg(target_os = "linux")]
fn chown_path(path: &Path, uid: libc::uid_t, gid: libc::gid_t) -> std::io::Result<()> {
    use std::ffi::CString;
    let c = CString::new(path.to_str().unwrap_or("")).unwrap();
    let ret = unsafe { libc::chown(c.as_ptr(), uid, gid) };
    if ret != 0 {
        Err(std::io::Error::last_os_error())
    } else {
        Ok(())
    }
}

/// Create the `vectorvolt` system user (Linux only, requires root).
///
/// Returns `Ok(true)` if created, `Ok(false)` if already exists.
#[cfg(target_os = "linux")]
pub fn ensure_system_user() -> std::io::Result<bool> {
    // Check if user already exists
    let name = b"vectorvolt\0";
    let pw = unsafe { libc::getpwnam(name.as_ptr() as *const libc::c_char) };
    if !pw.is_null() {
        return Ok(false); // already exists
    }
    let status = std::process::Command::new("useradd")
        .args([
            "--system",
            "--no-create-home",
            "--shell",
            "/usr/sbin/nologin",
            "vectorvolt",
        ])
        .status()?;
    if !status.success() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "useradd failed — are you running as root?",
        ));
    }
    Ok(true)
}

#[cfg(not(target_os = "linux"))]
pub fn ensure_system_user() -> std::io::Result<bool> {
    eprintln!("warning: system user creation is only supported on Linux");
    Ok(false)
}
