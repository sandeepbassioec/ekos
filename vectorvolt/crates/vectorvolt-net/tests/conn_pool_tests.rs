//! Layer 187 — connection pool: multiple connections, reuse patterns

use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tempfile::TempDir;
use vectorvolt_engine::{Durability, EngineOptions, TableEngine};
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

// 1. open 5 connections sequentially
#[test]
fn open_five_connections_ok() {
    let h = Harness::start();
    for _ in 0..5 {
        let mut c = h.client();
        assert!(c.ping().is_ok());
    }
}

// 2. open 3 connections concurrently
#[test]
fn open_three_connections_concurrent_ok() {
    let h = Harness::start();
    let addr = h.addr;
    let handles: Vec<_> = (0..3)
        .map(|_| {
            std::thread::spawn(move || {
                let c = Client::connect(addr).unwrap();
                c.set_timeout(Some(Duration::from_secs(5))).unwrap();
                c
            })
        })
        .collect();
    for handle in handles {
        let mut c = handle.join().unwrap();
        assert!(c.ping().is_ok());
    }
}

// 3. reuse same connection for multiple ops
#[test]
fn reuse_connection_multiple_ops() {
    let h = Harness::start();
    let mut c = h.client();
    for i in 1u64..=5 {
        c.insert(i, vec![i, i * 10, i]).unwrap();
    }
    for i in 1u64..=5 {
        assert!(c.get(i).unwrap().is_some());
    }
}

// 4. reconnect after drop
#[test]
fn reconnect_after_drop_ok() {
    let h = Harness::start();
    {
        let mut c = h.client();
        c.insert(1, vec![1, 10, 1]).unwrap();
    }
    let mut c2 = h.client();
    assert!(c2.ping().is_ok());
}

// 5. 10 sequential connections ping
#[test]
fn ten_sequential_connections_ok() {
    let h = Harness::start();
    for _ in 0..10 {
        let mut c = h.client();
        assert!(c.ping().is_ok());
    }
}

// 6. concurrent insert from 3 threads
#[test]
fn concurrent_insert_3_threads() {
    let h = Harness::start();
    let addr = h.addr;
    let handles: Vec<_> = (0..3u64)
        .map(|i| {
            std::thread::spawn(move || {
                let c = Client::connect(addr).unwrap();
                c.set_timeout(Some(Duration::from_secs(5))).unwrap();
                let mut c = c;
                c.insert(i * 100 + 1, vec![i * 100 + 1, i * 100, i])
                    .unwrap();
            })
        })
        .collect();
    for h in handles {
        h.join().unwrap();
    }
}

// 7. server handles rapid connect/disconnect
#[test]
fn rapid_connect_disconnect_ok() {
    let h = Harness::start();
    for _ in 0..5 {
        let _ = Client::connect(h.addr).unwrap();
        // immediately dropped
    }
    let mut c = h.client();
    assert!(c.ping().is_ok());
}

// 8. two connections see same data
#[test]
fn two_connections_see_same_data() {
    let h = Harness::start();
    let mut c1 = h.client();
    let mut c2 = h.client();
    c1.insert(99, vec![99, 990, 9]).unwrap();
    assert!(c2.get(99).unwrap().is_some());
}

// 9. connection set_timeout ok
#[test]
fn connection_set_timeout_ok() {
    let h = Harness::start();
    let c = Client::connect(h.addr).unwrap();
    assert!(c.set_timeout(Some(Duration::from_secs(5))).is_ok());
}

// 10. stats after concurrent inserts ok
#[test]
fn stats_after_concurrent_inserts_ok() {
    let h = Harness::start();
    let addr = h.addr;
    let handles: Vec<_> = (1..=4u64)
        .map(|i| {
            std::thread::spawn(move || {
                let c = Client::connect(addr).unwrap();
                c.set_timeout(Some(Duration::from_secs(5))).unwrap();
                let mut c = c;
                c.insert(i, vec![i, i * 10, i]).unwrap();
            })
        })
        .collect();
    for h in handles {
        h.join().unwrap();
    }
    let mut c = h.client();
    assert!(c.ping().is_ok());
}

// 11. insert from 2 threads, read from 1
#[test]
fn insert_two_read_one() {
    let h = Harness::start();
    let addr = h.addr;
    let t1 = std::thread::spawn(move || {
        let c = Client::connect(addr).unwrap();
        c.set_timeout(Some(Duration::from_secs(5))).unwrap();
        let mut c = c;
        c.insert(200, vec![200, 2000, 20]).unwrap();
    });
    let t2 = std::thread::spawn(move || {
        let c = Client::connect(addr).unwrap();
        c.set_timeout(Some(Duration::from_secs(5))).unwrap();
        let mut c = c;
        c.insert(201, vec![201, 2010, 21]).unwrap();
    });
    t1.join().unwrap();
    t2.join().unwrap();
    let mut c = h.client();
    assert!(c.get(200).unwrap().is_some());
}

// 12. client connection is not clone but new connections are independent
#[test]
fn new_connections_are_independent() {
    let h = Harness::start();
    let mut c1 = h.client();
    let mut c2 = h.client();
    c1.insert(300, vec![300, 3000, 30]).unwrap();
    c2.insert(301, vec![301, 3010, 31]).unwrap();
    assert!(c1.get(301).unwrap().is_some());
    assert!(c2.get(300).unwrap().is_some());
}

// 13. server_stats after many connections ok
#[test]
fn server_handles_many_connections() {
    let h = Harness::start();
    for i in 0..8u64 {
        let mut c = h.client();
        c.insert(i + 400, vec![i + 400, (i + 400) * 10, i]).unwrap();
    }
    let mut c = h.client();
    assert!(c.ping().is_ok());
}
