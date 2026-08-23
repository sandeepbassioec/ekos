//! Integration tests for L115: `vvctl catalog watch` (VV-CTL-011).

use std::time::Duration;

use tempfile::TempDir;
use vectorvolt_engine::{
    ColumnDef, SchemaRegistry, ShardCatalog, TableSchema, WatchControl, WatchEvent, watch_catalog,
};
use vvctl::commands::catalog::{CatalogArgs, CatalogCommand, run};

fn make_dir() -> TempDir {
    TempDir::new().unwrap()
}

fn args(dir: &TempDir, cmd: CatalogCommand) -> CatalogArgs {
    CatalogArgs {
        dir: dir.path().to_path_buf(),
        command: cmd,
    }
}

fn create_args(dir: &TempDir, name: &str) -> CatalogArgs {
    args(
        dir,
        CatalogCommand::CreateTable {
            name: name.to_string(),
            columns: vec!["pk".to_string(), "val".to_string()],
            flush_threshold: 1024,
            partitions: 1,
        },
    )
}

fn watch_n(dir: &TempDir, count: usize) -> CatalogArgs {
    args(
        dir,
        CatalogCommand::Watch {
            interval: 0,
            count,
            verbose: false,
        },
    )
}

fn watch_verbose(dir: &TempDir, count: usize) -> CatalogArgs {
    args(
        dir,
        CatalogCommand::Watch {
            interval: 0,
            count,
            verbose: true,
        },
    )
}

fn reg_path(dir: &TempDir) -> std::path::PathBuf {
    dir.path().join("registry")
}

fn add_schema(dir: &TempDir, name: &str) {
    let reg = SchemaRegistry::new(reg_path(dir));
    let schema = TableSchema::new(vec![ColumnDef::u64("pk"), ColumnDef::u64("val")]).unwrap();
    reg.save(name, &schema).unwrap();
}

// ── CLI integration ───────────────────────────────────────────────────────────

#[test]
fn watch_count_1_exits_after_initial() {
    let dir = make_dir();
    run(create_args(&dir, "orders")).unwrap();
    run(watch_n(&dir, 1)).unwrap();
}

#[test]
fn watch_count_2_exits_after_two_polls() {
    let dir = make_dir();
    run(create_args(&dir, "orders")).unwrap();
    run(watch_n(&dir, 2)).unwrap();
}

#[test]
fn watch_empty_catalog_count_1_exits() {
    let dir = make_dir();
    run(watch_n(&dir, 1)).unwrap();
}

#[test]
fn watch_verbose_count_2_exits() {
    let dir = make_dir();
    run(create_args(&dir, "t")).unwrap();
    run(watch_verbose(&dir, 2)).unwrap();
}

// ── engine API — WatchEvent kinds ────────────────────────────────────────────

#[test]
fn first_event_is_initial() {
    let dir = make_dir();
    add_schema(&dir, "orders");
    let mut saw_initial = false;
    watch_catalog(reg_path(&dir), Duration::ZERO, Some(1), |ev| {
        if let WatchEvent::Initial { .. } = ev {
            saw_initial = true;
        }
        WatchControl::Stop
    })
    .unwrap();
    assert!(saw_initial);
}

#[test]
fn second_event_unchanged_when_no_mutation() {
    let dir = make_dir();
    add_schema(&dir, "orders");
    let mut events: Vec<String> = Vec::new();
    watch_catalog(reg_path(&dir), Duration::ZERO, Some(2), |ev| {
        events.push(match &ev {
            WatchEvent::Initial { .. } => "initial".into(),
            WatchEvent::Unchanged => "unchanged".into(),
            WatchEvent::Changed { .. } => "changed".into(),
        });
        WatchControl::Continue
    })
    .unwrap();
    assert_eq!(events, vec!["initial", "unchanged"]);
}

#[test]
fn changed_event_emitted_after_table_added() {
    let dir = make_dir();
    add_schema(&dir, "orders");
    let mut events: Vec<String> = Vec::new();
    let mut call = 0usize;
    watch_catalog(reg_path(&dir), Duration::ZERO, Some(3), |ev| {
        call += 1;
        if call == 2 {
            // Add a table between poll 1 and poll 2.
            add_schema(&dir, "shipments");
        }
        events.push(match &ev {
            WatchEvent::Initial { .. } => "initial".into(),
            WatchEvent::Unchanged => "unchanged".into(),
            WatchEvent::Changed { .. } => "changed".into(),
        });
        WatchControl::Continue
    })
    .unwrap();
    assert!(events.contains(&"changed".to_string()));
}

