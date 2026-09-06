//! `vvctl install` — provision the host for VectorVolt (VV-SEC-003).
//!
//! Creates the `vectorvolt` system user (Linux, requires root), sets data
//! directory permissions to 700/600, and prints a master key to stdout for
//! the operator to persist in a secrets manager.

use anyhow::Result;
use clap::Args;
use std::path::PathBuf;
use vectorvolt_crypto::{KeyStore, ensure_system_user, harden_data_dir};

#[derive(Debug, Args)]
pub struct InstallArgs {
    /// Data directory to harden (created if absent)
    #[arg(long, default_value = "/var/lib/vectorvolt")]
    pub data_dir: PathBuf,

    /// Skip system user creation (useful when running as non-root in CI)
    #[arg(long, default_value_t = false)]
    pub skip_user: bool,

    /// Generate and print a new master key instead of requiring one
    #[arg(long, default_value_t = false)]
    pub gen_key: bool,

    /// Seal the master key into the TPM (requires Linux + TPM 2.0 hardware and --gen-key)
    #[arg(long, default_value_t = false)]
    pub tpm_seal: bool,
}

pub fn run(args: InstallArgs) -> Result<()> {
    println!("VectorVolt install — hardening host for data-at-rest security");
    println!();

    // ── Step 1: system user ──────────────────────────────────────────────────
    if !args.skip_user {
        match ensure_system_user() {
            Ok(true) => println!("  ✔ created system user 'vectorvolt'"),
            Ok(false) => println!("  · system user 'vectorvolt' already exists"),
            Err(e) => eprintln!("  ⚠ could not create system user: {e}  (re-run as root)"),
        }
    } else {
        println!("  · skipped system user creation (--skip-user)");
    }

    // ── Step 2: create + harden data dir ────────────────────────────────────
    if !args.data_dir.exists() {
        std::fs::create_dir_all(&args.data_dir)?;
        println!("  ✔ created {}", args.data_dir.display());
    }
    let report = harden_data_dir(&args.data_dir)?;
    println!(
        "  ✔ hardened {} dirs, {} files (chmod 700/600)",
        report.dirs_hardened, report.files_hardened
    );
    if report.chown_skipped {
        eprintln!("  ⚠ chown skipped — re-run as root to transfer ownership to 'vectorvolt'");
    }

    // ── Step 3: master key ───────────────────────────────────────────────────
    if args.gen_key {
        let key = KeyStore::generate()?;
        println!();
        println!("  ✔ generated master key (store in your secrets manager — shown once):");
        println!();
        println!("    VECTORVOLT_MASTER_KEY={}", key.to_hex());
        println!();

        if args.tpm_seal {
            let sealed_path = args.data_dir.join("tpm_sealed.bin");
            match vectorvolt_tpm::seal(key.to_hex().as_bytes(), &sealed_path) {
                Ok(()) => println!(
                    "  ✔ master key sealed into TPM at {}",
                    sealed_path.display()
                ),
                Err(e) => {
                    eprintln!("  ⚠ TPM seal failed: {e}  (is TPM 2.0 present and feature enabled?)")
                }
            }
        } else {
            println!("  Set this env var before starting vectorvolt-server.");
            println!("  The key is NEVER written to disk by VectorVolt.");
            println!("  tip: add --tpm-seal to bind the key to the TPM boot chain");
        }
    } else {
        println!();
        println!("  tip: run with --gen-key to generate a new master key");
    }

    println!();
    println!("  Installation complete.");
    Ok(())
}
