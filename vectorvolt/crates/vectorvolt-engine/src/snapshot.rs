/// Snapshot marker written to `data/snapshot.vvss` after a successful flush + compact.
///
/// Format v2: `[b"VVSS"][version:u16LE][row_count:u64LE][byte_size:u64LE][wal_seq:u64LE]`
/// Format v1 (read only): the same without `wal_seq`.
///
/// `wal_seq` is the WAL sequence through which every mutation is captured in the
/// partition files. On open, replay skips frames at or below it and applies every
/// later one. A v1 marker carries no position, so it is read as `wal_seq = 0`:
/// replay everything that is still in the WAL, which is always safe.
///
/// On an encrypted table the marker is a sealed side file (PP-56, see
/// `crate::side_crypto`) bound to the table identity: a forged `wal_seq` could
/// otherwise make recovery skip frames.
use std::path::Path;

use crate::side_crypto::{FileContents, SideKey, purpose};

const MAGIC: &[u8; 4] = b"VVSS";
const VERSION: u16 = 2;
const V1_LEN: usize = 22;
const V2_LEN: usize = 30;

/// Metadata returned by `TableEngine::take_snapshot`.
#[derive(Debug, Clone)]
pub struct SnapshotMeta {
    /// Number of live rows captured in the snapshot.
    pub row_count: u64,
    /// Total size in bytes of all partition files at snapshot time.
    pub byte_size: u64,
    /// WAL sequence the snapshot's partition files cover (0 = unknown).
    pub wal_seq: u64,
}

fn encode(meta: &SnapshotMeta) -> Vec<u8> {
    let mut buf = Vec::with_capacity(V2_LEN);
    buf.extend_from_slice(MAGIC);
    buf.extend_from_slice(&VERSION.to_le_bytes());
    buf.extend_from_slice(&meta.row_count.to_le_bytes());
    buf.extend_from_slice(&meta.byte_size.to_le_bytes());
    buf.extend_from_slice(&meta.wal_seq.to_le_bytes());
    buf
}

fn decode(buf: &[u8]) -> Option<SnapshotMeta> {
    if buf.len() < V1_LEN || &buf[..4] != MAGIC {
        return None;
    }
    let version = u16::from_le_bytes(buf[4..6].try_into().unwrap());
    let row_count = u64::from_le_bytes(buf[6..14].try_into().unwrap());
    let byte_size = u64::from_le_bytes(buf[14..22].try_into().unwrap());
    let wal_seq = if version >= 2 && buf.len() >= V2_LEN {
        u64::from_le_bytes(buf[22..30].try_into().unwrap())
    } else {
        0
    };
    Some(SnapshotMeta {
        row_count,
        byte_size,
        wal_seq,
    })
}

/// Write the marker in plaintext (unencrypted tables).
pub fn write_marker(path: &Path, meta: &SnapshotMeta) -> std::io::Result<()> {
    write_marker_keyed(path, meta, None)
}

/// Write the marker atomically (temp file, sync, rename), encrypted and
/// authenticated under `side` when set.
pub(crate) fn write_marker_keyed(
    path: &Path,
    meta: &SnapshotMeta,
    side: Option<&SideKey>,
) -> std::io::Result<()> {
    crate::side_crypto::write_file(path, side, purpose::SNAPSHOT, &encode(meta))
}

/// Read a plaintext marker. An encrypted one is an error naming the file.
pub fn read_marker(path: &Path) -> std::io::Result<Option<SnapshotMeta>> {
    Ok(match read_marker_keyed(path, None)? {
        MarkerRead::Missing => None,
        MarkerRead::Authenticated(m) => Some(m),
        MarkerRead::Plain(m) => m,
    })
}

/// What [`read_marker_keyed`] found.
pub(crate) enum MarkerRead {
    Missing,
    /// Encrypted under the table key and authenticated.
    Authenticated(SnapshotMeta),
    /// Legacy plaintext marker (`None` if unparseable). Not authenticated.
    Plain(Option<SnapshotMeta>),
}

/// Read the marker, decrypting it under `side` when it is sealed.
pub(crate) fn read_marker_keyed(
    path: &Path,
    side: Option<&SideKey>,
) -> std::io::Result<MarkerRead> {
    Ok(
        match crate::side_crypto::read_file(path, side, purpose::SNAPSHOT)? {
            FileContents::Missing => MarkerRead::Missing,
            FileContents::Plain(b) => MarkerRead::Plain(decode(&b)),
            FileContents::Decrypted(b) => match decode(&b) {
                Some(m) => MarkerRead::Authenticated(m),
                None => {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        format!("{}: malformed snapshot marker", path.display()),
                    ));
                }
            },
        },
    )
}

/// Returns the combined byte size of all `.vvsst` files in `partition_dir`.
pub fn partition_dir_byte_size(partition_dir: &Path) -> std::io::Result<u64> {
    let mut total = 0u64;
    for entry in std::fs::read_dir(partition_dir)? {
        let entry = entry?;
        if entry.path().extension().and_then(|s| s.to_str()) == Some("vvsst") {
            total += entry.metadata()?.len();
        }
    }
    Ok(total)
}
