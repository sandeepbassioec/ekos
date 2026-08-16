/// Layer 42 — HTTP gateway Bearer-token auth enforcement.
///
/// When `HttpServerConfig::with_token` is set, every data/DDL endpoint must
/// return 401 unless `Authorization: Bearer <token>` is present.
/// Observability routes (`/health`, `/metrics`) are always public.
use std::sync::Arc;

use reqwest::Client;
use serde_json::{Value, json};
use tempfile::TempDir;
use vectorvolt_engine::{EngineOptions, TableRegistry};
use vectorvolt_http::{HttpServer, HttpServerConfig};

const TOKEN: &str = "super-secret-token";
const FLUSH_THRESHOLD: usize = 1_048_576;

struct Harness {
    base_url: String,
    _dir: TempDir,
    authed: Client,
    unauthed: Client,
}

impl Harness {
    async fn start(token: Option<&str>) -> Self {
        let dir = TempDir::new().unwrap();
        let opts = EngineOptions::new(FLUSH_THRESHOLD);
        let reg = Arc::new(TableRegistry::open(dir.path(), opts).unwrap());
        let mut cfg = HttpServerConfig::new("127.0.0.1:0".parse().unwrap());
        if let Some(t) = token {
            cfg = cfg.with_token(t);
        }
        let srv = HttpServer::bind(reg, cfg).await.unwrap();
        let addr = srv.local_addr().unwrap();
        tokio::spawn(async move { srv.serve().await.unwrap() });
        let timeout = std::time::Duration::from_secs(5);
        let authed = Client::builder()
            .timeout(timeout)
            .default_headers({
                let mut h = reqwest::header::HeaderMap::new();
                if let Some(t) = token {
                    h.insert("Authorization", format!("Bearer {t}").parse().unwrap());
                }
                h
            })
            .build()
            .unwrap();
        let unauthed = Client::builder().timeout(timeout).build().unwrap();
        Harness {
            base_url: format!("http://{addr}"),
            _dir: dir,
            authed,
            unauthed,
        }
    }

    async fn start_with_table(token: Option<&str>, table: &str, cols: &[&str]) -> Self {
        let h = Self::start(token).await;
        let body = json!({ "columns": cols.iter().map(|s| s.to_string()).collect::<Vec<_>>() });
        let resp = h
            .authed
            .post(format!("{}/tables/{table}", h.base_url))
            .json(&body)
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status().as_u16(), 201, "setup: create table failed");
        h
    }

    fn url(&self, path: &str) -> String {
        format!("{}{path}", self.base_url)
    }
}

// ── no token configured — everything is public ───────────────────────────────

#[tokio::test]
async fn no_token_configured_allows_all() {
    let h = Harness::start_with_table(None, "t", &["pk", "val"]).await;
    let resp = h
        .unauthed
        .post(h.url("/tables/t/rows/1"))
        .json(&json!({ "values": [1, 99] }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status().as_u16(), 200);
}

// ── correct token is accepted ─────────────────────────────────────────────────

#[tokio::test]
async fn correct_token_allows_insert() {
    let h = Harness::start_with_table(Some(TOKEN), "t", &["pk", "val"]).await;
    let resp = h
        .authed
        .post(h.url("/tables/t/rows/1"))
        .json(&json!({ "values": [1, 42] }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status().as_u16(), 200);
}

#[tokio::test]
async fn correct_token_allows_get() {
    let h = Harness::start_with_table(Some(TOKEN), "t", &["pk", "val"]).await;
    h.authed
        .post(h.url("/tables/t/rows/7"))
        .json(&json!({ "values": [7, 11] }))
        .send()
        .await
        .unwrap();
    let resp = h
        .authed
        .get(h.url("/tables/t/rows/7"))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status().as_u16(), 200);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["pk"], 7);
}

