/// Snapshot marker written to `data/snapshot.vvss` after a successful flush + compact.
///
/// Format v2: `[b"VVSS"][version:u16LE][row_count:u64LE][byte_size:u64LE][wal_seq:u64LE]`
/// Format v1 (read only): the same without `wal_seq`.
///
/// `wal_seq` is the WAL sequence through which every mutation is captured in the
/// partition files. On open, replay skips frames at or below it and applies every
/// later one. A v1 marker carries no position, so it is read as `wal_seq = 0`:
/// replay everything that is still in the WAL, which is always safe.
use std::io::{Read, Write};
use std::path::Path;

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

pub fn write_marker(path: &Path, meta: &SnapshotMeta) -> std::io::Result<()> {
    let mut buf = Vec::with_capacity(V2_LEN);
    buf.write_all(MAGIC)?;
    buf.write_all(&VERSION.to_le_bytes())?;
    buf.write_all(&meta.row_count.to_le_bytes())?;
    buf.write_all(&meta.byte_size.to_le_bytes())?;
    buf.write_all(&meta.wal_seq.to_le_bytes())?;
    // atomic write: write to .tmp, sync, then rename
    let tmp = path.with_extension("vvss.tmp");
    {
        let mut f = std::fs::File::create(&tmp)?;
        f.write_all(&buf)?;
        f.sync_all()?;
    }
    std::fs::rename(&tmp, path)?;
    Ok(())
}

pub fn read_marker(path: &Path) -> std::io::Result<Option<SnapshotMeta>> {
    let mut f = match std::fs::File::open(path) {
        Ok(f) => f,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(e),
    };
    let mut buf = Vec::with_capacity(V2_LEN);
    f.read_to_end(&mut buf)?;
    if buf.len() < V1_LEN || &buf[..4] != MAGIC {
        return Ok(None);
    }
    let version = u16::from_le_bytes(buf[4..6].try_into().unwrap());
    let row_count = u64::from_le_bytes(buf[6..14].try_into().unwrap());
    let byte_size = u64::from_le_bytes(buf[14..22].try_into().unwrap());
    let wal_seq = if version >= 2 && buf.len() >= V2_LEN {
        u64::from_le_bytes(buf[22..30].try_into().unwrap())
    } else {
        0
    };
    Ok(Some(SnapshotMeta {
        row_count,
        byte_size,
        wal_seq,
    }))
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
