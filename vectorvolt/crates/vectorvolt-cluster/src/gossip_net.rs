//! Wire layer for SWIM gossip: message envelope, a real UDP transport, and
//! an in-memory network used by deterministic tests.

use crate::node::{NodeId, NodeMeta};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use tokio::net::UdpSocket;
use tokio::sync::{Mutex as AsyncMutex, mpsc};

/// SWIM message kind.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Kind {
    /// Direct probe.
    Ping,
    /// Reply to Ping / Join / relayed PingReq (carries the original `seq`).
    Ack,
    /// Ask the receiver to probe `target` on the sender's behalf.
    PingReq {
        target: NodeId,
        target_addr: SocketAddr,
    },
    /// Join request; answered by an Ack carrying a full membership snapshot.
    Join,
}

/// One datagram: sender identity + piggy-backed membership updates.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Envelope {
    pub seq: u64,
    pub kind: Kind,
    pub from: NodeMeta,
    pub updates: Vec<NodeMeta>,
}

/// Datagram transport abstraction (UDP in production, in-memory in tests).
#[async_trait]
pub trait Transport: Send + Sync {
    fn local_addr(&self) -> SocketAddr;
    async fn send(&self, to: SocketAddr, env: &Envelope);
    /// Next inbound datagram, or `None` when the transport is closed.
    async fn recv(&self) -> Option<(SocketAddr, Envelope)>;
}

/// Real UDP transport (JSON-encoded datagrams).
pub struct UdpTransport {
    socket: UdpSocket,
    local: SocketAddr,
}

impl UdpTransport {
    pub async fn bind(addr: SocketAddr) -> std::io::Result<Arc<Self>> {
        let socket = UdpSocket::bind(addr).await?;
        let local = socket.local_addr()?;
        Ok(Arc::new(Self { socket, local }))
    }
}

#[async_trait]
impl Transport for UdpTransport {
    fn local_addr(&self) -> SocketAddr {
        self.local
    }

    async fn send(&self, to: SocketAddr, env: &Envelope) {
        if let Ok(bytes) = serde_json::to_vec(env) {
            let _ = self.socket.send_to(&bytes, to).await;
        }
    }

    async fn recv(&self) -> Option<(SocketAddr, Envelope)> {
        let mut buf = vec![0u8; 65_535];
        loop {
            match self.socket.recv_from(&mut buf).await {
                Ok((n, src)) => {
                    if let Ok(env) = serde_json::from_slice::<Envelope>(&buf[..n]) {
                        return Some((src, env));
                    }
                    // malformed datagram: drop and keep listening
                }
                // Windows reports ICMP port-unreachable as a recv error; ignore.
                Err(e) if e.kind() == std::io::ErrorKind::ConnectionReset => {}
                Err(_) => return None,
            }
        }
    }
}

type Inbox = mpsc::UnboundedSender<(SocketAddr, Envelope)>;

/// In-memory datagram network with fault injection (node down, link cut).
#[derive(Default)]
pub struct MemNetwork {
    inboxes: Mutex<std::collections::HashMap<SocketAddr, Inbox>>,
    down: Mutex<HashSet<SocketAddr>>,
    cut: Mutex<HashSet<(SocketAddr, SocketAddr)>>,
}

impl MemNetwork {
    pub fn new() -> Arc<Self> {
        Arc::new(Self::default())
    }

    /// Attach an endpoint at `addr`.
    pub fn endpoint(self: &Arc<Self>, addr: SocketAddr) -> Arc<MemTransport> {
        let (tx, rx) = mpsc::unbounded_channel();
        self.inboxes.lock().unwrap().insert(addr, tx);
        Arc::new(MemTransport {
            net: self.clone(),
            addr,
            rx: AsyncMutex::new(rx),
        })
    }

    /// Take a node off the network (drops all its inbound and outbound traffic).
    pub fn set_down(&self, addr: SocketAddr, down: bool) {
        let mut d = self.down.lock().unwrap();
        if down {
            d.insert(addr);
        } else {
            d.remove(&addr);
        }
    }

    /// Cut or restore the link between two nodes (both directions).
    pub fn set_cut(&self, a: SocketAddr, b: SocketAddr, cut: bool) {
        let mut c = self.cut.lock().unwrap();
        if cut {
            c.insert((a, b));
            c.insert((b, a));
        } else {
            c.remove(&(a, b));
            c.remove(&(b, a));
        }
    }

    fn deliver(&self, from: SocketAddr, to: SocketAddr, env: Envelope) {
        {
            let d = self.down.lock().unwrap();
            if d.contains(&from) || d.contains(&to) {
                return;
            }
        }
        if self.cut.lock().unwrap().contains(&(from, to)) {
            return;
        }
        if let Some(tx) = self.inboxes.lock().unwrap().get(&to) {
            let _ = tx.send((from, env));
        }
    }
}

pub struct MemTransport {
    net: Arc<MemNetwork>,
    addr: SocketAddr,
    rx: AsyncMutex<mpsc::UnboundedReceiver<(SocketAddr, Envelope)>>,
}

#[async_trait]
impl Transport for MemTransport {
    fn local_addr(&self) -> SocketAddr {
        self.addr
    }

    async fn send(&self, to: SocketAddr, env: &Envelope) {
        self.net.deliver(self.addr, to, env.clone());
    }

    async fn recv(&self) -> Option<(SocketAddr, Envelope)> {
        self.rx.lock().await.recv().await
    }
}
