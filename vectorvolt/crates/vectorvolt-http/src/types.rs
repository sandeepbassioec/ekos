use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
pub struct RowResp {
    pub pk: u64,
    pub values: Vec<u64>,
}

// ── Layer 44 response types ───────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct CountResp {
    pub count: u64,
}

#[derive(Debug, Serialize)]
pub struct AggResp {
    pub value: Option<u64>,
}

#[derive(Debug, Serialize)]
pub struct PageResp {
    pub rows: Vec<RowResp>,
    pub next_cursor: Option<u64>,
}

#[derive(Debug, Serialize)]
pub struct RowsResp {
    pub rows: Vec<RowResp>,
}

#[derive(Debug, Deserialize)]
pub struct InsertBody {
    pub values: Vec<u64>,
}

#[derive(Debug, Deserialize)]
pub struct CreateTableBody {
    pub columns: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct ScanQuery {
    pub col: u32,
    pub lo: u64,
    pub hi: u64,
}

// shared col/lo/hi filter used by aggregate + delete-range endpoints
#[derive(Debug, Deserialize)]
pub struct RangeQuery {
    pub col: usize,
    pub lo: u64,
    pub hi: u64,
}

// ── Layer 50 query params ─────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct HistogramQuery {
    pub col: usize,
    pub lo: u64,
    pub hi: u64,
    pub buckets: usize,
}

#[derive(Debug, Deserialize)]
pub struct PercentileQuery {
    pub col: usize,
    pub lo: u64,
    pub hi: u64,
    /// Integer 0–100 (e.g. 95 means P95).
    pub p: u64,
}

// ── Layer 50 response types ───────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct BucketResp {
    pub lo: u64,
    pub hi: u64,
    pub count: u64,
}

#[derive(Debug, Serialize)]
pub struct HistogramResp {
    pub buckets: Vec<BucketResp>,
}

#[derive(Debug, Serialize)]
pub struct PercentileResp {
    pub value: Option<u64>,
}

#[derive(Debug, Deserialize)]
pub struct ScanSortedQuery {
    pub col: usize,
    pub lo: u64,
    pub hi: u64,
    pub sort: usize,
    pub asc: u8,
    pub limit: usize,
}

#[derive(Debug, Deserialize)]
pub struct ScanPageQuery {
    pub col: usize,
    pub lo: u64,
    pub hi: u64,
    pub after: u64,
    pub limit: usize,
}

#[derive(Debug, Deserialize)]
pub struct UpdateFieldBody {
    pub col: usize,
    pub val: u64,
}

#[derive(Debug, Deserialize)]
pub struct SecondaryQuery {
    pub col: u32,
    pub val: u64,
}

#[derive(Debug, Serialize)]
pub struct TablesResp {
    pub tables: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct SchemaResp {
    pub columns: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct ErrorResp {
    pub error: String,
}

#[derive(Debug, Serialize)]
pub struct OkResp {
    pub ok: bool,
}

// ── query endpoint ────────────────────────────────────────────────────────────

/// JSON envelope returned by `POST /tables/{table}/query`.
///
/// Every variant carries a `type` discriminant so callers can pattern-match
/// without inspecting field presence.
#[derive(Debug, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum QueryResp {
    Row {
        pk: u64,
        values: Vec<u64>,
    },
    Rows {
        rows: Vec<RowResp>,
    },
    TxnStarted {
        txn_id: u64,
    },
    Schema {
        columns: Vec<String>,
    },
    Stats {
        rows_buffered: usize,
        rows_flushed: u64,
        partition_count: usize,
        schema: Vec<String>,
    },
    Ok,
    NotFound,
    /// COUNT or DELETE RANGE result.
    Count {
        count: u64,
    },
    /// SUM / MIN / MAX / AVG result (None when no rows matched).
    Aggregate {
        value: Option<u64>,
    },
    /// SCAN PAGE result with optional next-cursor.
    Page {
        rows: Vec<RowResp>,
        next_cursor: Option<u64>,
    },
    /// HISTOGRAM result.
    Histogram {
        buckets: Vec<BucketResp>,
    },
    /// PERCENTILE result (None when no rows matched).
    Percentile {
        value: Option<u64>,
    },
    /// GROUP COUNT/SUM/MIN/MAX result.
    Group {
        entries: Vec<GroupEntryResp>,
    },
    /// Window function result.
    Window {
        rows: Vec<WindowRowResp>,
    },
    /// DISTINCT result.
    Distinct {
        values: Vec<u64>,
    },
    /// VALUE_COUNTS result.
    ValueCounts {
        entries: Vec<ValueCountEntry>,
    },
    /// UPDATE WHERE / INCREMENT WHERE / DECREMENT WHERE result.
    Updated {
        updated: usize,
    },
    /// BATCH GET result: rows (null entries for not-found pks).
    BatchRows {
        rows: Vec<Option<RowResp>>,
    },
    /// BATCH DELETE result.
    BatchDeleted {
        deleted: usize,
    },
    /// SCAN WHERE multi-predicate result.
    ScanWhere {
        rows: Vec<RowResp>,
    },
    /// SCAN PROJECT result (values contain only the requested columns).
    ProjectedRows {
        rows: Vec<RowResp>,
    },
    /// INSERT IF_ABSENT result.
    ConditionalInserted {
        inserted: bool,
    },
    /// CAS result.
    Swapped {
        swapped: bool,
    },
    /// BATCH INSERT result.
    BatchInserted {
        inserted: usize,
    },
    /// GET TTL result (row present and not expired).
    TtlRow {
        pk: u64,
        values: Vec<u64>,
    },
    /// SCAN TTL result.
    TtlRows {
        rows: Vec<RowResp>,
    },
    /// PURGE EXPIRED result.
    Purged {
        purged: usize,
    },
}

// ── Layer 53 response types ───────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct DistinctResp {
    pub values: Vec<u64>,
}

