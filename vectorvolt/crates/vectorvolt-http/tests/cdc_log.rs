/// Layer 67 — Persistent CDC Event Log.
///
/// Routes tested:
///   GET /tables/{t}/cdc/log           → full log
///   GET /tables/{t}/cdc/log?from=<n>  → filtered by LSN
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

    async fn get_log(&self, table: &str) -> Value {
        self.client
            .get(self.url(&format!("/tables/{table}/cdc/log")))
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap()
    }

    async fn get_log_from(&self, table: &str, from: u64) -> Value {
        self.client
            .get(self.url(&format!("/tables/{table}/cdc/log?from={from}")))
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap()
    }
}

// ── tests ─────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_cdc_log_empty_on_new_table() {
    let h = Harness::start().await;
    h.create_table("t", &["id", "val"]).await;
    let resp = h.get_log("t").await;
    assert_eq!(resp["count"], 0);
    assert!(resp["events"].as_array().unwrap().is_empty());
}

#[tokio::test]
async fn test_cdc_log_unknown_table_returns_404() {
    let h = Harness::start().await;
    let resp = h
        .client
        .get(h.url("/tables/nope/cdc/log"))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 404);
}

#[tokio::test]
async fn test_cdc_log_records_insert() {
    let h = Harness::start().await;
    h.create_table("t", &["id", "val"]).await;
    h.insert("t", 1, &[1, 100]).await;
    let resp = h.get_log("t").await;
    assert_eq!(resp["count"], 1);
    let events = resp["events"].as_array().unwrap();
    assert_eq!(events[0]["kind"], "insert");
    assert_eq!(events[0]["pk"], 1);
    assert_eq!(events[0]["lsn"], 1);
}

#[tokio::test]
async fn test_cdc_log_records_delete() {
    let h = Harness::start().await;
    h.create_table("t", &["id"]).await;
    h.insert("t", 5, &[5]).await;
    h.delete_row("t", 5).await;
    let resp = h.get_log("t").await;
    assert_eq!(resp["count"], 2);
    let events = resp["events"].as_array().unwrap();
    assert_eq!(events[1]["kind"], "delete");
    assert_eq!(events[1]["pk"], 5);
}

#[tokio::test]
async fn test_cdc_log_lsn_monotonically_increasing() {
    let h = Harness::start().await;
    h.create_table("t", &["id"]).await;
    for pk in 1u64..=5 {
        h.insert("t", pk, &[pk]).await;
    }
    let resp = h.get_log("t").await;
    let events = resp["events"].as_array().unwrap();
    let lsns: Vec<u64> = events.iter().map(|e| e["lsn"].as_u64().unwrap()).collect();
    for i in 1..lsns.len() {
        assert!(
            lsns[i] > lsns[i - 1],
            "LSNs must be monotonically increasing"
        );
    }
}

#[tokio::test]
async fn test_cdc_log_from_filters_old_events() {
    let h = Harness::start().await;
    h.create_table("t", &["id"]).await;
    h.insert("t", 1, &[1]).await;
    h.insert("t", 2, &[2]).await;
    h.insert("t", 3, &[3]).await;

    let all = h.get_log("t").await;
    assert_eq!(all["count"], 3);

    let from_2 = h.get_log_from("t", 2).await;
    assert_eq!(from_2["count"], 2);
    let events = from_2["events"].as_array().unwrap();
    assert_eq!(events[0]["lsn"], 2);
    assert_eq!(events[1]["lsn"], 3);
}

#[tokio::test]
async fn test_cdc_log_from_0_returns_all() {
    let h = Harness::start().await;
    h.create_table("t", &["id"]).await;
    h.insert("t", 1, &[1]).await;
    h.insert("t", 2, &[2]).await;

    let all = h.get_log("t").await;
    let from0 = h.get_log_from("t", 0).await;
    assert_eq!(all["count"], from0["count"]);
}

#[tokio::test]
async fn test_cdc_log_from_beyond_max_returns_empty() {
    let h = Harness::start().await;
    h.create_table("t", &["id"]).await;
    h.insert("t", 1, &[1]).await;

    let resp = h.get_log_from("t", 999).await;
    assert_eq!(resp["count"], 0);
}

