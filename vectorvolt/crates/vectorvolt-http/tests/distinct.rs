/// Layer 53 — HTTP distinct + value_counts endpoints, and DSL DISTINCT / VALUE_COUNTS.
///
/// New routes:
///   GET /tables/{t}/distinct?col=&lo=&hi=
///   GET /tables/{t}/value_counts?col=&lo=&hi=
///
/// New DSL statements:
///   DISTINCT     col=<n> lo=<lo> hi=<hi>
///   VALUE_COUNTS col=<n> lo=<lo> hi=<hi>
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
}

// ── distinct endpoint ─────────────────────────────────────────────────────────

#[tokio::test]
async fn distinct_returns_unique_values_only() {
    let h = Harness::start("t", &["pk", "cat"]).await;
    // 4 rows, 2 distinct categories
    h.insert("t", 1, &[1, 10]).await;
    h.insert("t", 2, &[2, 10]).await;
    h.insert("t", 3, &[3, 20]).await;
    h.insert("t", 4, &[4, 20]).await;
    let resp: Value = h
        .client
        .get(h.url("/tables/t/distinct?col=1&lo=10&hi=20"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let vals = resp["values"].as_array().expect("values array");
    assert_eq!(vals.len(), 2, "must return exactly 2 distinct values");
    assert_eq!(vals[0].as_u64().unwrap(), 10);
    assert_eq!(vals[1].as_u64().unwrap(), 20);
}

#[tokio::test]
async fn distinct_values_are_sorted_ascending() {
    let h = Harness::start("t", &["pk", "v"]).await;
    for pk in 0u64..5 {
        h.insert("t", pk, &[pk, (4 - pk) * 10]).await;
    }
    let resp: Value = h
        .client
        .get(h.url("/tables/t/distinct?col=1&lo=0&hi=40"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let vals: Vec<u64> = resp["values"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_u64().unwrap())
        .collect();
    for i in 1..vals.len() {
        assert!(
            vals[i] > vals[i - 1],
            "distinct values must be sorted ascending"
        );
    }
}

#[tokio::test]
async fn distinct_empty_range_returns_empty() {
    let h = Harness::start("t", &["pk", "v"]).await;
    for pk in 0u64..5 {
        h.insert("t", pk, &[pk, pk]).await;
    }
    let resp: Value = h
        .client
        .get(h.url("/tables/t/distinct?col=1&lo=100&hi=200"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let vals = resp["values"].as_array().unwrap();
    assert!(
        vals.is_empty(),
        "empty range must return no distinct values"
    );
}

#[tokio::test]
async fn distinct_single_value_all_rows_same() {
    let h = Harness::start("t", &["pk", "v"]).await;
    for pk in 0u64..5 {
        h.insert("t", pk, &[pk, 42]).await;
    }
    let resp: Value = h
        .client
        .get(h.url("/tables/t/distinct?col=1&lo=42&hi=42"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let vals = resp["values"].as_array().unwrap();
    assert_eq!(
        vals.len(),
        1,
        "must return exactly 1 distinct value when all are equal"
    );
    assert_eq!(vals[0].as_u64().unwrap(), 42);
}

// ── value_counts endpoint ─────────────────────────────────────────────────────

#[tokio::test]
async fn value_counts_sum_equals_row_count() {
    let h = Harness::start("t", &["pk", "cat"]).await;
    for pk in 0u64..6 {
        h.insert("t", pk, &[pk, pk % 3]).await;
    }
    let resp: Value = h
        .client
        .get(h.url("/tables/t/value_counts?col=1&lo=0&hi=2"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let entries = resp["entries"].as_array().expect("entries");
    let total: u64 = entries
        .iter()
        .map(|e| e["count"].as_u64().unwrap_or(0))
        .sum();
    assert_eq!(total, 6, "sum of all counts must equal row count");
}

#[tokio::test]
async fn value_counts_entries_sorted_by_value() {
    let h = Harness::start("t", &["pk", "v"]).await;
    h.insert("t", 1, &[1, 30]).await;
    h.insert("t", 2, &[2, 10]).await;
    h.insert("t", 3, &[3, 20]).await;
    let resp: Value = h
        .client
        .get(h.url("/tables/t/value_counts?col=1&lo=10&hi=30"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let entries = resp["entries"].as_array().unwrap();
    assert_eq!(entries.len(), 3);
    let vals: Vec<u64> = entries
        .iter()
        .map(|e| e["value"].as_u64().unwrap())
        .collect();
    assert_eq!(
        vals,
        vec![10, 20, 30],
        "entries must be sorted by value ascending"
    );
}

#[tokio::test]
async fn value_counts_each_entry_has_value_and_count() {
    let h = Harness::start("t", &["pk", "cat"]).await;
    h.insert("t", 1, &[1, 5]).await;
    h.insert("t", 2, &[2, 5]).await;
    let resp: Value = h
        .client
        .get(h.url("/tables/t/value_counts?col=1&lo=5&hi=5"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let entries = resp["entries"].as_array().unwrap();
    assert!(!entries.is_empty());
    let entry = &entries[0];
    assert!(entry.get("value").is_some(), "entry must have value field");
    assert!(entry.get("count").is_some(), "entry must have count field");
    assert_eq!(
        entry["count"].as_u64().unwrap(),
        2,
        "count must be 2 for two rows with same value"
    );
}

#[tokio::test]
async fn value_counts_empty_range_returns_empty() {
    let h = Harness::start("t", &["pk", "v"]).await;
    for pk in 0u64..5 {
        h.insert("t", pk, &[pk, pk]).await;
    }
    let resp: Value = h
        .client
        .get(h.url("/tables/t/value_counts?col=1&lo=100&hi=200"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let entries = resp["entries"].as_array().unwrap();
    assert!(entries.is_empty(), "empty range must return no entries");
}

// ── DSL bridge ────────────────────────────────────────────────────────────────

#[tokio::test]
async fn dsl_distinct_returns_values() {
    let h = Harness::start("t", &["pk", "cat"]).await;
    h.insert("t", 1, &[1, 10]).await;
    h.insert("t", 2, &[2, 20]).await;
    h.insert("t", 3, &[3, 10]).await;
    let resp = h
        .client
        .post(h.url("/tables/t/query"))
        .header("content-type", "text/plain")
        .body("DISTINCT col=1 lo=10 hi=20")
        .send()
        .await
        .unwrap();
    assert!(resp.status().is_success(), "DSL DISTINCT must succeed");
    let body: Value = resp.json().await.unwrap();
    assert!(
        body.get("values").is_some(),
        "DSL DISTINCT must return 'values'; got: {body}"
    );
}

#[tokio::test]
async fn dsl_value_counts_returns_entries() {
    let h = Harness::start("t", &["pk", "cat"]).await;
    for pk in 0u64..4 {
        h.insert("t", pk, &[pk, pk % 2]).await;
    }
    let resp = h
        .client
        .post(h.url("/tables/t/query"))
        .header("content-type", "text/plain")
        .body("VALUE_COUNTS col=1 lo=0 hi=1")
        .send()
        .await
        .unwrap();
    assert!(resp.status().is_success(), "DSL VALUE_COUNTS must succeed");
    let body: Value = resp.json().await.unwrap();
    assert!(
        body.get("entries").is_some(),
        "DSL VALUE_COUNTS must return 'entries'; got: {body}"
    );
}

#[tokio::test]
async fn dsl_distinct_count_matches_http_endpoint() {
    let h = Harness::start("t", &["pk", "v"]).await;
    for pk in 0u64..6 {
        h.insert("t", pk, &[pk, pk % 3]).await;
    }
    let http_resp: Value = h
        .client
        .get(h.url("/tables/t/distinct?col=1&lo=0&hi=2"))
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
        .body("DISTINCT col=1 lo=0 hi=2")
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(
        http_resp["values"], dsl_resp["values"],
        "HTTP and DSL DISTINCT must return identical values arrays"
    );
}

#[tokio::test]
async fn dsl_value_counts_totals_match_http_endpoint() {
    let h = Harness::start("t", &["pk", "cat"]).await;
    for pk in 0u64..6 {
        h.insert("t", pk, &[pk, pk % 3]).await;
    }
    let http_resp: Value = h
        .client
        .get(h.url("/tables/t/value_counts?col=1&lo=0&hi=2"))
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
        .body("VALUE_COUNTS col=1 lo=0 hi=2")
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let http_total: u64 = http_resp["entries"]
        .as_array()
        .unwrap()
        .iter()
        .map(|e| e["count"].as_u64().unwrap_or(0))
        .sum();
    let dsl_total: u64 = dsl_resp["entries"]
        .as_array()
        .unwrap()
        .iter()
        .map(|e| e["count"].as_u64().unwrap_or(0))
        .sum();
    assert_eq!(
        http_total, dsl_total,
        "HTTP and DSL VALUE_COUNTS totals must match"
    );
}

#[tokio::test]
async fn distinct_count_equals_value_counts_entry_count() {
    let h = Harness::start("t", &["pk", "v"]).await;
    for pk in 0u64..9 {
        h.insert("t", pk, &[pk, pk % 4]).await;
    }
    let distinct_resp: Value = h
        .client
        .get(h.url("/tables/t/distinct?col=1&lo=0&hi=3"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let vc_resp: Value = h
        .client
        .get(h.url("/tables/t/value_counts?col=1&lo=0&hi=3"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(
        distinct_resp["values"].as_array().unwrap().len(),
        vc_resp["entries"].as_array().unwrap().len(),
        "number of distinct values must equal number of value_counts entries"
    );
}
