//! CQRS projection management for [`TableEngine`] (VV-PRJ-004).
//!
//! Mutations written via `insert` and `delete` are automatically fanned to all
//! attached projections.  Use `attach_projection` to register a [`Reducer`] and
//! `query_projection` to read its current state.

use vectorvolt_projection::Reducer;

use crate::engine::TableEngine;

impl TableEngine {
    /// Register a reducer.  Mutations from this point forward are fanned to it.
    pub fn attach_projection(&self, reducer: Box<dyn Reducer>) {
        self.projections.lock().attach(reducer);
    }

    /// Remove a reducer by name.  Returns `true` if it was registered.
    pub fn detach_projection(&self, name: &str) -> bool {
        self.projections.lock().detach(name)
    }

    /// Query the current snapshot of a named projection.
    pub fn query_projection(&self, name: &str) -> Option<Vec<(u64, Vec<u64>)>> {
        self.projections.lock().query(name)
    }

    /// Names of all registered projections.
    pub fn projection_names(&self) -> Vec<String> {
        self.projections.lock().projection_names()
    }

    pub fn projection_count(&self) -> usize {
        self.projections.lock().len()
    }
}