#[test]
fn stop_control_halts_after_one_event() {
    let dir = make_dir();
    add_schema(&dir, "t");
    let mut count = 0;
    watch_catalog(reg_path(&dir), Duration::ZERO, None, |_| {
        count += 1;
        WatchControl::Stop
    })
    .unwrap();
    assert_eq!(count, 1);
}

#[test]
fn max_iterations_limits_total_callbacks() {
    let dir = make_dir();
    add_schema(&dir, "t");
    let mut count = 0;
    watch_catalog(reg_path(&dir), Duration::ZERO, Some(4), |_| {
        count += 1;
        WatchControl::Continue
    })
    .unwrap();
    assert_eq!(count, 4);
}

#[test]
fn initial_export_contains_all_tables() {
    let dir = make_dir();
    add_schema(&dir, "a");
    add_schema(&dir, "b");
    add_schema(&dir, "c");
    let mut table_count = 0;
    watch_catalog(reg_path(&dir), Duration::ZERO, Some(1), |ev| {
        if let WatchEvent::Initial { export } = ev {
            table_count = export.tables.len();
        }
        WatchControl::Stop
    })
    .unwrap();
    assert_eq!(table_count, 3);
}

// ── changed event content ─────────────────────────────────────────────────────

#[test]
fn changed_event_diff_has_added_table() {
    let dir = make_dir();
    add_schema(&dir, "orders");
    let mut added_count: Option<usize> = None;
    let mut call = 0usize;
    watch_catalog(reg_path(&dir), Duration::ZERO, Some(3), |ev| {
        call += 1;
        if call == 2 {
            add_schema(&dir, "shipments");
        }
        if let WatchEvent::Changed { diff, .. } = ev {
            added_count = Some(diff.added.len());
            return WatchControl::Stop;
        }
        WatchControl::Continue
    })
    .unwrap();
    assert_eq!(added_count, Some(1));
}

#[test]
fn changed_event_current_export_reflects_new_state() {
    let dir = make_dir();
    add_schema(&dir, "orders");
    let mut current_tables: Option<usize> = None;
    let mut call = 0usize;
    watch_catalog(reg_path(&dir), Duration::ZERO, Some(3), |ev| {
        call += 1;
        if call == 2 {
            add_schema(&dir, "shipments");
        }
        if let WatchEvent::Changed { current, .. } = ev {
            current_tables = Some(current.tables.len());
            return WatchControl::Stop;
        }
        WatchControl::Continue
    })
    .unwrap();
    assert_eq!(current_tables, Some(2));
}

// ── CLI+engine combined ────────────────────────────────────────────────────────

#[test]
fn watch_after_drop_table_shows_changed() {
    let dir = make_dir();
    // Create two tables via ShardCatalog (writes catalog dir + registry).
    let cat = dir.path().join("catalog");
    let reg = dir.path().join("registry");
    let catalog = ShardCatalog::with_registry(&cat, &reg).unwrap();
    let schema = TableSchema::new(vec![ColumnDef::u64("pk"), ColumnDef::u64("v")]).unwrap();
    catalog
        .create_table("orders", schema.clone(), 1024, 1)
        .unwrap();
    catalog.create_table("returns", schema, 1024, 1).unwrap();

    // Poll 1 = initial (2 tables), then drop "returns", poll 2 = changed.
    let mut events: Vec<String> = Vec::new();
    let mut call = 0usize;
    watch_catalog(&reg, Duration::ZERO, Some(3), |ev| {
        call += 1;
        if call == 1 {
            // Drop "returns" registry entry to simulate drop.
            let r = SchemaRegistry::new(&reg);
            r.remove("returns").ok();
        }
        events.push(match &ev {
            WatchEvent::Initial { .. } => "initial".into(),
            WatchEvent::Unchanged => "unchanged".into(),
            WatchEvent::Changed { .. } => "changed".into(),
        });
        WatchControl::Continue
    })
    .unwrap();
    assert!(events.iter().any(|e| e == "changed"));
}
