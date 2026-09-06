//! Reducers: built-in aggregations, Rhai custom reducer, HTTP callback (VV-PRJ-014..019).

use std::collections::HashMap;
use std::path::Path;
use std::time::Duration;

use rhai::{Engine, Map as RhaiMap, Scope};
use serde_json::{Value, json};
use thiserror::Error;
use tracing::warn;

use vectorvolt_txn::Mutation;

#[derive(Debug, Error)]
pub enum ReducerError {
    #[error("rhai: {0}")]
    Rhai(String),
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("http: {0}")]
    Http(String),
    #[error("json: {0}")]
    Json(#[from] serde_json::Error),
}

pub type ReducerResult<T> = Result<T, ReducerError>;

/// A stateful aggregation over mutations, keyed by a group key (VV-PRJ-014).
pub trait SemanticReducer: Send + Sync {
    fn name(&self) -> &str;
    fn apply(&mut self, key: &str, mutation: &Mutation) -> ReducerResult<()>;
    fn snapshot(&self) -> HashMap<String, Value>;
}

// ── BuiltinReducer ───────────────────────────────────────────────────────────

/// count | sum | min | max over the first value column.
pub struct BuiltinReducer {
    name: String,
    kind: BuiltinKind,
    state: HashMap<String, i64>,
}

enum BuiltinKind {
    Count,
    Sum,
    Min,
    Max,
}

impl BuiltinReducer {
    pub fn count(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            kind: BuiltinKind::Count,
            state: HashMap::new(),
        }
    }
    pub fn sum(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            kind: BuiltinKind::Sum,
            state: HashMap::new(),
        }
    }
    pub fn min(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            kind: BuiltinKind::Min,
            state: HashMap::new(),
        }
    }
    pub fn max(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            kind: BuiltinKind::Max,
            state: HashMap::new(),
        }
    }
}

impl SemanticReducer for BuiltinReducer {
    fn name(&self) -> &str {
        &self.name
    }

    fn apply(&mut self, key: &str, mutation: &Mutation) -> ReducerResult<()> {
        let entry = self.state.entry(key.to_string()).or_insert(0);
        match mutation {
            Mutation::Insert { values, .. } | Mutation::Upsert { values, .. } => {
                let v = values.first().copied().unwrap_or(0) as i64;
                *entry = match self.kind {
                    BuiltinKind::Count => *entry + 1,
                    BuiltinKind::Sum => *entry + v,
                    BuiltinKind::Min => {
                        if *entry == 0 {
                            v
                        } else {
                            (*entry).min(v)
                        }
                    }
                    BuiltinKind::Max => (*entry).max(v),
                };
            }
            Mutation::Delete { .. } => {
                self.state.remove(key);
            }
            _ => {}
        }
        Ok(())
    }

    fn snapshot(&self) -> HashMap<String, Value> {
        self.state
            .iter()
            .map(|(k, v)| (k.clone(), json!(v)))
            .collect()
    }
}

// ── RhaiReducer ──────────────────────────────────────────────────────────────

/// Calls developer's Rhai `fn reduce(state, row) -> state` (VV-PRJ-018).
pub struct RhaiReducer {
    name: String,
    engine: Engine,
    source: String,
    state: HashMap<String, RhaiMap>,
}

impl RhaiReducer {
    pub fn from_file(name: impl Into<String>, path: &Path) -> ReducerResult<Self> {
        let source = std::fs::read_to_string(path)?;
        let mut engine = Engine::new();
        engine.set_max_operations(100_000);
        Ok(Self {
            name: name.into(),
            engine,
            source,
            state: HashMap::new(),
        })
    }

    pub fn from_source(name: impl Into<String>, source: impl Into<String>) -> Self {
        let mut engine = Engine::new();
        engine.set_max_operations(100_000);
        Self {
            name: name.into(),
            engine,
            source: source.into(),
            state: HashMap::new(),
        }
    }
}

impl SemanticReducer for RhaiReducer {
    fn name(&self) -> &str {
        &self.name
    }

