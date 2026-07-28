//! SSTable (Sorted String Table) — immutable on-disk KV file produced by memtable flush.
//!
//! File layout:
//!
//!   [Header: 16 bytes]
//!     magic   : [u8; 8]  = b"VVST0001"
//!     count   : u32      (number of entries)
//!     _pad    : u32
//!
//!   [Data section: variable]
//!     For each entry (lexicographically sorted by key):
//!       key_len  : u32
//!       key      : [u8; key_len]
//!       val_type : u8    (0 = value, 1 = tombstone)
//!       val_len  : u32   (0 when tombstone)
//!       val      : [u8; val_len]
//!
//!   [Index section: variable]
//!     For each entry:
//!       key_len  : u32
//!       key      : [u8; key_len]
//!       offset   : u64   (byte offset of this entry in Data section)
//!
//!   [Footer: 16 bytes]
//!     index_offset : u64   (byte offset of Index section start from file start)
//!     magic        : [u8; 8]  = b"VVSTFOOT"

use std::{
    fs::{File, OpenOptions},
    io::{BufReader, BufWriter, Read, Seek, SeekFrom, Write},
    path::{Path, PathBuf},
};

use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use bytes::Bytes;

use crate::{StoreError, StoreResult, memtable::MemEntry};

const HEADER_MAGIC: &[u8; 8] = b"VVST0001";
const FOOTER_MAGIC: &[u8; 8] = b"VVSTFOOT";
const HEADER_LEN: u64 = 16;
const FOOTER_LEN: u64 = 16;
const VAL_TYPE_VALUE: u8 = 0;
const VAL_TYPE_TOMBSTONE: u8 = 1;

// --------------------------------------------------------------------------
// Writer
// --------------------------------------------------------------------------

/// Builds an SSTable file from a sorted sequence of MemEntry.
pub struct SsTableWriter {
    path: PathBuf,
    writer: BufWriter<File>,
    data_offset: u64,
    /// (key, data_offset) collected during write for the index
    index_entries: Vec<(Bytes, u64)>,
    count: u32,
}

impl SsTableWriter {
    pub fn create(path: impl AsRef<Path>) -> StoreResult<Self> {
        let file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(path.as_ref())
            .map_err(|e| StoreError::Io(e.to_string()))?;
        let mut writer = BufWriter::new(file);
        // Write placeholder header (will be seeked back to fill count)
        writer
            .write_all(HEADER_MAGIC)
            .map_err(|e| StoreError::Io(e.to_string()))?;
        writer
            .write_u32::<BigEndian>(0)
            .map_err(|e| StoreError::Io(e.to_string()))?; // count placeholder
        writer
            .write_u32::<BigEndian>(0)
            .map_err(|e| StoreError::Io(e.to_string()))?; // pad
        Ok(SsTableWriter {
            path: path.as_ref().to_path_buf(),
            writer,
            data_offset: HEADER_LEN,
            index_entries: Vec::new(),
            count: 0,
        })
    }

    /// Append one entry. Entries MUST be supplied in sorted key order.
    pub fn append(&mut self, entry: &MemEntry) -> StoreResult<()> {
        self.index_entries
            .push((entry.key.clone(), self.data_offset));

        let key_len = entry.key.len() as u32;
        self.writer
            .write_u32::<BigEndian>(key_len)
            .map_err(|e| StoreError::Io(e.to_string()))?;
        self.writer
            .write_all(&entry.key)
            .map_err(|e| StoreError::Io(e.to_string()))?;

        match &entry.value {
            Some(val) => {
                self.writer
                    .write_u8(VAL_TYPE_VALUE)
                    .map_err(|e| StoreError::Io(e.to_string()))?;
                self.writer
                    .write_u32::<BigEndian>(val.len() as u32)
                    .map_err(|e| StoreError::Io(e.to_string()))?;
                self.writer
                    .write_all(val)
                    .map_err(|e| StoreError::Io(e.to_string()))?;
                self.data_offset += 4 + key_len as u64 + 1 + 4 + val.len() as u64;
            }
            None => {
                self.writer
                    .write_u8(VAL_TYPE_TOMBSTONE)
                    .map_err(|e| StoreError::Io(e.to_string()))?;
                self.writer
                    .write_u32::<BigEndian>(0)
                    .map_err(|e| StoreError::Io(e.to_string()))?;
                self.data_offset += 4 + key_len as u64 + 1 + 4;
            }
        }

        self.count += 1;
        Ok(())
    }

