//! Language-agnostic entity + foreign-key detector.
//!
//! Phase 1 scope (honest, not an AST parser): regex-based heuristics over
//! source text, good enough to find "this looks like an entity" and "this
//! field looks like a reference to another entity" across common OOP
//! languages (C#, Java, TypeScript, Python, Go, Rust) without needing a
//! per-language parser. This will produce false positives/negatives on
//! unusual code style — that is an accepted, documented limitation of a
//! text-heuristic scanner, not a claim of full static analysis.

use regex::Regex;
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

/// One detected entity (class/struct) and the field names found "close to"
/// its definition that look like references to other entities.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DetectedEntity {
    pub name: String,
    /// Other entity names this entity appears to hold a foreign key to,
    /// e.g. `OrderItem` has field `OrderId` -> references `Order`.
    pub references: BTreeSet<String>,
}

/// Result of scanning a directory tree.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ScanResult {
    pub entities: BTreeMap<String, DetectedEntity>,
}

impl ScanResult {
    /// Entities that at least one other entity holds a foreign key to, and
    /// which do not themselves reference another entity — i.e. the "owning"
    /// side of an aggregate, per the plan's `AggregateRoot: Order (owns:
    /// OrderItem, Payment)` convention.
    pub fn aggregate_roots(&self) -> Vec<(&str, Vec<&str>)> {
        let mut children_of: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
        for entity in self.entities.values() {
            for referenced in &entity.references {
                if self.entities.contains_key(referenced) {
                    children_of
                        .entry(referenced.as_str())
                        .or_default()
                        .push(entity.name.as_str());
                }
            }
        }
        self.entities
            .values()
            .filter(|e| e.references.is_empty())
            .filter_map(|e| {
                children_of
                    .get(e.name.as_str())
                    .map(|children| (e.name.as_str(), children.clone()))
            })
            .collect()
    }

    /// Entities that are neither referenced by anyone nor reference anyone —
    /// likely lookup/reference data (e.g. `Product`, `Category`).
    pub fn reference_entities(&self) -> Vec<&str> {
        let mut referenced: BTreeSet<&str> = BTreeSet::new();
        for entity in self.entities.values() {
            for r in &entity.references {
                referenced.insert(r.as_str());
            }
        }
        self.entities
            .values()
            .filter(|e| e.references.is_empty() && !referenced.contains(e.name.as_str()))
            .map(|e| e.name.as_str())
            .collect()
    }
}

/// Matches `class Foo`, `struct Foo`, `type Foo struct` (Go), `interface Foo`.
fn entity_def_regex() -> Regex {
    Regex::new(r"(?:class|struct|interface)\s+([A-Z][A-Za-z0-9_]*)").unwrap()
}

/// Matches a field/property declaration whose name ends in `Id`/`_id`/`ID`,
/// e.g. `OrderId`, `order_id`, `customer_id`. Captures the referenced entity
/// name candidate (the part before the Id suffix, PascalCased).
fn fk_field_regex() -> Regex {
    Regex::new(r"\b([A-Za-z][A-Za-z0-9]*?)(?:_id|Id|ID)\b").unwrap()
}

fn to_pascal_case(s: &str) -> String {
    s.split('_')
        .filter(|p| !p.is_empty())
        .map(|part| {
            let mut c = part.chars();
            match c.next() {
                Some(f) => f.to_ascii_uppercase().to_string() + c.as_str(),
                None => String::new(),
            }
        })
        .collect()
}

/// Scan a single source file's text, returning any entity definitions found
/// and, for each, the FK-looking field names in the ~40 lines following the
/// definition (a cheap proxy for "fields declared inside this class/struct"
/// without a real brace-matching parser).
pub fn scan_source(text: &str) -> Vec<DetectedEntity> {
    let entity_re = entity_def_regex();
    let fk_re = fk_field_regex();
    let lines: Vec<&str> = text.lines().collect();

    let mut defs: Vec<(usize, String)> = Vec::new();
    for (i, line) in lines.iter().enumerate() {
        if let Some(cap) = entity_re.captures(line) {
            defs.push((i, cap[1].to_owned()));
        }
    }

    let mut out = Vec::new();
    for (idx, (start, name)) in defs.iter().enumerate() {
        let end = defs
            .get(idx + 1)
            .map(|(next_start, _)| *next_start)
            .unwrap_or(lines.len())
            .min(start + 60);
        let body = lines[*start..end].join("\n");

        let mut references = BTreeSet::new();
        for cap in fk_re.captures_iter(&body) {
            let base = to_pascal_case(&cap[1]);
            // Skip self-references and the entity's own name repeated in its Id
            // field (e.g. `OrderId` inside `Order` itself is a PK, not a FK).
            if !base.is_empty() && base != *name {
                references.insert(base);
            }
        }

        out.push(DetectedEntity {
            name: name.clone(),
            references,
        });
    }
    out
}

