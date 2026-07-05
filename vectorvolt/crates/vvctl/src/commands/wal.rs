use anyhow::Result;
use clap::Subcommand;
use vectorvolt_wal::{WalConfig, WalReader, list_segment_ids, segment_path, SegmentReader};

#[derive(Subcommand)]
pub enum WalCommand {
    /// Show WAL status: segments, frame count, chain tip hash.
    Status {
        #[arg(long, default_value = "./wal", help = "WAL directory")]
        dir: String,
    },

    /// Verify the hash chain across all segments.
    Verify {
        #[arg(long, default_value = "./wal")]
        dir: String,
    },

    /// Inspect frames in a specific segment.
    Inspect {
        #[arg(long, default_value = "./wal")]
        dir: String,

        /// Segment id to inspect (default: last segment).
        #[arg(long)]
        segment: Option<u64>,

        /// Print payloads as UTF-8 strings (if valid).
        #[arg(long, default_value_t = false)]
        payload: bool,
    },
}

pub fn run(cmd: WalCommand) -> Result<()> {
    match cmd {
        WalCommand::Status { dir } => cmd_status(&dir),
        WalCommand::Verify { dir } => cmd_verify(&dir),
        WalCommand::Inspect { dir, segment, payload } => cmd_inspect(&dir, segment, payload),
    }
}

fn cmd_status(dir: &str) -> Result<()> {
    let cfg = WalConfig::default().with_dir(dir).with_fsync(false);
    let reader = WalReader::open(cfg);
    let stats = reader.stats()?;

    println!("─── VectorVolt WAL Status ───────────────────────────────");
    println!("  Directory    : {}", dir);
    println!("  Segments     : {}", stats.segments);
    println!("  Total frames : {}", stats.total_frames);
    println!("  Total bytes  : {} ({:.2} MiB)", stats.total_bytes, stats.total_bytes as f64 / (1024.0 * 1024.0));
    println!("  First seq    : {}", stats.first_sequence);
    println!("  Last seq     : {}", stats.last_sequence);
    println!("  Chain tip    : {}", hex(&stats.chain_tip));
    println!("─────────────────────────────────────────────────────────");
    Ok(())
}

fn cmd_verify(dir: &str) -> Result<()> {
    let cfg = WalConfig::default().with_dir(dir).with_fsync(false);
    let reader = WalReader::open(cfg);

    print!("Verifying hash chain... ");
    match reader.verify() {
        Ok(n) => println!("✓ OK — {} frames verified", n),
        Err(e) => {
            println!("✗ FAILED");
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
    Ok(())
}

fn cmd_inspect(dir: &str, seg: Option<u64>, print_payload: bool) -> Result<()> {
    let ids = list_segment_ids(std::path::Path::new(dir))?;
    if ids.is_empty() {
        println!("No segments found in {}", dir);
        return Ok(());
    }

    let target_id = seg.unwrap_or(*ids.last().unwrap());
    let path = segment_path(std::path::Path::new(dir), target_id);

    if !path.exists() {
        anyhow::bail!("Segment {} not found at {}", target_id, path.display());
    }

    let mut reader = SegmentReader::open(&path)?;
    println!("─── Segment {:020} ─────────────────────────────────", target_id);
    println!("  File         : {}", path.display());
    println!("  Created      : {}", chrono::DateTime::from_timestamp_nanos(reader.header.created_ns as i64));
    println!("  Base seq     : {}", reader.header.base_seq);
    println!();

    let frames = reader.all_frames()?;
    println!("  Frames: {}", frames.len());
    println!();

    for f in &frames {
        let payload_display = if print_payload {
            match std::str::from_utf8(&f.payload) {
                Ok(s)  => format!("\"{}\"", s.chars().take(80).collect::<String>()),
                Err(_) => format!("<{} bytes binary>", f.payload.len()),
            }
        } else {
            format!("<{} bytes>", f.payload.len())
        };

        println!(
            "  seq={:<8} ts_ns={:<20} prev={} hash={} payload={}",
            f.sequence,
            f.timestamp_ns,
            &hex(&f.prev_hash)[..8],
            &hex(&f.frame_hash)[..8],
            payload_display,
        );
    }
    Ok(())
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}
