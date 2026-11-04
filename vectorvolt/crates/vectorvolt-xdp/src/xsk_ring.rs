//! Memory-mapped AF_XDP rings shared with the kernel (Linux only).
//!
//! The producer/consumer words live in the mapping; the cached-index logic is
//! in [`RingCursor`](crate::rings::RingCursor).  The consumer of a ring reads
//! the remote producer with `Acquire` and publishes its consumer index with
//! `Release`; the producer does the converse.

use std::marker::PhantomData;
use std::os::unix::io::RawFd;
use std::sync::atomic::{AtomicU32, Ordering};

use crate::rings::{RingCursor, RingOffset};
use crate::{XdpError, XdpResult};

/// `XDP_RING_NEED_WAKEUP` bit in a ring's `flags` word.
const XDP_RING_NEED_WAKEUP: u32 = 1;

/// `mmap` page offsets selecting each ring (`XDP_PGOFF_*`).
pub const PGOFF_RX_RING: i64 = 0;
pub const PGOFF_TX_RING: i64 = 0x8000_0000;
pub const PGOFF_FILL_RING: i64 = 0x1_0000_0000;
pub const PGOFF_COMPLETION_RING: i64 = 0x1_8000_0000;

/// One mmap'd ring whose entries are `T` (`u64` addresses or `XdpDesc`).
pub struct MmapRing<T: Copy> {
    map: *mut libc::c_void,
    map_len: usize,
    producer: *const AtomicU32,
    consumer: *const AtomicU32,
    flags: *const AtomicU32,
    descs: *mut T,
    cursor: RingCursor,
    _marker: PhantomData<T>,
}

// SAFETY: the mapping is owned exclusively by this struct; access to the shared
// words is through atomics and `&mut self` serialises our side of the ring.
unsafe impl<T: Copy + Send> Send for MmapRing<T> {}

impl<T: Copy> MmapRing<T> {
    /// Map a ring of `size` entries of the socket `fd` at page offset `pgoff`.
    /// `has_flags` says whether the kernel provides the ring `flags` word.
    pub fn map(
        fd: RawFd,
        size: u32,
        off: &RingOffset,
        has_flags: bool,
        pgoff: i64,
    ) -> XdpResult<Self> {
        let map_len = off.desc as usize + size as usize * std::mem::size_of::<T>();
        // SAFETY: plain shared mapping of a kernel-provided region.
        let map = unsafe {
            libc::mmap64(
                std::ptr::null_mut(),
                map_len,
                libc::PROT_READ | libc::PROT_WRITE,
                libc::MAP_SHARED | libc::MAP_POPULATE,
                fd,
                pgoff,
            )
        };
        if map == libc::MAP_FAILED {
            return Err(XdpError::Os(
                std::io::Error::last_os_error().raw_os_error().unwrap_or(0),
            ));
        }
        let base = map as *mut u8;
        // SAFETY: the offsets come from the kernel for exactly this mapping.
        let (producer, consumer, flags, descs) = unsafe {
            (
                base.add(off.producer as usize) as *const AtomicU32,
                base.add(off.consumer as usize) as *const AtomicU32,
                if has_flags {
                    base.add(off.flags as usize) as *const AtomicU32
                } else {
                    std::ptr::null()
                },
                base.add(off.desc as usize) as *mut T,
            )
        };
        // SAFETY: producer/consumer point into the mapping created above.
        let (p, c) = unsafe {
            (
                (*producer).load(Ordering::Acquire),
                (*consumer).load(Ordering::Acquire),
            )
        };
        Ok(Self {
            map,
            map_len,
            producer,
            consumer,
            flags,
            descs,
            cursor: RingCursor::with_indices(size, p, c),
            _marker: PhantomData,
        })
    }

    /// Producer side: enqueue one entry.  Returns `false` if the ring is full.
    pub fn push(&mut self, value: T) -> bool {
        // SAFETY: pointers are valid for the lifetime of the mapping.
        let consumer = unsafe { &*self.consumer };
        if self.cursor.prod_free(|| consumer.load(Ordering::Acquire)) == 0 {
            return false;
        }
        // SAFETY: slot index is masked into the ring.
        unsafe {
            self.descs
                .add(self.cursor.prod_slot() as usize)
                .write(value)
        };
        self.cursor.advance_prod(1);
        // SAFETY: as above; Release publishes the descriptor write.
        unsafe { &*self.producer }.store(self.cursor.cached_prod(), Ordering::Release);
        true
    }

    /// Consumer side: dequeue one entry, or `None` if the ring is empty.
    pub fn pop(&mut self) -> Option<T> {
        // SAFETY: pointers are valid for the lifetime of the mapping.
        let producer = unsafe { &*self.producer };
        if self.cursor.cons_avail(|| producer.load(Ordering::Acquire)) == 0 {
            return None;
        }
        // SAFETY: slot index is masked; Acquire above orders this read.
        let v = unsafe { self.descs.add(self.cursor.cons_slot() as usize).read() };
        self.cursor.advance_cons(1);
        // SAFETY: as above; Release hands the slot back to the kernel.
        unsafe { &*self.consumer }.store(self.cursor.cached_cons(), Ordering::Release);
        Some(v)
    }

    /// Entries the kernel has produced that we have not yet consumed.
    pub fn pending(&self) -> u32 {
        // SAFETY: pointer valid for the lifetime of the mapping.
        let p = unsafe { &*self.producer }.load(Ordering::Acquire);
        p.wrapping_sub(self.cursor.cached_cons())
    }

    /// True if the kernel set `XDP_RING_NEED_WAKEUP` on this ring.
    pub fn needs_wakeup(&self) -> bool {
        if self.flags.is_null() {
            return false;
        }
        // SAFETY: non-null flags pointer lies within the mapping.
        unsafe { &*self.flags }.load(Ordering::Acquire) & XDP_RING_NEED_WAKEUP != 0
    }
}

impl<T: Copy> Drop for MmapRing<T> {
    fn drop(&mut self) {
        // SAFETY: `map`/`map_len` come from our successful mmap.
        unsafe { libc::munmap(self.map, self.map_len) };
    }
}
