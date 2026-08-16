/// Layer 54 — HTTP bulk predicate field-update endpoints, and DSL UPDATE/INCREMENT/DECREMENT WHERE.
///
/// New routes:
///   GET /tables/{t}/update_where?filter_col=&lo=&hi=&target_col=&val=
///   GET /tables/{t}/increment_where?filter_col=&lo=&hi=&target_col=&delta=
///   GET /tables/{t}/decrement_where?filter_col=&lo=&hi=&target_col=&delta=
///
/// New DSL statements:
///   UPDATE WHERE     filter_col=<n> lo=<lo> hi=<hi> target_col=<c> val=<v>
///   INCREMENT WHERE  filter_col=<n> lo=<lo> hi=<hi> target_col=<c> delta=<d>
///   DECREMENT WHERE  filter_col=<n> lo=<lo> hi=<hi> target_col=<c> delta=<d>
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
}

// ── update_where endpoint ─────────────────────────────────────────────────────

#[tokio::test]
async fn update_where_returns_updated_count() {
    let h = Harness::start("t", &["pk", "status", "val"]).await;
    h.insert("t", 1, &[1, 1, 100]).await;
    h.insert("t", 2, &[2, 1, 200]).await;
    h.insert("t", 3, &[3, 2, 300]).await;
    let resp: Value = h
        .client
        .get(h.url("/tables/t/update_where?filter_col=1&lo=1&hi=1&target_col=2&val=999"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(
        resp["updated"].as_u64().unwrap(),
        2,
        "two rows matched status=1"
    );
}

#[tokio::test]
async fn update_where_modifies_correct_rows() {
    let h = Harness::start("t", &["pk", "cat", "score"]).await;
    h.insert("t", 1, &[1, 10, 5]).await;
    h.insert("t", 2, &[2, 10, 8]).await;
    h.insert("t", 3, &[3, 20, 3]).await;
    h.client
        .get(h.url("/tables/t/update_where?filter_col=1&lo=10&hi=10&target_col=2&val=42"))
        .send()
        .await
        .unwrap();
    let r1 = h.get_row("t", 1).await;
    let r2 = h.get_row("t", 2).await;
    let r3 = h.get_row("t", 3).await;
    assert_eq!(
        r1["values"][2].as_u64().unwrap(),
        42,
        "row 1 score must be 42"
    );
    assert_eq!(
        r2["values"][2].as_u64().unwrap(),
        42,
        "row 2 score must be 42"
    );
    assert_eq!(
        r3["values"][2].as_u64().unwrap(),
        3,
        "row 3 score unchanged"
    );
}

#[tokio::test]
async fn update_where_no_match_returns_zero() {
    let h = Harness::start("t", &["pk", "v", "x"]).await;
    h.insert("t", 1, &[1, 5, 100]).await;
    let resp: Value = h
        .client
        .get(h.url("/tables/t/update_where?filter_col=1&lo=99&hi=99&target_col=2&val=0"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(resp["updated"].as_u64().unwrap(), 0);
}

// ── increment_where endpoint ──────────────────────────────────────────────────

#[tokio::test]
async fn increment_where_increases_field_value() {
    let h = Harness::start("t", &["pk", "cat", "counter"]).await;
    h.insert("t", 1, &[1, 5, 10]).await;
    h.client
        .get(h.url("/tables/t/increment_where?filter_col=1&lo=5&hi=5&target_col=2&delta=3"))
        .send()
        .await
        .unwrap();
    let row = h.get_row("t", 1).await;
    assert_eq!(row["values"][2].as_u64().unwrap(), 13);
}

#[tokio::test]
async fn increment_where_returns_updated_count() {
    let h = Harness::start("t", &["pk", "flag", "n"]).await;
    for pk in 0u64..5 {
        h.insert("t", pk, &[pk, pk % 2, 1]).await;
    }
    let resp: Value = h
        .client
        .get(h.url("/tables/t/increment_where?filter_col=1&lo=1&hi=1&target_col=2&delta=10"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    // pks 1, 3 have flag=1 → 2 rows
    assert_eq!(resp["updated"].as_u64().unwrap(), 2);
}

#[tokio::test]
async fn increment_where_leaves_non_matching_rows_unchanged() {
    let h = Harness::start("t", &["pk", "type", "cnt"]).await;
    h.insert("t", 1, &[1, 1, 0]).await;
    h.insert("t", 2, &[2, 2, 0]).await;
    h.client
        .get(h.url("/tables/t/increment_where?filter_col=1&lo=1&hi=1&target_col=2&delta=5"))
        .send()
        .await
        .unwrap();
    let r2 = h.get_row("t", 2).await;
    assert_eq!(
        r2["values"][2].as_u64().unwrap(),
        0,
        "non-matching row must be unchanged"
    );
}

// ── decrement_where endpoint ──────────────────────────────────────────────────

#[tokio::test]
async fn decrement_where_decreases_field_value() {
    let h = Harness::start("t", &["pk", "cat", "stock"]).await;
    h.insert("t", 1, &[1, 7, 50]).await;
    h.client
        .get(h.url("/tables/t/decrement_where?filter_col=1&lo=7&hi=7&target_col=2&delta=5"))
        .send()
        .await
        .unwrap();
    let row = h.get_row("t", 1).await;
    assert_eq!(row["values"][2].as_u64().unwrap(), 45);
}

#[tokio::test]
async fn decrement_where_returns_updated_count() {
    let h = Harness::start("t", &["pk", "zone", "qty"]).await;
    for pk in 0u64..4 {
        h.insert("t", pk, &[pk, 3, 100]).await;
    }
    let resp: Value = h
        .client
        .get(h.url("/tables/t/decrement_where?filter_col=1&lo=3&hi=3&target_col=2&delta=1"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(resp["updated"].as_u64().unwrap(), 4);
}

#[tokio::test]
async fn decrement_where_no_match_returns_zero() {
    let h = Harness::start("t", &["pk", "v", "x"]).await;
    h.insert("t", 1, &[1, 1, 100]).await;
    let resp: Value = h
        .client
        .get(h.url("/tables/t/decrement_where?filter_col=1&lo=99&hi=99&target_col=2&delta=1"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(resp["updated"].as_u64().unwrap(), 0);
}

// ── DSL bridge ────────────────────────────────────────────────────────────────

#[tokio::test]
async fn dsl_update_where_returns_updated_count() {
    let h = Harness::start("t", &["pk", "status", "v"]).await;
    h.insert("t", 1, &[1, 1, 10]).await;
    h.insert("t", 2, &[2, 1, 20]).await;
    let resp = h
        .client
        .post(h.url("/tables/t/query"))
        .header("content-type", "text/plain")
        .body("UPDATE WHERE filter_col=1 lo=1 hi=1 target_col=2 val=99")
        .send()
        .await
        .unwrap();
    assert!(resp.status().is_success());
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["updated"].as_u64().unwrap(), 2);
}

#[tokio::test]
async fn dsl_increment_where_returns_updated() {
    let h = Harness::start("t", &["pk", "cat", "n"]).await;
    h.insert("t", 1, &[1, 5, 0]).await;
    let resp = h
        .client
        .post(h.url("/tables/t/query"))
        .header("content-type", "text/plain")
        .body("INCREMENT WHERE filter_col=1 lo=5 hi=5 target_col=2 delta=7")
        .send()
        .await
        .unwrap();
    assert!(resp.status().is_success());
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["updated"].as_u64().unwrap(), 1);
}

#[tokio::test]
async fn dsl_decrement_where_returns_updated() {
    let h = Harness::start("t", &["pk", "cat", "n"]).await;
    h.insert("t", 1, &[1, 3, 100]).await;
    let resp = h
        .client
        .post(h.url("/tables/t/query"))
        .header("content-type", "text/plain")
        .body("DECREMENT WHERE filter_col=1 lo=3 hi=3 target_col=2 delta=10")
        .send()
        .await
        .unwrap();
    assert!(resp.status().is_success());
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["updated"].as_u64().unwrap(), 1);
}

#[tokio::test]
async fn dsl_http_update_where_parity() {
    // Two separate tables with identical data so HTTP and DSL counts can be compared.
    let h = Harness::start("http_t", &["pk", "grp", "score"]).await;
    // Create second table
    h.client
        .post(h.url("/tables/dsl_t"))
        .json(&serde_json::json!({ "columns": ["pk", "grp", "score"] }))
        .send()
        .await
        .unwrap();
    for pk in 0u64..6 {
        h.insert("http_t", pk, &[pk, pk % 3, 10]).await;
        h.insert("dsl_t", pk, &[pk, pk % 3, 10]).await;
    }
    let http_resp: Value = h
        .client
        .get(h.url("/tables/http_t/update_where?filter_col=1&lo=0&hi=0&target_col=2&val=99"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let dsl_resp: Value = h
        .client
        .post(h.url("/tables/dsl_t/query"))
        .header("content-type", "text/plain")
        .body("UPDATE WHERE filter_col=1 lo=0 hi=0 target_col=2 val=99")
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(
        http_resp["updated"], dsl_resp["updated"],
        "HTTP and DSL UPDATE WHERE must return the same updated count"
    );
}

#[tokio::test]
async fn increment_then_read_back_value() {
    let h = Harness::start("t", &["pk", "grp", "n"]).await;
    h.insert("t", 42, &[42, 1, 100]).await;
    h.client
        .get(h.url("/tables/t/increment_where?filter_col=1&lo=1&hi=1&target_col=2&delta=25"))
        .send()
        .await
        .unwrap();
    let row = h.get_row("t", 42).await;
    assert_eq!(row["values"][2].as_u64().unwrap(), 125);
}
