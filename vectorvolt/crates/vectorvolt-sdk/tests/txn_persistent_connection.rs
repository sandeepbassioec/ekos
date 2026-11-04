//! A transaction must span SDK calls (E13): BEGIN ... COMMIT on one client
//! runs on one persistent connection.
use std::sync::Arc;

use tempfile::TempDir;
use vectorvolt_engine::TableEngine;
use vectorvolt_net::{AuthConfig, Server, ServerConfig};
use vectorvolt_sdk::{SdkError, VectorVoltClient};

fn start() -> (TempDir, String) {
    let tmp = TempDir::new().unwrap();
    let engine = Arc::new(
        TableEngine::open(vec!["pk".into(), "val".into()], tmp.path(), 1_048_576).unwrap(),
    );
    let cfg = ServerConfig::new("127.0.0.1:0".parse().unwrap()).with_auth(AuthConfig::disabled());
    let server = Server::bind(engine, cfg).unwrap();
    let addr = server.local_addr().unwrap().to_string();
    std::thread::spawn(move || server.serve().unwrap());
    (tmp, addr)
}

#[tokio::test]
async fn multi_call_transaction_commits_atomically() {
    let (_tmp, addr) = start();
    let a = VectorVoltClient::connect(&addr).await.unwrap();
    let observer = VectorVoltClient::connect(&addr).await.unwrap();

    let txn = a.begin_txn().await.unwrap();
    a.insert("default", 1, &[1, 100]).await.unwrap();
    a.insert("default", 2, &[2, 200]).await.unwrap();
    // Buffered on a's connection: another connection sees nothing yet.
    assert_eq!(observer.get("default", 1).await.unwrap(), None);
    a.commit_txn(txn).await.unwrap();

    assert_eq!(
        observer.get("default", 1).await.unwrap(),
        Some(vec![1, 100])
    );
    assert_eq!(a.get("default", 2).await.unwrap(), Some(vec![2, 200]));
}

#[tokio::test]
async fn multi_call_transaction_abort_discards_writes() {
    let (_tmp, addr) = start();
    let a = VectorVoltClient::connect(&addr).await.unwrap();
    let txn = a.begin_txn().await.unwrap();
    a.insert("default", 7, &[7, 70]).await.unwrap();
    a.abort_txn(txn).await.unwrap();
    assert_eq!(a.get("default", 7).await.unwrap(), None);
}

#[tokio::test]
async fn txn_misuse_is_rejected_without_losing_the_transaction() {
    let (_tmp, addr) = start();
    let a = VectorVoltClient::connect(&addr).await.unwrap();
    let txn = a.begin_txn().await.unwrap();
    assert!(matches!(a.begin_txn().await, Err(SdkError::Net(_))));
    assert!(matches!(
        a.commit_txn(txn + 99).await,
        Err(SdkError::Net(_))
    ));
    a.insert("default", 3, &[3, 30]).await.unwrap();
    a.commit_txn(txn).await.unwrap();
    assert_eq!(a.get("default", 3).await.unwrap(), Some(vec![3, 30]));
}
