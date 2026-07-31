//! [`ColumnValues`] — in-memory column storage with automatic encoding selection.
//!
//! ## Encodings
//!
//! | Variant | Shape it targets | Bytes for `n` rows | `get(row)` |
//! |---|---|---|---|
//! | `Raw` | anything | `8n` | O(1) |
//! | `DictEncoded` | ≤ 256 distinct values | `8d + 4n` | O(1) |
//! | `Rle` | long runs of one value | `12r` (`r` = runs) | O(log r) |
//! | `Frame` | narrow value range | `8 + ⌈nw/64⌉·8` | O(1) |
//! | `BlockedFrame` | sorted / monotonic / drifting | `8b + ⌈nw/64⌉·8` | O(1) |
//!
//! `w` is the bit width, `b = ⌈n/64⌉` the block count.
//!
//! ## Why there is no successive-delta encoding
//!
//! The obvious encoding for a sorted column is to store differences between
//! consecutive values. It compresses beautifully — and it is **unusable here**.
//! Reconstructing value `i` needs a prefix sum over values `0..i`, so a point
//! lookup becomes O(n). `read_value` is the path that measures ~200 ns p50 and
//! the whole read architecture rests on it being O(1).
//!
//! `BlockedFrame` gets the same compression without that cost. It splits the
//! column into 64-value blocks and stores, per block, an absolute base plus each
//! value's *offset from that base* — not from its predecessor. Sorted data has a
//! narrow range inside any 64-value window, so the bit width is just as small,
//! but value `i` is `base[i/64] + unpack(i)`: two reads, no scan, still O(1).
//!
//! This is the one place in the crate where a worse compression ratio was chosen
//! deliberately. It is not an oversight.
//!
//! ## RLE is the exception, and it is opt-in by cost
//!
//! `Rle` is the only encoding whose `get` is not O(1) — it binary-searches the
//! run-end array, O(log r). It is selected only when it wins on size by a clear
//! margin (see [`RLE_MIN_ADVANTAGE`]), because trading the O(1) guarantee for a
//! few percent of space is not worth it. A column of 1,000 rows in 4 runs is
//! `log₂(4) = 2` probes and an enormous space win; a column in 900 runs is
//! neither.
//!
//! ## Zone min/max
//!
//! Every variant provides `zone_min_max()` for ZoneMap population at write time,
//! and each computes it in O(metadata) rather than O(n) where possible — over the
//! dictionary, the run values, or the frame base plus width.

use crate::bitpack;
use crate::format::{
    ENCODING_BLOCKED_FRAME, ENCODING_DICT, ENCODING_FRAME, ENCODING_RAW, ENCODING_RLE,
};

/// Values per block in [`ColumnValues::BlockedFrame`].
///
/// 64 keeps the base array at 1/64th the row count (0.125 bytes/row) while being
/// small enough that sorted data has a narrow in-block range. It is also one
/// cache line of bases per 512 values.
pub const FRAME_BLOCK: usize = 64;

/// Maximum distinct values before dictionary encoding is abandoned.
///
/// Bounded by the u32 index width being worth paying: past this point the
/// dictionary itself costs more than it saves.
pub const DICT_MAX_DISTINCT: usize = 256;

/// How much smaller RLE must be to justify giving up O(1) `get`.
///
/// RLE must use less than this fraction of the next-best encoding's size. 0.5 —
/// it has to at least halve the column.
pub const RLE_MIN_ADVANTAGE: f64 = 0.5;

/// Column data for one micro-partition.
#[derive(Debug, Clone, PartialEq)]
pub enum ColumnValues {
    /// Each row stores a raw `u64` value.
    Raw(Vec<u64>),
    /// Low-cardinality column: values stored as indices into a dictionary.
    DictEncoded {
        /// Distinct values, in insertion order.
        dict: Vec<u64>,
        /// Per-row index into `dict`.
        indices: Vec<u32>,
    },
    /// Run-length encoded: `values[k]` covers rows `run_ends[k-1]..run_ends[k]`.
    ///
    /// `run_ends` holds **exclusive** cumulative ends, so it is sorted and
    /// `partition_point` locates a row's run directly. Storing ends rather than
    /// lengths is what keeps `get` at O(log r) instead of O(r).
    Rle {
        values: Vec<u64>,
        run_ends: Vec<u32>,
    },
    /// Frame of reference: every value stored as `base + offset`, offsets
    /// bit-packed at the width needed for `max - min`.
    Frame {
        base: u64,
        bit_width: u8,
        packed: Vec<u64>,
        row_count: usize,
    },
    /// Blocked frame of reference: one base per [`FRAME_BLOCK`] rows.
    ///
    /// The replacement for successive-delta encoding — see the module docs.
    BlockedFrame {
        block_bases: Vec<u64>,
        bit_width: u8,
        packed: Vec<u64>,
        row_count: usize,
    },
}

