//! Cluster network transport for Raft RPC (gap F1).
//!
//! * Framing: `[version u8][len u32 LE][crc32 u32 LE][binary payload]` (compact
//!   varint encoding, see `crate::raft::codec`); frames with a wrong version,
//!   oversize length or bad CRC are rejected and the connection is dropped. The
//!   codec is generic over `AsyncRead`/`AsyncWrite`.
//! * TLS: `TlsConfig` layers tokio-rustls over the same codec with mutual
//!   authentication (CA + node cert/key); `TcpTransport::bind` stays plaintext
//!   for tests, `TcpTransport::bind_tls` is the production entry point.
//! * [`TcpTransport`]: tokio TCP, one lazily-created writer task per peer with
//!   connect timeout, write timeout and exponential reconnect backoff.
//! * [`MemNetwork`]: in-memory network with partitions for tests.
//!
//! Delivery is best-effort and unordered across reconnects, which is all Raft
//! requires; lost messages are recovered by heartbeats/retries.

use crate::node::NodeId;
use crate::raft::codec::{CodecError, Reader, Wire};
use crate::raft::rpc::Rpc;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::io;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex as StdMutex};
use std::time::Duration;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{Mutex, mpsc};
use tokio::task::JoinHandle;

/// Wire version 2 = binary payloads (version 1 was JSON and is rejected).
pub const FRAME_VERSION: u8 = 2;
pub const MAX_FRAME_LEN: usize = 64 * 1024 * 1024;

#[derive(Debug, thiserror::Error)]
pub enum TransportError {
    #[error("unknown peer {0:?}")]
    UnknownPeer(NodeId),
    #[error("send queue full or peer unreachable")]
    Unavailable,
    #[error("io: {0}")]
    Io(#[from] io::Error),
    #[error("frame error: {0}")]
    Frame(String),
}

/// A message plus its sender.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Envelope {
    pub from: NodeId,
    pub msg: Rpc,
}

impl Wire for Envelope {
    fn to_wire(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(48);
        out.extend_from_slice(self.from.0.as_bytes());
        out.extend(self.msg.to_wire());
        out
    }

    fn from_wire(buf: &[u8]) -> Result<Self, CodecError> {
        let from = Reader::new(buf).node()?;
        let msg = Rpc::from_wire(&buf[16..])?;
        Ok(Envelope { from, msg })
    }
}

#[async_trait]
pub trait RaftTransport: Send + Sync + 'static {
    fn local_id(&self) -> &NodeId;
    /// Queue `msg` for `to`. Must not block on the network.
    async fn send(&self, to: &NodeId, msg: Rpc) -> Result<(), TransportError>;
    /// Next inbound message; `None` when the transport is closed.
    async fn recv(&self) -> Option<Envelope>;
}

// ---- framing ---------------------------------------------------------------

pub fn encode_frame<T: Wire>(value: &T) -> Result<Vec<u8>, TransportError> {
    let payload = value.to_wire();
    if payload.len() > MAX_FRAME_LEN {
        return Err(TransportError::Frame("frame too large".into()));
    }
    let mut out = Vec::with_capacity(9 + payload.len());
    out.push(FRAME_VERSION);
    out.extend_from_slice(&(payload.len() as u32).to_le_bytes());
    out.extend_from_slice(&crc32fast::hash(&payload).to_le_bytes());
    out.extend_from_slice(&payload);
    Ok(out)
}

pub async fn write_frame<W: AsyncWrite + Unpin, T: Wire>(
    w: &mut W,
    value: &T,
) -> Result<(), TransportError> {
    let buf = encode_frame(value)?;
    w.write_all(&buf).await?;
    w.flush().await?;
    Ok(())
}

