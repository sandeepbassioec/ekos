//! Crash-safe whole-file persistence for the JSON-backed stores.
//!
//! The store is rewritten in full on every mutation, so a plain `fs::write`
//! that is interrupted leaves a truncated file and loses everything. Instead
//! the new contents go to `<file>.tmp`, are fsynced, and are renamed over the
//! target; the parent directory is then fsynced so the rename itself is
//! durable. A crash at any point leaves either the old or the new file intact,
//! plus possibly a stale `.tmp`, which `read_committed` discards.

use std::fs::{self, File};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

fn tmp_path(path: &Path) -> PathBuf {
    let mut name = path.as_os_str().to_owned();
    name.push(".tmp");
    PathBuf::from(name)
}

/// fsync a directory so a rename inside it survives a crash. Windows has no
/// directory fsync (and NTFS journals the rename), so it is a no-op there.
fn sync_dir(dir: &Path) -> io::Result<()> {
    #[cfg(unix)]
    {
        File::open(dir)?.sync_all()?;
    }
    #[cfg(not(unix))]
    {
        let _ = dir;
    }
    Ok(())
}

/// Atomically replace `path` with `bytes`.
pub(crate) fn write_atomic(path: &Path, bytes: &[u8]) -> io::Result<()> {
    let tmp = tmp_path(path);
    let written = (|| {
        let mut f = File::create(&tmp)?;
        f.write_all(bytes)?;
        f.sync_all()?;
        drop(f);
        fs::rename(&tmp, path)
    })();
    if let Err(e) = written {
        let _ = fs::remove_file(&tmp);
        return Err(e);
    }
    match path.parent() {
        Some(dir) if !dir.as_os_str().is_empty() => sync_dir(dir),
        _ => sync_dir(Path::new(".")),
    }
}

/// Read the committed contents of `path`, or `None` when it does not exist
/// (an empty store). Any other read error is returned rather than treated as
/// empty. A leftover `.tmp` from a write that crashed before its rename is
/// never the committed state, so it is removed.
pub(crate) fn read_committed(path: &Path) -> io::Result<Option<Vec<u8>>> {
    match fs::remove_file(tmp_path(path)) {
        Ok(()) => {}
        Err(e) if e.kind() == io::ErrorKind::NotFound => {}
        Err(e) => return Err(e),
    }
    match fs::read(path) {
        Ok(raw) => Ok(Some(raw)),
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(e),
    }
}
