/// Layer 51 — HTTP group-by aggregate endpoints and DSL GROUP statements.
///
/// New routes:
///   GET /tables/{t}/group/count?filter_col=&lo=&hi=&group_col=
///   GET /tables/{t}/group/sum?filter_col=&lo=&hi=&group_col=&agg_col=
///   GET /tables/{t}/group/min?filter_col=&lo=&hi=&group_col=&agg_col=
///   GET /tables/{t}/group/max?filter_col=&lo=&hi=&group_col=&agg_col=
///
/// New DSL statements:
///   GROUP COUNT filter_col=<n> lo=<lo> hi=<hi> group_col=<g>
///   GROUP SUM   filter_col=<n> lo=<lo> hi=<hi> group_col=<g> agg_col=<a>
///   GROUP MIN   filter_col=<n> lo=<lo> hi=<hi> group_col=<g> agg_col=<a>
///   GROUP MAX   filter_col=<n> lo=<lo> hi=<hi> group_col=<g> agg_col=<a>
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

// ── group/count ───────────────────────────────────────────────────────────────

#[tokio::test]
async fn group_count_returns_entries() {
    // cols: pk, category, value
    // insert 4 rows: 2 in cat=1, 2 in cat=2
    let h = Harness::start("t", &["pk", "cat", "val"]).await;
    h.insert("t", 1, &[1, 1, 10]).await;
    h.insert("t", 2, &[2, 1, 20]).await;
    h.insert("t", 3, &[3, 2, 30]).await;
    h.insert("t", 4, &[4, 2, 40]).await;
    let resp: Value = h
        .client
        .get(h.url("/tables/t/group/count?filter_col=0&lo=1&hi=4&group_col=1"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let entries = resp["entries"].as_array().expect("entries array");
    assert!(!entries.is_empty(), "group count must return entries");
    let total: u64 = entries
        .iter()
        .map(|e| e["agg_val"].as_u64().unwrap_or(0))
        .sum();
    assert_eq!(total, 4, "total count across groups must equal row count");
}

#[tokio::test]
async fn group_count_two_groups_correct_counts() {
    let h = Harness::start("t", &["pk", "cat"]).await;
    for pk in 0u64..3 {
        h.insert("t", pk, &[pk, 10]).await;
    }
    for pk in 3u64..5 {
        h.insert("t", pk, &[pk, 20]).await;
    }
    let resp: Value = h
        .client
        .get(h.url("/tables/t/group/count?filter_col=0&lo=0&hi=4&group_col=1"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let entries = resp["entries"].as_array().unwrap();
    assert_eq!(entries.len(), 2, "must have exactly 2 distinct groups");
}

// ── group/sum ─────────────────────────────────────────────────────────────────

#[tokio::test]
async fn group_sum_totals_correct() {
    // cols: pk, cat, val
    let h = Harness::start("t", &["pk", "cat", "val"]).await;
    h.insert("t", 1, &[1, 1, 5]).await;
    h.insert("t", 2, &[2, 1, 15]).await; // cat=1 sum=20
    h.insert("t", 3, &[3, 2, 7]).await;
    h.insert("t", 4, &[4, 2, 13]).await; // cat=2 sum=20
    let resp: Value = h
        .client
        .get(h.url("/tables/t/group/sum?filter_col=0&lo=1&hi=4&group_col=1&agg_col=2"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let entries = resp["entries"].as_array().expect("entries");
    let total: u64 = entries
        .iter()
        .map(|e| e["agg_val"].as_u64().unwrap_or(0))
        .sum();
    assert_eq!(total, 40, "sum across all groups must be 40");
}

#[tokio::test]
async fn group_sum_empty_range_returns_empty() {
    let h = Harness::start("t", &["pk", "cat", "val"]).await;
    for pk in 0u64..5 {
        h.insert("t", pk, &[pk, pk % 2, pk * 10]).await;
    }
    let resp: Value = h
        .client
        .get(h.url("/tables/t/group/sum?filter_col=0&lo=100&hi=200&group_col=1&agg_col=2"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let entries = resp["entries"].as_array().unwrap();
    assert!(
        entries.is_empty(),
        "empty filter range must return no entries"
    );
}

// ── group/min ─────────────────────────────────────────────────────────────────

#[tokio::test]
async fn group_min_returns_minimum_per_group() {
    let h = Harness::start("t", &["pk", "cat", "val"]).await;
    h.insert("t", 1, &[1, 1, 3]).await;
    h.insert("t", 2, &[2, 1, 7]).await; // min for cat=1 is 3
    h.insert("t", 3, &[3, 2, 8]).await;
    h.insert("t", 4, &[4, 2, 2]).await; // min for cat=2 is 2
    let resp: Value = h
        .client
        .get(h.url("/tables/t/group/min?filter_col=0&lo=1&hi=4&group_col=1&agg_col=2"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let entries = resp["entries"].as_array().expect("entries");
    // overall minimum across groups should be 2
    let overall_min = entries
        .iter()
        .map(|e| e["agg_val"].as_u64().unwrap_or(u64::MAX))
        .min()
        .unwrap();
    assert_eq!(overall_min, 2, "minimum agg_val must be 2");
}

// ── group/max ─────────────────────────────────────────────────────────────────

#[tokio::test]
async fn group_max_returns_maximum_per_group() {
    let h = Harness::start("t", &["pk", "cat", "val"]).await;
    h.insert("t", 1, &[1, 1, 4]).await;
    h.insert("t", 2, &[2, 1, 9]).await; // max for cat=1 is 9
    h.insert("t", 3, &[3, 2, 6]).await;
    h.insert("t", 4, &[4, 2, 1]).await; // max for cat=2 is 6
    let resp: Value = h
        .client
        .get(h.url("/tables/t/group/max?filter_col=0&lo=1&hi=4&group_col=1&agg_col=2"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let entries = resp["entries"].as_array().expect("entries");
    let overall_max = entries
        .iter()
        .map(|e| e["agg_val"].as_u64().unwrap_or(0))
        .max()
        .unwrap();
    assert_eq!(overall_max, 9, "maximum agg_val must be 9");
}

// ── entry shape ───────────────────────────────────────────────────────────────

#[tokio::test]
async fn group_count_entry_has_group_val_and_agg_val_fields() {
    let h = Harness::start("t", &["pk", "cat"]).await;
    h.insert("t", 1, &[1, 5]).await;
    let resp: Value = h
        .client
        .get(h.url("/tables/t/group/count?filter_col=0&lo=1&hi=1&group_col=1"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let entries = resp["entries"].as_array().unwrap();
    assert!(!entries.is_empty());
    let entry = &entries[0];
    assert!(
        entry.get("group_val").is_some(),
        "entry must have group_val"
    );
    assert!(entry.get("agg_val").is_some(), "entry must have agg_val");
}

// ── DSL bridge ────────────────────────────────────────────────────────────────

#[tokio::test]
async fn dsl_group_count_returns_entries() {
    let h = Harness::start("t", &["pk", "cat"]).await;
    for pk in 0u64..4 {
        h.insert("t", pk, &[pk, pk % 2]).await;
    }
    let resp = h
        .client
        .post(h.url("/tables/t/query"))
        .header("content-type", "text/plain")
        .body("GROUP COUNT filter_col=0 lo=0 hi=3 group_col=1")
        .send()
        .await
        .unwrap();
    assert!(resp.status().is_success(), "DSL GROUP COUNT must succeed");
    let body: Value = resp.json().await.unwrap();
    assert!(
        body.get("entries").is_some(),
        "DSL GROUP COUNT must return 'entries'; got: {body}"
    );
}

#[tokio::test]
async fn dsl_group_sum_returns_entries() {
    let h = Harness::start("t", &["pk", "cat", "val"]).await;
    for pk in 0u64..4 {
        h.insert("t", pk, &[pk, pk % 2, pk * 5]).await;
    }
    let resp = h
        .client
        .post(h.url("/tables/t/query"))
        .header("content-type", "text/plain")
        .body("GROUP SUM filter_col=0 lo=0 hi=3 group_col=1 agg_col=2")
        .send()
        .await
        .unwrap();
    assert!(resp.status().is_success(), "DSL GROUP SUM must succeed");
    let body: Value = resp.json().await.unwrap();
    assert!(
        body.get("entries").is_some(),
        "DSL GROUP SUM must return 'entries'; got: {body}"
    );
}

#[tokio::test]
async fn dsl_group_min_returns_entries() {
    let h = Harness::start("t", &["pk", "cat", "val"]).await;
    for pk in 1u64..=4 {
        h.insert("t", pk, &[pk, pk % 2, pk * 3]).await;
    }
    let resp = h
        .client
        .post(h.url("/tables/t/query"))
        .header("content-type", "text/plain")
        .body("GROUP MIN filter_col=0 lo=1 hi=4 group_col=1 agg_col=2")
        .send()
        .await
        .unwrap();
    assert!(resp.status().is_success(), "DSL GROUP MIN must succeed");
    let body: Value = resp.json().await.unwrap();
    assert!(
        body.get("entries").is_some(),
        "DSL GROUP MIN must return 'entries'; got: {body}"
    );
}

#[tokio::test]
async fn dsl_group_max_returns_entries() {
    let h = Harness::start("t", &["pk", "cat", "val"]).await;
    for pk in 1u64..=4 {
        h.insert("t", pk, &[pk, pk % 2, pk * 3]).await;
    }
    let resp = h
        .client
        .post(h.url("/tables/t/query"))
        .header("content-type", "text/plain")
        .body("GROUP MAX filter_col=0 lo=1 hi=4 group_col=1 agg_col=2")
        .send()
        .await
        .unwrap();
    assert!(resp.status().is_success(), "DSL GROUP MAX must succeed");
    let body: Value = resp.json().await.unwrap();
    assert!(
        body.get("entries").is_some(),
        "DSL GROUP MAX must return 'entries'; got: {body}"
    );
}

#[tokio::test]
async fn dsl_group_count_matches_http_endpoint() {
    let h = Harness::start("t", &["pk", "cat"]).await;
    for pk in 0u64..6 {
        h.insert("t", pk, &[pk, pk % 3]).await;
    }
    let http_resp: Value = h
        .client
        .get(h.url("/tables/t/group/count?filter_col=0&lo=0&hi=5&group_col=1"))
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
        .body("GROUP COUNT filter_col=0 lo=0 hi=5 group_col=1")
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    // Both must have non-empty entries
    assert!(
        !http_resp["entries"].as_array().unwrap().is_empty(),
        "HTTP group count must be non-empty"
    );
    assert!(
        !dsl_resp["entries"].as_array().unwrap().is_empty(),
        "DSL group count must be non-empty"
    );
    // Total counts must match between HTTP and DSL
    let http_total: u64 = http_resp["entries"]
        .as_array()
        .unwrap()
        .iter()
        .map(|e| e["agg_val"].as_u64().unwrap_or(0))
        .sum();
    let dsl_total: u64 = dsl_resp["entries"]
        .as_array()
        .unwrap()
        .iter()
        .map(|e| e["agg_val"].as_u64().unwrap_or(0))
        .sum();
    assert_eq!(
        http_total, dsl_total,
        "HTTP and DSL group count totals must match"
    );
}
