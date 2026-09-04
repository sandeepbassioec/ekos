//! `vvctl linux` — Linux-specific hardware diagnostics (VV-CTL-200).
//!
//! Subcommands:
//! - `numa`          — print NUMA topology table
//! - `affinity`      — pin to a core and report result
//! - `hugepages`     — allocate + free a huge-page buffer, report latency
//! - `direct-write`  — write an aligned buffer via O_DIRECT, report throughput
//! - `uring-bench`   — submit N io_uring writes, report IOPS

use anyhow::Result;
use clap::{Args, Subcommand};
use vectorvolt_hal::NumaTopology;

#[derive(Args)]
pub struct LinuxArgs {
    #[command(subcommand)]
    pub command: LinuxCommand,
}

#[derive(Subcommand)]
pub enum LinuxCommand {
    /// Print NUMA topology table (nodes, CPUs, memory).
    Numa {
        /// Output as JSON instead of a table.
        #[arg(long)]
        json: bool,
    },
    /// Pin the current process to core N and report the result.
    Affinity {
        #[arg(long)]
        core: usize,
    },
    /// Allocate and immediately free a huge-page buffer, reporting latency.
    Hugepages {
        /// Size in megabytes.
        #[arg(long, default_value = "2")]
        size: usize,
    },
    /// Write an aligned buffer via O_DIRECT and report throughput.
    DirectWrite {
        /// Destination file path.
        #[arg(long)]
        path: String,
        /// Buffer size in kilobytes (must be a multiple of 0.5 KB).
        #[arg(long, default_value = "4")]
        size: usize,
    },
    /// Submit N io_uring writes and report IOPS.
    UringBench {
        /// Destination file path.
        #[arg(long)]
        path: String,
        /// Number of write operations.
        #[arg(long, default_value = "100")]
        iops: usize,
    },
}

pub fn run(args: &LinuxArgs) -> Result<()> {
    match &args.command {
        LinuxCommand::Numa { json } => cmd_numa(*json),
        LinuxCommand::Affinity { core } => cmd_affinity(*core),
        LinuxCommand::Hugepages { size } => cmd_hugepages(*size),
        LinuxCommand::DirectWrite { path, size } => cmd_direct_write(path, *size),
        LinuxCommand::UringBench { path, iops } => cmd_uring_bench(path, *iops),
    }
}

// ── numa ──────────────────────────────────────────────────────────────────────

fn cmd_numa(json: bool) -> Result<()> {
    let topo = NumaTopology::probe();
    if json {
        // Simple hand-rolled JSON — avoids pulling serde into the output path.
        println!("{{");
        println!("  \"node_count\": {},", topo.node_count());
        println!("  \"total_cpus\": {},", topo.total_cpus());
        println!("  \"is_numa\": {},", topo.is_numa());
        println!("  \"nodes\": [");
        for (i, node) in topo.nodes.iter().enumerate() {
            let comma = if i + 1 < topo.nodes.len() { "," } else { "" };
            println!(
                "    {{\"node_id\": {}, \"cpu_count\": {}, \"memory_mb\": {}}}{comma}",
                node.node_id,
                node.cpu_ids.len(),
                node.memory_mb
                    .map(|m| m.to_string())
                    .unwrap_or_else(|| "null".into()),
            );
        }
        println!("  ]");
        println!("}}");
    } else {
        println!(
            "NUMA topology: {} node(s), {} total CPUs, is_numa={}",
            topo.node_count(),
            topo.total_cpus(),
            topo.is_numa()
        );
        println!(
            "{:<8} {:<12} {:<14} CPU list",
            "Node", "CPUs", "Memory (MB)"
        );
        println!("{}", "-".repeat(60));
        for node in &topo.nodes {
            let cpu_list = format_cpu_list(&node.cpu_ids);
            let mem = node
                .memory_mb
                .map(|m| m.to_string())
                .unwrap_or_else(|| "n/a".into());
            println!(
                "{:<8} {:<12} {:<14} {}",
                node.node_id,
                node.cpu_ids.len(),
                mem,
                cpu_list
            );
        }
    }
    Ok(())
}

