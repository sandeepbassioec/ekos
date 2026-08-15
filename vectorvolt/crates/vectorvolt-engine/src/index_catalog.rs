//! Catalog that records which column indexes should survive a restart.
//!
//! Format: `{data_dir}/secondary_index.catalog`
//! Binary: `[count: u64 LE][col_0: u64 LE] ... [col_{n-1}: u64 LE]`
//!
//! The catalog is intentionally not crash-safe (no WAL, no CRC). A torn
//! write at shutdown at worst loses the catalog, forcing the user to call
//! `build_secondary_index` again after restart — the same state as before
//! this layer existed.  The `open_with` code treats a missing or unparseable
//! catalog as "no indexes" rather than an error.

use std::path::{Path, PathBuf};

/// Canonical path of the catalog file under `data_dir`.
pub(crate) fn catalog_path(data_dir: &Path) -> PathBuf {
    data_dir.join("secondary_index.catalog")
}

/// Write `cols` to `path` atomically (temp-file + rename on the same
/// filesystem, so a partial write never leaves a corrupt catalog).
pub(crate) fn save(path: &Path, cols: &[usize]) -> std::io::Result<()> {
    let dir = path.parent().unwrap_or(Path::new("."));
    let tmp = dir.join("secondary_index.catalog.tmp");

    let mut buf: Vec<u8> = Vec::with_capacity(8 + cols.len() * 8);
    buf.extend_from_slice(&(cols.len() as u64).to_le_bytes());
    for &c in cols {
        buf.extend_from_slice(&(c as u64).to_le_bytes());
    }
    std::fs::write(&tmp, &buf)?;
    std::fs::rename(&tmp, path)?;
    Ok(())
}

/// Load column indexes from `path`.
///
/// Returns an empty vec when the file is absent, too small, or has a
/// truncated body — all treated as "no indexes catalogued".
pub(crate) fn load(path: &Path) -> Vec<usize> {
    let bytes = match std::fs::read(path) {
        Ok(b) => b,
        Err(_) => return Vec::new(),
    };
    if bytes.len() < 8 {
        return Vec::new();
    }
    let count = u64::from_le_bytes(bytes[..8].try_into().unwrap()) as usize;
    let expected = 8 + count * 8;
    if bytes.len() < expected {
        return Vec::new();
    }
    (0..count)
        .map(|i| {
            let off = 8 + i * 8;
            u64::from_le_bytes(bytes[off..off + 8].try_into().unwrap()) as usize
        })
        .collect()
}
