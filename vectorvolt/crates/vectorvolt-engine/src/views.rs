//! Layer 66 — Materialized Views over CDC.
//!
//! A [`MaterializedView`] subscribes to a table's CDC broadcast channel and
//! auto-maintains a group-by aggregate (count / sum / min / max) in memory.
//! The view is always consistent with the committed row set: inserts add
//! contributions, deletes remove them (via a per-pk row mirror), and upserts
//! are handled atomically as delete-then-insert.

use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;

use parking_lot::{Mutex, RwLock};
use serde::{Deserialize, Serialize};

use crate::cdc::{ChangeEvent, ChangeKind};
use crate::registry::TableRegistry;

// ── public types ──────────────────────────────────────────────────────────────

/// Which aggregate function the view computes per group key.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ViewKind {
    Count,
    Sum { agg_col: usize },
    Min { agg_col: usize },
    Max { agg_col: usize },
}

/// Immutable description of what a view computes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViewDefinition {
    pub name: String,
    pub table: String,
    /// Column index used to filter rows fed into this view.
    pub filter_col: usize,
    /// Inclusive lower bound on `filter_col`.
    pub lo: u64,
    /// Inclusive upper bound on `filter_col`.
    pub hi: u64,
    /// Column whose value becomes the group key.
    pub group_col: usize,
    pub kind: ViewKind,
}

/// One row in the view result set.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViewEntry {
    pub group_val: u64,
    pub agg_val: u64,
}

// ── internal state ────────────────────────────────────────────────────────────

struct ViewState {
    def: ViewDefinition,
    /// group_val → current aggregate value.
    aggregates: BTreeMap<u64, u64>,
    /// For min/max: group_val → multiset of agg values (value → count).
    multiset: HashMap<u64, BTreeMap<u64, u64>>,
    /// pk → row values at time of last insert (used to undo on delete/upsert).
    row_mirror: HashMap<u64, Vec<u64>>,
}

impl ViewState {
    fn new(def: ViewDefinition) -> Self {
        Self {
            def,
            aggregates: BTreeMap::new(),
            multiset: HashMap::new(),
            row_mirror: HashMap::new(),
        }
    }

    fn apply(&mut self, ev: ChangeEvent) {
        match ev.kind {
            ChangeKind::Insert => {
                if let Some(values) = ev.values {
                    self.on_insert(ev.pk, values);
                }
            }
            ChangeKind::Delete => self.on_delete(ev.pk),
        }
    }

    fn on_insert(&mut self, pk: u64, values: Vec<u64>) {
        // If pk already tracked (upsert/update), undo old contribution first.
        if let Some(old) = self.row_mirror.remove(&pk) {
            self.undo(&old);
        }
        let d = &self.def;
        let in_range = d.filter_col < values.len()
            && values[d.filter_col] >= d.lo
            && values[d.filter_col] <= d.hi;
        if !in_range {
            return;
        }
        if d.group_col >= values.len() {
            return;
        }
        let gv = values[d.group_col];
        match d.kind {
            ViewKind::Count => {
                *self.aggregates.entry(gv).or_insert(0) += 1;
            }
            ViewKind::Sum { agg_col } => {
                if agg_col < values.len() {
                    *self.aggregates.entry(gv).or_insert(0) += values[agg_col];
                }
            }
            ViewKind::Min { agg_col } => {
                if agg_col < values.len() {
                    let av = values[agg_col];
                    *self.multiset.entry(gv).or_default().entry(av).or_insert(0) += 1;
                    let min = *self.multiset[&gv].keys().next().unwrap();
                    self.aggregates.insert(gv, min);
                }
            }
            ViewKind::Max { agg_col } => {
                if agg_col < values.len() {
                    let av = values[agg_col];
                    *self.multiset.entry(gv).or_default().entry(av).or_insert(0) += 1;
                    let max = *self.multiset[&gv].keys().next_back().unwrap();
                    self.aggregates.insert(gv, max);
                }
            }
        }
        self.row_mirror.insert(pk, values);
    }

    fn on_delete(&mut self, pk: u64) {
        if let Some(old) = self.row_mirror.remove(&pk) {
            self.undo(&old);
        }
    }

