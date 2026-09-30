//! `vectorvolt-blueprint.toml` — the machine-readable output of a scan.
//!
//! Entities + aggregate roots + reference (lookup) entities detected by a
//! scanner, plus domain event candidates (method-name based) and an
//! optional cluster sizing plan (rule engine, present only when the caller
//! supplied workload estimates — TPS/data volume can't be inferred from
//! static analysis). Projection detection is a later phase — this struct has
//! room to grow but does not fabricate fields it can't back with real
//! analysis yet.

use serde::Serialize;

use crate::events;
use crate::rules::SizingPlan;
use crate::scanner::ScanResult;

#[derive(Debug, Clone, Serialize)]
pub struct AggregateRootEntry {
    pub name: String,
    pub owns: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct Blueprint {
    pub aggregate_roots: Vec<AggregateRootEntry>,
    pub reference_entities: Vec<String>,
    /// Every entity name detected, regardless of role — useful for a human
    /// reviewing what the scanner actually found vs. missed.
    pub all_entities: Vec<String>,
    /// Domain event candidates suggested from method names (e.g. `Order`'s
    /// `Place()` method -> `OrderPlaced`). See `events.rs`.
    pub suggested_events: Vec<String>,
    /// Cluster sizing, only present when the caller passed `--tps`/`--data-gb`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sizing: Option<SizingPlan>,
}

impl Blueprint {
    pub fn from_scan(scan: &ScanResult) -> Self {
        let mut aggregate_roots: Vec<AggregateRootEntry> = scan
            .aggregate_roots()
            .into_iter()
            .map(|(name, mut owns)| {
                owns.sort();
                AggregateRootEntry {
                    name: name.to_owned(),
                    owns: owns.into_iter().map(str::to_owned).collect(),
                }
            })
            .collect();
        aggregate_roots.sort_by(|a, b| a.name.cmp(&b.name));

        let mut reference_entities: Vec<String> = scan
            .reference_entities()
            .into_iter()
            .map(str::to_owned)
            .collect();
        reference_entities.sort();

        let all_entities: Vec<String> = scan.entities.keys().cloned().collect();
        let suggested_events = events::detect_events(&scan.entities);

        Self {
            aggregate_roots,
            reference_entities,
            suggested_events,
            all_entities,
            sizing: None,
        }
    }

    /// The first aggregate root's name, lowercased + `_id` — a reasonable
    /// guess at a tenant/customer-scoping partition key when one exists.
    /// Returns `None` if no aggregate root was detected (the rule engine
    /// then falls back to "primary key HASH").
    pub fn suggested_partition_key(&self) -> Option<String> {
        self.aggregate_roots
            .first()
            .map(|r| format!("{}_id", r.name.to_lowercase()))
    }

    pub fn with_sizing(mut self, sizing: SizingPlan) -> Self {
        self.sizing = Some(sizing);
        self
    }

    pub fn to_toml(&self) -> anyhow::Result<String> {
        Ok(toml::to_string_pretty(self)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scanner::DetectedEntity;
    use std::collections::BTreeSet;

    #[test]
    fn blueprint_from_scan_matches_plan_example_shape() {
        let mut scan = ScanResult::default();
        scan.entities.insert(
            "Order".to_owned(),
            DetectedEntity {
                name: "Order".to_owned(),
                references: BTreeSet::new(),
                methods: BTreeSet::new(),
            },
        );
        scan.entities.insert(
            "OrderItem".to_owned(),
            DetectedEntity {
                name: "OrderItem".to_owned(),
                references: BTreeSet::from(["Order".to_owned()]),
                methods: BTreeSet::new(),
            },
        );
        let bp = Blueprint::from_scan(&scan);
        assert_eq!(bp.aggregate_roots.len(), 1);
        assert_eq!(bp.aggregate_roots[0].name, "Order");
        assert_eq!(bp.aggregate_roots[0].owns, vec!["OrderItem".to_owned()]);
        assert_eq!(bp.suggested_partition_key().as_deref(), Some("order_id"));
    }

    #[test]
    fn suggested_partition_key_is_none_without_aggregate_roots() {
        let bp = Blueprint::default();
        assert_eq!(bp.suggested_partition_key(), None);
    }

    #[test]
    fn with_sizing_attaches_plan_and_serializes_it() {
        use crate::rules::{SizingInputs, compute};
        let bp = Blueprint::default().with_sizing(compute(
            SizingInputs {
                estimated_tps: 1000,
                estimated_data_gb: 10.0,
                replication_factor: 2,
            },
            Some("order_id".to_owned()),
        ));
        assert!(bp.sizing.is_some());
        let toml = bp.to_toml().unwrap();
        assert!(toml.contains("node_count"));
    }

    #[test]
    fn to_toml_round_trips_parseable() {
        let bp = Blueprint {
            aggregate_roots: vec![AggregateRootEntry {
                name: "Order".to_owned(),
                owns: vec!["OrderItem".to_owned()],
            }],
            reference_entities: vec!["Product".to_owned()],
            all_entities: vec![
                "Order".to_owned(),
                "OrderItem".to_owned(),
                "Product".to_owned(),
            ],
            suggested_events: vec![],
            sizing: None,
        };
        let s = bp.to_toml().unwrap();
        assert!(s.contains("Order"));
        let _: toml::Value = toml::from_str(&s).expect("valid TOML");
    }
}
