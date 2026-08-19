//! Execute a [`Query`] against a [`TableEngine`], returning a [`QueryResponse`].

use vectorvolt_engine::{
    EngineRow, EngineStats, HistogramBucket, PageResult, TableEngine, TxnId, WindowRow,
};

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

    // ── Layer 52 ──────────────────────────────────────────────────────────────
    /// Window function result: Vec of (pk, window_val) rows.
    WindowRows(Vec<WindowRow>),

    // ── Layer 53 ──────────────────────────────────────────────────────────────
    /// Sorted unique values from DISTINCT.
    DistinctValues(Vec<u64>),

    /// (value, count) pairs from VALUE_COUNTS.
    ValueCounts(Vec<(u64, u64)>),

    // ── Layer 54 ──────────────────────────────────────────────────────────────
    /// Number of rows updated by UPDATE WHERE / INCREMENT WHERE / DECREMENT WHERE.
    UpdatedCount(usize),

    // ── Layer 55 ──────────────────────────────────────────────────────────────
    /// Results of a BATCH GET — one entry per requested pk (None = not found).
    BatchRows(Vec<Option<EngineRow>>),

    /// Number of rows deleted by BATCH DELETE.
    BatchDeleted(usize),

    // ── Layer 56 ──────────────────────────────────────────────────────────────
    /// Rows matching all predicates from SCAN WHERE.
    ScanWhereRows(Vec<EngineRow>),

    // ── Layer 57 ──────────────────────────────────────────────────────────────
    /// Projected rows from SCAN PROJECT (values contain only the requested cols).
    ProjectedRows(Vec<EngineRow>),

    // ── Layer 58 ──────────────────────────────────────────────────────────────
    /// Result of INSERT IF_ABSENT: true = row was inserted, false = already existed.
    ConditionalInserted(bool),

    /// Result of CAS: true = swap applied, false = expected values did not match.
    Swapped(bool),

    // ── Layer 59 ──────────────────────────────────────────────────────────────
    /// Number of rows inserted by BATCH INSERT.
    BatchInserted(usize),

    // ── Layer 60 ──────────────────────────────────────────────────────────────
    /// Result of GET TTL (None = row absent or expired).
    TtlRow(Option<EngineRow>),

    /// Rows from SCAN TTL (excludes expired rows).
    TtlRows(Vec<EngineRow>),

    /// Number of rows purged by PURGE EXPIRED.
    Purged(usize),

    // ── Layer 82: DDL ─────────────────────────────────────────────────────────
    /// CREATE TABLE succeeded; `name` is the table that was created.
    TableCreated { name: String },

    /// DROP TABLE succeeded; `name` is the table that was dropped.
    TableDropped { name: String },
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

        Query::Describe => Ok(QueryResponse::Schema(engine.schema.names())),

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

        // ── Layer 52: window functions ─────────────────────────────────────────
        Query::WindowRunningSum {
            filter_col,
            lo,
            hi,
            agg_col,
        } => {
            let rows = engine.running_sum(filter_col, lo, hi, agg_col)?;
            Ok(QueryResponse::WindowRows(rows))
        }

        Query::WindowRunningCount { filter_col, lo, hi } => {
            let rows = engine.running_count(filter_col, lo, hi)?;
            Ok(QueryResponse::WindowRows(rows))
        }

        Query::WindowRank {
            filter_col,
            lo,
            hi,
            sort_col,
            asc,
        } => {
            let rows = engine.rank_by(filter_col, lo, hi, sort_col, asc)?;
            Ok(QueryResponse::WindowRows(rows))
        }

        Query::WindowDenseRank {
            filter_col,
            lo,
            hi,
            sort_col,
            asc,
        } => {
            let rows = engine.dense_rank_by(filter_col, lo, hi, sort_col, asc)?;
            Ok(QueryResponse::WindowRows(rows))
        }

        // ── Layer 53: distinct + value counts ─────────────────────────────────
        Query::Distinct { col, lo, hi } => {
            let vals = engine.distinct_values(col, lo, hi)?;
            Ok(QueryResponse::DistinctValues(vals))
        }

        Query::ValueCounts { col, lo, hi } => {
            let pairs = engine.value_counts(col, lo, hi)?;
            Ok(QueryResponse::ValueCounts(pairs))
        }

        // ── Layer 54: bulk predicate field updates ─────────────────────────────
        Query::UpdateWhere {
            filter_col,
            lo,
            hi,
            target_col,
            val,
        } => {
            let n = engine.update_field_where(&[(filter_col, lo, hi)], target_col, val)?;
            Ok(QueryResponse::UpdatedCount(n))
        }

        Query::IncrementWhere {
            filter_col,
            lo,
            hi,
            target_col,
            delta,
        } => {
            let n = engine.increment_field_where(&[(filter_col, lo, hi)], target_col, delta)?;
            Ok(QueryResponse::UpdatedCount(n))
        }

        Query::DecrementWhere {
            filter_col,
            lo,
            hi,
            target_col,
            delta,
        } => {
            let n = engine.decrement_field_where(&[(filter_col, lo, hi)], target_col, delta)?;
            Ok(QueryResponse::UpdatedCount(n))
        }

        // ── Layer 55: batch operations ─────────────────────────────────────────
        Query::BatchGet { pks } => {
            let results = engine.batch_get(&pks)?;
            Ok(QueryResponse::BatchRows(results))
        }

        Query::BatchDelete { pks } => {
            let n = engine.batch_delete(&pks)?;
            Ok(QueryResponse::BatchDeleted(n))
        }

        // ── Layer 56: multi-predicate compound scan ────────────────────────────
        Query::ScanWhere { predicates } => {
            let rows = engine.scan_where(&predicates)?;
            Ok(QueryResponse::ScanWhereRows(rows))
        }

        // ── Layer 57: column projection scan ───────────────────────────────────
        Query::ScanProject {
            filter_col,
            lo,
            hi,
            projection,
        } => {
            let rows = engine.scan_project(filter_col, lo, hi, &projection)?;
            Ok(QueryResponse::ProjectedRows(rows))
        }

        // ── Layer 58: conditional writes ────────────────────────────────────────
        Query::InsertIfAbsent { pk, values } => {
            let inserted = engine.insert_if_absent(pk, values)?;
            Ok(QueryResponse::ConditionalInserted(inserted))
        }

        Query::CompareAndSwap {
            pk,
            expected,
            new_values,
        } => {
            let swapped = engine.compare_and_swap(pk, &expected, new_values)?;
            Ok(QueryResponse::Swapped(swapped))
        }

        // ── Layer 59: batch insert ───────────────────────────────────────────────
        Query::BatchInsert { rows } => {
            let n = engine.batch_insert(rows)?;
            Ok(QueryResponse::BatchInserted(n))
        }

        // ── Layer 60: TTL operations ─────────────────────────────────────────────
        Query::InsertWithTtl {
            pk,
            values,
            expire_at,
        } => {
            engine.insert_with_ttl(pk, values, expire_at)?;
            Ok(QueryResponse::Ok)
        }

        Query::GetWithTtl { pk, now } => {
            let row = engine.get_with_ttl(pk, now)?;
            Ok(QueryResponse::TtlRow(row))
        }

        Query::ScanWithTtl { now } => {
            let rows = engine.scan_with_ttl(now)?;
            Ok(QueryResponse::TtlRows(rows))
        }

        Query::PurgeExpired { now } => {
            let n = engine.purge_expired(now)?;
            Ok(QueryResponse::Purged(n))
        }

        // DDL queries require a registry; reject them here.
        Query::CreateTable { .. } | Query::DropTable { .. } => Err(
            crate::error::QueryError::Syntax("DDL queries must use execute_ddl()".into()),
        ),
    }
}

