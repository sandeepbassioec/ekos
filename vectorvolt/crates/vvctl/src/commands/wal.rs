use anyhow::{Context, Result};
use clap::Subcommand;
use std::path::{Path, PathBuf};
use vectorvolt_wal::{
    SegmentReader, WalConfig, WalCursor, WalReader, WalWriter, list_segment_ids, segment_path,
};

#[derive(Clone, Debug, Default, clap::ValueEnum)]
pub enum ReplayOutput {
    #[default]
    Table,
    Json,
}

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
        /// Output format.
        #[arg(long, short, value_enum, default_value = "table")]
        output: ReplayOutput,
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

    /// Read WAL entries from a directory and print a record count summary.
    ///
    /// Reads all WAL segment files in order and prints: total records,
    /// record types seen, and earliest/latest sequence numbers.
    /// Use `--limit` to cap how many records to read.
    Replay {
        /// WAL directory to read from.
        #[arg(short, long)]
        dir: PathBuf,
        /// Maximum records to read (0 = unlimited).
        #[arg(long, default_value = "0")]
        limit: u64,
        /// Output format.
        #[arg(long, short, value_enum, default_value = "table")]
        output: ReplayOutput,
    },

    /// Export WAL frames as JSON or table to stdout.
    Export {
        /// WAL directory.
        #[arg(short, long)]
        dir: PathBuf,
        /// Maximum frames to export (0 = unlimited).
        #[arg(long)]
        limit: Option<usize>,
        /// Output format.
        #[arg(long, short, value_enum, default_value = "table")]
        output: ReplayOutput,
    },

    /// Delete WAL segment files whose max sequence is less than `before`.
    Trim {
        /// WAL directory.
        #[arg(short, long)]
        dir: PathBuf,
        /// Delete segments whose max sequence < this value.
        #[arg(long)]
        before: u64,
    },

    /// Write a typed checkpoint record to the WAL (VV-WAL-011).
    ///
    /// Appends a `CHECKPOINT:<lsn>:<ts>` payload to the WAL so recovery
    /// can identify safe truncation points.
    Checkpoint {
        /// WAL directory.
        #[arg(short, long)]
        dir: PathBuf,
    },

    /// Print aggregate statistics for a WAL directory.
    ///
    /// Reads all segments and reports: total frames, total bytes,
    /// average frame size, first and last sequence numbers.
    Stats {
        /// WAL directory.
        #[arg(short, long)]
        dir: PathBuf,
        /// Output format.
        #[arg(long, short, value_enum, default_value = "table")]
        output: ReplayOutput,
    },
}

pub fn run(cmd: WalCommand) -> Result<()> {
    match cmd {
        WalCommand::Status { dir } => cmd_status(&dir),
        WalCommand::Verify { dir, output } => cmd_wal_verify(&dir, &output),
        WalCommand::Inspect {
            dir,
            segment,
            payload,
        } => cmd_inspect(&dir, segment, payload),
        WalCommand::Replay { dir, limit, output } => cmd_replay(&dir, limit, &output),
        WalCommand::Export { dir, limit, output } => cmd_wal_export(&dir, limit, &output),
        WalCommand::Trim { dir, before } => cmd_wal_trim(&dir, before),
        WalCommand::Stats { dir, output } => cmd_wal_stats(&dir, &output),
        WalCommand::Checkpoint { dir } => cmd_wal_checkpoint(&dir),
    }
}

fn cmd_wal_checkpoint(dir: &Path) -> Result<()> {
    if !dir.exists() {
        anyhow::bail!("WAL directory not found: {}", dir.display());
    }
    let cfg = WalConfig::default()
        .with_dir(dir.to_string_lossy().as_ref())
        .with_fsync(true);
    let writer = WalWriter::open(cfg).with_context(|| "open WAL writer")?;
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos() as u64;
    let payload = format!("CHECKPOINT:0:{ts}").into_bytes();
    let tip = writer
        .append(payload)
        .with_context(|| "append checkpoint")?;
    writer.flush().with_context(|| "flush checkpoint")?;
    println!("checkpoint written: seq={} ts={}", tip.sequence, ts);
    Ok(())
}

