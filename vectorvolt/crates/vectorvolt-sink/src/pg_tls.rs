//! TLS for [`PostgresSink`](crate::PostgresSink): a pure-Rust (rustls, `ring`)
//! connector for `tokio-postgres`. No OpenSSL / native TLS.
//!
//! Modes (mirroring libpq `sslmode`):
//! * `disable`     - plaintext (`NoTls`).
//! * `require`     - encrypted, server certificate is **not** verified
//!   (protects against passive sniffing only).
//! * `verify-full` - encrypted, certificate chain and host name verified
//!   against `ca_file` (PEM) or, when absent, the bundled Mozilla roots.

use std::future::Future;
use std::io;
use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};

use rustls::client::danger::{HandshakeSignatureValid, ServerCertVerified, ServerCertVerifier};
use rustls::crypto::{CryptoProvider, ring};
use rustls::pki_types::{CertificateDer, ServerName, UnixTime};
use rustls::{ClientConfig, DigitallySignedStruct, RootCertStore, SignatureScheme};
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};
use tokio_postgres::tls::{ChannelBinding, MakeTlsConnect, TlsConnect};

use crate::error::{SinkError, SinkResult};

/// TLS mode for the Postgres connection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PgTlsMode {
    #[default]
    Disable,
    Require,
    VerifyFull,
}

impl PgTlsMode {
    /// Parse `disable` / `require` / `verify-full` (also `verify_full`).
    pub fn parse(s: &str) -> SinkResult<Self> {
        match s.trim().to_ascii_lowercase().replace('_', "-").as_str() {
            "disable" | "disabled" | "off" => Ok(Self::Disable),
            "require" => Ok(Self::Require),
            "verify-full" => Ok(Self::VerifyFull),
            other => Err(SinkError::Other(format!(
                "postgres tls: unknown mode {other:?} (expected disable, require, verify-full)"
            ))),
        }
    }
}

/// TLS options for [`PostgresSinkConfig`](crate::PostgresSinkConfig).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PostgresTlsConfig {
    pub mode: PgTlsMode,
    /// PEM file with trusted CA certificate(s) (`verify-full`; also adds roots
    /// when set with other modes' verification, which `require` ignores).
    pub ca_file: Option<PathBuf>,
}

impl PostgresTlsConfig {
    /// Build the connector; `None` when TLS is disabled.
    pub fn connector(&self) -> SinkResult<Option<PgTlsConnector>> {
        match self.mode {
            PgTlsMode::Disable => Ok(None),
            PgTlsMode::Require => Ok(Some(PgTlsConnector::new(client_config(None, false)?))),
            PgTlsMode::VerifyFull => Ok(Some(PgTlsConnector::new(client_config(
                self.ca_file.as_deref(),
                true,
            )?))),
        }
    }
}

fn provider() -> Arc<CryptoProvider> {
    Arc::new(ring::default_provider())
}

fn client_config(ca_file: Option<&Path>, verify: bool) -> SinkResult<ClientConfig> {
    let builder = ClientConfig::builder_with_provider(provider())
        .with_safe_default_protocol_versions()
        .map_err(|e| SinkError::Other(format!("postgres tls: {e}")))?;
    if !verify {
        return Ok(builder
            .dangerous()
            .with_custom_certificate_verifier(Arc::new(NoVerify(provider())))
            .with_no_client_auth());
    }
    let mut roots = RootCertStore::empty();
    match ca_file {
        Some(path) => {
            let pem = std::fs::read(path).map_err(|e| {
                SinkError::Other(format!(
                    "postgres tls: read ca_file {}: {e}",
                    path.display()
                ))
            })?;
            let mut added = 0usize;
            for cert in rustls_pemfile::certs(&mut pem.as_slice()) {
                let cert = cert.map_err(|e| {
                    SinkError::Other(format!(
                        "postgres tls: parse ca_file {}: {e}",
                        path.display()
                    ))
                })?;
                roots
                    .add(cert)
                    .map_err(|e| SinkError::Other(format!("postgres tls: bad CA cert: {e}")))?;
                added += 1;
            }
            if added == 0 {
                return Err(SinkError::Other(format!(
                    "postgres tls: no certificates found in ca_file {}",
                    path.display()
                )));
            }
        }
        None => roots.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned()),
    }
    Ok(builder.with_root_certificates(roots).with_no_client_auth())
}

/// Accepts any server certificate (`sslmode=require` semantics) while still
/// verifying handshake signatures.
#[derive(Debug)]
struct NoVerify(Arc<CryptoProvider>);