#[derive(Debug, Serialize)]
pub struct ValueCountEntry {
    pub value: u64,
    pub count: u64,
}

#[derive(Debug, Serialize)]
pub struct ValueCountsResp {
    pub entries: Vec<ValueCountEntry>,
}

// ── Layer 54 query params + response types ────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct UpdateWhereQuery {
    pub filter_col: usize,
    pub lo: u64,
    pub hi: u64,
    pub target_col: usize,
    pub val: u64,
}

#[derive(Debug, Deserialize)]
pub struct IncrDecrWhereQuery {
    pub filter_col: usize,
    pub lo: u64,
    pub hi: u64,
    pub target_col: usize,
    pub delta: u64,
}

#[derive(Debug, Serialize)]
pub struct UpdatedResp {
    pub updated: usize,
}

// ── Layer 55 request + response types ────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct BatchPksBody {
    pub pks: Vec<u64>,
}

#[derive(Debug, Serialize)]
pub struct BatchGetResp {
    pub rows: Vec<Option<RowResp>>,
}

#[derive(Debug, Serialize)]
pub struct BatchDeleteResp {
    pub deleted: usize,
}

// ── Layer 56 request + response types ────────────────────────────────────────

/// Up to 3 AND-predicates: col0/lo0/hi0 required; col1/lo1/hi1 and col2/lo2/hi2 optional.
#[derive(Debug, Deserialize)]
pub struct ScanWhereQuery {
    pub col0: usize,
    pub lo0: u64,
    pub hi0: u64,
    pub col1: Option<usize>,
    pub lo1: Option<u64>,
    pub hi1: Option<u64>,
    pub col2: Option<usize>,
    pub lo2: Option<u64>,
    pub hi2: Option<u64>,
}

// ── Layer 57 query params + response types ────────────────────────────────────

/// `?filter_col=<n>&lo=<lo>&hi=<hi>&proj=<csv>` — `proj` is comma-separated
/// column indices (e.g. `"1,2,3"`); omitting it returns PKs only.
#[derive(Debug, Deserialize)]
pub struct ScanProjectQuery {
    pub filter_col: usize,
    pub lo: u64,
    pub hi: u64,
    pub proj: Option<String>,
}

impl ScanProjectQuery {
    pub fn into_projection(&self) -> Vec<usize> {
        self.proj
            .as_deref()
            .unwrap_or("")
            .split(',')
            .filter(|s| !s.is_empty())
            .filter_map(|s| s.trim().parse().ok())
            .collect()
    }
}

#[derive(Debug, Serialize)]
pub struct ScanProjectResp {
    pub rows: Vec<RowResp>,
}

impl ScanWhereQuery {
    pub fn into_predicates(self) -> Vec<(usize, u64, u64)> {
        let mut v = vec![(self.col0, self.lo0, self.hi0)];
        if let (Some(c), Some(lo), Some(hi)) = (self.col1, self.lo1, self.hi1) {
            v.push((c, lo, hi));
        }
        if let (Some(c), Some(lo), Some(hi)) = (self.col2, self.lo2, self.hi2) {
            v.push((c, lo, hi));
        }
        v
    }
}

