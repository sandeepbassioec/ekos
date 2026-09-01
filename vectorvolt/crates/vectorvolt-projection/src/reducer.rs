//! [`Reducer`] — the single trait that a CQRS read-model implements.

use vectorvolt_txn::Mutation;

/// A stateful function that folds [`Mutation`]s into a read model.
///
/// Implement this to build any projection — last-value store, counter,
/// aggregate, materialized view, etc.  Implementations are held behind
/// `Box<dyn Reducer>` inside [`crate::ProjectionStore`].
pub trait Reducer: Send + Sync {
    /// Unique identifier for this projection.
    fn name(&self) -> &str;

    /// Apply one mutation to the read model.
    fn apply(&mut self, mutation: &Mutation);

    /// Return a snapshot of the current read model as `(pk, values)` rows.
    fn snapshot(&self) -> Vec<(u64, Vec<u64>)>;
}
