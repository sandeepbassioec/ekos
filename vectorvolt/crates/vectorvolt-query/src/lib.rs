//! `vectorvolt-query` — text-based query DSL for VectorVolt.
//!
//! ## Syntax
//!
//! | Statement | Example |
//! |-----------|---------|
//! | `GET <pk>` | `GET 42` |
//! | `INSERT <pk> <v1> [v2 ...]` | `INSERT 1 1 25 99` |
//! | `DELETE <pk>` | `DELETE 7` |
//! | `SCAN col=<n> lo=<lo> hi=<hi>` | `SCAN col=1 lo=20 hi=30` |
//! | `SECONDARY col=<n> val=<v>` | `SECONDARY col=1 val=55` |
//! | `BEGIN` | starts a transaction, returns `TxnId` |
//! | `COMMIT <txn_id>` | `COMMIT 0` |
//! | `ROLLBACK <txn_id>` | `ROLLBACK 0` |
//! | `GET TXN <txn_id> <pk>` | `GET TXN 0 42` |
//! | `INSERT TXN <txn_id> <pk> <v1> ...` | `INSERT TXN 0 1 1 25` |
//! | `DELETE TXN <txn_id> <pk>` | `DELETE TXN 0 7` |
//! | `DESCRIBE` | returns schema column names |
//! | `STATS` | returns engine statistics |
//! | `HISTOGRAM col=<n> lo=<lo> hi=<hi> buckets=<b>` | `HISTOGRAM col=1 lo=0 hi=100 buckets=5` |
//! | `PERCENTILE col=<n> lo=<lo> hi=<hi> p=<0-100>` | `PERCENTILE col=1 lo=0 hi=100 p=95` |

pub mod catalog;
pub mod engine;
pub mod row;

mod error;
mod executor;
mod lexer;
mod parser;

pub use engine::QueryEngine;
pub use error::{QueryError, QueryResult};
pub use executor::{QueryResponse, execute, run};
pub use parser::{Query, parse};