// ── Layer 58 request + response types ────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct CasBody {
    pub expected: Vec<u64>,
    pub new_values: Vec<u64>,
}

#[derive(Debug, Serialize)]
pub struct ConditionalInsertResp {
    pub inserted: bool,
}

#[derive(Debug, Serialize)]
pub struct CasResp {
    pub swapped: bool,
}

// ── Layer 59 request + response types ────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct BatchInsertRowBody {
    pub pk: u64,
    pub values: Vec<u64>,
}

#[derive(Debug, Deserialize)]
pub struct BatchInsertBody {
    pub rows: Vec<BatchInsertRowBody>,
}

#[derive(Debug, Serialize)]
pub struct BatchInsertResp {
    pub inserted: usize,
}

// ── Layer 60 request + response types ────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct InsertTtlBody {
    pub values: Vec<u64>,
    pub expire_at: u64,
}

#[derive(Debug, Deserialize)]
pub struct TtlNowQuery {
    pub now: u64,
}

#[derive(Debug, Serialize)]
pub struct PurgedResp {
    pub purged: usize,
}

// ── Layer 51 query params + response types ────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct GroupByQuery {
    pub filter_col: usize,
    pub lo: u64,
    pub hi: u64,
    pub group_col: usize,
}

#[derive(Debug, Deserialize)]
pub struct GroupByAggQuery {
    pub filter_col: usize,
    pub lo: u64,
    pub hi: u64,
    pub group_col: usize,
    pub agg_col: usize,
}

#[derive(Debug, Serialize)]
pub struct GroupEntryResp {
    pub group_val: u64,
    pub agg_val: u64,
}

#[derive(Debug, Serialize)]
pub struct GroupResp {
    pub entries: Vec<GroupEntryResp>,
}

// ── Layer 52 query params + response types ────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct WindowAggQuery {
    pub filter_col: usize,
    pub lo: u64,
    pub hi: u64,
    pub agg_col: usize,
}

#[derive(Debug, Deserialize)]
pub struct WindowCountQuery {
    pub filter_col: usize,
    pub lo: u64,
    pub hi: u64,
}

#[derive(Debug, Deserialize)]
pub struct WindowRankQuery {
    pub filter_col: usize,
    pub lo: u64,
    pub hi: u64,
    pub sort_col: usize,
    pub asc: u8,
}

#[derive(Debug, Serialize)]
pub struct WindowRowResp {
    pub pk: u64,
    pub window_val: u64,
}

#[derive(Debug, Serialize)]
pub struct WindowResp {
    pub rows: Vec<WindowRowResp>,
}

// ── Layer 66 — Materialized Views ────────────────────────────────────────────

