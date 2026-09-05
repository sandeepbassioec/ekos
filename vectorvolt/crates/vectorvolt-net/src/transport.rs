//! Transport abstraction for the VectorVolt wire server (VV-NET-XDP-011).
//!
//! The VV wire protocol is framing-independent: a `WireFrameHeader` followed by
//! a payload works equally well over a TCP stream or an XDP socket.  This module
//! defines a [`ByteStream`] trait so the server can be driven by either without
//! knowing which is underneath.
//!
//! ## XDP transport
//!
//! AF_XDP maps each VV request/response to **one UMEM frame** (≤ 2048 bytes by
//! default).  For the wire sizes VectorVolt sends (a 16-byte header + ≤ 1 KB
//! typical payload) this is always sufficient.  Frames larger than
//! `UMEM_FRAME_SIZE` return [`NetError::PayloadTooLarge`].
//!
//! The XDP transport is only compiled on Linux because AF_XDP sockets require
//! kernel ≥ 4.18 and `CAP_NET_RAW`.

use std::io::{Read, Write};

#[cfg(target_os = "linux")]
use crate::NetResult;
#[cfg(target_os = "linux")]
use std::io;

// ── Trait ─────────────────────────────────────────────────────────────────────

/// A bidirectional byte stream used by the server's per-connection handler.
///
/// Implemented by both [`TcpStream`] (via the blanket impl) and
/// [`XdpStream`] (Linux-only).
pub trait ByteStream: Read + Write + Send + 'static {
    /// Human-readable peer address, e.g. `"192.168.1.1:51234"` or `"xdp:lo:q0"`.
    fn peer_label(&self) -> String;
    /// Transport kind tag — used in log lines and metrics labels.
    fn transport_name(&self) -> &'static str;
}

// ── TCP ───────────────────────────────────────────────────────────────────────

impl ByteStream for std::net::TcpStream {
    fn peer_label(&self) -> String {
        self.peer_addr()
            .map(|a| a.to_string())
            .unwrap_or_else(|_| "unknown".to_string())
    }

    fn transport_name(&self) -> &'static str {
        "tcp"
    }
}

// ── TransportKind (config) ────────────────────────────────────────────────────

/// Which transport the server uses for new connections.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum TransportKind {
    /// Standard POSIX TCP sockets (default, all platforms).
    #[default]
    Tcp,
    /// AF_XDP kernel-bypass sockets (Linux ≥ 4.18, requires CAP_NET_RAW).
    ///
    /// The server binds an XDP socket to `iface` queue `queue_id` and handles
    /// VV frames arriving there; no TCP listener is opened.
    #[cfg(target_os = "linux")]
    Xdp {
        /// Network interface name, e.g. `"eth0"` or `"lo"`.
        iface: String,
        /// NIC RX queue index (0 for single-queue NICs and loopback).
        queue_id: u32,
        /// XDP ring size (power of two; default 256).
        ring_size: u32,
    },
}

impl TransportKind {
    pub fn name(&self) -> &'static str {
        match self {
            TransportKind::Tcp => "tcp",
            #[cfg(target_os = "linux")]
            TransportKind::Xdp { .. } => "xdp",
        }
    }

    pub fn is_xdp(&self) -> bool {
        match self {
            TransportKind::Tcp => false,
            #[cfg(target_os = "linux")]
            TransportKind::Xdp { .. } => true,
        }
    }
}

// ── XDP stream (Linux only) ───────────────────────────────────────────────────

/// A [`ByteStream`] backed by an AF_XDP driver.
///
/// Each `write` call submits one XDP TX frame; each `read` call dequeues one
/// XDP RX frame.  Callers must not exceed `UMEM_FRAME_SIZE - headroom` bytes
/// per call.
#[cfg(target_os = "linux")]
pub struct XdpStream {
    driver: vectorvolt_xdp::XdpDriver,
    iface: String,
    queue_id: u32,
    /// Leftover bytes from the last `recv()` call that did not fit into the
    /// caller's buffer.  Real implementations would use a ring-buffer; this
    /// simple version keeps a `Vec` scratch pad.
    read_buf: Vec<u8>,
    read_pos: usize,
}

