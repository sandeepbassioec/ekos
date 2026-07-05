use std::path::PathBuf;

use anyhow::Result;
use bytes::Bytes;
use vectorvolt_store::{KvStore, StoreConfig};

pub fn cmd_stats(dir: PathBuf) -> Result<()> {
    let cfg = StoreConfig::new(&dir).with_fsync(false);
    let store = KvStore::open(cfg)?;
    let s = store.stats();

    println!("VectorVolt Store — {}", dir.display());
    println!("  Active memtable : {} entries ({} bytes)", s.active_memtable_entries, s.active_memtable_bytes);
    println!("  SSTables        : {}", s.sstable_count);
    println!("  SSTable entries : {}", s.sstable_entries);
    Ok(())
}

pub fn cmd_get(dir: PathBuf, key: String) -> Result<()> {
    let cfg = StoreConfig::new(&dir).with_fsync(false);
    let store = KvStore::open(cfg)?;
    match store.get(Bytes::from(key.clone()))? {
        Some(v) => {
            let s = std::str::from_utf8(&v).unwrap_or("<binary>");
            println!("{key} = {s}");
        }
        None => {
            println!("{key} = <not found>");
        }
    }
    Ok(())
}

pub fn cmd_put(dir: PathBuf, key: String, value: String) -> Result<()> {
    let cfg = StoreConfig::new(&dir).with_fsync(true);
    let store = KvStore::open(cfg)?;
    store.put(Bytes::from(key.clone()), Bytes::from(value.clone()))?;
    store.flush()?;
    println!("PUT {key} = {value}");
    Ok(())
}

pub fn cmd_delete(dir: PathBuf, key: String) -> Result<()> {
    let cfg = StoreConfig::new(&dir).with_fsync(true);
    let store = KvStore::open(cfg)?;
    store.delete(Bytes::from(key.clone()))?;
    store.flush()?;
    println!("DEL {key}");
    Ok(())
}

pub fn cmd_compact(dir: PathBuf) -> Result<()> {
    let cfg = StoreConfig::new(&dir).with_fsync(true);
    let store = KvStore::open(cfg)?;
    store.flush()?;
    let s = store.stats();
    println!("compaction complete — {} SSTables, {} entries", s.sstable_count, s.sstable_entries);
    Ok(())
}
