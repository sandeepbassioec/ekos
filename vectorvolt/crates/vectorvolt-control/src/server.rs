use crate::metrics::Metrics;
use crate::tenant::TenantRegistry;
use axum::Router;
use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Json};
use axum::routing::{get, post};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControlConfig {
    pub bind_addr: String,
    pub token: String,
}

pub struct AppState {
    pub config: ControlConfig,
    pub tenants: Arc<TenantRegistry>,
    pub metrics: Arc<Metrics>,
}

pub struct ControlServer {
    state: Arc<AppState>,
}

impl ControlServer {
    pub fn new(config: ControlConfig) -> Self {
        Self {
            state: Arc::new(AppState {
                config,
                tenants: Arc::new(TenantRegistry::new()),
                metrics: Arc::new(Metrics::new()),
            }),
        }
    }

    pub fn router(&self) -> Router {
        let state = Arc::clone(&self.state);
        Router::new()
            .route("/health", get(health_handler))
            .route("/cluster/nodes", get(cluster_nodes_handler))
            .route("/cluster/shards", get(cluster_shards_handler))
            .route("/cluster/nodes/{id}/remove", post(remove_node_handler))
            .route("/tables", get(tables_handler))
            .route("/tables/{name}/stats", get(table_stats_handler))
            .route("/tables/{name}/snapshot", post(table_snapshot_handler))
            .route("/metrics", get(metrics_handler))
            .with_state(state)
    }
}

fn check_auth(headers: &HeaderMap, token: &str) -> Result<(), StatusCode> {
    match headers.get("Authorization") {
        None => Err(StatusCode::UNAUTHORIZED),
        Some(val) => {
            let s = val.to_str().unwrap_or("");
            let expected = format!("Bearer {token}");
            if s == expected {
                Ok(())
            } else {
                Err(StatusCode::UNAUTHORIZED)
            }
        }
    }
}

async fn health_handler() -> impl IntoResponse {
    Json(json!({"status": "ok", "version": "0.1.0"}))
}

