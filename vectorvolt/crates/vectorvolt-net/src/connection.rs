//! Async TCP connection over the VectorVolt wire protocol (VV-NET-002).
//!
//! [`Connection`] wraps a [`tokio::net::TcpStream`] and provides
//! `send_frame` / `recv_frame` using the 64-byte [`WireFrameHeader`].
//! `ping` verifies round-trip liveness and returns the RTT.

use std::time::Duration;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::time::timeout;

use crate::frame::{Opcode, WIRE_HEADER_SIZE, WireFrameHeader};
use crate::{NetError, NetResult};

/// A single async TCP connection to a VectorVolt server.
///
/// Not `Clone` — each `Connection` owns one TCP stream.  Use
/// [`crate::pool::ConnectionPool`] when multiple concurrent connections are
/// needed.
#[derive(Debug)]
pub struct Connection {
    pub(crate) stream: TcpStream,
    pub(crate) next_id: u64,
}

impl Connection {
    /// Connect to `addr`.
    pub async fn connect(addr: impl tokio::net::ToSocketAddrs) -> NetResult<Self> {
        let stream = TcpStream::connect(addr).await?;
        // Disable Nagle: request/response protocols have nothing to coalesce.
        stream.set_nodelay(true)?;
        Ok(Self { stream, next_id: 1 })
    }

    /// Connect with a deadline.
    pub async fn connect_timeout(
        addr: impl tokio::net::ToSocketAddrs,
        dur: Duration,
    ) -> NetResult<Self> {
        match timeout(dur, TcpStream::connect(addr)).await {
            Ok(Ok(stream)) => {
                stream.set_nodelay(true)?;
                Ok(Self { stream, next_id: 1 })
            }
            Ok(Err(e)) => Err(NetError::Io(e)),
            Err(_) => Err(NetError::Io(std::io::Error::new(
                std::io::ErrorKind::TimedOut,
                "connection timed out",
            ))),
        }
    }

    /// Send a frame: writes the 64-byte header followed by `payload`.
    ///
    /// Returns the `request_id` allocated for this frame.
    pub async fn send_frame(&mut self, opcode: Opcode, payload: &[u8]) -> NetResult<u64> {
        let id = self.next_id;
        self.next_id = self.next_id.wrapping_add(1);
        let header = WireFrameHeader::new(opcode, payload, id)?;
        let wire = header.encode_with_payload(payload);
        self.stream.write_all(&wire).await?;
        Ok(id)
    }

    /// Receive one frame: reads header + payload, validates CRC.
    ///
    /// Returns `(header, payload_bytes)` on success.
    pub async fn recv_frame(&mut self) -> NetResult<(WireFrameHeader, Vec<u8>)> {
        // Read header bytes.
        let mut hbuf = vec![0u8; WIRE_HEADER_SIZE];
        self.stream.read_exact(&mut hbuf).await?;

        // Peek at payload_len from raw bytes to know how much more to read.
        let payload_len = u32::from_le_bytes(hbuf[8..12].try_into().unwrap()) as usize;

        // Read payload.
        let mut payload = vec![0u8; payload_len];
        if payload_len > 0 {
            self.stream.read_exact(&mut payload).await?;
        }

        // Decode and validate header + CRC.
        let header = WireFrameHeader::decode(&hbuf)?;
        header.verify_payload(&payload)?;

        Ok((header, payload))
    }

    /// Send a `Ping` and wait for a `PingAck`, returning the round-trip time.
    pub async fn ping(&mut self) -> NetResult<Duration> {
        let t0 = std::time::Instant::now();
        self.send_frame(Opcode::Ping, b"").await?;
        let (header, _) = self.recv_frame().await?;
        if header.opcode != Opcode::PingAck as u8 {
            return Err(NetError::Malformed("expected PingAck"));
        }
        Ok(t0.elapsed())
    }

    /// Local address of the underlying TCP stream.
    pub fn local_addr(&self) -> std::io::Result<std::net::SocketAddr> {
        self.stream.local_addr()
    }

    /// Remote address of the underlying TCP stream.
    pub fn peer_addr(&self) -> std::io::Result<std::net::SocketAddr> {
        self.stream.peer_addr()
    }
}

// ── tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use tokio::net::TcpListener;

    use super::*;
    use crate::frame::Opcode;

    /// Echo server: receives one frame, echoes it back as PingAck.
    async fn echo_server(listener: TcpListener) {
        let (stream, _) = listener.accept().await.unwrap();
        let mut conn = Connection { stream, next_id: 1 };
        let (_, payload) = conn.recv_frame().await.unwrap();
        conn.send_frame(Opcode::PingAck, &payload).await.unwrap();
    }

    async fn listen() -> (TcpListener, std::net::SocketAddr) {
        let l = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = l.local_addr().unwrap();
        (l, addr)
    }

    #[tokio::test]
    async fn connect_and_ping() {
        let (listener, addr) = listen().await;
        let srv = tokio::spawn(async move {
            let (stream, _) = listener.accept().await.unwrap();
            let mut conn = Connection { stream, next_id: 1 };
            let (_, _) = conn.recv_frame().await.unwrap();
            conn.send_frame(Opcode::PingAck, b"").await.unwrap();
        });
        let mut client = Connection::connect(addr).await.unwrap();
        let rtt = client.ping().await.unwrap();
        assert!(rtt.as_secs() < 5);
        srv.await.unwrap();
    }

    #[tokio::test]
    async fn send_recv_execute_frame() {
        let (listener, addr) = listen().await;
        let srv = tokio::spawn(echo_server(listener));
        let mut client = Connection::connect(addr).await.unwrap();
        let req_id = client
            .send_frame(Opcode::Execute, b"SELECT 1")
            .await
            .unwrap();
        let (header, payload) = client.recv_frame().await.unwrap();
        assert_eq!(header.request_id, req_id);
        assert_eq!(&payload, b"SELECT 1");
        srv.await.unwrap();
    }

    #[tokio::test]
    async fn send_recv_query_frame() {
        let (listener, addr) = listen().await;
        let srv = tokio::spawn(echo_server(listener));
        let mut client = Connection::connect(addr).await.unwrap();
        client.send_frame(Opcode::Query, b"GET 42").await.unwrap();
        let (_, payload) = client.recv_frame().await.unwrap();
        assert_eq!(&payload, b"GET 42");
        srv.await.unwrap();
    }

    #[tokio::test]
    async fn request_id_increments() {
        let (listener, addr) = listen().await;
        let srv = tokio::spawn(async move {
            let (stream, _) = listener.accept().await.unwrap();
            let mut conn = Connection { stream, next_id: 1 };
            for _ in 0..3 {
                let (_, p) = conn.recv_frame().await.unwrap();
                conn.send_frame(Opcode::PingAck, &p).await.unwrap();
            }
        });
        let mut client = Connection::connect(addr).await.unwrap();
        let id1 = client.send_frame(Opcode::Ping, b"").await.unwrap();
        let _ = client.recv_frame().await.unwrap();
        let id2 = client.send_frame(Opcode::Ping, b"").await.unwrap();
        let _ = client.recv_frame().await.unwrap();
        let id3 = client.send_frame(Opcode::Ping, b"").await.unwrap();
        let _ = client.recv_frame().await.unwrap();
        assert_eq!(id1 + 1, id2);
        assert_eq!(id2 + 1, id3);
        srv.await.unwrap();
    }

    #[tokio::test]
    async fn empty_payload_roundtrip() {
        let (listener, addr) = listen().await;
        let srv = tokio::spawn(echo_server(listener));
        let mut client = Connection::connect(addr).await.unwrap();
        client.send_frame(Opcode::Health, b"").await.unwrap();
        let (header, payload) = client.recv_frame().await.unwrap();
        assert_eq!(payload.len(), 0);
        assert_eq!(header.payload_len, 0);
        srv.await.unwrap();
    }

    #[tokio::test]
    async fn large_payload_roundtrip() {
        let (listener, addr) = listen().await;
        let srv = tokio::spawn(echo_server(listener));
        let mut client = Connection::connect(addr).await.unwrap();
        let data = vec![0xABu8; 65536];
        client.send_frame(Opcode::Execute, &data).await.unwrap();
        let (_, payload) = client.recv_frame().await.unwrap();
        assert_eq!(payload, data);
        srv.await.unwrap();
    }

    #[tokio::test]
    async fn peer_addr_available() {
        let (listener, addr) = listen().await;
        let srv = tokio::spawn(async move { listener.accept().await.unwrap() });
        let client = Connection::connect(addr).await.unwrap();
        assert_eq!(client.peer_addr().unwrap().port(), addr.port());
        srv.await.unwrap();
    }

    #[tokio::test]
    async fn local_addr_available() {
        let (listener, addr) = listen().await;
        let srv = tokio::spawn(async move { listener.accept().await.unwrap() });
        let client = Connection::connect(addr).await.unwrap();
        assert!(client.local_addr().is_ok());
        srv.await.unwrap();
    }

    #[tokio::test]
    async fn connect_timeout_succeeds() {
        let (listener, addr) = listen().await;
        let srv = tokio::spawn(async move { listener.accept().await.unwrap() });
        let conn = Connection::connect_timeout(addr, Duration::from_secs(5))
            .await
            .unwrap();
        assert!(conn.peer_addr().is_ok());
        srv.await.unwrap();
    }

    #[tokio::test]
    async fn health_opcode_roundtrip() {
        let (listener, addr) = listen().await;
        let srv = tokio::spawn(async move {
            let (stream, _) = listener.accept().await.unwrap();
            let mut conn = Connection { stream, next_id: 1 };
            let (hdr, payload) = conn.recv_frame().await.unwrap();
            assert_eq!(hdr.opcode, Opcode::Health as u8);
            conn.send_frame(Opcode::PingAck, &payload).await.unwrap();
        });
        let mut client = Connection::connect(addr).await.unwrap();
        client.send_frame(Opcode::Health, b"ok").await.unwrap();
        let (_, p) = client.recv_frame().await.unwrap();
        assert_eq!(&p, b"ok");
        srv.await.unwrap();
    }

    #[tokio::test]
    async fn txn_begin_opcode_roundtrip() {
        let (listener, addr) = listen().await;
        let srv = tokio::spawn(echo_server(listener));
        let mut client = Connection::connect(addr).await.unwrap();
        client.send_frame(Opcode::TxnBegin, b"txn").await.unwrap();
        let (_, p) = client.recv_frame().await.unwrap();
        assert_eq!(&p, b"txn");
        srv.await.unwrap();
    }
}