/// Byte cost of each candidate encoding for one column, for selection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Costs {
    raw: usize,
    dict: Option<usize>,
    rle: Option<usize>,
    frame: usize,
    blocked: usize,
}

impl ColumnValues {
    /// Build a `ColumnValues`, choosing the smallest encoding that preserves the
    /// O(1) `get` guarantee — plus RLE when it wins by [`RLE_MIN_ADVANTAGE`].
    ///
    /// Every candidate's exact byte cost is computed and compared. There is no
    /// heuristic and no sampling: the column is already in memory, so measuring
    /// is cheaper and more reliable than guessing, and the result is
    /// deterministic — the same input always produces the same encoding, which
    /// matters for reproducible files and for tests.
    pub fn auto_encode(values: Vec<u64>) -> Self {
        if values.is_empty() {
            return ColumnValues::Raw(values);
        }

        let n = values.len();
        let (min, max) = (
            values.iter().copied().min().unwrap(),
            values.iter().copied().max().unwrap(),
        );

        // ── candidate: dictionary ────────────────────────────────────────────
        let dict_parts = Self::build_dict(&values);
        // ── candidate: RLE ──────────────────────────────────────────────────
        let rle_parts = Self::build_rle(&values);
        // ── candidate: whole-column frame ───────────────────────────────────
        let frame_width = bitpack::bits_needed(max - min);
        // ── candidate: blocked frame ────────────────────────────────────────
        let blocked_width = Self::blocked_frame_width(&values);
        let block_count = n.div_ceil(FRAME_BLOCK);

        let costs = Costs {
            raw: n * 8,
            dict: dict_parts.as_ref().map(|(d, _)| d.len() * 8 + n * 4),
            rle: rle_parts.as_ref().map(|(v, _)| v.len() * 8 + v.len() * 4),
            frame: 8 + bitpack::packed_words(n, frame_width) * 8,
            blocked: block_count * 8 + bitpack::packed_words(n, blocked_width) * 8,
        };

        // Best among the O(1) encodings.
        let mut best = costs.raw;
        let mut choice = 0u8;
        if let Some(c) = costs.dict
            && c < best
        {
            best = c;
            choice = 1;
        }
        if costs.frame < best {
            best = costs.frame;
            choice = 2;
        }
        if costs.blocked < best {
            best = costs.blocked;
            choice = 3;
        }

        // RLE only if it beats the best O(1) encoding by a clear margin.
        if let Some(c) = costs.rle
            && (c as f64) < best as f64 * RLE_MIN_ADVANTAGE
        {
            let (rle_values, run_ends) = rle_parts.unwrap();
            return ColumnValues::Rle {
                values: rle_values,
                run_ends,
            };
        }

        match choice {
            1 => {
                let (dict, indices) = dict_parts.unwrap();
                ColumnValues::DictEncoded { dict, indices }
            }
            2 => {
                let offsets: Vec<u64> = values.iter().map(|v| v - min).collect();
                ColumnValues::Frame {
                    base: min,
                    bit_width: frame_width,
                    packed: bitpack::pack(&offsets, frame_width),
                    row_count: n,
                }
            }
            3 => {
                let (block_bases, offsets) = Self::blocked_frame_parts(&values);
                ColumnValues::BlockedFrame {
                    block_bases,
                    bit_width: blocked_width,
                    packed: bitpack::pack(&offsets, blocked_width),
                    row_count: n,
                }
            }
            _ => ColumnValues::Raw(values),
        }
    }

    /// Dictionary parts, or `None` if cardinality exceeds [`DICT_MAX_DISTINCT`].
    fn build_dict(values: &[u64]) -> Option<(Vec<u64>, Vec<u32>)> {
        use std::collections::HashMap;
        use std::collections::hash_map::Entry;

        let mut index_map: HashMap<u64, u32> = HashMap::new();
        let mut dict: Vec<u64> = Vec::new();
        for &v in values {
            if let Entry::Vacant(e) = index_map.entry(v) {
                if dict.len() >= DICT_MAX_DISTINCT {
                    return None;
                }
                e.insert(dict.len() as u32);
                dict.push(v);
            }
        }
        let indices = values.iter().map(|v| index_map[v]).collect();
        Some((dict, indices))
    }

