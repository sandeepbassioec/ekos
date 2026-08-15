/// Layer 37 — cursor-based pagination integration tests.
///
/// Schema: pk | category | score | flags
use tempfile::TempDir;
use vectorvolt_engine::{Durability, EngineOptions, TableEngine};

fn open() -> (TableEngine, TempDir) {
    let dir = TempDir::new().unwrap();
    let eng = TableEngine::open_with(
        vec![
            "pk".into(),
            "category".into(),
            "score".into(),
            "flags".into(),
        ],
        dir.path(),
        EngineOptions::new(1024).with_durability(Durability::WriteOnly),
    )
    .unwrap();
    (eng, dir)
}

fn pks(rows: &[vectorvolt_engine::EngineRow]) -> Vec<u64> {
    rows.iter().map(|r| r.pk).collect()
}

// ── basic pagination ──────────────────────────────────────────────────────────

#[test]
fn first_page_returns_limit_rows_ascending() {
    let (eng, _dir) = open();
    for i in 1u64..=10 {
        eng.insert(i, vec![i, 1, i * 10, 0]).unwrap();
    }
    eng.flush().unwrap();

    let page = eng.scan_page(1, 1, 1, 0, 3).unwrap();
    assert_eq!(pks(&page.rows), vec![1, 2, 3]);
    assert_eq!(page.next_cursor, Some(3));
}

#[test]
fn second_page_uses_cursor() {
    let (eng, _dir) = open();
    for i in 1u64..=10 {
        eng.insert(i, vec![i, 1, 0, 0]).unwrap();
    }
    eng.flush().unwrap();

    let page1 = eng.scan_page(1, 1, 1, 0, 4).unwrap();
    assert_eq!(pks(&page1.rows), vec![1, 2, 3, 4]);
    let cursor = page1.next_cursor.unwrap();

    let page2 = eng.scan_page(1, 1, 1, cursor, 4).unwrap();
    assert_eq!(pks(&page2.rows), vec![5, 6, 7, 8]);
    assert_eq!(page2.next_cursor, Some(8));
}

#[test]
fn last_page_has_no_next_cursor() {
    let (eng, _dir) = open();
    for i in 1u64..=5 {
        eng.insert(i, vec![i, 1, 0, 0]).unwrap();
    }
    eng.flush().unwrap();

    let page = eng.scan_page(1, 1, 1, 3, 10).unwrap();
    assert_eq!(pks(&page.rows), vec![4, 5]);
    assert_eq!(page.next_cursor, None);
}

#[test]
fn full_iteration_covers_all_rows() {
    let (eng, _dir) = open();
    for i in 1u64..=9 {
        eng.insert(i, vec![i, 1, 0, 0]).unwrap();
    }
    eng.flush().unwrap();

    let mut all: Vec<u64> = Vec::new();
    let mut cursor: u64 = 0;
    loop {
        let page = eng.scan_page(1, 1, 1, cursor, 3).unwrap();
        all.extend(pks(&page.rows));
        match page.next_cursor {
            None => break,
            Some(c) => cursor = c,
        }
    }
    assert_eq!(all, vec![1, 2, 3, 4, 5, 6, 7, 8, 9]);
}

// ── edge cases ────────────────────────────────────────────────────────────────

#[test]
fn limit_zero_returns_empty_no_cursor() {
    let (eng, _dir) = open();
    eng.insert(1, vec![1, 1, 0, 0]).unwrap();
    eng.flush().unwrap();

    let page = eng.scan_page(1, 1, 1, 0, 0).unwrap();
    assert!(page.rows.is_empty());
    assert_eq!(page.next_cursor, None);
}

#[test]
fn cursor_past_all_rows_returns_empty() {
    let (eng, _dir) = open();
    for i in 1u64..=5 {
        eng.insert(i, vec![i, 1, 0, 0]).unwrap();
    }
    eng.flush().unwrap();

    let page = eng.scan_page(1, 1, 1, 100, 10).unwrap();
    assert!(page.rows.is_empty());
    assert_eq!(page.next_cursor, None);
}

#[test]
fn exact_limit_match_has_no_cursor() {
    let (eng, _dir) = open();
    for i in 1u64..=3 {
        eng.insert(i, vec![i, 1, 0, 0]).unwrap();
    }
    eng.flush().unwrap();

    let page = eng.scan_page(1, 1, 1, 0, 3).unwrap();
    assert_eq!(pks(&page.rows), vec![1, 2, 3]);
    assert_eq!(page.next_cursor, None);
}

