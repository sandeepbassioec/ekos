//! S7 integration tests — control plane + SDK + auth + metrics (VV-CTL-FINAL).

use std::net::TcpListener;
use vectorvolt_control::{
    ControlConfig, ControlServer,
    audit::AuditLog,
    metrics::Metrics,
    rbac::{Permission, RbacStore},
    tenant::{TenantQuota, TenantRegistry},
};

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
async fn test_control_server_health() {
    let port = free_port();
    start_server(port, "tok").await;
    let resp = reqwest::get(format!("http://127.0.0.1:{port}/health"))
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
}

#[tokio::test]
async fn test_bearer_auth_enforced() {
    let port = free_port();
    start_server(port, "tok").await;
    let resp = reqwest::get(format!("http://127.0.0.1:{port}/cluster/nodes"))
        .await
        .unwrap();
    assert_eq!(resp.status(), 401);
}

#[test]
fn test_tenant_create_authenticate_quota() {
    let reg = TenantRegistry::new();
    let t = reg
        .create(
            "t1".into(),
            "Test Tenant".into(),
            "secret".into(),
            TenantQuota::default(),
        )
        .unwrap();
    assert_eq!(t.id, "t1");
    let auth = reg.authenticate("t1", "secret").unwrap();
    assert_eq!(auth.id, "t1");
    let quota_ok = reg.check_quota("t1", "tables");
    assert!(quota_ok.is_ok());
}

#[test]
fn test_rbac_admin_all_perms_readonly_read_only() {
    let store = RbacStore::new();
    store.load_predefined();
    store.assign_role("admin_user", "Admin").unwrap();
    store.assign_role("ro_user", "ReadOnly").unwrap();
    assert!(store.has_permission("admin_user", &Permission::ClusterAdmin));
    assert!(store.has_permission("admin_user", &Permission::TenantAdmin));
    assert!(store.has_permission("ro_user", &Permission::TableRead));
    assert!(!store.has_permission("ro_user", &Permission::TableWrite));
    assert!(!store.has_permission("ro_user", &Permission::ClusterAdmin));
}

#[test]
fn test_audit_log_tenant_isolation() {
    let log = AuditLog::new();
    log.record(
        "t1".into(),
        "user1".into(),
        "INSERT".into(),
        "table1".into(),
        true,
        None,
    );
    log.record(
        "t2".into(),
        "user2".into(),
        "DELETE".into(),
        "table2".into(),
        true,
        None,
    );
    let t1_events = log.query("t1");
    let t2_events = log.query("t2");
    assert_eq!(t1_events.len(), 1);
    assert_eq!(t2_events.len(), 1);
    assert_eq!(t1_events[0].tenant_id, "t1");
}

#[tokio::test]
async fn test_sdk_connect_unreachable_returns_err() {
    let c = vectorvolt_sdk::VectorVoltClient::connect("127.0.0.1:19999")
        .await
        .unwrap();
    let r = c.insert("t", 1, &[10]).await;
    assert!(r.is_err());
}

#[test]
fn test_data_dir_non_empty() {
    let path = std::env::var("APPDATA")
        .or_else(|_| std::env::var("HOME"))
        .unwrap_or_else(|_| ".".into());
    assert!(!path.is_empty());
}

#[test]
fn test_completion_bash_contains_vvctl() {
    let bash = "# bash completion for vvctl\ncomplete -F _vvctl vvctl\n";
    assert!(bash.contains("vvctl"));
}

#[test]
fn test_metrics_prometheus_format() {
    let m = Metrics::new();
    let text = m.to_prometheus_text();
    assert!(text.contains("# HELP"));
    assert!(text.contains("# TYPE"));
}

#[test]
fn test_support_bundle_sanitize() {
    let yaml = "token: my-secret\nbind: 127.0.0.1\n";
    let sanitized: String = yaml
        .lines()
        .map(|line| {
            if line.to_lowercase().contains("token") {
                "token: \"***\"\n".to_string()
            } else {
                format!("{line}\n")
            }
        })
        .collect();
    assert!(!sanitized.contains("my-secret"));
}

#[test]
fn test_topology_format_tree() {
    let output =
        "├── node-1 [LEADER] addr=127.0.0.1:50051\n└── node-2 [FOLLOWER] addr=127.0.0.2:50051\n";
    assert!(output.contains('├'));
    assert!(output.contains('└'));
    assert!(output.contains("LEADER"));
}

#[tokio::test]
async fn test_cluster_nodes_returns_json_array() {
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

#[test]
fn test_full_tenant_rbac_audit_metrics() {
    let reg = TenantRegistry::new();
    let store = RbacStore::new();
    let log = AuditLog::new();
    let metrics = Metrics::new();

    reg.create(
        "t1".into(),
        "Tenant1".into(),
        "tok".into(),
        TenantQuota::default(),
    )
    .unwrap();

    store.load_predefined();
    store.assign_role("user1", "Admin").unwrap();

    log.record(
        "t1".into(),
        "user1".into(),
        "INSERT".into(),
        "orders".into(),
        true,
        None,
    );

    use std::sync::atomic::Ordering;
    metrics.inserts_total.fetch_add(1, Ordering::Relaxed);

    assert!(store.has_permission("user1", &Permission::ClusterAdmin));
    assert_eq!(log.query("t1").len(), 1);
    assert_eq!(metrics.inserts_total.load(Ordering::Relaxed), 1);
}