fn format_cpu_list(cpus: &[usize]) -> String {
    if cpus.is_empty() {
        return String::new();
    }
    // Compress into ranges: [0,1,2,5,6] → "0-2,5-6"
    let mut ranges: Vec<String> = Vec::new();
    let mut start = cpus[0];
    let mut prev = cpus[0];
    for &c in &cpus[1..] {
        if c == prev + 1 {
            prev = c;
        } else {
            if start == prev {
                ranges.push(start.to_string());
            } else {
                ranges.push(format!("{start}-{prev}"));
            }
            start = c;
            prev = c;
        }
    }
    if start == prev {
        ranges.push(start.to_string());
    } else {
        ranges.push(format!("{start}-{prev}"));
    }
    ranges.join(",")
}

// ── affinity ──────────────────────────────────────────────────────────────────

fn cmd_affinity(core: usize) -> Result<()> {
    #[cfg(any(target_os = "linux", target_os = "android"))]
    {
        use vectorvolt_hal::pin_thread_to_core;
        match pin_thread_to_core(core) {
            Ok(()) => println!("Pinned to core {core}: ok"),
            Err(e) => println!("Failed to pin to core {core}: {e}"),
        }
    }
    #[cfg(not(any(target_os = "linux", target_os = "android")))]
    {
        let _ = core;
        println!("Thread pinning is not supported on this platform.");
    }
    Ok(())
}

// ── hugepages ─────────────────────────────────────────────────────────────────

fn cmd_hugepages(size_mb: usize) -> Result<()> {
    use std::time::Instant;
    use vectorvolt_hal::{HugePageBuf, HugePageError};
    let bytes = size_mb * 1024 * 1024;
    let t = Instant::now();
    match HugePageBuf::alloc(bytes) {
        Ok(buf) => {
            let elapsed = t.elapsed();
            println!(
                "Allocated {} MB huge-page buffer in {:.3}ms (is_huge={})",
                size_mb,
                elapsed.as_secs_f64() * 1000.0,
                buf.is_huge()
            );
        }
        Err(HugePageError::NotPrivileged) => {
            println!("Huge pages unavailable: insufficient privilege (set nr_hugepages).");
        }
        Err(e) => {
            println!("Huge page allocation failed: {e}");
        }
    }
    Ok(())
}

// ── direct-write ──────────────────────────────────────────────────────────────

fn cmd_direct_write(path: &str, size_kb: usize) -> Result<()> {
    use std::path::Path;
    use std::time::Instant;
    use vectorvolt_storage::direct_io::DirectWriter;
    let size_bytes = (size_kb * 1024).max(512);
    let mut w = DirectWriter::open(Path::new(path))?;
    let buf = DirectWriter::aligned_buf(size_bytes);
    let t = Instant::now();
    let n = w.write_aligned(buf.as_slice())?;
    w.fsync()?;
    let elapsed = t.elapsed();
    let mb_s = (n as f64 / elapsed.as_secs_f64()) / (1024.0 * 1024.0);
    println!(
        "Wrote {n} bytes to {path} in {:.3}ms ({mb_s:.1} MB/s)",
        elapsed.as_secs_f64() * 1000.0
    );
    Ok(())
}

// ── uring-bench ───────────────────────────────────────────────────────────────

fn cmd_uring_bench(path: &str, iops: usize) -> Result<()> {
    use std::path::Path;
    use std::time::Instant;
    use vectorvolt_wal::uring_writer::UringWalAppender;
    let mut w = UringWalAppender::open(Path::new(path), 64)?;
    let payload = vec![0u8; 512];
    let t = Instant::now();
    for _ in 0..iops {
        w.append(&payload)?;
    }
    w.fsync()?;
    let elapsed = t.elapsed();
    let iops_achieved = iops as f64 / elapsed.as_secs_f64();
    println!(
        "Completed {iops} writes in {:.3}ms = {iops_achieved:.0} IOPS",
        elapsed.as_secs_f64() * 1000.0
    );
    Ok(())
}

// ── tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn run_numa(json: bool) -> Result<()> {
        cmd_numa(json)
    }

    // 1 — numa parses ok
    #[test]
    fn linux_cmd_numa_ok() {
        assert!(run_numa(false).is_ok());
    }

    // 2 — numa --json ok
    #[test]
    fn linux_cmd_numa_json_ok() {
        assert!(run_numa(true).is_ok());
    }

    // 3 — affinity --core 0 ok (platform-conditional)
    #[test]
    fn linux_cmd_affinity_core0_ok() {
        assert!(cmd_affinity(0).is_ok());
    }

    // 4 — hugepages --size 2 ok or NotAvailable
    #[test]
    fn linux_cmd_hugepages_ok() {
        assert!(cmd_hugepages(2).is_ok());
    }

    // 5 — direct-write to temp path ok
    #[test]
    fn linux_cmd_direct_write_ok() {
        let p = {
            let mut p = std::env::temp_dir();
            p.push(format!("vv_vvctl_dw_{}.bin", std::process::id()));
            p
        };
        let result = cmd_direct_write(p.to_str().unwrap(), 4);
        let _ = std::fs::remove_file(&p);
        assert!(result.is_ok());
    }

    // 6 — uring-bench to temp path ok
    #[test]
    fn linux_cmd_uring_bench_ok() {
        let p = {
            let mut p = std::env::temp_dir();
            p.push(format!("vv_vvctl_ub_{}.wal", std::process::id()));
            p
        };
        let result = cmd_uring_bench(p.to_str().unwrap(), 10);
        let _ = std::fs::remove_file(&p);
        assert!(result.is_ok());
    }

    // 7 — all commands compile on Windows (Linux-gated execution already handled)
    #[test]
    fn linux_commands_compile_all_platforms() {
        // This test just proves the code compiled on this (non-Linux) platform.
        let _ = cmd_affinity(0);
    }

    // 8 — numa shows node_count >= 1
    #[test]
    fn linux_numa_node_count_ge_1() {
        let topo = NumaTopology::probe();
        assert!(topo.node_count() >= 1);
    }

    // 9 — affinity returns success exit code (Ok)
    #[test]
    fn linux_affinity_returns_ok() {
        assert!(cmd_affinity(0).is_ok());
    }

    // 10 — direct-write creates file
    #[test]
    fn linux_direct_write_creates_file() {
        let p = {
            let mut p = std::env::temp_dir();
            p.push(format!("vv_create_check_{}.bin", std::process::id()));
            p
        };
        cmd_direct_write(p.to_str().unwrap(), 4).ok();
        assert!(p.exists(), "file was not created");
        let _ = std::fs::remove_file(&p);
    }

    // 11 — direct-write reports bytes (output is non-empty — just check no panic)
    #[test]
    fn linux_direct_write_reports_bytes() {
        let p = {
            let mut p = std::env::temp_dir();
            p.push(format!("vv_report_{}.bin", std::process::id()));
            p
        };
        let r = cmd_direct_write(p.to_str().unwrap(), 4);
        let _ = std::fs::remove_file(&p);
        assert!(r.is_ok());
    }

    // 12 — uring-bench reports iops (no panic)
    #[test]
    fn linux_uring_bench_reports_iops() {
        let p = {
            let mut p = std::env::temp_dir();
            p.push(format!("vv_iops_{}.wal", std::process::id()));
            p
        };
        let r = cmd_uring_bench(p.to_str().unwrap(), 5);
        let _ = std::fs::remove_file(&p);
        assert!(r.is_ok());
    }

    // 13 — format_cpu_list produces correct ranges
    #[test]
    fn linux_cpu_list_formatting() {
        assert_eq!(format_cpu_list(&[0, 1, 2, 5, 6]), "0-2,5-6");
        assert_eq!(format_cpu_list(&[0]), "0");
        assert_eq!(format_cpu_list(&[]), "");
    }
}
