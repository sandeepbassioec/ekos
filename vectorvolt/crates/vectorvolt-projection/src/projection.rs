//! [`ProjectionStore`] — registry that fans WAL mutations to attached reducers.

use std::collections::HashMap;

use vectorvolt_txn::Mutation;

use crate::reducer::Reducer;

/// Holds zero or more named [`Reducer`]s and fans each mutation to all of them.
///
/// The store is not thread-safe by itself; wrap it in a `Mutex` when used from
/// multiple threads (as [`vectorvolt_engine`] does via `parking_lot::Mutex`).
pub struct ProjectionStore {
    projections: HashMap<String, Box<dyn Reducer>>,
}

impl ProjectionStore {
    pub fn new() -> Self {
        Self {
            projections: HashMap::new(),
        }
    }

    /// Register a reducer.  Replaces any existing reducer with the same name.
    pub fn attach(&mut self, reducer: Box<dyn Reducer>) {
        let name = reducer.name().to_string();
        self.projections.insert(name, reducer);
    }

    /// Remove the reducer with `name`.  Returns `true` if it existed.
    pub fn detach(&mut self, name: &str) -> bool {
        self.projections.remove(name).is_some()
    }

    /// Fan one mutation to every attached reducer.
    pub fn fan_out(&mut self, mutation: &Mutation) {
        for reducer in self.projections.values_mut() {
            reducer.apply(mutation);
        }
    }

    /// Query the snapshot of a named reducer.
    pub fn query(&self, name: &str) -> Option<Vec<(u64, Vec<u64>)>> {
        self.projections.get(name).map(|r| r.snapshot())
    }

    /// Names of all registered projections.
    pub fn projection_names(&self) -> Vec<String> {
        let mut names: Vec<String> = self.projections.keys().cloned().collect();
        names.sort();
        names
    }

    pub fn len(&self) -> usize {
        self.projections.len()
    }

    pub fn is_empty(&self) -> bool {
        self.projections.is_empty()
    }
}

impl Default for ProjectionStore {
    fn default() -> Self {
        Self::new()
    }
}

// ── tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use vectorvolt_txn::Mutation;

    struct LastValueReducer {
        name: String,
        rows: HashMap<u64, Vec<u64>>,
    }

    impl LastValueReducer {
        fn new(name: &str) -> Box<dyn Reducer> {
            Box::new(Self {
                name: name.to_string(),
                rows: HashMap::new(),
            })
        }
    }

    impl Reducer for LastValueReducer {
        fn name(&self) -> &str {
            &self.name
        }

        fn apply(&mut self, mutation: &Mutation) {
            match mutation {
                Mutation::Insert { pk, values } | Mutation::Upsert { pk, values } => {
                    self.rows.insert(*pk, values.clone());
                }
                Mutation::Delete { pk } => {
                    self.rows.remove(pk);
                }
                _ => {}
            }
        }

        fn snapshot(&self) -> Vec<(u64, Vec<u64>)> {
            let mut v: Vec<_> = self.rows.iter().map(|(&pk, v)| (pk, v.clone())).collect();
            v.sort_by_key(|(pk, _)| *pk);
            v
        }
    }

    #[test]
    fn new_store_is_empty() {
        let store = ProjectionStore::new();
        assert!(store.is_empty());
    }

    #[test]
    fn attach_adds_reducer() {
        let mut store = ProjectionStore::new();
        store.attach(LastValueReducer::new("test"));
        assert_eq!(store.len(), 1);
    }

    #[test]
    fn detach_removes_reducer() {
        let mut store = ProjectionStore::new();
        store.attach(LastValueReducer::new("test"));
        assert!(store.detach("test"));
        assert!(store.is_empty());
    }

    #[test]
    fn detach_missing_returns_false() {
        let mut store = ProjectionStore::new();
        assert!(!store.detach("nope"));
    }

    #[test]
    fn fan_out_insert_visible_in_snapshot() {
        let mut store = ProjectionStore::new();
        store.attach(LastValueReducer::new("r"));
        store.fan_out(&Mutation::Insert {
            pk: 1,
            values: vec![10, 20],
        });
        let snap = store.query("r").unwrap();
        assert_eq!(snap.len(), 1);
        assert_eq!(snap[0], (1, vec![10, 20]));
    }

    #[test]
    fn fan_out_delete_removes_from_snapshot() {
        let mut store = ProjectionStore::new();
        store.attach(LastValueReducer::new("r"));
        store.fan_out(&Mutation::Insert {
            pk: 1,
            values: vec![10],
        });
        store.fan_out(&Mutation::Delete { pk: 1 });
        let snap = store.query("r").unwrap();
        assert!(snap.is_empty());
    }

    #[test]
    fn query_unknown_returns_none() {
        let store = ProjectionStore::new();
        assert!(store.query("missing").is_none());
    }

    #[test]
    fn projection_names_sorted() {
        let mut store = ProjectionStore::new();
        store.attach(LastValueReducer::new("z"));
        store.attach(LastValueReducer::new("a"));
        let names = store.projection_names();
        assert_eq!(names, vec!["a", "z"]);
    }

    #[test]
    fn fan_out_to_multiple_reducers() {
        let mut store = ProjectionStore::new();
        store.attach(LastValueReducer::new("r1"));
        store.attach(LastValueReducer::new("r2"));
        store.fan_out(&Mutation::Insert {
            pk: 5,
            values: vec![99],
        });
        assert_eq!(store.query("r1").unwrap().len(), 1);
        assert_eq!(store.query("r2").unwrap().len(), 1);
    }

    #[test]
    fn attach_replaces_existing() {
        let mut store = ProjectionStore::new();
        store.attach(LastValueReducer::new("r"));
        store.fan_out(&Mutation::Insert {
            pk: 1,
            values: vec![1],
        });
        store.attach(LastValueReducer::new("r")); // fresh reducer
        let snap = store.query("r").unwrap();
        assert!(snap.is_empty(), "replaced reducer should start fresh");
    }

    #[test]
    fn upsert_updates_existing() {
        let mut store = ProjectionStore::new();
        store.attach(LastValueReducer::new("r"));
        store.fan_out(&Mutation::Insert {
            pk: 1,
            values: vec![10],
        });
        store.fan_out(&Mutation::Upsert {
            pk: 1,
            values: vec![99],
        });
        let snap = store.query("r").unwrap();
        assert_eq!(snap[0].1, vec![99]);
    }

    #[test]
    fn multiple_inserts_separate_rows() {
        let mut store = ProjectionStore::new();
        store.attach(LastValueReducer::new("r"));
        for i in 0..5u64 {
            store.fan_out(&Mutation::Insert {
                pk: i,
                values: vec![i * 10],
            });
        }
        assert_eq!(store.query("r").unwrap().len(), 5);
    }

    #[test]
    fn default_impl_same_as_new() {
        let store: ProjectionStore = Default::default();
        assert!(store.is_empty());
    }
}
