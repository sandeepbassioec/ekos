/// Layer 28 — group-by aggregate integration tests.
///
/// Schema: pk | score | category | region
/// Tests cover group_by_count, group_by_sum, group_by_min, group_by_max
/// with single and multi-partition data, filter range, error cases, and
/// edge conditions (empty table, inverted range, single group, all same group).
use tempfile::TempDir;
use vectorvolt_engine::{Durability, EngineOptions, TableEngine};

fn open() -> (TableEngine, TempDir) {
    let dir = TempDir::new().unwrap();
    let eng = TableEngine::open_with(
        vec![
            "pk".into(),
            "score".into(),
            "category".into(),
            "region".into(),
        ],
        dir.path(),
        EngineOptions::new(1024).with_durability(Durability::WriteOnly),
    )
    .unwrap();
    (eng, dir)
}

/// Insert 12 rows: pk=1..=12, score=pk*10, category=pk%3 (0/1/2), region=pk%4 (0/1/2/3)
fn seed(eng: &TableEngine) {
    for pk in 1u64..=12 {
        eng.insert(pk, vec![pk, pk * 10, pk % 3, pk % 4]).unwrap();
    }
}

fn find_group(groups: &[(u64, u64)], key: u64) -> Option<u64> {
    groups.iter().find(|&&(g, _)| g == key).map(|&(_, v)| v)
}

// ── group_by_count ────────────────────────────────────────────────────────────

#[test]
fn group_by_count_all_rows_three_categories() {
    let (eng, _dir) = open();
    seed(&eng);
    // filter score in [10, 120] — all 12 rows; group by category (col 2)
    let groups = eng.group_by_count(1, 10, 120, 2).unwrap();
    // category 0: pks 3,6,9,12 → 4; category 1: pks 1,4,7,10 → 4; category 2: pks 2,5,8,11 → 4
    assert_eq!(groups.len(), 3);
    assert_eq!(find_group(&groups, 0), Some(4));
    assert_eq!(find_group(&groups, 1), Some(4));
    assert_eq!(find_group(&groups, 2), Some(4));
}

#[test]
fn group_by_count_results_sorted_ascending_by_group() {
    let (eng, _dir) = open();
    seed(&eng);
    let groups = eng.group_by_count(1, 10, 120, 2).unwrap();
    let keys: Vec<u64> = groups.iter().map(|&(g, _)| g).collect();
    let mut sorted = keys.clone();
    sorted.sort_unstable();
    assert_eq!(keys, sorted, "group keys must be in ascending order");
}

#[test]
fn group_by_count_range_filters_before_grouping() {
    let (eng, _dir) = open();
    seed(&eng);
    // score in [10, 60] → pks 1..=6; category 0: pk3,6 → 2; cat 1: pk1,4 → 2; cat 2: pk2,5 → 2
    let groups = eng.group_by_count(1, 10, 60, 2).unwrap();
    assert_eq!(groups.len(), 3);
    for &(_, count) in &groups {
        assert_eq!(count, 2);
    }
}

#[test]
fn group_by_count_single_group() {
    let (eng, _dir) = open();
    // All rows have category=0
    for pk in 1u64..=5 {
        eng.insert(pk, vec![pk, pk * 10, 0, 0]).unwrap();
    }
    let groups = eng.group_by_count(1, 10, 50, 2).unwrap();
    assert_eq!(groups.len(), 1);
    assert_eq!(find_group(&groups, 0), Some(5));
}

#[test]
fn group_by_count_empty_table_returns_empty() {
    let (eng, _dir) = open();
    let groups = eng.group_by_count(1, 10, 100, 2).unwrap();
    assert!(groups.is_empty());
}

#[test]
fn group_by_count_inverted_range_returns_empty() {
    let (eng, _dir) = open();
    seed(&eng);
    let groups = eng.group_by_count(1, 100, 10, 2).unwrap();
    assert!(groups.is_empty());
}

#[test]
fn group_by_count_bad_filter_col_returns_error() {
    let (eng, _dir) = open();
    assert!(eng.group_by_count(99, 0, 100, 2).is_err());
}

#[test]
fn group_by_count_bad_group_col_returns_error() {
    let (eng, _dir) = open();
    assert!(eng.group_by_count(1, 0, 100, 99).is_err());
}

// ── group_by_sum ──────────────────────────────────────────────────────────────

#[test]
fn group_by_sum_sums_score_per_category() {
    let (eng, _dir) = open();
    seed(&eng);
    // group by category (col 2), sum score (col 1), filter score in [10,120]
    // category 0: pks 3,6,9,12 → scores 30+60+90+120 = 300
    // category 1: pks 1,4,7,10 → scores 10+40+70+100 = 220
    // category 2: pks 2,5,8,11 → scores 20+50+80+110 = 260
    let groups = eng.group_by_sum(1, 10, 120, 2, 1).unwrap();
    assert_eq!(groups.len(), 3);
    assert_eq!(find_group(&groups, 0), Some(300));
    assert_eq!(find_group(&groups, 1), Some(220));
    assert_eq!(find_group(&groups, 2), Some(260));
}

