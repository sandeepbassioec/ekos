//! Layer 76 — Index SIMD (VV-IDX-007..011)
//!
//! Covers: ZoneVec overlapping_pids, zone_overlap_indices HAL kernel,
//! CuckooFilter::contains_batch, CuckooMap::get_batch.

use vectorvolt_hal::simd;
use vectorvolt_index::{
    cuckoo_filter::CuckooFilter,
    cuckoo_map::CuckooMap,
    row_descriptor::RowDescriptor,
    zone_map::{ZoneMap, ZoneVec},
};

// ── ZoneVec tests ─────────────────────────────────────────────────────────────

#[test]
fn zone_vec_empty_returns_empty() {
    let v = ZoneVec::new();
    assert!(v.overlapping_pids(0, u64::MAX).is_empty());
}

#[test]
fn zone_vec_all_overlap_wide_query() {
    let mut v = ZoneVec::new();
    for i in 0u32..8 {
        v.push(i, i as u64 * 10, i as u64 * 10 + 5);
    }
    let mut hits = v.overlapping_pids(0, u64::MAX);
    hits.sort_unstable();
    assert_eq!(hits, (0u32..8).collect::<Vec<_>>());
}

#[test]
fn zone_vec_none_overlap_outside_query() {
    let mut v = ZoneVec::new();
    v.push(1, 100, 200);
    v.push(2, 300, 400);
    assert!(v.overlapping_pids(500, 600).is_empty());
    assert!(v.overlapping_pids(50, 99).is_empty());
}

#[test]
fn zone_vec_partial_overlap() {
    let mut v = ZoneVec::new();
    v.push(10, 0, 50);
    v.push(20, 100, 200);
    v.push(30, 40, 80);
    let mut hits = v.overlapping_pids(60, 110);
    hits.sort_unstable();
    // pid 10: [0,50] vs [60,110] — no overlap
    // pid 20: [100,200] vs [60,110] — overlap
    // pid 30: [40,80] vs [60,110] — overlap
    assert_eq!(hits, vec![20, 30]);
}

#[test]
fn zone_vec_matches_zone_map() {
    let mut zm = ZoneMap::new();
    zm.set(0, 0, 100);
    zm.set(1, 200, 300);
    zm.set(2, 50, 150);
    zm.set(3, 500, 600);

    let v = ZoneVec::from_zone_map(&zm);
    for (lo, hi) in [(80u64, 120), (0, 10), (250, 250), (1000, 2000)] {
        let mut zm_hits: Vec<u16> = zm.overlapping_partitions(lo, hi).collect();
        zm_hits.sort_unstable();

        let mut zv_hits: Vec<u32> = v.overlapping_pids(lo, hi);
        zv_hits.sort_unstable();

        let zm_u32: Vec<u32> = zm_hits.iter().map(|&x| x as u32).collect();
        assert_eq!(
            zv_hits, zm_u32,
            "ZoneVec and ZoneMap disagree for [{lo},{hi}]"
        );
    }
}

#[test]
fn zone_vec_simd_matches_scalar_on_many_zones() {
    let mut mins = Vec::new();
    let mut maxes = Vec::new();
    let mut pids = Vec::new();

    for i in 0u32..100 {
        let base = i as u64 * 1000;
        mins.push(base);
        maxes.push(base + 500);
        pids.push(i);
    }

    let lo = 25_000u64;
    let hi = 35_999u64;

    // Scalar oracle via the HAL kernel directly.
    let mut scalar_out: Vec<u32> = Vec::new();
    simd::scalar::zone_overlap_indices(&mins, &maxes, lo, hi, &mut scalar_out);

    // Dispatcher (may be AVX2 or scalar depending on CPU).
    let mut simd_out: Vec<u32> = Vec::new();
    simd::zone_overlap_indices(&mins, &maxes, lo, hi, &mut simd_out);

    assert_eq!(simd_out, scalar_out);
    // Sanity: we should have some hits.
    assert!(!simd_out.is_empty());
}

#[test]
fn zone_vec_handles_u64_max_boundary() {
    let mut v = ZoneVec::new();
    v.push(1, u64::MAX - 10, u64::MAX);
    v.push(2, 0, 10);

    let hits = v.overlapping_pids(u64::MAX - 5, u64::MAX);
    assert_eq!(hits, vec![1]);

    let hits2 = v.overlapping_pids(0, 0);
    assert_eq!(hits2, vec![2]);
}