fn cmd_status(dir: &str) -> Result<()> {
    let cfg = WalConfig::default().with_dir(dir).with_fsync(false);
    let reader = WalReader::open(cfg);
    let stats = reader.stats()?;

    println!("─── VectorVolt WAL Status ───────────────────────────────");
    println!("  Directory    : {}", dir);
    println!("  Segments     : {}", stats.segments);
    println!("  Total frames : {}", stats.total_frames);
    println!(
        "  Total bytes  : {} ({:.2} MiB)",
        stats.total_bytes,
        stats.total_bytes as f64 / (1024.0 * 1024.0)
    );
    println!("  First seq    : {}", stats.first_sequence);
    println!("  Last seq     : {}", stats.last_sequence);
    println!("  Chain tip    : {}", hex(&stats.chain_tip));
    println!("─────────────────────────────────────────────────────────");
    Ok(())
}

fn cmd_wal_verify(dir: &str, output: &ReplayOutput) -> Result<()> {
    let cfg = WalConfig::default().with_dir(dir).with_fsync(false);
    let reader = WalReader::open(cfg);
    match reader.verify() {
        Ok(n) => match output {
            ReplayOutput::Json => println!("{{\"ok\":true,\"frames\":{n}}}"),
            ReplayOutput::Table => println!("ok: {n} frames verified"),
        },
        Err(e) => {
            match output {
                ReplayOutput::Json => println!("{{\"ok\":false,\"error\":\"{e}\"}}"),
                ReplayOutput::Table => eprintln!("FAILED: {e}"),
            }
            std::process::exit(1);
        }
    }
    Ok(())
}

pub fn cmd_wal_export(dir: &Path, limit: Option<usize>, output: &ReplayOutput) -> Result<()> {
    if !dir.exists() {
        anyhow::bail!("WAL directory not found: {}", dir.display());
    }
    let cfg = WalConfig::default()
        .with_dir(dir.to_string_lossy().as_ref())
        .with_fsync(false);
    let reader = WalReader::open(cfg);
    let cap = limit.unwrap_or(usize::MAX);
    let mut frames_info: Vec<(u64, u8, usize)> = Vec::new();
    reader.scan(WalCursor::beginning(), |frame| {
        if frames_info.len() >= cap {
            return false;
        }
        frames_info.push((
            frame.sequence,
            frame.payload_type as u8,
            frame.payload.len(),
        ));
        frames_info.len() < cap
    })?;
    match output {
        ReplayOutput::Json => {
            let arr: Vec<serde_json::Value> = frames_info.iter().map(|(seq, op, plen)| {
                serde_json::json!({"seq": seq, "op_code": op, "payload_len": plen})
            }).collect();
            println!("{}", serde_json::to_string_pretty(&arr)?);
        }
        ReplayOutput::Table => {
            println!("{:<12} {:<8} {:<12}", "seq", "op_code", "payload_len");
            println!("{}", "─".repeat(34));
            for (seq, op, plen) in &frames_info {
                println!("{:<12} {:<8} {:<12}", seq, op, plen);
            }
            println!("\n{} frame(s)", frames_info.len());
        }
    }
    Ok(())
}

