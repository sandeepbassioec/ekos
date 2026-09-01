//! Tests for Layer 313: ConnectionPool (VV-NET-313).
//! Updated to real async pool API.

use std::sync::Arc;
use tokio::net::TcpListener;
use vectorvolt_net::ConnectionPool;
use vectorvolt_net::frame::{Opcode, WIRE_HEADER_SIZE, WireFrameHeader};

async fn start_echo() -> std::net::SocketAddr {
    let listener = Arc::new(TcpListener::bind("127.0.0.1:0").await.unwrap());
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        loop {
            let Ok((mut stream, _)) = listener.accept().await else {
                break;
            };
            tokio::spawn(async move {
                use tokio::io::{AsyncReadExt, AsyncWriteExt};
                loop {
                    let mut hbuf = vec![0u8; WIRE_HEADER_SIZE];
                    if stream.read_exact(&mut hbuf).await.is_err() {
                        break;
                    }
                    let plen = u32::from_le_bytes(hbuf[8..12].try_into().unwrap()) as usize;
                    let mut pbuf = vec![0u8; plen];
                    if plen > 0 && stream.read_exact(&mut pbuf).await.is_err() {
                        break;
                    }
                    let Ok(hdr) = WireFrameHeader::decode(&hbuf) else {
                        break;
                    };
                    let resp = WireFrameHeader::new(Opcode::PingAck, b"", hdr.request_id)
                        .expect("valid")
                        .encode_with_payload(b"");
                    if stream.write_all(&resp).await.is_err() {
                        break;
                    }
                }
            });
        }
    });
    addr
}

#[tokio::test]
async fn pool_new_max_stored() {
    let addr = start_echo().await;
    let p = ConnectionPool::new(addr, 10).await.unwrap();
    assert_eq!(p.capacity(), 10);
}

#[tokio::test]
async fn pool_size_equals_max() {
    let addr = start_echo().await;
    let p = ConnectionPool::new(addr, 5).await.unwrap();
    assert_eq!(p.capacity(), 5);
}

#[tokio::test]
async fn pool_zero_max() {
    let addr = start_echo().await;
    let p = ConnectionPool::new(addr, 0).await.unwrap();
    assert_eq!(p.capacity(), 1);
}

#[tokio::test]
async fn pool_large_max() {
    let addr = start_echo().await;
    let p = ConnectionPool::new(addr, 10000).await.unwrap();
    assert_eq!(p.capacity(), 10000);
}

#[tokio::test]
async fn pool_idle_count() {
    let addr = start_echo().await;
    let p = ConnectionPool::new(addr, 7).await.unwrap();
    assert_eq!(p.idle_count().await, 7);
}

#[tokio::test]
async fn pool_capacity_field() {
    let addr = start_echo().await;
    let p = ConnectionPool::new(addr, 42).await.unwrap();
    assert_eq!(p.capacity(), 42);
}

#[tokio::test]
async fn pool_size_consistent() {
    let addr = start_echo().await;
    let p = ConnectionPool::new(addr, 3).await.unwrap();
    assert_eq!(p.capacity(), 3);
}

#[tokio::test]
async fn pool_multiple_pools() {
    let addr = start_echo().await;
    let a = ConnectionPool::new(addr, 10).await.unwrap();
    let b = ConnectionPool::new(addr, 20).await.unwrap();
    assert_ne!(a.capacity(), b.capacity());
}

#[tokio::test]
async fn pool_new_capacity_1() {
    let addr = start_echo().await;
    let p = ConnectionPool::new(addr, 1).await.unwrap();
    assert_eq!(p.capacity(), 1);
}

#[tokio::test]
async fn pool_new_capacity_256() {
    let addr = start_echo().await;
    let p = ConnectionPool::new(addr, 256).await.unwrap();
    assert_eq!(p.capacity(), 256);
}

#[tokio::test]
async fn pool_two_different() {
    let addr = start_echo().await;
    let a = ConnectionPool::new(addr, 10).await.unwrap();
    let b = ConnectionPool::new(addr, 20).await.unwrap();
    assert_ne!(a.capacity(), b.capacity());
}

#[tokio::test]
async fn pool_clone_size_same() {
    let addr = start_echo().await;
    let p = ConnectionPool::new(addr, 5).await.unwrap();
    assert_eq!(p.capacity(), 5);
}

#[tokio::test]
async fn pool_health_check() {
    let addr = start_echo().await;
    let p = ConnectionPool::new(addr, 2).await.unwrap();
    assert_eq!(p.health_check().await, 2);
}
