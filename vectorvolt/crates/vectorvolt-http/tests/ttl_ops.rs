/// Layer 60 — TTL operations via HTTP + DSL.
///
/// New routes:
///   POST /tables/{t}/rows/{pk}/ttl     body: {"values":[...],"expire_at":N}  → {"ok":true}
///   GET  /tables/{t}/get_with_ttl/{pk}?now=N                                 → row or 404
///   GET  /tables/{t}/scan_with_ttl?now=N                                     → {"rows":[...]}
///   POST /tables/{t}/purge_expired?now=N                                     → {"purged":N}
///
/// New DSL statements:
///   INSERT TTL <pk> <v1> [v2 ...] EXPIRE <n>
///   GET TTL <pk> NOW <n>
///   SCAN TTL NOW <n>
///   PURGE EXPIRED NOW <n>
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
}

// ── insert_with_ttl HTTP ──────────────────────────────────────────────────────

#[tokio::test]
async fn insert_with_ttl_returns_ok() {
    let h = Harness::start("t", &["pk", "v"]).await;
    let resp: Value = h
        .client
        .post(h.url("/tables/t/rows/1/ttl"))
        .json(&serde_json::json!({ "values": [1, 42], "expire_at": 1000 }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(resp["ok"], true);
}

// ── get_with_ttl HTTP ─────────────────────────────────────────────────────────

#[tokio::test]
async fn get_with_ttl_returns_row_when_not_expired() {
    let h = Harness::start("t", &["pk", "v"]).await;
    h.client
        .post(h.url("/tables/t/rows/2/ttl"))
        .json(&serde_json::json!({ "values": [2, 77], "expire_at": 9999 }))
        .send()
        .await
        .unwrap();
    let resp = h
        .client
        .get(h.url("/tables/t/get_with_ttl/2?now=1000"))
        .send()
        .await
        .unwrap();
    assert!(resp.status().is_success());
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["pk"].as_u64().unwrap(), 2);
    assert_eq!(body["values"][1].as_u64().unwrap(), 77);
}

#[tokio::test]
async fn get_with_ttl_returns_404_when_expired() {
    let h = Harness::start("t", &["pk", "v"]).await;
    h.client
        .post(h.url("/tables/t/rows/3/ttl"))
        .json(&serde_json::json!({ "values": [3, 55], "expire_at": 100 }))
        .send()
        .await
        .unwrap();
    let resp = h
        .client
        .get(h.url("/tables/t/get_with_ttl/3?now=200"))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status().as_u16(), 404);
}

// ── scan_with_ttl HTTP ────────────────────────────────────────────────────────