#[test]
fn zone_overlap_indices_scalar_correctness() {
    let mins = vec![0u64, 100, 200, 300, 400];
    let maxes = vec![50u64, 150, 250, 350, 450];

    let mut out = Vec::new();
    simd::scalar::zone_overlap_indices(&mins, &maxes, 120, 220, &mut out);
    // [0,50] → no, [100,150] → yes (120<=150 && 150>=120), [200,250] → yes, [300,350] → no, [400,450] → no
    assert_eq!(out, vec![1, 2]);
}

#[test]
fn zone_overlap_indices_simd_matches_scalar_adversarial() {
    // Values around i64::MAX/u64::MAX stress-test the FLIP XOR trick.
    let mins = vec![0u64, i64::MAX as u64, u64::MAX - 100, 42];
    let maxes = vec![i64::MAX as u64, u64::MAX, u64::MAX, 42];

    let lo = i64::MAX as u64;
    let hi = u64::MAX;

    let mut scalar_out = Vec::new();
    simd::scalar::zone_overlap_indices(&mins, &maxes, lo, hi, &mut scalar_out);

    let mut dispatcher_out = Vec::new();
    simd::zone_overlap_indices(&mins, &maxes, lo, hi, &mut dispatcher_out);

    assert_eq!(dispatcher_out, scalar_out);
}

/// Verify dispatcher and scalar agree at every length from 0..=20, covering all
/// possible tail-loop sizes relative to the AVX2 LANES=4 boundary.
#[test]
fn zone_overlap_indices_all_lengths_0_to_20_agree() {
    for n in 0usize..=20 {
        let mins: Vec<u64> = (0..n).map(|i| i as u64 * 100).collect();
        let maxes: Vec<u64> = (0..n).map(|i| i as u64 * 100 + 50).collect();
        let lo = 250u64;
        let hi = 650u64;

        let mut sc = Vec::new();
        simd::scalar::zone_overlap_indices(&mins, &maxes, lo, hi, &mut sc);
        let mut disp = Vec::new();
        simd::zone_overlap_indices(&mins, &maxes, lo, hi, &mut disp);
        assert_eq!(disp, sc, "disagreement at length {n}");
    }
}

// ── CuckooFilter::contains_batch tests ────────────────────────────────────────

#[test]
fn filter_contains_batch_agrees_with_contains() {
    let mut f = CuckooFilter::with_capacity(256);
    let inserted: Vec<u64> = (0..50).map(|i| i * 7).collect();
    for &k in &inserted {
        f.insert(k).unwrap();
    }

    let probe: Vec<u64> = (0..200).collect();
    let batch = f.contains_batch(&probe);

    for (i, &k) in probe.iter().enumerate() {
        assert_eq!(
            batch[i],
            f.contains(k),
            "contains_batch disagrees with contains for key {k}"
        );
    }
}

#[test]
fn filter_contains_batch_empty_slice() {
    let f = CuckooFilter::with_capacity(64);
    assert!(f.contains_batch(&[]).is_empty());
}

// ── CuckooMap::get_batch tests ─────────────────────────────────────────────────

#[test]
fn map_get_batch_agrees_with_get() {
    let mut m = CuckooMap::with_capacity(128);
    for i in 0u64..64 {
        let rd = RowDescriptor::new(0, i as u16, i as u32);
        m.insert(i * 3 + 1, rd).unwrap();
    }

    let keys: Vec<u64> = (0..200).collect();
    let batch = m.get_batch(&keys);

    for (i, &k) in keys.iter().enumerate() {
        assert_eq!(
            batch[i],
            m.get(k),
            "get_batch disagrees with get for key {k}"
        );
    }
}

#[test]
fn map_get_batch_partial_hits() {
    let mut m = CuckooMap::with_capacity(32);
    let rd = RowDescriptor::new(0, 0, 99);
    m.insert(42, rd).unwrap();
    m.insert(100, rd).unwrap();

    let batch = m.get_batch(&[42, 43, 100, 101]);
    assert_eq!(batch[0], Some(rd));
    assert_eq!(batch[1], None);
    assert_eq!(batch[2], Some(rd));
    assert_eq!(batch[3], None);
}

#[test]
fn map_get_batch_empty_slice() {
    let m = CuckooMap::with_capacity(16);
    assert!(m.get_batch(&[]).is_empty());
}
