//! `vvctl config` — master VectorVolt configuration management (VV-CFG-012).
//!
//! ## Subcommands
//!
//! ```text
//! vvctl config init [--output vectorvolt.yaml] [--force]   # generate from host profile
//! vvctl config show  <file>                                 # parse and print config
//! vvctl config validate <file>                              # validate constraints
//! ```
//!
//! `init` probes the host via [`SystemProfile`] and writes a `VvConfig`
//! pre-populated with recommended settings.  All sections are included so
//! the operator can see and tweak every knob.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use clap::Subcommand;
use vectorvolt_engine::{ConfigDiffKind, SystemProfile, VvConfig};

#[derive(Subcommand)]
pub enum ConfigCommand {
    /// Generate a starter vectorvolt.yaml pre-filled from the host profile.
    ///
    /// Probes CPU, NUMA, SIMD tier, and huge-page availability, then writes
    /// recommended values for `threads`, `storage`, and `hal` sections.
    /// Existing file is refused unless `--force` is set.
    Init {
        /// Output path for the generated config file.
        #[arg(long, short, default_value = "vectorvolt.yaml")]
        output: PathBuf,
        /// Overwrite the file if it already exists.
        #[arg(long)]
        force: bool,
    },

    /// Parse and print a vectorvolt.yaml config file.
    Show {
        /// Path to the config file.
        file: PathBuf,
    },

    /// Validate a vectorvolt.yaml config file and report any constraint warnings.
    ///
    /// Exits 0 if the file parses cleanly and passes all constraint checks.
    /// Exits 1 if there are parse errors or warnings.
    Validate {
        /// Path to the config file.
        file: PathBuf,
    },

    /// Compare two vectorvolt.yaml config files field-by-field.
    ///
    /// Exits 0 if the files are identical; exits 1 if there are differences.
    /// Prints a human-readable diff table showing Added (+), Removed (-), and
    /// Changed (~) fields.
    Diff {
        /// Base config file.
        base: PathBuf,
        /// Other config file to compare against.
        other: PathBuf,
    },

    /// List all supported VECTORVOLT_* environment variables and their current values.
    ///
    /// Shows every env var that `vvctl` honours as a config override, the
    /// dot-notation key it maps to, a one-line description, and whether it is
    /// currently set in the process environment.
    EnvList,

    /// Merge two vectorvolt.yaml files (base + overlay) and emit the result.
    ///
    /// For each field: the overlay value wins when set; the base value is kept
    /// when the overlay field is absent.  The merged YAML is printed to stdout.
    /// Use `--output` to write it to a file instead, and `--show-diff` to
    /// print what changed.
    Merge {
        /// Base config file.
        base: PathBuf,
        /// Overlay config file (wins over base for every field it sets).
        overlay: PathBuf,
        /// Write merged config to this file instead of stdout.
        #[arg(long, short)]
        output: Option<PathBuf>,
        /// After merging, print a diff showing which fields changed.
        #[arg(long)]
        show_diff: bool,
    },

    /// Set a single field in a vectorvolt.yaml config file by dot-notation key.
    ///
    /// Loads the file, sets the key, validates, and writes back in-place.
    /// Use `--dry-run` to print the result without writing.
    SetKey {
        /// Path to the config file.
        file: PathBuf,
        /// Dot-notation key (e.g. `threads.writer_count`).
        key: String,
        /// New value (always a string; the engine parses to the target type).
        value: String,
        /// Print the result without writing the file.
        #[arg(long)]
        dry_run: bool,
    },
}

pub fn run(cmd: ConfigCommand) -> Result<()> {
    match cmd {
        ConfigCommand::Init { output, force } => cmd_init(&output, force),
        ConfigCommand::Show { file } => cmd_show(&file),
        ConfigCommand::Validate { file } => cmd_validate(&file),
        ConfigCommand::Diff { base, other } => cmd_diff(&base, &other),
        ConfigCommand::EnvList => cmd_env_list(),
        ConfigCommand::Merge {
            base,
            overlay,
            output,
            show_diff,
        } => cmd_merge(&base, &overlay, output.as_deref(), show_diff),
        ConfigCommand::SetKey {
            file,
            key,
            value,
            dry_run,
        } => cmd_set_key(&file, &key, &value, dry_run),
    }
}

fn cmd_init(output: &Path, force: bool) -> Result<()> {
    if output.exists() && !force {
        bail!(
            "config file '{}' already exists — use --force to overwrite",
            output.display()
        );
    }

    let profile = SystemProfile::probe();
    let cfg = config_from_profile(&profile);

    let yaml = cfg.to_yaml().context("serialize config")?;

    if let Some(parent) = output.parent()
        && !parent.as_os_str().is_empty()
    {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("create directory '{}'", parent.display()))?;
    }
    std::fs::write(output, &yaml)
        .with_context(|| format!("write config '{}'", output.display()))?;

    println!("Generated: {}", output.display());
    println!();
    println!("Host profile:");
    println!("  logical cores : {}", profile.logical_cores);
    println!("  physical cores: {}", profile.physical_cores);
    println!("  NUMA nodes    : {}", profile.numa_node_count);
    println!("  huge pages    : {}", profile.huge_pages_available);
    println!("  SIMD tier     : {:?}", profile.simd_tier);
    println!();
    println!("Applied defaults:");
    println!(
        "  threads.writer_count      = {}",
        profile.recommended_partition_count
    );
    println!(
        "  storage.l0_flush_threshold = {}",
        profile.recommended_flush_threshold
    );
    if profile.huge_pages_available {
        println!("  hal.huge_pages             = 2mb");
    }
    Ok(())
}