impl ServerCertVerifier for NoVerify {
    fn verify_server_cert(
        &self,
        _end_entity: &CertificateDer<'_>,
        _intermediates: &[CertificateDer<'_>],
        _server_name: &ServerName<'_>,
        _ocsp: &[u8],
        _now: UnixTime,
    ) -> Result<ServerCertVerified, rustls::Error> {
        Ok(ServerCertVerified::assertion())
    }

    fn verify_tls12_signature(
        &self,
        message: &[u8],
        cert: &CertificateDer<'_>,
        dss: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, rustls::Error> {
        rustls::crypto::verify_tls12_signature(
            message,
            cert,
            dss,
            &self.0.signature_verification_algorithms,
        )
    }

    fn verify_tls13_signature(
        &self,
        message: &[u8],
        cert: &CertificateDer<'_>,
        dss: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, rustls::Error> {
        rustls::crypto::verify_tls13_signature(
            message,
            cert,
            dss,
            &self.0.signature_verification_algorithms,
        )
    }

    fn supported_verify_schemes(&self) -> Vec<SignatureScheme> {
        self.0.signature_verification_algorithms.supported_schemes()
    }
}

/// `tokio-postgres` TLS connector backed by rustls.
#[derive(Clone)]
pub struct PgTlsConnector(tokio_rustls::TlsConnector);

impl PgTlsConnector {
    fn new(cfg: ClientConfig) -> Self {
        Self(tokio_rustls::TlsConnector::from(Arc::new(cfg)))
    }
}

impl<S> MakeTlsConnect<S> for PgTlsConnector
where
    S: AsyncRead + AsyncWrite + Unpin + Send + 'static,
{
    type Stream = PgTlsStream<S>;
    type TlsConnect = PgTlsConnect;
    type Error = rustls::pki_types::InvalidDnsNameError;

    fn make_tls_connect(&mut self, domain: &str) -> Result<PgTlsConnect, Self::Error> {
        let name = ServerName::try_from(domain.to_owned())?;
        Ok(PgTlsConnect {
            connector: self.0.clone(),
            name,
        })
    }
}

/// One in-flight TLS handshake.
pub struct PgTlsConnect {
    connector: tokio_rustls::TlsConnector,
    name: ServerName<'static>,
}

impl<S> TlsConnect<S> for PgTlsConnect
where
    S: AsyncRead + AsyncWrite + Unpin + Send + 'static,
{
    type Stream = PgTlsStream<S>;
    type Error = io::Error;
    type Future = Pin<Box<dyn Future<Output = io::Result<PgTlsStream<S>>> + Send>>;

    fn connect(self, stream: S) -> Self::Future {
        Box::pin(async move {
            let s = self.connector.connect(self.name, stream).await?;
            Ok(PgTlsStream(s))
        })
    }
}

/// TLS-wrapped stream handed to `tokio-postgres`.
pub struct PgTlsStream<S>(tokio_rustls::client::TlsStream<S>);

impl<S: AsyncRead + AsyncWrite + Unpin> AsyncRead for PgTlsStream<S> {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        Pin::new(&mut self.0).poll_read(cx, buf)
    }
}

impl<S: AsyncRead + AsyncWrite + Unpin> AsyncWrite for PgTlsStream<S> {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        Pin::new(&mut self.0).poll_write(cx, buf)
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Pin::new(&mut self.0).poll_flush(cx)
    }

    fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Pin::new(&mut self.0).poll_shutdown(cx)
    }
}

impl<S: AsyncRead + AsyncWrite + Unpin> tokio_postgres::tls::TlsStream for PgTlsStream<S> {
    fn channel_binding(&self) -> ChannelBinding {
        ChannelBinding::none()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mode_parsing() {
        assert_eq!(PgTlsMode::parse("disable").unwrap(), PgTlsMode::Disable);
        assert_eq!(PgTlsMode::parse("Require").unwrap(), PgTlsMode::Require);
        assert_eq!(
            PgTlsMode::parse("verify-full").unwrap(),
            PgTlsMode::VerifyFull
        );
        assert_eq!(
            PgTlsMode::parse("verify_full").unwrap(),
            PgTlsMode::VerifyFull
        );
        assert!(PgTlsMode::parse("prefer").is_err());
        assert!(PgTlsMode::parse("").is_err());
    }

    #[test]
    fn default_is_disabled_and_builds_no_connector() {
        let c = PostgresTlsConfig::default();
        assert_eq!(c.mode, PgTlsMode::Disable);
        assert!(c.connector().unwrap().is_none());
    }

    #[test]
    fn require_builds_without_ca() {
        let c = PostgresTlsConfig {
            mode: PgTlsMode::Require,
            ca_file: None,
        };
        assert!(c.connector().unwrap().is_some());
    }

    #[test]
    fn verify_full_uses_bundled_roots_without_ca_file() {
        let c = PostgresTlsConfig {
            mode: PgTlsMode::VerifyFull,
            ca_file: None,
        };
        assert!(c.connector().unwrap().is_some());
    }

    #[test]
    fn verify_full_missing_ca_file_errors() {
        let c = PostgresTlsConfig {
            mode: PgTlsMode::VerifyFull,
            ca_file: Some("/nonexistent/ca.pem".into()),
        };
        let e = c.connector().err().expect("must fail");
        assert!(e.to_string().contains("ca_file"), "{e}");
    }

    #[test]
    fn verify_full_empty_ca_file_errors() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("ca.pem");
        std::fs::write(&p, "not a certificate\n").unwrap();
        let c = PostgresTlsConfig {
            mode: PgTlsMode::VerifyFull,
            ca_file: Some(p),
        };
        let e = c.connector().err().expect("must fail");
        assert!(e.to_string().contains("no certificates"), "{e}");
    }
}
