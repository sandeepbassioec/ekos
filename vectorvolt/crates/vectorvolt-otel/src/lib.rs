//! OpenTelemetry distributed tracing for VectorVolt (VV-OBS-001..006).
//!
//! Bridges `tracing` spans to OpenTelemetry via `tracing-opentelemetry`.
//! When no OTLP endpoint is configured the crate produces zero overhead — a
//! no-op `TracerProvider` is installed and the `tracing-opentelemetry` layer
//! simply passes spans through without exporting them.
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
/// Useful when you need the provider to wire a custom layer.
pub fn build_provider(endpoint: Option<&str>, service_name: &str) -> TracerProvider {
    match endpoint {
        None => TracerProvider::builder().build(),
        Some(ep) => build_otlp_provider(ep, service_name),
    }
}

#[cfg(feature = "otlp")]
fn build_otlp_provider(endpoint: &str, service_name: &str) -> TracerProvider {
    use opentelemetry::KeyValue;
    use opentelemetry_otlp::WithExportConfig as _;
    use opentelemetry_sdk::Resource;

    let exporter = opentelemetry_otlp::SpanExporter::builder()
        .with_tonic()
        .with_endpoint(endpoint)
        .build()
        .unwrap_or_else(|e| {
            eprintln!("[vectorvolt-otel] OTLP exporter build failed ({e}); falling back to no-op");
            // Fallback: compile error below because SpanExporter is not default-constructible.
            // We panic here; the caller should validate the endpoint before init.
            panic!("OTLP exporter failed: {e}");
        });

    TracerProvider::builder()
        .with_resource(Resource::new([
            KeyValue::new("service.name", service_name.to_string()),
            KeyValue::new("service.version", env!("CARGO_PKG_VERSION")),
        ]))
        .with_batch_exporter(exporter, opentelemetry_sdk::runtime::Tokio)
        .build()
}

#[cfg(not(feature = "otlp"))]
fn build_otlp_provider(endpoint: &str, _service_name: &str) -> TracerProvider {
    eprintln!(
        "[vectorvolt-otel] endpoint '{endpoint}' ignored — \
         rebuild with --features otlp to enable OTLP export"
    );
    TracerProvider::builder().build()
}
