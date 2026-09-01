//! [`ServerConfig`] — VvServer tuning parameters.

use std::net::SocketAddr;

/// VvServer configuration.
#[derive(Debug, Clone)]
pub struct ServerConfig {
    /// Address to listen on.
    pub addr: SocketAddr,
    /// Rows buffered before an automatic flush to a new partition file.
    pub flush_threshold: usize,
}

impl Default for ServerConfig {
    fn default() -> Self {
        ServerConfig {
            addr: "127.0.0.1:5995".parse().unwrap(),
            flush_threshold: 1_000_000,
        }
    }
}

impl ServerConfig {
    pub fn new(addr: SocketAddr) -> Self {
        ServerConfig {
            addr,
            ..Default::default()
        }
    }
}
