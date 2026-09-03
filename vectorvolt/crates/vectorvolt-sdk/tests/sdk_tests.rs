//! vectorvolt-sdk unit tests (VV-SDK-001).
use vectorvolt_sdk::{SdkError, VectorVoltClient};

const UNREACHABLE: &str = "127.0.0.1:19900";

#[tokio::test]
async fn test_connect_returns_ok() {
    let c = VectorVoltClient::connect(UNREACHABLE).await;
    assert!(c.is_ok());
}

#[tokio::test]
async fn test_insert_unreachable_returns_net_err() {
    let c = VectorVoltClient::connect(UNREACHABLE).await.unwrap();
    let r = c.insert("t", 1, &[10, 20]).await;
    assert!(matches!(r, Err(SdkError::Net(_))));
}

#[tokio::test]
async fn test_get_unreachable_returns_net_err() {
    let c = VectorVoltClient::connect(UNREACHABLE).await.unwrap();
    let r = c.get("t", 1).await;
    assert!(matches!(r, Err(SdkError::Net(_))));
}

#[tokio::test]
async fn test_delete_unreachable_returns_net_err() {
    let c = VectorVoltClient::connect(UNREACHABLE).await.unwrap();
    let r = c.delete("t", 1).await;
    assert!(matches!(r, Err(SdkError::Net(_))));
}

#[tokio::test]
async fn test_scan_unreachable_returns_net_err() {
    let c = VectorVoltClient::connect(UNREACHABLE).await.unwrap();
    let r = c.scan("t").await;
    assert!(matches!(r, Err(SdkError::Net(_))));
}

#[tokio::test]
async fn test_ping_unreachable_returns_net_err() {
    let c = VectorVoltClient::connect(UNREACHABLE).await.unwrap();
    let r = c.ping().await;
    assert!(matches!(r, Err(SdkError::Net(_))));
}

#[tokio::test]
async fn test_health_unreachable_returns_net_err() {
    let c = VectorVoltClient::connect(UNREACHABLE).await.unwrap();
    let r = c.health().await;
    assert!(matches!(r, Err(SdkError::Net(_))));
}

#[tokio::test]
async fn test_begin_txn_unreachable_returns_net_err() {
    let c = VectorVoltClient::connect(UNREACHABLE).await.unwrap();
    let r = c.begin_txn().await;
    assert!(matches!(r, Err(SdkError::Net(_))));
}

#[tokio::test]
async fn test_commit_txn_unreachable_returns_net_err() {
    let c = VectorVoltClient::connect(UNREACHABLE).await.unwrap();
    let r = c.commit_txn(42).await;
    assert!(matches!(r, Err(SdkError::Net(_))));
}

#[tokio::test]
async fn test_abort_txn_unreachable_returns_net_err() {
    let c = VectorVoltClient::connect(UNREACHABLE).await.unwrap();
    let r = c.abort_txn(42).await;
    assert!(matches!(r, Err(SdkError::Net(_))));
}

#[test]
fn test_sdk_error_net_display() {
    let e = SdkError::Net("connection refused".into());
    assert!(e.to_string().contains("net:"));
}

#[test]
fn test_sdk_error_not_connected_display() {
    let e = SdkError::NotConnected;
    assert!(e.to_string().contains("not connected"));
}

#[test]
fn test_sdk_error_op_display() {
    let e = SdkError::Op("timeout".into());
    assert!(e.to_string().contains("operation failed:"));
}
