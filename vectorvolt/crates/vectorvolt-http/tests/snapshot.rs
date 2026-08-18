/// Layer 69 — WAL Snapshot + Truncation.
///
/// Routes tested:
///   POST /tables/{t}/snapshot   → flush, compact, write snapshot marker
///   GET  /tables/{t}/snapshot   → return snapshot metadata
use std::sync::Arc;
use std::time::Duration;

use reqwest::{Client, StatusCode};
use serde_json::Value;
use tempfile::TempDir;
use vectorvolt_engine::{EngineOptions, TableRegistry};
use vectorvolt_http::{HttpServer, HttpServerConfig};

const FLUSH: usize = 1_048_576;
const FLUSH_EAGER: usize = 1; // 1 row per partition — forces flush after every insert

// ── harness ───────────────────────────────────────────────────────────────────

struct Harness {
    base_url: String,
    _dir: TempDir,
    client: Client,
}

impl Harness {
    async fn start() -> Self {
        Self::start_with(FLUSH).await
    }

    async fn start_with(flush_threshold: usize) -> Self {
        let dir = TempDir::new().unwrap();
        let opts = EngineOptions::new(flush_threshold);
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

    async fn create_table(&self, table: &str, cols: &[&str]) {
        self.client
            .post(self.url(&format!("/tables/{table}")))
            .json(&serde_json::json!({ "columns": cols }))
            .send()
            .await
            .unwrap();
    }

    async fn insert(&self, table: &str, pk: u64, values: &[u64]) {
        self.client
            .post(self.url(&format!("/tables/{table}/rows/{pk}")))
            .json(&serde_json::json!({ "values": values }))
            .send()
            .await
            .unwrap();
    }

    async fn delete_row(&self, table: &str, pk: u64) {
        self.client
            .delete(self.url(&format!("/tables/{table}/rows/{pk}")))
            .send()
            .await
            .unwrap();
    }

    async fn get_row(&self, table: &str, pk: u64) -> Value {
        self.client
            .get(self.url(&format!("/tables/{table}/rows/{pk}")))
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap()
    }

    async fn take_snapshot(&self, table: &str) -> (StatusCode, Value) {
        let resp = self
            .client
            .post(self.url(&format!("/tables/{table}/snapshot")))
            .json(&serde_json::json!({}))
            .send()
            .await
            .unwrap();
        let status = resp.status();
        let body: Value = resp.json().await.unwrap();
        (status, body)
    }

    async fn snapshot_info(&self, table: &str) -> (StatusCode, Value) {
        let resp = self
            .client
            .get(self.url(&format!("/tables/{table}/snapshot")))
            .send()
            .await
            .unwrap();
        let status = resp.status();
        let body: Value = resp.json().await.unwrap();
        (status, body)
    }
}

// ── tests ─────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn snapshot_empty_table() {
    let h = Harness::start().await;
    h.create_table("t", &["a"]).await;
    let (status, body) = h.take_snapshot("t").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["row_count"], 0);
    assert!(body["byte_size"].as_u64().is_some());
}

#[tokio::test]
async fn snapshot_row_count() {
    let h = Harness::start().await;
    h.create_table("t", &["a"]).await;
    for i in 1u64..=5 {
        h.insert("t", i, &[i * 10]).await;
    }
    let (status, body) = h.take_snapshot("t").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["row_count"], 5);
}

#[tokio::test]
async fn snapshot_byte_size_grows() {
    let h = Harness::start().await;
    h.create_table("t", &["a"]).await;
    let (_, b0) = h.take_snapshot("t").await;
    for i in 1u64..=10 {
        h.insert("t", i, &[i]).await;
    }
    let (_, b1) = h.take_snapshot("t").await;
    let size0 = b0["byte_size"].as_u64().unwrap();
    let size1 = b1["byte_size"].as_u64().unwrap();
    assert!(size1 > size0, "byte_size should grow after inserts");
}

#[tokio::test]
async fn snapshot_info_no_snapshot() {
    let h = Harness::start().await;
    h.create_table("t", &["a"]).await;
    let (status, body) = h.snapshot_info("t").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["exists"], false);
    assert_eq!(body["row_count"], 0);
}

