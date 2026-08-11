use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
pub struct RowResp {
    pub pk: u64,
    pub values: Vec<u64>,
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
