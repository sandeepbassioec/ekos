//! Partition-file encryption at rest (PP-53).
//!
//! ## Format: an envelope around a complete v3 file
//!
//! An encrypted `.vvsst` is a 32-byte cleartext header followed by one
//! AES-256-GCM blob (`vectorvolt_crypto::encryptor::encrypt_with_aad`, magic
//! `VVE2`) whose plaintext is the **entire** v3 file, header and CRCs included:
//!
//! ```text
//! ┌─────────────────────────────────────────────┐
//! │ magic[8] "VVSST001"   version[4] = 4        │
//! │ file_id[2]  partition_id[2]  key_id[8]      │
//! │ reserved[4] = 0       header_crc32[4]       │
//! ├─────────────────────────────────────────────┤
//! │ "VVE2" nonce[12] ciphertext(v3 file) tag[16]│
//! └─────────────────────────────────────────────┘
//! ```
//!
//! Whole-file rather than per-column or per-block: the reader decrypts once
//! into private memory when it opens the file (the partition-reader cache
//! makes that once per file, not per read) and then serves the unchanged
//! zero-copy v3 reader over those bytes. Per-block sealing would only pay off
//! for a reader that decrypts lazily, which the v3 offset-based layout does not
//! support without a second format change.
//!
//! ## What the tag binds (AAD)
//!
//! `"vectorvolt-vvsst-v4" || header[0..28]`, where the header's
//! `file_id`/`partition_id` fields are replaced by the identity the reader
//! **expects** — taken from the file name when it follows the
//! `{file_id:05}_{partition_id:05}.vvsst` convention every writer uses. So the
//! version, the key id and the partition identity are all authenticated, and a
//! file copied over another partition's path (header and all) fails the tag
//! instead of serving the wrong rows. A file under a non-conventional name
//! falls back to the identity in its own header.
//!
//! ## Keys
//!
//! The header names the key id in clear, so a missing or wrong key is
//! reported as such (with the file name and both ids) before any decryption is
//! attempted. Callers that hold the key pass it explicitly
//! ([`crate::PartitionReader::open_with_key`]); [`crate::PartitionReader::open`]
//! looks the id up in a process-wide registry of live keys
//! ([`register_partition_key`]), which is how code that only has a path (zone
//! peeks, verification) reads an encrypted table. The registry holds weak
//! references: a key is resolvable only while its owner keeps it alive.

use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, Mutex, OnceLock, Weak};

use vectorvolt_crypto::encryptor::{decrypt_with_aad, encrypt_with_aad};
use vectorvolt_crypto::key_store::{KEY_ID_LEN, KeyId, KeyStore};
use zeroize::Zeroize;

use crate::format::{HEADER_FIXED_SIZE, MAGIC};
use crate::{StorageError, StorageResult};

/// Format version of an encrypted partition file. Plaintext files stay v3.
pub const ENCRYPTED_VERSION: u32 = 4;

/// Domain-separation prefix of the partition AAD.
const AAD_DOMAIN: &[u8] = b"vectorvolt-vvsst-v4";

/// Hex form of a key id, for error messages.
pub fn key_id_hex(id: &KeyId) -> String {
    id.iter().map(|b| format!("{b:02x}")).collect()
}

/// True if `bytes` starts with an encrypted (v4) partition header.
pub fn is_encrypted_partition(bytes: &[u8]) -> bool {
    bytes.len() >= 12
        && bytes[0..8] == MAGIC[..]
        && u32::from_le_bytes(bytes[8..12].try_into().unwrap()) == ENCRYPTED_VERSION
}

/// The cleartext envelope header of an encrypted partition.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EnvelopeHeader {
    pub file_id: u16,
    pub partition_id: u16,
    pub key_id: KeyId,
}

fn header_bytes(file_id: u16, partition_id: u16, key_id: &KeyId) -> [u8; HEADER_FIXED_SIZE] {
    let mut h = [0u8; HEADER_FIXED_SIZE];
    h[0..8].copy_from_slice(MAGIC);
    h[8..12].copy_from_slice(&ENCRYPTED_VERSION.to_le_bytes());
    h[12..14].copy_from_slice(&file_id.to_le_bytes());
    h[14..16].copy_from_slice(&partition_id.to_le_bytes());
    h[16..16 + KEY_ID_LEN].copy_from_slice(key_id);
    // h[24..28] reserved, zero.
    let crc = crc32fast::hash(&h[0..28]);
    h[28..32].copy_from_slice(&crc.to_le_bytes());
    h
}

