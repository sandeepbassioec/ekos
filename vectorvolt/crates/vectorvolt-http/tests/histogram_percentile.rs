/// Layer 50 — HTTP histogram + percentile endpoints, and DSL HISTOGRAM / PERCENTILE.
///
/// New routes:
///   GET /tables/{t}/histogram?col=&lo=&hi=&buckets=
///   GET /tables/{t}/percentile?col=&lo=&hi=&p=
///
/// New DSL statements:
///   HISTOGRAM col=<n> lo=<lo> hi=<hi> buckets=<b>
///   PERCENTILE col=<n> lo=<lo> hi=<hi> p=<0-100>
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

// ── histogram endpoint ────────────────────────────────────────────────────────

#[tokio::test]
async fn histogram_bucket_count_matches_n_buckets() {
    let h = Harness::start("t", &["pk", "val"]).await;
    for pk in 0u64..10 {
        h.insert("t", pk, &[pk, pk * 10]).await;
    }
    let resp: Value = h
        .client
        .get(h.url("/tables/t/histogram?col=1&lo=0&hi=90&buckets=3"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let buckets = resp["buckets"].as_array().expect("buckets array");
    assert_eq!(buckets.len(), 3, "must return exactly 3 buckets");
}

#[tokio::test]
async fn histogram_total_count_equals_row_count() {
    let h = Harness::start("t", &["pk", "val"]).await;
    for pk in 0u64..10 {
        h.insert("t", pk, &[pk, pk]).await;
    }
    let resp: Value = h
        .client
        .get(h.url("/tables/t/histogram?col=1&lo=0&hi=9&buckets=5"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let total: u64 = resp["buckets"]
        .as_array()
        .unwrap()
        .iter()
        .map(|b| b["count"].as_u64().unwrap_or(0))
        .sum();
    assert_eq!(
        total, 10,
        "total count across all buckets must equal row count"
    );
}

#[tokio::test]
async fn histogram_empty_range_returns_zero_count_buckets() {
    let h = Harness::start("t", &["pk", "val"]).await;
    for pk in 0u64..5 {
        h.insert("t", pk, &[pk, pk]).await;
    }
    let resp: Value = h
        .client
        .get(h.url("/tables/t/histogram?col=1&lo=100&hi=200&buckets=4"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let total: u64 = resp["buckets"]
        .as_array()
        .unwrap()
        .iter()
        .map(|b| b["count"].as_u64().unwrap_or(0))
        .sum();
    assert_eq!(total, 0, "empty range must produce all-zero buckets");
}

#[tokio::test]
async fn histogram_single_bucket_contains_all_rows() {
    let h = Harness::start("t", &["pk", "val"]).await;
    for pk in 1u64..=5 {
        h.insert("t", pk, &[pk, pk * 2]).await;
    }
    let resp: Value = h
        .client
        .get(h.url("/tables/t/histogram?col=1&lo=2&hi=10&buckets=1"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let buckets = resp["buckets"].as_array().unwrap();
    assert_eq!(buckets.len(), 1);
    assert_eq!(buckets[0]["count"].as_u64().unwrap(), 5);
}

#[tokio::test]
async fn histogram_bucket_lo_hi_are_monotone() {
    let h = Harness::start("t", &["pk", "val"]).await;
    for pk in 0u64..20 {
        h.insert("t", pk, &[pk, pk]).await;
    }
    let resp: Value = h
        .client
        .get(h.url("/tables/t/histogram?col=1&lo=0&hi=19&buckets=4"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let buckets = resp["buckets"].as_array().unwrap();
    for i in 1..buckets.len() {
        assert!(
            buckets[i]["lo"].as_u64().unwrap() > buckets[i - 1]["hi"].as_u64().unwrap()
                || buckets[i]["lo"].as_u64().unwrap() == buckets[i - 1]["hi"].as_u64().unwrap(),
            "bucket lo/hi must be non-overlapping and monotone"
        );
    }
}

#[tokio::test]
async fn histogram_after_flush_same_result() {
    let h = Harness::start("t", &["pk", "val"]).await;
    for pk in 0u64..10 {
        h.insert("t", pk, &[pk, pk * 10]).await;
    }
    // flush via DSL
    h.client
        .post(h.url("/tables/t/query"))
        .header("content-type", "text/plain")
        .body("STATS")
        .send()
        .await
        .unwrap();
    let resp: Value = h
        .client
        .get(h.url("/tables/t/histogram?col=1&lo=0&hi=90&buckets=3"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let total: u64 = resp["buckets"]
        .as_array()
        .unwrap()
        .iter()
        .map(|b| b["count"].as_u64().unwrap_or(0))
        .sum();
    assert_eq!(total, 10);
}

// ── percentile endpoint ───────────────────────────────────────────────────────

#[tokio::test]
async fn percentile_p50_is_median() {
    let h = Harness::start("t", &["pk", "val"]).await;
    // values 1..=9 — median is 5
    for pk in 1u64..=9 {
        h.insert("t", pk, &[pk, pk]).await;
    }
    let resp: Value = h
        .client
        .get(h.url("/tables/t/percentile?col=1&lo=1&hi=9&p=50"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let v = resp["value"].as_u64().expect("value must be present");
    assert_eq!(v, 5, "P50 of 1..=9 must be 5");
}

#[tokio::test]
async fn percentile_p100_is_max() {
    let h = Harness::start("t", &["pk", "val"]).await;
    for pk in 1u64..=10 {
        h.insert("t", pk, &[pk, pk]).await;
    }
    let resp: Value = h
        .client
        .get(h.url("/tables/t/percentile?col=1&lo=1&hi=10&p=100"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let v = resp["value"].as_u64().expect("value must be present");
    assert_eq!(v, 10, "P100 must equal the maximum");
}

#[tokio::test]
async fn percentile_p0_is_min() {
    let h = Harness::start("t", &["pk", "val"]).await;
    for pk in 1u64..=10 {
        h.insert("t", pk, &[pk, pk]).await;
    }
    let resp: Value = h
        .client
        .get(h.url("/tables/t/percentile?col=1&lo=1&hi=10&p=0"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let v = resp["value"].as_u64().expect("value must be present");
    assert_eq!(v, 1, "P0 must equal the minimum");
}

#[tokio::test]
async fn percentile_empty_range_returns_null() {
    let h = Harness::start("t", &["pk", "val"]).await;
    for pk in 1u64..=5 {
        h.insert("t", pk, &[pk, pk]).await;
    }
    let resp: Value = h
        .client
        .get(h.url("/tables/t/percentile?col=1&lo=100&hi=200&p=50"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert!(
        resp["value"].is_null(),
        "empty range must return null value; got: {resp}"
    );
}

#[tokio::test]
async fn percentile_p95_in_range() {
    let h = Harness::start("t", &["pk", "val"]).await;
    for pk in 1u64..=100 {
        h.insert("t", pk, &[pk, pk]).await;
    }
    let resp: Value = h
        .client
        .get(h.url("/tables/t/percentile?col=1&lo=1&hi=100&p=95"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let v = resp["value"].as_u64().expect("value must be present");
    assert!(
        (90..=100).contains(&v),
        "P95 of 1..=100 must be near 95; got {v}"
    );
}

// ── DSL bridge: HISTOGRAM + PERCENTILE via POST /query ────────────────────────

#[tokio::test]
async fn dsl_histogram_returns_buckets() {
    let h = Harness::start("t", &["pk", "val"]).await;
    for pk in 0u64..6 {
        h.insert("t", pk, &[pk, pk * 10]).await;
    }
    let resp = h
        .client
        .post(h.url("/tables/t/query"))
        .header("content-type", "text/plain")
        .body("HISTOGRAM col=1 lo=0 hi=50 buckets=2")
        .send()
        .await
        .unwrap();
    assert!(
        resp.status().is_success(),
        "DSL HISTOGRAM must succeed; status={}",
        resp.status()
    );
    let body: Value = resp.json().await.unwrap();
    assert!(
        body.get("buckets").is_some(),
        "DSL HISTOGRAM response must have 'buckets'; got: {body}"
    );
}

#[tokio::test]
async fn dsl_percentile_returns_value() {
    let h = Harness::start("t", &["pk", "val"]).await;
    for pk in 1u64..=10 {
        h.insert("t", pk, &[pk, pk]).await;
    }
    let resp = h
        .client
        .post(h.url("/tables/t/query"))
        .header("content-type", "text/plain")
        .body("PERCENTILE col=1 lo=1 hi=10 p=50")
        .send()
        .await
        .unwrap();
    assert!(
        resp.status().is_success(),
        "DSL PERCENTILE must succeed; status={}",
        resp.status()
    );
    let body: Value = resp.json().await.unwrap();
    assert!(
        body.get("value").is_some(),
        "DSL PERCENTILE response must have 'value'; got: {body}"
    );
}

#[tokio::test]
async fn dsl_percentile_p50_matches_http_endpoint() {
    let h = Harness::start("t", &["pk", "val"]).await;
    for pk in 1u64..=9 {
        h.insert("t", pk, &[pk, pk]).await;
    }
    // HTTP endpoint
    let http_resp: Value = h
        .client
        .get(h.url("/tables/t/percentile?col=1&lo=1&hi=9&p=50"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    // DSL endpoint
    let dsl_resp: Value = h
        .client
        .post(h.url("/tables/t/query"))
        .header("content-type", "text/plain")
        .body("PERCENTILE col=1 lo=1 hi=9 p=50")
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(
        http_resp["value"], dsl_resp["value"],
        "HTTP and DSL PERCENTILE must return the same value"
    );
}
