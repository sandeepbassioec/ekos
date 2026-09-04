use anyhow::Result;
use clap::Subcommand;
use std::path::PathBuf;
use vectorvolt_graph::{
    analytics::{clustering_coefficient, degree_centrality, pagerank},
    cascade::CascadeDelete,
    store::GraphStore,
    traversal::{bfs, dfs, shortest_path},
};

#[derive(Subcommand)]
pub enum GraphCommand {
    /// Add a directed edge to the graph store.
    AddEdge {
        /// Source node ID.
        src: u64,
        /// Destination node ID.
        dst: u64,
        /// Graph store directory.
        #[arg(short, long, default_value = ".")]
        dir: PathBuf,
    },

    /// Remove a directed edge from the graph store.
    RemoveEdge {
        /// Source node ID.
        src: u64,
        /// Destination node ID.
        dst: u64,
        /// Graph store directory.
        #[arg(short, long, default_value = ".")]
        dir: PathBuf,
    },

    /// Remove a node and all its cascade-dependent nodes.
    RemoveNode {
        /// Node ID to remove.
        node: u64,
        /// Graph store directory.
        #[arg(short, long, default_value = ".")]
        dir: PathBuf,
    },

    /// BFS traversal from a starting node.
    Bfs {
        /// Starting node ID.
        start: u64,
        /// Maximum depth to traverse.
        #[arg(short, long, default_value = "10")]
        max_depth: usize,
        /// Graph store directory.
        #[arg(short, long, default_value = ".")]
        dir: PathBuf,
    },

    /// DFS traversal from a starting node.
    Dfs {
        /// Starting node ID.
        start: u64,
        /// Maximum depth to traverse.
        #[arg(short, long, default_value = "10")]
        max_depth: usize,
        /// Graph store directory.
        #[arg(short, long, default_value = ".")]
        dir: PathBuf,
    },

    /// Shortest path between two nodes.
    ShortestPath {
        /// Source node ID.
        src: u64,
        /// Destination node ID.
        dst: u64,
        /// Graph store directory.
        #[arg(short, long, default_value = ".")]
        dir: PathBuf,
    },

    /// Show cascade-affected nodes before deleting.
    Cascade {
        /// Node ID to check cascade for.
        node: u64,
        /// Graph store directory.
        #[arg(short, long, default_value = ".")]
        dir: PathBuf,
    },

    /// Run PageRank and show top nodes.
    Pagerank {
        /// Damping factor (default 0.85).
        #[arg(long, default_value = "0.85")]
        damping: f64,
        /// Number of iterations.
        #[arg(long, default_value = "20")]
        iterations: usize,
        /// Top-N nodes to display.
        #[arg(short, long, default_value = "10")]
        top: usize,
        /// Graph store directory.
        #[arg(short, long, default_value = ".")]
        dir: PathBuf,
    },

    /// Show graph statistics.
    Stats {
        /// Graph store directory.
        #[arg(default_value = ".")]
        dir: PathBuf,
    },

    /// Save the in-memory graph to disk.
    Save {
        /// Graph store directory.
        #[arg(short, long, default_value = ".")]
        dir: PathBuf,
    },
}

pub fn run(cmd: GraphCommand) -> Result<()> {
    match cmd {
        GraphCommand::AddEdge { src, dst, dir } => {
            let store = GraphStore::open(&dir)?;
            store.add_edge(src, dst)?;
            store.save()?;
            println!("edge {src} -> {dst} added");
        }

        GraphCommand::RemoveEdge { src, dst, dir } => {
            let store = GraphStore::open(&dir)?;
            store.remove_edge(src, dst)?;
            store.save()?;
            println!("edge {src} -> {dst} removed");
        }

        GraphCommand::RemoveNode { node, dir } => {
            let store = GraphStore::open(&dir)?;
            let deleted = store.remove_node(node)?;
            store.save()?;
            println!("deleted {} node(s): {:?}", deleted.len(), deleted);
        }

        GraphCommand::Bfs {
            start,
            max_depth,
            dir,
        } => {
            let store = GraphStore::open(&dir)?;
            let snap = store.snapshot();
            let result = bfs(snap.forward(), start, max_depth);
            println!("BFS from {start} (max_depth={max_depth}):");
            for (node, depth) in result {
                println!("  depth={depth}  node={node}");
            }
        }

        GraphCommand::Dfs {
            start,
            max_depth,
            dir,
        } => {
            let store = GraphStore::open(&dir)?;
            let snap = store.snapshot();
            let result = dfs(snap.forward(), start, max_depth);
            println!("DFS from {start} (max_depth={max_depth}):");
            for (node, depth) in result {
                println!("  depth={depth}  node={node}");
            }
        }

        GraphCommand::ShortestPath { src, dst, dir } => {
            let store = GraphStore::open(&dir)?;
            let snap = store.snapshot();
            match shortest_path(snap.forward(), src, dst) {
                Some(path) => println!("path ({} hops): {:?}", path.len() - 1, path),
                None => println!("no path from {src} to {dst}"),
            }
        }

        GraphCommand::Cascade { node, dir } => {
            let store = GraphStore::open(&dir)?;
            let snap = store.snapshot();
            let cascade = CascadeDelete::new(snap);
            let affected = cascade.affected_nodes(node);
            let order = cascade.deletion_order(node);
            println!("cascade delete of node {node}:");
            println!("  affected: {:?}", affected);
            println!("  deletion order: {:?}", order);
        }

        GraphCommand::Pagerank {
            damping,
            iterations,
            top,
            dir,
        } => {
            let store = GraphStore::open(&dir)?;
            let snap = store.snapshot();
            let ranks = pagerank(snap.forward(), damping, iterations);
            let centrality = degree_centrality(snap.forward());
            let mut sorted: Vec<(u64, f64)> = ranks.into_iter().collect();
            sorted.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
            println!("PageRank (damping={damping}, iters={iterations}) — top {top}:");
            for (node, rank) in sorted.into_iter().take(top) {
                let dc = centrality.get(&node).copied().unwrap_or(0.0);
                let cc = clustering_coefficient(snap.forward(), node);
                println!(
                    "  node={node}  rank={rank:.6}  deg_centrality={dc:.4}  clustering={cc:.4}"
                );
            }
        }

        GraphCommand::Stats { dir } => {
            let store = GraphStore::open(&dir)?;
            let stats = store.stats();
            println!("nodes:      {}", stats.node_count);
            println!("edges:      {}", stats.edge_count);
            println!("file_bytes: {}", stats.file_size_bytes);
        }

        GraphCommand::Save { dir } => {
            let store = GraphStore::open(&dir)?;
            store.save()?;
            println!("graph saved to {}", dir.display());
        }
    }
    Ok(())
}