async fn cluster_nodes_handler(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> impl IntoResponse {
    if let Err(e) = check_auth(&headers, &state.config.token) {
        return e.into_response();
    }
    let nodes: Vec<Value> = vec![
        json!({"id": "node-1", "addr": "127.0.0.1:7001", "role": "leader", "healthy": true}),
        json!({"id": "node-2", "addr": "127.0.0.1:7002", "role": "follower", "healthy": true}),
        json!({"id": "node-3", "addr": "127.0.0.1:7003", "role": "follower", "healthy": true}),
    ];
    Json(nodes).into_response()
}

async fn cluster_shards_handler(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> impl IntoResponse {
    if let Err(e) = check_auth(&headers, &state.config.token) {
        return e.into_response();
    }
    let shards: Vec<Value> = vec![
        json!({"id": "shard-0", "range_start": 0, "range_end": 16383, "leader": "node-1"}),
        json!({"id": "shard-1", "range_start": 16384, "range_end": 32767, "leader": "node-2"}),
    ];
    Json(shards).into_response()
}

async fn remove_node_handler(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> impl IntoResponse {
    if let Err(e) = check_auth(&headers, &state.config.token) {
        return e.into_response();
    }
    Json(json!({"removed": true, "node_id": id})).into_response()
}

async fn tables_handler(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> impl IntoResponse {
    if let Err(e) = check_auth(&headers, &state.config.token) {
        return e.into_response();
    }
    let tables: Vec<String> = state
        .tenants
        .list()
        .iter()
        .flat_map(|t| t.tables.clone())
        .collect();
    Json(tables).into_response()
}

async fn table_stats_handler(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(name): Path<String>,
) -> impl IntoResponse {
    if let Err(e) = check_auth(&headers, &state.config.token) {
        return e.into_response();
    }
    Json(json!({
        "table": name,
        "row_count": 0u64,
        "bytes": 0u64,
        "shard_count": 1u32,
        "index_count": 0u32
    }))
    .into_response()
}

async fn table_snapshot_handler(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(name): Path<String>,
) -> impl IntoResponse {
    if let Err(e) = check_auth(&headers, &state.config.token) {
        return e.into_response();
    }
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    Json(json!({"snapshot_id": format!("snap-{name}-{ts}")})).into_response()
}

async fn metrics_handler(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> impl IntoResponse {
    if let Err(e) = check_auth(&headers, &state.config.token) {
        return e.into_response();
    }
    let body = state.metrics.to_prometheus_text();
    (
        StatusCode::OK,
        [("content-type", "text/plain; version=0.0.4")],
        body,
    )
        .into_response()
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::Request;
    use tower::ServiceExt;

    fn make_server() -> ControlServer {
        ControlServer::new(ControlConfig {
            bind_addr: "127.0.0.1:0".to_string(),
            token: "test-token".to_string(),
        })
    }

    fn auth_header() -> &'static str {
        "Bearer test-token"
    }

    async fn body_json(resp: axum::http::Response<Body>) -> Value {
        let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        serde_json::from_slice(&bytes).unwrap()
    }

    #[tokio::test]
    async fn test_health_200() {
        let app = make_server().router();
        let resp = app
            .oneshot(
                Request::builder()
                    .uri("/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_health_no_auth_still_works() {
        let app = make_server().router();
        let resp = app
            .oneshot(
                Request::builder()
                    .uri("/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body = body_json(resp).await;
        assert_eq!(body["status"], "ok");
    }

    #[tokio::test]
    async fn test_cluster_nodes_200() {
        let app = make_server().router();
        let resp = app
            .oneshot(
                Request::builder()
                    .uri("/cluster/nodes")
                    .header("Authorization", auth_header())
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_tables_200() {
        let app = make_server().router();
        let resp = app
            .oneshot(
                Request::builder()
                    .uri("/tables")
                    .header("Authorization", auth_header())
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_table_stats_200() {
        let app = make_server().router();
        let resp = app
            .oneshot(
                Request::builder()
                    .uri("/tables/my_table/stats")
                    .header("Authorization", auth_header())
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body = body_json(resp).await;
        assert_eq!(body["table"], "my_table");
    }

    #[tokio::test]
    async fn test_table_snapshot_200() {
        let app = make_server().router();
        let resp = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/tables/my_table/snapshot")
                    .header("Authorization", auth_header())
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body = body_json(resp).await;
        let snap_id = body["snapshot_id"].as_str().unwrap();
        assert!(snap_id.starts_with("snap-my_table-"));
    }

    #[tokio::test]
    async fn test_metrics_200() {
        let app = make_server().router();
        let resp = app
            .oneshot(
                Request::builder()
                    .uri("/metrics")
                    .header("Authorization", auth_header())
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_wrong_token_401() {
        let app = make_server().router();
        let resp = app
            .oneshot(
                Request::builder()
                    .uri("/cluster/nodes")
                    .header("Authorization", "Bearer wrong-token")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_missing_token_401() {
        let app = make_server().router();
        let resp = app
            .oneshot(
                Request::builder()
                    .uri("/cluster/nodes")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_unknown_route_404() {
        let app = make_server().router();
        let resp = app
            .oneshot(
                Request::builder()
                    .uri("/does-not-exist")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_cluster_shards_200() {
        let app = make_server().router();
        let resp = app
            .oneshot(
                Request::builder()
                    .uri("/cluster/shards")
                    .header("Authorization", auth_header())
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_remove_node_200() {
        let app = make_server().router();
        let resp = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/cluster/nodes/node-1/remove")
                    .header("Authorization", auth_header())
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body = body_json(resp).await;
        assert_eq!(body["removed"], true);
        assert_eq!(body["node_id"], "node-1");
    }

    #[tokio::test]
    async fn test_cluster_nodes_content_type_json() {
        let app = make_server().router();
        let resp = app
            .oneshot(
                Request::builder()
                    .uri("/cluster/nodes")
                    .header("Authorization", auth_header())
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let ct = resp
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");
        assert!(ct.contains("application/json"));
    }
}
