//! Execute a [`Query`] against a [`TableEngine`], returning a [`QueryResponse`].

use vectorvolt_engine::{EngineRow, EngineStats, TableEngine, TxnId};

use crate::error::QueryResult;
use crate::parser::Query;

/// The result of executing one query.
#[derive(Debug)]
pub enum QueryResponse {
    /// A single row was returned (GET, GET TXN).
    Row(EngineRow),

    /// Multiple rows (SCAN, SECONDARY).
    Rows(Vec<EngineRow>),

    /// A transaction was opened.
    TxnStarted(TxnId),

    /// Table schema column names.
    Schema(Vec<String>),

    /// Engine statistics snapshot.
    Stats(EngineStats),

    /// Operation succeeded with no data to return.
    Ok,

    /// The requested row was not found.
    NotFound,
}

/// Execute `query` against `engine`.
pub fn execute(engine: &TableEngine, query: Query) -> QueryResult<QueryResponse> {
    match query {
        Query::Get { pk } => match engine.get(pk)? {
            Some(row) => Ok(QueryResponse::Row(row)),
            None => Ok(QueryResponse::NotFound),
        },

        Query::Insert { pk, values } => {
            engine.insert(pk, values)?;
            Ok(QueryResponse::Ok)
        }

        Query::Delete { pk } => {
            engine.delete(pk)?;
            Ok(QueryResponse::Ok)
        }

        Query::Scan { col, lo, hi } => {
            let rows = engine.scan(col, lo, hi)?;
            Ok(QueryResponse::Rows(rows))
        }

        Query::Secondary { col, val } => {
            let vecs = engine.secondary_scan(col, val)?;
            let rows: Vec<EngineRow> = vecs
                .into_iter()
                .map(|values| {
                    let pk = values.first().copied().unwrap_or(0);
                    EngineRow { pk, values }
                })
                .collect();
            Ok(QueryResponse::Rows(rows))
        }

        Query::Begin => {
            let id = engine.begin();
            Ok(QueryResponse::TxnStarted(id))
        }

        Query::Commit { txn_id } => {
            engine.commit_txn(txn_id)?;
            Ok(QueryResponse::Ok)
        }

        Query::Rollback { txn_id } => {
            engine.rollback_txn(txn_id)?;
            Ok(QueryResponse::Ok)
        }

        Query::GetTxn { txn_id, pk } => match engine.get_txn(txn_id, pk)? {
            Some(row) => Ok(QueryResponse::Row(row)),
            None => Ok(QueryResponse::NotFound),
        },

        Query::InsertTxn { txn_id, pk, values } => {
            engine.insert_txn(txn_id, pk, values)?;
            Ok(QueryResponse::Ok)
        }

        Query::DeleteTxn { txn_id, pk } => {
            engine.delete_txn(txn_id, pk)?;
            Ok(QueryResponse::Ok)
        }

        Query::Describe => Ok(QueryResponse::Schema(engine.schema.clone())),

        Query::Stats => Ok(QueryResponse::Stats(engine.stats())),
    }
}

/// Parse and execute a query string in one step.
pub fn run(engine: &TableEngine, input: &str) -> QueryResult<QueryResponse> {
    let query = crate::parser::parse(input)?;
    execute(engine, query)
}
