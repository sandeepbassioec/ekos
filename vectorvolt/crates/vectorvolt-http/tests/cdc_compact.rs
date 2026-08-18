/// Layer 68 — CDC Log Compaction + Stats.
///
/// Routes tested:
///   GET  /tables/{t}/cdc/stats           → event_count, byte_size, min_lsn, max_lsn
///   POST /tables/{t}/cdc/compact?retain_lsn=<n> → removes old events
use std::sync::Arc;
use std::time::Duration;

use reqwest::Client;
use serde_json::Value;
use tempfile::TempDir;
use vectorvolt_engine::{EngineOptions, TableRegistry};
use vectorvolt_http::{HttpServer, HttpServerConfig};

const FLUSH: usize = 1_048_576;

// ── harness ───────────────────────────────────────────────────────────────────

struct Harness {
    base_url: String,
    _dir: TempDir,
    client: Client,
}

impl Harness {
    async fn start() -> Self {
        let dir = TempDir::new().unwrap();
        let opts = EngineOptions::new(FLUSH);
        let reg = Arc::new(TableRegistry::open(dir.path(), opts).unwrap());
        let cfg = HttpServerConfig::new("127.0.0.1:0".parse().unwrap());
        let srv = HttpServer::bind(reg, cfg).await.unwrap();
        let addr = srv.local_addr().unwrap();
        tokio::spawn(async move { srv.serve().await.unwrap() });
        let client = Client::builder()
            .timeout(Duration::from_secs(5))
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

    async fn stats(&self, table: &str) -> Value {
        self.client
            .get(self.url(&format!("/tables/{table}/cdc/stats")))
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap()
    }

    async fn compact(&self, table: &str, retain_lsn: u64) -> Value {
        self.client
            .post(self.url(&format!(
                "/tables/{table}/cdc/compact?retain_lsn={retain_lsn}"
            )))
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap()
    }

    async fn log_count(&self, table: &str) -> u64 {
        let resp: Value = self
            .client
            .get(self.url(&format!("/tables/{table}/cdc/log")))
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();
        resp["count"].as_u64().unwrap()
    }
}

// ── stats tests ───────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_stats_empty_table() {
    let h = Harness::start().await;
    h.create_table("t", &["id"]).await;
    let s = h.stats("t").await;
    assert_eq!(s["event_count"], 0);
    assert_eq!(s["min_lsn"], 0);
    assert_eq!(s["max_lsn"], 0);
}

#[tokio::test]
async fn test_stats_unknown_table_returns_404() {
    let h = Harness::start().await;
    let resp = h
        .client
        .get(h.url("/tables/nope/cdc/stats"))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 404);
}

#[tokio::test]
async fn test_stats_event_count_after_inserts() {
    let h = Harness::start().await;
    h.create_table("t", &["id"]).await;
    h.insert("t", 1, &[1]).await;
    h.insert("t", 2, &[2]).await;
    h.insert("t", 3, &[3]).await;
    let s = h.stats("t").await;
    assert_eq!(s["event_count"], 3);
}

#[tokio::test]
async fn test_stats_byte_size_grows_with_events() {
    let h = Harness::start().await;
    h.create_table("t", &["id"]).await;
    let s0 = h.stats("t").await;
    h.insert("t", 1, &[1]).await;
    let s1 = h.stats("t").await;
    assert!(s1["byte_size"].as_u64().unwrap() > s0["byte_size"].as_u64().unwrap());
}

#[tokio::test]
async fn test_stats_min_max_lsn() {
    let h = Harness::start().await;
    h.create_table("t", &["id"]).await;
    h.insert("t", 1, &[1]).await;
    h.insert("t", 2, &[2]).await;
    h.insert("t", 3, &[3]).await;
    let s = h.stats("t").await;
    assert_eq!(s["min_lsn"], 1);
    assert_eq!(s["max_lsn"], 3);
}

