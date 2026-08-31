//! Layer 188 — health + ping frame tests

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

// 1. ping returns Ok
#[test]
fn ping_returns_ok() {
    let h = Harness::start();
    let mut c = h.client();
    assert!(c.ping().is_ok());
}

// 2. ping 10 times without reconnect
#[test]
fn ping_ten_times_ok() {
    let h = Harness::start();
    let mut c = h.client();
    for _ in 0..10 {
        assert!(c.ping().is_ok());
    }
}

// 3. ping latency under 100ms
#[test]
fn ping_latency_under_100ms() {
    let h = Harness::start();
    let mut c = h.client();
    let start = std::time::Instant::now();
    c.ping().unwrap();
    assert!(start.elapsed().as_millis() < 100, "ping too slow");
}

// 4. ping after insert ok
#[test]
fn ping_after_insert_ok() {
    let h = Harness::start();
    let mut c = h.client();
    c.insert(1, vec![1, 10, 1]).unwrap();
    assert!(c.ping().is_ok());
}

// 5. ping from multiple clients ok
#[test]
fn ping_multiple_clients_ok() {
    let h = Harness::start();
    for _ in 0..3 {
        let mut c = h.client();
        assert!(c.ping().is_ok());
    }
}

// 6. server responds to ping immediately after start
#[test]
fn ping_immediately_after_start() {
    let h = Harness::start();
    let mut c = h.client();
    let start = std::time::Instant::now();
    c.ping().unwrap();
    assert!(start.elapsed().as_secs() < 2, "ping took too long");
}

// 7. health: get missing pk = None (no error = healthy)
#[test]
fn health_get_missing_is_none() {
    let h = Harness::start();
    let mut c = h.client();
    assert!(c.get(99999).unwrap().is_none());
}

// 8. health: insert + get = Some
#[test]
fn health_insert_get_some() {
    let h = Harness::start();
    let mut c = h.client();
    c.insert(1000, vec![1000, 10000, 10]).unwrap();
    assert!(c.get(1000).unwrap().is_some());
}

// 9. health: server stats after data
#[test]
fn health_server_handles_data() {
    let h = Harness::start();
    let mut c = h.client();
    for i in 1..=5u64 {
        c.insert(i, vec![i, i * 10, i]).unwrap();
    }
    assert!(c.ping().is_ok());
}

// 10. ping from thread ok
#[test]
fn ping_from_thread_ok() {
    let h = Harness::start();
    let addr = h.addr;
    let t = std::thread::spawn(move || {
        let c = Client::connect(addr).unwrap();
        c.set_timeout(Some(Duration::from_secs(5))).unwrap();
        let mut c = c;
        c.ping().unwrap();
    });
    t.join().unwrap();
}

// 11. ping not affected by prior delete
#[test]
fn ping_after_delete_ok() {
    let h = Harness::start();
    let mut c = h.client();
    c.insert(42, vec![42, 420, 4]).unwrap();
    c.delete(42).unwrap();
    assert!(c.ping().is_ok());
}

// 12. concurrent pings from 4 threads
#[test]
fn concurrent_pings_4_threads() {
    let h = Harness::start();
    let addr = h.addr;
    let handles: Vec<_> = (0..4)
        .map(|_| {
            std::thread::spawn(move || {
                let c = Client::connect(addr).unwrap();
                c.set_timeout(Some(Duration::from_secs(5))).unwrap();
                let mut c = c;
                c.ping().unwrap();
            })
        })
        .collect();
    for h in handles {
        h.join().unwrap();
    }
}

// 13. ping is deterministic across calls
#[test]
fn ping_deterministic() {
    let h = Harness::start();
    let mut c = h.client();
    let r1 = c.ping();
    let r2 = c.ping();
    assert!(r1.is_ok());
    assert!(r2.is_ok());
}
