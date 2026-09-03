use dashmap::DashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use thiserror::Error;

#[derive(Debug, Error, PartialEq)]
pub enum FenceError {
    #[error("stale epoch write: write={write}, current={current}")]
    StaleEpoch { write: u64, current: u64 },
}

pub struct EpochFence {
    current_epoch: AtomicU64,
    term_to_epoch: DashMap<u64, u64>,
}

impl EpochFence {
    pub fn new() -> Self {
        EpochFence {
            current_epoch: AtomicU64::new(0),
            term_to_epoch: DashMap::new(),
        }
    }

    pub fn advance(&self, raft_term: u64) -> u64 {
        // Check if already have an epoch for this term.
        if let Some(ep) = self.term_to_epoch.get(&raft_term) {
            return *ep;
        }
        let new_epoch = self.current_epoch.fetch_add(1, Ordering::SeqCst) + 1;
        self.term_to_epoch.insert(raft_term, new_epoch);
        new_epoch
    }

    pub fn current(&self) -> u64 {
        self.current_epoch.load(Ordering::SeqCst)
    }

    pub fn validate(&self, epoch: u64) -> Result<(), FenceError> {
        let cur = self.current();
        if epoch < cur {
            Err(FenceError::StaleEpoch {
                write: epoch,
                current: cur,
            })
        } else {
            Ok(())
        }
    }

    pub fn epoch_for_term(&self, term: u64) -> Option<u64> {
        self.term_to_epoch.get(&term).map(|v| *v)
    }

    pub fn is_fenced(&self, write_epoch: u64) -> bool {
        write_epoch < self.current()
    }
}

impl Default for EpochFence {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_fence_epoch_zero() {
        let f = EpochFence::new();
        assert_eq!(f.current(), 0);
    }

    #[test]
    fn advance_increments() {
        let f = EpochFence::new();
        let e = f.advance(1);
        assert_eq!(e, 1);
        assert_eq!(f.current(), 1);
    }

    #[test]
    fn advance_with_same_term_returns_same_epoch() {
        let f = EpochFence::new();
        let e1 = f.advance(1);
        let e2 = f.advance(1);
        assert_eq!(e1, e2);
    }

    #[test]
    fn advance_with_higher_term_returns_higher_epoch() {
        let f = EpochFence::new();
        let e1 = f.advance(1);
        let e2 = f.advance(2);
        assert!(e2 > e1);
    }

    #[test]
    fn validate_current_epoch_ok() {
        let f = EpochFence::new();
        let e = f.advance(1);
        assert!(f.validate(e).is_ok());
    }

    #[test]
    fn validate_stale_epoch_err() {
        let f = EpochFence::new();
        f.advance(1);
        f.advance(2); // current = 2
        assert!(f.validate(0).is_err());
    }

    #[test]
    fn is_fenced_stale_true() {
        let f = EpochFence::new();
        f.advance(1); // current = 1
        assert!(f.is_fenced(0));
    }

    #[test]
    fn is_fenced_current_false() {
        let f = EpochFence::new();
        let e = f.advance(1);
        assert!(!f.is_fenced(e));
    }

    #[test]
    fn epoch_monotonic_across_advances() {
        let f = EpochFence::new();
        let e1 = f.advance(1);
        let e2 = f.advance(2);
        let e3 = f.advance(3);
        assert!(e1 < e2 && e2 < e3);
    }

    #[test]
    fn two_advances_strictly_increasing() {
        let f = EpochFence::new();
        let a = f.advance(10);
        let b = f.advance(11);
        assert!(b > a);
    }

    #[test]
    fn epoch_for_term_known_term_returns_some() {
        let f = EpochFence::new();
        f.advance(5);
        assert!(f.epoch_for_term(5).is_some());
    }

    #[test]
    fn epoch_for_term_unknown_returns_none() {
        let f = EpochFence::new();
        assert!(f.epoch_for_term(999).is_none());
    }

    #[test]
    fn fence_error_has_correct_write_current_fields() {
        let f = EpochFence::new();
        f.advance(1);
        f.advance(2); // current = 2
        match f.validate(0) {
            Err(FenceError::StaleEpoch { write, current }) => {
                assert_eq!(write, 0);
                assert_eq!(current, 2);
            }
            Ok(_) => panic!("expected error"),
        }
    }
}