    /// Run values plus exclusive cumulative run ends.
    ///
    /// `None` if the column has more than `u32::MAX` rows, which the `.vvsst`
    /// header's `row_count: u32` already forbids — checked rather than assumed.
    fn build_rle(values: &[u64]) -> Option<(Vec<u64>, Vec<u32>)> {
        if values.len() > u32::MAX as usize {
            return None;
        }
        let mut run_values: Vec<u64> = Vec::new();
        let mut run_ends: Vec<u32> = Vec::new();
        for (i, &v) in values.iter().enumerate() {
            if run_values.last() == Some(&v) {
                *run_ends.last_mut().unwrap() = (i + 1) as u32;
            } else {
                run_values.push(v);
                run_ends.push((i + 1) as u32);
            }
        }
        Some((run_values, run_ends))
    }

    /// Widest in-block range across all blocks, as a bit width.
    ///
    /// One width for the whole column rather than per block: a per-block width
    /// would compress better but makes value `i`'s bit position depend on every
    /// preceding block's width, which is a prefix sum — the exact O(n) lookup
    /// this encoding exists to avoid.
    fn blocked_frame_width(values: &[u64]) -> u8 {
        let mut widest = 0u8;
        for block in values.chunks(FRAME_BLOCK) {
            let min = block.iter().copied().min().unwrap();
            let max = block.iter().copied().max().unwrap();
            widest = widest.max(bitpack::bits_needed(max - min));
        }
        widest
    }

    fn blocked_frame_parts(values: &[u64]) -> (Vec<u64>, Vec<u64>) {
        let mut bases = Vec::with_capacity(values.len().div_ceil(FRAME_BLOCK));
        let mut offsets = Vec::with_capacity(values.len());
        for block in values.chunks(FRAME_BLOCK) {
            let base = block.iter().copied().min().unwrap();
            bases.push(base);
            offsets.extend(block.iter().map(|v| v - base));
        }
        (bases, offsets)
    }

    /// Number of rows in this column.
    pub fn row_count(&self) -> usize {
        match self {
            ColumnValues::Raw(v) => v.len(),
            ColumnValues::DictEncoded { indices, .. } => indices.len(),
            ColumnValues::Rle { run_ends, .. } => run_ends.last().copied().unwrap_or(0) as usize,
            ColumnValues::Frame { row_count, .. }
            | ColumnValues::BlockedFrame { row_count, .. } => *row_count,
        }
    }

    /// `(min, max)` over all values — used for ZoneMap population.
    ///
    /// Returns `(0, 0)` if the column is empty. Every variant computes this over
    /// its compressed representation, not a decoded column: O(d) for a
    /// dictionary, O(r) for runs, O(1) for a whole-column frame, O(blocks) for a
    /// blocked frame.
    pub fn zone_min_max(&self) -> (u64, u64) {
        fn min_max(vs: &[u64]) -> (u64, u64) {
            match (vs.iter().copied().min(), vs.iter().copied().max()) {
                (Some(a), Some(b)) => (a, b),
                _ => (0, 0),
            }
        }
        match self {
            ColumnValues::Raw(v) => min_max(v),
            ColumnValues::DictEncoded { dict, .. } => min_max(dict),
            ColumnValues::Rle { values, .. } => min_max(values),
            ColumnValues::Frame {
                base,
                bit_width,
                packed,
                row_count,
            } => {
                if *row_count == 0 {
                    return (0, 0);
                }
                // The base is the true minimum by construction. The maximum needs
                // the widest offset actually present, not the width's ceiling —
                // `bits_needed` rounds up, so assuming `base + 2^w - 1` would
                // report a max the column does not contain and weaken pruning.
                let max_off = (0..*row_count)
                    .map(|i| bitpack::unpack_one(packed, *bit_width, i))
                    .max()
                    .unwrap_or(0);
                (*base, base + max_off)
            }
            ColumnValues::BlockedFrame {
                block_bases,
                bit_width,
                packed,
                row_count,
            } => {
                if *row_count == 0 {
                    return (0, 0);
                }
                let mut lo = u64::MAX;
                let mut hi = 0u64;
                for i in 0..*row_count {
                    let v =
                        block_bases[i / FRAME_BLOCK] + bitpack::unpack_one(packed, *bit_width, i);
                    lo = lo.min(v);
                    hi = hi.max(v);
                }
                (lo, hi)
            }
        }
    }

    /// Decode and return the value at `row`.
    ///
    /// O(1) for every variant except `Rle`, which is O(log r).
    ///
    /// # Panics
    ///
    /// Panics if `row` is out of range, matching `Vec` indexing. The file-backed
    /// path in `PartitionReader::read_value` bounds-checks first and returns an
    /// error instead; this is the in-memory write-side path.
    pub fn get(&self, row: usize) -> u64 {
        match self {
            ColumnValues::Raw(v) => v[row],
            ColumnValues::DictEncoded { dict, indices } => dict[indices[row] as usize],
            ColumnValues::Rle { values, run_ends } => {
                // First run whose exclusive end is > row.
                let k = run_ends.partition_point(|&e| e as usize <= row);
                values[k]
            }
            ColumnValues::Frame {
                base,
                bit_width,
                packed,
                row_count,
            } => {
                assert!(row < *row_count, "row {row} out of range {row_count}");
                base + bitpack::unpack_one(packed, *bit_width, row)
            }
            ColumnValues::BlockedFrame {
                block_bases,
                bit_width,
                packed,
                row_count,
            } => {
                assert!(row < *row_count, "row {row} out of range {row_count}");
                block_bases[row / FRAME_BLOCK] + bitpack::unpack_one(packed, *bit_width, row)
            }
        }
    }

