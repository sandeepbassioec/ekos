//! Tests for vvctl control subcommands (VV-CTL-006).
//!
//! Starts a real ControlServer on a free port for HTTP tests.

use std::net::TcpListener;
use vectorvolt_control::{ControlConfig, ControlServer};

fn free_port() -> u16 {
    TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port()
}

async fn start_server(port: u16, token: &str) {
    let cfg = ControlConfig {
        bind_addr: format!("127.0.0.1:{port}"),
        token: token.to_string(),
        users: vec![],
    };
    let server = ControlServer::new(cfg);
    let listener = tokio::net::TcpListener::bind(format!("127.0.0.1:{port}"))
        .await
        .unwrap();
    tokio::spawn(async move {
        axum::serve(listener, server.router()).await.unwrap();
    });
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
}

#[tokio::test]
async fn test_health_endpoint_returns_ok() {
    let port = free_port();
    start_server(port, "tok").await;
    let resp = reqwest::get(format!("http://127.0.0.1:{port}/health"))
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
}

#[tokio::test]
async fn test_nodes_without_auth_returns_401() {
    let port = free_port();
    start_server(port, "tok").await;
    let resp = reqwest::get(format!("http://127.0.0.1:{port}/cluster/nodes"))
        .await
        .unwrap();
    assert_eq!(resp.status(), 401);
}

#[tokio::test]
async fn test_nodes_without_source_returns_501() {
    let port = free_port();
    start_server(port, "tok").await;
    let client = reqwest::Client::new();
    let resp = client
        .get(format!("http://127.0.0.1:{port}/cluster/nodes"))
        .header("Authorization", "Bearer tok")
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 501);
}

#[tokio::test]
async fn test_shards_without_source_returns_501() {
    let port = free_port();
    start_server(port, "tok").await;
    let client = reqwest::Client::new();
    let resp = client
        .get(format!("http://127.0.0.1:{port}/cluster/shards"))
        .header("Authorization", "Bearer tok")
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 501);
}

#[tokio::test]
async fn test_metrics_with_auth_returns_200() {
    let port = free_port();
    start_server(port, "tok").await;
    let client = reqwest::Client::new();
    let resp = client
        .get(format!("http://127.0.0.1:{port}/metrics"))
        .header("Authorization", "Bearer tok")
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
}

#[tokio::test]
async fn test_snapshot_without_engine_returns_501() {
    let port = free_port();
    start_server(port, "tok").await;
    let client = reqwest::Client::new();
    let resp = client
        .post(format!("http://127.0.0.1:{port}/tables/orders/snapshot"))
        .header("Authorization", "Bearer tok")
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 501);
}

#[tokio::test]
async fn test_wrong_token_returns_401() {
    let port = free_port();
    start_server(port, "tok").await;
    let client = reqwest::Client::new();
    let resp = client
        .get(format!("http://127.0.0.1:{port}/cluster/nodes"))
        .header("Authorization", "Bearer wrong")
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 401);
}

#[tokio::test]
async fn test_nodes_501_body_says_not_available() {
    let port = free_port();
    start_server(port, "tok").await;
    let client = reqwest::Client::new();
    let resp = client
        .get(format!("http://127.0.0.1:{port}/cluster/nodes"))
        .header("Authorization", "Bearer tok")
        .send()
        .await
        .unwrap();
    let json: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(json["error"], "not available");
}

#[tokio::test]
async fn test_table_stats_without_engine_returns_501() {
    let port = free_port();
    start_server(port, "tok").await;
    let client = reqwest::Client::new();
    let resp = client
        .get(format!("http://127.0.0.1:{port}/tables/test/stats"))
        .header("Authorization", "Bearer tok")
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 501);
}

#[tokio::test]
async fn test_health_returns_status_ok() {
    let port = free_port();
    start_server(port, "tok").await;
    let resp = reqwest::get(format!("http://127.0.0.1:{port}/health"))
        .await
        .unwrap();
    let json: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(json["status"], "ok");
}

#[tokio::test]
async fn test_unknown_route_returns_404() {
    let port = free_port();
    start_server(port, "tok").await;
    let resp = reqwest::get(format!("http://127.0.0.1:{port}/does-not-exist"))
        .await
        .unwrap();
    assert_eq!(resp.status(), 404);
}

#[tokio::test]
async fn test_snapshot_501_has_no_fake_id() {
    let port = free_port();
    start_server(port, "tok").await;
    let client = reqwest::Client::new();
    let resp = client
        .post(format!("http://127.0.0.1:{port}/tables/orders/snapshot"))
        .header("Authorization", "Bearer tok")
        .send()
        .await
        .unwrap();
    let json: serde_json::Value = resp.json().await.unwrap();
    assert!(json.get("snapshot_id").is_none());
}

#[tokio::test]
async fn test_tables_endpoint_returns_200() {
    let port = free_port();
    start_server(port, "tok").await;
    let client = reqwest::Client::new();
    let resp = client
        .get(format!("http://127.0.0.1:{port}/tables"))
        .header("Authorization", "Bearer tok")
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
}

#[tokio::test]
async fn test_registry_backend_reports_real_tables_and_snapshots() {
    use std::sync::Arc;
    use vectorvolt_engine::{EngineOptions, TableRegistry};
    let dir = tempfile::tempdir().unwrap();
    let reg = Arc::new(TableRegistry::open(dir.path(), EngineOptions::default()).unwrap());
    let t = reg
        .create_table("orders", vec!["a".into(), "b".into()])
        .unwrap();
    t.insert(1, vec![10, 20]).unwrap();
    t.insert(2, vec![30, 40]).unwrap();

    let port = free_port();
    let server = ControlServer::with_backend(
        ControlConfig {
            bind_addr: format!("127.0.0.1:{port}"),
            token: "tok".into(),
            users: vec![],
        },
        Arc::new(vvctl::commands::control::RegistryBackend::new(reg)),
    );
    let listener = tokio::net::TcpListener::bind(format!("127.0.0.1:{port}"))
        .await
        .unwrap();
    tokio::spawn(async move {
        axum::serve(listener, server.router()).await.unwrap();
    });
    let base = format!("http://127.0.0.1:{port}");
    let c = reqwest::Client::new();
    let get = |p: &str| {
        c.get(format!("{base}{p}"))
            .header("Authorization", "Bearer tok")
    };

    let tables: Vec<String> = get("/tables").send().await.unwrap().json().await.unwrap();
    assert_eq!(tables, vec!["orders".to_string()]);

    let stats: serde_json::Value = get("/tables/orders/stats")
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(stats["columns"].as_array().unwrap().len(), 2);
    assert_eq!(stats["rows_buffered"], 2);

    assert_eq!(
        get("/tables/ghost/stats").send().await.unwrap().status(),
        404
    );

    let snap: serde_json::Value = c
        .post(format!("{base}/tables/orders/snapshot"))
        .header("Authorization", "Bearer tok")
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(snap["row_count"], 2);

    let health: serde_json::Value = reqwest::get(format!("{base}/health"))
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(health["table_count"], 1);
}
