//! AF_XDP zero-copy kernel-bypass networking (VV-NET-XDP-001..010).
//!
//! AF_XDP sockets bypass the kernel TCP/IP stack entirely — packets land
//! directly in user-space memory with no copies.  This module provides:
//!
//! - [`Umem`]   — mmap-backed packet frame pool registered with the kernel
//! - [`XdpRings`] — ring index arithmetic (pure; the live rings are mmap'd in `xsk_ring`)
//! - [`FrameAllocator`] — UMEM TX frame free list
//! - `xdp_prog` — hand-assembled XSKMAP redirect program loaded via bpf(2)
//! - [`XdpSocket`] — raw AF_XDP socket (Linux only, requires CAP_NET_RAW)
//! - [`XdpDriver`] — send/receive API built on top of the above
//!
//! ## Platform
//!
//! All runtime code is `#[cfg(target_os = "linux")]`.  Stubs are provided on
//! other platforms so the crate compiles everywhere.
//!
//! ## Kernel requirements
//!
//! Linux ≥ 4.18 (AF_XDP first appeared).  Generic-mode XDP works on any NIC;
//! native-mode requires driver support (most modern NICs).

// Pure logic (frame allocator, ring index math, BPF program builder) compiles
// and is unit-tested on every OS.
pub mod frames;
pub mod rings;
pub mod xdp_prog;

#[cfg(target_os = "linux")]
pub mod driver;
#[cfg(target_os = "linux")]
pub mod socket;
#[cfg(target_os = "linux")]
pub mod umem;
#[cfg(target_os = "linux")]
pub mod xsk_ring;

#[cfg(target_os = "linux")]
pub use driver::XdpDriver;
pub use frames::FrameAllocator;
pub use rings::{RingIndex, XdpRings};
#[cfg(target_os = "linux")]
pub use socket::XdpSocket;
#[cfg(target_os = "linux")]
pub use umem::{Umem, UmemConfig};

/// Error type for all XDP operations.
#[derive(Debug)]
pub enum XdpError {
    /// An OS-level error (errno).
    Os(i32),
    /// The requested operation requires a larger UMEM.
    UmemExhausted,
    /// Interface index could not be resolved.
    NoSuchInterface,
    /// The ring is empty (non-blocking recv found nothing).
    RingEmpty,
    /// The ring is full (non-blocking send has no space).
    RingFull,
    /// A blocking receive timed out with no packet.
    Timeout,
    /// Invalid caller-supplied configuration.
    InvalidConfig(&'static str),
    /// Loading/attaching the XDP redirect program (bpf(2)) failed.  Without
    /// it no packets reach the socket, so XDP mode refuses to start.  Typical
    /// causes: missing CAP_BPF/CAP_NET_ADMIN, kernel < 5.7, or another XDP
    /// program already attached to the interface.  Use the TCP transport as
    /// the fallback.
    Program { stage: &'static str, errno: i32 },
}

impl std::fmt::Display for XdpError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Os(e) => write!(f, "OS error {e}"),
            Self::UmemExhausted => write!(f, "UMEM frame pool exhausted"),
            Self::NoSuchInterface => write!(f, "network interface not found"),
            Self::RingEmpty => write!(f, "XDP ring empty"),
            Self::RingFull => write!(f, "XDP ring full"),
            Self::Timeout => write!(f, "XDP receive timed out"),
            Self::InvalidConfig(m) => write!(f, "invalid XDP configuration: {m}"),
            Self::Program { stage, errno } => write!(
                f,
                "XDP redirect program: {stage} failed (OS error {errno}); \
                 requires CAP_BPF+CAP_NET_ADMIN, Linux >= 5.7 and no other XDP program on the \
                 interface; use the TCP transport instead"
            ),
        }
    }
}

impl std::error::Error for XdpError {}

pub type XdpResult<T> = Result<T, XdpError>;

/// AF_XDP protocol family constant (Linux kernel uapi).
#[cfg(target_os = "linux")]
pub const PF_XDP: libc::c_int = 44;
/// SOL_XDP socket-option level.
#[cfg(target_os = "linux")]
pub const SOL_XDP: libc::c_int = 283;
