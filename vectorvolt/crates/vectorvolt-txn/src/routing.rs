//! Partition routing: map a primary key to a logical partition index.
//!
//! Uses multiplicative (Fibonacci) hashing for uniform distribution across
//! arbitrary partition counts without requiring a power-of-two size.
//!
//! ```
//! use vectorvolt_txn::routing::select_partition;
//! assert_eq!(select_partition(0, 1), 0);
//! assert!(select_partition(42, 4) < 4);
//! ```

/// Map `pk` to a logical partition index in `[0, partition_count)`.
///
/// Uses the 64-bit Fibonacci multiplier (`2^64 / φ`) for a high-quality
/// avalanche effect: nearby PKs scatter across different partitions, and
/// all partition counts (not just powers of two) work correctly.
///
/// # Panics
/// Panics in debug mode if `partition_count == 0`.
#[inline]
pub fn select_partition(pk: u64, partition_count: usize) -> usize {
    debug_assert!(partition_count > 0, "partition_count must be >= 1");
    // 0x9e3779b97f4a7c15 ≈ 2^64 / φ (golden ratio). High 32 bits carry the
    // most entropy after the multiplication, so we shift them down.
    let hash = pk.wrapping_mul(0x9e3779b97f4a7c15) >> 32;
    hash as usize % partition_count
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_partition_always_zero() {
        for pk in [0, 1, u64::MAX, 42, 999] {
            assert_eq!(select_partition(pk, 1), 0);
        }
    }

    #[test]
    fn result_always_in_range() {
        for n in 1..=16usize {
            for pk in 0u64..100 {
                let p = select_partition(pk, n);
                assert!(p < n, "select_partition({pk}, {n}) = {p} is out of range");
            }
        }
    }

    #[test]
    fn deterministic_same_inputs_same_output() {
        assert_eq!(select_partition(42, 8), select_partition(42, 8));
        assert_eq!(select_partition(u64::MAX, 3), select_partition(u64::MAX, 3));
    }

    #[test]
    fn all_partitions_reachable_with_enough_keys() {
        let n = 4;
        let mut seen = [false; 4];
        for pk in 0u64..1000 {
            seen[select_partition(pk, n)] = true;
        }
        assert!(seen.iter().all(|&s| s), "not all partitions were reached");
    }
}
