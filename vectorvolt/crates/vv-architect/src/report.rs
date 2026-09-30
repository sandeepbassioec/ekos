//! Human-readable report printer, matching the style sketched in the
//! implementation plan's `vv-architect` example output.

use crate::blueprint::Blueprint;

pub fn render(bp: &Blueprint) -> String {
    let mut out = String::new();
    out.push_str("VectorVolt Architecture Blueprint\n");
    out.push_str("══════════════════════════════════════════════════════\n");

    if let Some(sizing) = &bp.sizing {
        out.push_str(&crate::rules::render(sizing));
        out.push('\n');
    }

    out.push_str(&format!(
        "Entities detected: {} ({} aggregate root(s), {} reference entit(y/ies))\n\n",
        bp.all_entities.len(),
        bp.aggregate_roots.len(),
        bp.reference_entities.len()
    ));

    if bp.aggregate_roots.is_empty() {
        out.push_str("Aggregate roots: none detected\n");
    } else {
        for root in &bp.aggregate_roots {
            if root.owns.is_empty() {
                out.push_str(&format!("  AggregateRoot: {}\n", root.name));
            } else {
                out.push_str(&format!(
                    "  AggregateRoot: {:<12} (owns: {})\n",
                    root.name,
                    root.owns.join(", ")
                ));
            }
        }
    }

    if !bp.reference_entities.is_empty() {
        out.push('\n');
        out.push_str(&format!(
            "Reference data: {}\n",
            bp.reference_entities.join(", ")
        ));
    }

    let unclassified: Vec<&String> = bp
        .all_entities
        .iter()
        .filter(|e| {
            !bp.aggregate_roots.iter().any(|r| &r.name == *e)
                && !bp.aggregate_roots.iter().any(|r| r.owns.contains(e))
                && !bp.reference_entities.contains(e)
        })
        .collect();
    if !unclassified.is_empty() {
        out.push('\n');
        out.push_str(&format!(
            "Unclassified entities (references detected but no clear root/reference role): {}\n",
            unclassified
                .iter()
                .map(|s| s.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::blueprint::AggregateRootEntry;

    #[test]
    fn render_includes_aggregate_root_and_children() {
        let bp = Blueprint {
            aggregate_roots: vec![AggregateRootEntry {
                name: "Order".to_owned(),
                owns: vec!["OrderItem".to_owned(), "Payment".to_owned()],
            }],
            reference_entities: vec!["Product".to_owned()],
            all_entities: vec![
                "Order".to_owned(),
                "OrderItem".to_owned(),
                "Payment".to_owned(),
                "Product".to_owned(),
            ],
            sizing: None,
        };
        let text = render(&bp);
        assert!(text.contains("AggregateRoot: Order"));
        assert!(text.contains("OrderItem, Payment"));
        assert!(text.contains("Reference data: Product"));
    }

    #[test]
    fn render_handles_no_entities_found() {
        let bp = Blueprint::default();
        let text = render(&bp);
        assert!(text.contains("Entities detected: 0"));
        assert!(text.contains("none detected"));
    }

    #[test]
    fn render_includes_sizing_block_before_entities_when_present() {
        use crate::rules::{SizingInputs, compute};
        let bp = Blueprint::default().with_sizing(compute(
            SizingInputs {
                estimated_tps: 1000,
                estimated_data_gb: 10.0,
                replication_factor: 2,
            },
            None,
        ));
        let text = render(&bp);
        let sizing_pos = text.find("Cluster:").unwrap();
        let entities_pos = text.find("Entities detected:").unwrap();
        assert!(sizing_pos < entities_pos);
    }

    #[test]
    fn render_omits_sizing_block_when_absent() {
        let bp = Blueprint::default();
        assert!(!render(&bp).contains("Cluster:"));
    }
}
