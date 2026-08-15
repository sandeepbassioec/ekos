/// Layer 14 — HTTP/REST gateway integration tests.
///
/// Each test spins up an `HttpServer` bound to a random port on localhost,
/// backed by an in-memory `TableRegistry`, and drives it through `reqwest`.
use std::sync::Arc;

use reqwest::Client;
use serde_json::{Value, json};
use tempfile::TempDir;
use vectorvolt_engine::{EngineOptions, TableRegistry};
use vectorvolt_http::{HttpServer, HttpServerConfig};

const FLUSH_THRESHOLD: usize = 1_048_576;

struct Harness {
    base_url: String,
    _dir: TempDir,
    client: Client,
}

impl Harness {
    async fn start_multi() -> Self {
        let dir = TempDir::new().unwrap();
        let opts = EngineOptions::new(FLUSH_THRESHOLD);
        let reg = Arc::new(TableRegistry::open(dir.path(), opts).unwrap());
        let cfg = HttpServerConfig::new("127.0.0.1:0".parse().unwrap());
        let srv = HttpServer::bind(reg, cfg).await.unwrap();
        let addr = srv.local_addr().unwrap();
        tokio::spawn(async move { srv.serve().await.unwrap() });
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(5))
            .build()
            .unwrap();
        Harness {
            base_url: format!("http://{addr}"),
            _dir: dir,
            client,
        }
    }

    async fn start_with_table(name: &str, columns: &[&str]) -> Self {
        let h = Self::start_multi().await;
        let cols: Vec<String> = columns.iter().map(|s| s.to_string()).collect();
        let resp = h
            .client
            .post(format!("{}/tables/{name}", h.base_url))
            .json(&json!({ "columns": cols }))
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status().as_u16(), 201, "create table failed");
        h
    }

    fn url(&self, path: &str) -> String {
        format!("{}{path}", self.base_url)
    }
}

// ── list tables ───────────────────────────────────────────────────────────────

