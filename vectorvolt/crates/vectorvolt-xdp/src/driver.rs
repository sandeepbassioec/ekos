//! `XdpDriver` — high-level send/receive API over an XDP socket.
//!
//! Wires together the UMEM, the four mmap'd rings, the TX frame allocator and
//! the XDP redirect program:
//!
//! * UMEM frames are split in half: the lower half is the RX pool (fed to the
//!   fill ring), the upper half the TX pool (owned by a [`FrameAllocator`]).
//! * `send` takes a *free* TX frame, fills it, and enqueues a descriptor; the
//!   frame returns to the pool when its address comes back on the completion
//!   ring.  (Previously every send reused frame 0, corrupting in-flight data.)
//! * Blocking receive uses `poll(2)` on the XSK fd; the TX kick and fill-ring
//!   wakeups honour `XDP_USE_NEED_WAKEUP` instead of spinning or issuing a
//!   syscall per packet.
//! * Packets are only delivered because [`XdpProgram`] is attached to the
//!   interface and the socket is registered in its XSKMAP.

use crate::frames::{FrameAllocator, frame_of_addr};
use crate::rings::XdpDesc;
use crate::socket::{XDP_USE_NEED_WAKEUP, XdpSocket};
use crate::umem::Umem;
use crate::xdp_prog::XdpProgram;
use crate::xsk_ring::{
    MmapRing, PGOFF_COMPLETION_RING, PGOFF_FILL_RING, PGOFF_RX_RING, PGOFF_TX_RING,
};
use crate::{XdpError, XdpResult};

/// High-level driver combining a socket, UMEM, rings and the XDP program.
pub struct XdpDriver {
    // Field order is drop order: detach the program, unmap rings, close the
    // socket, then unmap the UMEM.
    _program: XdpProgram,
    fill: MmapRing<u64>,
    completion: MmapRing<u64>,
    rx: MmapRing<XdpDesc>,
    tx: MmapRing<XdpDesc>,
    pub socket: XdpSocket,
    pub umem: Umem,
    rx_frames: FrameAllocator,
    tx_frames: FrameAllocator,
    /// RX frame handed to the caller by the last `recv*`; recycled on the next call.
    held_rx: Option<u32>,
    use_wakeup: bool,
}

impl XdpDriver {
    /// Create a driver bound to `ifname` queue `queue_id`, load the XDP
    /// redirect program and attach it.
    ///
    /// Fails (rather than silently receiving nothing) if the program cannot be
    /// loaded/attached; see [`XdpError::Program`].  Requires CAP_NET_RAW,
    /// CAP_BPF and CAP_NET_ADMIN.  `ring_size` must be a power of two.  In
    /// tests, mark callers `#[ignore]`.
    pub fn bind(ifname: &str, queue_id: u32, ring_size: u32, umem: Umem) -> XdpResult<Self> {
        if !ring_size.is_power_of_two() {
            return Err(XdpError::InvalidConfig("ring_size must be a power of two"));
        }
        let frame_count = umem.cfg.frame_count;
        if frame_count < 2 {
            return Err(XdpError::InvalidConfig("UMEM needs at least 2 frames"));
        }
        let half = frame_count / 2;

        let sock = XdpSocket::new(&umem)?;
        sock.configure_rings(ring_size)?;
        let (offs, has_flags) = sock.mmap_offsets()?;
        let fd = sock.raw_fd();
        let fill = MmapRing::map(fd, ring_size, &offs.fill, has_flags, PGOFF_FILL_RING)?;
        let completion = MmapRing::map(
            fd,
            ring_size,
            &offs.completion,
            has_flags,
            PGOFF_COMPLETION_RING,
        )?;
        let rx = MmapRing::map(fd, ring_size, &offs.rx, has_flags, PGOFF_RX_RING)?;
        let tx = MmapRing::map(fd, ring_size, &offs.tx, has_flags, PGOFF_TX_RING)?;

        let ifindex = XdpSocket::ifindex_of(ifname)?;
        let mut use_wakeup = has_flags;
        if use_wakeup {
            match sock.bind_with_flags(ifindex, queue_id, XDP_USE_NEED_WAKEUP) {
                Ok(()) => {}
                Err(XdpError::Os(libc::EINVAL)) => {
                    use_wakeup = false;
                    sock.bind(ifindex, queue_id)?;
                }
                Err(e) => return Err(e),
            }
        } else {
            sock.bind(ifindex, queue_id)?;
        }

        // The socket must be bound before it can be put in the XSKMAP.
        let program = XdpProgram::load_and_attach(ifindex, queue_id)?;
        program.set_socket(queue_id, fd)?;

        let mut driver = Self {
            _program: program,
            fill,
            completion,
            rx,
            tx,
            socket: sock,
            umem,
            rx_frames: FrameAllocator::new(0..half),
            tx_frames: FrameAllocator::new(half..frame_count),
            held_rx: None,
            use_wakeup,
        };
        driver.refill_fill_ring();
        Ok(driver)
    }

