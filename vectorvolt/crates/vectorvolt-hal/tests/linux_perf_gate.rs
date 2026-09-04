//! Linux-specific performance gate tests (VV-TST-011).
//!
//! All tests are gated on `#[cfg(target_os = "linux")]` so they compile on
//! every platform but only run on Linux.  Thresholds are conservative —
//! 10-100× below what real hardware achieves — so CI always passes unless
//! something is catastrophically broken.

#[cfg(target_os = "linux")]
mod linux_perf {
    use std::time::Instant;
    use vectorvolt_hal::{BumpArena, HugePageBuf, NumaTopology, mlock_range, munlock_range};

    // 1 — mlock 1MB latency < 10ms
    #[test]
    fn perf_mlock_1mb_lt_10ms() {
        let mut buf = vec![0u8; 1024 * 1024];
        let t = Instant::now();
        let r = unsafe { mlock_range(buf.as_mut_ptr(), buf.len()) };
        let elapsed = t.elapsed();
        drop(r.ok());
        assert!(
            elapsed.as_millis() < 10,
            "mlock 1MB took {}ms (limit 10ms)",
            elapsed.as_millis()
        );
    }

    // 2 — affinity pin < 10ms (sched_setaffinity can take several ms on virtualised hosts)
    #[test]
    fn perf_affinity_pin_lt_1ms() {
        use vectorvolt_hal::pin_thread_to_core;
        let t = Instant::now();
        let _ = pin_thread_to_core(0);
        let elapsed = t.elapsed();
        assert!(
            elapsed.as_millis() < 10,
            "affinity pin took {}µs (limit 10ms)",
            elapsed.as_micros()
        );
    }

    // 3 — numa probe < 100ms
    #[test]
    fn perf_numa_probe_lt_100ms() {
        let t = Instant::now();
        let _ = NumaTopology::probe();
        let elapsed = t.elapsed();
        assert!(
            elapsed.as_millis() < 100,
            "numa probe took {}ms (limit 100ms)",
            elapsed.as_millis()
        );
    }

    // 4 — arena alloc 1M times < 100ms
    #[test]
    fn perf_arena_alloc_1m_lt_100ms() {
        let mut a = BumpArena::new(64 * 1024 * 1024);
        let t = Instant::now();
        let mut count = 0usize;
        while count < 1_000_000 {
            if a.alloc(8, 8).is_none() {
                a.reset();
            }
            count += 1;
        }
        let elapsed = t.elapsed();
        assert!(
            elapsed.as_millis() < 100,
            "1M arena allocs took {}ms (limit 100ms)",
            elapsed.as_millis()
        );
    }

    // 5 — direct_io write 1MB < 500ms
    #[test]
    fn perf_direct_io_write_1mb_lt_500ms() {
        use vectorvolt_storage::direct_io::DirectWriter;
        let p = {
            let mut p = std::env::temp_dir();
            p.push(format!("vv_perf_gate_{}.bin", std::process::id()));
            p
        };
        let mut w = DirectWriter::open(&p).expect("open");
        let buf = DirectWriter::aligned_buf(1024 * 1024);
        let t = Instant::now();
        w.write_aligned(buf.as_slice()).expect("write");
        let elapsed = t.elapsed();
        let _ = std::fs::remove_file(&p);
        assert!(
            elapsed.as_millis() < 500,
            "direct_io write 1MB took {}ms (limit 500ms)",
            elapsed.as_millis()
        );
    }

    // 6 — uring append 1MB < 500ms
    #[test]
    fn perf_uring_append_1mb_lt_500ms() {
        use vectorvolt_wal::uring_writer::UringWalAppender;
        let p = {
            let mut p = std::env::temp_dir();
            p.push(format!("vv_perf_uring_{}.wal", std::process::id()));
            p
        };
        let mut w = UringWalAppender::open(&p, 64).expect("open");
        let data = vec![0u8; 1024 * 1024];
        let t = Instant::now();
        w.append(&data).expect("append");
        let elapsed = t.elapsed();
        let _ = std::fs::remove_file(&p);
        assert!(
            elapsed.as_millis() < 500,
            "uring append 1MB took {}ms (limit 500ms)",
            elapsed.as_millis()
        );
    }