fn aad(file_id: u16, partition_id: u16, key_id: &KeyId) -> Vec<u8> {
    let h = header_bytes(file_id, partition_id, key_id);
    let mut a = Vec::with_capacity(AAD_DOMAIN.len() + 28);
    a.extend_from_slice(AAD_DOMAIN);
    a.extend_from_slice(&h[0..28]);
    a
}

/// Parse and CRC-check an encrypted partition's envelope header.
pub fn parse_envelope_header(bytes: &[u8], path: &Path) -> StorageResult<EnvelopeHeader> {
    if bytes.len() < HEADER_FIXED_SIZE || !is_encrypted_partition(bytes) {
        return Err(StorageError::InvalidData(format!(
            "{} is not an encrypted partition file",
            path.display()
        )));
    }
    let stored = u32::from_le_bytes(bytes[28..32].try_into().unwrap());
    let computed = crc32fast::hash(&bytes[0..28]);
    if stored != computed {
        return Err(StorageError::HeaderCrcMismatch {
            expected: computed,
            stored,
        });
    }
    let mut key_id = [0u8; KEY_ID_LEN];
    key_id.copy_from_slice(&bytes[16..16 + KEY_ID_LEN]);
    Ok(EnvelopeHeader {
        file_id: u16::from_le_bytes(bytes[12..14].try_into().unwrap()),
        partition_id: u16::from_le_bytes(bytes[14..16].try_into().unwrap()),
        key_id,
    })
}

/// `(file_id, partition_id)` from a `{file_id:05}_{partition_id:05}.vvsst`
/// file name (a trailing `.tmp` is ignored). `None` for any other name.
pub fn identity_from_path(path: &Path) -> Option<(u16, u16)> {
    let name = path.file_name()?.to_str()?;
    let name = name.strip_suffix(".tmp").unwrap_or(name);
    let stem = name.strip_suffix(".vvsst")?;
    let (f, p) = stem.split_once('_')?;
    if f.is_empty() || p.is_empty() || !f.bytes().chain(p.bytes()).all(|b| b.is_ascii_digit()) {
        return None;
    }
    Some((f.parse().ok()?, p.parse().ok()?))
}

/// Seal a complete plaintext (v3) partition file under `key`.
///
/// `file_id`/`partition_id` must be the identity the file will be published
/// under (they are bound into the tag). `encrypt_with_aad` records the key use.
pub fn seal_partition(
    plain: &[u8],
    file_id: u16,
    partition_id: u16,
    key: &KeyStore,
) -> StorageResult<Vec<u8>> {
    let key_id = key.key_id();
    let blob = encrypt_with_aad(key, plain, &aad(file_id, partition_id, &key_id))
        .map_err(|e| StorageError::Encryption(e.to_string()))?;
    let mut out = Vec::with_capacity(HEADER_FIXED_SIZE + blob.len());
    out.extend_from_slice(&header_bytes(file_id, partition_id, &key_id));
    out.extend_from_slice(&blob);
    Ok(out)
}

/// Where a reader gets the key for an encrypted file.
#[derive(Clone, Copy)]
pub(crate) enum KeySource<'a> {
    /// Exactly this key (or none): no registry fallback.
    Explicit(Option<&'a KeyStore>),
    /// Look the file's key id up in the process-wide registry.
    Registry,
}

/// Decrypt an encrypted partition file's bytes into its plaintext v3 image.
pub(crate) fn open_envelope(
    bytes: &[u8],
    path: &Path,
    source: KeySource<'_>,
) -> StorageResult<Plaintext> {
    let header = parse_envelope_header(bytes, path)?;
    let registered;
    let key: &KeyStore = match source {
        KeySource::Explicit(Some(k)) => k,
        KeySource::Explicit(None) => {
            return Err(StorageError::PartitionKeyMissing {
                path: path.to_path_buf(),
                key_id: key_id_hex(&header.key_id),
            });
        }
        KeySource::Registry => {
            registered = registered_key(&header.key_id).ok_or_else(|| {
                StorageError::PartitionKeyMissing {
                    path: path.to_path_buf(),
                    key_id: key_id_hex(&header.key_id),
                }
            })?;
            &registered
        }
    };
    if key.key_id() != header.key_id {
        return Err(StorageError::PartitionKeyMismatch {
            path: path.to_path_buf(),
            file_key_id: key_id_hex(&header.key_id),
            supplied_key_id: key_id_hex(&key.key_id()),
        });
    }
    let (file_id, partition_id) =
        identity_from_path(path).unwrap_or((header.file_id, header.partition_id));
    let plain = decrypt_with_aad(
        key,
        &bytes[HEADER_FIXED_SIZE..],
        &aad(file_id, partition_id, &header.key_id),
    )
    .map_err(|e| StorageError::PartitionDecrypt {
        path: path.to_path_buf(),
        reason: e.to_string(),
    })?;
    let plain = Plaintext(plain);
    // The tag already binds the header's identity; this catches a writer that
    // sealed a v3 image whose own header disagrees with the envelope.
    if plain.0.len() >= 16 {
        let inner_file = u16::from_le_bytes(plain.0[12..14].try_into().unwrap());
        let inner_part = u16::from_le_bytes(plain.0[14..16].try_into().unwrap());
        if (inner_file, inner_part) != (header.file_id, header.partition_id) {
            return Err(StorageError::InvalidData(format!(
                "{}: envelope names partition {}/{} but the sealed file is {inner_file}/{inner_part}",
                path.display(),
                header.file_id,
                header.partition_id
            )));
        }
    }
    Ok(plain)
}

