//! End-to-end AF_XDP test over a veth pair on a real Linux kernel.
//!
//! Needs root (CAP_BPF + CAP_NET_ADMIN) and a veth pair whose names are in
//! `VV_XDP_VETH="<xdp side>,<peer side>"`, e.g.:
//!
//! ```text
//! sudo ip link add vvx0 type veth peer name vvx1
//! sudo ip link set vvx0 up && sudo ip link set vvx1 up
//! sudo env "PATH=$PATH" VV_XDP_VETH=vvx0,vvx1 cargo test -p vectorvolt-xdp --test xdp_veth_e2e -- --ignored
//! ```
//!
//! Covers: the XSKMAP redirect program delivering RX frames to the socket,
//! poll-based `recv_wait`, TX through UMEM frames, and TX frame reclaim
//! (more sends than the TX pool holds).

#![cfg(target_os = "linux")]

use std::time::{Duration, Instant};

use vectorvolt_xdp::XdpDriver;
use vectorvolt_xdp::umem::{Umem, UmemConfig};

const ETHERTYPE: u16 = 0x88b5; // IEEE local experimental

fn veth_pair() -> (String, String) {
    let v = std::env::var("VV_XDP_VETH").expect("set VV_XDP_VETH=<xdp side>,<peer side>");
    let (a, b) = v.split_once(',').expect("VV_XDP_VETH must be a,b");
    (a.to_string(), b.to_string())
}

fn frame(seq: u32) -> Vec<u8> {
    let mut f = Vec::with_capacity(64);
    f.extend_from_slice(&[0xff; 6]); // dst: broadcast
    f.extend_from_slice(&[0x02, 0, 0, 0, 0, 1]); // src: locally administered
    f.extend_from_slice(&ETHERTYPE.to_be_bytes());
    f.extend_from_slice(b"vectorvolt-xdp");
    f.extend_from_slice(&seq.to_be_bytes());
    f.resize(64, 0);
    f
}

/// Raw AF_PACKET socket bound to `ifname`, receiving only our ethertype.
struct Packet {
    fd: libc::c_int,
    ifindex: libc::c_int,
}

impl Packet {
    fn open(ifname: &str) -> Self {
        let name = std::ffi::CString::new(ifname).unwrap();
        // SAFETY: plain libc calls with valid arguments; fd checked below.
        unsafe {
            let ifindex = libc::if_nametoindex(name.as_ptr()) as libc::c_int;
            assert!(ifindex > 0, "no interface {ifname}");
            let proto = (ETHERTYPE).to_be() as libc::c_int;
            let fd = libc::socket(libc::AF_PACKET, libc::SOCK_RAW, proto);
            assert!(
                fd >= 0,
                "AF_PACKET socket: {}",
                std::io::Error::last_os_error()
            );
            let mut sll: libc::sockaddr_ll = std::mem::zeroed();
            sll.sll_family = libc::AF_PACKET as u16;
            sll.sll_protocol = ETHERTYPE.to_be();
            sll.sll_ifindex = ifindex;
            let rc = libc::bind(
                fd,
                (&sll as *const libc::sockaddr_ll).cast(),
                std::mem::size_of::<libc::sockaddr_ll>() as u32,
            );
            assert_eq!(rc, 0, "bind: {}", std::io::Error::last_os_error());
            let tv = libc::timeval {
                tv_sec: 0,
                tv_usec: 200_000,
            };
            libc::setsockopt(
                fd,
                libc::SOL_SOCKET,
                libc::SO_RCVTIMEO,
                (&tv as *const libc::timeval).cast(),
                std::mem::size_of::<libc::timeval>() as u32,
            );
            Self { fd, ifindex }
        }
    }

    fn send(&self, data: &[u8]) {
        // SAFETY: data is a valid buffer; sockaddr_ll is fully initialised.
        unsafe {
            let mut sll: libc::sockaddr_ll = std::mem::zeroed();
            sll.sll_family = libc::AF_PACKET as u16;
            sll.sll_ifindex = self.ifindex;
            sll.sll_halen = 6;
            sll.sll_addr[..6].copy_from_slice(&data[..6]);
            let n = libc::sendto(
                self.fd,
                data.as_ptr().cast(),
                data.len(),
                0,
                (&sll as *const libc::sockaddr_ll).cast(),
                std::mem::size_of::<libc::sockaddr_ll>() as u32,
            );
            assert_eq!(
                n,
                data.len() as isize,
                "sendto: {}",
                std::io::Error::last_os_error()
            );
        }
    }

    fn recv(&self) -> Option<Vec<u8>> {
        let mut buf = vec![0u8; 2048];
        // SAFETY: buf is valid for buf.len() bytes.
        let n = unsafe { libc::recv(self.fd, buf.as_mut_ptr().cast(), buf.len(), 0) };
        if n <= 0 {
            return None;
        }
        buf.truncate(n as usize);
        Some(buf)
    }
}

impl Drop for Packet {
    fn drop(&mut self) {
        // SAFETY: fd is owned by this struct.
        unsafe { libc::close(self.fd) };
    }
}

fn bind(ifname: &str) -> XdpDriver {
    let umem = Umem::new(UmemConfig::default()).unwrap();
    XdpDriver::bind(ifname, 0, 64, umem).unwrap()
}

#[test]
#[ignore = "requires root and a veth pair in VV_XDP_VETH"]
fn rx_redirect_delivers_frames_to_socket() {
    let (xdp_if, peer_if) = veth_pair();
    let mut drv = bind(&xdp_if);
    let peer = Packet::open(&peer_if);

    for seq in 0..200u32 {
        let want = frame(seq);
        peer.send(&want);
        let deadline = Instant::now() + Duration::from_secs(2);
        loop {
            match drv.recv_wait(500) {
                Ok((_, got)) => {
                    if got.len() >= 12 + 2 && got[12..14] == ETHERTYPE.to_be_bytes() {
                        assert_eq!(&got[..want.len()], &want[..], "frame {seq} corrupted");
                        break;
                    }
                }
                Err(e) => assert!(Instant::now() < deadline, "frame {seq} not received: {e:?}"),
            }
        }
    }
}

#[test]
#[ignore = "requires root and a veth pair in VV_XDP_VETH"]
fn tx_sends_frames_and_reclaims_umem() {
    let (xdp_if, peer_if) = veth_pair();
    let mut drv = bind(&xdp_if);
    let peer = Packet::open(&peer_if);

    // Far more sends than the TX half of the UMEM: only works if completed
    // frames are reclaimed.
    let total = UmemConfig::default().frame_count * 2;
    let mut seen = 0u32;
    for seq in 0..total {
        let f = frame(seq);
        let deadline = Instant::now() + Duration::from_secs(2);
        loop {
            match drv.send(&f) {
                Ok(()) => break,
                Err(e) => {
                    assert!(Instant::now() < deadline, "send {seq} stuck: {e:?}");
                    std::thread::sleep(Duration::from_millis(1));
                }
            }
        }
        if let Some(got) = peer.recv() {
            assert_eq!(&got[..14], &f[..14]);
            seen += 1;
        }
    }
    while peer.recv().is_some() {
        seen += 1;
    }
    assert!(
        seen >= total * 9 / 10,
        "peer saw {seen} of {total} transmitted frames"
    );
}