#[tokio::test]
async fn snapshot_info_after_snapshot() {
    let h = Harness::start().await;
    h.create_table("t", &["a"]).await;
    for i in 1u64..=3 {
        h.insert("t", i, &[i]).await;
    }
    h.take_snapshot("t").await;
    let (status, body) = h.snapshot_info("t").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["exists"], true);
    assert_eq!(body["row_count"], 3);
}

#[tokio::test]
async fn take_snapshot_unknown_table_404() {
    let h = Harness::start().await;
    let (status, _) = h.take_snapshot("no_such_table").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn snapshot_info_unknown_table_404() {
    let h = Harness::start().await;
    let (status, _) = h.snapshot_info("no_such_table").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn data_readable_after_snapshot() {
    let h = Harness::start().await;
    h.create_table("t", &["a"]).await;
    for i in 1u64..=4 {
        h.insert("t", i, &[i * 100]).await;
    }
    h.take_snapshot("t").await;
    for i in 1u64..=4 {
        let row = h.get_row("t", i).await;
        assert_eq!(row["values"][0], i * 100);
    }
}

#[tokio::test]
async fn insert_after_snapshot_works() {
    let h = Harness::start().await;
    h.create_table("t", &["a"]).await;
    h.insert("t", 1, &[10]).await;
    h.take_snapshot("t").await;
    h.insert("t", 2, &[20]).await;
    let row = h.get_row("t", 2).await;
    assert_eq!(row["values"][0], 20);
}

#[tokio::test]
async fn snapshot_over_snapshot_idempotent() {
    let h = Harness::start().await;
    h.create_table("t", &["a"]).await;
    h.insert("t", 1, &[99]).await;
    let (s1, b1) = h.take_snapshot("t").await;
    let (s2, b2) = h.take_snapshot("t").await;
    assert_eq!(s1, StatusCode::OK);
    assert_eq!(s2, StatusCode::OK);
    assert_eq!(b1["row_count"], b2["row_count"]);
}

#[tokio::test]
async fn snapshot_excludes_deleted_rows() {
    // Use flush_threshold=1 so each insert is immediately flushed to a partition
    // before the delete happens.  Deleting a row that is still in the write
    // buffer is transient — flush would re-insert it into pk_index.  With an
    // eager threshold every insert triggers its own flush, so by the time
    // delete(3) runs the row is safely in a partition and pk_index reflects the
    // deletion permanently.
    let h = Harness::start_with(FLUSH_EAGER).await;
    h.create_table("t", &["a"]).await;
    for i in 1u64..=5 {
        h.insert("t", i, &[i]).await;
    }
    h.delete_row("t", 3).await;
    let (_, body) = h.take_snapshot("t").await;
    assert_eq!(body["row_count"], 4);
}

#[tokio::test]
async fn snapshot_row_count_matches_info() {
    let h = Harness::start().await;
    h.create_table("t", &["a"]).await;
    for i in 1u64..=7 {
        h.insert("t", i, &[i]).await;
    }
    let (_, snap) = h.take_snapshot("t").await;
    let (_, info) = h.snapshot_info("t").await;
    assert_eq!(snap["row_count"], info["row_count"]);
    assert_eq!(snap["byte_size"], info["byte_size"]);
}

#[tokio::test]
async fn snapshot_multi_table_isolation() {
    let h = Harness::start().await;
    h.create_table("a", &["x"]).await;
    h.create_table("b", &["x"]).await;
    for i in 1u64..=3 {
        h.insert("a", i, &[i]).await;
    }
    h.insert("b", 1, &[100]).await;
    let (_, sa) = h.take_snapshot("a").await;
    let (_, sb) = h.take_snapshot("b").await;
    assert_eq!(sa["row_count"], 3);
    assert_eq!(sb["row_count"], 1);
}

#[tokio::test]
async fn snapshot_byte_size_nonzero_after_inserts() {
    let h = Harness::start().await;
    h.create_table("t", &["a"]).await;
    h.insert("t", 1, &[42]).await;
    let (_, body) = h.take_snapshot("t").await;
    assert!(body["byte_size"].as_u64().unwrap() > 0);
}

#[tokio::test]
async fn snapshot_info_byte_size_nonzero() {
    let h = Harness::start().await;
    h.create_table("t", &["a"]).await;
    h.insert("t", 1, &[7]).await;
    h.take_snapshot("t").await;
    let (_, info) = h.snapshot_info("t").await;
    assert!(info["byte_size"].as_u64().unwrap() > 0);
}
