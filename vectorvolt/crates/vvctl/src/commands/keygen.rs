//! `vvctl keygen` -- generate a cryptographically random shared secret.
//!
//! Prints a 32-byte hex string suitable for use with `--secret` on both
//! `vvctl serve` and `vvctl connect`. Example:
//!
//! ```text
//! $ vvctl keygen
//! a3f1e8c2...  # 64 hex chars, copy-paste into --secret
//! ```

use anyhow::Result;
use clap::Args;

#[derive(Args)]
pub struct KeygenArgs {}

pub fn run(_args: KeygenArgs) -> Result<()> {
    let mut bytes = [0u8; 32];
    getrandom::fill(&mut bytes)
        .map_err(|e| anyhow::anyhow!("OS entropy source unavailable: {e}"))?;
    println!("{}", hex::encode(bytes));
    Ok(())
}
