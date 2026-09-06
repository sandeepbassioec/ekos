//! Named read model registry with lifecycle management (VV-PRJ-014).

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use serde_json::Value;
use vectorvolt_txn::Mutation;

use crate::cache::{ReadCache, ReadModelSnapshot};
use crate::reducer::SemanticReducer;

/// Named registration of a read model.
struct ReadModelEntry {
    reducer: Box<dyn SemanticReducer>,
    cache: Option<Arc<dyn ReadCache>>,
    #[allow(dead_code)]
    group_by_col: Option<String>,
    running: bool,
}

/// Registry of named read models (VV-PRJ-014).
///
/// Thread-safe: holds an inner Mutex.  In production, wrap in `Arc<ReadModelRegistry>`.
pub struct ReadModelRegistry {
    models: Mutex<HashMap<String, ReadModelEntry>>,
}

impl ReadModelRegistry {
    pub fn new() -> Self {
        Self {
            models: Mutex::new(HashMap::new()),
        }
    }

    /// Register a read model with an optional cache layer.
    pub fn register(
        &self,
        reducer: Box<dyn SemanticReducer>,
        cache: Option<Arc<dyn ReadCache>>,
        group_by_col: Option<String>,
    ) {
        let name = reducer.name().to_string();
        let entry = ReadModelEntry {
            reducer,
            cache,
            group_by_col,
            running: true,
        };
        self.models.lock().unwrap().insert(name, entry);
    }

    /// Deregister a read model (drops its reducer and cache).
    pub fn deregister(&self, name: &str) -> bool {
        self.models.lock().unwrap().remove(name).is_some()
    }

    /// Stop a read model without deregistering it.
    pub fn stop(&self, name: &str) {
        if let Some(e) = self.models.lock().unwrap().get_mut(name) {
            e.running = false;
        }
    }

    /// Resume a stopped read model.
    pub fn start(&self, name: &str) {
        if let Some(e) = self.models.lock().unwrap().get_mut(name) {
            e.running = true;
        }
    }

    /// Fan a mutation to the named model (if running).
    /// `group_key` is the partition/group value for this row.
    pub fn apply(&self, model_name: &str, group_key: &str, mutation: &Mutation) {
        let mut guard = self.models.lock().unwrap();
        if let Some(e) = guard.get_mut(model_name) {
            if !e.running {
                return;
            }
            if let Err(err) = e.reducer.apply(group_key, mutation) {
                tracing::warn!("ReadModelRegistry apply error [{model_name}]: {err}");
            }
            if let Some(cache) = &e.cache {
                cache.invalidate(group_key);
            }
        }
    }

    /// Fan a mutation to ALL running models.
    pub fn fan_out(&self, group_key: &str, mutation: &Mutation) {
        let mut guard = self.models.lock().unwrap();
        for (name, e) in guard.iter_mut() {
            if !e.running {
                continue;
            }
            if let Err(err) = e.reducer.apply(group_key, mutation) {
                tracing::warn!("ReadModelRegistry fan_out error [{name}]: {err}");
            }
            if let Some(cache) = &e.cache {
                cache.invalidate(group_key);
            }
        }
    }

    /// Query a named model, checking cache first.
    pub fn query(&self, model_name: &str) -> Option<Arc<ReadModelSnapshot>> {
        let guard = self.models.lock().unwrap();
        let e = guard.get(model_name)?;
        let snap: ReadModelSnapshot = e.reducer.snapshot();
        let arc = Arc::new(snap);
        if let Some(cache) = &e.cache {
            cache.set("__all__", Arc::clone(&arc));
        }
        Some(arc)
    }

    /// Query a named model for a specific group key, checking cache first.
    pub fn query_key(&self, model_name: &str, group_key: &str) -> Option<Value> {
        let guard = self.models.lock().unwrap();
        let e = guard.get(model_name)?;

        if let Some(cache) = &e.cache
            && let Some(snap) = cache.get(group_key)
        {
            return snap.get(group_key).cloned();
        }

        let snap = e.reducer.snapshot();
        let val = snap.get(group_key).cloned();

        if let (Some(cache), Some(v)) = (&e.cache, &val) {
            let mut single: ReadModelSnapshot = HashMap::new();
            single.insert(group_key.to_string(), v.clone());
            cache.set(group_key, Arc::new(single));
        }
        val
    }

    /// Sorted list of registered model names.
    pub fn model_names(&self) -> Vec<String> {
        let mut names: Vec<String> = self.models.lock().unwrap().keys().cloned().collect();
        names.sort();
        names
    }

    pub fn len(&self) -> usize {
        self.models.lock().unwrap().len()
    }

    pub fn is_empty(&self) -> bool {
        self.models.lock().unwrap().is_empty()
    }
}

impl Default for ReadModelRegistry {
    fn default() -> Self {
        Self::new()
    }
}
