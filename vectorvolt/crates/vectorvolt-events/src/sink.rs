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

impl EventSink for HttpWebhookSink {
    fn send(&self, envelope: &DomainEventEnvelope) -> SinkResult {
        let body = serde_json::to_string(envelope)?;
        let sig = self.sign(&body);

        let rt = tokio::runtime::Handle::try_current();
        if rt.is_ok() {
            // async context — fire-and-forget via spawn; caller gets Ok immediately
            let client = self.client.clone();
            let url = self.url.clone();
            let body2 = body.clone();
            let sig2 = sig.clone();
            let retry_max = self.retry_max;
            let backoff = self.retry_backoff;
            tokio::spawn(async move {
                webhook_post_with_retry(&client, &url, &body2, sig2.as_deref(), retry_max, backoff)
                    .await;
            });
            return Ok(());
        }

        // blocking context
        tokio::runtime::Runtime::new()
            .expect("tokio")
            .block_on(async {
                webhook_post_with_retry(
                    &self.client,
                    &self.url,
                    &body,
                    sig.as_deref(),
                    self.retry_max,
                    self.retry_backoff,
                )
                .await;
            });
        Ok(())
    }
}

async fn webhook_post_with_retry(
    client: &reqwest::Client,
    url: &str,
    body: &str,
    sig: Option<&str>,
    retry_max: u32,
    backoff: Duration,
) {
    let mut attempt = 0u32;
    loop {
        let mut req = client
            .post(url)
            .header("Content-Type", "application/json")
            .body(body.to_string());
        if let Some(s) = sig {
            req = req.header("X-VV-Signature", s);
        }
        match req.send().await {
            Ok(resp) if resp.status().is_success() => return,
            Ok(resp) => {
                tracing::warn!("webhook {url} returned {}", resp.status());
            }
            Err(e) => {
                tracing::warn!("webhook {url} error: {e}");
            }
        }
        attempt += 1;
        if attempt >= retry_max {
            break;
        }
        let delay = backoff * 2u32.pow(attempt - 1);
        tokio::time::sleep(delay).await;
    }
}

// ── RedisStreamSink ──────────────────────────────────────────────────────────

/// XADD to a Redis Stream per event (VV-EVT-005).
///
/// Actual Redis connectivity is behind a trait so tests can use a no-op impl.
/// Production wiring happens in `vvctl serve` using the `redis` crate directly.
pub struct RedisStreamSink {
    stream_name: String,
    maxlen: Option<u64>,
    /// Pluggable write fn for testability; production sets a real redis sender.
    writer: WriterFn,
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
}

impl EventSink for RedisStreamSink {
    fn send(&self, envelope: &DomainEventEnvelope) -> SinkResult {
        let body = serde_json::to_string(envelope)?;
        (self.writer)(&self.stream_name, self.maxlen.unwrap_or(0), &body)
    }
}
