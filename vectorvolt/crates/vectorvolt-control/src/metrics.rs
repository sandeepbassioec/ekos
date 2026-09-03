use std::sync::atomic::{AtomicU64, Ordering};

pub struct Metrics {
    pub inserts_total: AtomicU64,
    pub deletes_total: AtomicU64,
    pub queries_total: AtomicU64,
    pub bytes_written: AtomicU64,
    pub active_connections: AtomicU64,
    pub raft_term: AtomicU64,
    pub shard_count: AtomicU64,
}

impl Metrics {
    pub fn new() -> Self {
        Self {
            inserts_total: AtomicU64::new(0),
            deletes_total: AtomicU64::new(0),
            queries_total: AtomicU64::new(0),
            bytes_written: AtomicU64::new(0),
            active_connections: AtomicU64::new(0),
            raft_term: AtomicU64::new(0),
            shard_count: AtomicU64::new(0),
        }
    }

    pub fn to_prometheus_text(&self) -> String {
        let mut out = String::new();
        let pairs: &[(&str, &str, &AtomicU64)] = &[
            (
                "vv_inserts_total",
                "Total insert operations",
                &self.inserts_total,
            ),
            (
                "vv_deletes_total",
                "Total delete operations",
                &self.deletes_total,
            ),
            (
                "vv_queries_total",
                "Total query operations",
                &self.queries_total,
            ),
            (
                "vv_bytes_written",
                "Total bytes written",
                &self.bytes_written,
            ),
            (
                "vv_active_connections",
                "Active connections",
                &self.active_connections,
            ),
            ("vv_raft_term", "Current Raft term", &self.raft_term),
            ("vv_shard_count", "Number of shards", &self.shard_count),
        ];
        for (name, help, counter) in pairs {
            out.push_str(&format!("# HELP {name} {help}\n"));
            out.push_str(&format!("# TYPE {name} counter\n"));
            out.push_str(&format!("{name} {}\n", counter.load(Ordering::Relaxed)));
        }
        out
    }

    pub fn to_json(&self) -> String {
        serde_json::json!({
            "inserts_total": self.inserts_total.load(Ordering::Relaxed),
            "deletes_total": self.deletes_total.load(Ordering::Relaxed),
            "queries_total": self.queries_total.load(Ordering::Relaxed),
            "bytes_written": self.bytes_written.load(Ordering::Relaxed),
            "active_connections": self.active_connections.load(Ordering::Relaxed),
            "raft_term": self.raft_term.load(Ordering::Relaxed),
            "shard_count": self.shard_count.load(Ordering::Relaxed),
        })
        .to_string()
    }
}

impl Default for Metrics {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_zeros() {
        let m = Metrics::new();
        assert_eq!(m.inserts_total.load(Ordering::Relaxed), 0);
        assert_eq!(m.queries_total.load(Ordering::Relaxed), 0);
    }

    #[test]
    fn test_increment_inserts() {
        let m = Metrics::new();
        m.inserts_total.fetch_add(5, Ordering::Relaxed);
        assert_eq!(m.inserts_total.load(Ordering::Relaxed), 5);
    }

    #[test]
    fn test_prometheus_contains_help() {
        let m = Metrics::new();
        let text = m.to_prometheus_text();
        assert!(text.contains("# HELP vv_inserts_total"));
    }

    #[test]
    fn test_prometheus_contains_type() {
        let m = Metrics::new();
        let text = m.to_prometheus_text();
        assert!(text.contains("# TYPE vv_inserts_total counter"));
    }

    #[test]
    fn test_prometheus_all_metrics_present() {
        let m = Metrics::new();
        let text = m.to_prometheus_text();
        for name in &[
            "vv_inserts_total",
            "vv_deletes_total",
            "vv_queries_total",
            "vv_bytes_written",
            "vv_active_connections",
            "vv_raft_term",
            "vv_shard_count",
        ] {
            assert!(text.contains(name), "missing {name}");
        }
    }

    #[test]
    fn test_prometheus_value_reflected() {
        let m = Metrics::new();
        m.raft_term.store(42, Ordering::Relaxed);
        let text = m.to_prometheus_text();
        assert!(text.contains("vv_raft_term 42"));
    }

    #[test]
    fn test_json_valid() {
        let m = Metrics::new();
        let json = m.to_json();
        let v: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(v["inserts_total"], 0);
    }

    #[test]
    fn test_json_reflects_values() {
        let m = Metrics::new();
        m.queries_total.store(99, Ordering::Relaxed);
        let json = m.to_json();
        let v: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(v["queries_total"], 99);
    }

    #[test]
    fn test_active_connections() {
        let m = Metrics::new();
        m.active_connections.fetch_add(3, Ordering::Relaxed);
        assert_eq!(m.active_connections.load(Ordering::Relaxed), 3);
    }

    #[test]
    fn test_shard_count() {
        let m = Metrics::new();
        m.shard_count.store(16, Ordering::Relaxed);
        assert_eq!(m.shard_count.load(Ordering::Relaxed), 16);
    }

    #[test]
    fn test_bytes_written() {
        let m = Metrics::new();
        m.bytes_written.fetch_add(1024, Ordering::Relaxed);
        assert_eq!(m.bytes_written.load(Ordering::Relaxed), 1024);
    }

    #[test]
    fn test_prometheus_zero_values_present() {
        let m = Metrics::new();
        let text = m.to_prometheus_text();
        assert!(text.contains("vv_inserts_total 0"));
    }

    #[test]
    fn test_deletes_total() {
        let m = Metrics::new();
        m.deletes_total.fetch_add(7, Ordering::Relaxed);
        assert_eq!(m.deletes_total.load(Ordering::Relaxed), 7);
    }
}
