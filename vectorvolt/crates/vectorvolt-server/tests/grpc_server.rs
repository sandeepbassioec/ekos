//! Integration tests for the gRPC write server (VV-NET-004..016 / S16).

use vectorvolt_server::grpc::proto::write_service_client::WriteServiceClient;
use vectorvolt_server::grpc::proto::{
    AbortTxnRequest, BeginTxnRequest, CommitTxnRequest, DeleteRequest, ExecuteRequest,
    InsertRequest, PingRequest, QueryRequest,
};

async fn client(addr: std::net::SocketAddr) -> WriteServiceClient<tonic::transport::Channel> {
    let endpoint = format!("http://{addr}");
    WriteServiceClient::connect(endpoint).await.unwrap()
}

async fn start(
    dir: &tempfile::TempDir,
) -> (std::net::SocketAddr, tokio::sync::oneshot::Sender<()>) {
    vectorvolt_server::spawn_grpc_for_test(vec!["pk".to_string(), "age".to_string()], dir.path())
        .await
        .unwrap()
}

#[tokio::test]
async fn grpc_ping_returns_version() {
    let dir = tempfile::TempDir::new().unwrap();
    let (addr, _sd) = start(&dir).await;
    let mut c = client(addr).await;
    let reply = c.ping(PingRequest {}).await.unwrap().into_inner();
    assert!(!reply.version.is_empty());
}

#[tokio::test]
async fn grpc_insert_and_query() {
    let dir = tempfile::TempDir::new().unwrap();
    let (addr, _sd) = start(&dir).await;
    let mut c = client(addr).await;

    let r = c
        .insert(InsertRequest {
            pk: 1,
            values: vec![1, 25],
        })
        .await
        .unwrap()
        .into_inner();
    assert_eq!(r.status, "ok");

    let q = c.query(QueryRequest { pk: 1 }).await.unwrap().into_inner();
    assert!(q.found);
    assert_eq!(q.pk, 1);
    assert_eq!(q.values, vec![1, 25]);
}

#[tokio::test]
async fn grpc_query_missing_row() {
    let dir = tempfile::TempDir::new().unwrap();
    let (addr, _sd) = start(&dir).await;
    let mut c = client(addr).await;
    let q = c
        .query(QueryRequest { pk: 999 })
        .await
        .unwrap()
        .into_inner();
    assert!(!q.found);
}

#[tokio::test]
async fn grpc_delete_row() {
    let dir = tempfile::TempDir::new().unwrap();
    let (addr, _sd) = start(&dir).await;
    let mut c = client(addr).await;

    c.insert(InsertRequest {
        pk: 5,
        values: vec![5, 10],
    })
    .await
    .unwrap();
    let r = c
        .delete(DeleteRequest { pk: 5 })
        .await
        .unwrap()
        .into_inner();
    assert_eq!(r.status, "ok");

    let q = c.query(QueryRequest { pk: 5 }).await.unwrap().into_inner();
    assert!(!q.found);
}

#[tokio::test]
async fn grpc_txn_begin_returns_id() {
    let dir = tempfile::TempDir::new().unwrap();
    let (addr, _sd) = start(&dir).await;
    let mut c = client(addr).await;
    let reply = c.begin_txn(BeginTxnRequest {}).await.unwrap().into_inner();
    assert!(reply.txn_id > 0);
}

#[tokio::test]
async fn grpc_txn_commit_applies_ops() {
    let dir = tempfile::TempDir::new().unwrap();
    let (addr, _sd) = start(&dir).await;
    let mut c = client(addr).await;

    let txn_id = c
        .begin_txn(BeginTxnRequest {})
        .await
        .unwrap()
        .into_inner()
        .txn_id;

    let r = c
        .execute(ExecuteRequest {
            txn_id,
            op: "insert".into(),
            pk: 10,
            values: vec![10, 42],
        })
        .await
        .unwrap()
        .into_inner();
    assert_eq!(r.status, "buffered");

    let commit = c
        .commit_txn(CommitTxnRequest { txn_id })
        .await
        .unwrap()
        .into_inner();
    assert_eq!(commit.status, "committed");
    assert_eq!(commit.ops, 1);

    let q = c.query(QueryRequest { pk: 10 }).await.unwrap().into_inner();
    assert!(q.found);
}

