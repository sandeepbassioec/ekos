//! Unit tests for vectorvolt-otel (VV-OBS-001..006).
//! All tests run without a real OTLP collector — noop provider only.

use vectorvolt_otel::{OtelError, OtelGuard, build_provider, init, layer, try_build_provider};

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
    let _ = provider.shutdown();
}

#[test]
#[cfg(not(feature = "otlp"))]
fn non_otlp_build_with_endpoint_returns_noop() {
    // Without the `otlp` feature, endpoint is ignored and a noop provider is returned.
    let provider = build_provider(Some("http://localhost:4317"), "svc");
    let _ = provider.shutdown();
}

#[test]
fn init_returns_otel_guard_type() {
    fn takes_guard(_g: OtelGuard) {}
    let guard = init(None, "type-check");
    takes_guard(guard);
}

// ── PP-69 / B31: a bad endpoint must never panic ────────────────────────────

#[test]
fn bad_endpoint_is_an_error_not_a_panic() {
    for ep in [
        "not a url",
        "localhost:4317",
        "ftp://collector:4317",
        "http://",
        "http://:4317",
        "http://collector:notaport",
        "http://collector:99999",
    ] {
        match try_build_provider(Some(ep), "svc") {
            Err(OtelError::InvalidEndpoint { endpoint, .. }) => assert_eq!(endpoint, ep),
            Err(e) => panic!("{ep}: unexpected error {e}"),
            Ok(_) => panic!("{ep}: must be rejected"),
        }
    }
}

#[test]
fn bad_endpoint_falls_back_to_noop_in_build_provider_and_init() {
    let provider = build_provider(Some("not a url"), "svc");
    let _ = provider.shutdown();
    let guard = init(Some("ftp://nope"), "svc");
    let _l = layer(&guard);
    drop(guard);
}

#[test]
fn well_formed_endpoints_are_accepted() {
    for ep in [
        "http://localhost:4317",
        "https://collector.example.com",
        "http://10.0.0.1:4317/v1/traces",
        "http://[::1]:4317",
    ] {
        // Without the `otlp` feature this is the no-op provider; with it and
        // no runtime it is `NoRuntime` — never an endpoint error, never a panic.
        match try_build_provider(Some(ep), "svc") {
            Ok(p) => {
                let _ = p.shutdown();
            }
            Err(OtelError::NoRuntime) => {}
            Err(e) => panic!("{ep}: unexpected error {e}"),
        }
    }
}
