/// Layer 41 — secondary index persistence across restarts.
///
/// Each test opens an engine, builds one or more secondary indexes, inserts
/// data, then closes the engine by dropping it.  A second engine is opened
/// against the same directory and the index must already be live — no
/// explicit `build_secondary_index` call on the second open.
///
/// Schema: pk | category | score
use tempfile::TempDir;
use vectorvolt_engine::{Durability, EngineOptions, TableEngine};

fn opts() -> EngineOptions {
    EngineOptions::new(1024).with_durability(Durability::WriteOnly)
}

fn open(dir: &TempDir) -> TableEngine {
    TableEngine::open_with(
        vec!["pk".into(), "category".into(), "score".into()],
        dir.path(),
        opts(),
    )
    .unwrap()
}

// ── catalog file absent ───────────────────────────────────────────────────────

#[test]
fn fresh_engine_has_no_secondary_index() {
    let dir = TempDir::new().unwrap();
    let eng = open(&dir);
    assert!(!eng.has_secondary_index(1));
    assert!(!eng.has_secondary_index(2));
}

// ── catalog survives restart ──────────────────────────────────────────────────

#[test]
fn index_rebuilt_after_restart() {
    let dir = TempDir::new().unwrap();
    {
        let eng = open(&dir);
        eng.insert(1, vec![1, 10, 100]).unwrap();
        eng.insert(2, vec![2, 10, 200]).unwrap();
        eng.build_secondary_index(1).unwrap(); // index on category
    }

    let eng2 = open(&dir);
    assert!(eng2.has_secondary_index(1), "index must survive restart");
}

#[test]
fn lookup_works_after_restart() {
    let dir = TempDir::new().unwrap();
    {
        let eng = open(&dir);
        eng.insert(1, vec![1, 5, 100]).unwrap();
        eng.insert(2, vec![2, 5, 200]).unwrap();
        eng.insert(3, vec![3, 9, 300]).unwrap();
        eng.build_secondary_index(1).unwrap();
    }

    let eng2 = open(&dir);
    let mut pks = eng2.secondary_lookup(1, 5).unwrap();
    pks.sort_unstable();
    assert_eq!(pks, vec![1, 2]);
}

#[test]
fn range_lookup_works_after_restart() {
    let dir = TempDir::new().unwrap();
    {
        let eng = open(&dir);
        for i in 1u64..=5 {
            eng.insert(i, vec![i, i * 10, i]).unwrap();
        }
        eng.build_secondary_index(1).unwrap();
    }

    let eng2 = open(&dir);
    let mut pks = eng2.secondary_range_lookup(1, 10, 30).unwrap();
    pks.sort_unstable();
    assert_eq!(pks, vec![1, 2, 3]);
}

// ── multiple indexes ──────────────────────────────────────────────────────────

#[test]
fn two_indexes_both_rebuilt_after_restart() {
    let dir = TempDir::new().unwrap();
    {
        let eng = open(&dir);
        eng.insert(1, vec![1, 3, 77]).unwrap();
        eng.insert(2, vec![2, 3, 88]).unwrap();
        eng.build_secondary_index(1).unwrap();
        eng.build_secondary_index(2).unwrap();
    }

    let eng2 = open(&dir);
    assert!(eng2.has_secondary_index(1));
    assert!(eng2.has_secondary_index(2));
}

#[test]
fn two_indexes_entry_count_correct_after_restart() {
    let dir = TempDir::new().unwrap();
    {
        let eng = open(&dir);
        eng.insert(1, vec![1, 1, 10]).unwrap();
        eng.insert(2, vec![2, 1, 20]).unwrap();
        eng.insert(3, vec![3, 2, 10]).unwrap();
        eng.build_secondary_index(1).unwrap(); // 2 distinct category values
        eng.build_secondary_index(2).unwrap(); // 2 distinct score values
    }

    let eng2 = open(&dir);
    assert_eq!(eng2.secondary_index_entry_count(1), 2);
    assert_eq!(eng2.secondary_index_entry_count(2), 2);
}

// ── drop removes from catalog ─────────────────────────────────────────────────

#[test]
fn dropped_index_not_rebuilt_after_restart() {
    let dir = TempDir::new().unwrap();
    {
        let eng = open(&dir);
        eng.insert(1, vec![1, 7, 99]).unwrap();
        eng.build_secondary_index(1).unwrap();
        eng.build_secondary_index(2).unwrap();
        eng.drop_secondary_index(1); // drop col 1 only
    }

    let eng2 = open(&dir);
    assert!(
        !eng2.has_secondary_index(1),
        "dropped index must not come back"
    );
    assert!(eng2.has_secondary_index(2), "kept index must survive");
}

#[test]
fn drop_all_then_restart_has_no_indexes() {
    let dir = TempDir::new().unwrap();
    {
        let eng = open(&dir);
        eng.insert(1, vec![1, 1, 1]).unwrap();
        eng.build_secondary_index(1).unwrap();
        eng.drop_secondary_index(1);
    }

    let eng2 = open(&dir);
    assert!(!eng2.has_secondary_index(1));
}

