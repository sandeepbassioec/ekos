//! Unit tests for vectorvolt-otel (VV-OBS-001..006).
//! All tests run without a real OTLP collector — noop provider only.

use vectorvolt_otel::{OtelGuard, build_provider, init, layer};

#[test]
fn noop_init_does_not_panic() {
    let guard = init(None, "test-service");
    drop(guard);
}

#[test]
fn noop_provider_builds_without_endpoint() {
    let provider = build_provider(None, "svc");
    drop(provider);
}

#[test]
fn otel_guard_drops_cleanly() {
    let guard = init(None, "drop-test");
    // Explicit drop to check no panics in OtelGuard::drop
    std::mem::drop(guard);
}

#[test]
fn multiple_inits_do_not_conflict() {
    let g1 = init(None, "svc-a");
    drop(g1);
    let g2 = init(None, "svc-b");
    drop(g2);
}

#[test]
fn layer_builds_after_init() {
    let guard = init(None, "layer-test");
    // layer() must not panic even without a real subscriber
    let _l = layer(&guard);
}

#[test]
fn no_otlp_endpoint_is_noop() {
    let provider = build_provider(None, "noop-svc");
    // A noop provider should shut down cleanly (returns ())
    provider.shutdown();
}

#[test]
#[cfg(not(feature = "otlp"))]
fn non_otlp_build_with_endpoint_returns_noop() {
    // Without the `otlp` feature, endpoint is ignored and a noop provider is returned.
    let provider = build_provider(Some("http://localhost:4317"), "svc");
    provider.shutdown();
}

#[test]
fn init_returns_otel_guard_type() {
    fn takes_guard(_g: OtelGuard) {}
    let guard = init(None, "type-check");
    takes_guard(guard);
}
