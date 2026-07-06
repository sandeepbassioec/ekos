//! KoStore — the six Phase 1 syscalls over VectorVolt.
//!
//! All mutations go through WAL before touching the memtable; reads hit
//! memtable first then SSTables.  The store is cheaply cloneable (Arc inside).

use std::sync::Arc;

use bytes::Bytes;
use chrono::Utc;
use tracing::{debug, instrument};
use vectorvolt_store::{KvStore, ProjectionDispatcher, ScanRequest, StoreConfig};

use crate::{
    error::{KernelError, KernelResult},
    index,
    ko::{KnowledgeObject, KoStatus},
};

// --------------------------------------------------------------------------
// KoStore
// --------------------------------------------------------------------------

#[derive(Clone)]
pub struct KoStore {
    kv: Arc<KvStore>,
}

impl KoStore {
    pub fn open(cfg: StoreConfig) -> KernelResult<Self> {
        let kv = KvStore::open(cfg)?;
        Ok(KoStore { kv })
    }

    // ── Syscall 1: ko_put ──────────────────────────────────────────────────

    /// Create or update a Knowledge Object.
    ///
    /// - If a KO with the same id already exists its version is incremented.
    /// - Both the latest-version key and a history entry are written atomically
    ///   (two KV puts — WAL ensures both are durable or neither is).
    /// - Index entries (kind, tags) are kept in sync.
    #[instrument(skip(self, ko), fields(id = %ko.id, ns = %ko.namespace, kind = %ko.kind))]
    pub fn ko_put(&self, mut ko: KnowledgeObject) -> KernelResult<KnowledgeObject> {
        // If a previous version exists, increment the version counter and
        // remove old index entries before writing new ones.
        if let Some(prev) = self.ko_get_raw(&ko.namespace, &ko.id.as_str())? {
            if prev.is_deleted() {
                return Err(KernelError::Deleted(ko.id.to_string()));
            }
            ko.version = prev.version + 1;
            self.remove_indexes(&prev)?;
        }

        ko.tx_time = Utc::now();

        let bytes = ko.to_bytes().map_err(KernelError::from)?;
        let tx_ns = ko.tx_time.timestamp_nanos_opt().unwrap_or(0);

        // Write latest version
        self.kv.put(
            index::ko_key(&ko.namespace, &ko.id.as_str()),
            bytes.clone(),
        )?;

        // Write history entry
        self.kv.put(
            index::ko_history_key(&ko.namespace, &ko.id.as_str(), tx_ns),
            bytes,
        )?;

        // Write index entries
        self.write_indexes(&ko)?;

        debug!(version = ko.version, "ko_put complete");
        Ok(ko)
    }

    // ── Syscall 2: ko_get ─────────────────────────────────────────────────

    /// Fetch the latest active version of a KO by id.
    pub fn ko_get(&self, namespace: &str, id: &str) -> KernelResult<KnowledgeObject> {
        let ko = self.ko_get_raw(namespace, id)?
            .ok_or_else(|| KernelError::NotFound(format!("{namespace}/{id}")))?;
        if ko.is_deleted() {
            return Err(KernelError::Deleted(id.to_string()));
        }
        Ok(ko)
    }

    // ── Syscall 3: ko_delete ──────────────────────────────────────────────

    /// Soft-delete a KO by writing a tombstone version.
    /// The KO is retained in history; only the latest-version lookup is marked deleted.
    #[instrument(skip(self), fields(ns = namespace, id = id))]
    pub fn ko_delete(&self, namespace: &str, id: &str) -> KernelResult<()> {
        let mut ko = self.ko_get_raw(namespace, id)?
            .ok_or_else(|| KernelError::NotFound(format!("{namespace}/{id}")))?;

        if ko.is_deleted() {
            return Err(KernelError::Deleted(id.to_string()));
        }

        self.remove_indexes(&ko)?;

        ko.version += 1;
        ko.tx_time = Utc::now();
        ko.status = KoStatus::Deleted;

        let bytes = ko.to_bytes().map_err(KernelError::from)?;
        let tx_ns = ko.tx_time.timestamp_nanos_opt().unwrap_or(0);

        self.kv.put(index::ko_key(namespace, id), bytes.clone())?;
        self.kv.put(index::ko_history_key(namespace, id, tx_ns), bytes)?;

        debug!("ko_delete complete");
        Ok(())
    }

    // ── Syscall 4: ko_scan ────────────────────────────────────────────────

