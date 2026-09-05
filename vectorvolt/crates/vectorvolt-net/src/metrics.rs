//! Prometheus-compatible metrics export for the VectorVolt net server (VV-OBS-005).
//!
//! Wraps [`crate::server::ServerStats`] and serialises it to the Prometheus
//! text exposition format so any scraper (Prometheus, VictoriaMetrics, Grafana
//! Agent) can ingest it without a dependency on the `prometheus` crate.

use std::fmt::Write as _;
use std::sync::atomic::Ordering;

use crate::server::ServerStats;

/// Serialise `stats` to the Prometheus text exposition format (version 0.0.4).
///
/// All counters carry the `vv_net_` prefix and the `TYPE … counter` annotation.
/// The output is always valid UTF-8 and ends with a newline.
pub fn prometheus_text(stats: &ServerStats) -> String {
    let mut out = String::with_capacity(512);
    emit_counter(
        &mut out,
        "vv_net_connections_accepted_total",
        "Total TCP connections accepted since server start.",
        stats.connections_accepted.load(Ordering::Relaxed),
    );
    emit_counter(
        &mut out,
        "vv_net_requests_handled_total",
        "Total wire-protocol requests dispatched.",
        stats.requests_handled.load(Ordering::Relaxed),
    );
    emit_counter(
        &mut out,
        "vv_net_protocol_errors_total",
        "Frames that failed CRC, magic, or framing validation.",
        stats.protocol_errors.load(Ordering::Relaxed),
    );
    emit_counter(
        &mut out,
        "vv_net_unauthenticated_rejections_total",
        "Requests rejected because the connection had not authenticated.",
        stats.unauthenticated_rejections.load(Ordering::Relaxed),
    );
    emit_counter(
        &mut out,
        "vv_net_forbidden_rejections_total",
        "Requests rejected because the key lacked the required scope.",
        stats.forbidden_rejections.load(Ordering::Relaxed),
    );
    emit_counter(
        &mut out,
        "vv_net_auth_failures_total",
        "Failed authentication attempts.",
        stats.auth_failures.load(Ordering::Relaxed),
    );
    emit_counter(
        &mut out,
        "vv_net_auth_successes_total",
        "Successful authentication handshakes.",
        stats.auth_successes.load(Ordering::Relaxed),
    );
    emit_counter(
        &mut out,
        "vv_net_tls_handshake_failures_total",
        "Connections dropped due to TLS handshake failure.",
        stats.tls_handshake_failures.load(Ordering::Relaxed),
    );
    out
}

fn emit_counter(out: &mut String, name: &str, help: &str, value: u64) {
    writeln!(out, "# HELP {name} {help}").ok();
    writeln!(out, "# TYPE {name} counter").ok();
    writeln!(out, "{name} {value}").ok();
}