/// Parse and execute a query string in one step.
pub fn run(engine: &TableEngine, input: &str) -> QueryResult<QueryResponse> {
    let query = crate::parser::parse(input)?;
    execute(engine, query)
}

// ── Layer 82: DDL ─────────────────────────────────────────────────────────────

/// Execute a DDL [`Query`] (CREATE TABLE / DROP TABLE) against `registry`.
///
/// DML queries belong to [`execute`] which operates on a single `TableEngine`.
/// DDL queries operate on the registry and must use this function instead.
pub fn execute_ddl(
    registry: &vectorvolt_engine::TableRegistry,
    query: Query,
) -> QueryResult<QueryResponse> {
    use crate::error::QueryError;
    use vectorvolt_engine::schema::{ColumnDef, TableSchema};

    match query {
        Query::CreateTable { name, columns } => {
            let col_defs: Vec<ColumnDef> = columns
                .into_iter()
                .map(|(col_name, type_str)| ColumnDef::from_line(&format!("{col_name}:{type_str}")))
                .collect();
            let schema = TableSchema::new(col_defs).map_err(QueryError::Engine)?;
            registry
                .create_table_typed(&name, schema)
                .map_err(QueryError::Engine)?;
            Ok(QueryResponse::TableCreated { name })
        }

        Query::DropTable { name } => {
            registry.drop_table(&name).map_err(QueryError::Engine)?;
            Ok(QueryResponse::TableDropped { name })
        }

        other => Err(QueryError::Syntax(format!(
            "{other:?} is not a DDL query; use execute() for DML"
        ))),
    }
}

/// Parse and execute a DDL query string against `registry`.
pub fn run_ddl(
    registry: &vectorvolt_engine::TableRegistry,
    input: &str,
) -> QueryResult<QueryResponse> {
    let query = crate::parser::parse(input)?;
    execute_ddl(registry, query)
}
