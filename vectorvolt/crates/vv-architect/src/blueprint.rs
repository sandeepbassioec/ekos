//! `vectorvolt-blueprint.toml` — the machine-readable output of a scan.
//!
//! Phase 1 scope: entities + aggregate roots + reference (lookup) entities
//! detected by the code scanner. Cluster/CPU/RAM/shard sizing heuristics
//! (the rule engine) and event/projection detection are later phases — this
//! struct has room to grow but does not fabricate fields it can't back with
//! real analysis yet.

use serde::Serialize;

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

        Self {
            aggregate_roots,
            reference_entities,
            all_entities,
        }
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
            },
        );
        scan.entities.insert(
            "OrderItem".to_owned(),
            DetectedEntity {
                name: "OrderItem".to_owned(),
                references: BTreeSet::from(["Order".to_owned()]),
            },
        );
        let bp = Blueprint::from_scan(&scan);
        assert_eq!(bp.aggregate_roots.len(), 1);
        assert_eq!(bp.aggregate_roots[0].name, "Order");
        assert_eq!(bp.aggregate_roots[0].owns, vec!["OrderItem".to_owned()]);
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
        };
        let s = bp.to_toml().unwrap();
        assert!(s.contains("Order"));
        let _: toml::Value = toml::from_str(&s).expect("valid TOML");
    }
}
