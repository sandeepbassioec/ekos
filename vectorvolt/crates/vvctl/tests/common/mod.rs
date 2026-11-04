//! Shared helper: a real wire-protocol server on an ephemeral port.
#![allow(dead_code)]

use std::sync::Arc;

use vectorvolt_engine::{EngineOptions, TableRegistry};
use vectorvolt_net::server::{Server, ServerConfig, ShutdownHandle};

pub struct TestServer {
    pub port: u16,
    shutdown: ShutdownHandle,
    _dir: tempfile::TempDir,
}

impl TestServer {
    /// Start a server whose `default` table holds `rows` rows.
    pub fn start(rows: u64) -> Self {
        let dir = tempfile::tempdir().unwrap();
        let reg = Arc::new(TableRegistry::open(dir.path(), EngineOptions::default()).unwrap());
        let t = reg
            .create_table("default", vec!["a".into(), "b".into()])
            .unwrap();
        for pk in 0..rows {
            t.insert(pk, vec![pk, pk * 2]).unwrap();
        }
        t.flush().unwrap();
        let cfg = ServerConfig::new("127.0.0.1:0".parse().unwrap());
        let server = Server::bind_registry(reg, cfg).unwrap();
        let port = server.local_addr().unwrap().port();
        let shutdown = server.shutdown_handle();
        std::thread::spawn(move || {
            let _ = server.serve();
        });
        Self {
            port,
            shutdown,
            _dir: dir,
        }
    }
}

impl Drop for TestServer {
    fn drop(&mut self) {
        self.shutdown.shutdown();
    }
}

/// A port nothing listens on.
pub fn dead_port() -> u16 {
    std::net::TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port()
}
