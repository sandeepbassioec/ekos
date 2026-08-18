/// Layer 71 — Multi-Table Transactions.
///
/// Routes tested:
///   POST /multi_txn                   → begin, returns {txn_id}
///   GET  /multi_txn/{id}              → pending op count
///   POST /multi_txn/{id}/ops          → stage one op
///   POST /multi_txn/{id}/commit       → apply all ops
///   POST /multi_txn/{id}/rollback     → discard all ops
use std::sync::Arc;
use std::time::Duration;

use reqwest::{Client, StatusCode};
use serde_json::Value;
use tempfile::TempDir;
use vectorvolt_engine::{EngineOptions, TableRegistry};
use vectorvolt_http::{HttpServer, HttpServerConfig};

struct Harness {
    base_url: String,
    _dir: TempDir,
    client: Client,
}

impl Harness {
    async fn start() -> Self {
        let dir = TempDir::new().unwrap();
        let opts = EngineOptions::new(1_048_576);
        let reg = Arc::new(TableRegistry::open(dir.path(), opts).unwrap());
        let cfg = HttpServerConfig::new("127.0.0.1:0".parse().unwrap());
        let srv = HttpServer::bind(reg, cfg).await.unwrap();
        let addr = srv.local_addr().unwrap();
        tokio::spawn(async move { srv.serve().await.unwrap() });
        let client = Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .unwrap();
        Harness {
            base_url: format!("http://{addr}"),
            _dir: dir,
            client,
        }
    }

    fn url(&self, path: &str) -> String {
        format!("{}{path}", self.base_url)
    }

    async fn create_table(&self, table: &str) {
        self.client
            .post(self.url(&format!("/tables/{table}")))
            .json(&serde_json::json!({ "columns": ["a"] }))
            .send()
            .await
            .unwrap();
    }

    async fn get_row(&self, table: &str, pk: u64) -> (StatusCode, Value) {
        let resp = self
            .client
            .get(self.url(&format!("/tables/{table}/rows/{pk}")))
            .send()
            .await
            .unwrap();
        let status = resp.status();
        let body: Value = resp.json().await.unwrap();
        (status, body)
    }

    async fn begin(&self) -> u64 {
        let resp: Value = self
            .client
            .post(self.url("/multi_txn"))
            .json(&serde_json::json!({}))
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();
        resp["txn_id"].as_u64().unwrap()
    }

    async fn info(&self, txn_id: u64) -> (StatusCode, Value) {
        let resp = self
            .client
            .get(self.url(&format!("/multi_txn/{txn_id}")))
            .send()
            .await
            .unwrap();
        let status = resp.status();
        let body: Value = resp.json().await.unwrap();
        (status, body)
    }

    async fn stage(
        &self,
        txn_id: u64,
        table: &str,
        op: &str,
        pk: u64,
        values: Option<&[u64]>,
    ) -> (StatusCode, Value) {
        let mut body = serde_json::json!({ "table": table, "op": op, "pk": pk });
        if let Some(v) = values {
            body["values"] = serde_json::json!(v);
        }
        let resp = self
            .client
            .post(self.url(&format!("/multi_txn/{txn_id}/ops")))
            .json(&body)
            .send()
            .await
            .unwrap();
        let status = resp.status();
        let body: Value = resp.json().await.unwrap();
        (status, body)
    }

    async fn commit(&self, txn_id: u64) -> (StatusCode, Value) {
        let resp = self
            .client
            .post(self.url(&format!("/multi_txn/{txn_id}/commit")))
            .json(&serde_json::json!({}))
            .send()
            .await
            .unwrap();
        let status = resp.status();
        let body: Value = resp.json().await.unwrap();
        (status, body)
    }

    async fn rollback(&self, txn_id: u64) -> (StatusCode, Value) {
        let resp = self
            .client
            .post(self.url(&format!("/multi_txn/{txn_id}/rollback")))
            .json(&serde_json::json!({}))
            .send()
            .await
            .unwrap();
        let status = resp.status();
        let body: Value = resp.json().await.unwrap();
        (status, body)
    }
}

