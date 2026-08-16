/// Layer 44 — HTTP gateway expansion: analytical + write/mutate endpoints.
///
/// New routes under test:
///   GET  /tables/{t}/count?col=&lo=&hi=
///   GET  /tables/{t}/sum?col=&lo=&hi=
///   GET  /tables/{t}/min?col=&lo=&hi=
///   GET  /tables/{t}/max?col=&lo=&hi=
///   GET  /tables/{t}/avg?col=&lo=&hi=
///   GET  /tables/{t}/scan/sorted?col=&lo=&hi=&sort=&asc=&limit=
///   GET  /tables/{t}/scan/page?col=&lo=&hi=&after=&limit=
///   PUT  /tables/{t}/rows/{pk}         (upsert)
///   PATCH /tables/{t}/rows/{pk}/field  (update_field)
///   DELETE /tables/{t}/rows/range?col=&lo=&hi=
use std::sync::Arc;

use reqwest::Client;
use serde_json::Value;
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
    async fn start_with_table(name: &str, cols: &[&str]) -> Self {
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
        let base = format!("http://{addr}");
        // create the table
        let body = serde_json::json!({ "columns": cols });
        client
            .post(format!("{base}/tables/{name}"))
            .json(&body)
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

// ── COUNT ─────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn count_returns_matching_row_count() {
    let h = Harness::start_with_table("t", &["pk", "val"]).await;
    for i in 1u64..=5 {
        h.insert("t", i, &[i, i * 10]).await;
    }
    let resp: Value = h
        .client
        .get(h.url("/tables/t/count?col=1&lo=10&hi=30"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(resp["count"], 3);
}

#[tokio::test]
async fn count_zero_when_no_match() {
    let h = Harness::start_with_table("t", &["pk", "val"]).await;
    h.insert("t", 1, &[1, 99]).await;
    let resp: Value = h
        .client
        .get(h.url("/tables/t/count?col=1&lo=0&hi=10"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(resp["count"], 0);
}

// ── SUM ───────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn sum_returns_total() {
    let h = Harness::start_with_table("t", &["pk", "val"]).await;
    for i in 1u64..=4 {
        h.insert("t", i, &[i, i]).await; // sum = 10
    }
    let resp: Value = h
        .client
        .get(h.url("/tables/t/sum?col=1&lo=1&hi=4"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(resp["value"], 10);
}

// ── MIN / MAX ─────────────────────────────────────────────────────────────────

#[tokio::test]
async fn min_returns_smallest() {
    let h = Harness::start_with_table("t", &["pk", "val"]).await;
    h.insert("t", 1, &[1, 50]).await;
    h.insert("t", 2, &[2, 10]).await;
    h.insert("t", 3, &[3, 80]).await;
    let resp: Value = h
        .client
        .get(h.url("/tables/t/min?col=1&lo=1&hi=100"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(resp["value"], 10);
}

#[tokio::test]
async fn max_returns_largest() {
    let h = Harness::start_with_table("t", &["pk", "val"]).await;
    h.insert("t", 1, &[1, 50]).await;
    h.insert("t", 2, &[2, 10]).await;
    h.insert("t", 3, &[3, 80]).await;
    let resp: Value = h
        .client
        .get(h.url("/tables/t/max?col=1&lo=1&hi=100"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(resp["value"], 80);
}

// ── AVG ───────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn avg_returns_rounded_mean() {
    let h = Harness::start_with_table("t", &["pk", "val"]).await;
    h.insert("t", 1, &[1, 10]).await;
    h.insert("t", 2, &[2, 20]).await;
    h.insert("t", 3, &[3, 30]).await; // avg = 20
    let resp: Value = h
        .client
        .get(h.url("/tables/t/avg?col=1&lo=1&hi=100"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(resp["value"], 20);
}

#[tokio::test]
async fn avg_null_when_no_rows() {
    let h = Harness::start_with_table("t", &["pk", "val"]).await;
    let resp: Value = h
        .client
        .get(h.url("/tables/t/avg?col=1&lo=1&hi=100"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert!(resp["value"].is_null(), "empty table → null avg");
}

// ── SCAN SORTED ───────────────────────────────────────────────────────────────

#[tokio::test]
async fn scan_sorted_ascending() {
    let h = Harness::start_with_table("t", &["pk", "val"]).await;
    h.insert("t", 1, &[1, 30]).await;
    h.insert("t", 2, &[2, 10]).await;
    h.insert("t", 3, &[3, 20]).await;
    let resp: Value = h
        .client
        .get(h.url("/tables/t/scan/sorted?col=0&lo=1&hi=3&sort=1&asc=1&limit=10"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let vals: Vec<u64> = resp["rows"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| r["values"][1].as_u64().unwrap())
        .collect();
    assert_eq!(vals, vec![10, 20, 30]);
}

// ── SCAN PAGE ─────────────────────────────────────────────────────────────────

#[tokio::test]
async fn scan_page_returns_first_page_and_cursor() {
    let h = Harness::start_with_table("t", &["pk", "val"]).await;
    for i in 1u64..=5 {
        h.insert("t", i, &[i, i]).await;
    }
    let resp: Value = h
        .client
        .get(h.url("/tables/t/scan/page?col=0&lo=1&hi=5&after=0&limit=3"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(resp["rows"].as_array().unwrap().len(), 3);
    assert!(!resp["next_cursor"].is_null(), "must have a next cursor");
}

#[tokio::test]
async fn scan_page_second_page_exhausts() {
    let h = Harness::start_with_table("t", &["pk", "val"]).await;
    for i in 1u64..=5 {
        h.insert("t", i, &[i, i]).await;
    }
    let cursor = h
        .client
        .get(h.url("/tables/t/scan/page?col=0&lo=1&hi=5&after=0&limit=3"))
        .send()
        .await
        .unwrap()
        .json::<Value>()
        .await
        .unwrap()["next_cursor"]
        .as_u64()
        .unwrap();
    let resp: Value = h
        .client
        .get(h.url(&format!(
            "/tables/t/scan/page?col=0&lo=1&hi=5&after={cursor}&limit=3"
        )))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(resp["rows"].as_array().unwrap().len(), 2);
    assert!(resp["next_cursor"].is_null(), "second page is the last");
}

// ── UPSERT ────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn upsert_inserts_new_row() {
    let h = Harness::start_with_table("t", &["pk", "val"]).await;
    let status = h
        .client
        .put(h.url("/tables/t/rows/1"))
        .json(&serde_json::json!({ "values": [1, 42] }))
        .send()
        .await
        .unwrap()
        .status()
        .as_u16();
    assert_eq!(status, 200);
    let row: Value = h
        .client
        .get(h.url("/tables/t/rows/1"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(row["values"][1], 42);
}

#[tokio::test]
async fn upsert_overwrites_existing_row() {
    let h = Harness::start_with_table("t", &["pk", "val"]).await;
    h.insert("t", 1, &[1, 10]).await;
    h.client
        .put(h.url("/tables/t/rows/1"))
        .json(&serde_json::json!({ "values": [1, 99] }))
        .send()
        .await
        .unwrap();
    let row: Value = h
        .client
        .get(h.url("/tables/t/rows/1"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(row["values"][1], 99, "upsert must overwrite");
}

// ── UPDATE FIELD ──────────────────────────────────────────────────────────────

#[tokio::test]
async fn update_field_changes_one_column() {
    let h = Harness::start_with_table("t", &["pk", "val"]).await;
    h.insert("t", 1, &[1, 5]).await;
    let status = h
        .client
        .patch(h.url("/tables/t/rows/1/field"))
        .json(&serde_json::json!({ "col": 1, "val": 77 }))
        .send()
        .await
        .unwrap()
        .status()
        .as_u16();
    assert_eq!(status, 200);
    let row: Value = h
        .client
        .get(h.url("/tables/t/rows/1"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(row["values"][1], 77);
}

// ── DELETE RANGE ──────────────────────────────────────────────────────────────

#[tokio::test]
async fn delete_range_removes_rows_and_returns_count() {
    let h = Harness::start_with_table("t", &["pk", "val"]).await;
    for i in 1u64..=5 {
        h.insert("t", i, &[i, i]).await;
    }
    let resp: Value = h
        .client
        .delete(h.url("/tables/t/rows/range?col=0&lo=2&hi=4"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(resp["count"], 3, "pks 2,3,4 deleted");
    // pk=1 and pk=5 must survive
    assert_eq!(
        h.client
            .get(h.url("/tables/t/rows/1"))
            .send()
            .await
            .unwrap()
            .status()
            .as_u16(),
        200
    );
    assert_eq!(
        h.client
            .get(h.url("/tables/t/rows/3"))
            .send()
            .await
            .unwrap()
            .status()
            .as_u16(),
        404
    );
}

// ── cross-cutting: unknown table returns 4xx ──────────────────────────────────

#[tokio::test]
async fn count_unknown_table_returns_error() {
    let h = Harness::start_with_table("t", &["pk", "val"]).await;
    let status = h
        .client
        .get(h.url("/tables/no_such_table/count?col=0&lo=0&hi=100"))
        .send()
        .await
        .unwrap()
        .status()
        .as_u16();
    assert!(status >= 400, "unknown table must return 4xx, got {status}");
}
