//! XDP ring index arithmetic — no unsafe, no kernel interaction, any OS.
//!
//! All four rings (Fill, Completion, RX, TX) share the same producer/consumer
//! index scheme.  A ring with capacity `n` (power of two) uses a mask `n-1`
//! to wrap indices cheaply.  [`RingCursor`] is the cached-index logic used by
//! the mmap'd rings in `xsk_ring`; [`RingIndex`] is a simple standalone tracker.

/// Mmap offsets for one ring, as returned by
/// `getsockopt(SOL_XDP, XDP_MMAP_OFFSETS)`.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub struct RingOffset {
    pub producer: u64,
    pub consumer: u64,
    pub desc: u64,
    pub flags: u64,
}

/// All four ring offsets for a single XDP socket.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub struct XdpMmapOffsets {
    pub rx: RingOffset,
    pub tx: RingOffset,
    pub fill: RingOffset,
    pub completion: RingOffset,
}

/// Byte length of the pre-5.4 kernel `xdp_mmap_offsets` (no `flags` fields).
pub const MMAP_OFFSETS_V1_LEN: usize = 12 * 8;
/// Byte length of the current kernel `xdp_mmap_offsets` (with `flags`).
pub const MMAP_OFFSETS_V2_LEN: usize = 16 * 8;

impl XdpMmapOffsets {
    /// Decode the raw `getsockopt` output.  Returns the offsets and whether the
    /// kernel supplied ring `flags` words (needed for `XDP_USE_NEED_WAKEUP`;
    /// absent before Linux 5.4).  `None` if `len` is not a known layout.
    pub fn from_kernel(raw: &[u64; 16], len: usize) -> Option<(Self, bool)> {
        let (stride, has_flags) = match len {
            MMAP_OFFSETS_V2_LEN => (4, true),
            MMAP_OFFSETS_V1_LEN => (3, false),
            _ => return None,
        };
        let ring = |i: usize| RingOffset {
            producer: raw[i * stride],
            consumer: raw[i * stride + 1],
            desc: raw[i * stride + 2],
            flags: if has_flags { raw[i * stride + 3] } else { 0 },
        };
        Some((
            Self {
                rx: ring(0),
                tx: ring(1),
                fill: ring(2),
                completion: ring(3),
            },
            has_flags,
        ))
    }
}

/// Cached producer/consumer cursor for a ring shared with the kernel.
///
/// It caches the last observed value of the *remote* index and only re-reads
/// it (via the supplied closure) when the cached view says the ring is full
/// (producer role) or empty (consumer role).  Indices are free-running `u32`s
/// that wrap naturally.
///
/// `size` MUST be a power of two.
#[derive(Debug, Clone, Copy)]
pub struct RingCursor {
    size: u32,
    mask: u32,
    cached_prod: u32,
    cached_cons: u32,
}

impl RingCursor {
    pub fn new(size: u32) -> Self {
        Self::with_indices(size, 0, 0)
    }

    pub fn with_indices(size: u32, producer: u32, consumer: u32) -> Self {
        assert!(size.is_power_of_two(), "ring size must be power of two");
        Self {
            size,
            mask: size - 1,
            cached_prod: producer,
            cached_cons: consumer,
        }
    }

    /// Free slots for a producer.  Re-reads the kernel's consumer index through
    /// `load_consumer` only if the cached view shows no space.
    pub fn prod_free(&mut self, load_consumer: impl FnOnce() -> u32) -> u32 {
        let free = self
            .size
            .saturating_sub(self.cached_prod.wrapping_sub(self.cached_cons));
        if free > 0 {
            return free;
        }
        self.cached_cons = load_consumer();
        self.size
            .saturating_sub(self.cached_prod.wrapping_sub(self.cached_cons))
    }

    /// Entries available to a consumer.  Re-reads the kernel's producer index
    /// through `load_producer` only if the cached view shows nothing.
    pub fn cons_avail(&mut self, load_producer: impl FnOnce() -> u32) -> u32 {
        let avail = self.cached_prod.wrapping_sub(self.cached_cons);
        if avail > 0 {
            return avail;
        }
        self.cached_prod = load_producer();
        self.cached_prod.wrapping_sub(self.cached_cons)
    }

    pub fn prod_slot(&self) -> u32 {
        self.cached_prod & self.mask
    }

    pub fn cons_slot(&self) -> u32 {
        self.cached_cons & self.mask
    }

    pub fn advance_prod(&mut self, n: u32) {
        self.cached_prod = self.cached_prod.wrapping_add(n);
    }

    pub fn advance_cons(&mut self, n: u32) {
        self.cached_cons = self.cached_cons.wrapping_add(n);
    }

    pub fn cached_prod(&self) -> u32 {
        self.cached_prod
    }

    pub fn cached_cons(&self) -> u32 {
        self.cached_cons
    }

    pub fn size(&self) -> u32 {
        self.size
    }
}

/// Lightweight ring-index tracker for a single ring.
///
/// `capacity` MUST be a power of two.
#[derive(Debug, Clone, Copy)]
pub struct RingIndex {
    pub producer: u32,
    pub consumer: u32,
    capacity: u32,
    mask: u32,
}

impl RingIndex {
    pub fn new(capacity: u32) -> Self {
        assert!(
            capacity.is_power_of_two(),
            "ring capacity must be power of two"
        );
        Self {
            producer: 0,
            consumer: 0,
            capacity,
            mask: capacity - 1,
        }
    }