    /// Move free RX frames into the fill ring until it is full or none remain.
    fn refill_fill_ring(&mut self) {
        let frame_size = self.umem.cfg;
        while let Some(f) = self.rx_frames.alloc() {
            if !self.fill.push(frame_size.frame_addr(f)) {
                let _ = self.rx_frames.free(f);
                break;
            }
        }
    }

    /// Return completed TX frames to the free pool.
    fn reclaim_tx(&mut self) {
        let frame_size = self.umem.cfg.frame_size;
        while let Some(addr) = self.completion.pop() {
            let f = frame_of_addr(addr, frame_size);
            if self.tx_frames.owns(f) {
                let _ = self.tx_frames.free(f);
            }
        }
    }

    fn kick_if_needed(&self) -> XdpResult<()> {
        if !self.use_wakeup || self.tx.needs_wakeup() {
            self.socket.kick_tx()?;
        }
        Ok(())
    }

    /// Copy `data` into a free UMEM TX frame and enqueue it for transmission.
    ///
    /// Returns `Err(RingFull)` if the TX ring or the TX frame pool is
    /// exhausted (frames return as the kernel completes earlier sends).
    pub fn send(&mut self, data: &[u8]) -> XdpResult<()> {
        let cfg = self.umem.cfg;
        if data.len() > (cfg.frame_size - cfg.headroom) as usize {
            return Err(XdpError::InvalidConfig("packet larger than UMEM frame"));
        }
        self.reclaim_tx();
        let Some(frame) = self.tx_frames.alloc() else {
            self.kick_if_needed()?; // let the kernel make progress
            return Err(XdpError::RingFull);
        };
        if let Err(e) = self.umem.write_frame(frame, data) {
            let _ = self.tx_frames.free(frame);
            return Err(e);
        }
        let desc = XdpDesc {
            addr: self.umem.data_addr(frame),
            len: data.len() as u32,
            options: 0,
        };
        if !self.tx.push(desc) {
            let _ = self.tx_frames.free(frame);
            self.kick_if_needed()?;
            return Err(XdpError::RingFull);
        }
        self.kick_if_needed()
    }

    /// Recycle the previously returned RX frame, top up the fill ring and pop
    /// one RX descriptor.
    fn next_rx_desc(&mut self) -> Option<XdpDesc> {
        if let Some(f) = self.held_rx.take() {
            let _ = self.rx_frames.free(f);
        }
        self.refill_fill_ring();
        let desc = self.rx.pop()?;
        let frame = frame_of_addr(desc.addr, self.umem.cfg.frame_size);
        if self.rx_frames.owns(frame) {
            self.held_rx = Some(frame);
        }
        Some(desc)
    }

    fn rx_view(&self, desc: XdpDesc) -> XdpResult<(u32, &[u8])> {
        let frame = frame_of_addr(desc.addr, self.umem.cfg.frame_size);
        let data = self
            .umem
            .bytes_at(desc.addr, desc.len)
            .ok_or(XdpError::InvalidConfig(
                "kernel returned an out-of-range RX descriptor",
            ))?;
        Ok((frame, data))
    }

    /// Non-blocking receive: the next packet's `(frame index, bytes)`.
    ///
    /// The slice is valid until the next `recv*` call.  Returns
    /// `Err(RingEmpty)` when no packet is waiting.
    pub fn recv(&mut self) -> XdpResult<(u32, &[u8])> {
        let desc = self.next_rx_desc().ok_or(XdpError::RingEmpty)?;
        self.rx_view(desc)
    }

    /// Blocking receive: sleeps in `poll(2)` (no spinning) until a packet
    /// arrives or `timeout_ms` elapses (`-1` = forever).
    ///
    /// Returns `Err(Timeout)` on timeout.  `Err(RingEmpty)` is a spurious
    /// wakeup and the caller should simply retry.
    pub fn recv_wait(&mut self, timeout_ms: i32) -> XdpResult<(u32, &[u8])> {
        let desc = match self.next_rx_desc() {
            Some(d) => d,
            None => {
                if !self.socket.poll_readable(timeout_ms)? {
                    return Err(XdpError::Timeout);
                }
                self.next_rx_desc().ok_or(XdpError::RingEmpty)?
            }
        };
        self.rx_view(desc)
    }

    /// Return a snapshot of kernel-side XDP statistics.
    pub fn stats(&self) -> XdpResult<crate::socket::XdpStatistics> {
        self.socket.statistics()
    }

    /// Number of TX frames currently free for `send`.
    pub fn tx_free(&self) -> u32 {
        self.tx_frames.available() as u32
    }

    /// Number of pending RX packets.
    pub fn rx_pending(&self) -> u32 {
        self.rx.pending()
    }
}
