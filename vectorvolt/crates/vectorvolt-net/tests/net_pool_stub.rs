//! Tests for Layer 312: ConnectionPool (VV-CTL-312).
//! Updated from stub API to real async API.

use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use vectorvolt_net::ConnectionPool;
use vectorvolt_net::frame::Opcode;

async fn start_echo() -> (SocketAddr, tokio::task::JoinHandle<()>) {
    let listener = Arc::new(TcpListener::bind("127.0.0.1:0").await.unwrap());
    let addr = listener.local_addr().unwrap();
    let h = tokio::spawn(async move {
        loop {
            let Ok((mut stream, _)) = listener.accept().await else {
                break;
            };
            tokio::spawn(async move {
                use tokio::io::{AsyncReadExt, AsyncWriteExt};
                use vectorvolt_net::frame::{WIRE_HEADER_SIZE, WireFrameHeader};
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
    (addr, h)
}

#[tokio::test]
async fn pool_new_stores_max() {
    let (addr, _srv) = start_echo().await;
    let p = ConnectionPool::new(addr, 10).await.unwrap();
    assert_eq!(p.capacity(), 10);
}

#[tokio::test]
async fn pool_size_returns_max() {
    let (addr, _srv) = start_echo().await;
    let p = ConnectionPool::new(addr, 5).await.unwrap();
    assert_eq!(p.capacity(), 5);
}

#[tokio::test]
async fn pool_zero_capacity_becomes_one() {
    let (addr, _srv) = start_echo().await;
    let p = ConnectionPool::new(addr, 0).await.unwrap();
    assert_eq!(p.capacity(), 1);
}

#[tokio::test]
async fn pool_large_capacity() {
    let (addr, _srv) = start_echo().await;
    let p = ConnectionPool::new(addr, 10).await.unwrap();
    assert_eq!(p.capacity(), 10);
}

#[tokio::test]
async fn pool_acquire_returns_conn() {
    let (addr, _srv) = start_echo().await;
    let p = ConnectionPool::new(addr, 3).await.unwrap();
    let _conn = p.acquire().await.unwrap();
    assert_eq!(p.capacity(), 3);
}

#[tokio::test]
async fn pool_new_capacity_1() {
    let (addr, _srv) = start_echo().await;
    let p = ConnectionPool::new(addr, 1).await.unwrap();
    assert_eq!(p.capacity(), 1);
}

#[tokio::test]
async fn pool_new_capacity_5() {
    let (addr, _srv) = start_echo().await;
    let p = ConnectionPool::new(addr, 5).await.unwrap();
    assert_eq!(p.capacity(), 5);
}

#[tokio::test]
async fn pool_two_different_capacities() {
    let (addr, _srv) = start_echo().await;
    let a = ConnectionPool::new(addr, 10).await.unwrap();
    let b = ConnectionPool::new(addr, 20).await.unwrap();
    assert_ne!(a.capacity(), b.capacity());
}

#[tokio::test]
async fn pool_idle_count_on_fresh_pool() {
    let (addr, _srv) = start_echo().await;
    let p = ConnectionPool::new(addr, 4).await.unwrap();
    assert_eq!(p.idle_count().await, 4);
}

#[tokio::test]
async fn pool_health_check_all_healthy() {
    let (addr, _srv) = start_echo().await;
    let p = ConnectionPool::new(addr, 2).await.unwrap();
    let healthy = p.health_check().await;
    assert_eq!(healthy, 2);
}

#[tokio::test]
async fn pool_acquire_then_drop_returns_conn() {
    let (addr, _srv) = start_echo().await;
    let p = ConnectionPool::new(addr, 1).await.unwrap();
    {
        let _c = p.acquire().await.unwrap();
    }
    let _c2 = p.acquire().await.unwrap();
}

#[tokio::test]
async fn pool_capacity_constant_after_acquire() {
    let (addr, _srv) = start_echo().await;
    let p = ConnectionPool::new(addr, 3).await.unwrap();
    let _c = p.acquire().await.unwrap();
    assert_eq!(p.capacity(), 3);
}