#[tokio::test]
async fn test_cdc_log_insert_has_values() {
    let h = Harness::start().await;
    h.create_table("t", &["id", "score"]).await;
    h.insert("t", 7, &[7, 42]).await;

    let resp = h.get_log("t").await;
    let events = resp["events"].as_array().unwrap();
    let values = events[0]["values"].as_array().unwrap();
    assert_eq!(values[0], 7);
    assert_eq!(values[1], 42);
}

#[tokio::test]
async fn test_cdc_log_delete_has_no_values() {
    let h = Harness::start().await;
    h.create_table("t", &["id"]).await;
    h.insert("t", 3, &[3]).await;
    h.delete_row("t", 3).await;

    let resp = h.get_log("t").await;
    let events = resp["events"].as_array().unwrap();
    let del = &events[1];
    assert_eq!(del["kind"], "delete");
    assert!(del["values"].is_null());
}

#[tokio::test]
async fn test_cdc_log_multiple_tables_isolated() {
    let h = Harness::start().await;
    h.create_table("a", &["id"]).await;
    h.create_table("b", &["id"]).await;
    h.insert("a", 1, &[1]).await;
    h.insert("a", 2, &[2]).await;
    h.insert("b", 10, &[10]).await;

    let a_log = h.get_log("a").await;
    let b_log = h.get_log("b").await;
    assert_eq!(a_log["count"], 2);
    assert_eq!(b_log["count"], 1);
}

#[tokio::test]
async fn test_cdc_log_count_field_matches_events_array_length() {
    let h = Harness::start().await;
    h.create_table("t", &["id"]).await;
    for pk in 1u64..=7 {
        h.insert("t", pk, &[pk]).await;
    }
    let resp = h.get_log("t").await;
    let count = resp["count"].as_u64().unwrap() as usize;
    let len = resp["events"].as_array().unwrap().len();
    assert_eq!(count, len);
}

#[tokio::test]
async fn test_view_backfill_reflects_historical_data() {
    let h = Harness::start().await;
    h.create_table("t", &["id", "cat", "val"]).await;
    // Insert rows BEFORE creating the view.
    h.insert("t", 1, &[1, 0, 10]).await;
    h.insert("t", 2, &[2, 0, 20]).await;
    h.insert("t", 3, &[3, 1, 30]).await;

    // Create view WITH backfill — it should see all three rows.
    let resp = h
        .client
        .post(h.url("/views"))
        .json(&serde_json::json!({
            "name": "vsum",
            "table": "t",
            "filter_col": 0,
            "lo": 0,
            "hi": 99,
            "group_col": 1,
            "kind": "sum",
            "agg_col": 2,
            "backfill": true
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 201);

    // Give the background task time to settle.
    tokio::time::sleep(Duration::from_millis(50)).await;

    let v = h
        .client
        .get(h.url("/views/vsum"))
        .send()
        .await
        .unwrap()
        .json::<Value>()
        .await
        .unwrap();

    let entries = v["entries"].as_array().unwrap();
    let sum_for = |group: u64| -> Option<u64> {
        entries
            .iter()
            .find(|e| e["group_val"] == group)
            .map(|e| e["agg_val"].as_u64().unwrap())
    };
    assert_eq!(sum_for(0), Some(30)); // 10 + 20
    assert_eq!(sum_for(1), Some(30)); // 30
}

#[tokio::test]
async fn test_view_no_backfill_does_not_reflect_historical_data() {
    let h = Harness::start().await;
    h.create_table("t", &["id", "cat"]).await;
    // Insert BEFORE view — without backfill these should be invisible.
    h.insert("t", 1, &[1, 0]).await;
    h.insert("t", 2, &[2, 0]).await;

    let resp = h
        .client
        .post(h.url("/views"))
        .json(&serde_json::json!({
            "name": "vcount",
            "table": "t",
            "filter_col": 0,
            "lo": 0,
            "hi": 99,
            "group_col": 1,
            "kind": "count"
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 201);

    tokio::time::sleep(Duration::from_millis(50)).await;

    let v = h
        .client
        .get(h.url("/views/vcount"))
        .send()
        .await
        .unwrap()
        .json::<Value>()
        .await
        .unwrap();

    // Without backfill the view starts empty.
    let entries = v["entries"].as_array().unwrap();
    assert!(entries.is_empty());
}
