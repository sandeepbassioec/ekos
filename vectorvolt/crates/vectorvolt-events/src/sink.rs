//! EventSink trait + InProcessChannelSink, HttpWebhookSink, RedisStreamSink stubs.
//! VV-EVT-004..007.

use std::sync::Arc;
use std::time::Duration;

use hmac::{Hmac, Mac};
use sha2::Sha256;
use thiserror::Error;
use tokio::sync::broadcast;

use crate::envelope::DomainEventEnvelope;

#[derive(Debug, Error)]
pub enum SinkError {
    #[error("json: {0}")]
    Json(#[from] serde_json::Error),
    #[error("http: {0}")]
    Http(String),
    #[error("channel closed")]
    ChannelClosed,
    #[error("redis: {0}")]
    Redis(String),
}

pub type SinkResult = Result<(), SinkError>;

type WriterFn = Arc<dyn Fn(&str, u64, &str) -> SinkResult + Send + Sync>;

/// Pluggable delivery target for domain event envelopes (VV-EVT-004..007).
pub trait EventSink: Send + Sync {
    fn send(&self, envelope: &DomainEventEnvelope) -> SinkResult;
}

// ── InProcessChannelSink ─────────────────────────────────────────────────────

/// Broadcasts envelopes via a tokio broadcast channel (VV-EVT-007).
pub struct InProcessChannelSink {
    tx: broadcast::Sender<DomainEventEnvelope>,
}

impl InProcessChannelSink {
    pub fn new(capacity: usize) -> (Self, broadcast::Receiver<DomainEventEnvelope>) {
        let (tx, rx) = broadcast::channel(capacity);
        (Self { tx }, rx)
    }

    pub fn subscribe(&self) -> broadcast::Receiver<DomainEventEnvelope> {
        self.tx.subscribe()
    }
}

impl EventSink for InProcessChannelSink {
    fn send(&self, envelope: &DomainEventEnvelope) -> SinkResult {
        self.tx
            .send(envelope.clone())
            .map_err(|_| SinkError::ChannelClosed)?;
        Ok(())
    }
}

// ── HttpWebhookSink ──────────────────────────────────────────────────────────

/// POSTs envelope JSON with optional HMAC-SHA256 signature (VV-EVT-006).
pub struct HttpWebhookSink {
    url: String,
    hmac_secret: Option<String>,
    retry_max: u32,
    retry_backoff: Duration,
    client: reqwest::Client,
}

impl HttpWebhookSink {
    pub fn new(
        url: impl Into<String>,
        hmac_secret: Option<String>,
        timeout_s: u64,
        retry_max: u32,
        retry_backoff_ms: u64,
    ) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(timeout_s))
            // Each blocking `send` drives the request on its own short-lived
            // runtime; a pooled connection bound to an earlier (dropped)
            // runtime would fail on reuse, so keep no idle connections.
            .pool_max_idle_per_host(0)
            .build()
            .expect("webhook http client");
        Self {
            url: url.into(),
            hmac_secret,
            retry_max,
            retry_backoff: Duration::from_millis(retry_backoff_ms),
            client,
        }
    }

    fn sign(&self, body: &str) -> Option<String> {
        let secret = self.hmac_secret.as_deref()?;
        let mut mac = Hmac::<Sha256>::new_from_slice(secret.as_bytes()).ok()?;
        mac.update(body.as_bytes());
        Some(format!(
            "sha256={}",
            hex::encode(mac.finalize().into_bytes())
        ))
    }
}

impl HttpWebhookSink {
    /// Deliver `body` (with retries) and return the final outcome.
    fn deliver_blocking(&self, body: &str, sig: Option<&str>) -> SinkResult {
        let run = || -> SinkResult {
            tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .map_err(|e| SinkError::Http(format!("webhook runtime: {e}")))?
                .block_on(webhook_post_with_retry(
                    &self.client,
                    &self.url,
                    body,
                    sig,
                    self.retry_max,
                    self.retry_backoff,
                ))
        };
        if tokio::runtime::Handle::try_current().is_ok() {
            // Inside a runtime a nested `block_on` would panic, so drive the
            // request on a helper thread and wait for its result. This blocks
            // the calling thread — the price of a synchronous `send` that
            // reports the real outcome.
            std::thread::scope(|scope| {
                scope
                    .spawn(run)
                    .join()
                    .unwrap_or_else(|_| Err(SinkError::Http("webhook delivery panicked".into())))
            })
        } else {
            run()
        }
    }