#[tokio::test]
async fn list_tables_empty_on_fresh_registry() {
    let h = Harness::start_multi().await;
    let resp: Value = h
        .client
        .get(h.url("/tables"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(resp["tables"], json!([]));
}

// ── create + list ─────────────────────────────────────────────────────────────

#[tokio::test]
async fn create_table_appears_in_list() {
    let h = Harness::start_multi().await;
    let resp = h
        .client
        .post(h.url("/tables/items"))
        .json(&json!({ "columns": ["pk","name","qty"] }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status().as_u16(), 201);

    let list: Value = h
        .client
        .get(h.url("/tables"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let names = list["tables"].as_array().unwrap();
    assert!(names.iter().any(|n| n == "items"), "items must be listed");
}

// ── describe table ────────────────────────────────────────────────────────────

#[tokio::test]
async fn describe_table_returns_schema() {
    let h = Harness::start_with_table("users", &["pk", "age", "score"]).await;
    let resp: Value = h
        .client
        .get(h.url("/tables/users/schema"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(resp["columns"], json!(["pk", "age", "score"]));
}

// ── drop table ────────────────────────────────────────────────────────────────

#[tokio::test]
async fn drop_table_removes_from_list() {
    let h = Harness::start_with_table("tmp", &["pk", "val"]).await;
    let del = h.client.delete(h.url("/tables/tmp")).send().await.unwrap();
    assert!(del.status().is_success());

    let list: Value = h
        .client
        .get(h.url("/tables"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let names = list["tables"].as_array().unwrap();
    assert!(!names.iter().any(|n| n == "tmp"), "tmp must be gone");
}

// ── insert + get ──────────────────────────────────────────────────────────────

#[tokio::test]
async fn insert_and_get_row() {
    let h = Harness::start_with_table("t", &["pk", "age", "score"]).await;

    let ins: Value = h
        .client
        .post(h.url("/tables/t/rows/42"))
        .json(&json!({ "values": [42u64, 30u64, 99u64] }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(ins["pk"], 42);

    let get: Value = h
        .client
        .get(h.url("/tables/t/rows/42"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(get["pk"], 42);
    assert_eq!(get["values"], json!([42u64, 30u64, 99u64]));
}

// ── get missing row returns 404 ───────────────────────────────────────────────

#[tokio::test]
async fn get_missing_row_returns_404() {
    let h = Harness::start_with_table("t2", &["pk", "val"]).await;
    let resp = h
        .client
        .get(h.url("/tables/t2/rows/9999"))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status().as_u16(), 404);
}

// ── delete row ────────────────────────────────────────────────────────────────

#[tokio::test]
async fn delete_row_then_get_returns_404() {
    let h = Harness::start_with_table("t3", &["pk", "val"]).await;
    h.client
        .post(h.url("/tables/t3/rows/5"))
        .json(&json!({ "values": [5u64, 100u64] }))
        .send()
        .await
        .unwrap();
    let del = h
        .client
        .delete(h.url("/tables/t3/rows/5"))
        .send()
        .await
        .unwrap();
    assert!(del.status().is_success());
    let get = h
        .client
        .get(h.url("/tables/t3/rows/5"))
        .send()
        .await
        .unwrap();
    assert_eq!(get.status().as_u16(), 404);
}

// ── scan ──────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn scan_returns_matching_rows() {
    let h = Harness::start_with_table("t4", &["pk", "age", "score"]).await;
    for pk in 1u64..=5 {
        h.client
            .post(h.url(&format!("/tables/t4/rows/{pk}")))
            .json(&json!({ "values": [pk, pk * 10u64, 0u64] }))
            .send()
            .await
            .unwrap();
    }
    // col=1 (age), lo=20, hi=30 → pks 2 and 3
    let resp: Value = h
        .client
        .get(h.url("/tables/t4/scan?col=1&lo=20&hi=30"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let rows = resp["rows"].as_array().unwrap();
    let mut pks: Vec<u64> = rows.iter().map(|r| r["pk"].as_u64().unwrap()).collect();
    pks.sort();
    assert_eq!(pks, vec![2, 3]);
}

// ── secondary lookup ──────────────────────────────────────────────────────────

#[tokio::test]
async fn secondary_lookup_returns_matching_rows() {
    let h = Harness::start_with_table("t5", &["pk", "age", "score"]).await;
    // pks 1,2,3 all have age=55; pk 4 has age=99
    for pk in 1u64..=3 {
        h.client
            .post(h.url(&format!("/tables/t5/rows/{pk}")))
            .json(&json!({ "values": [pk, 55u64, 0u64] }))
            .send()
            .await
            .unwrap();
    }
    h.client
        .post(h.url("/tables/t5/rows/4"))
        .json(&json!({ "values": [4u64, 99u64, 0u64] }))
        .send()
        .await
        .unwrap();

    let resp: Value = h
        .client
        .get(h.url("/tables/t5/secondary?col=1&val=55"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let rows = resp["rows"].as_array().unwrap();
    let mut pks: Vec<u64> = rows.iter().map(|r| r["pk"].as_u64().unwrap()).collect();
    pks.sort();
    assert_eq!(pks, vec![1, 2, 3]);
}

// ── table not found returns 404 ───────────────────────────────────────────────

#[tokio::test]
async fn get_from_nonexistent_table_returns_404() {
    let h = Harness::start_multi().await;
    let resp = h
        .client
        .get(h.url("/tables/ghost/rows/1"))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status().as_u16(), 404);
}

// ── duplicate table returns 409 ───────────────────────────────────────────────

#[tokio::test]
async fn create_duplicate_table_returns_409() {
    let h = Harness::start_with_table("dup", &["pk", "val"]).await;
    let resp = h
        .client
        .post(h.url("/tables/dup"))
        .json(&json!({ "columns": ["pk","val"] }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status().as_u16(), 409);
}

// ── multi-table isolation ─────────────────────────────────────────────────────

#[tokio::test]
async fn two_tables_are_isolated() {
    let h = Harness::start_multi().await;
    for name in &["alpha", "beta"] {
        h.client
            .post(h.url(&format!("/tables/{name}")))
            .json(&json!({ "columns": ["pk","val"] }))
            .send()
            .await
            .unwrap();
    }
    // insert pk=1 val=10 into alpha and pk=1 val=20 into beta
    h.client
        .post(h.url("/tables/alpha/rows/1"))
        .json(&json!({ "values": [1u64, 10u64] }))
        .send()
        .await
        .unwrap();
    h.client
        .post(h.url("/tables/beta/rows/1"))
        .json(&json!({ "values": [1u64, 20u64] }))
        .send()
        .await
        .unwrap();

    let a: Value = h
        .client
        .get(h.url("/tables/alpha/rows/1"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let b: Value = h
        .client
        .get(h.url("/tables/beta/rows/1"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(a["values"][1], 10);
    assert_eq!(b["values"][1], 20);
}

// ── observability: health ─────────────────────────────────────────────────────

#[tokio::test]
async fn health_returns_ok_status() {
    let h = Harness::start_multi().await;
    let resp: Value = h
        .client
        .get(h.url("/health"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(resp["status"], "ok");
    assert!(resp["version"].is_string(), "version must be present");
}

#[tokio::test]
async fn health_table_count_reflects_creates() {
    let h = Harness::start_multi().await;
    let before: Value = h
        .client
        .get(h.url("/health"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let t0 = before["tables"].as_u64().unwrap();

    h.client
        .post(h.url("/tables/obs_t"))
        .json(&json!({ "columns": ["pk","v"] }))
        .send()
        .await
        .unwrap();

    let after: Value = h
        .client
        .get(h.url("/health"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(after["tables"].as_u64().unwrap(), t0 + 1);
}

#[tokio::test]
async fn health_request_counters_increment() {
    let h = Harness::start_with_table("cnt", &["pk", "v"]).await;

    // insert → ok
    h.client
        .post(h.url("/tables/cnt/rows/1"))
        .json(&json!({ "values": [1u64, 99u64] }))
        .send()
        .await
        .unwrap();
    // get missing → err
    h.client
        .get(h.url("/tables/cnt/rows/9999"))
        .send()
        .await
        .unwrap();

    let health: Value = h
        .client
        .get(h.url("/health"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert!(health["requests_total"].as_u64().unwrap() >= 2);
    assert!(health["requests_ok"].as_u64().unwrap() >= 1);
    assert!(health["requests_err"].as_u64().unwrap() >= 1);
}

// ── observability: Prometheus metrics ────────────────────────────────────────

#[tokio::test]
async fn metrics_endpoint_returns_prometheus_text() {
    let h = Harness::start_multi().await;
    let resp = h.client.get(h.url("/metrics")).send().await.unwrap();
    assert!(resp.status().is_success());
    let ct = resp
        .headers()
        .get("content-type")
        .unwrap()
        .to_str()
        .unwrap();
    assert!(
        ct.contains("text/plain"),
        "metrics must be text/plain, got {ct}"
    );
    let body = resp.text().await.unwrap();
    assert!(
        body.contains("vectorvolt_http_requests_total"),
        "missing counter"
    );
    assert!(body.contains("# TYPE"), "missing TYPE comments");
}

#[tokio::test]
async fn metrics_track_inserts() {
    let h = Harness::start_with_table("m", &["pk", "age"]).await;
    for pk in 1u64..=3 {
        h.client
            .post(h.url(&format!("/tables/m/rows/{pk}")))
            .json(&json!({ "values": [pk, 10u64] }))
            .send()
            .await
            .unwrap();
    }
    let body = h
        .client
        .get(h.url("/metrics"))
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();
    let inserted: u64 = body
        .lines()
        .find(|l| l.starts_with("vectorvolt_rows_inserted_total "))
        .and_then(|l| l.split_whitespace().last())
        .and_then(|v| v.parse().ok())
        .unwrap_or(0);
    assert!(
        inserted >= 3,
        "expected >= 3 inserts tracked, got {inserted}"
    );
}

#[tokio::test]
async fn metrics_include_per_table_stats() {
    let h = Harness::start_with_table("pt", &["pk", "v"]).await;
    h.client
        .post(h.url("/tables/pt/rows/7"))
        .json(&json!({ "values": [7u64, 42u64] }))
        .send()
        .await
        .unwrap();
    let body = h
        .client
        .get(h.url("/metrics"))
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();
    assert!(
        body.contains("vectorvolt_table_rows_buffered{table=\"pt\"}"),
        "per-table buffered gauge missing"
    );
}

// ── Layer 22: text query DSL over HTTP ────────────────────────────────────────

/// Helper: POST a text query to `/tables/{table}/query` and return the JSON.
async fn query(h: &Harness, table: &str, q: &str) -> Value {
    h.client
        .post(h.url(&format!("/tables/{table}/query")))
        .body(q.to_string())
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap()
}

#[tokio::test]
async fn query_insert_and_get() {
    let h = Harness::start_with_table("qt", &["pk", "age", "score"]).await;
    let ins = query(&h, "qt", "INSERT 10 10 25 99").await;
    assert_eq!(ins["type"], "ok");

    let got = query(&h, "qt", "GET 10").await;
    assert_eq!(got["type"], "row");
    assert_eq!(got["pk"], 10);
    assert_eq!(got["values"][1], 25);
}

#[tokio::test]
async fn query_get_not_found() {
    let h = Harness::start_with_table("qt2", &["pk", "age"]).await;
    let resp = query(&h, "qt2", "GET 999").await;
    assert_eq!(resp["type"], "not_found");
}

#[tokio::test]
async fn query_delete_row() {
    let h = Harness::start_with_table("qt3", &["pk", "val"]).await;
    query(&h, "qt3", "INSERT 1 1 100").await;
    let del = query(&h, "qt3", "DELETE 1").await;
    assert_eq!(del["type"], "ok");
    let got = query(&h, "qt3", "GET 1").await;
    assert_eq!(got["type"], "not_found");
}

#[tokio::test]
async fn query_scan_returns_rows() {
    let h = Harness::start_with_table("qt4", &["pk", "age"]).await;
    query(&h, "qt4", "INSERT 1 1 10").await;
    query(&h, "qt4", "INSERT 2 2 20").await;
    query(&h, "qt4", "INSERT 3 3 30").await;
    let resp = query(&h, "qt4", "SCAN col=1 lo=10 hi=20").await;
    assert_eq!(resp["type"], "rows");
    assert_eq!(resp["rows"].as_array().unwrap().len(), 2);
}

#[tokio::test]
async fn query_describe_returns_schema() {
    let h = Harness::start_with_table("qt5", &["pk", "name", "qty"]).await;
    let resp = query(&h, "qt5", "DESCRIBE").await;
    assert_eq!(resp["type"], "schema");
    assert_eq!(resp["columns"], json!(["pk", "name", "qty"]));
}

#[tokio::test]
async fn query_stats_returns_numeric_fields() {
    let h = Harness::start_with_table("qt6", &["pk", "v"]).await;
    query(&h, "qt6", "INSERT 1 1 42").await;
    let resp = query(&h, "qt6", "STATS").await;
    assert_eq!(resp["type"], "stats");
    assert!(resp["rows_buffered"].as_u64().unwrap() >= 1);
}

#[tokio::test]
async fn query_txn_begin_commit() {
    let h = Harness::start_with_table("qt7", &["pk", "val"]).await;
    let begun = query(&h, "qt7", "BEGIN").await;
    assert_eq!(begun["type"], "txn_started");
    let txn_id = begun["txn_id"].as_u64().unwrap();

    let ins = query(&h, "qt7", &format!("INSERT TXN {txn_id} 5 5 777")).await;
    assert_eq!(ins["type"], "ok");

    // Not visible outside txn yet
    let before = query(&h, "qt7", "GET 5").await;
    assert_eq!(before["type"], "not_found");

    let commit = query(&h, "qt7", &format!("COMMIT {txn_id}")).await;
    assert_eq!(commit["type"], "ok");

    let after = query(&h, "qt7", "GET 5").await;
    assert_eq!(after["type"], "row");
    assert_eq!(after["values"][1], 777);
}

#[tokio::test]
async fn query_txn_rollback_discards_writes() {
    let h = Harness::start_with_table("qt8", &["pk", "val"]).await;
    let begun = query(&h, "qt8", "BEGIN").await;
    let txn_id = begun["txn_id"].as_u64().unwrap();

    query(&h, "qt8", &format!("INSERT TXN {txn_id} 9 9 55")).await;
    let rb = query(&h, "qt8", &format!("ROLLBACK {txn_id}")).await;
    assert_eq!(rb["type"], "ok");

    let got = query(&h, "qt8", "GET 9").await;
    assert_eq!(got["type"], "not_found");
}

#[tokio::test]
async fn query_unknown_table_returns_error() {
    let h = Harness::start_multi().await;
    let resp = h
        .client
        .post(h.url("/tables/no_such_table/query"))
        .body("GET 1")
        .send()
        .await
        .unwrap();
    assert!(
        resp.status().is_client_error(),
        "expected 4xx for unknown table"
    );
}

#[tokio::test]
async fn query_syntax_error_returns_error() {
    let h = Harness::start_with_table("qt9", &["pk", "v"]).await;
    let resp = h
        .client
        .post(h.url("/tables/qt9/query"))
        .body("NOT A VALID QUERY @@@@")
        .send()
        .await
        .unwrap();
    assert!(resp.status().is_server_error() || resp.status().is_client_error());
    let body: Value = resp.json().await.unwrap();
    assert!(body["error"].is_string());
}

#[tokio::test]
async fn query_ok_increments_metrics_counter() {
    let h = Harness::start_with_table("qt10", &["pk", "v"]).await;
    let before: Value = h
        .client
        .get(h.url("/health"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let before_ok = before["requests_ok"].as_u64().unwrap();

    query(&h, "qt10", "INSERT 1 1 10").await;
    query(&h, "qt10", "GET 1").await;

    let after: Value = h
        .client
        .get(h.url("/health"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert!(after["requests_ok"].as_u64().unwrap() > before_ok);
}