    /// Decode all rows into a flat `Vec<u64>` (allocates).
    pub fn to_raw_vec(&self) -> Vec<u64> {
        let mut out = Vec::with_capacity(self.row_count());
        self.decode_into(&mut out);
        out
    }

    /// Decode every row, appending into `out` after clearing it.
    ///
    /// The allocation-reusing form of [`to_raw_vec`](Self::to_raw_vec). A scan
    /// over many partitions can keep one buffer alive instead of allocating and
    /// freeing a `Vec<u64>` per partition, which for a 4096-row column is a 32 KiB
    /// allocation per partition per query.
    pub fn decode_into(&self, out: &mut Vec<u64>) {
        out.clear();
        out.reserve(self.row_count());
        self.decode_append(out)
    }

    fn decode_append(&self, out: &mut Vec<u64>) {
        match self {
            ColumnValues::Raw(v) => out.extend_from_slice(v),
            ColumnValues::DictEncoded { dict, indices } => {
                out.extend(indices.iter().map(|&i| dict[i as usize]))
            }
            ColumnValues::Rle { values, run_ends } => {
                let mut start = 0usize;
                for (k, &end) in run_ends.iter().enumerate() {
                    let end = end as usize;
                    out.extend(std::iter::repeat_n(values[k], end - start));
                    start = end;
                }
            }
            // Unpacking straight into `out` rather than via `unpack_all` avoids a
            // second full-column Vec: the old path allocated the offsets, then
            // allocated again to add the base.
            ColumnValues::Frame {
                base,
                bit_width,
                packed,
                row_count,
            } => out
                .extend((0..*row_count).map(|i| base + bitpack::unpack_one(packed, *bit_width, i))),
            ColumnValues::BlockedFrame {
                block_bases,
                bit_width,
                packed,
                row_count,
            } => out.extend((0..*row_count).map(|i| {
                block_bases[i / FRAME_BLOCK] + bitpack::unpack_one(packed, *bit_width, i)
            })),
        }
    }

    /// Serialized payload size in bytes, excluding column metadata.
    ///
    /// Used by the writer to lay out the data section and by tests to assert that
    /// `auto_encode` actually chose the smallest option.
    pub fn payload_bytes(&self) -> usize {
        match self {
            ColumnValues::Raw(v) => v.len() * 8,
            ColumnValues::DictEncoded { dict, indices } => dict.len() * 8 + indices.len() * 4,
            ColumnValues::Rle { values, run_ends } => values.len() * 8 + run_ends.len() * 4,
            ColumnValues::Frame { packed, .. } => 8 + packed.len() * 8,
            ColumnValues::BlockedFrame {
                block_bases,
                packed,
                ..
            } => block_bases.len() * 8 + packed.len() * 8,
        }
    }

    /// Bit width, for encodings that have one. Written into column metadata.
    pub fn bit_width(&self) -> u8 {
        match self {
            ColumnValues::Frame { bit_width, .. }
            | ColumnValues::BlockedFrame { bit_width, .. } => *bit_width,
            _ => 0,
        }
    }

    /// Wire encoding byte for the binary file format.
    pub fn encoding_byte(&self) -> u8 {
        match self {
            ColumnValues::Raw(_) => ENCODING_RAW,
            ColumnValues::DictEncoded { .. } => ENCODING_DICT,
            ColumnValues::Rle { .. } => ENCODING_RLE,
            ColumnValues::Frame { .. } => ENCODING_FRAME,
            ColumnValues::BlockedFrame { .. } => ENCODING_BLOCKED_FRAME,
        }
    }
}

// ── tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    /// Dictionary wins only when the distinct values are few **and far apart**.
    ///
    /// Few-and-close loses to `Frame`, which is worth spelling out because it is
    /// counter-intuitive: 5 distinct values spanning a range of 4 need 3 bits
    /// each (384 B for 1000 rows) against the dictionary's 8·5 + 4·1000 = 4040 B.
    /// Low cardinality alone does not imply dictionary encoding — low cardinality
    /// with a *wide* span does, because then the frame width is the full 64 bits.
    #[test]
    fn auto_encode_chooses_dict_for_wide_spread_low_cardinality() {
        // 5 distinct values spread across the u64 domain: frame width is 64, so
        // the frame saves nothing and the dictionary's 4 bytes/row wins.
        // Order-shuffled so RLE cannot win instead.
        let distinct = [0u64, u64::MAX / 4, u64::MAX / 2, u64::MAX / 4 * 3, u64::MAX];
        let values: Vec<u64> = (0..1000).map(|i| distinct[(i * 7) % 5]).collect();
        let col = ColumnValues::auto_encode(values.clone());
        assert!(
            matches!(col, ColumnValues::DictEncoded { ref dict, .. } if dict.len() == 5),
            "expected dict, got encoding {}",
            col.encoding_byte()
        );
        assert_eq!(col.to_raw_vec(), values);
    }

    /// The counterpart: few distinct values packed close together go to `Frame`,
    /// not the dictionary, because 3 bits/row beats 4 bytes/row.
    #[test]
    fn narrow_spread_low_cardinality_prefers_frame_over_dict() {
        let values: Vec<u64> = (0u64..1000).map(|i| (i * 7) % 5 + 1_000_000).collect();
        let col = ColumnValues::auto_encode(values.clone());
        assert_eq!(
            col.encoding_byte(),
            ENCODING_FRAME,
            "5 near-adjacent values should be a frame, got encoding {}",
            col.encoding_byte()
        );
        assert!(
            col.payload_bytes() < 1000 * 4,
            "frame should beat the dictionary's 4 bytes/row, got {} B",
            col.payload_bytes()
        );
        assert_eq!(col.to_raw_vec(), values);
    }

    #[test]
    fn auto_encode_empty() {
        let col = ColumnValues::auto_encode(vec![]);
        assert_eq!(col.row_count(), 0);
        assert_eq!(col.zone_min_max(), (0, 0));
        assert!(col.to_raw_vec().is_empty());
    }

    #[test]
    fn zone_min_max_raw() {
        let col = ColumnValues::Raw(vec![10, 3, 50, 7]);
        assert_eq!(col.zone_min_max(), (3, 50));
    }

    #[test]
    fn get_round_trips() {
        let col = ColumnValues::auto_encode(vec![100, 200, 100, 300]);
        assert_eq!(col.get(0), 100);
        assert_eq!(col.get(1), 200);
        assert_eq!(col.get(2), 100);
        assert_eq!(col.get(3), 300);
    }

    #[test]
    fn dict_boundary_at_256() {
        // 256 distinct, high magnitude so no frame encoding undercuts the dict.
        let values: Vec<u64> = (0u64..256).map(|i| i * 1_000_000_007).collect();
        let (dict, _) = ColumnValues::build_dict(&values).expect("256 distinct must fit");
        assert_eq!(dict.len(), 256);
        // 257 distinct → dictionary is abandoned.
        let values2: Vec<u64> = (0u64..257).map(|i| i * 1_000_000_007).collect();
        assert!(
            ColumnValues::build_dict(&values2).is_none(),
            "257 distinct must exceed DICT_MAX_DISTINCT"
        );
    }
}

// ── encoding correctness ──────────────────────────────────────────────────────
//
// One property dominates: whatever encoding is chosen, decoding must reproduce
// the input exactly. A compression bug that loses or alters a value is
// indistinguishable from data loss, and unlike a performance regression it is
// silent. So every test here round-trips through `get`, `to_raw_vec`, AND
// `zone_min_max` — the three ways a value can be observed.

#[cfg(test)]
mod encoding_tests {
    use super::*;

    /// Value shapes chosen so that each encoding is the winner for at least one.
    fn shapes() -> Vec<(&'static str, Vec<u64>)> {
        vec![
            ("empty", vec![]),
            ("single", vec![42]),
            ("all-identical", vec![7; 500]),
            ("two-long-runs", {
                let mut v = vec![1u64; 300];
                v.extend(vec![2u64; 300]);
                v
            }),
            ("sequential", (0..500).collect()),
            ("sorted-sparse", (0..500).map(|i| i * 1_000_003).collect()),
            (
                "narrow-range",
                (0..500).map(|i| 1_000_000 + i % 100).collect(),
            ),
            ("low-cardinality", (0..500).map(|i| (i % 7) * 111).collect()),
            (
                "high-cardinality",
                (0..500).map(|i| i * 2_654_435_761).collect(),
            ),
            ("descending", (0..500).rev().collect()),
            ("u64-max", vec![u64::MAX, 0, u64::MAX, 1]),
            ("all-u64-max", vec![u64::MAX; 200]),
            ("zero-and-max", vec![0, u64::MAX]),
            ("block-boundary", (0..FRAME_BLOCK as u64 * 3 + 1).collect()),
            ("one-under-block", (0..FRAME_BLOCK as u64 - 1).collect()),
            ("exactly-one-block", (0..FRAME_BLOCK as u64).collect()),
            (
                "drifting",
                (0..500).map(|i| 5_000_000 + i * 3 + (i % 11)).collect(),
            ),
            (
                "alternating-extremes",
                (0..400)
                    .map(|i| if i % 2 == 0 { 0 } else { u64::MAX })
                    .collect(),
            ),
        ]
    }

