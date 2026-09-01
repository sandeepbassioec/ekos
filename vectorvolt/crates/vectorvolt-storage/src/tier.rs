use crate::writer::Compression;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum StorageTier {
    L0 = 0,
    L1 = 1,
    L2 = 2,
    L3 = 3,
    L4 = 4,
}

#[derive(Debug, Clone)]
pub struct TierConfig {
    pub max_bytes: u64,
    pub compression: Compression,
    pub target_tier: StorageTier,
}

impl TierConfig {
    pub fn is_valid(&self) -> bool {
        self.max_bytes > 0
    }
}

#[derive(Debug, Clone)]
pub struct TieredStorageConfig {
    pub tiers: [TierConfig; 5],
}

impl Default for TieredStorageConfig {
    fn default() -> Self {
        Self {
            tiers: [
                TierConfig {
                    max_bytes: 64 * 1024 * 1024,
                    compression: Compression::None,
                    target_tier: StorageTier::L0,
                },
                TierConfig {
                    max_bytes: 1024 * 1024 * 1024,
                    compression: Compression::None,
                    target_tier: StorageTier::L1,
                },
                TierConfig {
                    max_bytes: 10 * 1024 * 1024 * 1024,
                    compression: Compression::Lz4,
                    target_tier: StorageTier::L2,
                },
                TierConfig {
                    max_bytes: 100 * 1024 * 1024 * 1024,
                    compression: Compression::Zstd { level: 3 },
                    target_tier: StorageTier::L3,
                },
                TierConfig {
                    max_bytes: u64::MAX,
                    compression: Compression::Zstd { level: 19 },
                    target_tier: StorageTier::L4,
                },
            ],
        }
    }
}

#[derive(Debug, Clone)]
pub struct TierStats {
    pub tier: StorageTier,
    pub bytes_used: u64,
    pub file_count: u32,
    pub row_count: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_builds() {
        let _ = TieredStorageConfig::default();
    }

    #[test]
    fn tier_ordering() {
        assert!(StorageTier::L0 < StorageTier::L4);
    }

    #[test]
    fn l0_max_bytes() {
        let cfg = TieredStorageConfig::default();
        assert_eq!(cfg.tiers[0].max_bytes, 64 * 1024 * 1024);
    }

    #[test]
    fn l4_max_bytes() {
        let cfg = TieredStorageConfig::default();
        assert_eq!(cfg.tiers[4].max_bytes, u64::MAX);
    }

    #[test]
    fn l2_is_lz4() {
        let cfg = TieredStorageConfig::default();
        assert!(matches!(cfg.tiers[2].compression, Compression::Lz4));
    }

    #[test]
    fn l4_is_zstd() {
        let cfg = TieredStorageConfig::default();
        assert!(matches!(cfg.tiers[4].compression, Compression::Zstd { .. }));
    }

    #[test]
    fn l0_is_none() {
        let cfg = TieredStorageConfig::default();
        assert!(matches!(cfg.tiers[0].compression, Compression::None));
    }

    #[test]
    fn tier_stats_default() {
        let s = TierStats {
            tier: StorageTier::L0,
            bytes_used: 0,
            file_count: 0,
            row_count: 0,
        };
        assert_eq!(s.bytes_used, 0);
    }

    #[test]
    fn all_5_tiers_distinct() {
        let tiers = [
            StorageTier::L0,
            StorageTier::L1,
            StorageTier::L2,
            StorageTier::L3,
            StorageTier::L4,
        ];
        let mut set = std::collections::HashSet::new();
        for t in &tiers {
            set.insert(*t as u8);
        }
        assert_eq!(set.len(), 5);
    }

    #[test]
    fn l1_max_bytes() {
        let cfg = TieredStorageConfig::default();
        assert_eq!(cfg.tiers[1].max_bytes, 1024 * 1024 * 1024);
    }

    #[test]
    fn l3_max_bytes() {
        let cfg = TieredStorageConfig::default();
        assert_eq!(cfg.tiers[3].max_bytes, 100 * 1024 * 1024 * 1024);
    }

    #[test]
    fn max_bytes_zero_invalid() {
        let tc = TierConfig {
            max_bytes: 0,
            compression: Compression::None,
            target_tier: StorageTier::L0,
        };
        assert!(!tc.is_valid());
    }

    #[test]
    fn tiered_config_total_capacity() {
        let cfg = TieredStorageConfig::default();
        let total = cfg.tiers[0].max_bytes
            + cfg.tiers[1].max_bytes
            + cfg.tiers[2].max_bytes
            + cfg.tiers[3].max_bytes;
        let expected = 64u64 * 1024 * 1024
            + 1024 * 1024 * 1024
            + 10 * 1024 * 1024 * 1024
            + 100 * 1024 * 1024 * 1024;
        assert_eq!(total, expected);
    }
}