#[test]
fn group_by_sum_range_filters_before_summing() {
    let (eng, _dir) = open();
    seed(&eng);
    // score in [10, 30] → pks 1(score=10),2(20),3(30)
    // category 0: pk3 → 30; category 1: pk1 → 10; category 2: pk2 → 20
    let groups = eng.group_by_sum(1, 10, 30, 2, 1).unwrap();
    assert_eq!(find_group(&groups, 0), Some(30));
    assert_eq!(find_group(&groups, 1), Some(10));
    assert_eq!(find_group(&groups, 2), Some(20));
}

#[test]
fn group_by_sum_agg_col_different_from_filter_and_group() {
    let (eng, _dir) = open();
    seed(&eng);
    // filter score (col 1) in [10,120]; group by region (col 3); sum score (col 1)
    // region 0: pks 4,8,12 → 40+80+120 = 240
    // region 1: pks 1,5,9  → 10+50+90  = 150
    // region 2: pks 2,6,10 → 20+60+100 = 180
    // region 3: pks 3,7,11 → 30+70+110 = 210
    let groups = eng.group_by_sum(1, 10, 120, 3, 1).unwrap();
    assert_eq!(groups.len(), 4);
    assert_eq!(find_group(&groups, 0), Some(240));
    assert_eq!(find_group(&groups, 1), Some(150));
    assert_eq!(find_group(&groups, 2), Some(180));
    assert_eq!(find_group(&groups, 3), Some(210));
}

// ── group_by_min ──────────────────────────────────────────────────────────────

#[test]
fn group_by_min_min_score_per_category() {
    let (eng, _dir) = open();
    seed(&eng);
    // category 0 min score = 30 (pk 3); category 1 min = 10 (pk 1); category 2 min = 20 (pk 2)
    let groups = eng.group_by_min(1, 10, 120, 2, 1).unwrap();
    assert_eq!(find_group(&groups, 0), Some(30));
    assert_eq!(find_group(&groups, 1), Some(10));
    assert_eq!(find_group(&groups, 2), Some(20));
}

#[test]
fn group_by_min_single_row_per_group() {
    let (eng, _dir) = open();
    for pk in 1u64..=3 {
        eng.insert(pk, vec![pk, pk * 100, pk, 0]).unwrap();
    }
    let groups = eng.group_by_min(1, 100, 300, 2, 1).unwrap();
    // each group has exactly one row; min = that row's score
    assert_eq!(find_group(&groups, 1), Some(100));
    assert_eq!(find_group(&groups, 2), Some(200));
    assert_eq!(find_group(&groups, 3), Some(300));
}

// ── group_by_max ──────────────────────────────────────────────────────────────

#[test]
fn group_by_max_max_score_per_category() {
    let (eng, _dir) = open();
    seed(&eng);
    // category 0 max = 120 (pk 12); category 1 max = 100 (pk 10); category 2 max = 110 (pk 11)
    let groups = eng.group_by_max(1, 10, 120, 2, 1).unwrap();
    assert_eq!(find_group(&groups, 0), Some(120));
    assert_eq!(find_group(&groups, 1), Some(100));
    assert_eq!(find_group(&groups, 2), Some(110));
}

#[test]
fn group_by_max_range_filters_before_max() {
    let (eng, _dir) = open();
    seed(&eng);
    // score in [10, 60] → pks 1..=6
    // category 0: pks 3,6 → max = 60; category 1: pks 1,4 → max = 40; category 2: pks 2,5 → max = 50
    let groups = eng.group_by_max(1, 10, 60, 2, 1).unwrap();
    assert_eq!(find_group(&groups, 0), Some(60));
    assert_eq!(find_group(&groups, 1), Some(40));
    assert_eq!(find_group(&groups, 2), Some(50));
}

// ── multi-partition ───────────────────────────────────────────────────────────

#[test]
fn group_by_count_multi_partition_correct() {
    let dir = TempDir::new().unwrap();
    let eng = TableEngine::open_with(
        vec!["pk".into(), "score".into(), "cat".into(), "reg".into()],
        dir.path(),
        EngineOptions::new(4).with_durability(Durability::WriteOnly),
    )
    .unwrap();
    // 20 rows, each partition holds 4 rows
    for pk in 1u64..=20 {
        eng.insert(pk, vec![pk, pk * 5, pk % 2, 0]).unwrap();
    }
    // filter score in [5, 100], group by cat (0 or 1), count
    let groups = eng.group_by_count(1, 5, 100, 2).unwrap();
    assert_eq!(groups.len(), 2);
    // cat 0: pks 2,4,...,20 → 10 rows; cat 1: pks 1,3,...,19 → 10 rows
    assert_eq!(find_group(&groups, 0), Some(10));
    assert_eq!(find_group(&groups, 1), Some(10));
}

#[test]
fn group_by_sum_multi_partition_correct() {
    let dir = TempDir::new().unwrap();
    let eng = TableEngine::open_with(
        vec!["pk".into(), "score".into(), "cat".into(), "reg".into()],
        dir.path(),
        EngineOptions::new(4).with_durability(Durability::WriteOnly),
    )
    .unwrap();
    for pk in 1u64..=20 {
        eng.insert(pk, vec![pk, pk, pk % 2, 0]).unwrap();
    }
    // sum score per category; cat 0: even pks 2+4+...+20 = 110; cat 1: odd pks 1+3+...+19 = 100
    let groups = eng.group_by_sum(1, 1, 20, 2, 1).unwrap();
    assert_eq!(find_group(&groups, 0), Some(110));
    assert_eq!(find_group(&groups, 1), Some(100));
}