    /// Number of entries available to consume.
    pub fn available(&self) -> u32 {
        self.producer.wrapping_sub(self.consumer)
    }

    /// Number of free slots for the producer.
    pub fn free_slots(&self) -> u32 {
        self.capacity - self.available()
    }

    /// Advance consumer by `n`.
    pub fn consume(&mut self, n: u32) {
        self.consumer = self.consumer.wrapping_add(n);
    }

    /// Advance producer by `n`.
    pub fn produce(&mut self, n: u32) {
        self.producer = self.producer.wrapping_add(n);
    }

    /// Slot index for the current consumer head (before consuming).
    pub fn consumer_slot(&self) -> u32 {
        self.consumer & self.mask
    }

    /// Slot index for the current producer head (before producing).
    pub fn producer_slot(&self) -> u32 {
        self.producer & self.mask
    }

    pub fn capacity(&self) -> u32 {
        self.capacity
    }

    pub fn is_empty(&self) -> bool {
        self.available() == 0
    }

    pub fn is_full(&self) -> bool {
        self.free_slots() == 0
    }
}

/// Holds ring indices for all four XDP rings.
#[derive(Debug)]
pub struct XdpRings {
    pub fill: RingIndex,
    pub completion: RingIndex,
    pub rx: RingIndex,
    pub tx: RingIndex,
}

impl XdpRings {
    pub fn new(capacity: u32) -> Self {
        Self {
            fill: RingIndex::new(capacity),
            completion: RingIndex::new(capacity),
            rx: RingIndex::new(capacity),
            tx: RingIndex::new(capacity),
        }
    }
}

/// A single descriptor in the RX or TX ring (kernel `struct xdp_desc`).
#[derive(Debug, Clone, Copy, Default)]
#[repr(C)]
pub struct XdpDesc {
    /// Byte offset from start of UMEM to start of this packet.
    pub addr: u64,
    /// Length of the packet in bytes.
    pub len: u32,
    pub options: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn producer_fills_then_sees_consumer_progress() {
        let mut c = RingCursor::new(4);
        assert_eq!(c.prod_free(|| 0), 4);
        c.advance_prod(4);
        // Full under the cached view; the closure is consulted.
        assert_eq!(c.prod_free(|| 0), 0);
        assert_eq!(c.prod_free(|| 3), 3);
        assert_eq!(c.cached_cons(), 3);
    }

    #[test]
    fn producer_does_not_reload_while_space_remains() {
        let mut c = RingCursor::new(4);
        c.advance_prod(1);
        assert_eq!(c.prod_free(|| panic!("must not reload")), 3);
    }

    #[test]
    fn consumer_reloads_only_when_empty() {
        let mut c = RingCursor::new(8);
        assert_eq!(c.cons_avail(|| 5), 5);
        c.advance_cons(2);
        assert_eq!(c.cons_avail(|| panic!("must not reload")), 3);
        c.advance_cons(3);
        assert_eq!(c.cons_avail(|| 5), 0);
    }

    #[test]
    fn indices_wrap_around_u32() {
        let mut c = RingCursor::with_indices(4, u32::MAX - 1, u32::MAX - 1);
        assert_eq!(c.prod_free(|| 0), 4);
        c.advance_prod(3); // producer wraps to 1
        assert_eq!(c.cached_prod(), 1);
        assert_eq!(c.cons_avail(|| 1), 3);
        assert_eq!(c.prod_slot(), 1);
        assert_eq!(c.cons_slot(), (u32::MAX - 1) & 3);
        c.advance_cons(3);
        assert_eq!(c.cons_avail(|| 1), 0);
        assert_eq!(c.prod_free(|| 1), 4);
    }

    #[test]
    fn slots_mask_to_ring_size() {
        let mut c = RingCursor::new(8);
        c.advance_prod(11);
        assert_eq!(c.prod_slot(), 3);
    }

    #[test]
    fn corrupt_remote_index_never_overreports_free() {
        let mut c = RingCursor::new(4);
        c.advance_prod(4);
        // Remote consumer "ahead" of producer would underflow; saturate to 0.
        assert_eq!(c.prod_free(|| 9), 0);
    }

    #[test]
    fn mmap_offsets_v2_decode() {
        let mut raw = [0u64; 16];
        for (i, r) in raw.iter_mut().enumerate() {
            *r = i as u64 * 8;
        }
        let (o, has_flags) = XdpMmapOffsets::from_kernel(&raw, MMAP_OFFSETS_V2_LEN).unwrap();
        assert!(has_flags);
        assert_eq!(o.rx.flags, 24);
        assert_eq!(o.fill.producer, 64);
        assert_eq!(o.completion.desc, 112);
    }

    #[test]
    fn mmap_offsets_v1_decode_has_no_flags() {
        let mut raw = [0u64; 16];
        for (i, r) in raw.iter_mut().enumerate() {
            *r = i as u64;
        }
        let (o, has_flags) = XdpMmapOffsets::from_kernel(&raw, MMAP_OFFSETS_V1_LEN).unwrap();
        assert!(!has_flags);
        assert_eq!(o.tx.producer, 3);
        assert_eq!(o.completion.desc, 11);
        assert_eq!(o.rx.flags, 0);
        assert!(XdpMmapOffsets::from_kernel(&raw, 100).is_none());
    }

    #[test]
    fn legacy_ring_index_still_works() {
        let mut r = RingIndex::new(2);
        assert!(r.is_empty());
        r.produce(2);
        assert!(r.is_full());
        r.consume(1);
        assert_eq!(r.free_slots(), 1);
    }
}
