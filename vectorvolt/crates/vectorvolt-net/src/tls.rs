//! TLS transport (VV-SEC-005/006).
//!
//! Authentication answered "who is this". TLS answers the two questions it could
//! not: **is anyone listening**, and **am I talking to the right server**. Without
//! it the session after the handshake is plaintext, and a man in the middle can
//! relay the challenge–response exchange and then own the connection.
//!
//! ## Certificate pinning, not "disable verification"
//!
//! Almost every VectorVolt deployment will use a self-signed certificate: it is an
//! internal database, not a public website, and nobody is buying a CA cert for a
//! service on a private subnet.
//!
//! The usual accommodation for that is a `danger_accept_invalid_certs` switch. It
//! is a trap — it disables *all* verification, so the client will accept any
//! certificate from anyone, which is precisely the man-in-the-middle it was
//! supposed to prevent. It turns TLS into encryption without authentication, and
//! encryption to an unverified peer is encryption to the attacker.
//!
//! So this offers [`ClientTlsConfig::pin_certificate`] instead: trust exactly this
//! certificate and nothing else. It is *stronger* than public CA verification, not
//! weaker — a pinned client cannot be fooled by a mis-issued cert from any CA on
//! earth. It is also the honest shape of the deployment: you know which server you
//! mean, so say so.
//!
//! [`ClientTlsConfig::with_root_store`] exists for the case where a real CA is in
//! play. There is deliberately **no** option to skip verification.
//!
//! ## Hostname verification still applies
//!
//! Pinning the certificate does not disable the name check: the client still
//! verifies the server name against the certificate's SANs. Both matter — pinning
//! answers "is this the cert I expect", the name check answers "is this cert being
//! used for the host I asked for". Connecting to `127.0.0.1` with a cert issued for
//! `localhost` requires passing `localhost` as the server name, which is why
//! [`ClientTlsConfig`] carries it separately from the socket address.

use std::io::{Read, Write};
use std::net::TcpStream;
use std::sync::Arc;

use rustls::pki_types::{CertificateDer, PrivateKeyDer, ServerName};
use rustls::{ClientConfig, ClientConnection, RootCertStore, ServerConfig, ServerConnection};

use crate::{NetError, NetResult};

/// The crypto provider, built explicitly rather than installed globally.
///
/// `rustls` can take a process-wide default provider, but that is global mutable
/// state: a library that installs one fights any other crate that does, and
/// forgetting to install it turns into a panic at first use rather than a
/// compile error. Passing it per config costs one line and cannot fail that way.
fn provider() -> Arc<rustls::crypto::CryptoProvider> {
    Arc::new(rustls::crypto::ring::default_provider())
}

// ── Server ────────────────────────────────────────────────────────────────────

/// Server-side TLS: the certificate chain this server presents, and its key.
#[derive(Clone)]
pub struct ServerTlsConfig {
    inner: Arc<ServerConfig>,
}

impl std::fmt::Debug for ServerTlsConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // No contents. The config holds the private key.
        f.write_str("ServerTlsConfig(<configured>)")
    }
}

impl ServerTlsConfig {
    /// Build from PEM bytes.
    ///
    /// `cert_pem` may be a chain — leaf first, then intermediates, which is the
    /// order TLS requires. A chain in the wrong order fails at the client with a
    /// confusing "unknown issuer", so it is worth getting right at build time.
    pub fn from_pem(cert_pem: &[u8], key_pem: &[u8]) -> NetResult<Self> {
        let certs: Vec<CertificateDer<'static>> = rustls_pemfile::certs(&mut &cert_pem[..])
            .collect::<Result<_, _>>()
            .map_err(|e| NetError::Tls(format!("reading certificate PEM: {e}")))?;
        if certs.is_empty() {
            return Err(NetError::Tls(
                "certificate PEM contained no certificates".into(),
            ));
        }

        let key: PrivateKeyDer<'static> = rustls_pemfile::private_key(&mut &key_pem[..])
            .map_err(|e| NetError::Tls(format!("reading private key PEM: {e}")))?
            .ok_or_else(|| NetError::Tls("private key PEM contained no key".into()))?;

        let inner = ServerConfig::builder_with_provider(provider())
            .with_safe_default_protocol_versions()
            .map_err(|e| NetError::Tls(format!("protocol versions: {e}")))?
            // No client certificates. Client identity comes from the
            // challenge–response handshake, which already exists and already
            // carries a scope. Two identity systems that can disagree is worse
            // than one — see the module docs on why the handshake stays.
            .with_no_client_auth()
            .with_single_cert(certs, key)
            .map_err(|e| NetError::Tls(format!("certificate and key do not match: {e}")))?;

        Ok(ServerTlsConfig {
            inner: Arc::new(inner),
        })
    }

    /// Load from files.
    pub fn from_files(
        cert_path: impl AsRef<std::path::Path>,
        key_path: impl AsRef<std::path::Path>,
    ) -> NetResult<Self> {
        let cert = std::fs::read(cert_path)?;
        let key = std::fs::read(key_path)?;
        Self::from_pem(&cert, &key)
    }

