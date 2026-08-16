/// Layer 57 — HTTP column-projection scan endpoint, and DSL SCAN PROJECT.
///
/// New route:
///   GET /tables/{t}/scan/project?filter_col=&lo=&hi=&proj=<csv>
///
/// New DSL statement:
///   SCAN PROJECT col=<n> lo=<lo> hi=<hi> COLS <c1> [c2 ...]
use std::sync::Arc;

use reqwest::Client;
use serde_json::Value;
use tempfile::TempDir;
use vectorvolt_engine::{EngineOptions, TableRegistry};
use vectorvolt_http::{HttpServer, HttpServerConfig};

const FLUSH: usize = 1_048_576;

struct Harness {
    base_url: String,
    _dir: TempDir,
    client: Client,
}

impl Harness {
    async fn start(table: &str, cols: &[&str]) -> Self {
        let dir = TempDir::new().unwrap();
        let opts = EngineOptions::new(FLUSH);
        let reg = Arc::new(TableRegistry::open(dir.path(), opts).unwrap());
        let cfg = HttpServerConfig::new("127.0.0.1:0".parse().unwrap());
        let srv = HttpServer::bind(reg, cfg).await.unwrap();
        let addr = srv.local_addr().unwrap();
        tokio::spawn(async move { srv.serve().await.unwrap() });
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(5))
            .build()
            .unwrap();
        let base = format!("http://{addr}");
        client
            .post(format!("{base}/tables/{table}"))
            .json(&serde_json::json!({ "columns": cols }))
            .send()
            .await
            .unwrap();
        Harness {
            base_url: base,
            _dir: dir,
            client,
        }
    }

    fn url(&self, path: &str) -> String {
        format!("{}{path}", self.base_url)
    }

    async fn insert(&self, table: &str, pk: u64, values: &[u64]) {
        self.client
            .post(self.url(&format!("/tables/{table}/rows/{pk}")))
            .json(&serde_json::json!({ "values": values }))
            .send()
            .await
            .unwrap();
    }
}

// ── single-column projection ──────────────────────────────────────────────────