/// Request body for `POST /views`.
#[derive(Debug, Deserialize)]
pub struct CreateViewBody {
    pub name: String,
    pub table: String,
    pub filter_col: usize,
    pub lo: u64,
    pub hi: u64,
    pub group_col: usize,
    /// One of "count", "sum", "min", "max".
    pub kind: String,
    /// Required when `kind` is "sum", "min", or "max".
    pub agg_col: Option<usize>,
    /// When `true`, replay the table's CDC log into the view before going live.
    pub backfill: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct ViewEntryResp {
    pub group_val: u64,
    pub agg_val: u64,
}

#[derive(Debug, Serialize)]
pub struct ViewStateResp {
    pub name: String,
    pub entries: Vec<ViewEntryResp>,
}

#[derive(Debug, Serialize)]
pub struct ViewInfoResp {
    pub name: String,
    pub table: String,
    pub kind: String,
    pub filter_col: usize,
    pub lo: u64,
    pub hi: u64,
    pub group_col: usize,
}

#[derive(Debug, Serialize)]
pub struct ViewsListResp {
    pub views: Vec<ViewInfoResp>,
}

// ── Layer 65 query params ─────────────────────────────────────────────────────

/// Optional column-range filter for `GET /tables/{t}/subscribe`.
///
/// When all three fields are present, only events whose `values[col]` falls in
/// `[lo, hi]` are forwarded.  Delete events (no values) are suppressed when a
/// filter is active.
///
/// When `from` is set, historical events from the CDC log (lsn >= from) are
/// streamed before switching to the live broadcast.
#[derive(Debug, Clone, Copy, Deserialize)]
pub struct SubscribeQuery {
    pub col: Option<usize>,
    pub lo: Option<u64>,
    pub hi: Option<u64>,
    /// Replay CDC log starting from this LSN before switching to live events.
    pub from: Option<u64>,
}

// ── Layer 67 types ────────────────────────────────────────────────────────────

/// Query params for `GET /tables/{t}/cdc/log`.
#[derive(Debug, Clone, Copy, Deserialize)]
pub struct CdcLogQuery {
    /// Return only events with `lsn >= from`.  Defaults to 0 (all events).
    pub from: Option<u64>,
}

/// Response body for `GET /tables/{t}/cdc/log`.
#[derive(Debug, Serialize)]
pub struct CdcLogResp {
    pub events: Vec<serde_json::Value>,
    pub count: usize,
}

// ── Layer 68 types ────────────────────────────────────────────────────────────

/// Response body for `GET /tables/{t}/cdc/stats`.
#[derive(Debug, Serialize)]
pub struct CdcStatsResp {
    pub event_count: u64,
    pub byte_size: u64,
    pub min_lsn: u64,
    pub max_lsn: u64,
}

/// Query params for `POST /tables/{t}/cdc/compact`.
#[derive(Debug, Clone, Copy, Deserialize)]
pub struct CdcCompactQuery {
    /// Remove events with `lsn < retain_lsn`.
    pub retain_lsn: u64,
}

/// Response body for `POST /tables/{t}/cdc/compact`.
#[derive(Debug, Serialize)]
pub struct CdcCompactResp {
    /// Number of events removed (lsn < retain_lsn).
    pub removed: usize,
}

// ── Layer 69 types ────────────────────────────────────────────────────────────

/// Response body for `POST /tables/{t}/snapshot`.
#[derive(Debug, Serialize)]
pub struct SnapshotResp {
    pub row_count: u64,
    pub byte_size: u64,
}

/// Response body for `GET /tables/{t}/snapshot`.
#[derive(Debug, Serialize)]
pub struct SnapshotInfoResp {
    pub exists: bool,
    pub row_count: u64,
    pub byte_size: u64,
}

// ── Layer 70 types ────────────────────────────────────────────────────────────

/// Request body for `POST /tables/{t}/webhooks`.
#[derive(Debug, Deserialize)]
pub struct RegisterWebhookBody {
    /// HTTP/HTTPS URL to POST change events to.
    pub url: String,
}

/// One webhook sink entry in list/register responses.
#[derive(Debug, Serialize)]
pub struct WebhookEntryResp {
    pub id: u64,
    pub table: String,
    pub url: String,
}

/// Response body for `POST /tables/{t}/webhooks`.
#[derive(Debug, Serialize)]
pub struct RegisterWebhookResp {
    pub id: u64,
}

/// Response body for `GET /tables/{t}/webhooks`.
#[derive(Debug, Serialize)]
pub struct WebhooksListResp {
    pub webhooks: Vec<WebhookEntryResp>,
}

// ── Layer 71 types ────────────────────────────────────────────────────────────

/// Response body for `POST /multi_txn`.
#[derive(Debug, Serialize)]
pub struct MultiTxnBeginResp {
    pub txn_id: u64,
}

/// One operation staged into a multi-table transaction.
#[derive(Debug, Deserialize)]
pub struct MultiTxnOpBody {
    pub table: String,
    /// `"insert"` or `"delete"`.
    pub op: String,
    pub pk: u64,
    /// Required when `op` is `"insert"`.
    pub values: Option<Vec<u64>>,
}

/// Response body for `POST /multi_txn/{id}/ops`.
#[derive(Debug, Serialize)]
pub struct MultiTxnStagedResp {
    pub staged: usize,
}

/// Response body for `POST /multi_txn/{id}/commit`.
#[derive(Debug, Serialize)]
pub struct MultiTxnCommitResp {
    pub committed: usize,
}

/// Response body for `POST /multi_txn/{id}/rollback`.
#[derive(Debug, Serialize)]
pub struct MultiTxnRollbackResp {
    pub rolled_back: bool,
}

/// Response body for `GET /multi_txn/{id}`.
#[derive(Debug, Serialize)]
pub struct MultiTxnInfoResp {
    pub txn_id: u64,
    pub pending_ops: usize,
}
