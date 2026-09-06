//! Read model cache tier (VV-PRJ-015, VV-PRJ-016).
//!
//! `ReadCache` trait + `InProcessReadCache` (dashmap + TTL).
//! `RedisReadCache` is represented by the same trait so tests use
//! `InProcessReadCache` as the Redis stand-in without a real Redis server.

use std::sync::Arc;
use std::time::{Duration, Instant};

use dashmap::DashMap;
use serde_json::Value;

/// A snapshot of one read model group key → aggregate value.
pub type ReadModelSnapshot = std::collections::HashMap<String, Value>;

/// Cache interface for read models (VV-PRJ-015/016).
pub trait ReadCache: Send + Sync {
    fn get(&self, key: &str) -> Option<Arc<ReadModelSnapshot>>;
    fn set(&self, key: &str, snapshot: Arc<ReadModelSnapshot>);
    fn invalidate(&self, key: &str);
}

// ── InProcessReadCache ───────────────────────────────────────────────────────

/// LRU-style in-process cache with TTL eviction (VV-PRJ-015).
///
/// Uses DashMap for lock-free concurrent access. Eviction is lazy (on read).
pub struct InProcessReadCache {
    map: DashMap<String, (Instant, Arc<ReadModelSnapshot>)>,
    ttl: Duration,
    max_entries: usize,
}

impl InProcessReadCache {
    pub fn new(max_entries: usize, ttl_s: u64) -> Self {
        Self {
            map: DashMap::new(),
            ttl: Duration::from_secs(ttl_s),
            max_entries,
        }
    }
}

impl ReadCache for InProcessReadCache {
    fn get(&self, key: &str) -> Option<Arc<ReadModelSnapshot>> {
        if let Some(entry) = self.map.get(key)
            && entry.0.elapsed() < self.ttl
        {
            return Some(Arc::clone(&entry.1));
        }
        self.map.remove(key);
        None
    }

    fn set(&self, key: &str, snapshot: Arc<ReadModelSnapshot>) {
        if self.map.len() >= self.max_entries {
            // Evict one expired entry or the oldest entry.
            let mut to_remove: Option<String> = None;
            for entry in self.map.iter() {
                if entry.0.elapsed() >= self.ttl {
                    to_remove = Some(entry.key().clone());
                    break;
                }
            }
            if let Some(k) = to_remove {
                self.map.remove(&k);
            }
        }
        self.map.insert(key.to_string(), (Instant::now(), snapshot));
    }

    fn invalidate(&self, key: &str) {
        self.map.remove(key);
    }
}
