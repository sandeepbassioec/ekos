//! The XDP redirect program that steers packets into AF_XDP sockets.
//!
//! Without an attached XDP program the kernel never delivers packets to an
//! AF_XDP socket.  The program here is the canonical one:
//!
//! ```text
//! r2 = *(u32 *)(r1 + 16)        // ctx->rx_queue_index
//! r1 = <XSKMAP fd>              // BPF_LD_IMM64 | BPF_PSEUDO_MAP_FD
//! r3 = XDP_PASS                 // fallback action if no socket in the slot
//! call bpf_redirect_map
//! exit
//! ```
//!
//! It is hand-assembled and loaded through the raw `bpf(2)` syscall (no
//! libbpf/aya dependency).  The instruction builder is OS-independent so it
//! can be unit-tested anywhere; loading/attaching is Linux-only.
//!
//! ## Requirements / limits
//!
//! * `CAP_BPF` + `CAP_NET_ADMIN` (or root) to load and attach.
//! * Linux >= 5.7 (`BPF_LINK_CREATE` for XDP); the `bpf_redirect_map` fallback
//!   action in the flags argument needs >= 5.3.
//! * One XDP program per interface: attach fails (`EBUSY`) if another program
//!   is already attached.  The attachment is a BPF link, so it is removed
//!   automatically when the [`XdpProgram`] is dropped or the process exits.

/// One eBPF instruction (`struct bpf_insn`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub struct BpfInsn {
    pub code: u8,
    /// Low nibble = dst register, high nibble = src register.
    pub regs: u8,
    pub off: i16,
    pub imm: i32,
}

const BPF_LDX_MEM_W: u8 = 0x61; // BPF_LDX | BPF_MEM | BPF_W
const BPF_LD_IMM64: u8 = 0x18; // BPF_LD | BPF_DW | BPF_IMM
const BPF_MOV64_K: u8 = 0xb7; // BPF_ALU64 | BPF_MOV | BPF_K
const BPF_CALL: u8 = 0x85; // BPF_JMP | BPF_CALL
const BPF_EXIT: u8 = 0x95; // BPF_JMP | BPF_EXIT
const BPF_PSEUDO_MAP_FD: u8 = 1;
const BPF_FUNC_REDIRECT_MAP: i32 = 51;
const XDP_PASS: i32 = 2;
/// `offsetof(struct xdp_md, rx_queue_index)`.
const XDP_MD_RX_QUEUE_INDEX: i16 = 16;

const fn regs(dst: u8, src: u8) -> u8 {
    (src << 4) | (dst & 0x0f)
}

/// Number of instruction slots in the redirect program.
pub const REDIRECT_PROG_LEN: usize = 6;

/// Build the XSKMAP redirect program for the XSKMAP `map_fd`.
pub fn build_redirect_program(map_fd: i32) -> [BpfInsn; REDIRECT_PROG_LEN] {
    [
        // r2 = *(u32 *)(r1 + 16)
        BpfInsn {
            code: BPF_LDX_MEM_W,
            regs: regs(2, 1),
            off: XDP_MD_RX_QUEUE_INDEX,
            imm: 0,
        },
        // r1 = map_fd (two slots)
        BpfInsn {
            code: BPF_LD_IMM64,
            regs: regs(1, BPF_PSEUDO_MAP_FD),
            off: 0,
            imm: map_fd,
        },
        BpfInsn {
            code: 0,
            regs: 0,
            off: 0,
            imm: 0,
        },
        // r3 = XDP_PASS
        BpfInsn {
            code: BPF_MOV64_K,
            regs: regs(3, 0),
            off: 0,
            imm: XDP_PASS,
        },
        // call bpf_redirect_map
        BpfInsn {
            code: BPF_CALL,
            regs: 0,
            off: 0,
            imm: BPF_FUNC_REDIRECT_MAP,
        },
        // exit
        BpfInsn {
            code: BPF_EXIT,
            regs: 0,
            off: 0,
            imm: 0,
        },
    ]
}

#[cfg(target_os = "linux")]
pub use sys::XdpProgram;

#[cfg(target_os = "linux")]
mod sys {
    use super::{BpfInsn, build_redirect_program};
    use crate::{XdpError, XdpResult};
    use std::os::unix::io::{AsRawFd, FromRawFd, OwnedFd, RawFd};

    const BPF_MAP_CREATE: libc::c_long = 0;
    const BPF_MAP_UPDATE_ELEM: libc::c_long = 2;
    const BPF_PROG_LOAD: libc::c_long = 5;
    const BPF_LINK_CREATE: libc::c_long = 28;
    const BPF_MAP_TYPE_XSKMAP: u32 = 17;
    const BPF_PROG_TYPE_XDP: u32 = 6;
    const BPF_XDP: u32 = 37; // enum bpf_attach_type

    /// `union bpf_attr` arm for `BPF_MAP_CREATE`.
    #[repr(C)]
    #[derive(Default)]
    struct MapCreateAttr {
        map_type: u32,
        key_size: u32,
        value_size: u32,
        max_entries: u32,
        map_flags: u32,
    }

    /// `union bpf_attr` arm for `BPF_MAP_*_ELEM`.
    #[repr(C)]
    #[derive(Default)]
    struct MapElemAttr {
        map_fd: u32,
        _pad: u32,
        key: u64,
        value: u64,
        flags: u64,
    }

    /// `union bpf_attr` arm for `BPF_PROG_LOAD` (through `expected_attach_type`).
    #[repr(C)]
    #[derive(Default)]
    struct ProgLoadAttr {
        prog_type: u32,
        insn_cnt: u32,
        insns: u64,
        license: u64,
        log_level: u32,
        log_size: u32,
        log_buf: u64,
        kern_version: u32,
        prog_flags: u32,
        prog_name: [u8; 16],
        prog_ifindex: u32,
        expected_attach_type: u32,
    }

