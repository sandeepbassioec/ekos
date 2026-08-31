//! `vvctl profile` — emit the host SystemProfile as JSON or a human-readable table (VV-CFG-001).
//!
//! Operators use this for capacity planning, CI assertions on hardware class,
//! and piping into `vvctl config init` overrides.  The primary output is JSON
//! so it is machine-readable without further parsing.

use anyhow::Result;
use clap::{Args, ValueEnum};
use vectorvolt_engine::SystemProfile;

#[derive(Clone, Debug, Default, ValueEnum)]
pub enum ProfileOutput {
    /// Compact JSON object (default — machine-readable).
    #[default]
    Json,
    /// Human-readable key=value table.
    Table,
}

#[derive(Args)]
pub struct ProfileArgs {
    /// Output format.
    #[arg(long, short, value_enum, default_value = "json")]
    pub output: ProfileOutput,

    /// Print recommendations only (flush_threshold + partition_count).
    #[arg(long)]
    pub recommendations: bool,
}

pub fn run(args: ProfileArgs) -> Result<()> {
    let p = SystemProfile::probe();
    match args.output {
        ProfileOutput::Json => {
            if args.recommendations {
                println!(
                    "{{\"recommended_flush_threshold\":{},\"recommended_partition_count\":{}}}",
                    p.recommended_flush_threshold, p.recommended_partition_count
                );
            } else {
                println!(
                    concat!(
                        "{{",
                        "\"logical_cores\":{lc},",
                        "\"physical_cores\":{pc},",
                        "\"numa_node_count\":{nn},",
                        "\"simd_tier\":\"{simd}\",",
                        "\"huge_pages_available\":{hp},",
                        "\"os_page_size\":{ps},",
                        "\"recommended_flush_threshold\":{ft},",
                        "\"recommended_partition_count\":{pn}",
                        "}}"
                    ),
                    lc = p.logical_cores,
                    pc = p.physical_cores,
                    nn = p.numa_node_count,
                    simd = p.simd_tier.name(),
                    hp = p.huge_pages_available,
                    ps = p.os_page_size,
                    ft = p.recommended_flush_threshold,
                    pn = p.recommended_partition_count,
                );
            }
        }
        ProfileOutput::Table => {
            if args.recommendations {
                println!(
                    "recommended_flush_threshold  = {}",
                    p.recommended_flush_threshold
                );
                println!(
                    "recommended_partition_count  = {}",
                    p.recommended_partition_count
                );
            } else {
                println!("logical_cores               = {}", p.logical_cores);
                println!("physical_cores              = {}", p.physical_cores);
                println!("numa_node_count             = {}", p.numa_node_count);
                println!("simd_tier                   = {}", p.simd_tier.name());
                println!("huge_pages_available        = {}", p.huge_pages_available);
                println!("os_page_size                = {}", p.os_page_size);
                println!(
                    "recommended_flush_threshold = {}",
                    p.recommended_flush_threshold
                );
                println!(
                    "recommended_partition_count = {}",
                    p.recommended_partition_count
                );
            }
        }
    }
    Ok(())
}