#[tokio::test]
async fn begin_returns_txn_id() {
    let h = Harness::start().await;
    let txn_id = h.begin().await;
    assert!(txn_id > 0);
}

#[tokio::test]
async fn info_returns_pending_count() {
    let h = Harness::start().await;
    h.create_table("t").await;
    let txn_id = h.begin().await;
    let (status, body) = h.info(txn_id).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["txn_id"], txn_id);
    assert_eq!(body["pending_ops"], 0);
}

#[tokio::test]
async fn info_unknown_txn_404() {
    let h = Harness::start().await;
    let (status, _) = h.info(9999).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn stage_insert_increments_count() {
    let h = Harness::start().await;
    h.create_table("t").await;
    let txn_id = h.begin().await;
    let (status, body) = h.stage(txn_id, "t", "insert", 1, Some(&[10])).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["staged"], 1);
}

#[tokio::test]
async fn stage_on_unknown_txn_404() {
    let h = Harness::start().await;
    h.create_table("t").await;
    let (status, _) = h.stage(9999, "t", "insert", 1, Some(&[10])).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn stage_bad_op_400() {
    let h = Harness::start().await;
    h.create_table("t").await;
    let txn_id = h.begin().await;
    let (status, _) = h.stage(txn_id, "t", "upsert", 1, Some(&[10])).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn stage_insert_missing_values_400() {
    let h = Harness::start().await;
    h.create_table("t").await;
    let txn_id = h.begin().await;
    let (status, _) = h.stage(txn_id, "t", "insert", 1, None).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn commit_empty_txn() {
    let h = Harness::start().await;
    let txn_id = h.begin().await;
    let (status, body) = h.commit(txn_id).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["committed"], 0);
}

#[tokio::test]
async fn commit_unknown_txn_404() {
    let h = Harness::start().await;
    let (status, _) = h.commit(9999).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn commit_applies_inserts_across_tables() {
    let h = Harness::start().await;
    h.create_table("a").await;
    h.create_table("b").await;
    let txn_id = h.begin().await;
    h.stage(txn_id, "a", "insert", 1, Some(&[100])).await;
    h.stage(txn_id, "b", "insert", 2, Some(&[200])).await;
    let (status, body) = h.commit(txn_id).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["committed"], 2);
    let (s_a, row_a) = h.get_row("a", 1).await;
    let (s_b, row_b) = h.get_row("b", 2).await;
    assert_eq!(s_a, StatusCode::OK);
    assert_eq!(s_b, StatusCode::OK);
    assert_eq!(row_a["values"][0], 100);
    assert_eq!(row_b["values"][0], 200);
}

#[tokio::test]
async fn commit_applies_deletes() {
    let h = Harness::start().await;
    h.create_table("t").await;
    // Insert directly first.
    h.client
        .post(h.url("/tables/t/rows/5"))
        .json(&serde_json::json!({ "values": [55] }))
        .send()
        .await
        .unwrap();
    let txn_id = h.begin().await;
    h.stage(txn_id, "t", "delete", 5, None).await;
    h.commit(txn_id).await;
    let (status, _) = h.get_row("t", 5).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn rollback_discards_staged_ops() {
    let h = Harness::start().await;
    h.create_table("t").await;
    let txn_id = h.begin().await;
    h.stage(txn_id, "t", "insert", 7, Some(&[77])).await;
    let (status, body) = h.rollback(txn_id).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["rolled_back"], true);
    let (row_status, _) = h.get_row("t", 7).await;
    assert_eq!(row_status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn rollback_removes_txn() {
    let h = Harness::start().await;
    let txn_id = h.begin().await;
    h.rollback(txn_id).await;
    let (status, _) = h.info(txn_id).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn commit_removes_txn() {
    let h = Harness::start().await;
    let txn_id = h.begin().await;
    h.commit(txn_id).await;
    let (status, _) = h.info(txn_id).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}
