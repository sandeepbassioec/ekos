use crate::metrics::Metrics;
use crate::rbac::{Permission, RbacStore};
use crate::tenant::TenantRegistry;
use axum::Router;
use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Json};
use axum::routing::{get, post};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControlConfig {
    pub bind_addr: String,
    /// Legacy single admin token. It authenticates as the built-in `admin`
    /// user, which holds the `Admin` role.
    pub token: String,
    /// Additional API users, each limited to the permissions of their roles.
    #[serde(default)]
    pub users: Vec<ApiUser>,
}

/// One control-plane credential and the RBAC roles it acts with.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiUser {
    pub user_id: String,
    pub token: String,
    pub roles: Vec<String>,
}

/// Failure from a [`ControlBackend`] operation.
#[derive(Debug, Clone)]
pub enum BackendError {
    NotFound(String),
    Failed(String),
}

/// The live engine state the control API reports on.
///
/// Anything a backend cannot answer returns `None` (or the default), and the
/// API then answers 501 rather than inventing data.
pub trait ControlBackend: Send + Sync {
    fn list_tables(&self) -> Vec<String>;
    fn table_stats(&self, name: &str) -> Result<Value, BackendError>;
    fn snapshot_table(&self, name: &str) -> Result<Value, BackendError>;
    fn cluster_nodes(&self) -> Option<Vec<Value>> {
        None
    }
    fn cluster_shards(&self) -> Option<Vec<Value>> {
        None
    }
}

pub struct AppState {
    pub config: ControlConfig,
    /// Engine the endpoints report on; `None` = standalone, no engine data.
    pub backend: Option<Arc<dyn ControlBackend>>,
    pub tenants: Arc<TenantRegistry>,
    pub metrics: Arc<Metrics>,
    /// Role and permission store every handler checks (PP-37).
    pub rbac: Arc<RbacStore>,
    /// SHA-256 of each accepted token, with the user it authenticates as.
    credentials: Vec<([u8; 32], String)>,
}

fn hash_token(token: &str) -> [u8; 32] {
    use sha2::{Digest, Sha256};
    Sha256::digest(token.as_bytes()).into()
}

pub struct ControlServer {
    state: Arc<AppState>,
}

impl ControlServer {
    pub fn new(config: ControlConfig) -> Self {
        Self::build(config, None)
    }

    /// Control server reporting on a live engine.
    pub fn with_backend(config: ControlConfig, backend: Arc<dyn ControlBackend>) -> Self {
        Self::build(config, Some(backend))
    }

