//! Layer 186 — TCP framing: real TCP loopback frame send/receive

use std::io::Write;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tempfile::TempDir;
use vectorvolt_engine::{Durability, EngineOptions, TableEngine};
use vectorvolt_net::protocol::{OpCode, encode_frame};
use vectorvolt_net::{Client, Server, ServerConfig};

struct Harness {
    addr: SocketAddr,
    shutdown: vectorvolt_net::server::ShutdownHandle,
    _dir: TempDir,
}

impl Harness {
    fn start() -> Self {
        let dir = TempDir::new().unwrap();
        let engine = Arc::new(
            TableEngine::open_with(
                vec!["id".into(), "price".into(), "qty".into()],
                dir.path(),
                EngineOptions::new(1_000_000).with_durability(Durability::WriteOnly),
            )
            .unwrap(),
        );
        let cfg = ServerConfig::new("127.0.0.1:0".parse().unwrap());
        let server = Server::bind(engine, cfg).unwrap();
        let addr = server.local_addr().unwrap();
        let shutdown = server.shutdown_handle();
        std::thread::spawn(move || {
            let _ = server.serve();
        });
        Harness {
            addr,
            shutdown,
            _dir: dir,
        }
    }

    fn client(&self) -> Client {
        let c = Client::connect(self.addr).unwrap();
        c.set_timeout(Some(Duration::from_secs(10))).unwrap();
        c
    }
}

impl Drop for Harness {
    fn drop(&mut self) {
        self.shutdown.shutdown();
    }
}

// 1. server local_addr has nonzero port
#[test]
fn server_local_addr_nonzero_port() {
    let h = Harness::start();
    assert_ne!(h.addr.port(), 0);
}

// 2. client connect returns Ok
#[test]
fn client_connect_ok() {
    let h = Harness::start();
    assert!(Client::connect(h.addr).is_ok());
}

// 3. client ping round-trips
#[test]
fn client_ping_roundtrips() {
    let h = Harness::start();
    let mut c = h.client();
    assert!(c.ping().is_ok());
}

// 4. ping twice ok
#[test]
fn client_ping_twice_ok() {
    let h = Harness::start();
    let mut c = h.client();
    c.ping().unwrap();
    assert!(c.ping().is_ok());
}

// 5. insert returns Ok (values include all 3 columns)
#[test]
fn client_insert_ok() {
    let h = Harness::start();
    let mut c = h.client();
    assert!(c.insert(1, vec![1, 100, 7]).is_ok());
}

// 6. get after insert returns Some
#[test]
fn client_get_after_insert_some() {
    let h = Harness::start();
    let mut c = h.client();
    c.insert(42, vec![42, 420, 6]).unwrap();
    assert!(c.get(42).unwrap().is_some());
}

// 7. get missing pk returns None
#[test]
fn client_get_missing_none() {
    let h = Harness::start();
    let mut c = h.client();
    assert!(c.get(9999).unwrap().is_none());
}

// 8. delete after insert returns Ok
#[test]
fn client_delete_ok() {
    let h = Harness::start();
    let mut c = h.client();
    c.insert(5, vec![5, 50, 1]).unwrap();
    assert!(c.delete(5).is_ok());
}

// 9. get after delete returns None
#[test]
fn client_get_after_delete_none() {
    let h = Harness::start();
    let mut c = h.client();
    c.insert(7, vec![7, 70, 2]).unwrap();
    c.delete(7).unwrap();
    assert!(c.get(7).unwrap().is_none());
}

// 10. raw frame write to TCP stream ok
#[test]
fn raw_frame_write_ok() {
    use std::net::TcpStream;
    let h = Harness::start();
    let mut stream = TcpStream::connect_timeout(&h.addr, Duration::from_secs(2)).unwrap();
    let frame = encode_frame(1, OpCode::Ping, &[]).unwrap();
    assert!(stream.write_all(&frame).is_ok());
}

// 11. server survives connection drop then accepts new one
#[test]
fn server_survives_connection_drop() {
    let h = Harness::start();
    {
        let _ = Client::connect(h.addr).unwrap();
        // dropped immediately
    }
    let mut c = h.client();
    assert!(c.ping().is_ok());
}

// 12. two clients concurrently ok
#[test]
fn two_clients_concurrent_ok() {
    let h = Harness::start();
    let mut c1 = h.client();
    let mut c2 = h.client();
    c1.insert(10, vec![10, 100, 3]).unwrap();
    c2.insert(20, vec![20, 200, 4]).unwrap();
    assert!(c1.get(10).unwrap().is_some());
    assert!(c2.get(20).unwrap().is_some());
}

// 13. insert multiple rows via loop
#[test]
fn insert_multiple_rows_ok() {
    let h = Harness::start();
    let mut c = h.client();
    for i in 1u64..=5 {
        assert!(c.insert(i, vec![i, i * 100, i * 7]).is_ok());
    }
}
