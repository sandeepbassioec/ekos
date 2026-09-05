use anyhow::{Result, bail};
use clap::Subcommand;

#[derive(Subcommand)]
pub enum XdpCommand {
    /// Show XDP socket statistics for a NIC queue (Linux + CAP_NET_RAW required).
    Stats {
        /// Network interface name (e.g. eth0, lo).
        #[arg(short, long, default_value = "lo")]
        iface: String,
    },

    /// Run a short in-process XDP ring throughput benchmark (no NIC required).
    Bench {
        /// Number of TX/RX round-trips to simulate.
        #[arg(short = 'n', long, default_value = "65536")]
        count: u32,

        /// Ring capacity (power of two).
        #[arg(short = 'r', long, default_value = "256")]
        ring_size: u32,
    },

    /// Print UMEM layout for the given config (no root required).
    Umem {
        /// Number of frames (power of two).
        #[arg(long, default_value = "4096")]
        frames: u32,

        /// Frame size in bytes (power of two, ≥ 2048).
        #[arg(long, default_value = "2048")]
        frame_size: u32,

        /// Headroom bytes reserved before each packet.
        #[arg(long, default_value = "0")]
        headroom: u32,
    },
}

pub fn run(cmd: XdpCommand) -> Result<()> {
    match cmd {
        XdpCommand::Stats { iface } => cmd_stats(&iface),
        XdpCommand::Bench { count, ring_size } => cmd_bench(count, ring_size),
        XdpCommand::Umem {
            frames,
            frame_size,
            headroom,
        } => cmd_umem(frames, frame_size, headroom),
    }
}

#[cfg(target_os = "linux")]
fn cmd_stats(iface: &str) -> Result<()> {
    use vectorvolt_xdp::socket::XdpSocket;
    use vectorvolt_xdp::umem::{Umem, UmemConfig};

    let umem = Umem::new(UmemConfig::default())?;
    let sock = XdpSocket::new(&umem)?;
    sock.configure_rings(64)?;
    let ifindex = XdpSocket::ifindex_of(iface)?;
    sock.bind(ifindex, 0)?;
    let stats = sock.statistics()?;
    println!("XDP statistics for {iface}:");
    println!("  rx_dropped              : {}", stats.rx_dropped);
    println!("  rx_invalid_descs        : {}", stats.rx_invalid_descs);
    println!("  tx_invalid_descs        : {}", stats.tx_invalid_descs);
    println!("  rx_ring_full            : {}", stats.rx_ring_full);
    println!(
        "  rx_fill_ring_empty_descs: {}",
        stats.rx_fill_ring_empty_descs
    );
    println!("  tx_ring_empty_descs     : {}", stats.tx_ring_empty_descs);
    Ok(())
}

#[cfg(not(target_os = "linux"))]
fn cmd_stats(_iface: &str) -> Result<()> {
    bail!("XDP is only supported on Linux");
}

#[cfg(target_os = "linux")]
fn cmd_bench(count: u32, ring_size: u32) -> Result<()> {
    use vectorvolt_xdp::rings::RingIndex;

    if !ring_size.is_power_of_two() {
        bail!("--ring-size must be a power of two, got {ring_size}");
    }

    let mut tx = RingIndex::new(ring_size);
    let mut rx = RingIndex::new(ring_size);

    let start = std::time::Instant::now();
    let mut total: u64 = 0;
    let mut remaining = count;

    while remaining > 0 {
        let n = remaining.min(ring_size).min(tx.free_slots());
        if n == 0 {
            let drain = tx.available().min(rx.free_slots());
            rx.produce(drain);
            tx.consume(drain);
            rx.consume(drain);
            continue;
        }
        tx.produce(n);
        remaining -= n;
        total += n as u64;
        let drain = tx.available().min(rx.free_slots());
        rx.produce(drain);
        tx.consume(drain);
        rx.consume(drain);
    }

    let elapsed = start.elapsed();
    let ns_per_op = elapsed.as_nanos() as f64 / total.max(1) as f64;
    println!("XDP ring bench: {total} ops in {elapsed:.3?} ({ns_per_op:.1} ns/op)");
    Ok(())
}

#[cfg(not(target_os = "linux"))]
fn cmd_bench(_count: u32, _ring_size: u32) -> Result<()> {
    bail!("XDP ring bench is only supported on Linux");
}

#[cfg(target_os = "linux")]
fn cmd_umem(frames: u32, frame_size: u32, headroom: u32) -> Result<()> {
    use vectorvolt_xdp::umem::UmemConfig;

    let cfg = UmemConfig {
        frame_count: frames,
        frame_size,
        headroom,
    };
    if !cfg.is_valid() {
        bail!(
            "invalid UMEM config: frame_count and frame_size must be powers of two, frame_size > headroom"
        );
    }
    let total_mb = cfg.total_bytes() as f64 / 1_048_576.0;
    println!("UMEM layout:");
    println!("  frames     : {frames}");
    println!("  frame_size : {frame_size} B");
    println!("  headroom   : {headroom} B");
    println!("  data_offset: {headroom} B");
    println!("  data_window: {} B", frame_size - headroom);
    println!("  total      : {total_mb:.2} MiB");
    println!();
    println!("  frame 0 @ offset {}", cfg.frame_addr(0));
    if frames > 1 {
        println!("  frame 1 @ offset {}", cfg.frame_addr(1));
    }
    if frames > 2 {
        println!("  frame 2 @ offset {}", cfg.frame_addr(2));
    }
    println!("  ...");
    println!(
        "  frame {} @ offset {}",
        frames - 1,
        cfg.frame_addr(frames - 1)
    );
    Ok(())
}

#[cfg(not(target_os = "linux"))]
fn cmd_umem(_frames: u32, _frame_size: u32, _headroom: u32) -> Result<()> {
    bail!("XDP UMEM commands are only supported on Linux");
}