// ── rows written before restart are in the rebuilt index ─────────────────────

#[test]
fn rows_written_before_build_appear_in_rebuilt_index() {
    let dir = TempDir::new().unwrap();
    {
        let eng = open(&dir);
        eng.insert(10, vec![10, 42, 0]).unwrap();
        eng.insert(20, vec![20, 42, 0]).unwrap();
        eng.build_secondary_index(1).unwrap();
    }

    let eng2 = open(&dir);
    let mut pks = eng2.secondary_lookup(1, 42).unwrap();
    pks.sort_unstable();
    assert_eq!(pks, vec![10, 20]);
}

// ── rows written AFTER build and BEFORE restart also appear ──────────────────

#[test]
fn rows_written_after_build_appear_in_rebuilt_index() {
    let dir = TempDir::new().unwrap();
    {
        let eng = open(&dir);
        eng.insert(1, vec![1, 5, 0]).unwrap();
        eng.build_secondary_index(1).unwrap();
        eng.insert(2, vec![2, 5, 0]).unwrap(); // written after build; auto-maintained
    }

    // On restart the index is fully rebuilt from all partition data, so pk=2
    // must be present even if auto-maintenance wrote it only in memory.
    let eng2 = open(&dir);
    let mut pks = eng2.secondary_lookup(1, 5).unwrap();
    pks.sort_unstable();
    assert_eq!(pks, vec![1, 2]);
}

// ── deletes are reflected after restart ──────────────────────────────────────

#[test]
fn deleted_row_absent_in_rebuilt_index() {
    let dir = TempDir::new().unwrap();
    {
        let eng = open(&dir);
        eng.insert(1, vec![1, 7, 0]).unwrap();
        eng.insert(2, vec![2, 7, 0]).unwrap();
        eng.build_secondary_index(1).unwrap();
        eng.delete(1).unwrap();
    }

    let eng2 = open(&dir);
    let pks = eng2.secondary_lookup(1, 7).unwrap();
    assert_eq!(
        pks,
        vec![2],
        "deleted pk=1 must not be in the rebuilt index"
    );
}

// ── multiple restarts ─────────────────────────────────────────────────────────

#[test]
fn index_survives_two_restarts() {
    let dir = TempDir::new().unwrap();
    {
        let eng = open(&dir);
        eng.insert(1, vec![1, 3, 0]).unwrap();
        eng.build_secondary_index(1).unwrap();
    }
    {
        let eng = open(&dir);
        assert!(eng.has_secondary_index(1), "first restart");
    }
    {
        let eng = open(&dir);
        assert!(eng.has_secondary_index(1), "second restart");
        let pks = eng.secondary_lookup(1, 3).unwrap();
        assert_eq!(pks, vec![1]);
    }
}

// ── secondary_get works after restart ────────────────────────────────────────

#[test]
fn secondary_get_correct_after_restart() {
    let dir = TempDir::new().unwrap();
    {
        let eng = open(&dir);
        eng.insert(5, vec![5, 99, 42]).unwrap();
        eng.build_secondary_index(1).unwrap();
    }

    let eng2 = open(&dir);
    let rows = eng2.secondary_get(1, 99).unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].pk, 5);
    assert_eq!(rows[0].values[2], 42);
}

// ── rebuild uses current values, not stale partition bytes ───────────────────

#[test]
fn rebuilt_index_reflects_updates_not_stale_values() {
    // pk=1 starts with category=1, updated to category=2, then compacted
    // (single SST) before restart.  The rebuilt index must map value=2 → pk=1.
    let dir = TempDir::new().unwrap();
    {
        let eng = open(&dir);
        eng.insert(1, vec![1, 1, 0]).unwrap(); // category=1
        eng.flush().unwrap();
        eng.update_field(1, 1, 2).unwrap(); // category: 1 → 2
        eng.compact().unwrap(); // merge to single SST so restart sees one version
        eng.build_secondary_index(1).unwrap();
    }

    let eng2 = open(&dir);
    assert_eq!(
        eng2.secondary_lookup(1, 1).unwrap(),
        vec![],
        "stale value=1 must be absent"
    );
    assert_eq!(
        eng2.secondary_lookup(1, 2).unwrap(),
        vec![1],
        "current value=2 must be present"
    );
}

// ── catalog path is inside the data_dir ──────────────────────────────────────

#[test]
fn catalog_file_created_inside_data_dir() {
    let dir = TempDir::new().unwrap();
    {
        let eng = open(&dir);
        eng.insert(1, vec![1, 1, 1]).unwrap();
        eng.build_secondary_index(2).unwrap();
    }

    let catalog = dir.path().join("secondary_index.catalog");
    assert!(catalog.exists(), "catalog file must be in the data dir");
}