    /// Finalize: write index + footer, rewrite count in header, sync.
    pub fn finish(mut self) -> StoreResult<PathBuf> {
        let index_offset = self.data_offset;

        for (key, offset) in &self.index_entries {
            self.writer
                .write_u32::<BigEndian>(key.len() as u32)
                .map_err(|e| StoreError::Io(e.to_string()))?;
            self.writer
                .write_all(key)
                .map_err(|e| StoreError::Io(e.to_string()))?;
            self.writer
                .write_u64::<BigEndian>(*offset)
                .map_err(|e| StoreError::Io(e.to_string()))?;
        }

        // Footer
        self.writer
            .write_u64::<BigEndian>(index_offset)
            .map_err(|e| StoreError::Io(e.to_string()))?;
        self.writer
            .write_all(FOOTER_MAGIC)
            .map_err(|e| StoreError::Io(e.to_string()))?;

        self.writer
            .flush()
            .map_err(|e| StoreError::Io(e.to_string()))?;

        // Rewrite count in header
        {
            let mut file = self
                .writer
                .into_inner()
                .map_err(|e| StoreError::Io(e.to_string()))?;
            file.seek(SeekFrom::Start(8))
                .map_err(|e| StoreError::Io(e.to_string()))?;
            {
                let mut w = BufWriter::new(&mut file);
                w.write_u32::<BigEndian>(self.count)
                    .map_err(|e| StoreError::Io(e.to_string()))?;
                w.flush().map_err(|e| StoreError::Io(e.to_string()))?;
                // `w` drops here, releasing the mutable borrow on `file`
            }
            file.sync_all().map_err(|e| StoreError::Io(e.to_string()))?;
        }

        Ok(self.path)
    }
}

// --------------------------------------------------------------------------
// Reader / Index
// --------------------------------------------------------------------------

/// In-memory index entry (loaded from the index section of the SSTable).
#[derive(Debug, Clone)]
struct IndexEntry {
    key: Bytes,
    offset: u64,
}

/// Opened SSTable ready for point-lookups and full scans.
pub struct SsTable {
    pub path: PathBuf,
    pub count: u32,
    index: Vec<IndexEntry>,
    #[allow(dead_code)]
    index_offset: u64,
}

impl SsTable {
    /// Open an existing SSTable and load its index into memory.
    pub fn open(path: impl AsRef<Path>) -> StoreResult<Self> {
        let path = path.as_ref().to_path_buf();
        let mut file =
            BufReader::new(File::open(&path).map_err(|e| StoreError::Io(e.to_string()))?);

        // Validate header magic
        let mut magic = [0u8; 8];
        file.read_exact(&mut magic)
            .map_err(|e| StoreError::Io(e.to_string()))?;
        if &magic != HEADER_MAGIC {
            return Err(StoreError::Corrupt(format!(
                "{}: bad header magic",
                path.display()
            )));
        }
        let count = file
            .read_u32::<BigEndian>()
            .map_err(|e| StoreError::Io(e.to_string()))?;
        let _pad = file
            .read_u32::<BigEndian>()
            .map_err(|e| StoreError::Io(e.to_string()))?;

        // Read footer for index_offset
        let file_len = std::fs::metadata(&path)
            .map_err(|e| StoreError::Io(e.to_string()))?
            .len();
        if file_len < HEADER_LEN + FOOTER_LEN {
            return Err(StoreError::Corrupt(format!(
                "{}: file too short",
                path.display()
            )));
        }
        file.seek(SeekFrom::Start(file_len - FOOTER_LEN))
            .map_err(|e| StoreError::Io(e.to_string()))?;
        let index_offset = file
            .read_u64::<BigEndian>()
            .map_err(|e| StoreError::Io(e.to_string()))?;
        let mut footer_magic = [0u8; 8];
        file.read_exact(&mut footer_magic)
            .map_err(|e| StoreError::Io(e.to_string()))?;
        if &footer_magic != FOOTER_MAGIC {
            return Err(StoreError::Corrupt(format!(
                "{}: bad footer magic",
                path.display()
            )));
        }

        // Load index section
        file.seek(SeekFrom::Start(index_offset))
            .map_err(|e| StoreError::Io(e.to_string()))?;
        let index_end = file_len - FOOTER_LEN;
        let mut index = Vec::with_capacity(count as usize);
        while file
            .stream_position()
            .map_err(|e| StoreError::Io(e.to_string()))?
            < index_end
        {
            let key_len = file
                .read_u32::<BigEndian>()
                .map_err(|e| StoreError::Io(e.to_string()))? as usize;
            let mut key_buf = vec![0u8; key_len];
            file.read_exact(&mut key_buf)
                .map_err(|e| StoreError::Io(e.to_string()))?;
            let offset = file
                .read_u64::<BigEndian>()
                .map_err(|e| StoreError::Io(e.to_string()))?;
            index.push(IndexEntry {
                key: Bytes::from(key_buf),
                offset,
            });
        }

        Ok(SsTable {
            path,
            count,
            index,
            index_offset,
        })
    }