    /// The headline property: encode → decode is the identity, for every shape,
    /// observed through all three read paths.
    #[test]
    fn auto_encode_round_trips_every_shape() {
        for (name, values) in shapes() {
            let col = ColumnValues::auto_encode(values.clone());

            assert_eq!(
                col.row_count(),
                values.len(),
                "{name}: row_count wrong (encoding {})",
                col.encoding_byte()
            );
            assert_eq!(
                col.to_raw_vec(),
                values,
                "{name}: to_raw_vec mismatch (encoding {})",
                col.encoding_byte()
            );
            for (i, &want) in values.iter().enumerate() {
                assert_eq!(
                    col.get(i),
                    want,
                    "{name}: get({i}) wrong (encoding {})",
                    col.encoding_byte()
                );
            }

            let (zmin, zmax) = col.zone_min_max();
            if values.is_empty() {
                assert_eq!((zmin, zmax), (0, 0), "{name}: empty zone");
            } else {
                assert_eq!(
                    (zmin, zmax),
                    (
                        values.iter().copied().min().unwrap(),
                        values.iter().copied().max().unwrap()
                    ),
                    "{name}: zone_min_max wrong (encoding {})",
                    col.encoding_byte()
                );
            }
        }
    }

    /// Every variant round-trips when constructed directly, not only when chosen.
    ///
    /// `auto_encode` picks one encoding per shape, so a variant that is never
    /// optimal would otherwise go untested. Forcing each one closes that gap.
    #[test]
    fn every_variant_round_trips_when_forced() {
        let values: Vec<u64> = (0..300).map(|i| 900_000 + (i * 13) % 250).collect();

        let raw = ColumnValues::Raw(values.clone());
        let (dict, indices) = ColumnValues::build_dict(&values).unwrap();
        let dictc = ColumnValues::DictEncoded { dict, indices };
        let (rv, re) = ColumnValues::build_rle(&values).unwrap();
        let rle = ColumnValues::Rle {
            values: rv,
            run_ends: re,
        };
        let min = values.iter().copied().min().unwrap();
        let max = values.iter().copied().max().unwrap();
        let fw = bitpack::bits_needed(max - min);
        let frame = ColumnValues::Frame {
            base: min,
            bit_width: fw,
            packed: bitpack::pack(&values.iter().map(|v| v - min).collect::<Vec<_>>(), fw),
            row_count: values.len(),
        };
        let bw = ColumnValues::blocked_frame_width(&values);
        let (bases, offs) = ColumnValues::blocked_frame_parts(&values);
        let blocked = ColumnValues::BlockedFrame {
            block_bases: bases,
            bit_width: bw,
            packed: bitpack::pack(&offs, bw),
            row_count: values.len(),
        };

        for col in [raw, dictc, rle, frame, blocked] {
            let tag = col.encoding_byte();
            assert_eq!(col.row_count(), values.len(), "encoding {tag}: row_count");
            assert_eq!(col.to_raw_vec(), values, "encoding {tag}: to_raw_vec");
            assert_eq!(col.zone_min_max(), (min, max), "encoding {tag}: zone");
            for (i, &want) in values.iter().enumerate() {
                assert_eq!(col.get(i), want, "encoding {tag}: get({i})");
            }
            assert!(col.payload_bytes() > 0, "encoding {tag}: zero payload");
        }
    }

    /// `auto_encode` must actually pick a smallest-cost encoding, not merely a
    /// correct one. Without this, a selection bug is invisible — every encoding
    /// decodes correctly, so only the size reveals a wrong choice.
    #[test]
    fn auto_encode_is_not_beaten_by_another_encoding() {
        for (name, values) in shapes() {
            if values.is_empty() {
                continue;
            }
            let chosen = ColumnValues::auto_encode(values.clone());
            let chosen_bytes = chosen.payload_bytes();

            // Rebuild every legal alternative and compare.
            let mut alternatives: Vec<(&str, usize)> =
                vec![("raw", ColumnValues::Raw(values.clone()).payload_bytes())];
            if let Some((d, i)) = ColumnValues::build_dict(&values) {
                alternatives.push((
                    "dict",
                    ColumnValues::DictEncoded {
                        dict: d,
                        indices: i,
                    }
                    .payload_bytes(),
                ));
            }
            let min = values.iter().copied().min().unwrap();
            let max = values.iter().copied().max().unwrap();
            let fw = bitpack::bits_needed(max - min);
            alternatives.push(("frame", 8 + bitpack::packed_words(values.len(), fw) * 8));
            let bw = ColumnValues::blocked_frame_width(&values);
            alternatives.push((
                "blocked",
                values.len().div_ceil(FRAME_BLOCK) * 8
                    + bitpack::packed_words(values.len(), bw) * 8,
            ));

            // RLE is excluded from this comparison on purpose: it is allowed to be
            // smaller and still not chosen, because it costs the O(1) guarantee.
            for (alt_name, alt_bytes) in alternatives {
                assert!(
                    chosen_bytes <= alt_bytes,
                    "{name}: chose encoding {} at {chosen_bytes} B but {alt_name} was {alt_bytes} B",
                    chosen.encoding_byte()
                );
            }
        }
    }

