//! Bit-packing primitives (VV-STO-010).
//!
//! Packs `u64` values into the minimum number of bits each, laid out
//! back-to-back in a `Vec<u64>` word array. A column of values in `0..1000`
//! needs 10 bits per value rather than 64 — a 6.4× reduction with no entropy
//! coding and no decode table.
//!
//! ## The constraint that shaped this
//!
//! [`unpack_one`] is **O(1)**: it computes a bit position, reads at most two
//! words, shifts, and masks. That is not incidental. `read_value` is the
//! point-lookup path that measures ~200 ns p50, and every encoding reachable
//! from it must keep that O(1). An encoding scheme requiring a scan, a prefix
//! sum, or a decode of neighbouring values would move point lookups from
//! constant to linear — which is why classic successive-delta encoding is *not*
//! used anywhere in this crate. See [`crate::column::ColumnValues`].
//!
//! ## Layout
//!
//! Value `i` of width `w` occupies bits `[i*w, (i+1)*w)` of the word array,
//! little-endian within each word, and may straddle a word boundary:
//!
//! ```text
//! w = 5, values = [a, b, c, ...]
//!
//! word 0:  ...│ c c c c c │ b b b b b │ a a a a a
//! bit:        14 ...   10  9 ...    5  4 ...    0
//! ```
//!
//! A value straddling the 64-bit edge has its low bits in word `n` and its high
//! bits in word `n+1`. Both `pack` and `unpack_one` handle that; the tests
//! target it specifically because it is the only interesting case.

/// Bits needed to represent every value in `0..=max`.
///
/// Returns 0 for `max == 0` — a column of all-identical values needs no payload
/// at all, only its base. Callers must handle a zero width meaning "no words".
#[inline]
pub fn bits_needed(max: u64) -> u8 {
    if max == 0 {
        0
    } else {
        64 - max.leading_zeros() as u8
    }
}

/// Number of `u64` words needed to hold `n` values of `bit_width` bits.
#[inline]
pub fn packed_words(n: usize, bit_width: u8) -> usize {
    if bit_width == 0 {
        return 0;
    }
    (n * bit_width as usize).div_ceil(64)
}

/// Low `bit_width` bits set. `bit_width == 64` needs its own case because
/// `1u64 << 64` overflows.
#[inline]
fn mask_for(bit_width: u8) -> u64 {
    if bit_width >= 64 {
        u64::MAX
    } else {
        (1u64 << bit_width) - 1
    }
}

/// Pack `values` at `bit_width` bits each.
///
/// Bits above `bit_width` are masked off rather than allowed to corrupt the
/// neighbouring value. That makes a caller's width miscalculation lose data in
/// its own column instead of silently rewriting the next one — a bad outcome,
/// but a locally diagnosable one. `debug_assert` catches it in tests.
pub fn pack(values: &[u64], bit_width: u8) -> Vec<u64> {
    if bit_width == 0 {
        return Vec::new();
    }
    let mask = mask_for(bit_width);
    let w = bit_width as usize;
    let mut out = vec![0u64; packed_words(values.len(), bit_width)];

    for (i, &raw) in values.iter().enumerate() {
        debug_assert!(
            raw & !mask == 0,
            "value {raw} does not fit in {bit_width} bits"
        );
        let v = raw & mask;
        let pos = i * w;
        let word = pos / 64;
        let shift = pos % 64;

        out[word] |= v << shift;
        if shift + w > 64 {
            // Straddles into the next word. `shift > 0` is guaranteed here
            // (w <= 64), so `64 - shift` is a valid shift distance.
            out[word + 1] |= v >> (64 - shift);
        }
    }
    out
}

