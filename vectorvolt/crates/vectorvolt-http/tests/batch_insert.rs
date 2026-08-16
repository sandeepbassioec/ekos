/// Layer 59 — batch_insert via HTTP + DSL.
///
/// New route:
///   POST /tables/{t}/batch/insert  body: {"rows":[{"pk":N,"values":[...]},...]}
///   → {"inserted": N}
///
/// New DSL statement:
///   BATCH INSERT ROW <pk> <v1> [v2 ...] [ROW <pk> <v1> [v2 ...] ...]
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

    async fn get(&self, table: &str, pk: u64) -> Value {
        self.client
            .get(self.url(&format!("/tables/{table}/rows/{pk}")))
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap()
    }
}

// ── HTTP batch/insert ─────────────────────────────────────────────────────────

#[tokio::test]
async fn batch_insert_returns_correct_count() {
    let h = Harness::start("t", &["pk", "v"]).await;
    let resp: Value = h
        .client
        .post(h.url("/tables/t/batch/insert"))
        .json(&serde_json::json!({
            "rows": [
                {"pk": 1, "values": [1, 10]},
                {"pk": 2, "values": [2, 20]},
                {"pk": 3, "values": [3, 30]}
            ]
        }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(resp["inserted"].as_u64().unwrap(), 3);
}

#[tokio::test]
async fn batch_insert_rows_are_retrievable() {
    let h = Harness::start("t", &["pk", "x"]).await;
    h.client
        .post(h.url("/tables/t/batch/insert"))
        .json(&serde_json::json!({
            "rows": [
                {"pk": 10, "values": [10, 111]},
                {"pk": 11, "values": [11, 222]}
            ]
        }))
        .send()
        .await
        .unwrap();
    let r10 = h.get("t", 10).await;
    let r11 = h.get("t", 11).await;
    assert_eq!(r10["values"][1].as_u64().unwrap(), 111);
    assert_eq!(r11["values"][1].as_u64().unwrap(), 222);
}

#[tokio::test]
async fn batch_insert_single_row() {
    let h = Harness::start("t", &["pk", "y"]).await;
    let resp: Value = h
        .client
        .post(h.url("/tables/t/batch/insert"))
        .json(&serde_json::json!({
            "rows": [{"pk": 99, "values": [99, 77]}]
        }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(resp["inserted"].as_u64().unwrap(), 1);
    let row = h.get("t", 99).await;
    assert_eq!(row["values"][0].as_u64().unwrap(), 99);
    assert_eq!(row["values"][1].as_u64().unwrap(), 77);
}

#[tokio::test]
async fn batch_insert_five_rows_all_present() {
    let h = Harness::start("t", &["pk", "v"]).await;
    let rows: Vec<_> = (0u64..5)
        .map(|i| serde_json::json!({"pk": i, "values": [i, i * 10]}))
        .collect();
    h.client
        .post(h.url("/tables/t/batch/insert"))
        .json(&serde_json::json!({ "rows": rows }))
        .send()
        .await
        .unwrap();
    for i in 0u64..5 {
        let row = h.get("t", i).await;
        assert_eq!(row["values"][1].as_u64().unwrap(), i * 10, "pk {i}");
    }
}

#[tokio::test]
async fn batch_insert_empty_body_returns_zero() {
    let h = Harness::start("t", &["pk", "v"]).await;
    let resp: Value = h
        .client
        .post(h.url("/tables/t/batch/insert"))
        .json(&serde_json::json!({ "rows": [] }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(resp["inserted"].as_u64().unwrap(), 0);
}

// ── DSL: BATCH INSERT ─────────────────────────────────────────────────────────

#[tokio::test]
async fn dsl_batch_insert_single_row() {
    let h = Harness::start("t", &["pk", "v"]).await;
    let resp = h
        .client
        .post(h.url("/tables/t/query"))
        .header("content-type", "text/plain")
        .body("BATCH INSERT ROW 50 50 500")
        .send()
        .await
        .unwrap();
    assert!(resp.status().is_success());
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["inserted"].as_u64().unwrap(), 1);
    let row = h.get("t", 50).await;
    assert_eq!(row["values"][1].as_u64().unwrap(), 500);
}

#[tokio::test]
async fn dsl_batch_insert_two_rows() {
    let h = Harness::start("t", &["pk", "a", "b"]).await;
    let resp: Value = h
        .client
        .post(h.url("/tables/t/query"))
        .header("content-type", "text/plain")
        .body("BATCH INSERT ROW 1 1 10 20 ROW 2 2 30 40")
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(resp["inserted"].as_u64().unwrap(), 2);
    let r1 = h.get("t", 1).await;
    let r2 = h.get("t", 2).await;
    assert_eq!(r1["values"][1].as_u64().unwrap(), 10);
    assert_eq!(r1["values"][2].as_u64().unwrap(), 20);
    assert_eq!(r2["values"][1].as_u64().unwrap(), 30);
    assert_eq!(r2["values"][2].as_u64().unwrap(), 40);
}

#[tokio::test]
async fn dsl_and_http_batch_insert_same_count() {
    let h = Harness::start("t", &["pk", "v"]).await;
    let http_resp: Value = h
        .client
        .post(h.url("/tables/t/batch/insert"))
        .json(&serde_json::json!({
            "rows": [{"pk": 200, "values": [200, 1]}, {"pk": 201, "values": [201, 2]}]
        }))
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
        .body("BATCH INSERT ROW 202 202 3 ROW 203 203 4")
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(http_resp["inserted"].as_u64().unwrap(), 2);
    assert_eq!(dsl_resp["inserted"].as_u64().unwrap(), 2);
}

// ── large batch ───────────────────────────────────────────────────────────────

#[tokio::test]
async fn batch_insert_large_batch() {
    let h = Harness::start("t", &["pk", "v"]).await;
    let rows: Vec<_> = (0u64..50)
        .map(|i| serde_json::json!({"pk": i, "values": [i, i * 7]}))
        .collect();
    let resp: Value = h
        .client
        .post(h.url("/tables/t/batch/insert"))
        .json(&serde_json::json!({ "rows": rows }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(resp["inserted"].as_u64().unwrap(), 50);
    let row = h.get("t", 49).await;
    assert_eq!(row["values"][1].as_u64().unwrap(), 49 * 7);
}

// ── response shape ────────────────────────────────────────────────────────────

#[tokio::test]
async fn batch_insert_response_has_inserted_field() {
    let h = Harness::start("t", &["pk", "v"]).await;
    let resp: Value = h
        .client
        .post(h.url("/tables/t/batch/insert"))
        .json(&serde_json::json!({
            "rows": [{"pk": 77, "values": [77, 1]}]
        }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert!(
        resp.get("inserted").is_some(),
        "response must have 'inserted' field"
    );
}
