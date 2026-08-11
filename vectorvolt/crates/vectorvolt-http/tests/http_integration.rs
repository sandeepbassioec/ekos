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