    /// Point lookup. Returns `Some(None)` for tombstone, `None` for not-found.
    pub fn get(&self, key: &Bytes) -> StoreResult<Option<Option<Bytes>>> {
        let pos = self.index.partition_point(|e| e.key < *key);
        if pos >= self.index.len() || self.index[pos].key != *key {
            return Ok(None);
        }
        let entry = self.read_entry_at(self.index[pos].offset)?;
        Ok(Some(entry.value))
    }

    fn read_entry_at(&self, offset: u64) -> StoreResult<MemEntry> {
        let mut file =
            BufReader::new(File::open(&self.path).map_err(|e| StoreError::Io(e.to_string()))?);
        file.seek(SeekFrom::Start(offset))
            .map_err(|e| StoreError::Io(e.to_string()))?;

        let key_len = file
            .read_u32::<BigEndian>()
            .map_err(|e| StoreError::Io(e.to_string()))? as usize;
        let mut key_buf = vec![0u8; key_len];
        file.read_exact(&mut key_buf)
            .map_err(|e| StoreError::Io(e.to_string()))?;

        let val_type = file.read_u8().map_err(|e| StoreError::Io(e.to_string()))?;
        let val_len = file
            .read_u32::<BigEndian>()
            .map_err(|e| StoreError::Io(e.to_string()))? as usize;

        let value = match val_type {
            VAL_TYPE_VALUE => {
                let mut vbuf = vec![0u8; val_len];
                file.read_exact(&mut vbuf)
                    .map_err(|e| StoreError::Io(e.to_string()))?;
                Some(Bytes::from(vbuf))
            }
            VAL_TYPE_TOMBSTONE => None,
            _ => {
                return Err(StoreError::Corrupt(format!(
                    "unknown val_type {val_type} at offset {offset}"
                )));
            }
        };

        Ok(MemEntry {
            key: Bytes::from(key_buf),
            value,
        })
    }

    /// Iterate all entries in sorted key order.
    pub fn iter(&self) -> StoreResult<Vec<MemEntry>> {
        let mut entries = Vec::with_capacity(self.count as usize);
        for idx_entry in &self.index {
            entries.push(self.read_entry_at(idx_entry.offset)?);
        }
        Ok(entries)
    }

    /// True file size in bytes.
    pub fn file_size(&self) -> StoreResult<u64> {
        Ok(std::fs::metadata(&self.path)
            .map_err(|e| StoreError::Io(e.to_string()))?
            .len())
    }

    pub fn entry_count(&self) -> u32 {
        self.count
    }
}

// --------------------------------------------------------------------------
// Helpers for naming
// --------------------------------------------------------------------------

/// Canonical SSTable file name from a monotonic id.
pub fn sstable_path(dir: &Path, id: u64) -> PathBuf {
    dir.join(format!("{id:020}.vvsst"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memtable::MemEntry;
    use tempfile::TempDir;

    fn entries(pairs: &[(&str, Option<&str>)]) -> Vec<MemEntry> {
        pairs
            .iter()
            .map(|(k, v)| MemEntry {
                key: Bytes::from(k.to_string()),
                value: v.map(|s| Bytes::from(s.to_string())),
            })
            .collect()
    }

    #[test]
    fn write_read_point_lookup() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("test.vvsst");

        let mut w = SsTableWriter::create(&path).unwrap();
        for e in entries(&[("a", Some("1")), ("b", None), ("c", Some("3"))]) {
            w.append(&e).unwrap();
        }
        w.finish().unwrap();

        let sst = SsTable::open(&path).unwrap();
        assert_eq!(sst.count, 3);

        // Present key with value
        assert_eq!(
            sst.get(&Bytes::from("a")).unwrap().unwrap().unwrap(),
            Bytes::from("1")
        );
        // Tombstone
        assert!(sst.get(&Bytes::from("b")).unwrap().unwrap().is_none());
        // Present key
        assert_eq!(
            sst.get(&Bytes::from("c")).unwrap().unwrap().unwrap(),
            Bytes::from("3")
        );
        // Missing
        assert!(sst.get(&Bytes::from("z")).unwrap().is_none());
    }

    #[test]
    fn full_scan_ordered() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("scan.vvsst");

        let mut w = SsTableWriter::create(&path).unwrap();
        for e in entries(&[
            ("apple", Some("a")),
            ("banana", Some("b")),
            ("cherry", Some("c")),
        ]) {
            w.append(&e).unwrap();
        }
        w.finish().unwrap();

        let sst = SsTable::open(&path).unwrap();
        let all = sst.iter().unwrap();
        let keys: Vec<_> = all.iter().map(|e| e.key.clone()).collect();
        assert_eq!(keys, vec!["apple", "banana", "cherry"]);
    }
}
