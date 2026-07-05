//! KvStore — the top-level storage engine.
//!
//! Read path:  memtable → immutable memtables → SSTables (newest first)
//! Write path: WAL append → memtable insert → auto-flush when threshold exceeded
//!
//! Recovery:   on open, replay WAL into a fresh memtable, then scan existing
//!             SSTables from disk to rebuild the in-memory index list.

use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use bytes::Bytes;
use parking_lot::RwLock;

use vectorvolt_wal::{WalConfig, WalReader, WalWriter};

use crate::{
    compaction::compact,
    memtable::MemTable,
    record::LogRecord,
    sstable::{SsTable, SsTableWriter, sstable_path},
    StoreError, StoreResult,
};

// --------------------------------------------------------------------------
// Config
// --------------------------------------------------------------------------

#[derive(Clone, Debug)]
pub struct StoreConfig {
    pub dir: PathBuf,
    /// Flush active memtable to SSTable when it exceeds this size (bytes).
    pub memtable_flush_threshold: usize,
    /// Trigger compaction when the SSTable count exceeds this number.
    pub compaction_trigger: usize,
    /// fsync WAL on flush (disable for tests to go faster).
    pub fsync: bool,
}

impl StoreConfig {
    pub fn new(dir: impl AsRef<Path>) -> Self {
        StoreConfig {
            dir: dir.as_ref().to_path_buf(),
            memtable_flush_threshold: 4 * 1024 * 1024, // 4 MiB
            compaction_trigger: 4,
            fsync: true,
        }
    }

    pub fn with_flush_threshold(mut self, bytes: usize) -> Self {
        self.memtable_flush_threshold = bytes;
        self
    }

    pub fn with_compaction_trigger(mut self, n: usize) -> Self {
        self.compaction_trigger = n;
        self
    }

    pub fn with_fsync(mut self, v: bool) -> Self {
        self.fsync = v;
        self
    }
}

// --------------------------------------------------------------------------
// Manifest — tracks which SSTables are live and the next IDs
// --------------------------------------------------------------------------

struct Manifest {
    /// Live SSTables, oldest first.
    tables: Vec<(SsTable, u64)>,
    next_sst_id: u64,
}

impl Manifest {
    fn scan_dir(dir: &Path) -> StoreResult<Self> {
        let mut ids: Vec<u64> = Vec::new();
        if let Ok(rd) = std::fs::read_dir(dir) {
            for entry in rd.flatten() {
                let name = entry.file_name();
                let s = name.to_string_lossy();
                if s.ends_with(".vvsst") {
                    if let Ok(id) = s.trim_end_matches(".vvsst").parse::<u64>() {
                        ids.push(id);
                    }
                }
            }
        }
        ids.sort_unstable();

        let mut tables = Vec::with_capacity(ids.len());
        for id in &ids {
            let path = sstable_path(dir, *id);
            let sst = SsTable::open(&path)?;
            tables.push((sst, *id));
        }

        let next_sst_id = ids.last().map(|&id| id + 1).unwrap_or(1);
        Ok(Manifest { tables, next_sst_id })
    }

    fn next_id(&mut self) -> u64 {
        let id = self.next_sst_id;
        self.next_sst_id += 1;
        id
    }
}

// --------------------------------------------------------------------------
// Inner state (held under RwLock)
// --------------------------------------------------------------------------

struct StoreInner {
    wal: WalWriter,
    active: MemTable,
    /// Frozen memtables pending flush — older indices have lower indices.
    immutable: Vec<Arc<MemTable>>,
    manifest: Manifest,
    cfg: StoreConfig,
}

impl StoreInner {
    fn flush_active(&mut self) -> StoreResult<()> {
        if self.active.is_empty() {
            return Ok(());
        }
        let table = std::mem::replace(&mut self.active, MemTable::new());
        let id = self.manifest.next_id();
        let path = sstable_path(&self.cfg.dir, id);

        let mut writer = SsTableWriter::create(&path)?;
        for entry in table.into_entries() {
            writer.append(&entry)?;
        }
        writer.finish()?;

        let sst = SsTable::open(&path)?;
        self.manifest.tables.push((sst, id));
        Ok(())
    }

    fn maybe_compact(&mut self) -> StoreResult<()> {
        if self.manifest.tables.len() < self.cfg.compaction_trigger {
            return Ok(());
        }
        // Compact all current SSTables into one
        let id = self.manifest.next_id();
        let tables = std::mem::take(&mut self.manifest.tables);
        // Bottom-most compaction — safe to drop tombstones
        let result = compact(&tables, &self.cfg.dir, id, true)?;

        // Remove old files
        for (old_sst, _) in &tables {
            let _ = std::fs::remove_file(&old_sst.path);
        }

        let sst = SsTable::open(&result.output_path)?;
        self.manifest.tables = vec![(sst, id)];
        Ok(())
    }

    fn get(&self, key: &Bytes) -> StoreResult<Option<Bytes>> {
        // 1. Active memtable
        if let Some(v) = self.active.get(key) {
            return Ok(v.cloned());
        }
        // 2. Immutable memtables (newest first)
        for imm in self.immutable.iter().rev() {
            if let Some(v) = imm.get(key) {
                return Ok(v.cloned());
            }
        }
        // 3. SSTables (newest first)
        for (sst, _id) in self.manifest.tables.iter().rev() {
            if let Some(v) = sst.get(key)? {
                return Ok(v); // None = tombstone → return None
            }
        }
        Ok(None)
    }
}