/// Read one frame. `Ok(None)` on clean EOF at a frame boundary.
pub async fn read_frame<R: AsyncRead + Unpin, T: Wire>(
    r: &mut R,
) -> Result<Option<T>, TransportError> {
    let mut hdr = [0u8; 9];
    match r.read_exact(&mut hdr).await {
        Ok(_) => {}
        Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => return Ok(None),
        Err(e) => return Err(e.into()),
    }
    if hdr[0] != FRAME_VERSION {
        return Err(TransportError::Frame(format!(
            "unsupported version {}",
            hdr[0]
        )));
    }
    let len = u32::from_le_bytes(hdr[1..5].try_into().expect("4")) as usize;
    let want = u32::from_le_bytes(hdr[5..9].try_into().expect("4"));
    if len > MAX_FRAME_LEN {
        return Err(TransportError::Frame(format!(
            "frame length {len} too large"
        )));
    }
    let mut payload = vec![0u8; len];
    r.read_exact(&mut payload).await?;
    if crc32fast::hash(&payload) != want {
        return Err(TransportError::Frame("crc mismatch".into()));
    }
    T::from_wire(&payload)
        .map(Some)
        .map_err(|e| TransportError::Frame(e.to_string()))
}

// ---- TCP -------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct TcpConfig {
    pub connect_timeout: Duration,
    pub write_timeout: Duration,
    pub backoff_initial: Duration,
    pub backoff_max: Duration,
    /// Per-peer outbound queue length; overflow drops (Raft retries).
    pub queue_cap: usize,
}

impl Default for TcpConfig {
    fn default() -> Self {
        TcpConfig {
            connect_timeout: Duration::from_secs(1),
            write_timeout: Duration::from_secs(2),
            backoff_initial: Duration::from_millis(20),
            backoff_max: Duration::from_secs(2),
            queue_cap: 1024,
        }
    }
}

/// Mutual-TLS settings for the Raft transport.
///
/// Every node presents a certificate issued by the cluster CA and requires the
/// same of its peers (mTLS). Server certificates are verified against the
/// peer's IP address unless `server_name` is set, in which case that DNS name
/// must be in every node certificate's SANs.
///
/// Note: mTLS proves a peer holds *a* cluster certificate; it does not bind a
/// certificate to a particular `NodeId`.
#[derive(Clone)]
pub struct TlsConfig {
    server: Arc<rustls::ServerConfig>,
    client: Arc<rustls::ClientConfig>,
    server_name: Option<String>,
}

impl std::fmt::Debug for TlsConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // No contents: the config holds the private key.
        f.write_str("TlsConfig(<configured>)")
    }
}

fn tls_err(what: &str, e: impl std::fmt::Display) -> TransportError {
    TransportError::Io(io::Error::new(
        io::ErrorKind::InvalidInput,
        format!("tls: {what}: {e}"),
    ))
}

impl TlsConfig {
    /// Build from PEM bytes: the cluster `ca_pem`, this node's `cert_pem`
    /// chain (leaf first) and its private `key_pem`.
    pub fn from_pem(
        ca_pem: &[u8],
        cert_pem: &[u8],
        key_pem: &[u8],
        server_name: Option<&str>,
    ) -> Result<Self, TransportError> {
        let provider = Arc::new(rustls::crypto::ring::default_provider());
        let certs: Vec<rustls::pki_types::CertificateDer<'static>> =
            rustls_pemfile::certs(&mut &cert_pem[..])
                .collect::<Result<_, _>>()
                .map_err(|e| tls_err("reading node certificate", e))?;
        if certs.is_empty() {
            return Err(tls_err("node certificate", "no certificates in PEM"));
        }
        let key = || {
            rustls_pemfile::private_key(&mut &key_pem[..])
                .map_err(|e| tls_err("reading private key", e))?
                .ok_or_else(|| tls_err("private key", "no key in PEM"))
        };
        let mut roots = rustls::RootCertStore::empty();
        for c in rustls_pemfile::certs(&mut &ca_pem[..]) {
            let c = c.map_err(|e| tls_err("reading CA", e))?;
            roots.add(c).map_err(|e| tls_err("adding CA", e))?;
        }
        if roots.is_empty() {
            return Err(tls_err("CA", "no certificates in PEM"));
        }
        let roots = Arc::new(roots);

        let verifier = rustls::server::WebPkiClientVerifier::builder_with_provider(
            roots.clone(),
            provider.clone(),
        )
        .build()
        .map_err(|e| tls_err("client verifier", e))?;
        let server = rustls::ServerConfig::builder_with_provider(provider.clone())
            .with_safe_default_protocol_versions()
            .map_err(|e| tls_err("protocol versions", e))?
            .with_client_cert_verifier(verifier)
            .with_single_cert(certs.clone(), key()?)
            .map_err(|e| tls_err("server certificate/key", e))?;
        let client = rustls::ClientConfig::builder_with_provider(provider)
            .with_safe_default_protocol_versions()
            .map_err(|e| tls_err("protocol versions", e))?
            .with_root_certificates(roots)
            .with_client_auth_cert(certs, key()?)
            .map_err(|e| tls_err("client certificate/key", e))?;
        Ok(TlsConfig {
            server: Arc::new(server),
            client: Arc::new(client),
            server_name: server_name.map(str::to_owned),
        })
    }

