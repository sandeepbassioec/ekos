use std::collections::HashMap;

use vectorvolt_index::{CuckooFilter, CuckooMap, SecondaryIndex, ZoneMap};
use vectorvolt_storage::PartitionReader;

/// All index and storage structures for one table, held in memory.
///
/// The query engine borrows from this catalog for every query operation.
pub struct TableCatalog {
    /// PK → RowDescriptor (file_id, partition_id, row_offset).
    pub pk_index: CuckooMap,
    /// column-value hash → RoaringBitmap of u32 PKs (lower 32 bits of each PK).
    pub secondary_index: SecondaryIndex,
    /// partition_id → (zone_min, zone_max) for zone-map pruning.
    pub zone_map: ZoneMap,
    /// Probabilistic FK existence filter.
    pub fk_filter: CuckooFilter,
    /// (file_id, partition_id) → open PartitionReader.
    pub partitions: HashMap<(u16, u16), PartitionReader>,
}

impl TableCatalog {
    pub fn new(
        pk_index: CuckooMap,
        secondary_index: SecondaryIndex,
        zone_map: ZoneMap,
        fk_filter: CuckooFilter,
        partitions: HashMap<(u16, u16), PartitionReader>,
    ) -> Self {
        TableCatalog {
            pk_index,
            secondary_index,
            zone_map,
            fk_filter,
            partitions,
        }
    }
}