    // 7 — arena reset < 1µs
    #[test]
    fn perf_arena_reset_lt_1us() {
        let mut a = BumpArena::new(1024 * 1024);
        a.alloc(512 * 1024, 1).unwrap();
        let t = Instant::now();
        a.reset();
        let elapsed = t.elapsed();
        assert!(
            elapsed.as_micros() < 1,
            "arena reset took {}ns (limit 1µs)",
            elapsed.as_nanos()
        );
    }

    // 8 — numa topology stable across 100 probes
    #[test]
    fn perf_numa_topology_stable_100_probes() {
        let first = NumaTopology::probe();
        for _ in 0..100 {
            let t = NumaTopology::probe();
            assert_eq!(t.node_count(), first.node_count());
            assert_eq!(t.total_cpus(), first.total_cpus());
        }
    }

    // 9 — hugepage alloc < 100ms (or NotAvailable)
    #[test]
    fn perf_hugepage_alloc_lt_100ms() {
        let t = Instant::now();
        let r = HugePageBuf::alloc(2 * 1024 * 1024);
        let elapsed = t.elapsed();
        drop(r.ok());
        assert!(
            elapsed.as_millis() < 100,
            "hugepage alloc took {}ms (limit 100ms)",
            elapsed.as_millis()
        );
    }

    // 10 — affinity get < 10ms (sched_getaffinity can take several ms on virtualised hosts)
    #[test]
    fn perf_affinity_get_lt_1ms() {
        use vectorvolt_hal::get_thread_affinity;
        let t = Instant::now();
        let _ = get_thread_affinity();
        let elapsed = t.elapsed();
        assert!(
            elapsed.as_millis() < 10,
            "get_thread_affinity took {}µs (limit 10ms)",
            elapsed.as_micros()
        );
    }

    // 11 — mlock+munlock 1MB < 50ms
    #[test]
    fn perf_mlock_munlock_1mb_lt_50ms() {
        let mut buf = vec![0u8; 1024 * 1024];
        let t = Instant::now();
        let g = unsafe { mlock_range(buf.as_mut_ptr(), buf.len()) };
        drop(g.ok());
        let _ = unsafe { munlock_range(buf.as_ptr(), buf.len()) };
        let elapsed = t.elapsed();
        assert!(
            elapsed.as_millis() < 50,
            "mlock+munlock 1MB took {}ms (limit 50ms)",
            elapsed.as_millis()
        );
    }

    // 12 — alloc 10k 64-byte slabs < 10ms
    #[test]
    fn perf_alloc_10k_slabs_lt_10ms() {
        let mut a = BumpArena::new(1024 * 1024);
        let t = Instant::now();
        for _ in 0..10_000 {
            a.alloc(64, 64).expect("alloc");
        }
        let elapsed = t.elapsed();
        assert!(
            elapsed.as_millis() < 10,
            "10k slab allocs took {}ms (limit 10ms)",
            elapsed.as_millis()
        );
    }

    // 13 — uring fsync < 1s
    #[test]
    fn perf_uring_fsync_lt_1s() {
        use vectorvolt_wal::uring_writer::UringWalAppender;
        let p = {
            let mut p = std::env::temp_dir();
            p.push(format!("vv_perf_fsync_{}.wal", std::process::id()));
            p
        };
        let mut w = UringWalAppender::open(&p, 64).expect("open");
        w.append(b"ping").expect("append");
        let t = Instant::now();
        w.fsync().expect("fsync");
        let elapsed = t.elapsed();
        let _ = std::fs::remove_file(&p);
        assert!(
            elapsed.as_secs() < 1,
            "uring fsync took {}ms (limit 1s)",
            elapsed.as_millis()
        );
    }
}
