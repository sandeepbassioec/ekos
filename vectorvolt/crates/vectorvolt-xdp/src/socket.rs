//! AF_XDP socket — raw kernel interface.
//!
//! Requires CAP_NET_RAW (or root).  Tests that instantiate this type are
//! marked `#[ignore]` in CI since GH Actions runners lack the capability.

use libc::{SOCK_RAW, sockaddr};
use std::os::unix::io::{FromRawFd, OwnedFd, RawFd};

use crate::rings::XdpMmapOffsets;
use crate::umem::{Umem, UmemConfig};
use crate::{PF_XDP, SOL_XDP, XdpError, XdpResult};

// Linux uapi constants.
const XDP_UMEM_REG: libc::c_int = 4;
const XDP_RX_RING: libc::c_int = 1;
const XDP_TX_RING: libc::c_int = 2;
const XDP_UMEM_FILL_RING: libc::c_int = 5;
const XDP_UMEM_COMPLETION_RING: libc::c_int = 6;
const XDP_MMAP_OFFSETS: libc::c_int = 1;

/// `sxdp_flags` bit: kernel sets ring wakeup flags instead of needing a
/// syscall per batch (Linux >= 5.4).
pub const XDP_USE_NEED_WAKEUP: u16 = 1 << 3;

fn last_errno() -> XdpError {
    XdpError::Os(std::io::Error::last_os_error().raw_os_error().unwrap_or(0))
}

/// Kernel struct for UMEM registration (`XDP_UMEM_REG`).
#[repr(C)]
struct XdpUmemReg {
    addr: u64,
    len: u64,
    chunk_size: u32,
    headroom: u32,
    flags: u32,
    tx_metadata_len: u32,
}

/// `sockaddr_xdp` for binding an XDP socket to a NIC queue.
#[repr(C)]
pub struct SockaddrXdp {
    pub sxdp_family: u16, // AF_XDP = 44
    pub sxdp_flags: u16,
    pub sxdp_ifindex: u32,
    pub sxdp_queue_id: u32,
    pub sxdp_shared_umem_fd: u32,
}

/// An open AF_XDP socket.
pub struct XdpSocket {
    fd: OwnedFd,
}

impl XdpSocket {
    /// Create a new AF_XDP socket and register `umem` with the kernel.
    pub fn new(umem: &Umem) -> XdpResult<Self> {
        let fd = unsafe { libc::socket(PF_XDP, SOCK_RAW, 0) };
        if fd < 0 {
            return Err(XdpError::Os(unsafe { *libc::__errno_location() }));
        }
        // SAFETY: fd is valid and we're taking ownership.
        let owned = unsafe { OwnedFd::from_raw_fd(fd) };
        let sock = Self { fd: owned };
        sock.register_umem(umem)?;
        Ok(sock)
    }

    fn register_umem(&self, umem: &Umem) -> XdpResult<()> {
        let reg = XdpUmemReg {
            addr: umem.as_ptr() as u64,
            len: umem.len() as u64,
            chunk_size: umem.cfg.frame_size,
            headroom: umem.cfg.headroom,
            flags: 0,
            tx_metadata_len: 0,
        };
        let rc = unsafe {
            libc::setsockopt(
                self.raw_fd(),
                SOL_XDP,
                XDP_UMEM_REG,
                &reg as *const _ as *const libc::c_void,
                std::mem::size_of::<XdpUmemReg>() as u32,
            )
        };
        if rc < 0 {
            return Err(XdpError::Os(unsafe { *libc::__errno_location() }));
        }
        Ok(())
    }

    /// Configure ring sizes (call before bind).
    pub fn configure_rings(&self, ring_size: u32) -> XdpResult<()> {
        for opt in [
            XDP_RX_RING,
            XDP_TX_RING,
            XDP_UMEM_FILL_RING,
            XDP_UMEM_COMPLETION_RING,
        ] {
            let rc = unsafe {
                libc::setsockopt(
                    self.raw_fd(),
                    SOL_XDP,
                    opt,
                    &ring_size as *const u32 as *const libc::c_void,
                    std::mem::size_of::<u32>() as u32,
                )
            };
            if rc < 0 {
                return Err(XdpError::Os(unsafe { *libc::__errno_location() }));
            }
        }
        Ok(())
    }

    /// Bind the socket to a NIC queue (`ifindex`, `queue_id`).
    pub fn bind(&self, ifindex: u32, queue_id: u32) -> XdpResult<()> {
        self.bind_with_flags(ifindex, queue_id, 0)
    }

    /// Bind with explicit `sxdp_flags` (e.g. [`XDP_USE_NEED_WAKEUP`]).
    pub fn bind_with_flags(&self, ifindex: u32, queue_id: u32, flags: u16) -> XdpResult<()> {
        let sa = SockaddrXdp {
            sxdp_family: PF_XDP as u16,
            sxdp_flags: flags,
            sxdp_ifindex: ifindex,
            sxdp_queue_id: queue_id,
            sxdp_shared_umem_fd: 0,
        };
        let rc = unsafe {
            libc::bind(
                self.raw_fd(),
                &sa as *const SockaddrXdp as *const sockaddr,
                std::mem::size_of::<SockaddrXdp>() as u32,
            )
        };
        if rc < 0 {
            return Err(XdpError::Os(unsafe { *libc::__errno_location() }));
        }
        Ok(())
    }