    /// RLE is chosen when it halves the column, and declined when it does not —
    /// the O(1) trade being deliberate rather than accidental.
    #[test]
    fn rle_selected_only_when_it_wins_big() {
        // 2000 rows, 2 runs: RLE is 24 B against 8 B for an all-identical frame…
        // so use two DISTINCT long runs where frame cannot collapse to zero width.
        let mut two_runs = vec![1_000_000u64; 1000];
        two_runs.extend(vec![9_000_000u64; 1000]);
        let col = ColumnValues::auto_encode(two_runs.clone());
        assert_eq!(
            col.encoding_byte(),
            ENCODING_RLE,
            "two long runs should be RLE, got {}",
            col.encoding_byte()
        );
        assert_eq!(col.to_raw_vec(), two_runs);

        // Alternating values: every row is its own run, so RLE is 3× raw and must
        // be declined.
        let alternating: Vec<u64> = (0..1000).map(|i| if i % 2 == 0 { 5 } else { 9 }).collect();
        let col = ColumnValues::auto_encode(alternating.clone());
        assert_ne!(
            col.encoding_byte(),
            ENCODING_RLE,
            "run-per-row must not be RLE"
        );
        assert_eq!(col.to_raw_vec(), alternating);
    }

    /// The encoding actually beats raw on the shapes it exists for. This is the
    /// only test that asserts a compression *ratio*, and it is deliberately
    /// loose — the point is that the encoding is doing something, not that it
    /// hits a specific number.
    #[test]
    fn compression_beats_raw_on_targeted_shapes() {
        let cases: Vec<(&str, Vec<u64>, f64)> = vec![
            // Sorted dense: blocked frame → tiny in-block range.
            ("sequential", (0..4096).collect(), 0.15),
            // Narrow range around a large base: whole-column frame.
            (
                "narrow-range",
                (0..4096).map(|i| 10_000_000_000 + i % 1000).collect(),
                0.25,
            ),
            // Long runs: RLE.
            (
                "few-runs",
                (0..4096).map(|i| (i / 512) * 7_777_777).collect(),
                0.05,
            ),
            // Low cardinality, scattered: dictionary.
            (
                "low-card",
                (0..4096).map(|i| (i * 37 % 200) * 1_000_000_007).collect(),
                0.55,
            ),
        ];

        for (name, values, max_ratio) in cases {
            let col = ColumnValues::auto_encode(values.clone());
            let raw_bytes = values.len() * 8;
            let ratio = col.payload_bytes() as f64 / raw_bytes as f64;
            assert!(
                ratio <= max_ratio,
                "{name}: encoding {} gave ratio {ratio:.3}, wanted <= {max_ratio}",
                col.encoding_byte()
            );
            assert_eq!(col.to_raw_vec(), values, "{name}: round-trip");
        }
    }

    /// High-entropy data must fall back to `Raw` rather than inflate.
    ///
    /// The failure mode worth guarding: a scheme that always "compresses" and so
    /// makes incompressible columns larger than storing them plainly.
    #[test]
    fn incompressible_data_stays_raw() {
        // Full-width pseudo-random values: no narrow range, no runs, 4096 distinct.
        let values: Vec<u64> = (0..4096u64)
            .map(|i| {
                let mut x = i.wrapping_mul(0x9E37_79B9_7F4A_7C15);
                x ^= x >> 29;
                x = x.wrapping_mul(0xBF58_476D_1CE4_E5B9);
                x ^ (x >> 32)
            })
            .collect();
        let col = ColumnValues::auto_encode(values.clone());
        assert_eq!(
            col.encoding_byte(),
            ENCODING_RAW,
            "incompressible data got encoding {}",
            col.encoding_byte()
        );
        assert_eq!(
            col.payload_bytes(),
            values.len() * 8,
            "raw must not inflate"
        );
        assert_eq!(col.to_raw_vec(), values);
    }

