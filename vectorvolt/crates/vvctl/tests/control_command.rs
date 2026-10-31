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
async fn test_nodes_with_auth_returns_200() {
    let port = free_port();
    start_server(port, "tok").await;
    let client = reqwest::Client::new();
    let resp = client
        .get(format!("http://127.0.0.1:{port}/cluster/nodes"))
        .header("Authorization", "Bearer tok")
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
}

#[tokio::test]
async fn test_shards_with_auth_returns_200() {
    let port = free_port();
    start_server(port, "tok").await;
    let client = reqwest::Client::new();
    let resp = client
        .get(format!("http://127.0.0.1:{port}/cluster/shards"))
        .header("Authorization", "Bearer tok")
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
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
async fn test_snapshot_with_auth_returns_200() {
    let port = free_port();
    start_server(port, "tok").await;
    let client = reqwest::Client::new();
    let resp = client
        .post(format!("http://127.0.0.1:{port}/tables/orders/snapshot"))
        .header("Authorization", "Bearer tok")
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
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
async fn test_nodes_response_is_json_array() {
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
    assert!(json.is_array());
}

#[tokio::test]
async fn test_table_stats_returns_200() {
    let port = free_port();
    start_server(port, "tok").await;
    let client = reqwest::Client::new();
    let resp = client
        .get(format!("http://127.0.0.1:{port}/tables/test/stats"))
        .header("Authorization", "Bearer tok")
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
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
async fn test_snapshot_response_has_snapshot_id() {
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
    assert!(json["snapshot_id"].is_string());
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
