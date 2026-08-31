//! `vectorvolt-server` — server runtime (VV-SRV-187).

use vectorvolt_net::NetConfig;

/// Server configuration (VV-SRV-187).
#[derive(Default)]
pub struct ServerConfig {
    pub net: NetConfig,
}

/// Start the server (stub — returns immediately) (VV-SRV-187).
pub fn start(_cfg: ServerConfig) -> anyhow::Result<()> {
    Ok(())
}

/// Request types the server handles (VV-SRV-192).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Request {
    Ping,
    Status,
}

/// Response types the server sends (VV-SRV-192).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Response {
    Pong,
    StatusOk(String),
}

/// Handle a request and produce a response (VV-SRV-192).
pub fn handle(req: Request) -> Response {
    match req {
        Request::Ping => Response::Pong,
        Request::Status => Response::StatusOk(server_status()),
    }
}

/// Return a JSON status string (VV-SRV-193).
pub fn server_status() -> String {
    format!(
        "{{\"status\":\"ok\",\"version\":\"{}\"}}",
        env!("CARGO_PKG_VERSION")
    )
}

/// Per-server metrics (VV-SRV-197).
#[derive(Debug, Clone, Default)]
pub struct ServerMetrics {
    pub requests: u64,
    pub errors: u64,
}

impl ServerMetrics {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn record_request(&mut self) {
        self.requests += 1;
    }

    pub fn record_error(&mut self) {
        self.errors += 1;
    }

    pub fn success_rate(&self) -> f64 {
        if self.requests == 0 {
            return 1.0;
        }
        1.0 - (self.errors as f64 / self.requests as f64)
    }
}
