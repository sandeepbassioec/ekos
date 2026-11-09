//! [`SinkRegistry`] — fan-out hub that dispatches events to multiple sinks.

use std::collections::HashMap;
use std::sync::Arc;

use parking_lot::RwLock;

use crate::{
    error::SinkResult,
    sink::{Sink, SinkEvent},
};

/// Thread-safe registry that fans each [`SinkEvent`] to all registered sinks.
pub struct SinkRegistry {
    sinks: RwLock<HashMap<String, Arc<dyn Sink>>>,
}

impl SinkRegistry {
    pub fn new() -> Self {
        Self {
            sinks: RwLock::new(HashMap::new()),
        }
    }

    /// Register a sink.  Replaces any existing sink with the same name.
    pub fn register(&self, sink: Arc<dyn Sink>) {
        let name = sink.name().to_string();
        self.sinks.write().insert(name, sink);
    }

    /// Deregister a sink by name.  Returns `true` if it existed.
    pub fn deregister(&self, name: &str) -> bool {
        self.sinks.write().remove(name).is_some()
    }

    /// Fan one event to every registered sink.  Errors from individual sinks
    /// are collected and returned as the first encountered.
    pub async fn fan_out(&self, event: SinkEvent) -> SinkResult<()> {
        let sinks: Vec<Arc<dyn Sink>> = {
            let guard = self.sinks.read();
            guard.values().cloned().collect()
        };
        for sink in sinks {
            sink.send(event.clone()).await?;
        }
        Ok(())
    }

    /// Flush every registered sink; returns the first error (all are attempted).
    pub async fn flush_all(&self) -> SinkResult<()> {
        let sinks: Vec<Arc<dyn Sink>> = self.sinks.read().values().cloned().collect();
        let mut first = Ok(());
        for sink in sinks {
            if let Err(e) = sink.flush().await
                && first.is_ok()
            {
                first = Err(e);
            }
        }
        first
    }

    pub fn sink_count(&self) -> usize {
        self.sinks.read().len()
    }

    pub fn sink_names(&self) -> Vec<String> {
        let mut names: Vec<String> = self.sinks.read().keys().cloned().collect();
        names.sort();
        names
    }
}

impl Default for SinkRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// ── tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::SinkResult;
    use crate::sink::{Sink, SinkEvent, SinkEventKind};
    use async_trait::async_trait;
    use std::sync::atomic::{AtomicU64, Ordering};

    struct MockSink {
        name: String,
        count: Arc<AtomicU64>,
    }

    impl MockSink {
        fn new(name: &str) -> (Arc<Self>, Arc<AtomicU64>) {
            let count = Arc::new(AtomicU64::new(0));
            (
                Arc::new(Self {
                    name: name.to_string(),
                    count: count.clone(),
                }),
                count,
            )
        }
    }

    #[async_trait]
    impl Sink for MockSink {
        fn name(&self) -> &str {
            &self.name
        }

        async fn send(&self, _event: SinkEvent) -> SinkResult<()> {
            self.count.fetch_add(1, Ordering::Relaxed);
            Ok(())
        }

        async fn flush(&self) -> SinkResult<()> {
            Ok(())
        }
    }

    #[tokio::test]
    async fn new_registry_empty() {
        let reg = SinkRegistry::new();
        assert_eq!(reg.sink_count(), 0);
    }

    #[tokio::test]
    async fn register_adds_sink() {
        let reg = SinkRegistry::new();
        let (sink, _) = MockSink::new("s1");
        reg.register(sink);
        assert_eq!(reg.sink_count(), 1);
    }

    #[tokio::test]
    async fn deregister_removes_sink() {
        let reg = SinkRegistry::new();
        let (sink, _) = MockSink::new("s1");
        reg.register(sink);
        assert!(reg.deregister("s1"));
        assert_eq!(reg.sink_count(), 0);
    }

    #[tokio::test]
    async fn deregister_missing_returns_false() {
        let reg = SinkRegistry::new();
        assert!(!reg.deregister("nope"));
    }

    #[tokio::test]
    async fn fan_out_reaches_all_sinks() {
        let reg = SinkRegistry::new();
        let (s1, c1) = MockSink::new("s1");
        let (s2, c2) = MockSink::new("s2");
        reg.register(s1);
        reg.register(s2);
        let ev = SinkEvent::insert("t", 1, vec![10]);
        reg.fan_out(ev).await.unwrap();
        assert_eq!(c1.load(Ordering::Relaxed), 1);
        assert_eq!(c2.load(Ordering::Relaxed), 1);
    }

    #[tokio::test]
    async fn fan_out_empty_registry_ok() {
        let reg = SinkRegistry::new();
        let ev = SinkEvent::delete("t", 1);
        reg.fan_out(ev).await.unwrap();
    }

    #[tokio::test]
    async fn fan_out_multiple_events() {
        let reg = SinkRegistry::new();
        let (sink, count) = MockSink::new("s");
        reg.register(sink);
        for i in 0..5u64 {
            reg.fan_out(SinkEvent::insert("t", i, vec![i]))
                .await
                .unwrap();
        }
        assert_eq!(count.load(Ordering::Relaxed), 5);
    }

    #[tokio::test]
    async fn sink_names_sorted() {
        let reg = SinkRegistry::new();
        let (sz, _) = MockSink::new("z");
        let (sa, _) = MockSink::new("a");
        reg.register(sz);
        reg.register(sa);
        assert_eq!(reg.sink_names(), vec!["a", "z"]);
    }

    #[tokio::test]
    async fn register_replaces_existing() {
        let reg = SinkRegistry::new();
        let (s1, _) = MockSink::new("s");
        let (s2, c2) = MockSink::new("s");
        reg.register(s1);
        reg.register(s2);
        assert_eq!(reg.sink_count(), 1);
        reg.fan_out(SinkEvent::insert("t", 1, vec![]))
            .await
            .unwrap();
        assert_eq!(c2.load(Ordering::Relaxed), 1);
    }

    #[tokio::test]
    async fn delete_event_delivered() {
        let reg = SinkRegistry::new();
        let (sink, count) = MockSink::new("s");
        reg.register(sink);
        reg.fan_out(SinkEvent::delete("t", 99)).await.unwrap();
        assert_eq!(count.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn sink_event_insert_kind() {
        let ev = SinkEvent::insert("t", 1, vec![2]);
        assert_eq!(ev.kind, SinkEventKind::Insert);
    }

    #[test]
    fn sink_event_delete_kind() {
        let ev = SinkEvent::delete("t", 1);
        assert_eq!(ev.kind, SinkEventKind::Delete);
        assert!(ev.values.is_empty());
    }

    #[test]
    fn default_registry_is_empty() {
        let reg: SinkRegistry = Default::default();
        assert_eq!(reg.sink_count(), 0);
    }
}