#[tokio::test]
async fn test_stats_counts_deletes() {
    let h = Harness::start().await;
    h.create_table("t", &["id"]).await;
    h.insert("t", 1, &[1]).await;
    h.delete_row("t", 1).await;
    let s = h.stats("t").await;
    assert_eq!(s["event_count"], 2);
}

// ── compact tests ─────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_compact_unknown_table_returns_404() {
    let h = Harness::start().await;
    let resp = h
        .client
        .post(h.url("/tables/nope/cdc/compact?retain_lsn=1"))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 404);
}

#[tokio::test]
async fn test_compact_empty_log_removes_zero() {
    let h = Harness::start().await;
    h.create_table("t", &["id"]).await;
    let r = h.compact("t", 1).await;
    assert_eq!(r["removed"], 0);
}

#[tokio::test]
async fn test_compact_retain_all_removes_zero() {
    let h = Harness::start().await;
    h.create_table("t", &["id"]).await;
    h.insert("t", 1, &[1]).await;
    h.insert("t", 2, &[2]).await;
    // retain_lsn = 1 keeps everything (lsn >= 1)
    let r = h.compact("t", 1).await;
    assert_eq!(r["removed"], 0);
    assert_eq!(h.log_count("t").await, 2);
}

#[tokio::test]
async fn test_compact_removes_old_events() {
    let h = Harness::start().await;
    h.create_table("t", &["id"]).await;
    h.insert("t", 1, &[1]).await;
    h.insert("t", 2, &[2]).await;
    h.insert("t", 3, &[3]).await;
    // LSNs are 1,2,3 — retain from 3, removes 1 and 2
    let r = h.compact("t", 3).await;
    assert_eq!(r["removed"], 2);
    assert_eq!(h.log_count("t").await, 1);
}

#[tokio::test]
async fn test_compact_retain_lsn_beyond_max_removes_all() {
    let h = Harness::start().await;
    h.create_table("t", &["id"]).await;
    h.insert("t", 1, &[1]).await;
    h.insert("t", 2, &[2]).await;
    let r = h.compact("t", 999).await;
    assert_eq!(r["removed"], 2);
    assert_eq!(h.log_count("t").await, 0);
}

#[tokio::test]
async fn test_compact_events_still_readable_after_compact() {
    let h = Harness::start().await;
    h.create_table("t", &["id"]).await;
    h.insert("t", 1, &[1]).await;
    h.insert("t", 2, &[2]).await;
    h.insert("t", 3, &[3]).await;
    h.compact("t", 2).await;
    // Events with lsn >= 2 remain
    let log: Value = h
        .client
        .get(h.url("/tables/t/cdc/log"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let events = log["events"].as_array().unwrap();
    assert_eq!(events.len(), 2);
    assert_eq!(events[0]["lsn"], 2);
    assert_eq!(events[1]["lsn"], 3);
}

#[tokio::test]
async fn test_stats_reflect_compaction() {
    let h = Harness::start().await;
    h.create_table("t", &["id"]).await;
    h.insert("t", 1, &[1]).await;
    h.insert("t", 2, &[2]).await;
    h.insert("t", 3, &[3]).await;
    let before = h.stats("t").await;
    assert_eq!(before["event_count"], 3);
    h.compact("t", 3).await; // remove lsn 1 and 2
    let after = h.stats("t").await;
    assert_eq!(after["event_count"], 1);
    assert_eq!(after["min_lsn"], 3);
    assert_eq!(after["max_lsn"], 3);
}

#[tokio::test]
async fn test_compact_then_insert_appends_correctly() {
    let h = Harness::start().await;
    h.create_table("t", &["id"]).await;
    h.insert("t", 1, &[1]).await;
    h.insert("t", 2, &[2]).await;
    h.compact("t", 999).await; // remove all
    // New inserts should still work and be readable
    h.insert("t", 3, &[3]).await;
    assert_eq!(h.log_count("t").await, 1);
    let log: Value = h
        .client
        .get(h.url("/tables/t/cdc/log"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let events = log["events"].as_array().unwrap();
    assert_eq!(events[0]["pk"], 3);
}
