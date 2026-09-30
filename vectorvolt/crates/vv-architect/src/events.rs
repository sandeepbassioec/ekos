//! Domain event candidate detector — `method names containing Created/
//! Updated/Deleted/Placed/Shipped -> event candidates` per the plan.
//!
//! Honest scope: this matches method names against a curated whitelist of
//! common domain verbs (case-insensitive exact match), not a generic
//! English past-tense algorithm — naive suffix rules ("+ed") get real verbs
//! wrong (`ship` -> `shiped` instead of `shipped`, `cancel` -> `canceled`
//! instead of `cancelled`). An explicit lookup table is correct for the
//! verbs it lists and silent (no event suggested, not a wrong one) for verbs
//! it doesn't — extend the table as real-world use turns up more verbs.

use std::collections::BTreeMap;

use crate::scanner::DetectedEntity;

/// (lowercase verb, PascalCase past-tense form). Extend freely — this list
/// exists to be grown, not to be exhaustive on day one.
const DOMAIN_VERBS: &[(&str, &str)] = &[
    ("create", "Created"),
    ("update", "Updated"),
    ("delete", "Deleted"),
    ("place", "Placed"),
    ("ship", "Shipped"),
    ("cancel", "Cancelled"),
    ("process", "Processed"),
    ("refund", "Refunded"),
    ("approve", "Approved"),
    ("reject", "Rejected"),
    ("complete", "Completed"),
    ("start", "Started"),
    ("submit", "Submitted"),
    ("confirm", "Confirmed"),
    ("close", "Closed"),
    ("open", "Opened"),
    ("assign", "Assigned"),
    ("remove", "Removed"),
    ("add", "Added"),
    ("publish", "Published"),
    ("archive", "Archived"),
];

/// Strip a common prefix like `On`/`Handle`/`Do` that some codebases put in
/// front of the actual verb (`OnShip`, `HandlePlace`) before matching.
fn strip_common_prefix(method_name: &str) -> &str {
    for prefix in ["On", "Handle", "Do"] {
        if let Some(rest) = method_name.strip_prefix(prefix)
            && rest.chars().next().is_some_and(char::is_uppercase)
        {
            return rest;
        }
    }
    method_name
}

/// For every entity, check its methods against the domain-verb whitelist and
/// produce `{Entity}{PastTense}` candidates (e.g. `Order` + `Place()` ->
/// `OrderPlaced`). Returns a sorted, deduplicated list.
pub fn detect_events(entities: &BTreeMap<String, DetectedEntity>) -> Vec<String> {
    let mut events = Vec::new();
    for entity in entities.values() {
        for method in &entity.methods {
            let candidate = strip_common_prefix(method).to_lowercase();
            if let Some((_, past_tense)) = DOMAIN_VERBS.iter().find(|(verb, _)| *verb == candidate)
            {
                events.push(format!("{}{}", entity.name, past_tense));
            }
        }
    }
    events.sort();
    events.dedup();
    events
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    fn entity(name: &str, methods: &[&str]) -> DetectedEntity {
        DetectedEntity {
            name: name.to_owned(),
            references: BTreeSet::new(),
            methods: methods.iter().map(|s| s.to_string()).collect(),
        }
    }

    #[test]
    fn matches_plan_example_events_exactly() {
        let mut entities = BTreeMap::new();
        entities.insert(
            "Order".to_owned(),
            entity("Order", &["Place", "Ship", "Cancel"]),
        );
        entities.insert(
            "Payment".to_owned(),
            entity("Payment", &["Process", "Refund"]),
        );
        let events = detect_events(&entities);
        assert_eq!(
            events,
            vec![
                "OrderCancelled".to_owned(),
                "OrderPlaced".to_owned(),
                "OrderShipped".to_owned(),
                "PaymentProcessed".to_owned(),
                "PaymentRefunded".to_owned(),
            ]
        );
    }

    #[test]
    fn case_insensitive_method_names_still_match() {
        let mut entities = BTreeMap::new();
        entities.insert("order".to_owned(), entity("order", &["ship", "SHIP"]));
        let events = detect_events(&entities);
        // Both variants map to the same event; dedup collapses them.
        assert_eq!(events, vec!["orderShipped".to_owned()]);
    }

    #[test]
    fn unlisted_verb_produces_no_event_not_a_wrong_one() {
        let mut entities = BTreeMap::new();
        entities.insert(
            "Order".to_owned(),
            entity("Order", &["Recalculate", "ToString", "GetHashCode"]),
        );
        assert!(detect_events(&entities).is_empty());
    }

    #[test]
    fn strips_on_and_handle_prefixes() {
        let mut entities = BTreeMap::new();
        entities.insert(
            "Order".to_owned(),
            entity("Order", &["OnShip", "HandlePlace"]),
        );
        let events = detect_events(&entities);
        assert!(events.contains(&"OrderShipped".to_owned()));
        assert!(events.contains(&"OrderPlaced".to_owned()));
    }

    #[test]
    fn empty_entities_yields_no_events() {
        assert!(detect_events(&BTreeMap::new()).is_empty());
    }

    #[test]
    fn events_are_sorted_and_deduplicated_across_entities() {
        let mut entities = BTreeMap::new();
        entities.insert("Order".to_owned(), entity("Order", &["Create", "Create"]));
        let events = detect_events(&entities);
        assert_eq!(events, vec!["OrderCreated".to_owned()]);
    }
}
