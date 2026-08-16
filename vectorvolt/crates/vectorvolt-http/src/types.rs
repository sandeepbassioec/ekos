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
