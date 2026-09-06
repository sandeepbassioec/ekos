//! DomainEventEmitter — post-commit hook (VV-EVT-002).

use std::collections::HashMap;
use std::sync::Arc;

use rhai::{Engine, Scope};
use tracing::warn;
use vectorvolt_txn::Mutation;

use crate::config::{DomainEventSchema, TriggerKind};
use crate::envelope::DomainEventEnvelope;
use crate::field::resolve_fields;
use crate::sink::EventSink;

/// Fires after every committed WAL entry, maps Mutation → typed DomainEventEnvelope,
/// evaluates `when:` filter, and dispatches to registered sinks.
pub struct DomainEventEmitter {
    entity: String,
    schemas: Vec<DomainEventSchema>,
    sinks: HashMap<String, Arc<dyn EventSink>>,
    filter_engine: Engine,
}

impl DomainEventEmitter {
    pub fn new(entity: impl Into<String>, schemas: Vec<DomainEventSchema>) -> Self {
        let mut filter_engine = Engine::new();
        filter_engine.set_max_operations(10_000);
        Self {
            entity: entity.into(),
            schemas,
            sinks: HashMap::new(),
            filter_engine,
        }
    }

    /// Register a named sink (name matches `EventRouteConfig::sink` serialised value).
    pub fn register_sink(&mut self, name: impl Into<String>, sink: Arc<dyn EventSink>) {
        self.sinks.insert(name.into(), sink);
    }

    /// Called after a committed mutation. Evaluates all matching schemas and dispatches.
    pub fn on_commit<F>(&self, mutation: &Mutation, lookup_fn: &F)
    where
        F: Fn(&str, u64) -> Option<Vec<u64>>,
    {
        let (pk, values, trigger) = match mutation {
            Mutation::Insert { pk, values } => (*pk, values.as_slice(), TriggerKind::Insert),
            Mutation::Upsert { pk, values } => (*pk, values.as_slice(), TriggerKind::Update),
            Mutation::Delete { pk } => (*pk, [].as_slice(), TriggerKind::Delete),
            _ => return,
        };

        for schema in &self.schemas {
            if schema.trigger != trigger {
                continue;
            }
            if !self.eval_when_filter(schema, pk, values) {
                continue;
            }
            let payload = resolve_fields(&schema.fields, pk, values, lookup_fn);
            let envelope =
                DomainEventEnvelope::new(&schema.name, &schema.version, &self.entity, pk, payload);
            self.dispatch(schema, envelope);
        }
    }

    fn eval_when_filter(&self, schema: &DomainEventSchema, pk: u64, values: &[u64]) -> bool {
        let Some(expr) = &schema.when else {
            return true;
        };
        let mut scope = Scope::new();
        scope.push("pk", pk as i64);
        let arr: rhai::Array = values
            .iter()
            .map(|&v| rhai::Dynamic::from(v as i64))
            .collect();
        scope.push("values", arr);
        match self
            .filter_engine
            .eval_expression_with_scope::<bool>(&mut scope, expr)
        {
            Ok(v) => v,
            Err(e) => {
                warn!("event filter eval error [{}]: {e}", schema.name);
                false
            }
        }
    }

    fn dispatch(&self, schema: &DomainEventSchema, envelope: DomainEventEnvelope) {
        for route in &schema.routes {
            let sink_key = serde_json::to_value(&route.sink)
                .ok()
                .and_then(|v| v.as_str().map(str::to_string))
                .unwrap_or_default();
            if let Some(sink) = self.sinks.get(&sink_key)
                && let Err(e) = sink.send(&envelope)
            {
                warn!("sink [{sink_key}] failed for event [{}]: {e}", schema.name);
            }
        }
    }

    pub fn entity(&self) -> &str {
        &self.entity
    }

    pub fn schema_count(&self) -> usize {
        self.schemas.len()
    }
}
