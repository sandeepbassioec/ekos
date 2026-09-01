//! `vectorvolt-sink-sdk` — public SDK for third-party VectorVolt sink adapters (VV-SNK-042).
//!
//! Re-exports the core [`Sink`] trait and supporting types so external crates
//! can implement custom sinks without depending on internal crates.
//!
//! ## Implementing a custom sink
//!
//! ```rust,no_run
//! use vectorvolt_sink_sdk::{Sink, SinkEvent, SinkResult};
//! use async_trait::async_trait;
//!
//! pub struct MySink;
//!
//! #[async_trait]
//! impl Sink for MySink {
//!     fn name(&self) -> &str { "my-sink" }
//!     async fn send(&self, _event: SinkEvent) -> SinkResult<()> { Ok(()) }
//!     async fn flush(&self) -> SinkResult<()> { Ok(()) }
//! }
//! ```

pub use vectorvolt_sink::{Sink, SinkError, SinkEvent, SinkEventKind, SinkRegistry, SinkResult};

// Re-export async-trait so downstream crates don't need to depend on it
// separately.
pub use async_trait::async_trait;

/// A [`Sink`] that writes each event to stdout as a single line.
///
/// Useful as a debugging sink and as a minimal reference implementation.
pub struct StdoutSink {
    name: String,
}

impl StdoutSink {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
        }
    }
}

#[async_trait]
impl Sink for StdoutSink {
    fn name(&self) -> &str {
        &self.name
    }

    async fn send(&self, event: SinkEvent) -> SinkResult<()> {
        let kind = match event.kind {
            SinkEventKind::Insert => "INSERT",
            SinkEventKind::Update => "UPDATE",
            SinkEventKind::Delete => "DELETE",
        };
        println!(
            "[{}] {kind} table={} pk={} values={:?}",
            self.name, event.table, event.pk, event.values
        );
        Ok(())
    }

    async fn flush(&self) -> SinkResult<()> {
        Ok(())
    }
}

// ── tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    fn ev_insert() -> SinkEvent {
        SinkEvent::insert("orders", 1, vec![10, 20])
    }

    fn ev_delete() -> SinkEvent {
        SinkEvent::delete("orders", 1)
    }

    #[tokio::test]
    async fn stdout_sink_send_ok() {
        let s = StdoutSink::new("test");
        s.send(ev_insert()).await.unwrap();
    }

    #[tokio::test]
    async fn stdout_sink_flush_ok() {
        let s = StdoutSink::new("test");
        s.flush().await.unwrap();
    }

    #[test]
    fn stdout_sink_name() {
        let s = StdoutSink::new("my-stdout");
        assert_eq!(s.name(), "my-stdout");
    }

    #[tokio::test]
    async fn stdout_sink_delete_ok() {
        let s = StdoutSink::new("test");
        s.send(ev_delete()).await.unwrap();
    }

    #[tokio::test]
    async fn register_stdout_in_registry() {
        let reg = SinkRegistry::new();
        reg.register(Arc::new(StdoutSink::new("s")));
        assert_eq!(reg.sink_count(), 1);
    }

    #[tokio::test]
    async fn fan_out_via_registry() {
        let reg = SinkRegistry::new();
        reg.register(Arc::new(StdoutSink::new("s1")));
        reg.register(Arc::new(StdoutSink::new("s2")));
        reg.fan_out(ev_insert()).await.unwrap();
    }

    #[tokio::test]
    async fn deregister_works() {
        let reg = SinkRegistry::new();
        reg.register(Arc::new(StdoutSink::new("s")));
        assert!(reg.deregister("s"));
        assert_eq!(reg.sink_count(), 0);
    }

    #[tokio::test]
    async fn sink_event_insert_kind() {
        let ev = SinkEvent::insert("t", 1, vec![2]);
        assert_eq!(ev.kind, SinkEventKind::Insert);
    }

    #[tokio::test]
    async fn sink_event_delete_kind() {
        let ev = SinkEvent::delete("t", 5);
        assert_eq!(ev.kind, SinkEventKind::Delete);
        assert!(ev.values.is_empty());
    }

    #[tokio::test]
    async fn multiple_send_no_error() {
        let s = StdoutSink::new("t");
        for i in 0..5u64 {
            s.send(SinkEvent::insert("t", i, vec![i])).await.unwrap();
        }
    }

    #[tokio::test]
    async fn fan_out_empty_registry_ok() {
        let reg = SinkRegistry::new();
        reg.fan_out(ev_insert()).await.unwrap();
    }

    #[tokio::test]
    async fn sink_names_accessible() {
        let reg = SinkRegistry::new();
        reg.register(Arc::new(StdoutSink::new("z")));
        reg.register(Arc::new(StdoutSink::new("a")));
        let names = reg.sink_names();
        assert_eq!(names, vec!["a", "z"]);
    }
}
