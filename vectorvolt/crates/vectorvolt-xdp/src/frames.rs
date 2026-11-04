//! UMEM frame allocator — pure bookkeeping, no kernel interaction, any OS.
//!
//! TX frames are taken from the allocator, handed to the kernel via the TX
//! ring, and returned when their address shows up on the completion ring.
//! Double frees and out-of-range frees are detected rather than silently
//! corrupting the free list.

use std::ops::Range;

/// Error returned by [`FrameAllocator::free`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrameError {
    /// The frame index is outside this allocator's range.
    OutOfRange,
    /// The frame is already free.
    DoubleFree,
}

/// Free-list allocator over a contiguous range of UMEM frame indices.
#[derive(Debug)]
pub struct FrameAllocator {
    start: u32,
    free_list: Vec<u32>,
    is_free: Vec<bool>,
}

impl FrameAllocator {
    /// Manage frames `range.start..range.end`, all initially free.
    pub fn new(range: Range<u32>) -> Self {
        let n = range.end.saturating_sub(range.start);
        // Reverse so that `pop` hands out the lowest index first.
        let free_list: Vec<u32> = (range.start..range.start + n).rev().collect();
        Self {
            start: range.start,
            free_list,
            is_free: vec![true; n as usize],
        }
    }

    /// Take one free frame.
    pub fn alloc(&mut self) -> Option<u32> {
        let idx = self.free_list.pop()?;
        self.is_free[(idx - self.start) as usize] = false;
        Some(idx)
    }

    /// Return a frame to the pool.
    pub fn free(&mut self, idx: u32) -> Result<(), FrameError> {
        let off = idx.checked_sub(self.start).ok_or(FrameError::OutOfRange)? as usize;
        let slot = self.is_free.get_mut(off).ok_or(FrameError::OutOfRange)?;
        if *slot {
            return Err(FrameError::DoubleFree);
        }
        *slot = true;
        self.free_list.push(idx);
        Ok(())
    }

    /// Number of frames currently free.
    pub fn available(&self) -> usize {
        self.free_list.len()
    }

    /// Total frames managed.
    pub fn capacity(&self) -> usize {
        self.is_free.len()
    }

    /// True if `idx` belongs to this allocator's range.
    pub fn owns(&self, idx: u32) -> bool {
        idx.checked_sub(self.start)
            .is_some_and(|o| (o as usize) < self.is_free.len())
    }
}

/// UMEM frame index containing byte offset `addr` (chunk-aligned mode).
pub fn frame_of_addr(addr: u64, frame_size: u32) -> u32 {
    (addr / frame_size as u64) as u32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allocates_distinct_frames_lowest_first() {
        let mut a = FrameAllocator::new(4..8);
        assert_eq!(a.capacity(), 4);
        let got: Vec<u32> = std::iter::from_fn(|| a.alloc()).collect();
        assert_eq!(got, vec![4, 5, 6, 7]);
        assert_eq!(a.available(), 0);
        assert_eq!(a.alloc(), None);
    }

    #[test]
    fn freed_frame_is_reused() {
        let mut a = FrameAllocator::new(0..2);
        let f0 = a.alloc().unwrap();
        let _f1 = a.alloc().unwrap();
        a.free(f0).unwrap();
        assert_eq!(a.available(), 1);
        assert_eq!(a.alloc(), Some(f0));
    }

    #[test]
    fn double_free_is_rejected() {
        let mut a = FrameAllocator::new(0..2);
        let f = a.alloc().unwrap();
        a.free(f).unwrap();
        assert_eq!(a.free(f), Err(FrameError::DoubleFree));
        // Never allocated.
        assert_eq!(a.free(1), Err(FrameError::DoubleFree));
        assert_eq!(a.available(), 2);
    }

    #[test]
    fn out_of_range_free_is_rejected() {
        let mut a = FrameAllocator::new(4..6);
        assert_eq!(a.free(3), Err(FrameError::OutOfRange));
        assert_eq!(a.free(6), Err(FrameError::OutOfRange));
        assert!(a.owns(5));
        assert!(!a.owns(6));
        assert!(!a.owns(0));
    }

    #[test]
    fn empty_range_is_empty() {
        let mut a = FrameAllocator::new(3..3);
        assert_eq!(a.alloc(), None);
        assert_eq!(a.capacity(), 0);
    }

    #[test]
    fn tx_frames_cycle_through_completion() {
        let mut a = FrameAllocator::new(0..4);
        let in_flight: Vec<u32> = (0..4).map(|_| a.alloc().unwrap()).collect();
        assert_eq!(a.alloc(), None);
        for &f in in_flight.iter().rev() {
            a.free(f).unwrap();
        }
        let mut again: Vec<u32> = (0..4).map(|_| a.alloc().unwrap()).collect();
        again.sort_unstable();
        assert_eq!(again, vec![0, 1, 2, 3]);
    }

    #[test]
    fn addr_to_frame() {
        assert_eq!(frame_of_addr(0, 2048), 0);
        assert_eq!(frame_of_addr(2047, 2048), 0);
        assert_eq!(frame_of_addr(2048 + 256, 2048), 1);
    }
}