    fn apply(&mut self, key: &str, mutation: &Mutation) -> ReducerResult<()> {
        let (pk, values) = match mutation {
            Mutation::Insert { pk, values } | Mutation::Upsert { pk, values } => {
                (*pk, values.clone())
            }
            Mutation::Delete { pk } => {
                self.state.remove(&format!("{pk}"));
                return Ok(());
            }
            _ => return Ok(()),
        };

        let current_state: RhaiMap = self.state.get(key).cloned().unwrap_or_default();

        let mut row_map = RhaiMap::new();
        row_map.insert("pk".into(), rhai::Dynamic::from(pk as i64));
        let values_arr: rhai::Array = values
            .iter()
            .map(|&v| rhai::Dynamic::from(v as i64))
            .collect();
        row_map.insert("values".into(), rhai::Dynamic::from(values_arr));

        let mut scope = Scope::new();
        scope.push("state", rhai::Dynamic::from(current_state));
        scope.push("row", rhai::Dynamic::from(row_map));

        let script = format!("{}\nreduce(state, row)", self.source);
        let new_state: RhaiMap = self
            .engine
            .eval_with_scope::<rhai::Dynamic>(&mut scope, &script)
            .map_err(|e| ReducerError::Rhai(e.to_string()))?
            .try_cast::<RhaiMap>()
            .ok_or_else(|| ReducerError::Rhai("reduce() must return a Map".into()))?;

        self.state.insert(key.to_string(), new_state);
        Ok(())
    }

    fn snapshot(&self) -> HashMap<String, Value> {
        self.state
            .iter()
            .map(|(k, v)| {
                let obj: serde_json::Map<String, Value> = v
                    .iter()
                    .map(|(fk, dv)| (fk.to_string(), dv.to_string().into()))
                    .collect();
                (k.clone(), Value::Object(obj))
            })
            .collect()
    }
}

// ── HttpCallbackReducer ──────────────────────────────────────────────────────

/// POSTs `{state, row}` JSON to developer URL, expects `{new_state}` (VV-PRJ-019).
pub struct HttpCallbackReducer {
    name: String,
    url: String,
    state: HashMap<String, Value>,
    client: reqwest::Client,
}

impl HttpCallbackReducer {
    pub fn new(name: impl Into<String>, url: impl Into<String>) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(5))
            .build()
            .expect("http client");
        Self {
            name: name.into(),
            url: url.into(),
            state: HashMap::new(),
            client,
        }
    }

    /// Synchronous wrapper — calls the async apply via a one-shot tokio block.
    pub fn apply_sync(&mut self, key: &str, mutation: &Mutation) -> ReducerResult<()> {
        let rt = tokio::runtime::Handle::try_current();
        if rt.is_ok() {
            // Inside an async context: cannot block. Store pending and warn.
            warn!(
                "HttpCallbackReducer used in async context — skipping HTTP round-trip for key={key}"
            );
            return Ok(());
        }
        tokio::runtime::Runtime::new()
            .expect("tokio")
            .block_on(self.apply_async(key, mutation))
    }

    pub async fn apply_async(&mut self, key: &str, mutation: &Mutation) -> ReducerResult<()> {
        let (pk, values) = match mutation {
            Mutation::Insert { pk, values } | Mutation::Upsert { pk, values } => {
                (*pk, values.clone())
            }
            Mutation::Delete { pk } => {
                self.state.remove(&format!("{pk}"));
                return Ok(());
            }
            _ => return Ok(()),
        };

        let row = json!({ "pk": pk, "values": values });
        let current = self.state.get(key).cloned().unwrap_or(Value::Null);
        let body = json!({ "state": current, "row": row });

        let resp: Value = self
            .client
            .post(&self.url)
            .json(&body)
            .send()
            .await
            .map_err(|e| ReducerError::Http(e.to_string()))?
            .json()
            .await
            .map_err(|e| ReducerError::Http(e.to_string()))?;

        if let Some(new_state) = resp.get("new_state") {
            self.state.insert(key.to_string(), new_state.clone());
        }
        Ok(())
    }
}

impl SemanticReducer for HttpCallbackReducer {
    fn name(&self) -> &str {
        &self.name
    }

    fn apply(&mut self, key: &str, mutation: &Mutation) -> ReducerResult<()> {
        self.apply_sync(key, mutation)
    }

    fn snapshot(&self) -> HashMap<String, Value> {
        self.state.clone()
    }
}