    /// A single u64 span must never overflow while computing `max - min`.
    ///
    /// `[0, u64::MAX]` has a range of `u64::MAX`, needing all 64 bits. A frame
    /// encoding here saves nothing and must not wrap.
    #[test]
    fn full_domain_range_does_not_overflow() {
        let values = vec![0u64, u64::MAX, 1, u64::MAX - 1];
        let col = ColumnValues::auto_encode(values.clone());
        assert_eq!(col.to_raw_vec(), values);
        assert_eq!(col.zone_min_max(), (0, u64::MAX));
        // 64-bit width means frame costs 8 B more than raw, so raw or dict wins.
        assert_ne!(col.encoding_byte(), ENCODING_FRAME);
    }

    /// `zone_min_max` on a frame must report the widest offset PRESENT, not the
    /// widest the bit width could hold.
    ///
    /// `bits_needed` rounds up to a whole bit, so `base + 2^w - 1` overstates the
    /// maximum — e.g. values 0..=100 need 7 bits but 2^7-1 = 127. Overstating it
    /// is safe for correctness (the zone stays a superset) but it weakens
    /// pruning for free, so it is asserted tight.
    #[test]
    fn frame_zone_max_is_tight_not_width_ceiling() {
        let values: Vec<u64> = (0..=100).collect();
        let fw = bitpack::bits_needed(100);
        assert_eq!(fw, 7, "100 needs 7 bits");
        let frame = ColumnValues::Frame {
            base: 0,
            bit_width: fw,
            packed: bitpack::pack(&values, fw),
            row_count: values.len(),
        };
        let (_, zmax) = frame.zone_min_max();
        assert_eq!(zmax, 100, "zone max must be 100, not the 7-bit ceiling 127");
    }

    /// RLE `get` is O(log r), which means the binary search must land on the
    /// right run at every boundary — the classic off-by-one site.
    #[test]
    fn rle_get_lands_correctly_at_every_run_boundary() {
        // Runs of deliberately uneven length, including length 1.
        let spec: Vec<(u64, usize)> = vec![(10, 1), (20, 5), (30, 1), (40, 100), (50, 2), (60, 1)];
        let mut values = Vec::new();
        for (v, n) in &spec {
            values.extend(std::iter::repeat_n(*v, *n));
        }
        let (rv, re) = ColumnValues::build_rle(&values).unwrap();
        assert_eq!(rv.len(), spec.len(), "run count wrong");
        let rle = ColumnValues::Rle {
            values: rv,
            run_ends: re,
        };
        for (i, &want) in values.iter().enumerate() {
            assert_eq!(rle.get(i), want, "index {i}");
        }
        assert_eq!(rle.row_count(), values.len());
        assert_eq!(rle.to_raw_vec(), values);
    }

    /// Blocked frame boundaries: the base index is `row / FRAME_BLOCK`, so the
    /// first and last row of every block are where an off-by-one shows up.
    #[test]
    fn blocked_frame_correct_at_block_edges() {
        for n in [
            1,
            FRAME_BLOCK - 1,
            FRAME_BLOCK,
            FRAME_BLOCK + 1,
            FRAME_BLOCK * 3,
            FRAME_BLOCK * 3 + 1,
        ] {
            // Big jumps between blocks so a wrong base is unmistakable.
            let values: Vec<u64> = (0..n as u64).map(|i| i * 1_000_000).collect();
            let bw = ColumnValues::blocked_frame_width(&values);
            let (bases, offs) = ColumnValues::blocked_frame_parts(&values);
            assert_eq!(bases.len(), n.div_ceil(FRAME_BLOCK), "n={n}: base count");
            let col = ColumnValues::BlockedFrame {
                block_bases: bases,
                bit_width: bw,
                packed: bitpack::pack(&offs, bw),
                row_count: n,
            };
            for (i, &want) in values.iter().enumerate() {
                assert_eq!(col.get(i), want, "n={n}, index {i}");
            }
            assert_eq!(col.to_raw_vec(), values, "n={n}");
        }
    }

    /// Selection is deterministic: the same input always yields the same
    /// encoding. Required for byte-reproducible `.vvsst` files, and it would be
    /// easy to lose by iterating a HashMap somewhere in the cost computation.
    #[test]
    fn selection_is_deterministic() {
        for (name, values) in shapes() {
            let first = ColumnValues::auto_encode(values.clone());
            for _ in 0..8 {
                let again = ColumnValues::auto_encode(values.clone());
                assert_eq!(
                    again.encoding_byte(),
                    first.encoding_byte(),
                    "{name}: encoding choice varied between runs"
                );
                assert_eq!(again, first, "{name}: encoded representation varied");
            }
        }
    }
}