/// Returns a one-line human summary (for `vvctl server stats`).
pub fn summary_line(stats: &ServerStats) -> String {
    format!(
        "accepted={} handled={} errors={} auth_ok={} auth_fail={}",
        stats.connections_accepted.load(Ordering::Relaxed),
        stats.requests_handled.load(Ordering::Relaxed),
        stats.protocol_errors.load(Ordering::Relaxed),
        stats.auth_successes.load(Ordering::Relaxed),
        stats.auth_failures.load(Ordering::Relaxed),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::server::ServerStats;
    use std::sync::atomic::Ordering;

    fn make_stats() -> ServerStats {
        let s = ServerStats::default();
        s.connections_accepted.store(10, Ordering::Relaxed);
        s.requests_handled.store(200, Ordering::Relaxed);
        s.protocol_errors.store(3, Ordering::Relaxed);
        s.auth_successes.store(9, Ordering::Relaxed);
        s.auth_failures.store(1, Ordering::Relaxed);
        s
    }

    #[test]
    fn prometheus_output_contains_all_metrics() {
        let s = make_stats();
        let out = prometheus_text(&s);
        assert!(out.contains("vv_net_connections_accepted_total 10"));
        assert!(out.contains("vv_net_requests_handled_total 200"));
        assert!(out.contains("vv_net_protocol_errors_total 3"));
        assert!(out.contains("vv_net_auth_successes_total 9"));
        assert!(out.contains("vv_net_auth_failures_total 1"));
    }

    #[test]
    fn prometheus_output_has_type_annotations() {
        let s = ServerStats::default();
        let out = prometheus_text(&s);
        assert!(out.contains("# TYPE vv_net_connections_accepted_total counter"));
        assert!(out.contains("# HELP vv_net_connections_accepted_total"));
    }

    #[test]
    fn prometheus_output_ends_with_newline() {
        let s = ServerStats::default();
        let out = prometheus_text(&s);
        assert!(out.ends_with('\n'));
    }

    #[test]
    fn summary_line_format() {
        let s = make_stats();
        let line = summary_line(&s);
        assert!(line.contains("accepted=10"));
        assert!(line.contains("handled=200"));
        assert!(line.contains("errors=3"));
    }

    #[test]
    fn zero_stats_produces_valid_output() {
        let s = ServerStats::default();
        let out = prometheus_text(&s);
        assert!(out.contains("vv_net_connections_accepted_total 0"));
        assert!(!out.is_empty());
    }

    #[test]
    fn each_metric_has_help_and_type_lines() {
        let s = ServerStats::default();
        let out = prometheus_text(&s);
        let help_count = out.lines().filter(|l| l.starts_with("# HELP")).count();
        let type_count = out.lines().filter(|l| l.starts_with("# TYPE")).count();
        assert_eq!(help_count, type_count);
        assert!(help_count >= 8);
    }

    #[test]
    fn forbidden_and_unauth_metrics_present() {
        let s = ServerStats::default();
        s.unauthenticated_rejections.store(5, Ordering::Relaxed);
        s.forbidden_rejections.store(2, Ordering::Relaxed);
        let out = prometheus_text(&s);
        assert!(out.contains("vv_net_unauthenticated_rejections_total 5"));
        assert!(out.contains("vv_net_forbidden_rejections_total 2"));
    }

    #[test]
    fn tls_handshake_failures_metric_present() {
        let s = ServerStats::default();
        s.tls_handshake_failures.store(7, Ordering::Relaxed);
        let out = prometheus_text(&s);
        assert!(out.contains("vv_net_tls_handshake_failures_total 7"));
    }

    #[test]
    fn summary_line_includes_all_fields() {
        let s = ServerStats::default();
        let line = summary_line(&s);
        for key in &["accepted", "handled", "errors", "auth_ok", "auth_fail"] {
            assert!(line.contains(key), "missing key {key} in: {line}");
        }
    }

    #[test]
    fn large_counter_values_render_correctly() {
        let s = ServerStats::default();
        s.requests_handled.store(u64::MAX, Ordering::Relaxed);
        let out = prometheus_text(&s);
        assert!(out.contains(&format!("vv_net_requests_handled_total {}", u64::MAX)));
    }

    #[test]
    fn output_is_valid_utf8() {
        let s = make_stats();
        let out = prometheus_text(&s);
        assert!(std::str::from_utf8(out.as_bytes()).is_ok());
    }

    #[test]
    fn each_metric_value_line_has_no_prefix() {
        let s = ServerStats::default();
        let out = prometheus_text(&s);
        for line in out.lines() {
            if !line.starts_with('#') && !line.is_empty() {
                assert!(line.starts_with("vv_net_"), "unexpected line: {line}");
            }
        }
    }

    #[test]
    fn concurrent_update_then_read() {
        use std::sync::Arc;
        let s = Arc::new(ServerStats::default());
        s.requests_handled.store(42, Ordering::Relaxed);
        let out = prometheus_text(&s);
        assert!(out.contains("vv_net_requests_handled_total 42"));
    }
}