pub fn cmd_wal_trim(dir: &Path, before: u64) -> Result<()> {
    if !dir.exists() {
        anyhow::bail!("WAL directory not found: {}", dir.display());
    }
    let ids = list_segment_ids(dir)?;
    let mut deleted = 0usize;
    for id in &ids {
        let path = segment_path(dir, *id);
        let mut seg = SegmentReader::open(&path)?;
        let frames = seg.all_frames()?;
        let max_seq = frames.iter().map(|f| f.sequence).max().unwrap_or(0);
        if max_seq < before {
            std::fs::remove_file(&path).with_context(|| format!("remove '{}'", path.display()))?;
            deleted += 1;
        }
    }
    println!("{deleted} segment(s) deleted");
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
    println!(
        "─── Segment {:020} ─────────────────────────────────",
        target_id
    );
    println!("  File         : {}", path.display());
    println!(
        "  Created      : {}",
        chrono::DateTime::from_timestamp_nanos(reader.header.created_ns as i64)
    );
    println!("  Base seq     : {}", reader.header.base_seq);
    println!();

    let frames = reader.all_frames()?;
    println!("  Frames: {}", frames.len());
    println!();

    for f in &frames {
        let payload_display = if print_payload {
            match std::str::from_utf8(&f.payload) {
                Ok(s) => format!("\"{}\"", s.chars().take(80).collect::<String>()),
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

pub fn cmd_replay(dir: &Path, limit: u64, output: &ReplayOutput) -> Result<()> {
    if !dir.exists() {
        anyhow::bail!("WAL directory not found: {}", dir.display());
    }

    let cfg = WalConfig::default()
        .with_dir(dir.to_string_lossy().as_ref())
        .with_fsync(false);
    let reader = WalReader::open(cfg);

    let mut total_frames: u64 = 0;
    let mut first_seq: Option<u64> = None;
    let mut last_seq: Option<u64> = None;

    let cap = if limit == 0 { u64::MAX } else { limit };

    reader.scan(WalCursor::beginning(), |frame| {
        if total_frames >= cap {
            return false;
        }
        if first_seq.is_none() {
            first_seq = Some(frame.sequence);
        }
        last_seq = Some(frame.sequence);
        total_frames += 1;
        total_frames < cap
    })?;

    let ids = list_segment_ids(dir)?;
    let segments = ids.len() as u64;

    match output {
        ReplayOutput::Table => {
            println!("─── VectorVolt WAL Replay ───────────────────────────────");
            println!("  Directory    : {}", dir.display());
            println!("  Segments     : {}", segments);
            println!("  Records read : {}", total_frames);
            println!(
                "  Limit        : {}",
                if limit == 0 {
                    "unlimited".into()
                } else {
                    limit.to_string()
                }
            );
            println!("  First seq    : {}", first_seq.unwrap_or(0));
            println!("  Last seq     : {}", last_seq.unwrap_or(0));
            println!("─────────────────────────────────────────────────────────");
        }
        ReplayOutput::Json => {
            println!(
                "{{\"directory\":\"{}\",\"segments\":{},\"records_read\":{},\"limit\":{},\"first_seq\":{},\"last_seq\":{}}}",
                dir.display(),
                segments,
                total_frames,
                limit,
                first_seq.unwrap_or(0),
                last_seq.unwrap_or(0),
            );
        }
    }

    Ok(())
}

pub fn cmd_wal_stats(dir: &Path, output: &ReplayOutput) -> Result<()> {
    if !dir.exists() {
        anyhow::bail!("WAL directory not found: {}", dir.display());
    }

    let cfg = WalConfig::default()
        .with_dir(dir.to_string_lossy().as_ref())
        .with_fsync(false);
    let reader = WalReader::open(cfg);
    let stats = reader.stats()?;

    let avg_frame_bytes = stats
        .total_bytes
        .checked_div(stats.total_frames)
        .unwrap_or(0);

    match output {
        ReplayOutput::Table => {
            println!("─── VectorVolt WAL Stats ────────────────────────────────");
            println!("  Directory    : {}", dir.display());
            println!("  Segments     : {}", stats.segments);
            println!("  Total frames : {}", stats.total_frames);
            println!("  Total bytes  : {}", stats.total_bytes);
            println!("  Avg frame    : {} bytes", avg_frame_bytes);
            println!("  First seq    : {}", stats.first_sequence);
            println!("  Last seq     : {}", stats.last_sequence);
            println!("─────────────────────────────────────────────────────────");
        }
        ReplayOutput::Json => {
            println!(
                "{{\"directory\":\"{}\",\"segments\":{},\"total_frames\":{},\"total_bytes\":{},\"avg_frame_bytes\":{},\"first_seq\":{},\"last_seq\":{}}}",
                dir.display(),
                stats.segments,
                stats.total_frames,
                stats.total_bytes,
                avg_frame_bytes,
                stats.first_sequence,
                stats.last_sequence,
            );
        }
    }

    Ok(())
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}