    /// Scan active KOs in a namespace, optionally filtered by kind.
    /// Returns KOs in key order (which is creation-time order for UUID v7 ids).
    pub fn ko_scan(
        &self,
        namespace: &str,
        kind: Option<&str>,
        limit: Option<usize>,
    ) -> KernelResult<Vec<KnowledgeObject>> {
        let pd = ProjectionDispatcher::new(self.kv.clone());

        let entries = if let Some(k) = kind {
            // Use the kind index for efficient filtered scan
            let req = {
                let mut r = ScanRequest::prefix(index::idx_kind_prefix(namespace, k));
                if let Some(n) = limit { r = r.with_limit(n); }
                r
            };
            let ids: Vec<String> = pd
                .keys(req)?
                .into_iter()
                .filter_map(|key| index::id_from_index_key(&key).map(|s| s.to_string()))
                .collect();

            let mut kos = Vec::with_capacity(ids.len());
            for id in ids {
                if let Ok(ko) = self.ko_get(namespace, &id) {
                    kos.push(ko);
                }
            }
            kos
        } else {
            // Full namespace scan via primary key prefix
            let req = {
                let mut r = ScanRequest::prefix(index::ko_namespace_prefix(namespace));
                if let Some(n) = limit { r = r.with_limit(n); }
                r
            };
            pd.scan(req)?
                .into_iter()
                .filter_map(|e| {
                    KnowledgeObject::from_bytes(&e.value).ok()
                })
                .filter(|ko| !ko.is_deleted())
                .collect()
        };

        Ok(entries)
    }

    // ── Syscall 5: ko_history ─────────────────────────────────────────────

    /// Return all recorded versions of a KO, oldest first (ascending tx_time).
    pub fn ko_history(
        &self,
        namespace: &str,
        id: &str,
    ) -> KernelResult<Vec<KnowledgeObject>> {
        let pd = ProjectionDispatcher::new(self.kv.clone());
        let req = ScanRequest::prefix(index::ko_history_prefix(namespace, id));

        let versions: Vec<KnowledgeObject> = pd
            .scan(req)?
            .into_iter()
            .filter_map(|e| KnowledgeObject::from_bytes(&e.value).ok())
            .collect();

        if versions.is_empty() {
            return Err(KernelError::NotFound(format!("{namespace}/{id}")));
        }
        Ok(versions)
    }

    // ── Syscall 6: ko_query_tag ───────────────────────────────────────────

    /// Find all active KOs in a namespace that carry a specific tag.
    pub fn ko_query_tag(
        &self,
        namespace: &str,
        tag: &str,
        limit: Option<usize>,
    ) -> KernelResult<Vec<KnowledgeObject>> {
        let pd = ProjectionDispatcher::new(self.kv.clone());
        let req = {
            let mut r = ScanRequest::prefix(index::idx_tag_prefix(namespace, tag));
            if let Some(n) = limit { r = r.with_limit(n); }
            r
        };

        let ids: Vec<String> = pd
            .keys(req)?
            .into_iter()
            .filter_map(|key| index::id_from_index_key(&key).map(|s| s.to_string()))
            .collect();

        let mut kos = Vec::with_capacity(ids.len());
        for id in ids {
            if let Ok(ko) = self.ko_get(namespace, &id) {
                kos.push(ko);
            }
        }
        Ok(kos)
    }

    // ── Internal helpers ──────────────────────────────────────────────────

    fn ko_get_raw(&self, namespace: &str, id: &str) -> KernelResult<Option<KnowledgeObject>> {
        match self.kv.get(index::ko_key(namespace, id))? {
            None => Ok(None),
            Some(bytes) => Ok(Some(
                KnowledgeObject::from_bytes(&bytes).map_err(KernelError::from)?
            )),
        }
    }

    fn write_indexes(&self, ko: &KnowledgeObject) -> KernelResult<()> {
        self.kv.put(
            index::idx_kind_key(&ko.namespace, ko.kind.as_str(), &ko.id.as_str()),
            Bytes::new(),
        )?;
        for tag in &ko.tags {
            self.kv.put(
                index::idx_tag_key(&ko.namespace, tag, &ko.id.as_str()),
                Bytes::new(),
            )?;
        }
        Ok(())
    }

    fn remove_indexes(&self, ko: &KnowledgeObject) -> KernelResult<()> {
        self.kv.delete(
            index::idx_kind_key(&ko.namespace, ko.kind.as_str(), &ko.id.as_str()),
        )?;
        for tag in &ko.tags {
            self.kv.delete(
                index::idx_tag_key(&ko.namespace, tag, &ko.id.as_str()),
            )?;
        }
        Ok(())
    }
}
