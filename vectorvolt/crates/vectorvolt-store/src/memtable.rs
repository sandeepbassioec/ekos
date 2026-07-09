//! MemTable — in-memory sorted KV buffer that absorbs writes before SSTable flush.
//!
//! Design:
//!   - BTreeMap<Bytes, Option<Bytes>>: None = tombstone (delete marker)
//!   - size_bytes tracks approximate heap usage (key + value lengths)
//!   - Immutable snapshot via `freeze()` returns an Arc<MemTable> for background flush
//!   - Thread-safe via external RwLock in KvStore; MemTable itself is not Sync

use bytes::Bytes;
use std::collections::BTreeMap;

/// A single in-memory KV entry.
#[derive(Debug, Clone)]
pub struct MemEntry {
    pub key: Bytes,
    /// `None` = delete tombstone
    pub value: Option<Bytes>,
}

pub struct MemTable {
    data: BTreeMap<Bytes, Option<Bytes>>,
    size_bytes: usize,
}

impl MemTable {
    pub fn new() -> Self {
        MemTable {
            data: BTreeMap::new(),
            size_bytes: 0,
        }
    }

    pub fn put(&mut self, key: Bytes, value: Bytes) {
        let added = key.len() + value.len();
        let removed = self
            .data
            .get(&key)
            .map(|v| key.len() + v.as_ref().map_or(0, |b| b.len()))
            .unwrap_or(0);
        self.size_bytes = self
            .size_bytes
            .saturating_add(added)
            .saturating_sub(removed);
        self.data.insert(key, Some(value));
    }

    pub fn delete(&mut self, key: Bytes) {
        let removed = self
            .data
            .get(&key)
            .map(|v| key.len() + v.as_ref().map_or(0, |b| b.len()))
            .unwrap_or(0);
        // Tombstone: key.len() bytes for the key slot
        self.size_bytes = self
            .size_bytes
            .saturating_add(key.len())
            .saturating_sub(removed);
        self.data.insert(key, None);
    }

    pub fn get(&self, key: &Bytes) -> Option<Option<&Bytes>> {
        self.data.get(key).map(|v| v.as_ref())
    }

    pub fn size_bytes(&self) -> usize {
        self.size_bytes
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Ordered iterator over all entries — used for SSTable flush.
    pub fn iter(&self) -> impl Iterator<Item = MemEntry> + '_ {
        self.data.iter().map(|(k, v)| MemEntry {
            key: k.clone(),
            value: v.clone(),
        })
    }

    /// Consume self into a sorted vec (used by SSTable writer).
    pub fn into_entries(self) -> Vec<MemEntry> {
        self.data
            .into_iter()
            .map(|(key, value)| MemEntry { key, value })
            .collect()
    }
}

impl Default for MemTable {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn put_get_delete() {
        let mut m = MemTable::new();
        m.put(Bytes::from("k1"), Bytes::from("v1"));
        m.put(Bytes::from("k2"), Bytes::from("v2"));

        // present key
        assert_eq!(m.get(&Bytes::from("k1")).unwrap().unwrap(), b"v1".as_ref());
        // missing key
        assert!(m.get(&Bytes::from("nope")).is_none());

        m.delete(Bytes::from("k1"));
        // tombstone
        assert!(m.get(&Bytes::from("k1")).unwrap().is_none());
    }

    #[test]
    fn size_tracking() {
        let mut m = MemTable::new();
        assert_eq!(m.size_bytes(), 0);
        m.put(Bytes::from("key"), Bytes::from("value")); // 3 + 5 = 8
        assert_eq!(m.size_bytes(), 8);
        m.put(Bytes::from("key"), Bytes::from("v2")); // overwrite: 3+2=5
        assert_eq!(m.size_bytes(), 5);
        m.delete(Bytes::from("key")); // tombstone: 3
        assert_eq!(m.size_bytes(), 3);
    }

    #[test]
    fn ordered_iteration() {
        let mut m = MemTable::new();
        m.put(Bytes::from("c"), Bytes::from("3"));
        m.put(Bytes::from("a"), Bytes::from("1"));
        m.put(Bytes::from("b"), Bytes::from("2"));
        let keys: Vec<_> = m.iter().map(|e| e.key.clone()).collect();
        assert_eq!(
            keys,
            vec![Bytes::from("a"), Bytes::from("b"), Bytes::from("c")]
        );
    }
}
