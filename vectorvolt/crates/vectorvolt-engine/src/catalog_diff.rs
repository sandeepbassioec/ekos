//! Catalog schema diff (VV-CTL-007).
//!
//! [`diff_catalogs`] compares two [`CatalogExport`] snapshots and returns a
//! [`CatalogDiff`] describing what was added, removed, or changed between them.
//! Column-level changes are tracked per table.

use crate::catalog_export::{CatalogExport, ExportedTable};

// ── Result types ──────────────────────────────────────────────────────────────

/// Column-level change inside a single table.
#[derive(Debug, Clone, PartialEq)]
pub struct ColumnChange {
    pub index: usize,
    pub kind: ColumnChangeKind,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ColumnChangeKind {
    /// Column present in left but absent in right (dropped).
    Removed { name: String, col_type: String },
    /// Column present in right but absent in left (added).
    Added { name: String, col_type: String },
    /// Column present in both but type changed.
    TypeChanged {
        name: String,
        from: String,
        to: String,
    },
    /// Column present in both but name changed (same position).
    Renamed {
        from: String,
        to: String,
        col_type: String,
    },
}

/// Per-table diff when a table exists in both exports but columns differ.
#[derive(Debug, Clone, PartialEq)]
pub struct TableDiff {
    pub name: String,
    pub changes: Vec<ColumnChange>,
}

/// Top-level diff result returned by [`diff_catalogs`].
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CatalogDiff {
    /// Tables present in `right` but not in `left`.
    pub added: Vec<ExportedTable>,
    /// Tables present in `left` but not in `right`.
    pub removed: Vec<ExportedTable>,
    /// Tables present in both but with column-level changes.
    pub changed: Vec<TableDiff>,
}

impl CatalogDiff {
    /// `true` when both exports are identical.
    pub fn is_empty(&self) -> bool {
        self.added.is_empty() && self.removed.is_empty() && self.changed.is_empty()
    }

