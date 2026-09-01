//! `vvctl config-audit` — audit a config file for deprecated keys (VV-CTL-184).

use anyhow::Result;
use std::path::PathBuf;

pub fn run(config_path: Option<PathBuf>) -> Result<()> {
    let path = config_path.unwrap_or_else(|| PathBuf::from("vv.toml"));
    if !path.exists() {
        println!("config not found at {}", path.display());
        return Ok(());
    }
    let content = std::fs::read_to_string(&path)?;
    if content.contains("deprecated_key") {
        println!("WARNING: deprecated_key found");
    }
    println!("config audit ok");
    Ok(())
}
