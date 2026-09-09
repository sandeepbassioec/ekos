//! `vvctl events` subcommand — schema discovery + DLQ management (VV-EVT-013).

use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::{Args, Subcommand};
use vectorvolt_events::registry::SchemaVersionRegistry;

#[derive(Debug, Args)]
pub struct EventsArgs {
    #[command(subcommand)]
    pub command: EventsCommand,
}

#[derive(Debug, Subcommand)]
pub enum EventsCommand {
    /// List all domains and their events (optionally filter by domain).
    List {
        /// Show only events under this domain/entity.
        #[arg(long)]
        domain: Option<String>,
        /// Path to the event_schemas directory (default: ./event_schemas).
        #[arg(long, default_value = "event_schemas")]
        registry: PathBuf,
    },
    /// Show a specific schema version.
    Show {
        /// Entity/domain name (e.g. orders).
        domain: String,
        /// Event name (e.g. OrderPlaced).
        event: String,
        /// Schema version (e.g. 1.2.0).
        version: String,
        #[arg(long, default_value = "event_schemas")]
        registry: PathBuf,
    },
    /// Diff two versions of a schema.
    Diff {
        domain: String,
        event: String,
        v1: String,
        v2: String,
        #[arg(long, default_value = "event_schemas")]
        registry: PathBuf,
    },
}

pub fn run(args: EventsArgs) -> Result<()> {
    match args.command {
        EventsCommand::List { domain, registry } => cmd_list(&registry, domain.as_deref()),
        EventsCommand::Show {
            domain,
            event,
            version,
            registry,
        } => cmd_show(&registry, &domain, &event, &version),
        EventsCommand::Diff {
            domain,
            event,
            v1,
            v2,
            registry,
        } => cmd_diff(&registry, &domain, &event, &v1, &v2),
    }
}

fn cmd_list(registry: &PathBuf, filter_domain: Option<&str>) -> Result<()> {
    let reg = SchemaVersionRegistry::new(registry)
        .with_context(|| format!("opening registry at {}", registry.display()))?;

    let domains = reg.list_domains()?;
    if domains.is_empty() {
        println!("(no schemas registered)");
        return Ok(());
    }

    for domain in &domains {
        if filter_domain.is_some_and(|f| domain != f) {
            continue;
        }
        println!("[{domain}]");
        let events = reg.list_events(domain)?;
        for event in &events {
            let versions = reg.list_versions(domain, event)?;
            let latest = versions.last().map(String::as_str).unwrap_or("—");
            println!(
                "  {event}  (latest: {latest}, {} version(s))",
                versions.len()
            );
        }
    }
    Ok(())
}

fn cmd_show(registry: &PathBuf, domain: &str, event: &str, version: &str) -> Result<()> {
    let reg = SchemaVersionRegistry::new(registry)?;
    let schema = reg.show(domain, event, version)?;
    println!("{}", serde_json::to_string_pretty(&schema)?);
    Ok(())
}

fn cmd_diff(registry: &PathBuf, domain: &str, event: &str, v1: &str, v2: &str) -> Result<()> {
    let reg = SchemaVersionRegistry::new(registry)?;
    let diffs = reg.diff(domain, event, v1, v2)?;
    if diffs.is_empty() {
        println!("No differences between {v1} and {v2}.");
    } else {
        for line in &diffs {
            println!("{line}");
        }
    }
    Ok(())
}
