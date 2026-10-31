//! Keyed tail seal (PP-38).
//!
//! The frame chain is SHA3-256 but unkeyed: anyone who can write the WAL can
//! rewrite its last frames, or cut the log short, and recompute a chain that
//! still verifies. The seal closes that gap when a key is available. After a
//! frame is synced, the writer records `(sequence, frame_hash)` of the tip
//! together with a MAC under a key derived from the table's encryption key.
//!
//! Because every frame commits to the hash of the one before it, sealing the
//! tip covers the whole log up to that point. On open, [`verify_seal`] checks
//! that the sealed frame is still present with the same hash. A truncated log
//! (the sealed frame is gone), an edited sealed frame (its hash differs), or a
//! rewritten earlier frame (which changes every later hash) all fail.
//!
//! Frames appended after the newest seal are not covered. The seal is written
//! only after the WAL is synced, so it never points past durable frames, and a
//! lost seal update only makes it older, never wrong.
//!
//! File: `<wal_dir>/tail.seal`, 72 bytes:
//! `[b"VVTS"][version:u32 LE][sequence:u64 LE][frame_hash:32][mac:32]`.

use std::path::Path;

use sha3::{Digest, Sha3_256};

use crate::config::WalConfig;
use crate::error::{WalError, WalResult};
use crate::reader::{WalCursor, WalReader};
use crate::writer::WalTip;

pub const SEAL_FILE: &str = "tail.seal";
const MAGIC: &[u8; 4] = b"VVTS";
const VERSION: u32 = 1;
const SEAL_LEN: usize = 4 + 4 + 8 + 32 + 32;

/// Derive the seal MAC key from a table's secret key material.
///
/// Domain-separated so the seal key never equals the encryption key itself.
pub fn derive_seal_key(secret: &[u8]) -> [u8; 32] {
    let mut h = Sha3_256::new();
    h.update(b"vectorvolt-wal-tail-seal-v1");
    h.update(secret);
    h.finalize().into()
}

fn mac(key: &[u8; 32], sequence: u64, frame_hash: &[u8; 32]) -> [u8; 32] {
    // SHA3 is not length-extendable, so a keyed prefix hash is a sound MAC.
    let mut h = Sha3_256::new();
    h.update(key);
    h.update(sequence.to_le_bytes());
    h.update(frame_hash);
    h.finalize().into()
}

/// Record `tip` as the newest sealed frame. Call only after `tip` is synced.
///
/// Written to a temp file and renamed so a reader never sees a partial seal.
pub fn write_seal(dir: &Path, key: &[u8; 32], tip: &WalTip) -> WalResult<()> {
    let mut buf = Vec::with_capacity(SEAL_LEN);
    buf.extend_from_slice(MAGIC);
    buf.extend_from_slice(&VERSION.to_le_bytes());
    buf.extend_from_slice(&tip.sequence.to_le_bytes());
    buf.extend_from_slice(&tip.frame_hash);
    buf.extend_from_slice(&mac(key, tip.sequence, &tip.frame_hash));
    let tmp = dir.join("tail.seal.tmp");
    std::fs::write(&tmp, &buf)?;
    std::fs::rename(&tmp, dir.join(SEAL_FILE))?;
    Ok(())
}