/// Walk `dir` recursively, scan every file with a recognized source
/// extension, and merge results into one [`ScanResult`]. Entities with the
/// same name found in multiple files have their reference sets unioned.
pub fn scan_directory(dir: &Path) -> anyhow::Result<ScanResult> {
    const EXTENSIONS: &[&str] = &["cs", "java", "ts", "tsx", "py", "go", "rs"];
    let mut result = ScanResult::default();

    for entry in walkdir::WalkDir::new(dir)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if !entry.file_type().is_file() {
            continue;
        }
        let ext = entry
            .path()
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");
        if !EXTENSIONS.contains(&ext) {
            continue;
        }
        let Ok(text) = std::fs::read_to_string(entry.path()) else {
            continue;
        };
        for detected in scan_source(&text) {
            result
                .entities
                .entry(detected.name.clone())
                .and_modify(|e| e.references.extend(detected.references.iter().cloned()))
                .or_insert(detected);
        }
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_csharp_class() {
        let src = "public class Order {\n  public int Id;\n}\n";
        let found = scan_source(src);
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].name, "Order");
    }

    #[test]
    fn detects_rust_struct() {
        let src = "pub struct Customer {\n  pub id: u64,\n}\n";
        let found = scan_source(src);
        assert_eq!(found[0].name, "Customer");
    }

    #[test]
    fn detects_fk_reference_between_entities() {
        let src =
            "class Order {\n  int Id;\n}\n\nclass OrderItem {\n  int Id;\n  int OrderId;\n}\n";
        let found = scan_source(src);
        let order_item = found.iter().find(|e| e.name == "OrderItem").unwrap();
        assert!(order_item.references.contains("Order"));
        let order = found.iter().find(|e| e.name == "Order").unwrap();
        assert!(order.references.is_empty());
    }

    #[test]
    fn snake_case_fk_field_is_pascal_cased_for_matching() {
        let src = "class Payment {\n  int id;\n  int order_id;\n}\n";
        let found = scan_source(src);
        assert!(found[0].references.contains("Order"));
    }

    #[test]
    fn self_referencing_id_field_is_not_a_reference() {
        // Order's own `OrderId` (its PK) must not count as a self-FK.
        let src = "class Order {\n  int OrderId;\n}\n";
        let found = scan_source(src);
        assert!(found[0].references.is_empty());
    }

    #[test]
    fn aggregate_root_detection_matches_plan_example_shape() {
        let mut result = ScanResult::default();
        result.entities.insert(
            "Order".to_owned(),
            DetectedEntity {
                name: "Order".to_owned(),
                references: BTreeSet::new(),
            },
        );
        result.entities.insert(
            "OrderItem".to_owned(),
            DetectedEntity {
                name: "OrderItem".to_owned(),
                references: BTreeSet::from(["Order".to_owned()]),
            },
        );
        result.entities.insert(
            "Payment".to_owned(),
            DetectedEntity {
                name: "Payment".to_owned(),
                references: BTreeSet::from(["Order".to_owned()]),
            },
        );
        let roots = result.aggregate_roots();
        assert_eq!(roots.len(), 1);
        assert_eq!(roots[0].0, "Order");
        let mut children = roots[0].1.clone();
        children.sort();
        assert_eq!(children, vec!["OrderItem", "Payment"]);
    }

    #[test]
    fn reference_entities_have_no_edges_either_way() {
        let mut result = ScanResult::default();
        result.entities.insert(
            "Product".to_owned(),
            DetectedEntity {
                name: "Product".to_owned(),
                references: BTreeSet::new(),
            },
        );
        result.entities.insert(
            "Order".to_owned(),
            DetectedEntity {
                name: "Order".to_owned(),
                references: BTreeSet::new(),
            },
        );
        result.entities.insert(
            "OrderItem".to_owned(),
            DetectedEntity {
                name: "OrderItem".to_owned(),
                references: BTreeSet::from(["Order".to_owned(), "Product".to_owned()]),
            },
        );
        let refs = result.reference_entities();
        // Product is referenced by OrderItem, so it's NOT a pure reference
        // entity by this definition — only truly isolated entities qualify.
        assert!(!refs.contains(&"Order"));
    }

    #[test]
    fn scan_directory_merges_entities_across_files() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("order.cs"), "class Order {\n  int Id;\n}\n").unwrap();
        std::fs::write(
            dir.path().join("order_item.cs"),
            "class OrderItem {\n  int OrderId;\n}\n",
        )
        .unwrap();
        std::fs::write(dir.path().join("readme.txt"), "not source code").unwrap();

        let result = scan_directory(dir.path()).unwrap();
        assert_eq!(result.entities.len(), 2);
        assert!(result.entities["OrderItem"].references.contains("Order"));
    }

    #[test]
    fn empty_directory_yields_no_entities() {
        let dir = tempfile::tempdir().unwrap();
        let result = scan_directory(dir.path()).unwrap();
        assert!(result.entities.is_empty());
    }
}