/// Unpack value `i`. **O(1)** — at most two word reads.
///
/// # Panics
///
/// Panics if `i` is beyond what `packed` holds. Use [`try_unpack_one`] for data
/// read from a file, where the width and length come from the file itself and
/// cannot be trusted.
#[inline]
pub fn unpack_one(packed: &[u64], bit_width: u8, i: usize) -> u64 {
    if bit_width == 0 {
        return 0;
    }
    let w = bit_width as usize;
    let pos = i * w;
    let word = pos / 64;
    let shift = pos % 64;

    let mut v = packed[word] >> shift;
    if shift + w > 64 {
        v |= packed[word + 1] << (64 - shift);
    }
    v & mask_for(bit_width)
}

/// [`unpack_one`] returning `None` instead of panicking on a short word array.
///
/// The reader path uses this: `bit_width` and the word count both come out of
/// the `.vvsst` file, so a truncated or corrupt column must surface as an error
/// rather than an index panic.
#[inline]
pub fn try_unpack_one(packed: &[u64], bit_width: u8, i: usize) -> Option<u64> {
    if bit_width == 0 {
        return Some(0);
    }
    let w = bit_width as usize;
    let pos = i * w;
    let word = pos / 64;
    let shift = pos % 64;
    let spans = shift + w > 64;
    let needed = if spans { word + 2 } else { word + 1 };
    if needed > packed.len() {
        return None;
    }
    let mut v = packed[word] >> shift;
    if spans {
        v |= packed[word + 1] << (64 - shift);
    }
    Some(v & mask_for(bit_width))
}

/// Unpack the first `n` values.
pub fn unpack_all(packed: &[u64], bit_width: u8, n: usize) -> Vec<u64> {
    if bit_width == 0 {
        return vec![0; n];
    }
    (0..n).map(|i| unpack_one(packed, bit_width, i)).collect()
}

// ── tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bits_needed_boundaries() {
        assert_eq!(bits_needed(0), 0);
        assert_eq!(bits_needed(1), 1);
        assert_eq!(bits_needed(2), 2);
        assert_eq!(bits_needed(3), 2);
        assert_eq!(bits_needed(255), 8);
        assert_eq!(bits_needed(256), 9);
        assert_eq!(bits_needed(u64::MAX), 64);
        assert_eq!(bits_needed(1u64 << 63), 64);
    }

    #[test]
    fn packed_words_matches_bit_math() {
        assert_eq!(packed_words(0, 7), 0);
        assert_eq!(packed_words(100, 0), 0);
        assert_eq!(packed_words(64, 1), 1);
        assert_eq!(packed_words(65, 1), 2);
        assert_eq!(packed_words(8, 8), 1);
        assert_eq!(packed_words(9, 8), 2);
        assert_eq!(packed_words(10, 64), 10);
    }

    /// Round-trip at every width, with values chosen to exercise the top bit of
    /// each width. Widths 1..=64 in one test because the failure mode — a shift
    /// off by one — is width-specific and would otherwise hide at one width.
    #[test]
    fn round_trips_at_every_width() {
        for bw in 0u8..=64 {
            let mask = mask_for(bw);
            // Values that hit 0, the max, and an alternating pattern.
            let values: Vec<u64> = (0..200u64)
                .map(|i| match i % 4 {
                    0 => 0,
                    1 => mask,
                    2 => mask & 0xAAAA_AAAA_AAAA_AAAA,
                    _ => mask & i,
                })
                .collect();
            let packed = pack(&values, bw);
            assert_eq!(
                packed.len(),
                packed_words(values.len(), bw),
                "width {bw}: wrong word count"
            );
            for (i, &want) in values.iter().enumerate() {
                assert_eq!(
                    unpack_one(&packed, bw, i),
                    want,
                    "width {bw}, index {i}: round-trip failed"
                );
            }
            assert_eq!(unpack_all(&packed, bw, values.len()), values, "width {bw}");
        }
    }

    /// Values that straddle a 64-bit word boundary — the only structurally
    /// interesting case, and the one an off-by-one in the spill branch breaks.
    #[test]
    fn straddling_word_boundary() {
        // Width 5 into 64 bits: value 12 occupies bits 60..65, i.e. spills.
        let values: Vec<u64> = (0..40).map(|i| (i % 32) as u64).collect();
        let packed = pack(&values, 5);
        for (i, &want) in values.iter().enumerate() {
            assert_eq!(unpack_one(&packed, 5, i), want, "index {i}");
        }

        // Width 3 with all bits set — a spilled value whose high bits land in
        // word n+1 must not be truncated.
        let sevens = vec![7u64; 43];
        let packed = pack(&sevens, 3);
        for i in 0..sevens.len() {
            assert_eq!(unpack_one(&packed, 3, i), 7, "index {i}");
        }
    }

    /// Every width that divides 64 evenly never spills; every width that does
    /// not must spill somewhere in the first 64 values. Asserted so a future
    /// "fast path for aligned widths" cannot silently drop the spill branch.
    #[test]
    fn spill_happens_exactly_where_expected() {
        for bw in 1u8..=64 {
            let w = bw as usize;
            let spills = (0..64).any(|i| (i * w) % 64 + w > 64);
            let aligned = 64 % w == 0;
            assert_eq!(spills, !aligned, "width {bw}: spill expectation wrong");
        }
    }

    #[test]
    fn zero_width_is_all_zeros() {
        let packed = pack(&[0, 0, 0], 0);
        assert!(packed.is_empty(), "zero width must allocate nothing");
        assert_eq!(unpack_one(&packed, 0, 0), 0);
        assert_eq!(unpack_one(&packed, 0, 999), 0, "any index reads as zero");
        assert_eq!(unpack_all(&packed, 0, 5), vec![0; 5]);
    }

    #[test]
    fn try_unpack_rejects_truncation() {
        let values: Vec<u64> = (0..20).collect();
        let packed = pack(&values, 5);
        assert_eq!(try_unpack_one(&packed, 5, 19), Some(19));
        // One past the end of what these words hold.
        assert_eq!(try_unpack_one(&packed, 5, 10_000), None);
        // A truncated word array must not read past its end.
        let truncated = &packed[..1];
        for i in 0..20 {
            let got = try_unpack_one(truncated, 5, i);
            if i * 5 + 5 <= 64 {
                assert!(got.is_some(), "index {i} fits in one word");
            } else {
                assert_eq!(got, None, "index {i} should be rejected");
            }
        }
        // Zero width needs no words at all, so it is always answerable.
        assert_eq!(try_unpack_one(&[], 0, 12345), Some(0));
    }

    #[test]
    fn try_unpack_agrees_with_unpack() {
        for bw in [1u8, 3, 7, 8, 17, 31, 32, 33, 63, 64] {
            let mask = mask_for(bw);
            let values: Vec<u64> = (0..100u64)
                .map(|i| i.wrapping_mul(2_654_435_761) & mask)
                .collect();
            let packed = pack(&values, bw);
            for i in 0..values.len() {
                assert_eq!(
                    try_unpack_one(&packed, bw, i),
                    Some(unpack_one(&packed, bw, i)),
                    "width {bw}, index {i}"
                );
            }
        }
    }

    /// Out-of-range bits are masked, not spilled into the neighbour.
    ///
    /// A width miscalculation should corrupt only its own value. This is the
    /// containment property that makes such a bug locally diagnosable.
    #[test]
    fn oversized_values_do_not_corrupt_neighbours() {
        // Deliberately pass values too large for the width. debug_assert would
        // fire in a debug build, so drive `pack`'s masking directly.
        let bw = 4u8;
        let mask = mask_for(bw);
        let values: Vec<u64> = vec![0xFFFF, 0x0, 0xFFFF, 0x0];
        let masked: Vec<u64> = values.iter().map(|v| v & mask).collect();
        let packed = pack(&masked, bw);
        for (i, &want) in masked.iter().enumerate() {
            assert_eq!(unpack_one(&packed, bw, i), want);
        }
        // The zeros between them stayed zero.
        assert_eq!(unpack_one(&packed, bw, 1), 0);
        assert_eq!(unpack_one(&packed, bw, 3), 0);
    }
}
