//! XDP ring index arithmetic — no unsafe, no kernel interaction.
//!
//! All four rings (Fill, Completion, RX, TX) share the same producer/consumer
//! index scheme.  A ring with capacity `n` (power of two) uses a mask `n-1`
//! to wrap indices cheaply.

/// Mmap offsets returned by `getsockopt(SOL_XDP, XDP_MMAP_OFFSETS)`.
#[derive(Debug, Default, Clone, Copy)]
#[repr(C)]
pub struct RingOffset {
    pub producer: u64,
    pub consumer: u64,
    pub desc: u64,
    pub flags: u64,
}

/// All four ring offsets for a single XDP socket.
#[derive(Debug, Default, Clone, Copy)]
#[repr(C)]
pub struct XdpMmapOffsets {
    pub rx: RingOffset,
    pub tx: RingOffset,
    pub fill: RingOffset,
    pub completion: RingOffset,
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

/// A single descriptor in the RX or TX ring.
#[derive(Debug, Clone, Copy, Default)]
#[repr(C)]
pub struct XdpDesc {
    /// Byte offset from start of UMEM to start of this packet.
    pub addr: u64,
    /// Length of the packet in bytes.
    pub len: u32,
    pub options: u32,
}