#[tokio::test]
async fn correct_token_allows_list_tables() {
    let h = Harness::start_with_table(Some(TOKEN), "t", &["pk", "val"]).await;
    let resp = h.authed.get(h.url("/tables")).send().await.unwrap();
    assert_eq!(resp.status().as_u16(), 200);
}

// ── missing token returns 401 ─────────────────────────────────────────────────

#[tokio::test]
async fn missing_token_insert_returns_401() {
    let h = Harness::start_with_table(Some(TOKEN), "t", &["pk", "val"]).await;
    let resp = h
        .unauthed
        .post(h.url("/tables/t/rows/1"))
        .json(&json!({ "values": [1, 99] }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status().as_u16(), 401);
}

#[tokio::test]
async fn missing_token_get_returns_401() {
    let h = Harness::start_with_table(Some(TOKEN), "t", &["pk", "val"]).await;
    let resp = h
        .unauthed
        .get(h.url("/tables/t/rows/1"))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status().as_u16(), 401);
}

#[tokio::test]
async fn missing_token_delete_returns_401() {
    let h = Harness::start_with_table(Some(TOKEN), "t", &["pk", "val"]).await;
    let resp = h
        .unauthed
        .delete(h.url("/tables/t/rows/1"))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status().as_u16(), 401);
}

#[tokio::test]
async fn missing_token_scan_returns_401() {
    let h = Harness::start_with_table(Some(TOKEN), "t", &["pk", "val"]).await;
    let resp = h
        .unauthed
        .get(h.url("/tables/t/scan?col=0&lo=0&hi=100"))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status().as_u16(), 401);
}

#[tokio::test]
async fn missing_token_list_tables_returns_401() {
    let h = Harness::start(Some(TOKEN)).await;
    let resp = h.unauthed.get(h.url("/tables")).send().await.unwrap();
    assert_eq!(resp.status().as_u16(), 401);
}

#[tokio::test]
async fn missing_token_create_table_returns_401() {
    let h = Harness::start(Some(TOKEN)).await;
    let resp = h
        .unauthed
        .post(h.url("/tables/new"))
        .json(&json!({ "columns": ["pk", "val"] }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status().as_u16(), 401);
}

// ── wrong token returns 401 ───────────────────────────────────────────────────

#[tokio::test]
async fn wrong_token_returns_401() {
    let h = Harness::start_with_table(Some(TOKEN), "t", &["pk", "val"]).await;
    let resp = h
        .unauthed
        .get(h.url("/tables/t/rows/1"))
        .header("Authorization", "Bearer wrong-token")
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status().as_u16(), 401);
}

#[tokio::test]
async fn malformed_auth_header_returns_401() {
    let h = Harness::start_with_table(Some(TOKEN), "t", &["pk", "val"]).await;
    let resp = h
        .unauthed
        .get(h.url("/tables/t/rows/1"))
        .header("Authorization", TOKEN) // missing "Bearer " prefix
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status().as_u16(), 401);
}

// ── observability routes are always public ────────────────────────────────────

#[tokio::test]
async fn health_is_always_public() {
    let h = Harness::start(Some(TOKEN)).await;
    let resp = h.unauthed.get(h.url("/health")).send().await.unwrap();
    assert_eq!(resp.status().as_u16(), 200);
}

#[tokio::test]
async fn metrics_is_always_public() {
    let h = Harness::start(Some(TOKEN)).await;
    let resp = h.unauthed.get(h.url("/metrics")).send().await.unwrap();
    assert_eq!(resp.status().as_u16(), 200);
}

// ── 401 response body is valid JSON ──────────────────────────────────────────

#[tokio::test]
async fn unauthorized_response_body_is_json() {
    let h = Harness::start(Some(TOKEN)).await;
    let resp = h.unauthed.get(h.url("/tables")).send().await.unwrap();
    assert_eq!(resp.status().as_u16(), 401);
    let body: Value = resp.json().await.expect("body must be valid JSON");
    assert!(body["error"].is_string(), "error field must be present");
}