    /// `union bpf_attr` arm for `BPF_LINK_CREATE`.
    #[repr(C)]
    #[derive(Default)]
    struct LinkCreateAttr {
        prog_fd: u32,
        target_ifindex: u32,
        attach_type: u32,
        flags: u32,
    }

    fn bpf<T>(cmd: libc::c_long, attr: &T, stage: &'static str) -> XdpResult<RawFd> {
        // SAFETY: `attr` is a live #[repr(C)] struct matching a prefix of the
        // kernel's `union bpf_attr`; the kernel zero-extends the remainder.
        let rc = unsafe {
            libc::syscall(
                libc::SYS_bpf,
                cmd,
                attr as *const T as *const libc::c_void,
                std::mem::size_of::<T>() as libc::c_uint,
            )
        };
        if rc < 0 {
            return Err(XdpError::Program {
                stage,
                errno: std::io::Error::last_os_error().raw_os_error().unwrap_or(0),
            });
        }
        Ok(rc as RawFd)
    }

    fn owned(fd: RawFd) -> OwnedFd {
        // SAFETY: `fd` was just returned by a successful bpf(2) call.
        unsafe { OwnedFd::from_raw_fd(fd) }
    }

    /// A loaded redirect program attached to one interface, plus its XSKMAP.
    ///
    /// Dropping it closes the link (detaching the program), the program and
    /// the map.
    #[derive(Debug)]
    pub struct XdpProgram {
        // Field order = drop order: detach first.
        _link: OwnedFd,
        _prog: OwnedFd,
        map: OwnedFd,
    }

    impl XdpProgram {
        /// Create an XSKMAP with room for `queue_id + 1` sockets, load the
        /// redirect program and attach it to `ifindex`.
        pub fn load_and_attach(ifindex: u32, queue_id: u32) -> XdpResult<Self> {
            let map_attr = MapCreateAttr {
                map_type: BPF_MAP_TYPE_XSKMAP,
                key_size: 4,
                value_size: 4,
                max_entries: queue_id
                    .checked_add(1)
                    .ok_or(XdpError::InvalidConfig("queue id out of range"))?,
                map_flags: 0,
            };
            let map = owned(bpf(BPF_MAP_CREATE, &map_attr, "create XSKMAP")?);

            let insns: [BpfInsn; super::REDIRECT_PROG_LEN] =
                build_redirect_program(map.as_raw_fd());
            let license = b"GPL\0";
            let prog_attr = ProgLoadAttr {
                prog_type: BPF_PROG_TYPE_XDP,
                insn_cnt: insns.len() as u32,
                insns: insns.as_ptr() as u64,
                license: license.as_ptr() as u64,
                ..Default::default()
            };
            let prog = owned(bpf(BPF_PROG_LOAD, &prog_attr, "load XDP program")?);

            let link_attr = LinkCreateAttr {
                prog_fd: prog.as_raw_fd() as u32,
                target_ifindex: ifindex,
                attach_type: BPF_XDP,
                flags: 0,
            };
            let link = owned(bpf(BPF_LINK_CREATE, &link_attr, "attach XDP program")?);

            Ok(Self {
                _link: link,
                _prog: prog,
                map,
            })
        }

        /// Install `xsk_fd` (a bound AF_XDP socket) at XSKMAP slot `queue_id`.
        /// The socket must already be bound; the kernel rejects it otherwise.
        pub fn set_socket(&self, queue_id: u32, xsk_fd: RawFd) -> XdpResult<()> {
            let key = queue_id;
            let value = xsk_fd as u32;
            let attr = MapElemAttr {
                map_fd: self.map.as_raw_fd() as u32,
                key: &key as *const u32 as u64,
                value: &value as *const u32 as u64,
                ..Default::default()
            };
            bpf(BPF_MAP_UPDATE_ELEM, &attr, "register socket in XSKMAP").map(|_| ())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn program_has_expected_shape() {
        let p = build_redirect_program(7);
        assert_eq!(p.len(), REDIRECT_PROG_LEN);
        assert_eq!(std::mem::size_of::<BpfInsn>(), 8);
        // Ends in exit, and calls bpf_redirect_map exactly once.
        assert_eq!(p[p.len() - 1].code, BPF_EXIT);
        assert_eq!(p.iter().filter(|i| i.code == BPF_CALL).count(), 1);
    }

    #[test]
    fn loads_rx_queue_index_from_ctx() {
        let p = build_redirect_program(3);
        assert_eq!(p[0].code, 0x61);
        assert_eq!(p[0].regs, 0x12); // dst r2, src r1
        assert_eq!(p[0].off, 16);
    }

    #[test]
    fn map_fd_is_embedded_as_pseudo_map_fd() {
        let p = build_redirect_program(42);
        assert_eq!(p[1].code, 0x18);
        assert_eq!(p[1].regs, 0x11); // dst r1, src BPF_PSEUDO_MAP_FD
        assert_eq!(p[1].imm, 42);
        // Second slot of the 16-byte instruction is all zero.
        assert_eq!(
            p[2],
            BpfInsn {
                code: 0,
                regs: 0,
                off: 0,
                imm: 0
            }
        );
    }

    #[test]
    fn falls_back_to_xdp_pass_and_calls_redirect_map() {
        let p = build_redirect_program(1);
        assert_eq!((p[3].code, p[3].regs, p[3].imm), (0xb7, 0x03, 2));
        assert_eq!((p[4].code, p[4].imm), (0x85, 51));
    }
}