#[tokio::test]
async fn grpc_txn_abort_discards_ops() {
    let dir = tempfile::TempDir::new().unwrap();
    let (addr, _sd) = start(&dir).await;
    let mut c = client(addr).await;

    let txn_id = c
        .begin_txn(BeginTxnRequest {})
        .await
        .unwrap()
        .into_inner()
        .txn_id;
    c.execute(ExecuteRequest {
        txn_id,
        op: "insert".into(),
        pk: 20,
        values: vec![20, 99],
    })
    .await
    .unwrap();

    let r = c
        .abort_txn(AbortTxnRequest { txn_id })
        .await
        .unwrap()
        .into_inner();
    assert_eq!(r.status, "aborted");

    let q = c.query(QueryRequest { pk: 20 }).await.unwrap().into_inner();
    assert!(!q.found);
}

#[tokio::test]
async fn grpc_txn_commit_multiple_ops() {
    let dir = tempfile::TempDir::new().unwrap();
    let (addr, _sd) = start(&dir).await;
    let mut c = client(addr).await;

    let txn_id = c
        .begin_txn(BeginTxnRequest {})
        .await
        .unwrap()
        .into_inner()
        .txn_id;
    for pk in [100u64, 101, 102] {
        c.execute(ExecuteRequest {
            txn_id,
            op: "insert".into(),
            pk,
            values: vec![pk, pk * 2],
        })
        .await
        .unwrap();
    }
    let commit = c
        .commit_txn(CommitTxnRequest { txn_id })
        .await
        .unwrap()
        .into_inner();
    assert_eq!(commit.ops, 3);
}

#[tokio::test]
async fn grpc_execute_unknown_op_errors() {
    let dir = tempfile::TempDir::new().unwrap();
    let (addr, _sd) = start(&dir).await;
    let mut c = client(addr).await;

    let txn_id = c
        .begin_txn(BeginTxnRequest {})
        .await
        .unwrap()
        .into_inner()
        .txn_id;
    let result = c
        .execute(ExecuteRequest {
            txn_id,
            op: "invalid_op".into(),
            pk: 1,
            values: vec![],
        })
        .await;
    assert!(result.is_err(), "unknown op should return gRPC error");
}

#[tokio::test]
async fn grpc_commit_unknown_txn_errors() {
    let dir = tempfile::TempDir::new().unwrap();
    let (addr, _sd) = start(&dir).await;
    let mut c = client(addr).await;
    let result = c.commit_txn(CommitTxnRequest { txn_id: 9999 }).await;
    assert!(
        result.is_err(),
        "committing unknown txn_id should return gRPC error"
    );
}

#[tokio::test]
async fn grpc_multiple_txn_ids_distinct() {
    let dir = tempfile::TempDir::new().unwrap();
    let (addr, _sd) = start(&dir).await;
    let mut c = client(addr).await;
    let id1 = c
        .begin_txn(BeginTxnRequest {})
        .await
        .unwrap()
        .into_inner()
        .txn_id;
    let id2 = c
        .begin_txn(BeginTxnRequest {})
        .await
        .unwrap()
        .into_inner()
        .txn_id;
    assert_ne!(id1, id2);
}

#[tokio::test]
async fn grpc_insert_with_all_columns() {
    let dir = tempfile::TempDir::new().unwrap();
    let (addr, _sd) = start(&dir).await;
    let mut c = client(addr).await;
    let r = c
        .insert(InsertRequest { pk: 77, values: vec![77, 30] })
        .await
        .unwrap()
        .into_inner();
    assert_eq!(r.status, "ok");
    let q = c.query(QueryRequest { pk: 77 }).await.unwrap().into_inner();
    assert!(q.found);
}