// --------------------------------------------------------------------------
// Public KvStore
// --------------------------------------------------------------------------

pub struct KvStore {
    inner: RwLock<StoreInner>,
}

impl KvStore {
    /// Open (or create) a store in `cfg.dir`, recovering from WAL if present.
    pub fn open(cfg: StoreConfig) -> StoreResult<Arc<Self>> {
        std::fs::create_dir_all(&cfg.dir).map_err(|e| StoreError::Io(e.to_string()))?;

        let wal_cfg = WalConfig::default()
            .with_dir(&cfg.dir)
            .with_fsync(cfg.fsync);
        let wal = WalWriter::open(wal_cfg.clone()).map_err(|e| StoreError::Wal(e.to_string()))?;

        let manifest = Manifest::scan_dir(&cfg.dir)?;

        // Replay WAL into a fresh memtable (frames the SSTables don't cover)
        let mut active = MemTable::new();
        let wal_reader = WalReader::open(wal_cfg);
        let frames = wal_reader.read_all().map_err(|e| StoreError::Wal(e.to_string()))?;
        for frame in frames {
            let rec = LogRecord::decode(Bytes::from(frame.payload))?;
            match rec {
                LogRecord::Put { key, value } => active.put(key, value),
                LogRecord::Delete { key } => active.delete(key),
            }
        }

        Ok(Arc::new(KvStore {
            inner: RwLock::new(StoreInner {
                wal,
                active,
                immutable: Vec::new(),
                manifest,
                cfg,
            }),
        }))
    }

    pub fn get(&self, key: impl Into<Bytes>) -> StoreResult<Option<Bytes>> {
        self.inner.read().get(&key.into())
    }

    pub fn put(&self, key: impl Into<Bytes>, value: impl Into<Bytes>) -> StoreResult<()> {
        let key = key.into();
        let value = value.into();
        let rec = LogRecord::Put { key: key.clone(), value: value.clone() };

        let mut g = self.inner.write();
        g.wal.append(rec.encode().to_vec()).map_err(|e| StoreError::Wal(e.to_string()))?;
        g.active.put(key, value);

        let threshold = g.cfg.memtable_flush_threshold;
        if g.active.size_bytes() >= threshold {
            g.flush_active()?;
            g.maybe_compact()?;
        }
        Ok(())
    }

    pub fn delete(&self, key: impl Into<Bytes>) -> StoreResult<()> {
        let key = key.into();
        let rec = LogRecord::Delete { key: key.clone() };

        let mut g = self.inner.write();
        g.wal.append(rec.encode().to_vec()).map_err(|e| StoreError::Wal(e.to_string()))?;
        g.active.delete(key);
        Ok(())
    }

    /// Force flush the active memtable to disk (for testing / graceful shutdown).
    pub fn flush(&self) -> StoreResult<()> {
        let mut g = self.inner.write();
        g.wal.flush().map_err(|e| StoreError::Wal(e.to_string()))?;
        g.flush_active()?;
        g.maybe_compact()
    }

    /// Ordered, tombstone-free scan over memtable + SSTables.
    /// Used by ProjectionDispatcher — not intended for direct use.
    pub fn scan_raw(&self, req: &crate::scan::ScanRequest) -> StoreResult<Vec<crate::scan::ScanEntry>> {
        use std::collections::BTreeMap;
        use crate::scan::ScanEntry;

        let g = self.inner.read();

        // Merge all sources into a single BTreeMap keyed by Bytes.
        // Sources are applied newest → oldest so newer values overwrite older ones.
        // None = tombstone (will be filtered out at the end).
        let mut merged: BTreeMap<Bytes, Option<Bytes>> = BTreeMap::new();

        // 1. SSTables oldest → newest (so newer overwrites older in the map)
        for (sst, _) in &g.manifest.tables {
            for entry in sst.iter()? {
                if req.range.contains(&entry.key) {
                    merged.insert(entry.key, entry.value);
                }
            }
        }

        // 2. Active memtable (newest — overwrites everything)
        for entry in g.active.iter() {
            if req.range.contains(&entry.key) {
                merged.insert(entry.key, entry.value);
            }
        }

        // Collect live entries (tombstones removed), respect limit
        let limit = req.limit.unwrap_or(usize::MAX);
        let mut results = Vec::new();
        for (key, value) in merged {
            if results.len() >= limit { break; }
            if req.range.past_end(&key) { break; }
            if let Some(v) = value {
                results.push(ScanEntry { key, value: v });
            }
        }
        Ok(results)
    }

    /// Snapshot of store statistics.
    pub fn stats(&self) -> StoreStats {
        let g = self.inner.read();
        StoreStats {
            active_memtable_bytes: g.active.size_bytes(),
            active_memtable_entries: g.active.len(),
            sstable_count: g.manifest.tables.len(),
            sstable_entries: g.manifest.tables.iter().map(|(s, _)| s.entry_count() as u64).sum(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct StoreStats {
    pub active_memtable_bytes: usize,
    pub active_memtable_entries: usize,
    pub sstable_count: usize,
    pub sstable_entries: u64,
}