/// Check the WAL in `dir` against its seal.
///
/// No seal file is not an error: nothing has been sealed yet (a new table, or
/// one written before encryption was enabled).
pub fn verify_seal(dir: &Path, key: &[u8; 32]) -> WalResult<()> {
    let bytes = match std::fs::read(dir.join(SEAL_FILE)) {
        Ok(b) => b,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(e) => return Err(e.into()),
    };
    if bytes.len() != SEAL_LEN || &bytes[..4] != MAGIC {
        return Err(WalError::SealMismatch {
            reason: "seal file is malformed".into(),
        });
    }
    let version = u32::from_le_bytes(bytes[4..8].try_into().unwrap());
    if version != VERSION {
        return Err(WalError::SealMismatch {
            reason: format!("unsupported seal version {version}"),
        });
    }
    let sequence = u64::from_le_bytes(bytes[8..16].try_into().unwrap());
    let sealed_hash: [u8; 32] = bytes[16..48].try_into().unwrap();
    let stored_mac: [u8; 32] = bytes[48..80].try_into().unwrap();

    let expected = mac(key, sequence, &sealed_hash);
    let mac_ok = expected
        .iter()
        .zip(stored_mac.iter())
        .fold(0u8, |acc, (a, b)| acc | (a ^ b))
        == 0;
    if !mac_ok {
        return Err(WalError::SealMismatch {
            reason: "seal MAC does not verify (wrong key or edited seal)".into(),
        });
    }

    let mut found: Option<[u8; 32]> = None;
    let cfg = WalConfig::default().with_dir(dir).with_fsync(false);
    WalReader::open(cfg).scan(WalCursor::beginning(), |frame| {
        if frame.sequence == sequence {
            found = Some(frame.frame_hash);
            return false;
        }
        true
    })?;
    match found {
        None => Err(WalError::SealMismatch {
            reason: format!("sealed frame {sequence} is missing: the log was truncated"),
        }),
        Some(h) if h != sealed_hash => Err(WalError::SealMismatch {
            reason: format!(
                "frame {sequence} differs from the sealed frame: the log was rewritten"
            ),
        }),
        Some(_) => Ok(()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::writer::WalWriter;
    use tempfile::TempDir;

    fn writer(dir: &Path) -> WalWriter {
        WalWriter::open(WalConfig::default().with_dir(dir).with_fsync(false)).unwrap()
    }

    #[test]
    fn untouched_log_verifies() {
        let dir = TempDir::new().unwrap();
        let key = derive_seal_key(b"secret");
        let w = writer(dir.path());
        w.append(b"a".to_vec()).unwrap();
        let tip = w.append(b"b".to_vec()).unwrap();
        w.flush().unwrap();
        write_seal(dir.path(), &key, &tip).unwrap();
        w.append(b"c".to_vec()).unwrap();
        w.flush().unwrap();
        drop(w);
        verify_seal(dir.path(), &key).unwrap();
    }

    #[test]
    fn no_seal_is_not_an_error() {
        let dir = TempDir::new().unwrap();
        verify_seal(dir.path(), &derive_seal_key(b"k")).unwrap();
    }

    #[test]
    fn truncated_log_is_detected() {
        let dir = TempDir::new().unwrap();
        let key = derive_seal_key(b"secret");
        let w = writer(dir.path());
        w.append(b"a".to_vec()).unwrap();
        let tip = w.append(b"b".to_vec()).unwrap();
        w.flush().unwrap();
        write_seal(dir.path(), &key, &tip).unwrap();
        drop(w);
        // Wipe every segment: the sealed frame no longer exists.
        for e in std::fs::read_dir(dir.path()).unwrap() {
            let p = e.unwrap().path();
            if p.extension().and_then(|s| s.to_str()) == Some("vwal") {
                std::fs::remove_file(p).unwrap();
            }
        }
        writer(dir.path());
        assert!(matches!(
            verify_seal(dir.path(), &key),
            Err(WalError::SealMismatch { .. })
        ));
    }

    #[test]
    fn forged_seal_is_detected() {
        let dir = TempDir::new().unwrap();
        let w = writer(dir.path());
        let tip = w.append(b"a".to_vec()).unwrap();
        w.flush().unwrap();
        // Sealed under a different key, e.g. by someone who rewrote the log.
        write_seal(dir.path(), &derive_seal_key(b"attacker"), &tip).unwrap();
        drop(w);
        assert!(matches!(
            verify_seal(dir.path(), &derive_seal_key(b"secret")),
            Err(WalError::SealMismatch { .. })
        ));
    }
}
