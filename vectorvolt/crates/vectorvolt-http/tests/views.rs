/// Layer 66 — Materialized Views over CDC.
///
/// Routes:
///   POST   /views           → create view (201 Created)
///   GET    /views           → list views
///   GET    /views/{name}    → current aggregate state
///   DELETE /views/{name}    → drop view
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

    async fn create_view(&self, body: serde_json::Value) -> reqwest::Response {
        self.client
            .post(self.url("/views"))
            .json(&body)
            .send()
            .await
            .unwrap()
    }

    async fn get_view(&self, name: &str) -> Value {
        self.client
            .get(self.url(&format!("/views/{name}")))
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap()
    }

    async fn get_view_entry(&self, name: &str, group_val: u64) -> Option<u64> {
        let v = self.get_view(name).await;
        v["entries"]
            .as_array()?
            .iter()
            .find(|e| e["group_val"] == group_val)
            .and_then(|e| e["agg_val"].as_u64())
    }

    /// Small sleep so the background view task processes events.
    async fn wait_for_view() {
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
}

// ── tests ─────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_create_view_returns_201() {
    let h = Harness::start().await;
    h.create_table("orders", &["id", "dept", "amount"]).await;
    let resp = h
        .create_view(serde_json::json!({
            "name": "v1", "table": "orders",
            "filter_col": 0, "lo": 0, "hi": 9999,
            "group_col": 1, "kind": "count"
        }))
        .await;
    assert_eq!(resp.status().as_u16(), 201);
}

#[tokio::test]
async fn test_create_view_unknown_table_returns_error() {
    let h = Harness::start().await;
    let resp = h
        .create_view(serde_json::json!({
            "name": "v_bad", "table": "no_such_table",
            "filter_col": 0, "lo": 0, "hi": 9999,
            "group_col": 0, "kind": "count"
        }))
        .await;
    assert!(!resp.status().is_success());
}

#[tokio::test]
async fn test_create_view_duplicate_name_returns_conflict() {
    let h = Harness::start().await;
    h.create_table("t", &["a", "b"]).await;
    let body = serde_json::json!({
        "name": "dup", "table": "t",
        "filter_col": 0, "lo": 0, "hi": 9999,
        "group_col": 0, "kind": "count"
    });
    h.create_view(body.clone()).await;
    let resp = h.create_view(body).await;
    assert_eq!(resp.status().as_u16(), 409);
}