    /// Total count of changed tables (added + removed + modified).
    pub fn change_count(&self) -> usize {
        self.added.len() + self.removed.len() + self.changed.len()
    }
}

// ── Diff logic ────────────────────────────────────────────────────────────────

/// Compare `left` and `right` and return a [`CatalogDiff`].
///
/// Tables are matched by name.  Within a shared table, columns are compared
/// positionally: same index, name check then type check.  Extra/missing
/// columns at the end are reported as Added/Removed.
pub fn diff_catalogs(left: &CatalogExport, right: &CatalogExport) -> CatalogDiff {
    use std::collections::HashMap;

    let left_map: HashMap<&str, &ExportedTable> =
        left.tables.iter().map(|t| (t.name.as_str(), t)).collect();
    let right_map: HashMap<&str, &ExportedTable> =
        right.tables.iter().map(|t| (t.name.as_str(), t)).collect();

    let mut diff = CatalogDiff::default();

    // Added: in right but not left.
    let mut added: Vec<ExportedTable> = right
        .tables
        .iter()
        .filter(|t| !left_map.contains_key(t.name.as_str()))
        .cloned()
        .collect();
    added.sort_by(|a, b| a.name.cmp(&b.name));
    diff.added = added;

    // Removed: in left but not right.
    let mut removed: Vec<ExportedTable> = left
        .tables
        .iter()
        .filter(|t| !right_map.contains_key(t.name.as_str()))
        .cloned()
        .collect();
    removed.sort_by(|a, b| a.name.cmp(&b.name));
    diff.removed = removed;

    // Changed: in both — compare columns positionally.
    let mut shared_names: Vec<&str> = left_map
        .keys()
        .filter(|&&n| right_map.contains_key(n))
        .copied()
        .collect();
    shared_names.sort();

    for name in shared_names {
        let l_cols = &left_map[name].columns;
        let r_cols = &right_map[name].columns;
        let max_len = l_cols.len().max(r_cols.len());
        let mut changes: Vec<ColumnChange> = Vec::new();

        for i in 0..max_len {
            match (l_cols.get(i), r_cols.get(i)) {
                (Some(lc), Some(rc)) => {
                    let (l_name, l_type) = split_col(lc);
                    let (r_name, r_type) = split_col(rc);
                    if l_name != r_name && l_type == r_type {
                        changes.push(ColumnChange {
                            index: i,
                            kind: ColumnChangeKind::Renamed {
                                from: l_name.to_owned(),
                                to: r_name.to_owned(),
                                col_type: l_type.to_owned(),
                            },
                        });
                    } else if l_name == r_name && l_type != r_type {
                        changes.push(ColumnChange {
                            index: i,
                            kind: ColumnChangeKind::TypeChanged {
                                name: l_name.to_owned(),
                                from: l_type.to_owned(),
                                to: r_type.to_owned(),
                            },
                        });
                    } else if l_name != r_name || l_type != r_type {
                        // Both name and type differ — report as remove + add.
                        changes.push(ColumnChange {
                            index: i,
                            kind: ColumnChangeKind::Removed {
                                name: l_name.to_owned(),
                                col_type: l_type.to_owned(),
                            },
                        });
                        changes.push(ColumnChange {
                            index: i,
                            kind: ColumnChangeKind::Added {
                                name: r_name.to_owned(),
                                col_type: r_type.to_owned(),
                            },
                        });
                    }
                }
                (Some(lc), None) => {
                    let (n, t) = split_col(lc);
                    changes.push(ColumnChange {
                        index: i,
                        kind: ColumnChangeKind::Removed {
                            name: n.to_owned(),
                            col_type: t.to_owned(),
                        },
                    });
                }
                (None, Some(rc)) => {
                    let (n, t) = split_col(rc);
                    changes.push(ColumnChange {
                        index: i,
                        kind: ColumnChangeKind::Added {
                            name: n.to_owned(),
                            col_type: t.to_owned(),
                        },
                    });
                }
                (None, None) => {}
            }
        }

        if !changes.is_empty() {
            diff.changed.push(TableDiff {
                name: name.to_owned(),
                changes,
            });
        }
    }

    diff.changed.sort_by(|a, b| a.name.cmp(&b.name));
    diff
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn split_col(s: &str) -> (&str, &str) {
    if let Some(pos) = s.find(':') {
        (&s[..pos], &s[pos + 1..])
    } else {
        (s, "u64")
    }
}

// ── Unit tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog_export::ExportedTable;

    fn export(tables: &[(&str, &[&str])]) -> CatalogExport {
        CatalogExport {
            version: 1,
            tables: tables
                .iter()
                .map(|(name, cols)| ExportedTable {
                    name: name.to_string(),
                    columns: cols.iter().map(|c| c.to_string()).collect(),
                })
                .collect(),
        }
    }

    #[test]
    fn identical_exports_produce_empty_diff() {
        let e = export(&[("orders", &["pk:u64", "val:f64"])]);
        let d = diff_catalogs(&e, &e);
        assert!(d.is_empty());
    }

    #[test]
    fn added_table_detected() {
        let left = export(&[("a", &["pk:u64"])]);
        let right = export(&[("a", &["pk:u64"]), ("b", &["pk:u64"])]);
        let d = diff_catalogs(&left, &right);
        assert_eq!(d.added.len(), 1);
        assert_eq!(d.added[0].name, "b");
        assert!(d.removed.is_empty());
        assert!(d.changed.is_empty());
    }

    #[test]
    fn removed_table_detected() {
        let left = export(&[("a", &["pk:u64"]), ("b", &["pk:u64"])]);
        let right = export(&[("a", &["pk:u64"])]);
        let d = diff_catalogs(&left, &right);
        assert_eq!(d.removed.len(), 1);
        assert_eq!(d.removed[0].name, "b");
    }

    #[test]
    fn column_type_change_detected() {
        let left = export(&[("t", &["pk:u64", "val:u64"])]);
        let right = export(&[("t", &["pk:u64", "val:f64"])]);
        let d = diff_catalogs(&left, &right);
        assert_eq!(d.changed.len(), 1);
        assert!(matches!(
            &d.changed[0].changes[0].kind,
            ColumnChangeKind::TypeChanged { from, to, .. } if from == "u64" && to == "f64"
        ));
    }

    #[test]
    fn column_added_detected() {
        let left = export(&[("t", &["pk:u64"])]);
        let right = export(&[("t", &["pk:u64", "extra:f64"])]);
        let d = diff_catalogs(&left, &right);
        assert_eq!(d.changed.len(), 1);
        assert!(matches!(
            &d.changed[0].changes[0].kind,
            ColumnChangeKind::Added { name, .. } if name == "extra"
        ));
    }

    #[test]
    fn column_removed_detected() {
        let left = export(&[("t", &["pk:u64", "extra:f64"])]);
        let right = export(&[("t", &["pk:u64"])]);
        let d = diff_catalogs(&left, &right);
        assert_eq!(d.changed.len(), 1);
        assert!(matches!(
            &d.changed[0].changes[0].kind,
            ColumnChangeKind::Removed { name, .. } if name == "extra"
        ));
    }

    #[test]
    fn column_renamed_detected() {
        let left = export(&[("t", &["pk:u64", "old_name:f64"])]);
        let right = export(&[("t", &["pk:u64", "new_name:f64"])]);
        let d = diff_catalogs(&left, &right);
        assert_eq!(d.changed.len(), 1);
        assert!(matches!(
            &d.changed[0].changes[0].kind,
            ColumnChangeKind::Renamed { from, to, .. } if from == "old_name" && to == "new_name"
        ));
    }

    #[test]
    fn change_count_sums_all_categories() {
        let left = export(&[("a", &["pk:u64"]), ("b", &["pk:u64", "v:u64"])]);
        let right = export(&[("b", &["pk:u64", "v:f64"]), ("c", &["pk:u64"])]);
        let d = diff_catalogs(&left, &right);
        assert_eq!(d.removed.len(), 1); // "a"
        assert_eq!(d.added.len(), 1); // "c"
        assert_eq!(d.changed.len(), 1); // "b" type change
        assert_eq!(d.change_count(), 3);
    }
}
