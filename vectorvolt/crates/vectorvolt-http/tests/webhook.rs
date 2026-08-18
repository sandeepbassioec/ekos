/// Layer 70 — CDC Webhook Sink.
///
/// Routes tested:
///   POST /tables/{t}/webhooks        → register sink, returns {id}
///   GET  /tables/{t}/webhooks        → list sinks for table
///   DELETE /tables/{t}/webhooks/{id} → remove sink
///
/// Delivery tests use a tiny in-process capture server (axum on :0) that
/// records received JSON payloads.
use std::sync::Arc;
use std::time::Duration;

use axum::Router;
use axum::extract::State;
use axum::routing::post;
use parking_lot::Mutex;
use reqwest::{Client, StatusCode};
use serde_json::Value;
use tempfile::TempDir;
use tokio::net::TcpListener;
use vectorvolt_engine::{EngineOptions, TableRegistry};
use vectorvolt_http::{HttpServer, HttpServerConfig};

const FLUSH: usize = 1_048_576;

// ── capture server ────────────────────────────────────────────────────────────

type Captured = Arc<Mutex<Vec<Value>>>;

async fn capture_handler(State(store): State<Captured>, body: axum::body::Bytes) -> StatusCode {
    if let Ok(v) = serde_json::from_slice::<Value>(&body) {
        store.lock().push(v);
    }
    StatusCode::OK
}

async fn start_capture_server() -> (String, Captured) {
    let store: Captured = Arc::new(Mutex::new(Vec::new()));
    let app = Router::new()
        .route("/events", post(capture_handler))
        .with_state(Arc::clone(&store));
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    (format!("http://{addr}/events"), store)
}

// ── VectorVolt harness ────────────────────────────────────────────────────────

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
            .timeout(Duration::from_secs(10))
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

    async fn create_table(&self, table: &str) {
        self.client
            .post(self.url(&format!("/tables/{table}")))
            .json(&serde_json::json!({ "columns": ["a"] }))
            .send()
            .await
            .unwrap();
    }

    async fn insert(&self, table: &str, pk: u64, val: u64) {
        self.client
            .post(self.url(&format!("/tables/{table}/rows/{pk}")))
            .json(&serde_json::json!({ "values": [val] }))
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

    async fn add_webhook(&self, table: &str, url: &str) -> (StatusCode, Value) {
        let resp = self
            .client
            .post(self.url(&format!("/tables/{table}/webhooks")))
            .json(&serde_json::json!({ "url": url }))
            .send()
            .await
            .unwrap();
        let status = resp.status();
        let body: Value = resp.json().await.unwrap();
        (status, body)
    }

    async fn list_webhooks(&self, table: &str) -> (StatusCode, Value) {
        let resp = self
            .client
            .get(self.url(&format!("/tables/{table}/webhooks")))
            .send()
            .await
            .unwrap();
        let status = resp.status();
        let body: Value = resp.json().await.unwrap();
        (status, body)
    }

    async fn delete_webhook(&self, table: &str, id: u64) -> StatusCode {
        self.client
            .delete(self.url(&format!("/tables/{table}/webhooks/{id}")))
            .send()
            .await
            .unwrap()
            .status()
    }
}

// ── tests ─────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn register_webhook_returns_id() {
    let h = Harness::start().await;
    h.create_table("t").await;
    let (status, body) = h.add_webhook("t", "http://example.com/events").await;
    assert_eq!(status, StatusCode::OK);
    assert!(body["id"].as_u64().is_some());
}

#[tokio::test]
async fn register_webhook_unknown_table_404() {
    let h = Harness::start().await;
    let (status, _) = h.add_webhook("no_such_table", "http://example.com").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn list_webhooks_empty() {
    let h = Harness::start().await;
    h.create_table("t").await;
    let (status, body) = h.list_webhooks("t").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["webhooks"].as_array().unwrap().len(), 0);
}

