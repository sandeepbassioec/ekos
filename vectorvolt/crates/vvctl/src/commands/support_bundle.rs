//! `vvctl support-bundle` — collect diagnostics into a bundle file (VV-CTL-200).

use anyhow::Result;
use clap::Args;
use std::path::PathBuf;

#[derive(Args)]
pub struct SupportBundleArgs {
    /// Output file path (text bundle).
    #[arg(long, default_value = "vvctl-support-bundle.txt")]
    pub output: PathBuf,
}

pub fn run(args: SupportBundleArgs) -> Result<()> {
    let version = env!("CARGO_PKG_VERSION");
    let hal_desc = vectorvolt_hal::describe();

    let mut lines = Vec::new();
    lines.push("vvctl support bundle".to_string());
    lines.push(format!("version:  {version}"));
    lines.push(format!("hal:      {hal_desc}"));
    lines.push(format!("platform: {}", std::env::consts::OS));
    lines.push(format!("arch:     {}", std::env::consts::ARCH));

    // Collect any .yaml / .toml files in current dir
    if let Ok(entries) = std::fs::read_dir(".") {
        for entry in entries.flatten() {
            let p = entry.path();
            if p.extension().is_some_and(|e| e == "yaml" || e == "toml") {
                lines.push(format!("config: {}", p.display()));
            }
        }
    }

    let content = lines.join("\n") + "\n";
    std::fs::write(&args.output, content.as_bytes())
        .map_err(|e| anyhow::anyhow!("write bundle to {}: {e}", args.output.display()))?;

    println!("support bundle written to: {}", args.output.display());
    Ok(())
}
