//! Encryption at rest for the engine's side files (PP-54 / PP-56).
//!
//! The WAL is encrypted by `TxnManager`; everything else a keyed table writes
//! (`cdc.log`, `snapshot.vvss`, `secondary_index.catalog`, the `--multi`
//! `schema.txt`, the cross-table intent log) goes through this module.
//!
//! ## Sealed blob
//!
//! `[b"VVSF"][version:u8][key_id:8][VVE2 blob]`, where the `VVE2` blob is
//! [`encrypt_with_aad`] output. The AAD is
//! `purpose || 0 || identity || context || key_id`:
//!
//! - `purpose` names the kind of file, so a snapshot marker can never be read
//!   as an index catalog;
//! - `identity` is the table's random id from its WAL table marker (below), so
//!   one table's file copied over another's fails to authenticate;
//! - `context` binds a record to its position where one exists (a CDC record
//!   to its LSN, an intent record to its ordinal).
//!
//! The key id is in the clear so a wrong or missing key is reported as such,
//! naming the file, instead of as an opaque authentication failure.
//!
//! ## Table marker (A12 follow-up)
//!
//! The first keyed open of a table appends a plaintext, MAC'd frame
//! `[0x21][b"VVTM"][version:u8][table_id:16][mac:32]` to its WAL through
//! `TxnManager::append_marker`, which syncs it and writes `wal/tail.seal`
//! before returning. The marker therefore means "this table has been sealed",
//! and it carries the table identity used above. It sits inside the hash
//! chain, and every later encrypted frame is AAD-bound to its sequence, so it
//! cannot be dropped without breaking the log. See [`check_keyed_wal`].
//!
//! [`encrypt_with_aad`]: vectorvolt_crypto::encryptor::encrypt_with_aad

use std::io;
use std::path::Path;
use std::sync::Arc;

use sha3::{Digest, Sha3_256};
use vectorvolt_crypto::KeyStore;
use vectorvolt_crypto::encryptor::{decrypt_with_aad, encrypt_with_aad};
use vectorvolt_wal::segment::key_id_hex;

use crate::{EngineError, EngineResult};

const MAGIC: &[u8; 4] = b"VVSF";
const VERSION: u8 = 1;
const KEY_ID_LEN: usize = vectorvolt_crypto::KEY_ID_LEN;
const HEADER_LEN: usize = 4 + 1 + KEY_ID_LEN;

/// Random per-table identity, recorded in the WAL table marker.
pub type TableId = [u8; 16];

/// Purpose labels bound into every sealed blob's AAD.
pub(crate) mod purpose {
    pub const CDC: &[u8] = b"vectorvolt-cdc-log-v1";
    pub const SNAPSHOT: &[u8] = b"vectorvolt-snapshot-marker-v1";
    pub const INDEX_CATALOG: &[u8] = b"vectorvolt-secondary-index-catalog-v1";
    pub const SCHEMA: &[u8] = b"vectorvolt-schema-v1";
    pub const INTENT: &[u8] = b"vectorvolt-txn-intent-log-v1";
}

/// Identity bound into intent-log records: the log is per database, not per
/// table.
pub(crate) const INTENT_IDENTITY: &[u8] = b"txn-intent-log";

/// A table's key together with its identity.
#[derive(Clone)]
pub struct SideKey {
    key: Arc<KeyStore>,
    table_id: TableId,
}

impl SideKey {
    pub(crate) fn new(key: Arc<KeyStore>, table_id: TableId) -> Self {
        Self { key, table_id }
    }

    /// The table identity bound into this table's sealed files.
    pub fn table_id(&self) -> TableId {
        self.table_id
    }

    pub(crate) fn seal(&self, purpose: &[u8], ctx: &[u8], plaintext: &[u8]) -> io::Result<Vec<u8>> {
        seal(&self.key, purpose, &self.table_id, ctx, plaintext)
    }

    pub(crate) fn open(
        &self,
        purpose: &[u8],
        ctx: &[u8],
        blob: &[u8],
        file: &Path,
    ) -> io::Result<Vec<u8>> {
        open(Some(&self.key), purpose, &self.table_id, ctx, blob, file)
    }
}

/// True if `bytes` is a sealed blob (as opposed to a legacy plaintext file).
pub(crate) fn is_sealed(bytes: &[u8]) -> bool {
    bytes.len() >= HEADER_LEN && &bytes[..4] == MAGIC
}