    fn new_connection(&self) -> NetResult<ServerConnection> {
        ServerConnection::new(Arc::clone(&self.inner))
            .map_err(|e| NetError::Tls(format!("starting the TLS session: {e}")))
    }
}

// ── Client ────────────────────────────────────────────────────────────────────

/// Client-side TLS: what to trust, and the name to verify against.
#[derive(Clone)]
pub struct ClientTlsConfig {
    inner: Arc<ClientConfig>,
    server_name: String,
}

impl std::fmt::Debug for ClientTlsConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ClientTlsConfig(server_name: {})", self.server_name)
    }
}

impl ClientTlsConfig {
    /// Trust exactly this certificate, and no other.
    ///
    /// The right choice for a self-signed internal deployment. Stronger than
    /// public CA verification: a pinned client cannot be fooled by a mis-issued
    /// certificate from any CA.
    ///
    /// `server_name` must match a SAN on the certificate. It is separate from the
    /// address because they legitimately differ — connecting to `127.0.0.1` with a
    /// certificate issued for `localhost` is normal and must still verify the name.
    pub fn pin_certificate(cert_pem: &[u8], server_name: impl Into<String>) -> NetResult<Self> {
        let certs: Vec<CertificateDer<'static>> = rustls_pemfile::certs(&mut &cert_pem[..])
            .collect::<Result<_, _>>()
            .map_err(|e| NetError::Tls(format!("reading pinned certificate: {e}")))?;
        if certs.is_empty() {
            return Err(NetError::Tls(
                "pinned certificate PEM contained no certificates".into(),
            ));
        }
        let mut roots = RootCertStore::empty();
        for c in certs {
            roots
                .add(c)
                .map_err(|e| NetError::Tls(format!("pinning certificate: {e}")))?;
        }
        Self::with_root_store(roots, server_name)
    }

    /// Trust a caller-supplied root store. For deployments with a real CA.
    pub fn with_root_store(
        roots: RootCertStore,
        server_name: impl Into<String>,
    ) -> NetResult<Self> {
        let server_name = server_name.into();
        // Validated here rather than at connect time, so a bad name is a
        // configuration error rather than a runtime surprise on first use.
        ServerName::try_from(server_name.clone())
            .map_err(|e| NetError::Tls(format!("invalid server name {server_name:?}: {e}")))?;

        let inner = ClientConfig::builder_with_provider(provider())
            .with_safe_default_protocol_versions()
            .map_err(|e| NetError::Tls(format!("protocol versions: {e}")))?
            .with_root_certificates(roots)
            .with_no_client_auth();

        Ok(ClientTlsConfig {
            inner: Arc::new(inner),
            server_name,
        })
    }

    pub fn server_name(&self) -> &str {
        &self.server_name
    }

    fn new_connection(&self) -> NetResult<ClientConnection> {
        let name = ServerName::try_from(self.server_name.clone())
            .map_err(|e| NetError::Tls(format!("invalid server name: {e}")))?;
        ClientConnection::new(Arc::clone(&self.inner), name)
            .map_err(|e| NetError::Tls(format!("starting the TLS session: {e}")))
    }
}

// ── Connections ───────────────────────────────────────────────────────────────

macro_rules! conn_impl {
    ($name:ident, $tls:ty) => {
        impl $name {
            /// The underlying socket, for `set_nodelay`, timeouts, and addresses.
            pub fn socket(&self) -> &TcpStream {
                match self {
                    $name::Plain(s) => s,
                    $name::Tls(s) => s.get_ref(),
                }
            }

            pub fn is_tls(&self) -> bool {
                matches!(self, $name::Tls(_))
            }
        }

        impl Read for $name {
            fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
                match self {
                    $name::Plain(s) => s.read(buf),
                    $name::Tls(s) => s.read(buf),
                }
            }
        }

        impl Write for $name {
            fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
                match self {
                    $name::Plain(s) => s.write(buf),
                    $name::Tls(s) => s.write(buf),
                }
            }
            fn flush(&mut self) -> std::io::Result<()> {
                match self {
                    $name::Plain(s) => s.flush(),
                    $name::Tls(s) => s.flush(),
                }
            }
        }

        impl std::fmt::Debug for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(
                    f,
                    "{}({})",
                    stringify!($name),
                    if self.is_tls() { "tls" } else { "plain" }
                )
            }
        }
        // Silence the unused-type warning when the macro is expanded.
        const _: fn() = || {
            fn _assert(_: Option<Box<$tls>>) {}
        };
    };
}

/// A server-side connection, encrypted or not.
pub enum ServerConn {
    Plain(TcpStream),
    Tls(Box<rustls::StreamOwned<ServerConnection, TcpStream>>),
}

conn_impl!(ServerConn, rustls::StreamOwned<ServerConnection, TcpStream>);

