//! Unit tests for AF_XDP primitives (VV-NET-XDP-001..010).
//! These run without root — they test layout math and UMEM I/O only.
//! Tests that open a real XDP socket are marked `#[ignore]` (need CAP_NET_RAW).

use vectorvolt_xdp::XdpError;

// ── UmemConfig ────────────────────────────────────────────────────────────────

#[cfg(target_os = "linux")]
mod linux {
    use vectorvolt_xdp::rings::{RingIndex, XdpRings};
    use vectorvolt_xdp::umem::{Umem, UmemConfig};

    #[test]
    fn umem_config_default_is_valid() {
        assert!(UmemConfig::default().is_valid());
    }

    #[test]
    fn umem_config_total_bytes() {
        let cfg = UmemConfig {
            frame_count: 4,
            frame_size: 2048,
            headroom: 0,
        };
        assert_eq!(cfg.total_bytes(), 8192);
    }

    #[test]
    fn umem_config_frame_addr() {
        let cfg = UmemConfig {
            frame_count: 8,
            frame_size: 2048,
            headroom: 0,
        };
        assert_eq!(cfg.frame_addr(0), 0);
        assert_eq!(cfg.frame_addr(1), 2048);
        assert_eq!(cfg.frame_addr(3), 6144);
    }

    #[test]
    fn umem_config_non_power_of_two_invalid() {
        let cfg = UmemConfig {
            frame_count: 3,
            frame_size: 2048,
            headroom: 0,
        };
        assert!(!cfg.is_valid());
    }

    #[test]
    fn umem_alloc_and_write_read() {
        let cfg = UmemConfig {
            frame_count: 4,
            frame_size: 2048,
            headroom: 0,
        };
        let mut u = Umem::new(cfg).unwrap();
        u.write_frame(0, b"hello xdp").unwrap();
        let data = u.read_frame(0, 9);
        assert_eq!(data, b"hello xdp");
    }

    #[test]
    fn umem_write_respects_headroom() {
        let cfg = UmemConfig {
            frame_count: 4,
            frame_size: 2048,
            headroom: 64,
        };
        let mut u = Umem::new(cfg).unwrap();
        u.write_frame(0, b"pkt").unwrap();
        let data = u.read_frame(0, 3);
        assert_eq!(data, b"pkt");
    }

    #[test]
    fn umem_write_overflow_errors() {
        let cfg = UmemConfig {
            frame_count: 4,
            frame_size: 2048,
            headroom: 0,
        };
        let mut u = Umem::new(cfg).unwrap();
        let big = vec![0u8; 3000];
        assert!(u.write_frame(0, &big).is_err());
    }

    // ── RingIndex ─────────────────────────────────────────────────────────────

    #[test]
    fn ring_index_new() {
        let r = RingIndex::new(64);
        assert_eq!(r.capacity(), 64);
        assert!(r.is_empty());
        assert!(!r.is_full());
    }

    #[test]
    fn ring_index_produce_consume() {
        let mut r = RingIndex::new(8);
        r.produce(3);
        assert_eq!(r.available(), 3);
        assert_eq!(r.free_slots(), 5);
        r.consume(2);
        assert_eq!(r.available(), 1);
    }

    #[test]
    fn ring_index_full_when_capacity_reached() {
        let mut r = RingIndex::new(4);
        r.produce(4);
        assert!(r.is_full());
        assert_eq!(r.free_slots(), 0);
    }

    #[test]
    fn ring_index_wraps_correctly() {
        let mut r = RingIndex::new(4);
        r.produce(4);
        r.consume(4);
        assert!(r.is_empty());
        r.produce(2);
        assert_eq!(r.producer_slot(), 2);
        assert_eq!(r.consumer_slot(), 2);
    }

    #[test]
    fn ring_index_slot_mask() {
        let mut r = RingIndex::new(8);
        for i in 0..8u32 {
            assert_eq!(r.producer_slot(), i);
            r.produce(1);
        }
        // Wraps back to 0
        assert_eq!(r.producer_slot(), 0);
    }

    #[test]
    fn xdp_rings_new() {
        let rings = XdpRings::new(256);
        assert_eq!(rings.fill.capacity(), 256);
        assert_eq!(rings.completion.capacity(), 256);
        assert_eq!(rings.rx.capacity(), 256);
        assert_eq!(rings.tx.capacity(), 256);
    }

    #[test]
    fn xdp_error_display() {
        assert!(
            vectorvolt_xdp::XdpError::RingEmpty
                .to_string()
                .contains("empty")
        );
        assert!(
            vectorvolt_xdp::XdpError::RingFull
                .to_string()
                .contains("full")
        );
        assert!(
            vectorvolt_xdp::XdpError::UmemExhausted
                .to_string()
                .contains("UMEM")
        );
        assert!(
            vectorvolt_xdp::XdpError::NoSuchInterface
                .to_string()
                .contains("interface")
        );
    }

    #[test]
    fn xdp_error_os_display() {
        assert!(vectorvolt_xdp::XdpError::Os(12).to_string().contains("12"));
    }

    // ── Integration tests (require CAP_NET_RAW — ignored in CI) ───────────────

    #[test]
    #[ignore = "requires CAP_NET_RAW / root"]
    fn xdp_socket_create_and_register_umem() {
        use vectorvolt_xdp::socket::XdpSocket;
        let umem = Umem::new(UmemConfig::default()).unwrap();
        let _sock = XdpSocket::new(&umem).unwrap();
    }

    #[test]
    #[ignore = "requires CAP_NET_RAW / root"]
    fn xdp_socket_configure_rings() {
        use vectorvolt_xdp::socket::XdpSocket;
        let umem = Umem::new(UmemConfig::default()).unwrap();
        let sock = XdpSocket::new(&umem).unwrap();
        sock.configure_rings(64).unwrap();
    }

    #[test]
    #[ignore = "requires CAP_NET_RAW / root + loopback interface"]
    fn xdp_driver_bind_loopback() {
        use vectorvolt_xdp::XdpDriver;
        use vectorvolt_xdp::umem::UmemConfig;
        let umem = Umem::new(UmemConfig::default()).unwrap();
        let _driver = XdpDriver::bind("lo", 64, umem).unwrap();
    }
}

// Compile-time smoke test on non-Linux (no runtime behaviour).
#[test]
fn xdp_error_is_send_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<XdpError>();
}