    fn undo(&mut self, old_vals: &[u64]) {
        let d = &self.def;
        if d.filter_col >= old_vals.len() {
            return;
        }
        let in_range = old_vals[d.filter_col] >= d.lo && old_vals[d.filter_col] <= d.hi;
        if !in_range || d.group_col >= old_vals.len() {
            return;
        }
        let gv = old_vals[d.group_col];
        match d.kind {
            ViewKind::Count => {
                let e = self.aggregates.entry(gv).or_insert(0);
                *e = e.saturating_sub(1);
                if *e == 0 {
                    self.aggregates.remove(&gv);
                }
            }
            ViewKind::Sum { agg_col } => {
                if agg_col < old_vals.len() {
                    let e = self.aggregates.entry(gv).or_insert(0);
                    *e = e.saturating_sub(old_vals[agg_col]);
                    if *e == 0 {
                        self.aggregates.remove(&gv);
                    }
                }
            }
            ViewKind::Min { agg_col } | ViewKind::Max { agg_col } => {
                if agg_col < old_vals.len() {
                    let av = old_vals[agg_col];
                    if let Some(ms) = self.multiset.get_mut(&gv) {
                        if let Some(cnt) = ms.get_mut(&av) {
                            *cnt -= 1;
                            if *cnt == 0 {
                                ms.remove(&av);
                            }
                        }
                        if ms.is_empty() {
                            self.multiset.remove(&gv);
                            self.aggregates.remove(&gv);
                        } else {
                            let new_agg = match d.kind {
                                ViewKind::Min { .. } => *ms.keys().next().unwrap(),
                                ViewKind::Max { .. } => *ms.keys().next_back().unwrap(),
                                _ => unreachable!(),
                            };
                            self.aggregates.insert(gv, new_agg);
                        }
                    }
                }
            }
        }
    }

    fn entries(&self) -> Vec<ViewEntry> {
        self.aggregates
            .iter()
            .map(|(&g, &a)| ViewEntry {
                group_val: g,
                agg_val: a,
            })
            .collect()
    }
}

// ── live view (task + shared state) ──────────────────────────────────────────

struct LiveView {
    definition: ViewDefinition,
    state: Arc<RwLock<ViewState>>,
    _handle: tokio::task::JoinHandle<()>,
}

async fn run_view_task(
    state: Arc<RwLock<ViewState>>,
    mut rx: tokio::sync::broadcast::Receiver<ChangeEvent>,
) {
    loop {
        match rx.recv().await {
            Ok(ev) => state.write().apply(ev),
            Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => continue,
            Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
        }
    }
}

// ── registry ──────────────────────────────────────────────────────────────────

/// Global registry of all live materialized views.
pub struct ViewRegistry {
    inner: Mutex<HashMap<String, LiveView>>,
}

impl ViewRegistry {
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(HashMap::new()),
        }
    }

    /// Create a new materialized view.  Fails if the name is taken or the
    /// source table does not exist.
    ///
    /// When `backfill` is `true`, the table's CDC log is replayed into the
    /// view before it starts consuming live events, so the initial state
    /// reflects all committed mutations rather than only future ones.
    pub fn create(
        &self,
        def: ViewDefinition,
        registry: &Arc<TableRegistry>,
        backfill: bool,
    ) -> Result<(), String> {
        let mut inner = self.inner.lock();
        if inner.contains_key(&def.name) {
            return Err(format!("view '{}' already exists", def.name));
        }
        let eng = registry.get(&def.table).map_err(|e| e.to_string())?;
        let rx = eng.subscribe_changes();
        let state = Arc::new(RwLock::new(ViewState::new(def.clone())));
        if backfill {
            let history = eng.read_log_from(0).map_err(|e| e.to_string())?;
            let mut st = state.write();
            for ev in history {
                st.apply(ev);
            }
        }
        let state2 = Arc::clone(&state);
        let handle = tokio::spawn(run_view_task(state2, rx));
        inner.insert(
            def.name.clone(),
            LiveView {
                definition: def,
                state,
                _handle: handle,
            },
        );
        Ok(())
    }

    /// List all view definitions.
    pub fn list(&self) -> Vec<ViewDefinition> {
        self.inner
            .lock()
            .values()
            .map(|v| v.definition.clone())
            .collect()
    }

    /// Return the current aggregate entries for a named view, or `None` if
    /// the view does not exist.
    pub fn get_entries(&self, name: &str) -> Option<Vec<ViewEntry>> {
        let inner = self.inner.lock();
        let live = inner.get(name)?;
        Some(live.state.read().entries())
    }

    /// Drop a view.  Returns `true` if the view existed.
    pub fn drop_view(&self, name: &str) -> bool {
        self.inner.lock().remove(name).is_some()
    }
}

impl Default for ViewRegistry {
    fn default() -> Self {
        Self::new()
    }
}
