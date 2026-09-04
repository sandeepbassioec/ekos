//! AF_XDP zero-copy kernel-bypass networking (VV-NET-XDP-001..010).
//!
//! AF_XDP sockets bypass the kernel TCP/IP stack entirely — packets land
//! directly in user-space memory with no copies.  This module provides:
//!
//! - [`Umem`]   — mmap-backed packet frame pool registered with the kernel
//! - [`XdpRings`] — Fill/Completion/RX/TX ring index arithmetic
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

#[cfg(target_os = "linux")]
pub mod driver;
#[cfg(target_os = "linux")]
pub mod rings;
#[cfg(target_os = "linux")]
pub mod socket;
#[cfg(target_os = "linux")]
pub mod umem;

#[cfg(target_os = "linux")]
pub use driver::XdpDriver;
#[cfg(target_os = "linux")]
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
}

impl std::fmt::Display for XdpError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Os(e) => write!(f, "OS error {e}"),
            Self::UmemExhausted => write!(f, "UMEM frame pool exhausted"),
            Self::NoSuchInterface => write!(f, "network interface not found"),
            Self::RingEmpty => write!(f, "XDP ring empty"),
            Self::RingFull => write!(f, "XDP ring full"),
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