    /// Load the CA, node certificate and key from PEM files.
    pub fn from_files(
        ca: impl AsRef<std::path::Path>,
        cert: impl AsRef<std::path::Path>,
        key: impl AsRef<std::path::Path>,
        server_name: Option<&str>,
    ) -> Result<Self, TransportError> {
        Self::from_pem(
            &std::fs::read(ca)?,
            &std::fs::read(cert)?,
            &std::fs::read(key)?,
            server_name,
        )
    }

    fn name_for(
        &self,
        addr: SocketAddr,
    ) -> Result<rustls::pki_types::ServerName<'static>, TransportError> {
        match &self.server_name {
            Some(n) => rustls::pki_types::ServerName::try_from(n.clone())
                .map_err(|e| tls_err("server name", e)),
            None => Ok(rustls::pki_types::ServerName::IpAddress(addr.ip().into())),
        }
    }
}

pub struct TcpTransport {
    id: NodeId,
    local_addr: SocketAddr,
    cfg: TcpConfig,
    tls: Option<TlsConfig>,
    peers: StdMutex<HashMap<NodeId, SocketAddr>>,
    writers: StdMutex<HashMap<NodeId, mpsc::Sender<Envelope>>>,
    inbox: Mutex<mpsc::Receiver<Envelope>>,
    tasks: StdMutex<Vec<JoinHandle<()>>>,
}

/// Read frames until EOF or the first bad frame, forwarding envelopes.
async fn pump<S: AsyncRead + Unpin>(mut stream: S, tx: mpsc::Sender<Envelope>) {
    // Any frame/CRC/version error drops the connection.
    while let Ok(Some(env)) = read_frame::<_, Envelope>(&mut stream).await {
        if tx.send(env).await.is_err() {
            return;
        }
    }
}

const TLS_HANDSHAKE_TIMEOUT: Duration = Duration::from_secs(5);

impl TcpTransport {
    /// Bind a plaintext listener on `addr` (use port 0 for an ephemeral port).
    /// Intended for tests and trusted loopback use; see [`Self::bind_tls`].
    pub async fn bind(
        id: NodeId,
        addr: SocketAddr,
        peers: HashMap<NodeId, SocketAddr>,
        cfg: TcpConfig,
    ) -> io::Result<Self> {
        Self::bind_inner(id, addr, peers, cfg, None).await
    }

    /// Bind a listener that requires mutual TLS from every peer and uses TLS
    /// for all outbound connections.
    pub async fn bind_tls(
        id: NodeId,
        addr: SocketAddr,
        peers: HashMap<NodeId, SocketAddr>,
        cfg: TcpConfig,
        tls: TlsConfig,
    ) -> io::Result<Self> {
        Self::bind_inner(id, addr, peers, cfg, Some(tls)).await
    }

