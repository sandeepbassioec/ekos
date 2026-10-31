//! OpenTelemetry distributed tracing for VectorVolt (VV-OBS-001..006).
//!
//! Bridges `tracing` spans to OpenTelemetry via `tracing-opentelemetry`.
//! When no OTLP endpoint is configured the crate produces zero overhead — a
//! no-op `TracerProvider` is installed and the `tracing-opentelemetry` layer
//! simply passes spans through without exporting them.
//!
//! A bad endpoint never panics: [`init`] / [`build_provider`] log the problem
//! and fall back to the no-op provider, and [`try_build_provider`] returns it
//! as an [`OtelError`] for callers that want to fail fast instead.
//!
//! # Usage in `main.rs`
//!
//! ```rust,no_run
//! // 1. Init OTel BEFORE the tracing subscriber.
//! let otel_guard = vectorvolt_otel::init(None, "vvctl");
//!
//! // 2. Build subscriber with the OTel layer included.
//! use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
//! tracing_subscriber::registry()
//!     .with(vectorvolt_otel::layer(&otel_guard))
//!     .with(tracing_subscriber::fmt::layer().compact())
//!     .init();
//!
//! // 3. `otel_guard` flushes spans on drop (end of main).
//! ```

use opentelemetry::trace::TracerProvider as OtelTracerProvider;
use opentelemetry_sdk::trace::{Tracer as SdkTracer, TracerProvider};
use tracing_opentelemetry::OpenTelemetryLayer;
use tracing_subscriber::Registry;

/// Why an OTLP tracer provider could not be built.
#[derive(Debug, thiserror::Error)]
pub enum OtelError {
    /// The endpoint is not an `http://` or `https://` URL with a host.
    #[error("invalid OTLP endpoint '{endpoint}': {reason}")]
    InvalidEndpoint { endpoint: String, reason: String },
    /// The OTLP exporter rejected its configuration.
    #[error("OTLP exporter build failed: {0}")]
    Exporter(String),
    /// The batch exporter needs a Tokio runtime and none is running.
    #[error("OTLP export needs a running Tokio runtime")]
    NoRuntime,
}

/// Cheap structural check of an OTLP endpoint (`http(s)://host[:port][/path]`).
fn validate_endpoint(endpoint: &str) -> Result<(), OtelError> {
    let invalid = |reason: &str| OtelError::InvalidEndpoint {
        endpoint: endpoint.to_string(),
        reason: reason.to_string(),
    };
    let rest = endpoint
        .strip_prefix("http://")
        .or_else(|| endpoint.strip_prefix("https://"))
        .ok_or_else(|| invalid("scheme must be http:// or https://"))?;
    let authority = rest.split(['/', '?', '#']).next().unwrap_or("");
    let (host, port) = if let Some(v6) = authority.strip_prefix('[') {
        let (host, after) = v6
            .split_once(']')
            .ok_or_else(|| invalid("unterminated IPv6 literal"))?;
        match after {
            "" => (host, None),
            _ => (
                host,
                Some(
                    after
                        .strip_prefix(':')
                        .ok_or_else(|| invalid("junk after IPv6 literal"))?,
                ),
            ),
        }
    } else {
        match authority.rsplit_once(':') {
            Some((host, port)) => (host, Some(port)),
            None => (authority, None),
        }
    };
    if host.is_empty() || host.chars().any(char::is_whitespace) {
        return Err(invalid("missing or malformed host"));
    }
    if let Some(port) = port
        && port.parse::<u16>().is_err()
    {
        return Err(invalid("port is not a number in 0..=65535"));
    }
    Ok(())
}

/// RAII guard — flushes pending spans and shuts down the OTel provider on drop.
pub struct OtelGuard {
    provider: TracerProvider,
}

impl Drop for OtelGuard {
    fn drop(&mut self) {
        let _ = self.provider.shutdown();
    }
}

/// Initialize the global OTel tracer provider.
///
/// - `endpoint`: OTLP gRPC URL (e.g. `http://localhost:4317`). `None` → no-op.
/// - `service_name`: value for the `service.name` OTel resource attribute.
///
/// **Call this before constructing the `tracing_subscriber`.**
/// The returned `OtelGuard` must be kept alive for the process lifetime.
pub fn init(endpoint: Option<&str>, service_name: &str) -> OtelGuard {
    let provider = build_provider(endpoint, service_name);
    OtelGuard { provider }
}

/// Build a `tracing-opentelemetry` layer from the guard's `TracerProvider`.
///
/// `SdkTracer` implements `PreSampledTracer` (required by `OpenTelemetryLayer`);
/// the global `BoxedTracer` does not. Call after [`init`] and before `.init()`.
pub fn layer(guard: &OtelGuard) -> OpenTelemetryLayer<Registry, SdkTracer> {
    let tracer: SdkTracer = guard.provider.tracer("vectorvolt");
    tracing_opentelemetry::layer().with_tracer(tracer)
}

/// Build an `TracerProvider` without setting it as global.
///
/// Useful when you need the provider to wire a custom layer. Never panics: if
/// the OTLP provider cannot be built (see [`try_build_provider`]) the error is
/// logged to stderr and a no-op provider is returned instead.
pub fn build_provider(endpoint: Option<&str>, service_name: &str) -> TracerProvider {
    try_build_provider(endpoint, service_name).unwrap_or_else(|e| {
        eprintln!("[vectorvolt-otel] {e}; falling back to no-op tracing");
        TracerProvider::builder().build()
    })
}

/// Like [`build_provider`], but reports a bad endpoint as an error instead of
/// falling back to the no-op provider.
pub fn try_build_provider(
    endpoint: Option<&str>,
    service_name: &str,
) -> Result<TracerProvider, OtelError> {
    match endpoint {
        None => Ok(TracerProvider::builder().build()),
        Some(ep) => {
            validate_endpoint(ep)?;
            build_otlp_provider(ep, service_name)
        }
    }
}

#[cfg(feature = "otlp")]
fn build_otlp_provider(endpoint: &str, service_name: &str) -> Result<TracerProvider, OtelError> {
    use opentelemetry::KeyValue;
    use opentelemetry_otlp::WithExportConfig as _;
    use opentelemetry_sdk::Resource;

    // The batch processor spawns onto Tokio; outside a runtime that panics.
    if tokio::runtime::Handle::try_current().is_err() {
        return Err(OtelError::NoRuntime);
    }

    let exporter = opentelemetry_otlp::SpanExporter::builder()
        .with_tonic()
        .with_endpoint(endpoint)
        .build()
        .map_err(|e| OtelError::Exporter(e.to_string()))?;

    Ok(TracerProvider::builder()
        .with_resource(Resource::new([
            KeyValue::new("service.name", service_name.to_string()),
            KeyValue::new("service.version", env!("CARGO_PKG_VERSION")),
        ]))
        .with_batch_exporter(exporter, opentelemetry_sdk::runtime::Tokio)
        .build())
}

#[cfg(not(feature = "otlp"))]
fn build_otlp_provider(endpoint: &str, _service_name: &str) -> Result<TracerProvider, OtelError> {
    eprintln!(
        "[vectorvolt-otel] endpoint '{endpoint}' ignored — \
         rebuild with --features otlp to enable OTLP export"
    );
    Ok(TracerProvider::builder().build())
}
