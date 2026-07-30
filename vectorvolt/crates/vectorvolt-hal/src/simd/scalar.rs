//! Portable scalar kernels.
//!
//! These are the **correctness oracle**: every SIMD kernel in this crate is
//! tested for bit-exact agreement with the function of the same name here, over
//! randomized inputs at every length around a vector-width boundary. If a SIMD
//! path and its scalar twin ever disagree, the SIMD path is wrong.
//!
//! They are also the fallback on any CPU without the required feature, and on
//! architectures where we have not written vector kernels yet. Nothing in
//! VectorVolt is allowed to *require* SIMD for correctness.

/// Append the index of every element in `lo..=hi` to `out`.
#[inline]
pub fn filter_range(values: &[u64], lo: u64, hi: u64, out: &mut Vec<u32>) {
    for (i, &v) in values.iter().enumerate() {
        if v >= lo && v <= hi {
            out.push(i as u32);
        }
    }
}

/// Count elements in `lo..=hi`.
#[inline]
pub fn count_range(values: &[u64], lo: u64, hi: u64) -> usize {
    values.iter().filter(|&&v| v >= lo && v <= hi).count()
}

/// `(min, max)` of `values`, or `None` if empty. Unsigned comparison.
#[inline]
pub fn min_max(values: &[u64]) -> Option<(u64, u64)> {
    let (&first, rest) = values.split_first()?;
    let mut lo = first;
    let mut hi = first;
    for &v in rest {
        if v < lo {
            lo = v;
        }
        if v > hi {
            hi = v;
        }
    }
    Some((lo, hi))
}

/// Wrapping sum of `values`.
#[inline]
pub fn sum(values: &[u64]) -> u64 {
    values.iter().fold(0u64, |a, &b| a.wrapping_add(b))
}

/// Index of the first element equal to `needle`.
#[inline]
pub fn find_first_eq(values: &[u64], needle: u64) -> Option<usize> {
    values.iter().position(|&v| v == needle)
}

/// Total set bits across `words`.
///
/// Already one `POPCNT` instruction per word on any modern CPU — the SIMD
/// variants of this only win on very large bitmaps, so there is no separate
/// AVX2 implementation.
#[inline]
pub fn popcount(words: &[u64]) -> u64 {
    words.iter().map(|w| w.count_ones() as u64).sum()
}

/// CRC-32C (Castagnoli, polynomial `0x1EDC6F41`, reflected `0x82F63B78`).
///
/// **Not** the same polynomial as CRC-32/ISO-HDLC used by `crc32fast` and by the
/// existing WAL frame format. These produce different values for the same input
/// and are not interchangeable. This function exists because CRC-32C is what the
/// `crc32` hardware instruction computes, making it ~20× faster; new formats
/// should use it, but swapping it into the WAL would be a breaking format change.
#[inline]
pub fn crc32c(data: &[u8]) -> u32 {
    const POLY: u32 = 0x82F6_3B78;
    let mut crc = !0u32;
    for &byte in data {
        crc ^= byte as u32;
        for _ in 0..8 {
            let mask = (crc & 1).wrapping_neg();
            crc = (crc >> 1) ^ (POLY & mask);
        }
    }
    !crc
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn filter_and_count_agree() {
        let v: Vec<u64> = (0..100).collect();
        let mut out = Vec::new();
        filter_range(&v, 10, 20, &mut out);
        assert_eq!(out.len(), 11);
        assert_eq!(out[0], 10);
        assert_eq!(out[10], 20);
        assert_eq!(count_range(&v, 10, 20), 11);
    }

    #[test]
    fn range_handles_full_u64_domain() {
        // Values above i64::MAX must compare correctly as unsigned.
        let v = vec![0, u64::MAX, i64::MAX as u64, (i64::MAX as u64) + 1];
        assert_eq!(count_range(&v, 0, u64::MAX), 4);
        assert_eq!(count_range(&v, (i64::MAX as u64) + 1, u64::MAX), 2);
        assert_eq!(min_max(&v), Some((0, u64::MAX)));
    }

    #[test]
    fn min_max_empty_is_none() {
        assert_eq!(min_max(&[]), None);
        assert_eq!(min_max(&[7]), Some((7, 7)));
    }

    #[test]
    fn sum_wraps() {
        assert_eq!(sum(&[u64::MAX, 1]), 0);
        assert_eq!(sum(&[]), 0);
        assert_eq!(sum(&[1, 2, 3]), 6);
    }

    #[test]
    fn find_first_eq_finds_first_not_any() {
        let v = vec![5, 9, 5, 9];
        assert_eq!(find_first_eq(&v, 9), Some(1));
        assert_eq!(find_first_eq(&v, 5), Some(0));
        assert_eq!(find_first_eq(&v, 42), None);
    }

    #[test]
    fn popcount_counts_all_words() {
        assert_eq!(popcount(&[]), 0);
        assert_eq!(popcount(&[0]), 0);
        assert_eq!(popcount(&[u64::MAX]), 64);
        assert_eq!(popcount(&[u64::MAX, u64::MAX]), 128);
        assert_eq!(popcount(&[0b1011]), 3);
    }

    #[test]
    fn crc32c_matches_known_vectors() {
        // `0xE3069283` for "123456789" is the CRC-32C *specification check
        // value* (the standard's own conformance constant). If this line passes,
        // the polynomial, reflection, init, and final-xor are all correct.
        assert_eq!(crc32c(b"123456789"), 0xE306_9283);

        // Corroborating vectors.
        assert_eq!(crc32c(b""), 0x0000_0000);
        assert_eq!(crc32c(b"a"), 0xC1D0_4330);
        assert_eq!(
            crc32c(b"The quick brown fox jumps over the lazy dog"),
            0x2262_0404
        );
    }
}
