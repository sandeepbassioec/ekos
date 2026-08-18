/// Snapshot marker written to `data/snapshot.vvss` after a successful flush + compact.
///
/// Format: `[b"VVSS"][version:u16LE][row_count:u64LE][byte_size:u64LE]`
///
/// The marker signals `open_with` to truncate the WAL before replaying, since every
/// row is already durable in compacted partition files.
use std::io::{Read, Write};
use std::path::Path;

const MAGIC: &[u8; 4] = b"VVSS";
const VERSION: u16 = 1;

/// Metadata returned by `TableEngine::take_snapshot`.
#[derive(Debug, Clone)]
pub struct SnapshotMeta {
    /// Number of live rows captured in the snapshot.
    pub row_count: u64,
    /// Total size in bytes of all partition files at snapshot time.
    pub byte_size: u64,
}

pub fn write_marker(path: &Path, meta: &SnapshotMeta) -> std::io::Result<()> {
    let mut buf = Vec::with_capacity(22);
    buf.write_all(MAGIC)?;
    buf.write_all(&VERSION.to_le_bytes())?;
    buf.write_all(&meta.row_count.to_le_bytes())?;
    buf.write_all(&meta.byte_size.to_le_bytes())?;
    // atomic write: write to .tmp then rename
    let tmp = path.with_extension("vvss.tmp");
    std::fs::write(&tmp, &buf)?;
    std::fs::rename(&tmp, path)?;
    Ok(())
}

pub fn read_marker(path: &Path) -> std::io::Result<Option<SnapshotMeta>> {
    let mut f = match std::fs::File::open(path) {
        Ok(f) => f,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(e),
    };
    let mut buf = [0u8; 22];
    f.read_exact(&mut buf)?;
    if &buf[..4] != MAGIC {
        return Ok(None);
    }
    let row_count = u64::from_le_bytes(buf[6..14].try_into().unwrap());
    let byte_size = u64::from_le_bytes(buf[14..22].try_into().unwrap());
    Ok(Some(SnapshotMeta {
        row_count,
        byte_size,
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

/// Delete all WAL segment files in `wal_dir` so the next `WalWriter::open` starts fresh.
pub fn truncate_wal_dir(wal_dir: &Path) -> std::io::Result<()> {
    for entry in std::fs::read_dir(wal_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            std::fs::remove_file(&path)?;
        }
    }
    Ok(())
}