impl ServerConn {
    /// Wrap an accepted socket, performing the TLS handshake if configured.
    ///
    /// The handshake is completed **here**, not lazily on first read.
    /// `StreamOwned` would do it on demand, which reports a handshake failure as
    /// an error on an unrelated later read — so a client presenting a bad
    /// certificate would look like a protocol error rather than a TLS one. Doing
    /// it up front means the failure is attributed correctly and the connection is
    /// dropped before any request is parsed.
    pub fn accept(stream: TcpStream, tls: Option<&ServerTlsConfig>) -> NetResult<Self> {
        match tls {
            None => Ok(ServerConn::Plain(stream)),
            Some(cfg) => {
                let mut conn = cfg.new_connection()?;
                let mut stream = stream;
                while conn.is_handshaking() {
                    conn.complete_io(&mut stream)
                        .map_err(|e| NetError::Tls(format!("handshake failed: {e}")))?;
                }
                Ok(ServerConn::Tls(Box::new(rustls::StreamOwned::new(
                    conn, stream,
                ))))
            }
        }
    }
}

/// A client-side connection, encrypted or not.
pub enum ClientConn {
    Plain(TcpStream),
    Tls(Box<rustls::StreamOwned<ClientConnection, TcpStream>>),
}

conn_impl!(ClientConn, rustls::StreamOwned<ClientConnection, TcpStream>);

impl ClientConn {
    /// Wrap a connected socket, performing the TLS handshake if configured.
    ///
    /// Completed eagerly for the same reason as the server side, plus one that
    /// matters more here: **certificate verification happens during the
    /// handshake**. Deferring it would mean `connect` succeeds against a server
    /// with an untrusted certificate and only fails later, which is exactly the
    /// window where a caller might have already sent credentials.
    pub fn connect(stream: TcpStream, tls: Option<&ClientTlsConfig>) -> NetResult<Self> {
        match tls {
            None => Ok(ClientConn::Plain(stream)),
            Some(cfg) => {
                let mut conn = cfg.new_connection()?;
                let mut stream = stream;
                while conn.is_handshaking() {
                    conn.complete_io(&mut stream).map_err(|e| {
                        NetError::Tls(format!("handshake failed (certificate rejected?): {e}"))
                    })?;
                }
                Ok(ClientConn::Tls(Box::new(rustls::StreamOwned::new(
                    conn, stream,
                ))))
            }
        }
    }
}

// ── tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn self_signed(names: Vec<String>) -> (Vec<u8>, Vec<u8>) {
        let c = rcgen::generate_simple_self_signed(names).unwrap();
        (
            c.cert.pem().into_bytes(),
            c.key_pair.serialize_pem().into_bytes(),
        )
    }

    #[test]
    fn a_valid_certificate_and_key_load() {
        let (cert, key) = self_signed(vec!["localhost".into()]);
        ServerTlsConfig::from_pem(&cert, &key).unwrap();
        ClientTlsConfig::pin_certificate(&cert, "localhost").unwrap();
    }

    /// A mismatched key is rejected at configuration time, not at first connect.
    #[test]
    fn a_key_that_does_not_match_the_certificate_is_rejected() {
        let (cert_a, _) = self_signed(vec!["localhost".into()]);
        let (_, key_b) = self_signed(vec!["localhost".into()]);
        let err = ServerTlsConfig::from_pem(&cert_a, &key_b).unwrap_err();
        assert!(
            err.to_string().contains("do not match") || err.to_string().contains("TLS"),
            "unhelpful error: {err}"
        );
    }

    #[test]
    fn empty_or_garbage_pem_is_rejected() {
        let (cert, key) = self_signed(vec!["localhost".into()]);
        assert!(ServerTlsConfig::from_pem(b"", &key).is_err());
        assert!(ServerTlsConfig::from_pem(&cert, b"").is_err());
        assert!(ServerTlsConfig::from_pem(b"not a pem file", b"nor is this").is_err());
        assert!(ClientTlsConfig::pin_certificate(b"", "localhost").is_err());
        assert!(ClientTlsConfig::pin_certificate(b"garbage", "localhost").is_err());
    }

    /// An invalid server name fails when the config is built, not on first use.
    #[test]
    fn an_invalid_server_name_is_caught_at_configuration_time() {
        let (cert, _) = self_signed(vec!["localhost".into()]);
        assert!(ClientTlsConfig::pin_certificate(&cert, "not a valid name!!").is_err());
        assert!(ClientTlsConfig::pin_certificate(&cert, "").is_err());
        assert!(ClientTlsConfig::pin_certificate(&cert, "example.com").is_ok());
    }

    /// Neither config prints key material.
    #[test]
    fn tls_configs_do_not_leak_key_material_in_debug() {
        let (cert, key) = self_signed(vec!["localhost".into()]);
        let s = ServerTlsConfig::from_pem(&cert, &key).unwrap();
        let printed = format!("{s:?}");
        assert!(!printed.contains("PRIVATE"), "key leaked: {printed}");
        assert!(!printed.contains("BEGIN"), "PEM leaked: {printed}");

        let c = ClientTlsConfig::pin_certificate(&cert, "localhost").unwrap();
        assert_eq!(c.server_name(), "localhost");
    }
}