#[tokio::test]
async fn test_list_views_empty() {
    let h = Harness::start().await;
    let v: Value = h
        .client
        .get(h.url("/views"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(v["views"].as_array().unwrap().len(), 0);
}

#[tokio::test]
async fn test_list_views_shows_created() {
    let h = Harness::start().await;
    h.create_table("sales", &["id", "region", "rev"]).await;
    h.create_view(serde_json::json!({
        "name": "sales_count", "table": "sales",
        "filter_col": 0, "lo": 0, "hi": 9999,
        "group_col": 1, "kind": "count"
    }))
    .await;
    let v: Value = h
        .client
        .get(h.url("/views"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let views = v["views"].as_array().unwrap();
    assert_eq!(views.len(), 1);
    assert_eq!(views[0]["name"], "sales_count");
    assert_eq!(views[0]["kind"], "count");
}

#[tokio::test]
async fn test_get_view_not_found() {
    let h = Harness::start().await;
    let resp = h
        .client
        .get(h.url("/views/nonexistent"))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status().as_u16(), 404);
}

#[tokio::test]
async fn test_view_count_increments_on_insert() {
    let h = Harness::start().await;
    h.create_table("events", &["id", "type", "val"]).await;
    h.create_view(serde_json::json!({
        "name": "ev_count", "table": "events",
        "filter_col": 0, "lo": 0, "hi": 9999,
        "group_col": 1, "kind": "count"
    }))
    .await;
    // group=10, group=10, group=20
    h.insert("events", 1, &[1, 10, 100]).await;
    h.insert("events", 2, &[2, 10, 200]).await;
    h.insert("events", 3, &[3, 20, 150]).await;
    Harness::wait_for_view().await;
    assert_eq!(h.get_view_entry("ev_count", 10).await, Some(2));
    assert_eq!(h.get_view_entry("ev_count", 20).await, Some(1));
}

#[tokio::test]
async fn test_view_count_decrements_on_delete() {
    let h = Harness::start().await;
    h.create_table("items", &["id", "cat", "price"]).await;
    h.create_view(serde_json::json!({
        "name": "item_count", "table": "items",
        "filter_col": 0, "lo": 0, "hi": 9999,
        "group_col": 1, "kind": "count"
    }))
    .await;
    h.insert("items", 1, &[1, 5, 100]).await;
    h.insert("items", 2, &[2, 5, 200]).await;
    Harness::wait_for_view().await;
    assert_eq!(h.get_view_entry("item_count", 5).await, Some(2));
    h.delete_row("items", 1).await;
    Harness::wait_for_view().await;
    assert_eq!(h.get_view_entry("item_count", 5).await, Some(1));
}

#[tokio::test]
async fn test_view_sum_on_insert() {
    let h = Harness::start().await;
    h.create_table("txns", &["id", "region", "amount"]).await;
    h.create_view(serde_json::json!({
        "name": "txn_sum", "table": "txns",
        "filter_col": 0, "lo": 0, "hi": 9999,
        "group_col": 1, "kind": "sum", "agg_col": 2
    }))
    .await;
    h.insert("txns", 1, &[1, 1, 100]).await;
    h.insert("txns", 2, &[2, 1, 200]).await;
    h.insert("txns", 3, &[3, 2, 500]).await;
    Harness::wait_for_view().await;
    assert_eq!(h.get_view_entry("txn_sum", 1).await, Some(300));
    assert_eq!(h.get_view_entry("txn_sum", 2).await, Some(500));
}

#[tokio::test]
async fn test_view_sum_decreases_on_delete() {
    let h = Harness::start().await;
    h.create_table("sales2", &["id", "cat", "amt"]).await;
    h.create_view(serde_json::json!({
        "name": "sum_del", "table": "sales2",
        "filter_col": 0, "lo": 0, "hi": 9999,
        "group_col": 1, "kind": "sum", "agg_col": 2
    }))
    .await;
    h.insert("sales2", 1, &[1, 7, 1000]).await;
    h.insert("sales2", 2, &[2, 7, 500]).await;
    Harness::wait_for_view().await;
    assert_eq!(h.get_view_entry("sum_del", 7).await, Some(1500));
    h.delete_row("sales2", 1).await;
    Harness::wait_for_view().await;
    assert_eq!(h.get_view_entry("sum_del", 7).await, Some(500));
}

#[tokio::test]
async fn test_view_min_tracks_minimum() {
    let h = Harness::start().await;
    h.create_table("temps", &["id", "sensor", "reading"]).await;
    h.create_view(serde_json::json!({
        "name": "temp_min", "table": "temps",
        "filter_col": 0, "lo": 0, "hi": 9999,
        "group_col": 1, "kind": "min", "agg_col": 2
    }))
    .await;
    h.insert("temps", 1, &[1, 3, 80]).await;
    h.insert("temps", 2, &[2, 3, 60]).await;
    h.insert("temps", 3, &[3, 3, 70]).await;
    Harness::wait_for_view().await;
    assert_eq!(h.get_view_entry("temp_min", 3).await, Some(60));
}

#[tokio::test]
async fn test_view_max_tracks_maximum() {
    let h = Harness::start().await;
    h.create_table("scores", &["id", "player", "score"]).await;
    h.create_view(serde_json::json!({
        "name": "high_score", "table": "scores",
        "filter_col": 0, "lo": 0, "hi": 9999,
        "group_col": 1, "kind": "max", "agg_col": 2
    }))
    .await;
    h.insert("scores", 1, &[1, 42, 100]).await;
    h.insert("scores", 2, &[2, 42, 500]).await;
    h.insert("scores", 3, &[3, 42, 300]).await;
    Harness::wait_for_view().await;
    assert_eq!(h.get_view_entry("high_score", 42).await, Some(500));
}

#[tokio::test]
async fn test_view_filter_excludes_out_of_range_rows() {
    let h = Harness::start().await;
    h.create_table("filt", &["id", "grp", "val"]).await;
    h.create_view(serde_json::json!({
        "name": "filt_view", "table": "filt",
        "filter_col": 0, "lo": 10, "hi": 20,
        "group_col": 1, "kind": "count"
    }))
    .await;
    h.insert("filt", 5, &[5, 1, 0]).await; // id=5 < lo=10, excluded
    h.insert("filt", 15, &[15, 1, 0]).await; // id=15 in range, included
    h.insert("filt", 25, &[25, 1, 0]).await; // id=25 > hi=20, excluded
    Harness::wait_for_view().await;
    assert_eq!(h.get_view_entry("filt_view", 1).await, Some(1));
}

#[tokio::test]
async fn test_view_upsert_updates_aggregate() {
    let h = Harness::start().await;
    h.create_table("upsert_t", &["id", "grp", "val"]).await;
    h.create_view(serde_json::json!({
        "name": "upsert_sum", "table": "upsert_t",
        "filter_col": 0, "lo": 0, "hi": 9999,
        "group_col": 1, "kind": "sum", "agg_col": 2
    }))
    .await;
    // Insert pk=1 with val=100
    h.insert("upsert_t", 1, &[1, 5, 100]).await;
    Harness::wait_for_view().await;
    assert_eq!(h.get_view_entry("upsert_sum", 5).await, Some(100));
    // Upsert pk=1 with val=200 (PUT /rows/{pk})
    h.client
        .put(h.url("/tables/upsert_t/rows/1"))
        .json(&serde_json::json!({ "values": [1, 5, 200] }))
        .send()
        .await
        .unwrap();
    Harness::wait_for_view().await;
    // Old contribution (100) removed, new (200) added
    assert_eq!(h.get_view_entry("upsert_sum", 5).await, Some(200));
}

#[tokio::test]
async fn test_drop_view_removes_it() {
    let h = Harness::start().await;
    h.create_table("drop_t", &["id", "grp"]).await;
    h.create_view(serde_json::json!({
        "name": "to_drop", "table": "drop_t",
        "filter_col": 0, "lo": 0, "hi": 9999,
        "group_col": 1, "kind": "count"
    }))
    .await;
    let resp = h
        .client
        .delete(h.url("/views/to_drop"))
        .send()
        .await
        .unwrap();
    assert!(resp.status().is_success());
    // Should now be 404
    let resp2 = h.client.get(h.url("/views/to_drop")).send().await.unwrap();
    assert_eq!(resp2.status().as_u16(), 404);
}

#[tokio::test]
async fn test_drop_view_not_found() {
    let h = Harness::start().await;
    let resp = h.client.delete(h.url("/views/ghost")).send().await.unwrap();
    assert_eq!(resp.status().as_u16(), 404);
}

#[tokio::test]
async fn test_view_sum_missing_agg_col_returns_400() {
    let h = Harness::start().await;
    h.create_table("t2", &["a", "b"]).await;
    let resp = h
        .create_view(serde_json::json!({
            "name": "bad_sum", "table": "t2",
            "filter_col": 0, "lo": 0, "hi": 9999,
            "group_col": 0, "kind": "sum"
        }))
        .await;
    assert_eq!(resp.status().as_u16(), 400);
}

#[tokio::test]
async fn test_multiple_views_on_same_table() {
    let h = Harness::start().await;
    h.create_table("multi", &["id", "region", "salary"]).await;
    h.create_view(serde_json::json!({
        "name": "headcount", "table": "multi",
        "filter_col": 0, "lo": 0, "hi": 9999,
        "group_col": 1, "kind": "count"
    }))
    .await;
    h.create_view(serde_json::json!({
        "name": "payroll", "table": "multi",
        "filter_col": 0, "lo": 0, "hi": 9999,
        "group_col": 1, "kind": "sum", "agg_col": 2
    }))
    .await;
    h.insert("multi", 1, &[1, 10, 5000]).await;
    h.insert("multi", 2, &[2, 10, 6000]).await;
    Harness::wait_for_view().await;
    assert_eq!(h.get_view_entry("headcount", 10).await, Some(2));
    assert_eq!(h.get_view_entry("payroll", 10).await, Some(11000));
}

#[tokio::test]
async fn test_view_get_returns_all_groups() {
    let h = Harness::start().await;
    h.create_table("grps", &["id", "dept", "cost"]).await;
    h.create_view(serde_json::json!({
        "name": "dept_cost", "table": "grps",
        "filter_col": 0, "lo": 0, "hi": 9999,
        "group_col": 1, "kind": "sum", "agg_col": 2
    }))
    .await;
    h.insert("grps", 1, &[1, 1, 100]).await;
    h.insert("grps", 2, &[2, 2, 200]).await;
    h.insert("grps", 3, &[3, 3, 300]).await;
    Harness::wait_for_view().await;
    let v = h.get_view("dept_cost").await;
    let entries = v["entries"].as_array().unwrap();
    assert_eq!(entries.len(), 3);
}
