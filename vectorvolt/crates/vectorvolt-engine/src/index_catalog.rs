//! Catalog that records which column indexes should survive a restart.
//!
//! Format: `{data_dir}/secondary_index.catalog`
//! Binary: `[count: u64 LE][col_0: u64 LE] ... [col_{n-1}: u64 LE]`
//!
//! On an encrypted table the same bytes are stored as a sealed side file
//! bound to the table identity (PP-54, see `crate::side_crypto`).
//!
//! The catalog is intentionally not crash-safe (no WAL, no CRC). A torn
//! write at shutdown at worst loses the catalog, forcing the user to call
//! `build_secondary_index` again after restart — the same state as before
//! this layer existed.  The `open_with` code treats a missing or unparseable
//! plaintext catalog as "no indexes" rather than an error. An encrypted
//! catalog that does not decrypt (wrong key, altered, copied from another
//! table) is an error naming the file.

use std::path::{Path, PathBuf};

use crate::side_crypto::{FileContents, SideKey, purpose};

/// Canonical path of the catalog file under `data_dir`.
pub(crate) fn catalog_path(data_dir: &Path) -> PathBuf {
    data_dir.join("secondary_index.catalog")
}

/// Write `cols` to `path` atomically (temp-file + rename on the same
/// filesystem, so a partial write never leaves a corrupt catalog).
pub(crate) fn save(path: &Path, cols: &[usize], side: Option<&SideKey>) -> std::io::Result<()> {
    let mut buf: Vec<u8> = Vec::with_capacity(8 + cols.len() * 8);
    buf.extend_from_slice(&(cols.len() as u64).to_le_bytes());
    for &c in cols {
        buf.extend_from_slice(&(c as u64).to_le_bytes());
    }
    crate::side_crypto::write_file(path, side, purpose::INDEX_CATALOG, &buf)
}

/// Load column indexes from `path`.
///
/// Returns an empty vec when the file is absent, too small, or has a
/// truncated body — all treated as "no indexes catalogued".
pub(crate) fn load(path: &Path, side: Option<&SideKey>) -> std::io::Result<Vec<usize>> {
    let bytes = match crate::side_crypto::read_file(path, side, purpose::INDEX_CATALOG) {
        Ok(FileContents::Missing) => return Ok(Vec::new()),
        Ok(FileContents::Plain(b)) | Ok(FileContents::Decrypted(b)) => b,
        Err(e) if e.kind() == std::io::ErrorKind::InvalidData => return Err(e),
        Err(_) => return Ok(Vec::new()),
    };
    if bytes.len() < 8 {
        return Ok(Vec::new());
    }
    let count = u64::from_le_bytes(bytes[..8].try_into().unwrap()) as usize;
    let expected = 8usize.saturating_add(count.saturating_mul(8));
    if bytes.len() < expected {
        return Ok(Vec::new());
    }
    Ok((0..count)
        .map(|i| {
            let off = 8 + i * 8;
            u64::from_le_bytes(bytes[off..off + 8].try_into().unwrap()) as usize
        })
        .collect())
}
