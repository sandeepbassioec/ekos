//! D1 — the HTTP gateway over TLS, and mutual TLS.

use std::sync::Arc;
use std::time::Duration;

use rustls::pki_types::{CertificateDer, PrivateKeyDer, PrivatePkcs8KeyDer};
use tempfile::TempDir;
use vectorvolt_engine::{EngineOptions, TableRegistry};
use vectorvolt_http::{HttpServer, HttpServerConfig};

struct Pki {
    server_cert_pem: String,
    server_cert: CertificateDer<'static>,
    server_key: PrivateKeyDer<'static>,
    ca_cert: CertificateDer<'static>,
    /// Client certificate + key PEM, as `reqwest::Identity::from_pem` wants.
    client_identity_pem: String,
}

fn pki() -> Pki {
    let server =
        rcgen::generate_simple_self_signed(vec!["localhost".to_string(), "127.0.0.1".to_string()])
            .unwrap();
    let ca_key = rcgen::KeyPair::generate().unwrap();
    let mut ca_params = rcgen::CertificateParams::new(Vec::<String>::new()).unwrap();
    ca_params.is_ca = rcgen::IsCa::Ca(rcgen::BasicConstraints::Unconstrained);
    ca_params
        .distinguished_name
        .push(rcgen::DnType::CommonName, "http test client CA");
    let ca = ca_params.self_signed(&ca_key).unwrap();
    let client_key = rcgen::KeyPair::generate().unwrap();
    let mut client_params = rcgen::CertificateParams::new(vec!["client".to_string()]).unwrap();
    client_params.extended_key_usages = vec![rcgen::ExtendedKeyUsagePurpose::ClientAuth];
    let client = client_params.signed_by(&client_key, &ca, &ca_key).unwrap();
    Pki {
        server_cert_pem: server.cert.pem(),
        server_cert: server.cert.der().clone(),
        server_key: PrivateKeyDer::Pkcs8(PrivatePkcs8KeyDer::from(server.key_pair.serialize_der())),
        ca_cert: ca.der().clone(),
        client_identity_pem: format!("{}{}", client.pem(), client_key.serialize_pem()),
    }
}

fn provider() -> Arc<rustls::crypto::CryptoProvider> {
    Arc::new(rustls::crypto::ring::default_provider())
}

fn server_tls(p: &Pki, mtls: bool) -> Arc<rustls::ServerConfig> {
    let b = rustls::ServerConfig::builder_with_provider(provider())
        .with_safe_default_protocol_versions()
        .unwrap();
    let b = if mtls {
        let mut roots = rustls::RootCertStore::empty();
        roots.add(p.ca_cert.clone()).unwrap();
        let v = rustls::server::WebPkiClientVerifier::builder_with_provider(
            Arc::new(roots),
            provider(),
        )
        .build()
        .unwrap();
        b.with_client_cert_verifier(v)
    } else {
        b.with_no_client_auth()
    };
    Arc::new(
        b.with_single_cert(vec![p.server_cert.clone()], p.server_key.clone_key())
            .unwrap(),
    )
}

async fn start(dir: &TempDir, tls: Arc<rustls::ServerConfig>) -> std::net::SocketAddr {
    let reg = Arc::new(TableRegistry::open(dir.path(), EngineOptions::new(1_048_576)).unwrap());
    let cfg = HttpServerConfig::new("127.0.0.1:0".parse().unwrap()).with_tls(tls);
    let srv = HttpServer::bind(reg, cfg).await.unwrap();
    assert!(srv.is_tls());
    let addr = srv.local_addr().unwrap();
    tokio::spawn(srv.serve());
    addr
}

fn client(p: &Pki, identity: bool) -> reqwest::Client {
    let mut b = reqwest::Client::builder()
        .use_rustls_tls()
        .tls_built_in_root_certs(false)
        .add_root_certificate(reqwest::Certificate::from_pem(p.server_cert_pem.as_bytes()).unwrap())
        .timeout(Duration::from_secs(10));
    if identity {
        b = b.identity(reqwest::Identity::from_pem(p.client_identity_pem.as_bytes()).unwrap());
    }
    b.build().unwrap()
}

#[tokio::test]
async fn https_handshake_succeeds_and_serves_requests() {
    let dir = TempDir::new().unwrap();
    let p = pki();
    let addr = start(&dir, server_tls(&p, false)).await;
    let resp = client(&p, false)
        .get(format!("https://localhost:{}/health", addr.port()))
        .send()
        .await
        .unwrap();
    assert!(resp.status().is_success());
}

#[tokio::test]
async fn plaintext_http_is_not_served_on_a_tls_listener() {
    let dir = TempDir::new().unwrap();
    let p = pki();
    let addr = start(&dir, server_tls(&p, false)).await;
    let r = reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .build()
        .unwrap()
        .get(format!("http://{addr}/health"))
        .send()
        .await;
    assert!(
        r.is_err(),
        "plaintext HTTP was answered by the TLS listener"
    );
    // ...and the listener survives it.
    let ok = client(&p, false)
        .get(format!("https://localhost:{}/health", addr.port()))
        .send()
        .await
        .unwrap();
    assert!(ok.status().is_success());
}

#[tokio::test]
async fn mtls_rejects_a_client_without_a_certificate() {
    let dir = TempDir::new().unwrap();
    let p = pki();
    let addr = start(&dir, server_tls(&p, true)).await;
    let r = client(&p, false)
        .get(format!("https://localhost:{}/health", addr.port()))
        .send()
        .await;
    assert!(r.is_err(), "a client without a certificate was served");
}

#[tokio::test]
async fn mtls_accepts_a_client_with_a_ca_issued_certificate() {
    let dir = TempDir::new().unwrap();
    let p = pki();
    let addr = start(&dir, server_tls(&p, true)).await;
    let resp = client(&p, true)
        .get(format!("https://localhost:{}/health", addr.port()))
        .send()
        .await
        .unwrap();
    assert!(resp.status().is_success());
}
