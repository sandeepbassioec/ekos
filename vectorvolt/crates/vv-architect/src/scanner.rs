//! Real AST-based entity + foreign-key detector, via tree-sitter.
//!
//! Phase 1 scope: C# and Java (the two languages tree-sitter grammars are
//! wired for so far). Each language's class declarations and field/property
//! declarations are walked from the REAL parse tree — not text/regex
//! heuristics — so nested classes, comments, and string literals containing
//! the word "class" can't produce false positives, and field types come from
//! the grammar's own type nodes rather than guesswork.
//!
//! Adding another language means adding its tree-sitter grammar crate + one
//! `LanguageSupport` entry below; the entity/FK detection logic itself is
//! language-agnostic over the small set of node-kind names each language's
//! `LanguageSupport` declares.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use tree_sitter::{Language, Node, Parser};

/// One detected entity (class) and the field names found inside its body
/// that look like references to other entities.
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

/// Per-language AST node-kind names needed to find classes and their fields.
/// Adding a language = one more entry here (plus its grammar crate).
struct LanguageSupport {
    extensions: &'static [&'static str],
    language: fn() -> Language,
    /// Node kind for a class definition (tree-sitter node "kind" string).
    class_kind: &'static str,
    /// Field of the class node holding its name identifier.
    class_name_field: &'static str,
    /// Field of the class node holding its body block.
    class_body_field: &'static str,
    /// Node kinds inside a class body that declare a field/property.
    field_kinds: &'static [&'static str],
}
// Field-declaration name extraction is per-language (e.g. C#'s
// `field_declaration` nests the name one level deeper inside a
// `variable_declarator`) — see `field_name_of` below rather than a struct field.

fn field_name_of(lang_key: &str, node: Node, src: &[u8]) -> Vec<String> {
    match lang_key {
        "csharp" => {
            // field_declaration -> variable_declaration -> variable_declarator(s) -> name
            // property_declaration -> name directly
            if node.kind() == "property_declaration" {
                node.child_by_field_name("name")
                    .map(|n| text_of(n, src))
                    .into_iter()
                    .collect()
            } else {
                let mut names = Vec::new();
                let mut cursor = node.walk();
                for child in node.children(&mut cursor) {
                    if child.kind() == "variable_declaration" {
                        let mut c2 = child.walk();
                        for decl in child.children(&mut c2) {
                            if decl.kind() == "variable_declarator"
                                && let Some(n) = decl.child_by_field_name("name")
                            {
                                names.push(text_of(n, src));
                            }
                        }
                    }
                }
                names
            }
        }
        "java" => node
            .child_by_field_name("declarator")
            .and_then(|d| d.child_by_field_name("name"))
            .map(|n| text_of(n, src))
            .into_iter()
            .collect(),
        _ => Vec::new(),
    }
}

fn text_of(node: Node, src: &[u8]) -> String {
    node.utf8_text(src).unwrap_or("").to_owned()
}

fn languages() -> Vec<(&'static str, LanguageSupport)> {
    vec![
        (
            "csharp",
            LanguageSupport {
                extensions: &["cs"],
                language: || tree_sitter_c_sharp::LANGUAGE.into(),
                class_kind: "class_declaration",
                class_name_field: "name",
                class_body_field: "body",
                field_kinds: &["field_declaration", "property_declaration"],
            },
        ),
        (
            "java",
            LanguageSupport {
                extensions: &["java"],
                language: || tree_sitter_java::LANGUAGE.into(),
                class_kind: "class_declaration",
                class_name_field: "name",
                class_body_field: "body",
                field_kinds: &["field_declaration"],
            },
        ),
    ]
}

fn to_pascal_case(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        Some(f) => f.to_ascii_uppercase().to_string() + c.as_str(),
        None => String::new(),
    }
}

/// If `field_name` looks like a foreign key (`OrderId`, `orderId`), return the
/// PascalCase entity name it appears to reference (`Order`).
fn fk_target_from_field_name(field_name: &str, owner_entity: &str) -> Option<String> {
    let stripped = field_name
        .strip_suffix("Id")
        .or_else(|| field_name.strip_suffix("ID"))?;
    if stripped.is_empty() {
        return None;
    }
    let target = to_pascal_case(stripped);
    if target == owner_entity {
        None // e.g. `OrderId` as Order's own PK, not a self-FK
    } else {
        Some(target)
    }
}

fn walk_classes(
    lang_key: &str,
    support: &LanguageSupport,
    node: Node,
    src: &[u8],
    out: &mut Vec<DetectedEntity>,
) {
    if node.kind() == support.class_kind
        && let Some(name_node) = node.child_by_field_name(support.class_name_field)
    {
        let name = text_of(name_node, src);
        let mut references = BTreeSet::new();
        if let Some(body) = node.child_by_field_name(support.class_body_field) {
            let mut cursor = body.walk();
            for member in body.children(&mut cursor) {
                if support.field_kinds.contains(&member.kind()) {
                    for field_name in field_name_of(lang_key, member, src) {
                        if let Some(target) = fk_target_from_field_name(&field_name, &name) {
                            references.insert(target);
                        }
                    }
                }
            }
        }
        out.push(DetectedEntity { name, references });
    }
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        walk_classes(lang_key, support, child, src, out);
    }
}