fn aad(purpose: &[u8], identity: &[u8], ctx: &[u8], key_id: &[u8]) -> Vec<u8> {
    let mut a = Vec::with_capacity(purpose.len() + 1 + identity.len() + ctx.len() + key_id.len());
    a.extend_from_slice(purpose);
    a.push(0);
    a.extend_from_slice(identity);
    a.extend_from_slice(ctx);
    a.extend_from_slice(key_id);
    a
}

/// Encrypt `plaintext` into a sealed blob.
pub(crate) fn seal(
    key: &KeyStore,
    purpose: &[u8],
    identity: &[u8],
    ctx: &[u8],
    plaintext: &[u8],
) -> io::Result<Vec<u8>> {
    let key_id = key.key_id();
    let blob = encrypt_with_aad(key, plaintext, &aad(purpose, identity, ctx, &key_id))
        .map_err(|e| io::Error::other(format!("side-file encrypt failed: {e}")))?;
    let mut out = Vec::with_capacity(HEADER_LEN + blob.len());
    out.extend_from_slice(MAGIC);
    out.push(VERSION);
    out.extend_from_slice(&key_id);
    out.extend_from_slice(&blob);
    Ok(out)
}

fn invalid(file: &Path, reason: String) -> io::Error {
    io::Error::new(
        io::ErrorKind::InvalidData,
        format!("{}: {reason}", file.display()),
    )
}

/// Decrypt a sealed blob read from `file`. Every failure names the file.
pub(crate) fn open(
    key: Option<&KeyStore>,
    purpose: &[u8],
    identity: &[u8],
    ctx: &[u8],
    blob: &[u8],
    file: &Path,
) -> io::Result<Vec<u8>> {
    if !is_sealed(blob) {
        return Err(invalid(file, "not an encrypted side file".into()));
    }
    if blob[4] != VERSION {
        return Err(invalid(
            file,
            format!("unsupported encrypted side-file version {}", blob[4]),
        ));
    }
    let mut want = [0u8; KEY_ID_LEN];
    want.copy_from_slice(&blob[5..HEADER_LEN]);
    let Some(key) = key else {
        return Err(invalid(
            file,
            format!(
                "encrypted under key id {} but no key was provided",
                key_id_hex(&want)
            ),
        ));
    };
    let got = key.key_id();
    if got != want {
        return Err(invalid(
            file,
            format!(
                "encrypted under key id {} but the key provided has id {}",
                key_id_hex(&want),
                key_id_hex(&got)
            ),
        ));
    }
    decrypt_with_aad(
        key,
        &blob[HEADER_LEN..],
        &aad(purpose, identity, ctx, &want),
    )
    .map_err(|e| {
        invalid(
            file,
            format!(
                "authentication failed ({e}): the file was altered or copied from \
                 another table or position"
            ),
        )
    })
}

/// Error for a plaintext file found where a keyed table only writes
/// encrypted ones.
pub(crate) fn unauthenticated(file: &Path, what: &str) -> io::Error {
    invalid(
        file,
        format!(
            "plaintext {what} on an encrypted table is not authenticated; refusing to trust it"
        ),
    )
}

// ── MACs ─────────────────────────────────────────────────────────────────────

/// Derive a MAC key from the table key, separated by `domain`, so it never
/// equals the encryption key or the WAL seal key.
pub(crate) fn derive_mac_key(key: &KeyStore, domain: &[u8]) -> [u8; 32] {
    let mut h = Sha3_256::new();
    h.update(domain);
    h.update(key.as_bytes());
    h.finalize().into()
}

/// Keyed-prefix SHA3-256 (not length-extendable, so a sound MAC).
pub(crate) fn mac(mac_key: &[u8; 32], data: &[u8]) -> [u8; 32] {
    let mut h = Sha3_256::new();
    h.update(mac_key);
    h.update(data);
    h.finalize().into()
}

pub(crate) fn ct_eq(a: &[u8], b: &[u8]) -> bool {
    a.len() == b.len() && a.iter().zip(b).fold(0u8, |acc, (x, y)| acc | (x ^ y)) == 0
}

pub(crate) fn to_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

pub(crate) fn from_hex(s: &str) -> Option<Vec<u8>> {
    if !s.len().is_multiple_of(2) {
        return None;
    }
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(s.get(i..i + 2)?, 16).ok())
        .collect()
}

// ── WAL table marker ─────────────────────────────────────────────────────────