    /// Fetch the ring mmap offsets.  Returns the offsets and whether the kernel
    /// provides ring `flags` words (needed for `XDP_USE_NEED_WAKEUP`).
    pub fn mmap_offsets(&self) -> XdpResult<(XdpMmapOffsets, bool)> {
        let mut raw = [0u64; 16];
        let mut len = std::mem::size_of_val(&raw) as libc::socklen_t;
        // SAFETY: `raw` is a valid writable buffer of `len` bytes.
        let rc = unsafe {
            libc::getsockopt(
                self.raw_fd(),
                SOL_XDP,
                XDP_MMAP_OFFSETS,
                raw.as_mut_ptr() as *mut libc::c_void,
                &mut len,
            )
        };
        if rc < 0 {
            return Err(last_errno());
        }
        XdpMmapOffsets::from_kernel(&raw, len as usize)
            .ok_or(XdpError::InvalidConfig("unexpected xdp_mmap_offsets size"))
    }

    /// Block until the socket is readable (a packet is waiting) or
    /// `timeout_ms` elapses (`-1` = forever).  Returns `Ok(true)` if readable.
    ///
    /// `poll(2)` on an XSK also performs the fill-ring wakeup when the kernel
    /// has set `XDP_RING_NEED_WAKEUP` on it.
    pub fn poll_readable(&self, timeout_ms: i32) -> XdpResult<bool> {
        let mut pfd = libc::pollfd {
            fd: self.raw_fd(),
            events: libc::POLLIN,
            revents: 0,
        };
        loop {
            // SAFETY: `pfd` is a valid pollfd and nfds = 1.
            let rc = unsafe { libc::poll(&mut pfd, 1, timeout_ms) };
            if rc < 0 {
                let e = last_errno();
                if matches!(e, XdpError::Os(libc::EINTR)) {
                    continue;
                }
                return Err(e);
            }
            return Ok(rc > 0 && pfd.revents & (libc::POLLIN | libc::POLLERR) != 0);
        }
    }

    /// Kick the kernel to process the TX ring (`sendto` with no data).
    /// Transient back-pressure errors are not failures: the frames stay queued.
    pub fn kick_tx(&self) -> XdpResult<()> {
        // SAFETY: zero-length send with null buffer/address is the documented kick.
        let rc = unsafe {
            libc::sendto(
                self.raw_fd(),
                std::ptr::null(),
                0,
                libc::MSG_DONTWAIT,
                std::ptr::null(),
                0,
            )
        };
        if rc >= 0 {
            return Ok(());
        }
        match last_errno() {
            XdpError::Os(
                libc::EAGAIN | libc::EBUSY | libc::ENOBUFS | libc::ENETDOWN | libc::EINTR,
            ) => Ok(()),
            e => Err(e),
        }
    }

    pub fn raw_fd(&self) -> RawFd {
        use std::os::unix::io::AsRawFd;
        self.fd.as_raw_fd()
    }

    /// Resolve a network interface name to its ifindex.
    pub fn ifindex_of(name: &str) -> XdpResult<u32> {
        let mut buf = [0u8; libc::IFNAMSIZ];
        let bytes = name.as_bytes();
        if bytes.len() >= libc::IFNAMSIZ {
            return Err(XdpError::NoSuchInterface);
        }
        buf[..bytes.len()].copy_from_slice(bytes);
        let idx = unsafe { libc::if_nametoindex(buf.as_ptr() as *const libc::c_char) };
        if idx == 0 {
            return Err(XdpError::NoSuchInterface);
        }
        Ok(idx)
    }
}

/// XDP statistics returned by `getsockopt(SOL_XDP, XDP_STATISTICS)`.
#[derive(Debug, Default, Clone, Copy)]
#[repr(C)]
pub struct XdpStatistics {
    pub rx_dropped: u64,
    pub rx_invalid_descs: u64,
    pub tx_invalid_descs: u64,
    pub rx_ring_full: u64,
    pub rx_fill_ring_empty_descs: u64,
    pub tx_ring_empty_descs: u64,
}

const XDP_STATISTICS: libc::c_int = 8;

impl XdpSocket {
    pub fn statistics(&self) -> XdpResult<XdpStatistics> {
        let mut stats = XdpStatistics::default();
        let mut len = std::mem::size_of::<XdpStatistics>() as u32;
        let rc = unsafe {
            libc::getsockopt(
                self.raw_fd(),
                SOL_XDP,
                XDP_STATISTICS,
                &mut stats as *mut _ as *mut libc::c_void,
                &mut len,
            )
        };
        if rc < 0 {
            return Err(XdpError::Os(unsafe { *libc::__errno_location() }));
        }
        Ok(stats)
    }
}

/// Create a UMEM with default config for use in tests/examples.
pub fn make_test_umem() -> XdpResult<Umem> {
    Umem::new(UmemConfig::default())
}