/// Parse a single file's source text with the appropriate grammar (chosen by
/// `lang_key`), returning every class/entity found.
pub fn scan_source(lang_key: &str, text: &str) -> anyhow::Result<Vec<DetectedEntity>> {
    let langs = languages();
    let Some((_, support)) = langs.iter().find(|(k, _)| *k == lang_key) else {
        anyhow::bail!("unsupported language key: {lang_key}");
    };
    let mut parser = Parser::new();
    parser.set_language(&(support.language)())?;
    let tree = parser
        .parse(text, None)
        .ok_or_else(|| anyhow::anyhow!("tree-sitter failed to parse source"))?;
    let mut out = Vec::new();
    walk_classes(
        lang_key,
        support,
        tree.root_node(),
        text.as_bytes(),
        &mut out,
    );
    Ok(out)
}

/// Walk `dir` recursively, parse every file whose extension matches a
/// supported language's grammar, and merge results into one [`ScanResult`].
/// Entities with the same name found in multiple files have their reference
/// sets unioned.
pub fn scan_directory(dir: &Path) -> anyhow::Result<ScanResult> {
    let langs = languages();
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
        let Some((lang_key, _)) = langs.iter().find(|(_, s)| s.extensions.contains(&ext)) else {
            continue;
        };
        let Ok(text) = std::fs::read_to_string(entry.path()) else {
            continue;
        };
        let detected = scan_source(lang_key, &text)?;
        for entity in detected {
            result
                .entities
                .entry(entity.name.clone())
                .and_modify(|e| e.references.extend(entity.references.iter().cloned()))
                .or_insert(entity);
        }
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_csharp_class_via_ast() {
        let src = "public class Order {\n  public int Id;\n}\n";
        let found = scan_source("csharp", src).unwrap();
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].name, "Order");
    }

    #[test]
    fn detects_java_class_via_ast() {
        let src = "public class Customer {\n  private int id;\n}\n";
        let found = scan_source("java", src).unwrap();
        assert_eq!(found[0].name, "Customer");
    }

    #[test]
    fn detects_fk_field_via_real_ast_not_comments_or_strings() {
        let src = r#"
public class Order {
    public int Id;
}

// class OrderFake should NOT be detected — it's a comment
public class OrderItem {
    public int Id;
    public int OrderId; // references Order
    public string Note = "class NotAClass";
}
"#;
        let found = scan_source("csharp", src).unwrap();
        assert_eq!(
            found.len(),
            2,
            "comment and string must not be parsed as classes"
        );
        let order_item = found.iter().find(|e| e.name == "OrderItem").unwrap();
        assert!(order_item.references.contains("Order"));
        let order = found.iter().find(|e| e.name == "Order").unwrap();
        assert!(order.references.is_empty());
    }

    #[test]
    fn detects_csharp_property_declaration_fk() {
        let src = "public class Order {\n  public int Id { get; set; }\n}\n\npublic class Payment {\n  public int OrderId { get; set; }\n}\n";
        let found = scan_source("csharp", src).unwrap();
        let payment = found.iter().find(|e| e.name == "Payment").unwrap();
        assert!(payment.references.contains("Order"));
    }

    #[test]
    fn detects_java_camelcase_fk_field() {
        let src = "public class Payment {\n  private int orderId;\n}\n";
        let found = scan_source("java", src).unwrap();
        assert!(found[0].references.contains("Order"));
    }

    #[test]
    fn self_referencing_id_field_is_not_a_reference() {
        // Order's own `OrderId` (its PK) must not count as a self-FK.
        let src = "public class Order {\n  public int OrderId;\n}\n";
        let found = scan_source("csharp", src).unwrap();
        assert!(found[0].references.is_empty());
    }

    #[test]
    fn nested_class_is_still_detected() {
        // A real AST walk (unlike a text-window heuristic) correctly handles
        // nesting instead of getting confused by brace/indentation depth.
        let src = "public class Outer {\n  public class Inner {\n    public int OuterId;\n  }\n}\n";
        let found = scan_source("csharp", src).unwrap();
        assert_eq!(found.len(), 2);
        assert!(found.iter().any(|e| e.name == "Inner"));
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
        assert!(!refs.contains(&"Order"));
        assert!(!refs.contains(&"Product")); // referenced by OrderItem
    }

    #[test]
    fn scan_directory_merges_entities_across_files_and_languages() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join("Order.cs"),
            "public class Order {\n  public int Id;\n}\n",
        )
        .unwrap();
        std::fs::write(
            dir.path().join("OrderItem.java"),
            "public class OrderItem {\n  private int orderId;\n}\n",
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
