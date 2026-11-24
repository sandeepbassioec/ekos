use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use vectorvolt_cluster::versionstamp::*;

fn fake(t: &Arc<AtomicU64>) -> WallClock {
    let t = t.clone();
    Arc::new(move || t.load(Ordering::SeqCst))
}

#[test]
fn monotonic_under_backwards_clock() {
    let t = Arc::new(AtomicU64::new(10_000));
    let c = VersionstampClock::open(1, 1, None, fake(&t)).unwrap();
    let mut prev = c.next().unwrap();
    for i in 0..200_000u64 {
        if i % 50_000 == 0 {
            t.fetch_sub(5_000, Ordering::SeqCst); // clock steps back
        }
        let s = c.next().unwrap();
        assert!(s > prev);
        assert!(s.to_bytes() > prev.to_bytes());
        prev = s;
    }
}

#[test]
fn persists_across_restart_even_with_older_clock() {
    let dir = tempfile::tempdir().unwrap();
    let p = dir.path().join("hlc");
    let t = Arc::new(AtomicU64::new(50_000));
    let c = VersionstampClock::open(7, 3, Some(p.clone()), fake(&t)).unwrap();
    let mut last = c.next().unwrap();
    for _ in 0..10 {
        last = c.next().unwrap();
    }
    drop(c);
    t.store(1_000, Ordering::SeqCst); // restart with a clock far in the past
    let c2 = VersionstampClock::open(7, 0, Some(p), fake(&t)).unwrap();
    assert_eq!(c2.epoch(), 3);
    assert!(c2.next().unwrap() > last);
}

#[test]
fn causal_order_across_three_nodes() {
    let ts: Vec<_> = [1_000u64, 500, 2_000]
        .iter()
        .map(|v| Arc::new(AtomicU64::new(*v)))
        .collect();
    let n: Vec<_> = ts
        .iter()
        .enumerate()
        .map(|(i, t)| VersionstampClock::open(i as u16 + 1, 1, None, fake(t)).unwrap())
        .collect();
    let a = n[0].next().unwrap();
    let b = n[1].observe(a).unwrap(); // node 2's clock is behind node 1's
    assert!(b > a);
    let c = n[2].observe(b).unwrap();
    assert!(c > b);
    let d = n[1].observe(c).unwrap();
    assert!(d > c);
    let e = n[0].observe(d).unwrap();
    assert!(e > d && e > a);
    // epoch propagates
    n[2].set_epoch(5).unwrap();
    let f = n[2].next().unwrap();
    let g = n[0].observe(f).unwrap();
    assert_eq!(g.epoch, 5);
    assert!(g > f);
    // distinct nodes never collide
    let x = n[0].next().unwrap();
    let y = n[1].next().unwrap();
    assert_ne!(x, y);
}

#[test]
fn byte_order_matches_logical_order() {
    let mut v = Vec::new();
    for epoch in [0u16, 1, 256, 65535] {
        for phys in [0u64, 1, 255, 256, 1 << 40, (1 << 48) - 1] {
            for logical in [0u16, 1, 255, 256, 65535] {
                for node in [0u16, 1, 300] {
                    v.push(Versionstamp {
                        epoch,
                        physical_ms: phys,
                        logical,
                        node,
                    });
                }
            }
        }
    }
    for a in &v {
        assert_eq!(Versionstamp::from_bytes(&a.to_bytes()), Some(*a));
        for b in &v {
            assert_eq!(a.cmp(b), a.to_bytes().cmp(&b.to_bytes()));
        }
    }
}
