/// Layer 56 — HTTP multi-predicate compound scan endpoint, and DSL SCAN WHERE.
///
/// New route:
///   GET /tables/{t}/scan_where?col0=&lo0=&hi0=[&col1=&lo1=&hi1=][&col2=&lo2=&hi2=]
///
/// New DSL statement:
///   SCAN WHERE col=<n> lo=<lo> hi=<hi> [AND col=<n> lo=<lo> hi=<hi> ...]
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

// ── single-predicate scan_where ───────────────────────────────────────────────

#[tokio::test]
async fn scan_where_single_predicate_returns_matching_rows() {
    let h = Harness::start("t", &["pk", "cat"]).await;
    h.insert("t", 1, &[1, 10]).await;
    h.insert("t", 2, &[2, 20]).await;
    h.insert("t", 3, &[3, 10]).await;
    let resp: Value = h
        .client
        .get(h.url("/tables/t/scan_where?col0=1&lo0=10&hi0=10"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let rows = resp["rows"].as_array().expect("rows");
    assert_eq!(rows.len(), 2, "only rows with cat=10");
}

#[tokio::test]
async fn scan_where_single_predicate_no_match_returns_empty() {
    let h = Harness::start("t", &["pk", "v"]).await;
    for pk in 0u64..5 {
        h.insert("t", pk, &[pk, pk]).await;
    }
    let resp: Value = h
        .client
        .get(h.url("/tables/t/scan_where?col0=1&lo0=100&hi0=200"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let rows = resp["rows"].as_array().unwrap();
    assert!(rows.is_empty());
}

// ── two-predicate AND scan ────────────────────────────────────────────────────

#[tokio::test]
async fn scan_where_two_predicates_both_must_match() {
    let h = Harness::start("t", &["pk", "region", "score"]).await;
    // region=1, score=50 -> should match (region 1-1 AND score 40-60)
    h.insert("t", 1, &[1, 1, 50]).await;
    // region=1, score=90 -> region matches, score doesn't
    h.insert("t", 2, &[2, 1, 90]).await;
    // region=2, score=50 -> score matches, region doesn't
    h.insert("t", 3, &[3, 2, 50]).await;
    // region=2, score=90 -> neither matches
    h.insert("t", 4, &[4, 2, 90]).await;
    let resp: Value = h
        .client
        .get(h.url("/tables/t/scan_where?col0=1&lo0=1&hi0=1&col1=2&lo1=40&hi1=60"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let rows = resp["rows"].as_array().unwrap();
    assert_eq!(rows.len(), 1, "only pk=1 satisfies both predicates");
    assert_eq!(rows[0]["pk"].as_u64().unwrap(), 1);
}

#[tokio::test]
async fn scan_where_two_predicates_no_row_satisfies_both() {
    let h = Harness::start("t", &["pk", "a", "b"]).await;
    h.insert("t", 1, &[1, 5, 100]).await; // a in range, b not
    h.insert("t", 2, &[2, 20, 50]).await; // b in range, a not
    let resp: Value = h
        .client
        .get(h.url("/tables/t/scan_where?col0=1&lo0=1&hi0=10&col1=2&lo1=40&hi1=60"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let rows = resp["rows"].as_array().unwrap();
    assert!(rows.is_empty(), "no row satisfies both predicates");
}

#[tokio::test]
async fn scan_where_two_predicates_multiple_rows_match() {
    let h = Harness::start("t", &["pk", "type", "level"]).await;
    for pk in 0u64..6 {
        h.insert("t", pk, &[pk, pk % 3, pk % 4]).await;
    }
    // type in [0,0] AND level in [0,1]
    let resp: Value = h
        .client
        .get(h.url("/tables/t/scan_where?col0=1&lo0=0&hi0=0&col1=2&lo1=0&hi1=1"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let rows = resp["rows"].as_array().unwrap();
    // pk=0: type=0, level=0 -> match; pk=3: type=0, level=3 -> level out; pk=6 doesn't exist
    assert!(
        rows.iter().all(|r| {
            let vals = r["values"].as_array().unwrap();
            vals[1].as_u64().unwrap() == 0 && vals[2].as_u64().unwrap() <= 1
        }),
        "all rows must satisfy both predicates"
    );
}

// ── three-predicate AND scan ──────────────────────────────────────────────────

#[tokio::test]
async fn scan_where_three_predicates_narrows_result() {
    let h = Harness::start("t", &["pk", "a", "b", "c"]).await;
    h.insert("t", 1, &[1, 1, 1, 1]).await; // all match
    h.insert("t", 2, &[2, 1, 1, 9]).await; // c fails
    h.insert("t", 3, &[3, 1, 9, 1]).await; // b fails
    h.insert("t", 4, &[4, 9, 1, 1]).await; // a fails
    let resp: Value = h
        .client
        .get(h.url("/tables/t/scan_where?col0=1&lo0=1&hi0=1&col1=2&lo1=1&hi1=1&col2=3&lo2=1&hi2=1"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let rows = resp["rows"].as_array().unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0]["pk"].as_u64().unwrap(), 1);
}

// ── row shape ─────────────────────────────────────────────────────────────────

#[tokio::test]
async fn scan_where_rows_have_pk_and_values() {
    let h = Harness::start("t", &["pk", "x", "y"]).await;
    h.insert("t", 7, &[7, 3, 5]).await;
    let resp: Value = h
        .client
        .get(h.url("/tables/t/scan_where?col0=1&lo0=3&hi0=3"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let row = &resp["rows"][0];
    assert!(row.get("pk").is_some());
    assert!(row.get("values").is_some());
    assert_eq!(row["pk"].as_u64().unwrap(), 7);
}

// ── DSL bridge ────────────────────────────────────────────────────────────────

#[tokio::test]
async fn dsl_scan_where_single_predicate() {
    let h = Harness::start("t", &["pk", "cat"]).await;
    h.insert("t", 1, &[1, 5]).await;
    h.insert("t", 2, &[2, 5]).await;
    h.insert("t", 3, &[3, 9]).await;
    let resp = h
        .client
        .post(h.url("/tables/t/query"))
        .header("content-type", "text/plain")
        .body("SCAN WHERE col=1 lo=5 hi=5")
        .send()
        .await
        .unwrap();
    assert!(resp.status().is_success());
    let body: Value = resp.json().await.unwrap();
    let rows = body["rows"].as_array().expect("rows");
    assert_eq!(rows.len(), 2);
}

#[tokio::test]
async fn dsl_scan_where_two_predicates_with_and() {
    let h = Harness::start("t", &["pk", "region", "score"]).await;
    h.insert("t", 1, &[1, 1, 50]).await;
    h.insert("t", 2, &[2, 1, 90]).await;
    h.insert("t", 3, &[3, 2, 50]).await;
    let resp = h
        .client
        .post(h.url("/tables/t/query"))
        .header("content-type", "text/plain")
        .body("SCAN WHERE col=1 lo=1 hi=1 AND col=2 lo=40 hi=60")
        .send()
        .await
        .unwrap();
    assert!(resp.status().is_success());
    let body: Value = resp.json().await.unwrap();
    let rows = body["rows"].as_array().unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0]["pk"].as_u64().unwrap(), 1);
}

#[tokio::test]
async fn dsl_scan_where_count_matches_http() {
    let h = Harness::start("t", &["pk", "type", "val"]).await;
    for pk in 0u64..8 {
        h.insert("t", pk, &[pk, pk % 4, pk % 3]).await;
    }
    let http_resp: Value = h
        .client
        .get(h.url("/tables/t/scan_where?col0=1&lo0=0&hi0=1&col1=2&lo1=0&hi1=1"))
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
        .body("SCAN WHERE col=1 lo=0 hi=1 AND col=2 lo=0 hi=1")
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(
        http_resp["rows"].as_array().unwrap().len(),
        dsl_resp["rows"].as_array().unwrap().len(),
        "HTTP and DSL SCAN WHERE must return the same row count"
    );
}

#[tokio::test]
async fn scan_where_subset_of_plain_scan() {
    let h = Harness::start("t", &["pk", "a", "b"]).await;
    for pk in 0u64..10 {
        h.insert("t", pk, &[pk, pk % 5, pk % 3]).await;
    }
    let scan_resp: Value = h
        .client
        .get(h.url("/tables/t/scan?col=1&lo=0&hi=4"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let sw_resp: Value = h
        .client
        .get(h.url("/tables/t/scan_where?col0=1&lo0=0&hi0=4&col1=2&lo1=0&hi1=2"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    // scan_where with an extra predicate must return <= rows from plain scan
    assert!(
        sw_resp["rows"].as_array().unwrap().len() <= scan_resp["rows"].as_array().unwrap().len(),
        "scan_where result must be a subset of plain scan"
    );
}