    async fn bind_inner(
        id: NodeId,
        addr: SocketAddr,
        peers: HashMap<NodeId, SocketAddr>,
        cfg: TcpConfig,
        tls: Option<TlsConfig>,
    ) -> io::Result<Self> {
        let listener = TcpListener::bind(addr).await?;
        let local_addr = listener.local_addr()?;
        let (tx, rx) = mpsc::channel(4096);
        let acceptor = tls
            .as_ref()
            .map(|t| tokio_rustls::TlsAcceptor::from(t.server.clone()));
        let accept = tokio::spawn(async move {
            loop {
                let Ok((stream, _)) = listener.accept().await else {
                    continue;
                };
                let _ = stream.set_nodelay(true);
                let tx = tx.clone();
                let acceptor = acceptor.clone();
                tokio::spawn(async move {
                    match acceptor {
                        None => pump(stream, tx).await,
                        Some(a) => {
                            // Handshake failures (no/untrusted client cert, plaintext
                            // peer, ...) just drop the connection.
                            if let Ok(Ok(s)) =
                                tokio::time::timeout(TLS_HANDSHAKE_TIMEOUT, a.accept(stream)).await
                            {
                                pump(s, tx).await;
                            }
                        }
                    }
                });
            }
        });
        Ok(TcpTransport {
            id,
            local_addr,
            cfg,
            tls,
            peers: StdMutex::new(peers),
            writers: StdMutex::new(HashMap::new()),
            inbox: Mutex::new(rx),
            tasks: StdMutex::new(vec![accept]),
        })
    }

    pub fn local_addr(&self) -> SocketAddr {
        self.local_addr
    }

    /// Add or update a peer address (takes effect for new connections).
    pub fn set_peer(&self, id: NodeId, addr: SocketAddr) {
        self.peers.lock().expect("peers").insert(id.clone(), addr);
        self.writers.lock().expect("writers").remove(&id);
    }

    /// Forget a peer (e.g. after it was removed from the cluster).
    pub fn remove_peer(&self, id: &NodeId) {
        self.peers.lock().expect("peers").remove(id);
        self.writers.lock().expect("writers").remove(id);
    }

    fn writer_for(&self, to: &NodeId) -> Result<mpsc::Sender<Envelope>, TransportError> {
        let mut writers = self.writers.lock().expect("writers");
        if let Some(w) = writers.get(to).filter(|w| !w.is_closed()) {
            return Ok(w.clone());
        }
        let addr = *self
            .peers
            .lock()
            .expect("peers")
            .get(to)
            .ok_or_else(|| TransportError::UnknownPeer(to.clone()))?;
        let (tx, rx) = mpsc::channel(self.cfg.queue_cap);
        let task = tokio::spawn(peer_writer(addr, rx, self.cfg.clone(), self.tls.clone()));
        self.tasks.lock().expect("tasks").push(task);
        writers.insert(to.clone(), tx.clone());
        Ok(tx)
    }
}

trait Conn: AsyncWrite + Unpin + Send {}
impl<T: AsyncWrite + Unpin + Send> Conn for T {}

async fn connect(
    addr: SocketAddr,
    cfg: &TcpConfig,
    tls: &Option<TlsConfig>,
) -> Option<Box<dyn Conn>> {
    let fut = async {
        let s = TcpStream::connect(addr).await.ok()?;
        let _ = s.set_nodelay(true);
        match tls {
            None => Some(Box::new(s) as Box<dyn Conn>),
            Some(t) => {
                let name = t.name_for(addr).ok()?;
                let c = tokio_rustls::TlsConnector::from(t.client.clone());
                let s = c.connect(name, s).await.ok()?;
                Some(Box::new(s) as Box<dyn Conn>)
            }
        }
    };
    tokio::time::timeout(cfg.connect_timeout.max(TLS_HANDSHAKE_TIMEOUT), fut)
        .await
        .ok()
        .flatten()
}

