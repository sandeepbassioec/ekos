//! Integration tests for `vectorvolt-index`.
//!
//! Each test exercises multiple index types together to simulate a real
//! shard-level index operation: insert rows, look them up, scan ranges,
//! and check FK existence.

use vectorvolt_index::{CuckooFilter, CuckooMap, RowDescriptor, SecondaryIndex, ZoneMap};

// ── helpers ───────────────────────────────────────────────────────────────────

fn rd(file: u16, part: u16, offset: u32) -> RowDescriptor {
    RowDescriptor::new(file, part, offset)
}

/// Tiny FNV-1a u32 hash — matches what a real projection worker would use.
fn hash32(v: u64) -> u32 {
    let mut h: u32 = 2_166_136_261;
    for byte in v.to_le_bytes() {
        h ^= byte as u32;
        h = h.wrapping_mul(16_777_619);
    }
    h
}

// ── tests ─────────────────────────────────────────────────────────────────────

#[test]
fn pk_lookup_round_trip() {
    // 1 000 rows across 4 files, 8 partitions each.
    let mut pk = CuckooMap::with_capacity(1024);
    for pk_val in 0u64..1000 {
        let file_id = (pk_val / 250) as u16;
        let partition_id = ((pk_val % 250) / 32) as u16;
        let row_offset = (pk_val % 32) as u32;
        pk.insert(pk_val, rd(file_id, partition_id, row_offset))
            .expect("insert ok");
    }
    assert_eq!(pk.len(), 1000);
    for pk_val in 0u64..1000 {
        let desc = pk
            .get(pk_val)
            .unwrap_or_else(|| panic!("pk {pk_val} missing"));
        let file_id = (pk_val / 250) as u16;
        assert_eq!(desc.file_id(), file_id);
    }
}

#[test]
fn secondary_index_multi_value_scan() {
    // Simulate indexing an integer column "status" with values 0..5.
    let mut sec = SecondaryIndex::new();
    for row_id in 0u32..500 {
        let status = (row_id % 5) as u64;
        sec.insert(hash32(status), row_id);
    }
    // Each status value appears in 100 rows.
    for status in 0u64..5 {
        let mut rows = sec.get_vec(hash32(status));
        rows.sort_unstable();
        assert_eq!(rows.len(), 100, "status={status} should have 100 rows");
    }
}

#[test]
fn zone_map_prunes_partitions() {
    // Partition layout:
    //   p0: age ∈ [20, 35]
    //   p1: age ∈ [36, 50]
    //   p2: age ∈ [51, 65]
    //   p3: age ∈ [66, 80]
    let mut zm = ZoneMap::new();
    zm.set(0, 20, 35);
    zm.set(1, 36, 50);
    zm.set(2, 51, 65);
    zm.set(3, 66, 80);

    // Query: age BETWEEN 40 AND 60
    let mut hits: Vec<u16> = zm.overlapping_partitions(40, 60).collect();
    hits.sort_unstable();
    assert_eq!(hits, vec![1, 2], "should hit p1 and p2 only");

    let mut pruned = zm.pruned_partitions(40, 60);
    pruned.sort_unstable();
    assert_eq!(pruned, vec![0, 3], "should prune p0 and p3");
}

#[test]
fn cuckoo_filter_fk_check() {
    // Simulate FK check: table A has 500 PKs; verify FK references.
    let mut filter = CuckooFilter::with_capacity(512);
    for pk in 0u64..500 {
        filter.insert(pk).expect("filter insert ok");
    }
    // All inserted keys must be found (zero false negatives).
    for pk in 0u64..500 {
        assert!(filter.contains(pk), "pk {pk} must be in filter");
    }
    // Keys 1000..1050 were never inserted; most should not match.
    let mut false_positives = 0usize;
    for fk in 1000u64..1050 {
        if filter.contains(fk) {
            false_positives += 1;
        }
    }
    assert!(false_positives < 5, "too many FP: {false_positives}");
}

#[test]
fn full_shard_index_workflow() {
    // Simulate a complete shard index: PK map + secondary index + zone map + filter.
    let mut pk_map = CuckooMap::with_capacity(256);
    let mut sec_idx = SecondaryIndex::new();
    let mut zone_map = ZoneMap::new();
    let mut fk_filter = CuckooFilter::with_capacity(256);

    // Insert 200 rows: PK is row_id, "category" column has 10 distinct values.
    for row_id in 0u32..200 {
        let pk = row_id as u64;
        let partition_id = (row_id / 50) as u16; // 4 partitions of 50 rows each
        let category = (row_id % 10) as u64;

        pk_map.insert(pk, rd(0, partition_id, row_id)).unwrap();
        sec_idx.insert(hash32(category), row_id);
        zone_map.update(partition_id, category);
        fk_filter.insert(pk).unwrap();
    }

    // 1. PK lookup
    let desc = pk_map.get(77).unwrap();
    assert_eq!(desc.partition_id(), 1); // row 77 ∈ partition 1 (rows 50-99)
    assert_eq!(desc.row_offset(), 77);

    // 2. Secondary scan: category = 3 → 20 rows (3, 13, 23, ... 193)
    let mut cat3_rows = sec_idx.get_vec(hash32(3));
    cat3_rows.sort_unstable();
    assert_eq!(cat3_rows.len(), 20);
    assert_eq!(cat3_rows[0], 3);
    assert_eq!(cat3_rows[1], 13);

    // 3. Zone map: category BETWEEN 8 AND 9 → only p0, p1, p2, p3 all have 8..9,
    //    so all overlap; query category BETWEEN 0 AND 0 should still hit all partitions
    //    (each partition has category=0 rows).
    let overlap: Vec<u16> = zone_map.overlapping_partitions(0, 0).collect();
    // All 4 partitions have category=0, so all should overlap.
    assert_eq!(overlap.len(), 4);

    // 4. FK filter: all 200 PKs must be found, unknown PKs mostly absent.
    for pk in 0u64..200 {
        assert!(fk_filter.contains(pk));
    }
    let mut fp_count = 0usize;
    for pk in 500u64..600 {
        if fk_filter.contains(pk) {
            fp_count += 1;
        }
    }
    assert!(fp_count < 10, "excessive FP in FK filter: {fp_count}");
}

#[test]
fn row_descriptor_bit_packing_survives_index_round_trip() {
    // Verify that RowDescriptor round-trips through CuckooMap without corruption.
    let cases = [
        rd(0, 0, 0),
        rd(u16::MAX, u16::MAX, u32::MAX),
        rd(1, 2, 3),
        rd(256, 128, 99_999),
    ];
    let mut map = CuckooMap::with_capacity(16);
    for (i, desc) in cases.iter().enumerate() {
        map.insert(i as u64, *desc).unwrap();
    }
    for (i, desc) in cases.iter().enumerate() {
        let got = map.get(i as u64).unwrap();
        assert_eq!(got, *desc);
        assert_eq!(got.file_id(), desc.file_id());
        assert_eq!(got.partition_id(), desc.partition_id());
        assert_eq!(got.row_offset(), desc.row_offset());
    }
}