#[tokio::test]
async fn scan_with_ttl_excludes_expired_rows() {
    let h = Harness::start("t", &["pk", "v"]).await;
    // expire_at=500 → expired at now=1000
    h.client
        .post(h.url("/tables/t/rows/10/ttl"))
        .json(&serde_json::json!({ "values": [10, 1], "expire_at": 500 }))
        .send()
        .await
        .unwrap();
    // expire_at=2000 → live at now=1000
    h.client
        .post(h.url("/tables/t/rows/11/ttl"))
        .json(&serde_json::json!({ "values": [11, 2], "expire_at": 2000 }))
        .send()
        .await
        .unwrap();
    let resp: Value = h
        .client
        .get(h.url("/tables/t/scan_with_ttl?now=1000"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let rows = resp["rows"].as_array().unwrap();
    assert_eq!(rows.len(), 1, "only non-expired row returned");
    assert_eq!(rows[0]["pk"].as_u64().unwrap(), 11);
}

#[tokio::test]
async fn scan_with_ttl_returns_all_when_none_expired() {
    let h = Harness::start("t", &["pk", "v"]).await;
    for pk in 1u64..=4 {
        h.client
            .post(h.url(&format!("/tables/t/rows/{pk}/ttl")))
            .json(&serde_json::json!({ "values": [pk, pk * 10], "expire_at": 9999 }))
            .send()
            .await
            .unwrap();
    }
    let resp: Value = h
        .client
        .get(h.url("/tables/t/scan_with_ttl?now=1000"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(resp["rows"].as_array().unwrap().len(), 4);
}

// ── purge_expired HTTP ────────────────────────────────────────────────────────

#[tokio::test]
async fn purge_expired_removes_expired_rows() {
    let h = Harness::start("t", &["pk", "v"]).await;
    for pk in 0u64..3 {
        h.client
            .post(h.url(&format!("/tables/t/rows/{pk}/ttl")))
            .json(&serde_json::json!({ "values": [pk, pk], "expire_at": 100 }))
            .send()
            .await
            .unwrap();
    }
    // live row
    h.client
        .post(h.url("/tables/t/rows/99/ttl"))
        .json(&serde_json::json!({ "values": [99, 99], "expire_at": 9999 }))
        .send()
        .await
        .unwrap();
    let purge: Value = h
        .client
        .post(h.url("/tables/t/purge_expired?now=200"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(purge["purged"].as_u64().unwrap(), 3);
    // live row still accessible
    let live = h
        .client
        .get(h.url("/tables/t/get_with_ttl/99?now=200"))
        .send()
        .await
        .unwrap();
    assert!(live.status().is_success());
}

#[tokio::test]
async fn purge_expired_returns_zero_when_nothing_expired() {
    let h = Harness::start("t", &["pk", "v"]).await;
    h.client
        .post(h.url("/tables/t/rows/1/ttl"))
        .json(&serde_json::json!({ "values": [1, 1], "expire_at": 9999 }))
        .send()
        .await
        .unwrap();
    let resp: Value = h
        .client
        .post(h.url("/tables/t/purge_expired?now=100"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(resp["purged"].as_u64().unwrap(), 0);
}

// ── DSL: INSERT TTL ───────────────────────────────────────────────────────────

#[tokio::test]
async fn dsl_insert_ttl_returns_ok() {
    let h = Harness::start("t", &["pk", "v"]).await;
    let resp = h
        .client
        .post(h.url("/tables/t/query"))
        .header("content-type", "text/plain")
        .body("INSERT TTL 5 5 50 EXPIRE 8888")
        .send()
        .await
        .unwrap();
    assert!(resp.status().is_success());
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["type"], "ok");
}

#[tokio::test]
async fn dsl_insert_ttl_then_http_get_with_ttl() {
    let h = Harness::start("t", &["pk", "v"]).await;
    h.client
        .post(h.url("/tables/t/query"))
        .header("content-type", "text/plain")
        .body("INSERT TTL 20 20 200 EXPIRE 5000")
        .send()
        .await
        .unwrap();
    let row = h
        .client
        .get(h.url("/tables/t/get_with_ttl/20?now=1000"))
        .send()
        .await
        .unwrap();
    assert!(row.status().is_success());
    let body: Value = row.json().await.unwrap();
    assert_eq!(body["values"][1].as_u64().unwrap(), 200);
}

// ── DSL: SCAN TTL ─────────────────────────────────────────────────────────────

#[tokio::test]
async fn dsl_scan_ttl_returns_live_rows_only() {
    let h = Harness::start("t", &["pk", "v"]).await;
    h.client
        .post(h.url("/tables/t/rows/40/ttl"))
        .json(&serde_json::json!({ "values": [40, 1], "expire_at": 500 }))
        .send()
        .await
        .unwrap();
    h.client
        .post(h.url("/tables/t/rows/41/ttl"))
        .json(&serde_json::json!({ "values": [41, 2], "expire_at": 9999 }))
        .send()
        .await
        .unwrap();
    let resp: Value = h
        .client
        .post(h.url("/tables/t/query"))
        .header("content-type", "text/plain")
        .body("SCAN TTL NOW 1000")
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let rows = resp["rows"].as_array().unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0]["pk"].as_u64().unwrap(), 41);
}

// ── DSL: GET TTL ──────────────────────────────────────────────────────────────

#[tokio::test]
async fn dsl_get_ttl_returns_row_when_live() {
    let h = Harness::start("t", &["pk", "v"]).await;
    h.client
        .post(h.url("/tables/t/rows/60/ttl"))
        .json(&serde_json::json!({ "values": [60, 600], "expire_at": 9999 }))
        .send()
        .await
        .unwrap();
    let resp: Value = h
        .client
        .post(h.url("/tables/t/query"))
        .header("content-type", "text/plain")
        .body("GET TTL 60 NOW 1000")
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(resp["pk"].as_u64().unwrap(), 60);
    assert_eq!(resp["values"][1].as_u64().unwrap(), 600);
}

#[tokio::test]
async fn dsl_get_ttl_returns_not_found_when_expired() {
    let h = Harness::start("t", &["pk", "v"]).await;
    h.client
        .post(h.url("/tables/t/rows/61/ttl"))
        .json(&serde_json::json!({ "values": [61, 1], "expire_at": 100 }))
        .send()
        .await
        .unwrap();
    let resp: Value = h
        .client
        .post(h.url("/tables/t/query"))
        .header("content-type", "text/plain")
        .body("GET TTL 61 NOW 5000")
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(resp["type"], "not_found");
}

// ── DSL: PURGE EXPIRED ────────────────────────────────────────────────────────

#[tokio::test]
async fn dsl_purge_expired_returns_count() {
    let h = Harness::start("t", &["pk", "v"]).await;
    for pk in 0u64..3 {
        h.client
            .post(h.url(&format!("/tables/t/rows/{pk}/ttl")))
            .json(&serde_json::json!({ "values": [pk, pk], "expire_at": 50 }))
            .send()
            .await
            .unwrap();
    }
    let resp: Value = h
        .client
        .post(h.url("/tables/t/query"))
        .header("content-type", "text/plain")
        .body("PURGE EXPIRED NOW 200")
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(resp["purged"].as_u64().unwrap(), 3);
}