async fn peer_writer(
    addr: SocketAddr,
    mut rx: mpsc::Receiver<Envelope>,
    cfg: TcpConfig,
    tls: Option<TlsConfig>,
) {
    let mut conn: Option<Box<dyn Conn>> = None;
    let mut backoff = cfg.backoff_initial;
    while let Some(env) = rx.recv().await {
        if conn.is_none() {
            match connect(addr, &cfg, &tls).await {
                Some(s) => {
                    conn = Some(s);
                    backoff = cfg.backoff_initial;
                }
                None => {
                    // Unreachable: drop this and any queued (stale) messages, back off.
                    tokio::time::sleep(backoff).await;
                    backoff = (backoff * 2).min(cfg.backoff_max);
                    while rx.try_recv().is_ok() {}
                    continue;
                }
            }
        }
        let stream = conn.as_mut().expect("connected");
        let res = tokio::time::timeout(cfg.write_timeout, write_frame(stream, &env)).await;
        if !matches!(res, Ok(Ok(()))) {
            conn = None; // reconnect on next message
        }
    }
}

impl Drop for TcpTransport {
    fn drop(&mut self) {
        for t in self.tasks.lock().expect("tasks").drain(..) {
            t.abort();
        }
    }
}

#[async_trait]
impl RaftTransport for TcpTransport {
    fn local_id(&self) -> &NodeId {
        &self.id
    }

    async fn send(&self, to: &NodeId, msg: Rpc) -> Result<(), TransportError> {
        let w = self.writer_for(to)?;
        w.try_send(Envelope {
            from: self.id.clone(),
            msg,
        })
        .map_err(|_| TransportError::Unavailable)
    }

    async fn recv(&self) -> Option<Envelope> {
        self.inbox.lock().await.recv().await
    }
}

// ---- in-memory -------------------------------------------------------------

#[derive(Default)]
struct MemInner {
    inboxes: HashMap<NodeId, mpsc::UnboundedSender<Envelope>>,
    /// Partition group per node; messages flow only within equal groups.
    group: HashMap<NodeId, u32>,
    down: HashSet<NodeId>,
}

/// Shared in-memory network. Cheap to clone.
#[derive(Clone, Default)]
pub struct MemNetwork {
    inner: Arc<StdMutex<MemInner>>,
}

impl MemNetwork {
    pub fn new() -> Self {
        Self::default()
    }

    /// Register (or re-register after a restart) a node and get its transport.
    pub fn transport(&self, id: NodeId) -> MemTransport {
        let (tx, rx) = mpsc::unbounded_channel();
        let mut g = self.inner.lock().expect("net");
        g.inboxes.insert(id.clone(), tx);
        g.down.remove(&id);
        MemTransport {
            id,
            net: self.clone(),
            rx: Mutex::new(rx),
        }
    }

    /// Mark a node down: its traffic (in and out) is dropped.
    pub fn set_down(&self, id: &NodeId, down: bool) {
        let mut g = self.inner.lock().expect("net");
        if down {
            g.down.insert(id.clone());
        } else {
            g.down.remove(id);
        }
    }

    /// Split the network: nodes in different groups cannot communicate.
    /// Unlisted nodes are in group 0.
    pub fn partition(&self, groups: &[&[NodeId]]) {
        let mut g = self.inner.lock().expect("net");
        g.group.clear();
        for (i, grp) in groups.iter().enumerate() {
            for n in *grp {
                g.group.insert(n.clone(), i as u32 + 1);
            }
        }
    }

    pub fn heal(&self) {
        self.inner.lock().expect("net").group.clear();
    }

