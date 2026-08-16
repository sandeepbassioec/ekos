//! Execute a [`Query`] against a [`TableEngine`], returning a [`QueryResponse`].

use vectorvolt_engine::{EngineRow, EngineStats, HistogramBucket, PageResult, TableEngine, TxnId};

use crate::error::QueryResult;
use crate::parser::Query;

/// The result of executing one query.
#[derive(Debug)]
pub enum QueryResponse {
    /// A single row was returned (GET, GET TXN).
    Row(EngineRow),

    /// Multiple rows (SCAN, SECONDARY, SCAN SORTED).
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

    /// Scalar count result (COUNT, DELETE RANGE).
    Count(u64),

    /// Scalar aggregate result (SUM, MIN, MAX, AVG — None when no rows match).
    Aggregate(Option<u64>),

    /// Paginated result from SCAN PAGE.
    Page(PageResult),

    // ── Layer 50 ──────────────────────────────────────────────────────────────
    /// Histogram buckets from HISTOGRAM.
    Histogram(Vec<HistogramBucket>),

    /// Percentile value from PERCENTILE (None when no rows matched).
    Percentile(Option<u64>),

    // ── Layer 51 ──────────────────────────────────────────────────────────────
    /// Group-by result: Vec of (group_val, agg_val) pairs.
    GroupResult(Vec<(u64, u64)>),
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

        // ── Layer 43: analytical ──────────────────────────────────────────────
        Query::Count { col, lo, hi } => {
            let n = engine.count_range(col, lo, hi)?;
            Ok(QueryResponse::Count(n))
        }

        Query::Sum { col, lo, hi } => {
            let v = engine.sum_range(col, lo, hi)?;
            Ok(QueryResponse::Aggregate(Some(v)))
        }

        Query::Min { col, lo, hi } => {
            let v = engine.min_range(col, lo, hi)?;
            Ok(QueryResponse::Aggregate(v))
        }

        Query::Max { col, lo, hi } => {
            let v = engine.max_range(col, lo, hi)?;
            Ok(QueryResponse::Aggregate(v))
        }

        Query::Avg { col, lo, hi } => {
            let v = engine.avg_range(col, lo, hi)?.map(|f| f.round() as u64);
            Ok(QueryResponse::Aggregate(v))
        }

        Query::ScanSorted {
            col,
            lo,
            hi,
            sort,
            asc,
            limit,
        } => {
            let rows = engine.scan_sorted(col, lo, hi, sort, asc, limit)?;
            Ok(QueryResponse::Rows(rows))
        }

        Query::ScanPage {
            col,
            lo,
            hi,
            after,
            limit,
        } => {
            let page = engine.scan_page(col, lo, hi, after, limit)?;
            Ok(QueryResponse::Page(page))
        }

        // ── Layer 43: write/mutate ────────────────────────────────────────────
        Query::Upsert { pk, values } => {
            engine.upsert(pk, values)?;
            Ok(QueryResponse::Ok)
        }

        Query::UpdateField { pk, col, val } => {
            engine.update_field(pk, col, val)?;
            Ok(QueryResponse::Ok)
        }

        Query::DeleteRange { col, lo, hi } => {
            let n = engine.delete_range(col, lo, hi)?;
            Ok(QueryResponse::Count(n as u64))
        }

        // ── Layer 50: histogram + percentile ─────────────────────────────────
        Query::Histogram {
            col,
            lo,
            hi,
            buckets,
        } => {
            let buckets = engine.histogram_range(col, lo, hi, buckets)?;
            Ok(QueryResponse::Histogram(buckets))
        }

        Query::Percentile { col, lo, hi, p } => {
            let v = engine.percentile_range(col, lo, hi, p as f64)?;
            Ok(QueryResponse::Percentile(v))
        }

        // ── Layer 51: group-by aggregates ─────────────────────────────────────
        Query::GroupCount {
            filter_col,
            lo,
            hi,
            group_col,
        } => {
            let pairs = engine.group_by_count(filter_col, lo, hi, group_col)?;
            Ok(QueryResponse::GroupResult(pairs))
        }

        Query::GroupSum {
            filter_col,
            lo,
            hi,
            group_col,
            agg_col,
        } => {
            let pairs = engine.group_by_sum(filter_col, lo, hi, group_col, agg_col)?;
            Ok(QueryResponse::GroupResult(pairs))
        }

        Query::GroupMin {
            filter_col,
            lo,
            hi,
            group_col,
            agg_col,
        } => {
            let pairs = engine.group_by_min(filter_col, lo, hi, group_col, agg_col)?;
            Ok(QueryResponse::GroupResult(pairs))
        }

        Query::GroupMax {
            filter_col,
            lo,
            hi,
            group_col,
            agg_col,
        } => {
            let pairs = engine.group_by_max(filter_col, lo, hi, group_col, agg_col)?;
            Ok(QueryResponse::GroupResult(pairs))
        }
    }
}

/// Parse and execute a query string in one step.
pub fn run(engine: &TableEngine, input: &str) -> QueryResult<QueryResponse> {
    let query = crate::parser::parse(input)?;
    execute(engine, query)
}
