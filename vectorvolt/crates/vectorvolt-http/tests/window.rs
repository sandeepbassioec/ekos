/// Layer 52 — HTTP window function endpoints and DSL WINDOW statements.
///
/// New routes:
///   GET /tables/{t}/window/running_sum?filter_col=&lo=&hi=&agg_col=
///   GET /tables/{t}/window/running_count?filter_col=&lo=&hi=
///   GET /tables/{t}/window/rank?filter_col=&lo=&hi=&sort_col=&asc=
///   GET /tables/{t}/window/dense_rank?filter_col=&lo=&hi=&sort_col=&asc=
///
/// New DSL statements:
///   WINDOW RUNNING_SUM   filter_col=<n> lo=<lo> hi=<hi> agg_col=<a>
///   WINDOW RUNNING_COUNT filter_col=<n> lo=<lo> hi=<hi>
///   WINDOW RANK          filter_col=<n> lo=<lo> hi=<hi> sort_col=<s> asc=<0|1>
///   WINDOW DENSE_RANK    filter_col=<n> lo=<lo> hi=<hi> sort_col=<s> asc=<0|1>
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

// ── running_sum ───────────────────────────────────────────────────────────────

#[tokio::test]
async fn running_sum_last_entry_equals_total_sum() {
    // cols: pk, filter, val
    let h = Harness::start("t", &["pk", "f", "val"]).await;
    for pk in 1u64..=5 {
        h.insert("t", pk, &[pk, pk, pk * 10]).await;
    }
    let resp: Value = h
        .client
        .get(h.url("/tables/t/window/running_sum?filter_col=0&lo=1&hi=5&agg_col=2"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let rows = resp["rows"].as_array().expect("rows array");
    assert_eq!(rows.len(), 5, "must return 5 window rows");
    let last = rows.last().unwrap()["window_val"].as_u64().unwrap();
    // sum of 10+20+30+40+50 = 150
    assert_eq!(last, 150, "last running_sum must equal total sum");
}

#[tokio::test]
async fn running_sum_is_monotonically_non_decreasing() {
    let h = Harness::start("t", &["pk", "val"]).await;
    for pk in 0u64..6 {
        h.insert("t", pk, &[pk, pk * 5]).await;
    }
    let resp: Value = h
        .client
        .get(h.url("/tables/t/window/running_sum?filter_col=0&lo=0&hi=5&agg_col=1"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let rows = resp["rows"].as_array().unwrap();
    for i in 1..rows.len() {
        let prev = rows[i - 1]["window_val"].as_u64().unwrap();
        let curr = rows[i]["window_val"].as_u64().unwrap();
        assert!(curr >= prev, "running_sum must be non-decreasing");
    }
}

#[tokio::test]
async fn running_sum_empty_range_returns_empty_rows() {
    let h = Harness::start("t", &["pk", "val"]).await;
    for pk in 0u64..5 {
        h.insert("t", pk, &[pk, pk]).await;
    }
    let resp: Value = h
        .client
        .get(h.url("/tables/t/window/running_sum?filter_col=0&lo=100&hi=200&agg_col=1"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let rows = resp["rows"].as_array().unwrap();
    assert!(rows.is_empty(), "empty range must return no rows");
}

// ── running_count ─────────────────────────────────────────────────────────────

#[tokio::test]
async fn running_count_values_are_sequential_from_one() {
    let h = Harness::start("t", &["pk"]).await;
    for pk in 1u64..=4 {
        h.insert("t", pk, &[pk]).await;
    }
    let resp: Value = h
        .client
        .get(h.url("/tables/t/window/running_count?filter_col=0&lo=1&hi=4"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let rows = resp["rows"].as_array().expect("rows");
    assert_eq!(rows.len(), 4);
    for (i, row) in rows.iter().enumerate() {
        assert_eq!(
            row["window_val"].as_u64().unwrap(),
            i as u64 + 1,
            "running_count must be 1-based sequential"
        );
    }
}

#[tokio::test]
async fn running_count_row_count_matches_inserted() {
    let h = Harness::start("t", &["pk"]).await;
    for pk in 0u64..7 {
        h.insert("t", pk, &[pk]).await;
    }
    let resp: Value = h
        .client
        .get(h.url("/tables/t/window/running_count?filter_col=0&lo=0&hi=6"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let rows = resp["rows"].as_array().unwrap();
    assert_eq!(rows.len(), 7);
    assert_eq!(rows.last().unwrap()["window_val"].as_u64().unwrap(), 7);
}

// ── rank ──────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn rank_tied_rows_get_same_rank() {
    // cols: pk, score — two rows with same score should share rank
    let h = Harness::start("t", &["pk", "score"]).await;
    h.insert("t", 1, &[1, 10]).await;
    h.insert("t", 2, &[2, 10]).await; // tie at score=10
    h.insert("t", 3, &[3, 20]).await;
    let resp: Value = h
        .client
        .get(h.url("/tables/t/window/rank?filter_col=0&lo=1&hi=3&sort_col=1&asc=1"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let rows = resp["rows"].as_array().expect("rows");
    assert_eq!(rows.len(), 3);
    let rank_vals: Vec<u64> = rows
        .iter()
        .map(|r| r["window_val"].as_u64().unwrap())
        .collect();
    // first two should share rank 1, last should be rank 3
    assert_eq!(rank_vals[0], rank_vals[1], "tied rows must share rank");
    assert_eq!(rank_vals[2], 3, "third row must have rank 3 (SQL rank gap)");
}

#[tokio::test]
async fn rank_ascending_first_row_has_rank_one() {
    let h = Harness::start("t", &["pk", "score"]).await;
    for pk in 1u64..=5 {
        h.insert("t", pk, &[pk, pk * 10]).await;
    }
    let resp: Value = h
        .client
        .get(h.url("/tables/t/window/rank?filter_col=0&lo=1&hi=5&sort_col=1&asc=1"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let rows = resp["rows"].as_array().unwrap();
    assert_eq!(
        rows[0]["window_val"].as_u64().unwrap(),
        1,
        "first row ascending must have rank 1"
    );
}

// ── dense_rank ────────────────────────────────────────────────────────────────

#[tokio::test]
async fn dense_rank_tied_rows_next_rank_has_no_gap() {
    let h = Harness::start("t", &["pk", "score"]).await;
    h.insert("t", 1, &[1, 10]).await;
    h.insert("t", 2, &[2, 10]).await; // tie
    h.insert("t", 3, &[3, 20]).await;
    let resp: Value = h
        .client
        .get(h.url("/tables/t/window/dense_rank?filter_col=0&lo=1&hi=3&sort_col=1&asc=1"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let rows = resp["rows"].as_array().expect("rows");
    let rank_vals: Vec<u64> = rows
        .iter()
        .map(|r| r["window_val"].as_u64().unwrap())
        .collect();
    assert_eq!(
        rank_vals[0], rank_vals[1],
        "tied rows must share dense rank"
    );
    assert_eq!(
        rank_vals[2], 2,
        "next distinct value must have dense rank 2 (no gap)"
    );
}

#[tokio::test]
async fn dense_rank_all_distinct_equals_rank() {
    let h = Harness::start("t", &["pk", "score"]).await;
    for pk in 1u64..=4 {
        h.insert("t", pk, &[pk, pk * 3]).await;
    }
    let rank_resp: Value = h
        .client
        .get(h.url("/tables/t/window/rank?filter_col=0&lo=1&hi=4&sort_col=1&asc=1"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let dense_resp: Value = h
        .client
        .get(h.url("/tables/t/window/dense_rank?filter_col=0&lo=1&hi=4&sort_col=1&asc=1"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let rank_vals: Vec<u64> = rank_resp["rows"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| r["window_val"].as_u64().unwrap())
        .collect();
    let dense_vals: Vec<u64> = dense_resp["rows"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| r["window_val"].as_u64().unwrap())
        .collect();
    assert_eq!(
        rank_vals, dense_vals,
        "rank and dense_rank must be identical when all values are distinct"
    );
}

// ── entry shape ───────────────────────────────────────────────────────────────

#[tokio::test]
async fn window_row_has_pk_and_window_val_fields() {
    let h = Harness::start("t", &["pk", "val"]).await;
    h.insert("t", 42, &[42, 99]).await;
    let resp: Value = h
        .client
        .get(h.url("/tables/t/window/running_count?filter_col=0&lo=42&hi=42"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let rows = resp["rows"].as_array().unwrap();
    assert!(!rows.is_empty());
    let row = &rows[0];
    assert!(row.get("pk").is_some(), "row must have pk field");
    assert!(
        row.get("window_val").is_some(),
        "row must have window_val field"
    );
}

// ── DSL bridge ────────────────────────────────────────────────────────────────

#[tokio::test]
async fn dsl_window_running_sum_returns_rows() {
    let h = Harness::start("t", &["pk", "val"]).await;
    for pk in 1u64..=3 {
        h.insert("t", pk, &[pk, pk * 10]).await;
    }
    let resp = h
        .client
        .post(h.url("/tables/t/query"))
        .header("content-type", "text/plain")
        .body("WINDOW RUNNING_SUM filter_col=0 lo=1 hi=3 agg_col=1")
        .send()
        .await
        .unwrap();
    assert!(
        resp.status().is_success(),
        "DSL WINDOW RUNNING_SUM must succeed"
    );
    let body: Value = resp.json().await.unwrap();
    assert!(
        body.get("rows").is_some(),
        "DSL WINDOW RUNNING_SUM must return 'rows'; got: {body}"
    );
}

#[tokio::test]
async fn dsl_window_running_count_returns_rows() {
    let h = Harness::start("t", &["pk"]).await;
    for pk in 0u64..4 {
        h.insert("t", pk, &[pk]).await;
    }
    let resp = h
        .client
        .post(h.url("/tables/t/query"))
        .header("content-type", "text/plain")
        .body("WINDOW RUNNING_COUNT filter_col=0 lo=0 hi=3")
        .send()
        .await
        .unwrap();
    assert!(
        resp.status().is_success(),
        "DSL WINDOW RUNNING_COUNT must succeed"
    );
    let body: Value = resp.json().await.unwrap();
    assert!(
        body.get("rows").is_some(),
        "DSL WINDOW RUNNING_COUNT must return 'rows'; got: {body}"
    );
}

#[tokio::test]
async fn dsl_window_rank_returns_rows() {
    let h = Harness::start("t", &["pk", "score"]).await;
    for pk in 1u64..=3 {
        h.insert("t", pk, &[pk, pk * 5]).await;
    }
    let resp = h
        .client
        .post(h.url("/tables/t/query"))
        .header("content-type", "text/plain")
        .body("WINDOW RANK filter_col=0 lo=1 hi=3 sort_col=1 asc=1")
        .send()
        .await
        .unwrap();
    assert!(resp.status().is_success(), "DSL WINDOW RANK must succeed");
    let body: Value = resp.json().await.unwrap();
    assert!(
        body.get("rows").is_some(),
        "DSL WINDOW RANK must return 'rows'; got: {body}"
    );
}

#[tokio::test]
async fn dsl_window_dense_rank_returns_rows() {
    let h = Harness::start("t", &["pk", "score"]).await;
    for pk in 1u64..=3 {
        h.insert("t", pk, &[pk, pk * 5]).await;
    }
    let resp = h
        .client
        .post(h.url("/tables/t/query"))
        .header("content-type", "text/plain")
        .body("WINDOW DENSE_RANK filter_col=0 lo=1 hi=3 sort_col=1 asc=0")
        .send()
        .await
        .unwrap();
    assert!(
        resp.status().is_success(),
        "DSL WINDOW DENSE_RANK must succeed"
    );
    let body: Value = resp.json().await.unwrap();
    assert!(
        body.get("rows").is_some(),
        "DSL WINDOW DENSE_RANK must return 'rows'; got: {body}"
    );
}

#[tokio::test]
async fn dsl_running_sum_matches_http_endpoint() {
    let h = Harness::start("t", &["pk", "val"]).await;
    for pk in 1u64..=5 {
        h.insert("t", pk, &[pk, pk]).await;
    }
    let http_resp: Value = h
        .client
        .get(h.url("/tables/t/window/running_sum?filter_col=0&lo=1&hi=5&agg_col=1"))
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
        .body("WINDOW RUNNING_SUM filter_col=0 lo=1 hi=5 agg_col=1")
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let http_last = http_resp["rows"].as_array().unwrap().last().unwrap()["window_val"]
        .as_u64()
        .unwrap();
    let dsl_last = dsl_resp["rows"].as_array().unwrap().last().unwrap()["window_val"]
        .as_u64()
        .unwrap();
    assert_eq!(
        http_last, dsl_last,
        "HTTP and DSL running_sum must agree on the final value"
    );
}
