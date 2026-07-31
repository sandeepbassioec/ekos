//! Write buffer — accumulates INSERT rows in memory until flush threshold.
//!
//! Rows are stored as `(pk, values)` pairs.  When `len() >= capacity` the
//! caller should call `TxnManager::flush()` to persist to a MicroPartition.

pub struct WriteBuffer {
    rows: Vec<(u64, Vec<u64>)>,
    capacity: usize,
    pub schema_width: usize,
}

impl WriteBuffer {
    pub fn new(capacity: usize, schema_width: usize) -> Self {
        WriteBuffer {
            rows: Vec::with_capacity(capacity),
            capacity,
            schema_width,
        }
    }

    /// Append a row. Returns `true` if the buffer is now at or above capacity.
    pub fn push(&mut self, pk: u64, values: Vec<u64>) -> bool {
        self.rows.push((pk, values));
        self.rows.len() >= self.capacity
    }

    pub fn is_full(&self) -> bool {
        self.rows.len() >= self.capacity
    }

    pub fn len(&self) -> usize {
        self.rows.len()
    }

    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }

    /// Drain all rows, leaving the buffer empty.
    /// The row at `offset`, if it is there and belongs to `pk`.
    ///
    /// The pk check is not paranoia. A `RowDescriptor` records a buffer offset,
    /// and a flush empties the buffer — so an offset captured before a flush can
    /// point at a slot that a *different* row now occupies. Returning that row
    /// would silently answer a lookup with someone else's data, which is worse
    /// than returning nothing. Verifying the key makes a stale offset a miss
    /// instead of a wrong answer.
    pub fn row_for(&self, pk: u64, offset: usize) -> Option<&[u64]> {
        match self.rows.get(offset) {
            Some((k, values)) if *k == pk => Some(values),
            _ => None,
        }
    }

    pub fn drain(&mut self) -> Vec<(u64, Vec<u64>)> {
        std::mem::take(&mut self.rows)
    }
}

// ── tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn push_and_drain() {
        let mut buf = WriteBuffer::new(3, 2);
        assert!(!buf.push(1, vec![10, 20]));
        assert!(!buf.push(2, vec![30, 40]));
        assert!(buf.push(3, vec![50, 60])); // hits capacity
        assert!(buf.is_full());

        let rows = buf.drain();
        assert_eq!(rows.len(), 3);
        assert!(buf.is_empty());
    }

    #[test]
    fn drain_empty_is_ok() {
        let mut buf = WriteBuffer::new(4, 1);
        assert!(buf.drain().is_empty());
    }
}