#[cfg(target_os = "linux")]
impl XdpStream {
    /// Bind an XDP socket to `iface` queue `queue_id`.
    ///
    /// Requires `CAP_NET_RAW`.
    pub fn bind(iface: &str, queue_id: u32, ring_size: u32) -> NetResult<Self> {
        use vectorvolt_xdp::umem::{Umem, UmemConfig};
        let umem = Umem::new(UmemConfig::default()).map_err(|e| NetError::Engine(e.to_string()))?;
        let driver = vectorvolt_xdp::XdpDriver::bind(iface, ring_size, umem)
            .map_err(|e| NetError::Engine(e.to_string()))?;
        Ok(Self {
            driver,
            iface: iface.to_string(),
            queue_id,
            read_buf: Vec::new(),
            read_pos: 0,
        })
    }
}

#[cfg(target_os = "linux")]
impl Read for XdpStream {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if self.read_pos < self.read_buf.len() {
            let available = &self.read_buf[self.read_pos..];
            let n = buf.len().min(available.len());
            buf[..n].copy_from_slice(&available[..n]);
            self.read_pos += n;
            return Ok(n);
        }
        // Block until a frame arrives (spin — real impl would use poll).
        loop {
            match self.driver.recv() {
                Ok((_slot, data)) => {
                    let n = buf.len().min(data.len());
                    buf[..n].copy_from_slice(&data[..n]);
                    if data.len() > n {
                        self.read_buf = data[n..].to_vec();
                        self.read_pos = 0;
                    } else {
                        self.read_buf.clear();
                        self.read_pos = 0;
                    }
                    return Ok(n);
                }
                Err(vectorvolt_xdp::XdpError::RingEmpty) => {
                    std::hint::spin_loop();
                }
                Err(e) => {
                    return Err(io::Error::new(io::ErrorKind::Other, e.to_string()));
                }
            }
        }
    }
}

#[cfg(target_os = "linux")]
impl Write for XdpStream {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        const FRAME_IDX: u32 = 0;
        self.driver
            .send(FRAME_IDX, buf)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

#[cfg(target_os = "linux")]
impl ByteStream for XdpStream {
    fn peer_label(&self) -> String {
        format!("xdp:{}:q{}", self.iface, self.queue_id)
    }

    fn transport_name(&self) -> &'static str {
        "xdp"
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transport_kind_default_is_tcp() {
        assert_eq!(TransportKind::default(), TransportKind::Tcp);
    }

    #[test]
    fn transport_kind_tcp_name() {
        assert_eq!(TransportKind::Tcp.name(), "tcp");
    }

    #[test]
    fn transport_kind_tcp_is_not_xdp() {
        assert!(!TransportKind::Tcp.is_xdp());
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn transport_kind_xdp_name() {
        let k = TransportKind::Xdp {
            iface: "lo".to_string(),
            queue_id: 0,
            ring_size: 256,
        };
        assert_eq!(k.name(), "xdp");
        assert!(k.is_xdp());
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn transport_kind_xdp_clone() {
        let k = TransportKind::Xdp {
            iface: "eth0".to_string(),
            queue_id: 1,
            ring_size: 512,
        };
        let k2 = k.clone();
        assert_eq!(k, k2);
    }

    #[test]
    fn tcp_stream_byte_stream_impl_compile() {
        fn assert_byte_stream<T: ByteStream>() {}
        assert_byte_stream::<std::net::TcpStream>();
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn xdp_stream_byte_stream_impl_compile() {
        fn assert_byte_stream<T: ByteStream>() {}
        assert_byte_stream::<XdpStream>();
    }

    #[test]
    fn transport_kind_debug() {
        let s = format!("{:?}", TransportKind::Tcp);
        assert!(s.contains("Tcp"));
    }

    #[test]
    fn transport_kind_eq() {
        assert_eq!(TransportKind::Tcp, TransportKind::Tcp);
    }

    #[cfg(target_os = "linux")]
    #[test]
    #[ignore = "requires CAP_NET_RAW + loopback"]
    fn xdp_stream_bind_loopback() {
        let stream = XdpStream::bind("lo", 0, 64).unwrap();
        assert_eq!(stream.peer_label(), "xdp:lo:q0");
        assert_eq!(stream.transport_name(), "xdp");
    }

    #[test]
    fn tcp_transport_name_is_tcp() {
        use std::net::{TcpListener, TcpStream};
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        let stream = TcpStream::connect(addr).unwrap();
        assert_eq!(stream.transport_name(), "tcp");
    }

    #[test]
    fn tcp_peer_label_non_empty() {
        use std::net::{TcpListener, TcpStream};
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        let stream = TcpStream::connect(addr).unwrap();
        assert!(!stream.peer_label().is_empty());
    }

    #[test]
    fn transport_kind_clone_tcp() {
        let k = TransportKind::Tcp;
        assert_eq!(k.clone(), TransportKind::Tcp);
    }
}
