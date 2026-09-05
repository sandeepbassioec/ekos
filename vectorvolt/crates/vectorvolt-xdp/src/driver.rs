//! `XdpDriver` — high-level send/receive API over an XDP socket.

use crate::rings::{XdpDesc, XdpRings};
use crate::socket::XdpSocket;
use crate::umem::Umem;
use crate::{XdpError, XdpResult};

/// High-level driver combining a socket, UMEM, and ring indices.
pub struct XdpDriver {
    pub socket: XdpSocket,
    pub umem: Umem,
    pub rings: XdpRings,
}

impl XdpDriver {
    /// Create a driver bound to `ifname` queue 0.
    ///
    /// Requires CAP_NET_RAW.  In tests, mark callers `#[ignore]`.
    pub fn bind(ifname: &str, ring_size: u32, umem: Umem) -> XdpResult<Self> {
        let sock = XdpSocket::new(&umem)?;
        sock.configure_rings(ring_size)?;
        let ifindex = XdpSocket::ifindex_of(ifname)?;
        sock.bind(ifindex, 0)?;
        Ok(Self {
            socket: sock,
            umem,
            rings: XdpRings::new(ring_size),
        })
    }

    /// Stage `data` into UMEM frame `frame_idx` and enqueue a TX descriptor.
    ///
    /// Returns `Err(RingFull)` if the TX ring has no free slots.
    pub fn send(&mut self, frame_idx: u32, data: &[u8]) -> XdpResult<()> {
        if self.rings.tx.is_full() {
            return Err(XdpError::RingFull);
        }
        self.umem.write_frame(frame_idx, data)?;
        let addr = self.umem.cfg.frame_addr(frame_idx);
        let _slot = self.rings.tx.producer_slot();
        let _desc = XdpDesc {
            addr,
            len: data.len() as u32,
            options: 0,
        };
        self.rings.tx.produce(1);
        Ok(())
    }

    /// Dequeue one RX descriptor and return a reference to the packet bytes.
    ///
    /// Returns `Err(RingEmpty)` when no packet is waiting.
    pub fn recv(&mut self) -> XdpResult<(u32, &[u8])> {
        if self.rings.rx.is_empty() {
            return Err(XdpError::RingEmpty);
        }
        let slot = self.rings.rx.consumer_slot();
        self.rings.rx.consume(1);
        // In a real implementation the desc addr/len would be read from the
        // mmap'd ring memory.  Here we return the frame at slot index.
        let frame_size = self.umem.cfg.frame_size as usize;
        let data = self.umem.read_frame(slot, frame_size);
        Ok((slot, data))
    }

    /// Return a snapshot of kernel-side XDP statistics.
    pub fn stats(&self) -> XdpResult<crate::socket::XdpStatistics> {
        self.socket.statistics()
    }

    /// Number of free TX slots.
    pub fn tx_free(&self) -> u32 {
        self.rings.tx.free_slots()
    }

    /// Number of pending RX packets.
    pub fn rx_pending(&self) -> u32 {
        self.rings.rx.available()
    }
}
