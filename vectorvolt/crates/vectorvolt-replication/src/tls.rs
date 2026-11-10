//! TLS / mutual-TLS for the replication connection (tokio-rustls).
//!
//! Plaintext stays the default (tests, loopback). A leader given a
//! [`ReplicationTlsServer`] only speaks TLS; with a client CA it also demands
//! and verifies a client certificate (mTLS). A follower given a
//! [`ReplicationTlsClient`] verifies the leader's certificate against its CA
//! and, for mTLS, presents its own.

use std::path::Path;
use std::sync::Arc;

use rustls::pki_types::{CertificateDer, PrivateKeyDer, ServerName};
use rustls::server::WebPkiClientVerifier;
use rustls::{ClientConfig, RootCertStore, ServerConfig};
use tokio::io::{AsyncRead, AsyncWrite};
use tokio_rustls::{TlsAcceptor, TlsConnector};

use crate::error::{ReplResult, ReplicationError};

/// A bidirectional byte stream (plain TCP or TLS over TCP).
pub(crate) trait AsyncStream: AsyncRead + AsyncWrite + Unpin + Send {}
impl<T: AsyncRead + AsyncWrite + Unpin + Send> AsyncStream for T {}

/// Boxed transport used by the protocol loops.
pub(crate) type Conn = Box<dyn AsyncStream>;
pub(crate) type ConnRead = tokio::io::ReadHalf<Conn>;
pub(crate) type ConnWrite = tokio::io::WriteHalf<Conn>;

fn provider() -> Arc<rustls::crypto::CryptoProvider> {
    Arc::new(rustls::crypto::ring::default_provider())
}

fn tls_err(what: &str, e: impl std::fmt::Display) -> ReplicationError {
    ReplicationError::Config(format!("tls: {what}: {e}"))
}

fn certs(pem: &[u8]) -> ReplResult<Vec<CertificateDer<'static>>> {
    let v: Vec<_> = rustls_pemfile::certs(&mut &*pem)
        .collect::<Result<_, _>>()
        .map_err(|e| tls_err("read certificates", e))?;
    if v.is_empty() {
        return Err(ReplicationError::Config(
            "tls: no certificate in PEM".into(),
        ));
    }
    Ok(v)
}

fn key(pem: &[u8]) -> ReplResult<PrivateKeyDer<'static>> {
    rustls_pemfile::private_key(&mut &*pem)
        .map_err(|e| tls_err("read private key", e))?
        .ok_or_else(|| ReplicationError::Config("tls: no private key in PEM".into()))
}

fn roots(pem: &[u8]) -> ReplResult<RootCertStore> {
    let mut store = RootCertStore::empty();
    for c in certs(pem)? {
        store.add(c).map_err(|e| tls_err("add CA", e))?;
    }
    Ok(store)
}

fn read(p: &Path) -> ReplResult<Vec<u8>> {
    std::fs::read(p).map_err(|e| ReplicationError::Config(format!("{}: {e}", p.display())))
}

/// Leader-side TLS settings.
#[derive(Clone)]
pub struct ReplicationTlsServer {
    pub(crate) acceptor: TlsAcceptor,
    mutual: bool,
}

impl ReplicationTlsServer {
    /// `cert_pem` is the server chain, `key_pem` its private key. With
    /// `client_ca_pem` set, every follower must present a certificate that
    /// chains to it (mTLS).
    pub fn from_pem(
        cert_pem: &[u8],
        key_pem: &[u8],
        client_ca_pem: Option<&[u8]>,
    ) -> ReplResult<Self> {
        let builder = ServerConfig::builder_with_provider(provider())
            .with_safe_default_protocol_versions()
            .map_err(|e| tls_err("protocol versions", e))?;
        let builder = match client_ca_pem {
            Some(ca) => {
                let verifier =
                    WebPkiClientVerifier::builder_with_provider(Arc::new(roots(ca)?), provider())
                        .build()
                        .map_err(|e| tls_err("client verifier", e))?;
                builder.with_client_cert_verifier(verifier)
            }
            None => builder.with_no_client_auth(),
        };
        let cfg = builder
            .with_single_cert(certs(cert_pem)?, key(key_pem)?)
            .map_err(|e| tls_err("server certificate", e))?;
        Ok(Self {
            acceptor: TlsAcceptor::from(Arc::new(cfg)),
            mutual: client_ca_pem.is_some(),
        })
    }

    /// [`from_pem`](Self::from_pem) reading the PEM files at the given paths.
    pub fn from_files(cert: &Path, key: &Path, client_ca: Option<&Path>) -> ReplResult<Self> {
        let ca = client_ca.map(read).transpose()?;
        Self::from_pem(&read(cert)?, &read(key)?, ca.as_deref())
    }

    /// Whether followers must present a client certificate.
    pub fn requires_client_cert(&self) -> bool {
        self.mutual
    }
}

/// Follower-side TLS settings.
#[derive(Clone)]
pub struct ReplicationTlsClient {
    pub(crate) connector: TlsConnector,
    /// Name the leader's certificate must be valid for; `None` = the host part
    /// of the leader address.
    server_name: Option<String>,
}

impl ReplicationTlsClient {
    /// `ca_pem` is the CA that signed the leader's certificate. For mTLS pass
    /// the follower's own `(cert chain, key)` as `client_identity`.
    pub fn from_pem(
        ca_pem: &[u8],
        client_identity: Option<(&[u8], &[u8])>,
        server_name: Option<String>,
    ) -> ReplResult<Self> {
        let builder = ClientConfig::builder_with_provider(provider())
            .with_safe_default_protocol_versions()
            .map_err(|e| tls_err("protocol versions", e))?
            .with_root_certificates(roots(ca_pem)?);
        let cfg = match client_identity {
            Some((c, k)) => builder
                .with_client_auth_cert(certs(c)?, key(k)?)
                .map_err(|e| tls_err("client certificate", e))?,
            None => builder.with_no_client_auth(),
        };
        Ok(Self {
            connector: TlsConnector::from(Arc::new(cfg)),
            server_name,
        })
    }

    /// [`from_pem`](Self::from_pem) reading PEM files.
    pub fn from_files(
        ca: &Path,
        client_identity: Option<(&Path, &Path)>,
        server_name: Option<String>,
    ) -> ReplResult<Self> {
        let id = client_identity
            .map(|(c, k)| Ok::<_, ReplicationError>((read(c)?, read(k)?)))
            .transpose()?;
        Self::from_pem(
            &read(ca)?,
            id.as_ref().map(|(c, k)| (c.as_slice(), k.as_slice())),
            server_name,
        )
    }

    pub(crate) fn name_for(&self, leader_addr: &str) -> ReplResult<ServerName<'static>> {
        let host = match &self.server_name {
            Some(n) => n.as_str(),
            None => leader_addr
                .rsplit_once(':')
                .map(|(h, _)| h)
                .unwrap_or(leader_addr)
                .trim_start_matches('[')
                .trim_end_matches(']'),
        };
        ServerName::try_from(host.to_owned())
            .map_err(|e| ReplicationError::Config(format!("tls: bad server name {host:?}: {e}")))
    }
}
