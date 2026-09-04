//! `vvctl chaos` — DST simulation and replay commands.

use anyhow::Result;
use clap::Subcommand;
use serde_json::json;
use vectorvolt_chaos::{DstEventKind, DstHarness, Fault};

/// Chaos fault variant names accepted on the CLI.
#[derive(clap::ValueEnum, Clone, Debug)]
pub enum FaultArg {
    Delay,
    Loss,
    Corrupt,
    Crash,
}

#[derive(Subcommand)]
pub enum ChaosCommand {
    /// Run a deterministic simulation and print the event log as JSON.
    Simulate {
        /// RNG seed for the harness.
        #[arg(long, default_value = "42")]
        seed: u64,
        /// Number of ticks to simulate.
        #[arg(long, default_value = "10")]
        ticks: u64,
        /// Fault to inject (optional).
        #[arg(long)]
        fault: Option<FaultArg>,
    },

    /// Replay a previously captured DST event log.
    Replay {
        /// RNG seed used in the original run.
        #[arg(long, default_value = "42")]
        seed: u64,
        /// JSON array of event ticks (e.g. `[1,3,5]`).
        #[arg(long, default_value = "[]")]
        events: String,
    },
}

pub fn run(cmd: ChaosCommand) -> Result<()> {
    match cmd {
        ChaosCommand::Simulate { seed, ticks, fault } => {
            let mut harness = DstHarness::new(seed);

            // Optionally inject a fault into the injector.
            if let Some(f) = &fault {
                let fault_spec = match f {
                    FaultArg::Delay => Fault::Delay { min_ms: 1, max_ms: 10 },
                    FaultArg::Loss => Fault::PacketLoss { probability: 0.5 },
                    FaultArg::Corrupt => Fault::Corrupt { probability: 0.1 },
                    FaultArg::Crash => Fault::Crash { after_n_ops: ticks.saturating_sub(1).max(1) },
                };
                harness.injector.add_fault(fault_spec);
            }

            // Schedule one Tick event per tick.
            for t in 1..=ticks {
                harness.schedule(t, DstEventKind::Tick);
            }

            harness.run_until(ticks);

            let log: Vec<serde_json::Value> = harness
                .replay()
                .map(|e| json!({ "tick": e.at_tick, "kind": "Tick" }))
                .collect();

            println!("{}", serde_json::to_string_pretty(&log)?);
        }

        ChaosCommand::Replay { seed, events } => {
            let ticks: Vec<u64> = serde_json::from_str(&events)?;
            let mut harness = DstHarness::new(seed);
            for t in &ticks {
                harness.schedule(*t, DstEventKind::Tick);
            }
            let max_tick = ticks.iter().copied().max().unwrap_or(0);
            harness.run_until(max_tick);

            let log: Vec<serde_json::Value> = harness
                .replay()
                .map(|e| json!({ "tick": e.at_tick, "kind": "Tick" }))
                .collect();

            println!("{}", serde_json::to_string_pretty(&log)?);
        }
    }
    Ok(())
}