/// Decrypted partition bytes in private heap memory, wiped on drop.
pub(crate) struct Plaintext(pub(crate) Vec<u8>);

impl Drop for Plaintext {
    fn drop(&mut self) {
        self.0.zeroize();
    }
}

impl std::ops::Deref for Plaintext {
    type Target = [u8];
    fn deref(&self) -> &[u8] {
        &self.0
    }
}

// ── Key registry ─────────────────────────────────────────────────────────────

// Several owners can hold separate `Arc`s to keys with the same id (e.g. two
// tables opened from the same master key). Every live one is kept, so dropping
// one owner's `Arc` never hides the key from another owner that is still alive.
fn registry() -> &'static Mutex<HashMap<KeyId, Vec<Weak<KeyStore>>>> {
    static REG: OnceLock<Mutex<HashMap<KeyId, Vec<Weak<KeyStore>>>>> = OnceLock::new();
    REG.get_or_init(|| Mutex::new(HashMap::new()))
}

/// Make `key` resolvable by [`crate::PartitionReader::open`] (and the zone
/// peek, verification and anything else that opens a partition by path) for
/// as long as some owner keeps the `Arc` alive.
///
/// `TxnManager::with_wal_encryption` calls this. Code that scans a table's
/// partition files **before** building its `TxnManager` must call it first, or
/// those opens fail with [`StorageError::PartitionKeyMissing`].
pub fn register_partition_key(key: &Arc<KeyStore>) {
    let mut reg = registry().lock().unwrap_or_else(|e| e.into_inner());
    for entries in reg.values_mut() {
        entries.retain(|w| w.strong_count() > 0);
    }
    reg.retain(|_, entries| !entries.is_empty());
    let entries = reg.entry(key.key_id()).or_default();
    if !entries
        .iter()
        .any(|w| std::ptr::eq(w.as_ptr(), Arc::as_ptr(key)))
    {
        entries.push(Arc::downgrade(key));
    }
}

/// The live registered key with this id, if any.
pub fn registered_key(id: &KeyId) -> Option<Arc<KeyStore>> {
    let reg = registry().lock().unwrap_or_else(|e| e.into_inner());
    reg.get(id)?.iter().find_map(Weak::upgrade)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dropping_one_owner_keeps_the_key_for_another() {
        let bytes = [7u8; 32];
        let a = Arc::new(KeyStore::from_bytes(bytes).unwrap());
        let b = Arc::new(KeyStore::from_bytes(bytes).unwrap());
        register_partition_key(&a);
        register_partition_key(&b);
        let id = a.key_id();
        drop(b);
        assert!(registered_key(&id).is_some(), "a is still alive");
        drop(a);
        assert!(registered_key(&id).is_none());
    }

    #[test]
    fn identity_parses_only_the_convention() {
        assert_eq!(
            identity_from_path(Path::new("d/00003_00017.vvsst")),
            Some((3, 17))
        );
        assert_eq!(
            identity_from_path(Path::new("00003_00017.vvsst.tmp")),
            Some((3, 17))
        );
        assert_eq!(identity_from_path(Path::new("x.vvsst")), None);
        assert_eq!(identity_from_path(Path::new("a_b.vvsst")), None);
        assert_eq!(identity_from_path(Path::new("00001_l1.vvsst")), None);
    }

    #[test]
    fn registry_is_weak() {
        let k = Arc::new(KeyStore::generate().unwrap());
        let id = k.key_id();
        register_partition_key(&k);
        assert!(registered_key(&id).is_some());
        drop(k);
        assert!(registered_key(&id).is_none());
    }
}
