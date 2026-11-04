//! D1 — the gRPC write server over TLS, and mutual TLS.

use std::sync::Arc;
use std::time::Duration;

use tonic::transport::{Certificate, Channel, ClientTlsConfig, Identity};
use vectorvolt_engine::TableEngine;
use vectorvolt_server::grpc::GrpcServer;
use vectorvolt_server::grpc::proto::PingRequest;
use vectorvolt_server::grpc::proto::write_service_client::WriteServiceClient;

struct Pki {
    server_cert: String,
    server_key: String,
    ca_cert: String,
    client_cert: String,
    client_key: String,
}

fn pki() -> Pki {
    let server = rcgen::generate_simple_self_signed(vec!["localhost".to_string()]).unwrap();
    let ca_key = rcgen::KeyPair::generate().unwrap();
    let mut ca_params = rcgen::CertificateParams::new(Vec::<String>::new()).unwrap();
    ca_params.is_ca = rcgen::IsCa::Ca(rcgen::BasicConstraints::Unconstrained);
    ca_params
        .distinguished_name
        .push(rcgen::DnType::CommonName, "grpc test client CA");
    let ca = ca_params.self_signed(&ca_key).unwrap();
    let client_key = rcgen::KeyPair::generate().unwrap();
    let mut client_params = rcgen::CertificateParams::new(vec!["client".to_string()]).unwrap();
    client_params.extended_key_usages = vec![rcgen::ExtendedKeyUsagePurpose::ClientAuth];
    let client = client_params.signed_by(&client_key, &ca, &ca_key).unwrap();
    Pki {
        server_cert: server.cert.pem(),
        server_key: server.key_pair.serialize_pem(),
        ca_cert: ca.pem(),
        client_cert: client.pem(),
        client_key: client_key.serialize_pem(),
    }
}

async fn start(
    dir: &tempfile::TempDir,
    p: &Pki,
    mtls: bool,
) -> (std::net::SocketAddr, tokio::sync::oneshot::Sender<()>) {
    let engine = Arc::new(
        TableEngine::open(
            vec!["pk".to_string(), "v".to_string()],
            dir.path(),
            1_000_000,
        )
        .unwrap(),
    );
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let ca = mtls.then_some(p.ca_cert.as_bytes());
    let srv = GrpcServer::standalone(addr, engine).with_tls_pem(&p.server_cert, &p.server_key, ca);
    assert!(srv.is_tls());
    let (tx, rx) = tokio::sync::oneshot::channel();
    tokio::spawn(srv.run_with_listener(listener, rx));
    (addr, tx)
}

async fn ping(
    addr: std::net::SocketAddr,
    p: &Pki,
    identity: bool,
) -> Result<String, Box<dyn std::error::Error>> {
    let mut tls = ClientTlsConfig::new()
        .ca_certificate(Certificate::from_pem(&p.server_cert))
        .domain_name("localhost");
    if identity {
        tls = tls.identity(Identity::from_pem(&p.client_cert, &p.client_key));
    }
    let channel = Channel::from_shared(format!("https://127.0.0.1:{}", addr.port()))?
        .tls_config(tls)?
        .connect_timeout(Duration::from_secs(5))
        .timeout(Duration::from_secs(5))
        .connect()
        .await?;
    let reply = WriteServiceClient::new(channel)
        .ping(PingRequest {})
        .await?;
    Ok(reply.into_inner().version)
}

#[tokio::test]
async fn grpc_tls_handshake_succeeds_and_serves_rpcs() {
    let dir = tempfile::TempDir::new().unwrap();
    let p = pki();
    let (addr, _sd) = start(&dir, &p, false).await;
    let version = ping(addr, &p, false).await.unwrap();
    assert!(!version.is_empty());
}

#[tokio::test]
async fn grpc_tls_refuses_plaintext_clients() {
    let dir = tempfile::TempDir::new().unwrap();
    let p = pki();
    let (addr, _sd) = start(&dir, &p, false).await;
    let plain = async {
        let mut c = WriteServiceClient::connect(format!("http://{addr}")).await?;
        c.ping(PingRequest {}).await?;
        Ok::<_, Box<dyn std::error::Error>>(())
    };
    let r = tokio::time::timeout(Duration::from_secs(10), plain).await;
    assert!(
        !matches!(r, Ok(Ok(()))),
        "plaintext gRPC was served by the TLS listener"
    );
}

#[tokio::test]
async fn grpc_mtls_rejects_a_client_without_a_certificate() {
    let dir = tempfile::TempDir::new().unwrap();
    let p = pki();
    let (addr, _sd) = start(&dir, &p, true).await;
    assert!(
        ping(addr, &p, false).await.is_err(),
        "a client without a certificate was served"
    );
}

#[tokio::test]
async fn grpc_mtls_accepts_a_client_with_a_ca_issued_certificate() {
    let dir = tempfile::TempDir::new().unwrap();
    let p = pki();
    let (addr, _sd) = start(&dir, &p, true).await;
    assert!(!ping(addr, &p, true).await.unwrap().is_empty());
}