/// WAL opcode of the table marker. Not a mutation opcode, so replay skips it;
/// not `OP_SNAPSHOT_MARKER`, so PITR listing ignores it.
pub(crate) const OP_TABLE_MARKER: u8 = 0x21;
const TABLE_MARKER_MAGIC: &[u8; 4] = b"VVTM";
const TABLE_MARKER_VERSION: u8 = 1;
const TABLE_MARKER_LEN: usize = 1 + 4 + 1 + 16 + 32;
const TABLE_MARKER_DOMAIN: &[u8] = b"vectorvolt-wal-table-marker-v1";

/// Encode a table marker frame payload for `table_id`.
pub(crate) fn encode_table_marker(key: &KeyStore, table_id: &TableId) -> Vec<u8> {
    let mut p = Vec::with_capacity(TABLE_MARKER_LEN);
    p.push(OP_TABLE_MARKER);
    p.extend_from_slice(TABLE_MARKER_MAGIC);
    p.push(TABLE_MARKER_VERSION);
    p.extend_from_slice(table_id);
    let tag = mac(&derive_mac_key(key, TABLE_MARKER_DOMAIN), &p[1..22]);
    p.extend_from_slice(&tag);
    p
}

fn is_table_marker(payload: &[u8]) -> bool {
    payload.len() >= 5 && payload[0] == OP_TABLE_MARKER && &payload[1..5] == TABLE_MARKER_MAGIC
}

/// What [`check_keyed_wal`] found.
#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct KeyedWal {
    /// Identity from the first authenticated table marker, if any.
    pub table_id: Option<TableId>,
    /// The table marker is the WAL's last frame and `tail.seal` is missing:
    /// the crash window between the marker's sync and its seal write. The
    /// caller re-appends the marker, which writes the seal.
    pub needs_reseal: bool,
}

/// Verify a keyed table's WAL: its tail seal, its table marker, and that a
/// table that has been sealed still has its seal (A12 follow-up).
///
/// - `tail.seal` present: verified by `vectorvolt_wal::seal::verify_seal`.
/// - A table marker that fails its MAC is a forgery: error.
/// - A table marker followed by any frame while `tail.seal` is missing means
///   the seal was deleted: error. `append_marker` writes the seal before it
///   returns, so a sealed table can only lack one in the window between the
///   marker's sync and its seal write — when the marker is still the last
///   frame. That case is accepted and reported as `needs_reseal`.
/// - No marker: a new table, or one never opened by this version; nothing
///   says it was sealed, so a missing seal is fine.
pub(crate) fn check_keyed_wal(wal_dir: &Path, key: &KeyStore) -> EngineResult<KeyedWal> {
    let seal_key = vectorvolt_wal::seal::derive_seal_key(key.as_bytes());
    vectorvolt_wal::seal::verify_seal(wal_dir, &seal_key)?;
    let seal_present = wal_dir.join(vectorvolt_wal::seal::SEAL_FILE).exists();

    let mac_key = derive_mac_key(key, TABLE_MARKER_DOMAIN);
    let mut table_id: Option<TableId> = None;
    let mut last_seq = 0u64;
    let mut last_marker_seq = 0u64;
    let mut forged: Option<u64> = None;
    let cfg = vectorvolt_wal::WalConfig::default()
        .with_dir(wal_dir)
        .with_fsync(false);
    vectorvolt_wal::WalReader::open(cfg).scan(vectorvolt_wal::WalCursor::beginning(), |frame| {
        last_seq = frame.sequence;
        let p = &frame.payload;
        if !is_table_marker(p) {
            return true;
        }
        let ok = p.len() == TABLE_MARKER_LEN
            && p[5] == TABLE_MARKER_VERSION
            && ct_eq(&mac(&mac_key, &p[1..22]), &p[22..]);
        if !ok {
            forged = Some(frame.sequence);
            return false;
        }
        if table_id.is_none() {
            let mut id = [0u8; 16];
            id.copy_from_slice(&p[6..22]);
            table_id = Some(id);
        }
        last_marker_seq = frame.sequence;
        true
    })?;
    if let Some(seq) = forged {
        return Err(EngineError::WalDecrypt {
            seq,
            reason: format!(
                "WAL table marker in {} does not authenticate under this key (forged, \
                 or written under another key)",
                wal_dir.display()
            ),
        });
    }
    let mut needs_reseal = false;
    if table_id.is_some() && !seal_present {
        if last_seq != last_marker_seq {
            return Err(EngineError::Wal(vectorvolt_wal::WalError::SealMismatch {
                reason: format!(
                    "{} is missing but the WAL records that this table was sealed \
                     (table marker at sequence {last_marker_seq}, tip {last_seq}): \
                     the seal was deleted",
                    wal_dir.join(vectorvolt_wal::seal::SEAL_FILE).display()
                ),
            }));
        }
        needs_reseal = true;
    }
    Ok(KeyedWal {
        table_id,
        needs_reseal,
    })
}

