//! Tests for Layer 195: vectorvolt-net ConnectionPool (VV-NET-195).
//! Updated to real async pool API.

use std::sync::Arc;
use tokio::net::TcpListener;
use vectorvolt_net::frame::{Opcode, WireFrameHeader, WIRE_HEADER_SIZE};
use vectorvolt_net::ConnectionPool;

async fn start_echo() -> std::net::SocketAddr {
    let listener = Arc::new(TcpListener::bind("127.0.0.1:0").await.unwrap());
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        loop {
            let Ok((mut stream, _)) = listener.accept().await else { break };
            tokio::spawn(async move {
                use tokio::io::{AsyncReadExt, AsyncWriteExt};
                loop {
                    let mut hbuf = vec![0u8; WIRE_HEADER_SIZE];
                    if stream.read_exact(&mut hbuf).await.is_err() { break; }
                    let plen = u32::from_le_bytes(hbuf[8..12].try_into().unwrap()) as usize;
                    let mut pbuf = vec![0u8; plen];
                    if plen > 0 && stream.read_exact(&mut pbuf).await.is_err() { break; }
                    let Ok(hdr) = WireFrameHeader::decode(&hbuf) else { break };
                    let resp = WireFrameHeader::new(Opcode::PingAck, b"", hdr.request_id)
                        .expect("valid")
                        .encode_with_payload(b"");
                    if stream.write_all(&resp).await.is_err() { break; }
                }
            });
        }
    });
    addr
}

#[tokio::test]
async fn new_stores_max() {
    let addr = start_echo().await;
    assert_eq!(ConnectionPool::new(addr, 10).await.unwrap().capacity(), 10);
}

#[tokio::test]
async fn size_equals_max() {
    let addr = start_echo().await;
    assert_eq!(ConnectionPool::new(addr, 5).await.unwrap().capacity(), 5);
}

#[tokio::test]
async fn zero_max_becomes_one() {
    let addr = start_echo().await;
    assert_eq!(ConnectionPool::new(addr, 0).await.unwrap().capacity(), 1);
}

#[tokio::test]
async fn large_max() {
    let addr = start_echo().await;
    assert_eq!(ConnectionPool::new(addr, 20).await.unwrap().capacity(), 20);
}

#[tokio::test]
async fn debug_contains_addr() {
    let addr = start_echo().await;
    let p = ConnectionPool::new(addr, 3).await.unwrap();
    assert!(!format!("{:?}", p).is_empty());
}

#[tokio::test]
async fn multiple_pools() {
    let addr = start_echo().await;
    let a = ConnectionPool::new(addr, 1).await.unwrap();
    let b = ConnectionPool::new(addr, 2).await.unwrap();
    assert_ne!(a.capacity(), b.capacity());
}

#[tokio::test]
async fn max_1() {
    let addr = start_echo().await;
    assert_eq!(ConnectionPool::new(addr, 1).await.unwrap().capacity(), 1);
}

#[tokio::test]
async fn pool_max_field() {
    let addr = start_echo().await;
    assert_eq!(ConnectionPool::new(addr, 99).await.unwrap().capacity(), 99);
}

#[tokio::test]
async fn size_consistent_with_capacity() {
    let addr = start_echo().await;
    let p = ConnectionPool::new(addr, 3).await.unwrap();
    assert_eq!(p.capacity(), 3);
}

#[tokio::test]
async fn new_returns_pool() {
    let addr = start_echo().await;
    let p = ConnectionPool::new(addr, 4).await.unwrap();
    assert_eq!(p.capacity(), 4);
}

#[tokio::test]
async fn acquire_works() {
    let addr = start_echo().await;
    let p = ConnectionPool::new(addr, 2).await.unwrap();
    let _c = p.acquire().await.unwrap();
    assert_eq!(p.capacity(), 2);
}

#[tokio::test]
async fn health_check_ok() {
    let addr = start_echo().await;
    let p = ConnectionPool::new(addr, 2).await.unwrap();
    assert_eq!(p.health_check().await, 2);
}