    fn build(config: ControlConfig, backend: Option<Arc<dyn ControlBackend>>) -> Self {
        let rbac = RbacStore::new();
        rbac.load_predefined();
        let mut credentials = Vec::new();
        rbac.assign_role("admin", "Admin")
            .expect("Admin is a predefined role");
        credentials.push((hash_token(&config.token), "admin".to_string()));
        for user in &config.users {
            for role in &user.roles {
                // An unknown role grants nothing, so the user simply has fewer
                // permissions; it is reported rather than failing startup.
                if let Err(e) = rbac.assign_role(&user.user_id, role) {
                    tracing::warn!(user = %user.user_id, error = %e, "ignoring unknown role");
                }
            }
            credentials.push((hash_token(&user.token), user.user_id.clone()));
        }
        Self {
            state: Arc::new(AppState {
                config,
                backend,
                tenants: Arc::new(TenantRegistry::new()),
                metrics: Arc::new(Metrics::new()),
                rbac: Arc::new(rbac),
                credentials,
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

/// Authenticate the bearer token and require `perm` for its user.
///
/// 401 when no token matches, 403 when the user lacks the permission. Every
/// stored credential is compared, in constant time, so the response time does
/// not reveal which one (if any) matched.
fn authorize(
    headers: &HeaderMap,
    state: &AppState,
    perm: Permission,
) -> Result<String, StatusCode> {
    use subtle::ConstantTimeEq;
    let presented = headers
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer "))
        .ok_or(StatusCode::UNAUTHORIZED)?;
    let presented = hash_token(presented);
    let mut user: Option<&str> = None;
    for (hash, user_id) in &state.credentials {
        if bool::from(hash.ct_eq(&presented)) {
            user = Some(user_id);
        }
    }
    let user = user.ok_or(StatusCode::UNAUTHORIZED)?;
    if state.rbac.has_permission(user, &perm) {
        Ok(user.to_string())
    } else {
        Err(StatusCode::FORBIDDEN)
    }
}

fn not_available(what: &str) -> axum::response::Response {
    (
        StatusCode::NOT_IMPLEMENTED,
        Json(json!({"error": "not available", "detail": what})),
    )
        .into_response()
}

fn backend_error(e: BackendError) -> axum::response::Response {
    match e {
        BackendError::NotFound(n) => (
            StatusCode::NOT_FOUND,
            Json(json!({"error": "table not found", "table": n})),
        )
            .into_response(),
        BackendError::Failed(m) => {
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": m}))).into_response()
        }
    }
}

async fn health_handler(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let mut body = json!({
        "status": "ok",
        "version": env!("CARGO_PKG_VERSION"),
        "engine_attached": state.backend.is_some(),
    });
    if let Some(b) = &state.backend {
        body["table_count"] = json!(b.list_tables().len());
    }
    Json(body)
}

async fn cluster_nodes_handler(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> impl IntoResponse {
    if let Err(e) = authorize(&headers, &state, Permission::ClusterRead) {
        return e.into_response();
    }
    match state.backend.as_ref().and_then(|b| b.cluster_nodes()) {
        Some(nodes) => Json(nodes).into_response(),
        None => not_available("no cluster membership source is attached"),
    }
}

async fn cluster_shards_handler(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> impl IntoResponse {
    if let Err(e) = authorize(&headers, &state, Permission::ClusterRead) {
        return e.into_response();
    }
    match state.backend.as_ref().and_then(|b| b.cluster_shards()) {
        Some(shards) => Json(shards).into_response(),
        None => not_available("no shard map source is attached"),
    }
}

async fn remove_node_handler(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> impl IntoResponse {
    if let Err(e) = authorize(&headers, &state, Permission::ClusterAdmin) {
        return e.into_response();
    }
    let _ = id;
    not_available("node removal is not supported by the attached backend")
}

async fn tables_handler(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> impl IntoResponse {
    if let Err(e) = authorize(&headers, &state, Permission::TableRead) {
        return e.into_response();
    }
    let tables: Vec<String> = match &state.backend {
        Some(b) => b.list_tables(),
        None => state
            .tenants
            .list()
            .iter()
            .flat_map(|t| t.tables.clone())
            .collect(),
    };
    Json(tables).into_response()
}

async fn table_stats_handler(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(name): Path<String>,
) -> impl IntoResponse {
    if let Err(e) = authorize(&headers, &state, Permission::TableRead) {
        return e.into_response();
    }
    let Some(backend) = state.backend.clone() else {
        return not_available("no engine is attached; table stats unavailable");
    };
    match backend.table_stats(&name) {
        Ok(stats) => Json(stats).into_response(),
        Err(e) => backend_error(e),
    }
}

async fn table_snapshot_handler(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(name): Path<String>,
) -> impl IntoResponse {
    if let Err(e) = authorize(&headers, &state, Permission::TableAdmin) {
        return e.into_response();
    }
    let Some(backend) = state.backend.clone() else {
        return not_available("no engine is attached; snapshots unavailable");
    };
    let n = name.clone();
    // Snapshots flush and compact: keep them off the async workers.
    match tokio::task::spawn_blocking(move || backend.snapshot_table(&n)).await {
        Ok(Ok(v)) => Json(v).into_response(),
        Ok(Err(e)) => backend_error(e),
        Err(e) => backend_error(BackendError::Failed(e.to_string())),
    }
}

async fn metrics_handler(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> impl IntoResponse {
    if let Err(e) = authorize(&headers, &state, Permission::MetricsRead) {
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
            users: vec![],
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

    fn rbac_server() -> ControlServer {
        ControlServer::new(ControlConfig {
            bind_addr: "127.0.0.1:0".to_string(),
            token: "test-token".to_string(),
            users: vec![ApiUser {
                user_id: "viewer".into(),
                token: "viewer-token".into(),
                roles: vec!["ReadOnly".into()],
            }],
        })
    }

    async fn status(app: Router, method: &str, uri: &str, token: &str) -> StatusCode {
        app.oneshot(
            Request::builder()
                .method(method)
                .uri(uri)
                .header("Authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap()
        .status()
    }

    #[tokio::test]
    async fn read_only_user_can_read_but_not_administer() {
        let s = rbac_server();
        assert_eq!(
            status(s.router(), "GET", "/tables", "viewer-token").await,
            StatusCode::OK
        );
        assert_eq!(
            status(s.router(), "POST", "/tables/t/snapshot", "viewer-token").await,
            StatusCode::FORBIDDEN
        );
        assert_eq!(
            status(
                s.router(),
                "POST",
                "/cluster/nodes/n1/remove",
                "viewer-token"
            )
            .await,
            StatusCode::FORBIDDEN
        );
        assert_eq!(
            status(s.router(), "GET", "/cluster/nodes", "viewer-token").await,
            StatusCode::FORBIDDEN
        );
    }

    #[tokio::test]
    async fn admin_token_keeps_full_access() {
        let s = rbac_server();
        // Authorised, but no engine attached: honest 501, not a fake snapshot.
        assert_eq!(
            status(s.router(), "POST", "/tables/t/snapshot", "test-token").await,
            StatusCode::NOT_IMPLEMENTED
        );
    }

    #[tokio::test]
    async fn unknown_token_is_unauthorized() {
        let s = rbac_server();
        assert_eq!(
            status(s.router(), "GET", "/tables", "nope").await,
            StatusCode::UNAUTHORIZED
        );
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

    struct MockBackend;
    impl ControlBackend for MockBackend {
        fn list_tables(&self) -> Vec<String> {
            vec!["orders".into(), "users".into()]
        }
        fn table_stats(&self, name: &str) -> Result<Value, BackendError> {
            if name == "orders" {
                Ok(json!({"table": "orders", "rows_flushed": 7}))
            } else {
                Err(BackendError::NotFound(name.into()))
            }
        }
        fn snapshot_table(&self, name: &str) -> Result<Value, BackendError> {
            self.table_stats(name).map(|_| json!({"row_count": 7}))
        }
    }

    fn backed() -> Router {
        ControlServer::with_backend(
            ControlConfig {
                bind_addr: "127.0.0.1:0".into(),
                token: "test-token".into(),
                users: vec![],
            },
            Arc::new(MockBackend),
        )
        .router()
    }

    async fn call(app: Router, method: &str, uri: &str) -> (StatusCode, Value) {
        let resp = app
            .oneshot(
                Request::builder()
                    .method(method)
                    .uri(uri)
                    .header("Authorization", auth_header())
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let st = resp.status();
        (st, body_json(resp).await)
    }

    #[tokio::test]
    async fn no_backend_means_501_not_fake_data() {
        for (m, u) in [
            ("GET", "/tables/t/stats"),
            ("POST", "/tables/t/snapshot"),
            ("GET", "/cluster/nodes"),
            ("GET", "/cluster/shards"),
            ("POST", "/cluster/nodes/n1/remove"),
        ] {
            let (st, body) = call(make_server().router(), m, u).await;
            assert_eq!(st, StatusCode::NOT_IMPLEMENTED, "{m} {u}");
            assert_eq!(body["error"], "not available");
        }
    }

    #[tokio::test]
    async fn backend_tables_stats_snapshot_are_real() {
        let (st, body) = call(backed(), "GET", "/tables").await;
        assert_eq!(st, StatusCode::OK);
        assert_eq!(body, json!(["orders", "users"]));
        let (st, body) = call(backed(), "GET", "/tables/orders/stats").await;
        assert_eq!(st, StatusCode::OK);
        assert_eq!(body["rows_flushed"], 7);
        let (st, _) = call(backed(), "GET", "/tables/ghost/stats").await;
        assert_eq!(st, StatusCode::NOT_FOUND);
        let (st, body) = call(backed(), "POST", "/tables/orders/snapshot").await;
        assert_eq!(st, StatusCode::OK);
        assert_eq!(body["row_count"], 7);
        let (st, _) = call(backed(), "POST", "/tables/ghost/snapshot").await;
        assert_eq!(st, StatusCode::NOT_FOUND);
        // Backend has no cluster source: still 501.
        let (st, _) = call(backed(), "GET", "/cluster/nodes").await;
        assert_eq!(st, StatusCode::NOT_IMPLEMENTED);
    }

    #[tokio::test]
    async fn health_reports_real_version_and_engine_state() {
        let (_, standalone) = {
            let resp = make_server()
                .router()
                .oneshot(
                    Request::builder()
                        .uri("/health")
                        .body(Body::empty())
                        .unwrap(),
                )
                .await
                .unwrap();
            (0, body_json(resp).await)
        };
        assert_eq!(standalone["version"], env!("CARGO_PKG_VERSION"));
        assert_eq!(standalone["engine_attached"], false);
        let resp = backed()
            .oneshot(
                Request::builder()
                    .uri("/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let b = body_json(resp).await;
        assert_eq!(b["engine_attached"], true);
        assert_eq!(b["table_count"], 2);
    }
}