#[tokio::test]
async fn scan_project_single_col_returns_only_that_col() {
    let h = Harness::start("t", &["pk", "a", "b", "c"]).await;
    h.insert("t", 1, &[1, 10, 20, 30]).await;
    h.insert("t", 2, &[2, 11, 21, 31]).await;
    let resp: Value = h
        .client
        .get(h.url("/tables/t/scan/project?filter_col=0&lo=1&hi=2&proj=2"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let rows = resp["rows"].as_array().expect("rows");
    assert_eq!(rows.len(), 2);
    for row in rows {
        let vals = row["values"].as_array().unwrap();
        assert_eq!(vals.len(), 1, "only col 2 projected");
    }
}

#[tokio::test]
async fn scan_project_values_match_projected_column() {
    let h = Harness::start("t", &["pk", "x", "y"]).await;
    h.insert("t", 5, &[5, 99, 77]).await;
    let resp: Value = h
        .client
        .get(h.url("/tables/t/scan/project?filter_col=0&lo=5&hi=5&proj=2"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let row = &resp["rows"][0];
    assert_eq!(row["pk"].as_u64().unwrap(), 5);
    assert_eq!(row["values"][0].as_u64().unwrap(), 77, "y column value");
}

// ── multi-column projection ───────────────────────────────────────────────────

#[tokio::test]
async fn scan_project_two_cols_returns_two_values() {
    let h = Harness::start("t", &["pk", "a", "b", "c"]).await;
    h.insert("t", 1, &[1, 10, 20, 30]).await;
    let resp: Value = h
        .client
        .get(h.url("/tables/t/scan/project?filter_col=0&lo=1&hi=1&proj=1,3"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let row = &resp["rows"][0];
    let vals = row["values"].as_array().unwrap();
    assert_eq!(vals.len(), 2);
    assert_eq!(vals[0].as_u64().unwrap(), 10, "a");
    assert_eq!(vals[1].as_u64().unwrap(), 30, "c");
}

#[tokio::test]
async fn scan_project_filter_col_not_in_projection() {
    // filter on col 1 but project col 2 only — filter col need not be returned
    let h = Harness::start("t", &["pk", "cat", "score"]).await;
    h.insert("t", 1, &[1, 5, 100]).await;
    h.insert("t", 2, &[2, 5, 200]).await;
    h.insert("t", 3, &[3, 9, 300]).await;
    let resp: Value = h
        .client
        .get(h.url("/tables/t/scan/project?filter_col=1&lo=5&hi=5&proj=2"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let rows = resp["rows"].as_array().unwrap();
    assert_eq!(rows.len(), 2, "only cat=5 rows");
    for row in rows {
        // values has only score
        assert_eq!(row["values"].as_array().unwrap().len(), 1);
    }
}

// ── empty projection ──────────────────────────────────────────────────────────

#[tokio::test]
async fn scan_project_empty_proj_returns_pks_only() {
    let h = Harness::start("t", &["pk", "v"]).await;
    h.insert("t", 3, &[3, 99]).await;
    h.insert("t", 7, &[7, 99]).await;
    let resp: Value = h
        .client
        .get(h.url("/tables/t/scan/project?filter_col=1&lo=99&hi=99"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let rows = resp["rows"].as_array().unwrap();
    assert_eq!(rows.len(), 2);
    for row in rows {
        // no proj → values is empty
        assert_eq!(row["values"].as_array().unwrap().len(), 0);
    }
}

// ── no match ─────────────────────────────────────────────────────────────────

#[tokio::test]
async fn scan_project_no_match_returns_empty() {
    let h = Harness::start("t", &["pk", "v"]).await;
    h.insert("t", 1, &[1, 5]).await;
    let resp: Value = h
        .client
        .get(h.url("/tables/t/scan/project?filter_col=1&lo=100&hi=200&proj=1"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert!(resp["rows"].as_array().unwrap().is_empty());
}

// ── row shape ─────────────────────────────────────────────────────────────────

#[tokio::test]
async fn scan_project_rows_have_pk_and_values_fields() {
    let h = Harness::start("t", &["pk", "x"]).await;
    h.insert("t", 42, &[42, 7]).await;
    let resp: Value = h
        .client
        .get(h.url("/tables/t/scan/project?filter_col=0&lo=42&hi=42&proj=1"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let row = &resp["rows"][0];
    assert!(row.get("pk").is_some());
    assert!(row.get("values").is_some());
    assert_eq!(row["pk"].as_u64().unwrap(), 42);
}

// ── DSL bridge ────────────────────────────────────────────────────────────────

#[tokio::test]
async fn dsl_scan_project_single_col() {
    let h = Harness::start("t", &["pk", "a", "b"]).await;
    h.insert("t", 1, &[1, 10, 20]).await;
    h.insert("t", 2, &[2, 11, 21]).await;
    let resp = h
        .client
        .post(h.url("/tables/t/query"))
        .header("content-type", "text/plain")
        .body("SCAN PROJECT col=0 lo=1 hi=2 COLS 2")
        .send()
        .await
        .unwrap();
    assert!(resp.status().is_success());
    let body: Value = resp.json().await.unwrap();
    let rows = body["rows"].as_array().expect("rows");
    assert_eq!(rows.len(), 2);
    for row in rows {
        let vals = row["values"].as_array().unwrap();
        assert_eq!(vals.len(), 1, "only col 2");
    }
}

#[tokio::test]
async fn dsl_scan_project_two_cols() {
    let h = Harness::start("t", &["pk", "a", "b", "c"]).await;
    h.insert("t", 10, &[10, 1, 2, 3]).await;
    let resp = h
        .client
        .post(h.url("/tables/t/query"))
        .header("content-type", "text/plain")
        .body("SCAN PROJECT col=0 lo=10 hi=10 COLS 1 3")
        .send()
        .await
        .unwrap();
    assert!(resp.status().is_success());
    let body: Value = resp.json().await.unwrap();
    let row = &body["rows"][0];
    let vals = row["values"].as_array().unwrap();
    assert_eq!(vals.len(), 2);
    assert_eq!(vals[0].as_u64().unwrap(), 1);
    assert_eq!(vals[1].as_u64().unwrap(), 3);
}

#[tokio::test]
async fn dsl_scan_project_row_count_matches_http() {
    let h = Harness::start("t", &["pk", "cat", "score"]).await;
    for pk in 0u64..8 {
        h.insert("t", pk, &[pk, pk % 2, pk * 10]).await;
    }
    let http_resp: Value = h
        .client
        .get(h.url("/tables/t/scan/project?filter_col=1&lo=0&hi=0&proj=2"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let dsl_resp: Value = h
        .client
        .post(h.url("/tables/t/query"))
        .header("content-type", "text/plain")
        .body("SCAN PROJECT col=1 lo=0 hi=0 COLS 2")
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(
        http_resp["rows"].as_array().unwrap().len(),
        dsl_resp["rows"].as_array().unwrap().len(),
        "HTTP and DSL SCAN PROJECT must return the same row count"
    );
}

// ── projection correctness vs full scan ──────────────────────────────────────

#[tokio::test]
async fn projected_values_match_full_scan_values() {
    let h = Harness::start("t", &["pk", "a", "b"]).await;
    for pk in 1u64..=5 {
        h.insert("t", pk, &[pk, pk * 3, pk * 7]).await;
    }
    // full scan to get ground truth
    let full: Value = h
        .client
        .get(h.url("/tables/t/scan?col=0&lo=1&hi=5"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let proj: Value = h
        .client
        .get(h.url("/tables/t/scan/project?filter_col=0&lo=1&hi=5&proj=2"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    let full_rows = full["rows"].as_array().unwrap();
    let proj_rows = proj["rows"].as_array().unwrap();
    assert_eq!(full_rows.len(), proj_rows.len());

    // build pk → b_value map from full scan
    let mut b_map: std::collections::HashMap<u64, u64> = std::collections::HashMap::new();
    for r in full_rows {
        let pk = r["pk"].as_u64().unwrap();
        let b = r["values"][2].as_u64().unwrap();
        b_map.insert(pk, b);
    }
    for r in proj_rows {
        let pk = r["pk"].as_u64().unwrap();
        let projected_b = r["values"][0].as_u64().unwrap();
        assert_eq!(
            projected_b,
            *b_map.get(&pk).unwrap(),
            "projected col 2 must match full scan col 2"
        );
    }
}

#[tokio::test]
async fn scan_project_fewer_cols_than_full_row() {
    let h = Harness::start("t", &["pk", "a", "b", "c", "d"]).await;
    h.insert("t", 1, &[1, 2, 3, 4, 5]).await;
    let resp: Value = h
        .client
        .get(h.url("/tables/t/scan/project?filter_col=0&lo=1&hi=1&proj=1,3"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let row = &resp["rows"][0];
    // full row has 5 columns; projection has only 2
    assert_eq!(row["values"].as_array().unwrap().len(), 2);
}

#[tokio::test]
async fn scan_project_multiple_rows_all_same_projected_len() {
    let h = Harness::start("t", &["pk", "x", "y", "z"]).await;
    for pk in 0u64..6 {
        h.insert("t", pk, &[pk, pk, pk, pk]).await;
    }
    let resp: Value = h
        .client
        .get(h.url("/tables/t/scan/project?filter_col=0&lo=0&hi=5&proj=1,2"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let rows = resp["rows"].as_array().unwrap();
    assert_eq!(rows.len(), 6);
    for row in rows {
        assert_eq!(row["values"].as_array().unwrap().len(), 2);
    }
}

#[tokio::test]
async fn scan_project_pk_field_always_correct() {
    // pk is always set from col 0 regardless of projection
    let h = Harness::start("t", &["pk", "a", "b"]).await;
    for pk in 10u64..=14 {
        h.insert("t", pk, &[pk, pk * 2, pk * 3]).await;
    }
    let resp: Value = h
        .client
        .get(h.url("/tables/t/scan/project?filter_col=0&lo=10&hi=14&proj=2"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let rows = resp["rows"].as_array().unwrap();
    assert_eq!(rows.len(), 5);
    let mut pks: Vec<u64> = rows.iter().map(|r| r["pk"].as_u64().unwrap()).collect();
    pks.sort_unstable();
    assert_eq!(pks, vec![10, 11, 12, 13, 14]);
}

#[tokio::test]
async fn dsl_scan_project_filter_narrows_rows() {
    let h = Harness::start("t", &["pk", "group", "val"]).await;
    for pk in 0u64..10 {
        h.insert("t", pk, &[pk, pk % 3, pk * 5]).await;
    }
    // only rows with group=0
    let resp = h
        .client
        .post(h.url("/tables/t/query"))
        .header("content-type", "text/plain")
        .body("SCAN PROJECT col=1 lo=0 hi=0 COLS 2")
        .send()
        .await
        .unwrap();
    assert!(resp.status().is_success());
    let body: Value = resp.json().await.unwrap();
    let rows = body["rows"].as_array().unwrap();
    // group=0 for pk 0,3,6,9 → 4 rows
    assert_eq!(rows.len(), 4);
    for row in rows {
        // projected col is val (col 2); values has exactly 1 entry
        assert_eq!(row["values"].as_array().unwrap().len(), 1);
    }
}