/// A fresh random table id.
pub(crate) fn new_table_id() -> EngineResult<TableId> {
    // `KeyStore::generate` draws 32 bytes from the OS CSPRNG; half of a fresh
    // key is a random id without another RNG dependency.
    let k = KeyStore::generate().map_err(|e| EngineError::Other(format!("table id: {e}")))?;
    let mut id = [0u8; 16];
    id.copy_from_slice(&k.as_bytes()[..16]);
    Ok(id)
}

/// Read a keyed table's identity from its WAL without opening the engine
/// (the registry needs it to decrypt `schema.txt` before the open).
pub(crate) fn read_table_id(wal_dir: &Path, key: &KeyStore) -> EngineResult<Option<TableId>> {
    if !wal_dir.is_dir() {
        return Ok(None);
    }
    Ok(check_keyed_wal(wal_dir, key)?.table_id)
}

// ── whole-file helpers ───────────────────────────────────────────────────────

/// Write `plaintext` to `path` atomically, sealed when `side` is set.
pub(crate) fn write_file(
    path: &Path,
    side: Option<&SideKey>,
    purpose: &[u8],
    plaintext: &[u8],
) -> io::Result<()> {
    let bytes = match side {
        Some(s) => s.seal(purpose, b"", plaintext)?,
        None => plaintext.to_vec(),
    };
    let mut tmp = path.as_os_str().to_owned();
    tmp.push(".tmp");
    let tmp = std::path::PathBuf::from(tmp);
    {
        use std::io::Write;
        let mut f = std::fs::File::create(&tmp)?;
        f.write_all(&bytes)?;
        f.sync_all()?;
    }
    std::fs::rename(&tmp, path)
}

/// Contents of a whole side file as read from disk.
pub(crate) enum FileContents {
    Missing,
    /// Sealed and authenticated.
    Decrypted(Vec<u8>),
    /// Legacy plaintext.
    Plain(Vec<u8>),
}

/// Read `path`, decrypting it when it is sealed. A sealed file with a
/// wrong or missing key is an error naming the file.
pub(crate) fn read_file(
    path: &Path,
    side: Option<&SideKey>,
    purpose: &[u8],
) -> io::Result<FileContents> {
    let bytes = match std::fs::read(path) {
        Ok(b) => b,
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(FileContents::Missing),
        Err(e) => return Err(e),
    };
    if !is_sealed(&bytes) {
        return Ok(FileContents::Plain(bytes));
    }
    let pt = match side {
        Some(s) => s.open(purpose, b"", &bytes, path)?,
        None => open(None, purpose, b"", b"", &bytes, path)?,
    };
    Ok(FileContents::Decrypted(pt))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seal_round_trips_and_binds_everything() {
        let k = Arc::new(KeyStore::generate().unwrap());
        let s = SideKey::new(Arc::clone(&k), [1; 16]);
        let p = Path::new("x");
        let b = s.seal(purpose::CDC, b"ctx", b"hello").unwrap();
        assert_eq!(s.open(purpose::CDC, b"ctx", &b, p).unwrap(), b"hello");
        assert!(s.open(purpose::SNAPSHOT, b"ctx", &b, p).is_err());
        assert!(s.open(purpose::CDC, b"other", &b, p).is_err());
        let other = SideKey::new(k, [2; 16]);
        assert!(other.open(purpose::CDC, b"ctx", &b, p).is_err());
        let wrong = SideKey::new(Arc::new(KeyStore::generate().unwrap()), [1; 16]);
        let e = wrong.open(purpose::CDC, b"ctx", &b, p).unwrap_err();
        assert!(e.to_string().contains("key id"), "{e}");
    }

    #[test]
    fn hex_round_trips() {
        assert_eq!(
            from_hex(&to_hex(&[0, 1, 254, 255])).unwrap(),
            [0, 1, 254, 255]
        );
        assert!(from_hex("abc").is_none());
    }
}
