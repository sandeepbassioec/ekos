//! [`RowDescriptor`] — a bit-packed `u64` encoding the physical location of a row.
//!
//! ```text
//! bits 63-48  file_id      (u16) — SSTable file number, 0-65535
//! bits 47-32  partition_id (u16) — micro-partition within file, 0-65535
//! bits 31-0   row_offset   (u32) — row index within partition, 0-4294967295
//! ```
//!
//! Fitting the entire row location in one `u64` means a primary-key lookup
//! requires only one cache-line read after the Cuckoo hash probe.

/// Physical address of a row: file + partition + offset, packed into 8 bytes.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct RowDescriptor(u64);

impl RowDescriptor {
    /// Construct from components. Panics in debug mode if values overflow their bit widths.
    #[inline]
    pub fn new(file_id: u16, partition_id: u16, row_offset: u32) -> Self {
        let v = ((file_id as u64) << 48) | ((partition_id as u64) << 32) | (row_offset as u64);
        RowDescriptor(v)
    }

    /// SSTable file number.
    #[inline]
    pub fn file_id(self) -> u16 {
        (self.0 >> 48) as u16
    }

    /// Micro-partition index within the file.
    #[inline]
    pub fn partition_id(self) -> u16 {
        ((self.0 >> 32) & 0xFFFF) as u16
    }

    /// Row index within the micro-partition.
    #[inline]
    pub fn row_offset(self) -> u32 {
        self.0 as u32
    }

    /// Raw packed value (useful for serialization).
    #[inline]
    pub fn as_u64(self) -> u64 {
        self.0
    }

    /// Reconstruct from a raw packed value.
    #[inline]
    pub const fn from_u64(v: u64) -> Self {
        RowDescriptor(v)
    }
}

impl std::fmt::Display for RowDescriptor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "file={} partition={} offset={}",
            self.file_id(),
            self.partition_id(),
            self.row_offset()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip() {
        let rd = RowDescriptor::new(7, 42, 1_000_000);
        assert_eq!(rd.file_id(), 7);
        assert_eq!(rd.partition_id(), 42);
        assert_eq!(rd.row_offset(), 1_000_000);
    }

    #[test]
    fn max_values() {
        let rd = RowDescriptor::new(u16::MAX, u16::MAX, u32::MAX);
        assert_eq!(rd.file_id(), u16::MAX);
        assert_eq!(rd.partition_id(), u16::MAX);
        assert_eq!(rd.row_offset(), u32::MAX);
    }

    #[test]
    fn zero_values() {
        let rd = RowDescriptor::new(0, 0, 0);
        assert_eq!(rd.as_u64(), 0);
        assert_eq!(RowDescriptor::from_u64(0), rd);
    }

    #[test]
    fn u64_round_trip() {
        let rd = RowDescriptor::new(3, 17, 99);
        assert_eq!(RowDescriptor::from_u64(rd.as_u64()), rd);
    }

    #[test]
    fn fields_do_not_bleed_into_each_other() {
        // file_id=1, rest=0
        let rd = RowDescriptor::new(1, 0, 0);
        assert_eq!(rd.partition_id(), 0);
        assert_eq!(rd.row_offset(), 0);

        // partition_id=1, rest=0
        let rd = RowDescriptor::new(0, 1, 0);
        assert_eq!(rd.file_id(), 0);
        assert_eq!(rd.row_offset(), 0);
    }
}
