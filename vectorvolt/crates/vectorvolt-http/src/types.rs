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
}