    fn deliver(&self, env: Envelope, to: &NodeId) -> Result<(), TransportError> {
        let g = self.inner.lock().expect("net");
        if g.down.contains(to) || g.down.contains(&env.from) {
            return Ok(());
        }
        let gf = g.group.get(&env.from).copied().unwrap_or(0);
        let gt = g.group.get(to).copied().unwrap_or(0);
        if gf != gt {
            return Ok(()); // partitioned: silently dropped like a real network
        }
        match g.inboxes.get(to) {
            Some(tx) => {
                let _ = tx.send(env);
                Ok(())
            }
            None => Err(TransportError::UnknownPeer(to.clone())),
        }
    }
}

pub struct MemTransport {
    id: NodeId,
    net: MemNetwork,
    rx: Mutex<mpsc::UnboundedReceiver<Envelope>>,
}

#[async_trait]
impl RaftTransport for MemTransport {
    fn local_id(&self) -> &NodeId {
        &self.id
    }

    async fn send(&self, to: &NodeId, msg: Rpc) -> Result<(), TransportError> {
        self.net.deliver(
            Envelope {
                from: self.id.clone(),
                msg,
            },
            to,
        )
    }

    async fn recv(&self) -> Option<Envelope> {
        self.rx.lock().await.recv().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rpc() -> Rpc {
        Rpc::RequestVote {
            term: 3,
            last_log_index: 4,
            last_log_term: 2,
        }
    }

    #[tokio::test]
    async fn frame_roundtrip() {
        let buf = encode_frame(&rpc()).unwrap();
        let mut r = &buf[..];
        let got: Rpc = read_frame(&mut r).await.unwrap().unwrap();
        assert_eq!(got, rpc());
        assert!(read_frame::<_, Rpc>(&mut r).await.unwrap().is_none());
    }

    #[tokio::test]
    async fn frame_rejects_corruption_and_bad_version() {
        let mut buf = encode_frame(&rpc()).unwrap();
        let last = buf.len() - 1;
        buf[last] ^= 0xff;
        assert!(read_frame::<_, Rpc>(&mut &buf[..]).await.is_err());
        let mut buf = encode_frame(&rpc()).unwrap();
        buf[0] = 9;
        assert!(read_frame::<_, Rpc>(&mut &buf[..]).await.is_err());
        let mut buf = encode_frame(&rpc()).unwrap();
        buf[1..5].copy_from_slice(&u32::MAX.to_le_bytes());
        assert!(read_frame::<_, Rpc>(&mut &buf[..]).await.is_err());
    }

    #[tokio::test]
    async fn mem_partition_drops() {
        let net = MemNetwork::new();
        let (a, b) = (NodeId::new(), NodeId::new());
        let ta = net.transport(a.clone());
        let tb = net.transport(b.clone());
        net.partition(&[std::slice::from_ref(&a), std::slice::from_ref(&b)]);
        ta.send(&b, rpc()).await.unwrap();
        net.heal();
        ta.send(&b, rpc()).await.unwrap();
        assert_eq!(tb.recv().await.unwrap().from, a);
    }

    #[tokio::test]
    async fn tcp_send_recv_and_reconnect() {
        let (a, b) = (NodeId::new(), NodeId::new());
        let lo: SocketAddr = "127.0.0.1:0".parse().unwrap();
        let tb = TcpTransport::bind(b.clone(), lo, HashMap::new(), TcpConfig::default())
            .await
            .unwrap();
        let baddr = tb.local_addr();
        let ta = TcpTransport::bind(
            a.clone(),
            lo,
            HashMap::from([(b.clone(), baddr)]),
            TcpConfig::default(),
        )
        .await
        .unwrap();
        ta.send(&b, rpc()).await.unwrap();
        let env = tokio::time::timeout(Duration::from_secs(3), tb.recv())
            .await
            .unwrap()
            .unwrap();
        assert_eq!(env.from, a);
        assert_eq!(env.msg, rpc());
        assert!(matches!(
            ta.send(&NodeId::new(), rpc()).await,
            Err(TransportError::UnknownPeer(_))
        ));
    }
}