#[tokio::test]
async fn list_webhooks_unknown_table_404() {
    let h = Harness::start().await;
    let (status, _) = h.list_webhooks("no_such_table").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn list_webhooks_shows_registered() {
    let h = Harness::start().await;
    h.create_table("t").await;
    let (_, b1) = h.add_webhook("t", "http://example.com/a").await;
    let (_, b2) = h.add_webhook("t", "http://example.com/b").await;
    let id1 = b1["id"].as_u64().unwrap();
    let id2 = b2["id"].as_u64().unwrap();
    let (_, list) = h.list_webhooks("t").await;
    let hooks = list["webhooks"].as_array().unwrap();
    assert_eq!(hooks.len(), 2);
    let ids: Vec<u64> = hooks.iter().map(|e| e["id"].as_u64().unwrap()).collect();
    assert!(ids.contains(&id1));
    assert!(ids.contains(&id2));
}

#[tokio::test]
async fn delete_webhook_removes_it() {
    let h = Harness::start().await;
    h.create_table("t").await;
    let (_, body) = h.add_webhook("t", "http://example.com").await;
    let id = body["id"].as_u64().unwrap();
    let status = h.delete_webhook("t", id).await;
    assert_eq!(status, StatusCode::OK);
    let (_, list) = h.list_webhooks("t").await;
    assert_eq!(list["webhooks"].as_array().unwrap().len(), 0);
}

#[tokio::test]
async fn delete_webhook_unknown_id_404() {
    let h = Harness::start().await;
    h.create_table("t").await;
    let status = h.delete_webhook("t", 9999).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn delete_webhook_wrong_table_404() {
    let h = Harness::start().await;
    h.create_table("a").await;
    h.create_table("b").await;
    let (_, body) = h.add_webhook("a", "http://example.com").await;
    let id = body["id"].as_u64().unwrap();
    let status = h.delete_webhook("b", id).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn webhook_receives_insert_event() {
    let h = Harness::start().await;
    h.create_table("t").await;
    let (cap_url, store) = start_capture_server().await;
    h.add_webhook("t", &cap_url).await;
    h.insert("t", 1, 100).await;
    tokio::time::sleep(Duration::from_millis(200)).await;
    let captured = store.lock().clone();
    assert!(!captured.is_empty(), "expected at least one captured event");
    let ev = &captured[0];
    assert_eq!(ev["pk"], 1);
    assert_eq!(ev["kind"], "insert");
}

#[tokio::test]
async fn webhook_receives_delete_event() {
    let h = Harness::start().await;
    h.create_table("t").await;
    h.insert("t", 1, 10).await;
    let (cap_url, store) = start_capture_server().await;
    h.add_webhook("t", &cap_url).await;
    h.delete_row("t", 1).await;
    tokio::time::sleep(Duration::from_millis(200)).await;
    let captured = store.lock().clone();
    assert!(!captured.is_empty());
    let ev = &captured[0];
    assert_eq!(ev["pk"], 1);
    assert_eq!(ev["kind"], "delete");
}

#[tokio::test]
async fn webhook_multiple_events_delivered() {
    let h = Harness::start().await;
    h.create_table("t").await;
    let (cap_url, store) = start_capture_server().await;
    h.add_webhook("t", &cap_url).await;
    for i in 1u64..=5 {
        h.insert("t", i, i * 10).await;
    }
    tokio::time::sleep(Duration::from_millis(300)).await;
    let captured = store.lock().clone();
    assert_eq!(captured.len(), 5);
}

#[tokio::test]
async fn webhook_stopped_after_delete() {
    let h = Harness::start().await;
    h.create_table("t").await;
    let (cap_url, store) = start_capture_server().await;
    let (_, body) = h.add_webhook("t", &cap_url).await;
    let id = body["id"].as_u64().unwrap();
    h.delete_webhook("t", id).await;
    // Give the abort a moment to take effect.
    tokio::time::sleep(Duration::from_millis(50)).await;
    h.insert("t", 1, 99).await;
    tokio::time::sleep(Duration::from_millis(200)).await;
    let captured = store.lock().clone();
    // After deletion the background task is aborted — no events should arrive.
    assert_eq!(captured.len(), 0);
}

#[tokio::test]
async fn webhook_multi_table_isolation() {
    let h = Harness::start().await;
    h.create_table("a").await;
    h.create_table("b").await;
    let (cap_a, store_a) = start_capture_server().await;
    let (cap_b, store_b) = start_capture_server().await;
    h.add_webhook("a", &cap_a).await;
    h.add_webhook("b", &cap_b).await;
    h.insert("a", 1, 1).await;
    h.insert("b", 2, 2).await;
    tokio::time::sleep(Duration::from_millis(300)).await;
    let a = store_a.lock().clone();
    let b = store_b.lock().clone();
    assert_eq!(a.len(), 1);
    assert_eq!(a[0]["pk"], 1);
    assert_eq!(b.len(), 1);
    assert_eq!(b[0]["pk"], 2);
}

#[tokio::test]
async fn webhook_list_shows_url() {
    let h = Harness::start().await;
    h.create_table("t").await;
    h.add_webhook("t", "http://example.com/hook").await;
    let (_, list) = h.list_webhooks("t").await;
    let hooks = list["webhooks"].as_array().unwrap();
    assert_eq!(hooks[0]["url"], "http://example.com/hook");
}