    /// Fire-and-forget delivery: returns as soon as the request is scheduled
    /// and never reports HTTP failures (they are only logged).
    ///
    /// This is the old behaviour of [`EventSink::send`]; it is kept for
    /// callers that explicitly do not want delivery feedback. Anything that
    /// routes failures to an outbox/DLQ must use `send` instead.
    pub fn send_fire_and_forget(&self, envelope: &DomainEventEnvelope) -> SinkResult {
        let body = serde_json::to_string(envelope)?;
        let sig = self.sign(&body);
        let client = self.client.clone();
        let url = self.url.clone();
        let retry_max = self.retry_max;
        let backoff = self.retry_backoff;
        let task = async move {
            // Failures are already logged inside; nothing to report to.
            let _ =
                webhook_post_with_retry(&client, &url, &body, sig.as_deref(), retry_max, backoff)
                    .await;
        };
        if let Ok(handle) = tokio::runtime::Handle::try_current() {
            handle.spawn(task);
        } else {
            std::thread::spawn(move || {
                if let Ok(rt) = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                {
                    rt.block_on(task);
                }
            });
        }
        Ok(())
    }
}

impl EventSink for HttpWebhookSink {
    /// POST the envelope, retrying up to `retry_max` attempts, and return the
    /// real outcome: `Err(SinkError::Http)` if every attempt failed, so the
    /// outbox can move the envelope to the DLQ (PP-66). Blocks until done.
    fn send(&self, envelope: &DomainEventEnvelope) -> SinkResult {
        let body = serde_json::to_string(envelope)?;
        let sig = self.sign(&body);
        self.deliver_blocking(&body, sig.as_deref())
    }
}

async fn webhook_post_with_retry(
    client: &reqwest::Client,
    url: &str,
    body: &str,
    sig: Option<&str>,
    retry_max: u32,
    backoff: Duration,
) -> SinkResult {
    let mut attempt = 0u32;
    loop {
        let mut req = client
            .post(url)
            .header("Content-Type", "application/json")
            .body(body.to_string());
        if let Some(s) = sig {
            req = req.header("X-VV-Signature", s);
        }
        let failure = match req.send().await {
            Ok(resp) if resp.status().is_success() => return Ok(()),
            Ok(resp) => {
                tracing::warn!("webhook {url} returned {}", resp.status());
                format!("webhook {url} returned {}", resp.status())
            }
            Err(e) => {
                tracing::warn!("webhook {url} error: {e}");
                format!("webhook {url} error: {e}")
            }
        };
        attempt += 1;
        if attempt >= retry_max {
            return Err(SinkError::Http(format!(
                "{failure} (gave up after {attempt} attempt(s))"
            )));
        }
        let delay = backoff * 2u32.pow(attempt - 1);
        tokio::time::sleep(delay).await;
    }
}

// ── RedisStreamSink ──────────────────────────────────────────────────────────

/// XADD to a Redis Stream per event (VV-EVT-005).
///
/// [`RedisStreamSink::connect`] builds a real sink on the `redis` crate (sync
/// API; `send` blocks until Redis acknowledges the XADD). [`noop`] and [`new`]
/// (custom writer closure) exist for tests.
///
/// [`noop`]: RedisStreamSink::noop
/// [`new`]: RedisStreamSink::new
pub struct RedisStreamSink {
    stream_name: String,
    maxlen: Option<u64>,
    /// Write fn `(stream, maxlen (0 = unbounded), json body)`.
    writer: WriterFn,
}

/// Build the `XADD stream [MAXLEN ~ n] * data <body>` command.
pub fn xadd_cmd(stream: &str, maxlen: u64, body: &str) -> redis::Cmd {
    let mut cmd = redis::cmd("XADD");
    cmd.arg(stream);
    if maxlen > 0 {
        cmd.arg("MAXLEN").arg("~").arg(maxlen);
    }
    cmd.arg("*").arg("data").arg(body);
    cmd
}

impl RedisStreamSink {
    pub fn new(stream_name: impl Into<String>, maxlen: Option<u64>, writer: WriterFn) -> Self {
        Self {
            stream_name: stream_name.into(),
            maxlen,
            writer,
        }
    }

