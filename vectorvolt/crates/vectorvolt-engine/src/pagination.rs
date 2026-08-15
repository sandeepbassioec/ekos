//! Cursor-based pagination over a filtered row set.
//!
//! ## API
//!
//! ```text
//! scan_page(filter_col, lo, hi, after_pk, limit) -> PageResult
//! ```
//!
//! Returns up to `limit` rows from the filtered set whose pk is strictly
//! greater than `after_pk`, ordered by ascending pk.  A `next_cursor` of
//! `Some(pk)` indicates that more rows exist; pass it as `after_pk` on the
//! next call.  `None` means the result set is exhausted.
//!
//! Pass `after_pk = 0` to start from the beginning (pk 0 is never a valid
//! primary key in VectorVolt because pk is stored as `values[0]` and the
//! engine inserts it verbatim — callers must use pk > 0 for real rows).
//!
//! ## Duplicate-PK safety
//!
//! `scan()` can return the same PK from multiple partition files when a row
//! has been updated since the last compaction.  `scan_page` deduplicates by
//! pk (keeping the highest-valued row for determinism) before applying the
//! cursor and limit, so callers always see at most one row per PK.
//!
//! ## Edge cases
//!
//! | Situation | Behaviour |
//! |-----------|-----------|
//! | `limit = 0` | Returns empty rows, `next_cursor = None` |
//! | `after_pk` past all matching PKs | Returns empty rows, `next_cursor = None` |
//! | Exact `limit` rows remain | Returns them, `next_cursor = None` |
//! | More than `limit` rows remain | Returns first `limit`, `next_cursor = Some(last_pk)` |

use crate::engine::TableEngine;
use crate::{EngineResult, EngineRow};

/// Result of a paginated scan.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PageResult {
    /// Rows in ascending pk order.
    pub rows: Vec<EngineRow>,
    /// Pass this as `after_pk` on the next call to continue iteration.
    /// `None` means the result set is exhausted.
    pub next_cursor: Option<u64>,
}

impl TableEngine {
    /// Return up to `limit` rows from `filter_col ∈ [lo, hi]` with pk > `after_pk`.
    ///
    /// Rows are ordered by ascending pk.  See module-level docs for cursor
    /// semantics and edge cases.
    pub fn scan_page(
        &self,
        filter_col: usize,
        lo: u64,
        hi: u64,
        after_pk: u64,
        limit: usize,
    ) -> EngineResult<PageResult> {
        self.check_col(filter_col)?;

        if limit == 0 {
            return Ok(PageResult {
                rows: Vec::new(),
                next_cursor: None,
            });
        }

        // scan_deduped gives one authoritative row per PK (re-verified via
        // get()), so no further dedup is needed here.
        let mut rows = self.scan_deduped(filter_col, lo, hi)?;
        rows.sort_unstable_by_key(|r| r.pk);
        let start = rows.partition_point(|r| r.pk <= after_pk);
        let rows = &rows[start..];

        if rows.len() <= limit {
            return Ok(PageResult {
                rows: rows.to_vec(),
                next_cursor: None,
            });
        }

        // More rows remain after this page.
        let page = rows[..limit].to_vec();
        let next_cursor = Some(page.last().unwrap().pk);
        Ok(PageResult {
            rows: page,
            next_cursor,
        })
    }
}
