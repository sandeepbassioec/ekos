//! UMEM — the mmap-backed packet frame pool registered with the kernel.
//!
//! The kernel and user space share this memory region.  Each "frame" is a
//! fixed-size packet buffer.  The UMEM must be registered via
//! `setsockopt(SOL_XDP, XDP_UMEM_REG)` before creating rings.

use libc::{MAP_ANONYMOUS, MAP_FAILED, MAP_SHARED, PROT_READ, PROT_WRITE};
use std::ptr::NonNull;

use crate::{XdpError, XdpResult};

/// Configuration for a UMEM allocation.
#[derive(Debug, Clone, Copy)]
pub struct UmemConfig {
    /// Number of packet frames.  Must be a power of two.
    pub frame_count: u32,
    /// Size of each frame in bytes (typically 2048 or 4096).
    pub frame_size: u32,
    /// Headroom reserved before the packet data in each frame.
    pub headroom: u32,
}

impl Default for UmemConfig {
    fn default() -> Self {
        Self {
            frame_count: 4096,
            frame_size: 2048,
            headroom: 0,
        }
    }
}

impl UmemConfig {
    pub fn total_bytes(self) -> usize {
        self.frame_count as usize * self.frame_size as usize
    }

    /// Returns true if frame_count is a power of two.
    pub fn is_valid(self) -> bool {
        self.frame_count.is_power_of_two() && self.frame_size >= 2048
    }

    /// Byte offset of frame `idx` within the UMEM.
    pub fn frame_addr(self, idx: u32) -> u64 {
        idx as u64 * self.frame_size as u64
    }
}

/// A mmap-backed UMEM region.
pub struct Umem {
    pub(crate) cfg: UmemConfig,
    ptr: NonNull<u8>,
    len: usize,
}

impl Umem {
    /// Allocate a new UMEM region with the given config.
    pub fn new(cfg: UmemConfig) -> XdpResult<Self> {
        if !cfg.is_valid() {
            return Err(XdpError::UmemExhausted);
        }
        let len = cfg.total_bytes();
        // SAFETY: mmap with MAP_ANONYMOUS | MAP_SHARED — kernel zeroes the pages.
        let ptr = unsafe {
            libc::mmap(
                std::ptr::null_mut(),
                len,
                PROT_READ | PROT_WRITE,
                MAP_ANONYMOUS | MAP_SHARED,
                -1,
                0,
            )
        };
        if ptr == MAP_FAILED {
            return Err(XdpError::Os(unsafe { *libc::__errno_location() }));
        }
        Ok(Self {
            cfg,
            ptr: NonNull::new(ptr as *mut u8).unwrap(),
            len,
        })
    }

    /// Raw pointer to the start of the UMEM region.
    pub fn as_ptr(&self) -> *mut u8 {
        self.ptr.as_ptr()
    }

    /// Total byte length of the UMEM.
    pub fn len(&self) -> usize {
        self.len
    }

    /// Write `data` into frame `idx` (offset by headroom).
    pub fn write_frame(&mut self, idx: u32, data: &[u8]) -> XdpResult<()> {
        let offset = self.cfg.frame_addr(idx) as usize + self.cfg.headroom as usize;
        let avail = self.cfg.frame_size as usize - self.cfg.headroom as usize;
        if data.len() > avail {
            return Err(XdpError::UmemExhausted);
        }
        // SAFETY: offset is within the mmap'd region we own.
        unsafe {
            std::ptr::copy_nonoverlapping(data.as_ptr(), self.ptr.as_ptr().add(offset), data.len());
        }
        Ok(())
    }

    /// Read a frame slice (headroom-adjusted, length `len`).
    pub fn read_frame(&self, idx: u32, len: usize) -> &[u8] {
        let offset = self.cfg.frame_addr(idx) as usize + self.cfg.headroom as usize;
        // SAFETY: same region.
        unsafe { std::slice::from_raw_parts(self.ptr.as_ptr().add(offset), len) }
    }
}

impl Drop for Umem {
    fn drop(&mut self) {
        // SAFETY: ptr and len come from our mmap call.
        unsafe { libc::munmap(self.ptr.as_ptr() as *mut libc::c_void, self.len) };
    }
}

// SAFETY: Umem owns its region exclusively and the pointer is valid for its lifetime.
unsafe impl Send for Umem {}
