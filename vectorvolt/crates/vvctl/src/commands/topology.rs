//! `vvctl topology` — show detected CPU/NUMA topology (VV-CTL-180).

use anyhow::Result;
use clap::Args;
use vectorvolt_hal::CpuTopology;

#[derive(Args)]
pub struct TopologyArgs {
    /// Show core list with IDs and classes.
    #[arg(long)]
    pub cores: bool,
    /// Show NUMA node info.
    #[arg(long)]
    pub numa: bool,
}

pub fn run(args: TopologyArgs) -> Result<()> {
    let topo = CpuTopology::detect();
    println!("logical_cores:  {}", topo.logical.len());
    println!("numa_nodes:     {}", topo.numa_nodes.len());
    println!("hybrid_cpu:     {}", topo.hybrid);
    println!("degraded:       {}", topo.degraded);
    if args.cores {
        for lc in &topo.logical {
            println!(
                "  core {} physical={} socket={} numa={} class={:?}",
                lc.id, lc.physical_id, lc.socket_id, lc.numa_node, lc.class
            );
        }
    }
    if args.numa {
        for nn in &topo.numa_nodes {
            let mem = nn
                .memory_bytes
                .map(|b| format!("{} bytes", b))
                .unwrap_or_else(|| "unknown".into());
            println!("  numa {} cpus={} memory={}", nn.id, nn.cpus.len(), mem);
        }
    }
    Ok(())
}