#[test]
fn empty_table_returns_empty_no_cursor() {
    let (eng, _dir) = open();

    let page = eng.scan_page(1, 1, 1, 0, 5).unwrap();
    assert!(page.rows.is_empty());
    assert_eq!(page.next_cursor, None);
}

#[test]
fn bad_filter_col_returns_error() {
    let (eng, _dir) = open();
    eng.insert(1, vec![1, 1, 0, 0]).unwrap();
    eng.flush().unwrap();

    assert!(eng.scan_page(99, 1, 1, 0, 5).is_err());
}

// ── filter interaction ────────────────────────────────────────────────────────

#[test]
fn filter_applied_before_pagination() {
    let (eng, _dir) = open();
    // pks 1-6 cat=1; pks 7-10 cat=2
    for i in 1u64..=6 {
        eng.insert(i, vec![i, 1, 0, 0]).unwrap();
    }
    for i in 7u64..=10 {
        eng.insert(i, vec![i, 2, 0, 0]).unwrap();
    }
    eng.flush().unwrap();

    // Page cat=1 only, limit 4 — should see pks 1-4, cursor 4
    let page = eng.scan_page(1, 1, 1, 0, 4).unwrap();
    assert_eq!(pks(&page.rows), vec![1, 2, 3, 4]);
    assert_eq!(page.next_cursor, Some(4));

    // Second page cat=1, after cursor 4 — pks 5, 6 only, no more
    let page2 = eng.scan_page(1, 1, 1, 4, 4).unwrap();
    assert_eq!(pks(&page2.rows), vec![5, 6]);
    assert_eq!(page2.next_cursor, None);
}

#[test]
fn cursor_is_exclusive_not_inclusive() {
    let (eng, _dir) = open();
    for i in 1u64..=5 {
        eng.insert(i, vec![i, 1, 0, 0]).unwrap();
    }
    eng.flush().unwrap();

    // After cursor=3, we should get pks 4, 5 (not 3).
    let page = eng.scan_page(1, 1, 1, 3, 10).unwrap();
    assert_eq!(pks(&page.rows), vec![4, 5]);
}

// ── dedup across partition files ──────────────────────────────────────────────

#[test]
fn updated_row_appears_once_in_page() {
    let (eng, _dir) = open();
    eng.insert(1, vec![1, 1, 0, 0]).unwrap();
    eng.flush().unwrap();

    // Update pk=1 — old partition retains physical bytes until compaction.
    eng.update_field(1, 2, 99).unwrap();

    let page = eng.scan_page(1, 1, 1, 0, 10).unwrap();
    assert_eq!(page.rows.len(), 1, "pk=1 must appear exactly once");
    assert_eq!(page.rows[0].pk, 1);
    assert_eq!(page.rows[0].values[2], 99, "updated value should win");
}

#[test]
fn multiple_flushed_updates_deduplicated_correctly() {
    let (eng, _dir) = open();
    for i in 1u64..=4 {
        eng.insert(i, vec![i, 1, 0, 0]).unwrap();
    }
    eng.flush().unwrap();

    // Re-insert pk=2 and pk=3 to create stale partition bytes.
    eng.update_field(2, 2, 20).unwrap();
    eng.update_field(3, 2, 30).unwrap();

    let page = eng.scan_page(1, 1, 1, 0, 10).unwrap();
    // Must see exactly 4 unique pks.
    assert_eq!(page.rows.len(), 4);
    assert_eq!(pks(&page.rows), vec![1, 2, 3, 4]);
}

// ── row values are correct ────────────────────────────────────────────────────

#[test]
fn row_values_are_fully_materialised() {
    let (eng, _dir) = open();
    eng.insert(5, vec![5, 3, 42, 7]).unwrap();
    eng.flush().unwrap();

    let page = eng.scan_page(1, 3, 3, 0, 1).unwrap();
    assert_eq!(page.rows.len(), 1);
    let row = &page.rows[0];
    assert_eq!(row.pk, 5);
    assert_eq!(row.values, vec![5, 3, 42, 7]);
}

#[test]
fn single_row_table_paginates_in_one_page() {
    let (eng, _dir) = open();
    eng.insert(42, vec![42, 1, 100, 0]).unwrap();
    eng.flush().unwrap();

    let page = eng.scan_page(1, 1, 1, 0, 10).unwrap();
    assert_eq!(pks(&page.rows), vec![42]);
    assert_eq!(page.next_cursor, None);
}
