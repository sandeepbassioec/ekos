//! `vectorvolt-projection` — CQRS read-model projections for VectorVolt (VV-PRJ-003).
//!
//! ## Quick start
//!
//! ```rust
//! use vectorvolt_projection::{ProjectionStore, Reducer};
//! use vectorvolt_txn::Mutation;
//!
//! struct CountReducer { count: u64 }
//!
//! impl Reducer for CountReducer {
//!     fn name(&self) -> &str { "count" }
//!     fn apply(&mut self, m: &Mutation) {
//!         if matches!(m, Mutation::Insert { .. }) { self.count += 1; }
//!     }
//!     fn snapshot(&self) -> Vec<(u64, Vec<u64>)> {
//!         vec![(0, vec![self.count])]
//!     }
//! }
//!
//! let mut store = ProjectionStore::new();
//! store.attach(Box::new(CountReducer { count: 0 }));
//! store.fan_out(&Mutation::Insert { pk: 1, values: vec![42] });
//! assert_eq!(store.query("count").unwrap()[0].1[0], 1);
//! ```

pub mod projection;
pub mod reducer;

pub use projection::ProjectionStore;
pub use reducer::Reducer;