    /// No-op sink for testing (records nothing, always succeeds).
    pub fn noop(stream_name: impl Into<String>) -> Self {
        Self::new(stream_name, None, Arc::new(|_, _, _| Ok(())))
    }

    /// Connect to Redis (`redis://host:port/db`). Fails if unreachable.
    /// A broken connection is dropped and re-established on the next send.
    pub fn connect(
        url: &str,
        stream_name: impl Into<String>,
        maxlen: Option<u64>,
        connect_timeout: Duration,
    ) -> Result<Self, SinkError> {
        let client = redis::Client::open(url).map_err(|e| SinkError::Redis(e.to_string()))?;
        let first = client
            .get_connection_with_timeout(connect_timeout)
            .map_err(|e| SinkError::Redis(format!("connect: {e}")))?;
        let conn = std::sync::Mutex::new(Some(first));
        let writer: WriterFn = Arc::new(move |stream, maxlen, body| {
            let mut guard = conn.lock().unwrap_or_else(|p| p.into_inner());
            if guard.is_none() {
                *guard = Some(
                    client
                        .get_connection_with_timeout(connect_timeout)
                        .map_err(|e| SinkError::Redis(format!("reconnect: {e}")))?,
                );
            }
            let c = guard.as_mut().expect("connection present");
            match xadd_cmd(stream, maxlen, body).query::<String>(c) {
                Ok(_) => Ok(()),
                Err(e) => {
                    *guard = None;
                    Err(SinkError::Redis(format!("XADD: {e}")))
                }
            }
        });
        Ok(Self::new(stream_name, maxlen, writer))
    }

    /// Connect using a parsed [`RedisSinkConfig`](crate::config::RedisSinkConfig).
    pub fn from_config(
        cfg: &crate::config::RedisSinkConfig,
        stream_name: impl Into<String>,
        maxlen: Option<u64>,
    ) -> Result<Self, SinkError> {
        Self::connect(
            &cfg.url,
            stream_name,
            maxlen,
            Duration::from_millis(cfg.connect_timeout_ms),
        )
    }
}

impl EventSink for RedisStreamSink {
    fn send(&self, envelope: &DomainEventEnvelope) -> SinkResult {
        let body = serde_json::to_string(envelope)?;
        (self.writer)(&self.stream_name, self.maxlen.unwrap_or(0), &body)
    }
}

#[cfg(test)]
mod redis_tests {
    use super::*;

    #[test]
    fn xadd_cmd_encoding() {
        let packed = xadd_cmd("s", 0, "{}").get_packed_command();
        let text = String::from_utf8_lossy(&packed).to_string();
        assert!(text.contains("XADD") && text.contains("data"), "{text:?}");
        assert!(!text.contains("MAXLEN"));
        let packed = xadd_cmd("s", 1000, "{}").get_packed_command();
        let text = String::from_utf8_lossy(&packed).to_string();
        assert!(text.contains("MAXLEN") && text.contains("1000"), "{text:?}");
    }

    #[test]
    fn connect_unreachable_errors() {
        let r = RedisStreamSink::connect(
            "redis://127.0.0.1:1/",
            "s",
            None,
            Duration::from_millis(500),
        );
        assert!(matches!(r, Err(SinkError::Redis(_))));
    }

    #[test]
    fn connect_bad_url_errors() {
        let r = RedisStreamSink::connect("not a url", "s", None, Duration::from_millis(100));
        assert!(r.is_err());
    }

    /// Needs a live Redis: `VV_TEST_REDIS=redis://host:6379/`.
    #[test]
    #[ignore = "requires VV_TEST_REDIS"]
    fn live_xadd() {
        let url = std::env::var("VV_TEST_REDIS").expect("VV_TEST_REDIS");
        let sink =
            RedisStreamSink::connect(&url, "vv:test:events", Some(100), Duration::from_secs(3))
                .unwrap();
        (sink.writer)("vv:test:events", 100, "{\"k\":1}").unwrap();
        let client = redis::Client::open(url.as_str()).unwrap();
        let mut c = client.get_connection().unwrap();
        let n: u64 = redis::cmd("XLEN")
            .arg("vv:test:events")
            .query(&mut c)
            .unwrap();
        assert!(n >= 1);
        let _: () = redis::cmd("DEL")
            .arg("vv:test:events")
            .query(&mut c)
            .unwrap();
    }
}
