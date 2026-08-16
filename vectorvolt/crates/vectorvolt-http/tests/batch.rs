/// Layer 55 — HTTP batch get + batch delete endpoints, and DSL BATCH GET / BATCH DELETE.
///
/// New routes:
///   POST /tables/{t}/batch/get    body: {"pks": [...]}
///   POST /tables/{t}/batch/delete body: {"pks": [...]}
///
/// New DSL statements:
///   BATCH GET pk1 pk2 ...
///   BATCH DELETE pk1 pk2 ...
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

// ── batch/get endpoint ────────────────────────────────────────────────────────

#[tokio::test]
async fn batch_get_returns_all_found_rows() {
    let h = Harness::start("t", &["pk", "v"]).await;
    h.insert("t", 1, &[1, 10]).await;
    h.insert("t", 2, &[2, 20]).await;
    h.insert("t", 3, &[3, 30]).await;
    let resp: Value = h
        .client
        .post(h.url("/tables/t/batch/get"))
        .json(&serde_json::json!({ "pks": [1, 2, 3] }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let rows = resp["rows"].as_array().expect("rows array");
    assert_eq!(rows.len(), 3, "must return one entry per requested pk");
    assert!(
        rows.iter().all(|r| !r.is_null()),
        "all rows must be present"
    );
}

#[tokio::test]
async fn batch_get_returns_null_for_missing_pks() {
    let h = Harness::start("t", &["pk", "v"]).await;
    h.insert("t", 1, &[1, 10]).await;
    let resp: Value = h
        .client
        .post(h.url("/tables/t/batch/get"))
        .json(&serde_json::json!({ "pks": [1, 99, 100] }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let rows = resp["rows"].as_array().unwrap();
    assert_eq!(rows.len(), 3);
    assert!(!rows[0].is_null(), "pk=1 must be present");
    assert!(rows[1].is_null(), "pk=99 must be null");
    assert!(rows[2].is_null(), "pk=100 must be null");
}

#[tokio::test]
async fn batch_get_preserves_request_order() {
    let h = Harness::start("t", &["pk", "v"]).await;
    h.insert("t", 10, &[10, 100]).await;
    h.insert("t", 20, &[20, 200]).await;
    h.insert("t", 30, &[30, 300]).await;
    let resp: Value = h
        .client
        .post(h.url("/tables/t/batch/get"))
        .json(&serde_json::json!({ "pks": [30, 10, 20] }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let rows = resp["rows"].as_array().unwrap();
    assert_eq!(rows[0]["pk"].as_u64().unwrap(), 30);
    assert_eq!(rows[1]["pk"].as_u64().unwrap(), 10);
    assert_eq!(rows[2]["pk"].as_u64().unwrap(), 20);
}

#[tokio::test]
async fn batch_get_empty_pks_returns_empty() {
    let h = Harness::start("t", &["pk", "v"]).await;
    h.insert("t", 1, &[1, 10]).await;
    let resp: Value = h
        .client
        .post(h.url("/tables/t/batch/get"))
        .json(&serde_json::json!({ "pks": [] }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let rows = resp["rows"].as_array().unwrap();
    assert!(rows.is_empty());
}

#[tokio::test]
async fn batch_get_row_has_pk_and_values_fields() {
    let h = Harness::start("t", &["pk", "a", "b"]).await;
    h.insert("t", 5, &[5, 11, 22]).await;
    let resp: Value = h
        .client
        .post(h.url("/tables/t/batch/get"))
        .json(&serde_json::json!({ "pks": [5] }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let row = &resp["rows"][0];
    assert!(row.get("pk").is_some(), "row must have pk field");
    assert!(row.get("values").is_some(), "row must have values field");
    assert_eq!(row["pk"].as_u64().unwrap(), 5);
}

// ── batch/delete endpoint ─────────────────────────────────────────────────────

#[tokio::test]
async fn batch_delete_removes_all_specified_rows() {
    let h = Harness::start("t", &["pk", "v"]).await;
    h.insert("t", 1, &[1, 10]).await;
    h.insert("t", 2, &[2, 20]).await;
    h.insert("t", 3, &[3, 30]).await;
    let del_resp: Value = h
        .client
        .post(h.url("/tables/t/batch/delete"))
        .json(&serde_json::json!({ "pks": [1, 2] }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(del_resp["deleted"].as_u64().unwrap(), 2);
    // verify they're gone
    let get_resp: Value = h
        .client
        .post(h.url("/tables/t/batch/get"))
        .json(&serde_json::json!({ "pks": [1, 2, 3] }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let rows = get_resp["rows"].as_array().unwrap();
    assert!(rows[0].is_null(), "pk=1 must be gone");
    assert!(rows[1].is_null(), "pk=2 must be gone");
    assert!(!rows[2].is_null(), "pk=3 must remain");
}

#[tokio::test]
async fn batch_delete_returns_deleted_count() {
    let h = Harness::start("t", &["pk", "v"]).await;
    for pk in 0u64..5 {
        h.insert("t", pk, &[pk, pk * 10]).await;
    }
    let resp: Value = h
        .client
        .post(h.url("/tables/t/batch/delete"))
        .json(&serde_json::json!({ "pks": [0, 1, 2, 3, 4] }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(resp["deleted"].as_u64().unwrap(), 5);
}

#[tokio::test]
async fn batch_delete_missing_pks_silently_skipped() {
    let h = Harness::start("t", &["pk", "v"]).await;
    h.insert("t", 1, &[1, 10]).await;
    let resp: Value = h
        .client
        .post(h.url("/tables/t/batch/delete"))
        .json(&serde_json::json!({ "pks": [1, 99, 100] }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    // engine silently skips missing pks; at least 1 deleted
    assert!(resp["deleted"].as_u64().unwrap() >= 1);
}

#[tokio::test]
async fn batch_delete_empty_pks_returns_zero() {
    let h = Harness::start("t", &["pk", "v"]).await;
    h.insert("t", 1, &[1, 10]).await;
    let resp: Value = h
        .client
        .post(h.url("/tables/t/batch/delete"))
        .json(&serde_json::json!({ "pks": [] }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(resp["deleted"].as_u64().unwrap(), 0);
}

// ── DSL bridge ────────────────────────────────────────────────────────────────

#[tokio::test]
async fn dsl_batch_get_returns_rows() {
    let h = Harness::start("t", &["pk", "v"]).await;
    h.insert("t", 1, &[1, 10]).await;
    h.insert("t", 2, &[2, 20]).await;
    let resp = h
        .client
        .post(h.url("/tables/t/query"))
        .header("content-type", "text/plain")
        .body("BATCH GET 1 2 99")
        .send()
        .await
        .unwrap();
    assert!(resp.status().is_success());
    let body: Value = resp.json().await.unwrap();
    let rows = body["rows"].as_array().expect("rows array");
    assert_eq!(rows.len(), 3);
    assert!(!rows[0].is_null());
    assert!(!rows[1].is_null());
    assert!(rows[2].is_null(), "pk=99 not found -> null");
}

#[tokio::test]
async fn dsl_batch_delete_returns_deleted() {
    let h = Harness::start("t", &["pk", "v"]).await;
    h.insert("t", 10, &[10, 1]).await;
    h.insert("t", 20, &[20, 2]).await;
    let resp = h
        .client
        .post(h.url("/tables/t/query"))
        .header("content-type", "text/plain")
        .body("BATCH DELETE 10 20")
        .send()
        .await
        .unwrap();
    assert!(resp.status().is_success());
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["deleted"].as_u64().unwrap(), 2);
}

#[tokio::test]
async fn dsl_batch_get_count_matches_http() {
    let h = Harness::start("t", &["pk", "v"]).await;
    for pk in 1u64..=4 {
        h.insert("t", pk, &[pk, pk * 5]).await;
    }
    let http_resp: Value = h
        .client
        .post(h.url("/tables/t/batch/get"))
        .json(&serde_json::json!({ "pks": [1, 2, 3, 4] }))
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
        .body("BATCH GET 1 2 3 4")
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(
        http_resp["rows"].as_array().unwrap().len(),
        dsl_resp["rows"].as_array().unwrap().len(),
        "HTTP and DSL BATCH GET must return same number of entries"
    );
}

#[tokio::test]
async fn batch_delete_then_batch_get_returns_all_null() {
    let h = Harness::start("t", &["pk", "v"]).await;
    for pk in 1u64..=3 {
        h.insert("t", pk, &[pk, pk]).await;
    }
    h.client
        .post(h.url("/tables/t/batch/delete"))
        .json(&serde_json::json!({ "pks": [1, 2, 3] }))
        .send()
        .await
        .unwrap();
    let resp: Value = h
        .client
        .post(h.url("/tables/t/batch/get"))
        .json(&serde_json::json!({ "pks": [1, 2, 3] }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let rows = resp["rows"].as_array().unwrap();
    assert!(
        rows.iter().all(|r| r.is_null()),
        "all deleted rows must return null"
    );
}
