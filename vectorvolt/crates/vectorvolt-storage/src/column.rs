//! [`ColumnValues`] — in-memory column storage with optional dictionary encoding.
//!
//! ## Encodings
//!
//! | Variant | When used | Storage cost |
//! |---|---|---|
//! | `Raw` | > 256 distinct values | 8 bytes/row |
//! | `DictEncoded` | ≤ 256 distinct values | dict·8 + 4 bytes/row |
//!
//! Dictionary encoding pays off when rows >> distinct values — a typical
//! "status", "category", or "country_code" column.
//!
//! ## Zone min/max
//!
//! Both variants provide `zone_min_max()` for ZoneMap population at write time.
//! For `DictEncoded`, min/max are computed over the dictionary (not the index
//! array), so only O(distinct) rather than O(n).

use crate::format::{ENCODING_DICT, ENCODING_RAW};

/// Column data for one micro-partition, either raw or dictionary-encoded.
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
}

impl ColumnValues {
    /// Build a `ColumnValues` from a raw value vector, choosing the encoding
    /// automatically: dictionary if ≤ 256 distinct values, raw otherwise.
    pub fn auto_encode(values: Vec<u64>) -> Self {
        use std::collections::HashMap;
        use std::collections::hash_map::Entry;

        if values.is_empty() {
            return ColumnValues::Raw(values);
        }

        let mut index_map: HashMap<u64, u32> = HashMap::new();
        let mut dict: Vec<u64> = Vec::new();

        for &v in &values {
            if let Entry::Vacant(e) = index_map.entry(v) {
                if dict.len() >= 256 {
                    // Too many distinct values — fall back to raw.
                    return ColumnValues::Raw(values);
                }
                let idx = dict.len() as u32;
                dict.push(v);
                e.insert(idx);
            }
        }

        let indices: Vec<u32> = values.iter().map(|v| *index_map.get(v).unwrap()).collect();
        ColumnValues::DictEncoded { dict, indices }
    }

    /// Number of rows in this column.
    pub fn row_count(&self) -> usize {
        match self {
            ColumnValues::Raw(v) => v.len(),
            ColumnValues::DictEncoded { indices, .. } => indices.len(),
        }
    }

    /// `(min, max)` over all values — used for ZoneMap population.
    ///
    /// Returns `(0, 0)` if the column is empty.
    pub fn zone_min_max(&self) -> (u64, u64) {
        match self {
            ColumnValues::Raw(v) => {
                if v.is_empty() {
                    return (0, 0);
                }
                let min = v.iter().copied().min().unwrap();
                let max = v.iter().copied().max().unwrap();
                (min, max)
            }
            ColumnValues::DictEncoded { dict, .. } => {
                if dict.is_empty() {
                    return (0, 0);
                }
                let min = dict.iter().copied().min().unwrap();
                let max = dict.iter().copied().max().unwrap();
                (min, max)
            }
        }
    }

    /// Decode and return the value at `row`.
    pub fn get(&self, row: usize) -> u64 {
        match self {
            ColumnValues::Raw(v) => v[row],
            ColumnValues::DictEncoded { dict, indices } => dict[indices[row] as usize],
        }
    }

    /// Decode all rows into a flat `Vec<u64>` (allocates).
    pub fn to_raw_vec(&self) -> Vec<u64> {
        match self {
            ColumnValues::Raw(v) => v.clone(),
            ColumnValues::DictEncoded { dict, indices } => {
                indices.iter().map(|&i| dict[i as usize]).collect()
            }
        }
    }

    /// Wire encoding byte for the binary file format.
    pub fn encoding_byte(&self) -> u8 {
        match self {
            ColumnValues::Raw(_) => ENCODING_RAW,
            ColumnValues::DictEncoded { .. } => ENCODING_DICT,
        }
    }
}

// ── tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn auto_encode_chooses_dict_for_low_cardinality() {
        let values: Vec<u64> = (0u64..1000).map(|i| i % 5).collect(); // 5 distinct
        let col = ColumnValues::auto_encode(values.clone());
        assert!(matches!(col, ColumnValues::DictEncoded { ref dict, .. } if dict.len() == 5));
        // Decoding round-trips.
        assert_eq!(col.to_raw_vec(), values);
    }

    #[test]
    fn auto_encode_chooses_raw_for_high_cardinality() {
        let values: Vec<u64> = (0u64..1000).collect(); // 1000 distinct
        let col = ColumnValues::auto_encode(values.clone());
        assert!(matches!(col, ColumnValues::Raw(_)));
        assert_eq!(col.to_raw_vec(), values);
    }

    #[test]
    fn auto_encode_empty() {
        let col = ColumnValues::auto_encode(vec![]);
        assert_eq!(col.row_count(), 0);
    }

    #[test]
    fn zone_min_max_raw() {
        let col = ColumnValues::Raw(vec![10, 3, 50, 7]);
        assert_eq!(col.zone_min_max(), (3, 50));
    }

    #[test]
    fn zone_min_max_dict() {
        let col = ColumnValues::auto_encode(vec![10, 3, 50, 7, 10, 3]);
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
        // 256 distinct values → still fits (boundary case).
        let values: Vec<u64> = (0u64..256).collect();
        let col = ColumnValues::auto_encode(values.clone());
        assert!(matches!(col, ColumnValues::DictEncoded { .. }));
        assert_eq!(col.to_raw_vec(), values);

        // 257 distinct values → raw.
        let values2: Vec<u64> = (0u64..257).collect();
        let col2 = ColumnValues::auto_encode(values2.clone());
        assert!(matches!(col2, ColumnValues::Raw(_)));
    }
}
