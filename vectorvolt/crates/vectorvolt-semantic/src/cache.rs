//! Read model cache tier (VV-PRJ-015, VV-PRJ-016).
//!
//! `ReadCache` trait + `InProcessReadCache` (dashmap + TTL).
//! [`RedisReadCache`] is the real external tier (GET / SET EX / DEL via the
//! `redis` crate). Cache failures never fail a read: a Redis error is logged
//! and treated as a miss.

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

// ── RedisReadCache ───────────────────────────────────────────────────────────

/// Redis-backed read cache: `SET key json EX ttl` / `GET` / `DEL`.
///
/// Uses the synchronous `redis` API behind a mutex-held connection that is
/// dropped on error and re-established lazily.
pub struct RedisReadCache {
    client: redis::Client,
    conn: std::sync::Mutex<Option<redis::Connection>>,
    prefix: String,
    ttl_s: u64,
    connect_timeout: Duration,
}

/// Errors creating a [`RedisReadCache`].
#[derive(Debug, thiserror::Error)]
pub enum RedisCacheError {
    #[error("redis: {0}")]
    Redis(String),
}

/// Serialize a snapshot to its Redis string value.
pub fn encode_snapshot(snapshot: &ReadModelSnapshot) -> Result<String, serde_json::Error> {
    serde_json::to_string(snapshot)
}

/// Parse a Redis string value back to a snapshot.
pub fn decode_snapshot(raw: &str) -> Result<ReadModelSnapshot, serde_json::Error> {
    serde_json::from_str(raw)
}

impl RedisReadCache {
    /// Connect to Redis (`redis://host:port/db`); fails if unreachable.
    /// `ttl_s` is applied to every entry (minimum 1 second).
    pub fn connect(
        url: &str,
        prefix: &str,
        ttl_s: u64,
        connect_timeout: Duration,
    ) -> Result<Self, RedisCacheError> {
        let client = redis::Client::open(url).map_err(|e| RedisCacheError::Redis(e.to_string()))?;
        let conn = client
            .get_connection_with_timeout(connect_timeout)
            .map_err(|e| RedisCacheError::Redis(format!("connect: {e}")))?;
        Ok(Self {
            client,
            conn: std::sync::Mutex::new(Some(conn)),
            prefix: prefix.to_string(),
            ttl_s: ttl_s.max(1),
            connect_timeout,
        })
    }

    fn full_key(&self, key: &str) -> String {
        format!("{}{}", self.prefix, key)
    }

    /// Run `f` on the connection, reconnecting if needed; drops the
    /// connection on error.
    fn with_conn<T>(
        &self,
        f: impl FnOnce(&mut redis::Connection) -> redis::RedisResult<T>,
    ) -> Option<T> {
        let mut guard = self.conn.lock().unwrap_or_else(|p| p.into_inner());
        if guard.is_none() {
            match self
                .client
                .get_connection_with_timeout(self.connect_timeout)
            {
                Ok(c) => *guard = Some(c),
                Err(e) => {
                    tracing::warn!("redis cache reconnect failed: {e}");
                    return None;
                }
            }
        }
        match f(guard.as_mut().expect("connection present")) {
            Ok(v) => Some(v),
            Err(e) => {
                tracing::warn!("redis cache error: {e}");
                *guard = None;
                None
            }
        }
    }
}

impl ReadCache for RedisReadCache {
    fn get(&self, key: &str) -> Option<Arc<ReadModelSnapshot>> {
        let k = self.full_key(key);
        let raw: Option<String> = self
            .with_conn(|c| redis::cmd("GET").arg(&k).query(c))
            .flatten();
        match decode_snapshot(&raw?) {
            Ok(s) => Some(Arc::new(s)),
            Err(e) => {
                tracing::warn!("redis cache: undecodable value for {k}: {e}");
                None
            }
        }
    }

    fn set(&self, key: &str, snapshot: Arc<ReadModelSnapshot>) {
        let Ok(body) = encode_snapshot(&snapshot) else {
            return;
        };
        let k = self.full_key(key);
        let ttl = self.ttl_s;
        let _: Option<()> = self.with_conn(|c| {
            redis::cmd("SET")
                .arg(&k)
                .arg(&body)
                .arg("EX")
                .arg(ttl)
                .query(c)
        });
    }

    fn invalidate(&self, key: &str) {
        let k = self.full_key(key);
        let _: Option<u64> = self.with_conn(|c| redis::cmd("DEL").arg(&k).query(c));
    }
}

#[cfg(test)]
mod redis_tests {
    use super::*;

    #[test]
    fn snapshot_roundtrip() {
        let mut s = ReadModelSnapshot::new();
        s.insert("a".into(), serde_json::json!(1));
        s.insert("b".into(), serde_json::json!({"x": [1, 2]}));
        let raw = encode_snapshot(&s).unwrap();
        assert_eq!(decode_snapshot(&raw).unwrap(), s);
        assert!(decode_snapshot("not json").is_err());
    }

    #[test]
    fn connect_unreachable_errors() {
        let r = RedisReadCache::connect(
            "redis://127.0.0.1:1/",
            "vv:",
            60,
            Duration::from_millis(500),
        );
        assert!(r.is_err());
    }

    /// Needs a live Redis: `VV_TEST_REDIS=redis://host:6379/`.
    #[test]
    #[ignore = "requires VV_TEST_REDIS"]
    fn live_set_get_invalidate_ttl() {
        let url = std::env::var("VV_TEST_REDIS").expect("VV_TEST_REDIS");
        let cache = RedisReadCache::connect(&url, "vvtest:", 1, Duration::from_secs(3)).unwrap();
        let mut s = ReadModelSnapshot::new();
        s.insert("n".into(), serde_json::json!(5));
        cache.set("k", Arc::new(s.clone()));
        assert_eq!(*cache.get("k").unwrap(), s);
        std::thread::sleep(Duration::from_millis(2200));
        assert!(cache.get("k").is_none(), "TTL should expire entry");
        cache.set("k2", Arc::new(s));
        cache.invalidate("k2");
        assert!(cache.get("k2").is_none());
    }
}