fn cmd_show(file: &Path) -> Result<()> {
    let cfg =
        VvConfig::from_file(file).with_context(|| format!("load config '{}'", file.display()))?;
    let yaml = cfg.to_yaml().context("serialize config")?;
    print!("{yaml}");
    Ok(())
}

fn cmd_validate(file: &Path) -> Result<()> {
    let cfg =
        VvConfig::from_file(file).with_context(|| format!("load config '{}'", file.display()))?;
    let warns = cfg.validate();
    if warns.is_empty() {
        println!("config ok: {}", file.display());
        Ok(())
    } else {
        for w in &warns {
            eprintln!("warning: {w}");
        }
        bail!("config has {} warning(s)", warns.len());
    }
}

fn cmd_diff(base_path: &Path, other_path: &Path) -> Result<()> {
    let base = VvConfig::from_file(base_path)
        .with_context(|| format!("load '{}'", base_path.display()))?;
    let other = VvConfig::from_file(other_path)
        .with_context(|| format!("load '{}'", other_path.display()))?;
    let diffs = base.diff(&other);
    if diffs.is_empty() {
        println!("configs are identical");
        return Ok(());
    }
    let key_width = diffs.iter().map(|d| d.key.len()).max().unwrap_or(0);
    for d in &diffs {
        let sigil = match d.kind {
            ConfigDiffKind::Added => '+',
            ConfigDiffKind::Removed => '-',
            ConfigDiffKind::Changed => '~',
        };
        let base_val = d.base.as_deref().unwrap_or("(unset)");
        let other_val = d.other.as_deref().unwrap_or("(unset)");
        println!("{sigil} {:<key_width$}  {base_val} -> {other_val}", d.key);
    }
    bail!("{} difference(s) found", diffs.len());
}

fn cmd_env_list() -> Result<()> {
    let table = VvConfig::env_var_table();
    let env_col = table.iter().map(|(e, _, _)| e.len()).max().unwrap_or(0);
    let key_col = table.iter().map(|(_, k, _)| k.len()).max().unwrap_or(0);
    println!("{:<env_col$}  {:<key_col$}  VALUE", "ENV VAR", "CONFIG KEY");
    println!("{}", "-".repeat(env_col + key_col + 10));
    for (env_key, dot_key, _desc) in table {
        let val = std::env::var(env_key).unwrap_or_else(|_| "(unset)".into());
        println!("{:<env_col$}  {:<key_col$}  {val}", env_key, dot_key);
    }
    Ok(())
}

fn cmd_merge(
    base_path: &Path,
    overlay_path: &Path,
    output: Option<&Path>,
    show_diff: bool,
) -> Result<()> {
    let base = VvConfig::from_file(base_path)
        .with_context(|| format!("load base '{}'", base_path.display()))?;
    let overlay = VvConfig::from_file(overlay_path)
        .with_context(|| format!("load overlay '{}'", overlay_path.display()))?;
    let merged = base.merge_with(&overlay);
    let yaml = merged.to_yaml().context("serialize merged config")?;

    match output {
        Some(path) => {
            if let Some(parent) = path.parent()
                && !parent.as_os_str().is_empty()
            {
                std::fs::create_dir_all(parent)
                    .with_context(|| format!("create directory '{}'", parent.display()))?;
            }
            std::fs::write(path, &yaml)
                .with_context(|| format!("write merged config to '{}'", path.display()))?;
            println!("Merged config written to: {}", path.display());
        }
        None => print!("{yaml}"),
    }

    if show_diff {
        let diffs = base.diff(&merged);
        if diffs.is_empty() {
            println!("# no fields changed (overlay was empty)");
        } else {
            let key_width = diffs.iter().map(|d| d.key.len()).max().unwrap_or(0);
            for d in &diffs {
                let sigil = match d.kind {
                    ConfigDiffKind::Added => '+',
                    ConfigDiffKind::Removed => '-',
                    ConfigDiffKind::Changed => '~',
                };
                let base_val = d.base.as_deref().unwrap_or("(unset)");
                let merged_val = d.other.as_deref().unwrap_or("(unset)");
                println!("{sigil} {:<key_width$}  {base_val} -> {merged_val}", d.key);
            }
        }
    }
    Ok(())
}

fn cmd_set_key(file: &Path, key: &str, value: &str, dry_run: bool) -> Result<()> {
    let mut cfg =
        VvConfig::from_file(file).with_context(|| format!("load '{}'", file.display()))?;
    cfg.set_key(key, value)
        .map_err(|e| anyhow::anyhow!("{e}"))?;
    let yaml = cfg.to_yaml().context("serialize config")?;
    if dry_run {
        print!("{yaml}");
    } else {
        std::fs::write(file, &yaml).with_context(|| format!("write '{}'", file.display()))?;
        println!("Updated: {} = {}", key, value);
    }
    Ok(())
}

/// Build a [`VvConfig`] pre-populated from a [`SystemProfile`].
pub fn config_from_profile(profile: &SystemProfile) -> VvConfig {
    let mut cfg = VvConfig::default();
    cfg.threads.writer_count = Some(profile.recommended_partition_count as u32);
    cfg.storage.l0_flush_threshold = Some(profile.recommended_flush_threshold as u32);
    cfg.storage.compaction_strategy = Some("leveled".into());
    cfg.storage.block_size = Some(65536);
    cfg.observability.trace_level = Some("info".into());
    cfg.network.bind_addr = Some("127.0.0.1:5995".into());
    cfg.network.max_connections = Some(256);
    cfg.query.index_hint = Some("auto".into());
    if profile.huge_pages_available {
        cfg.hal.huge_pages = Some("2mb".into());
    }
    cfg
}
