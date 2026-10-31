//! `vvctl vector` — insert, search, and get vectors (VV-IDX-012 / VV-HAL-016).

use std::path::PathBuf;
use std::sync::Arc;

use anyhow::{Context, Result};
use clap::{Args, Subcommand};
use vectorvolt_engine::{EngineOptions, TableRegistry};

#[derive(Args, Debug)]
pub struct VectorArgs {
    #[command(subcommand)]
    pub command: VectorCommand,
}

#[derive(Subcommand, Debug)]
pub enum VectorCommand {
    /// Insert a vector for a given row PK.
    Insert(VectorInsertArgs),
    /// Search for the k nearest neighbours to a query vector.
    Search(VectorSearchArgs),
    /// Retrieve the vector stored for a given row PK.
    Get(VectorGetArgs),
}

#[derive(Args, Debug)]
pub struct VectorInsertArgs {
    /// Table name.
    pub table: String,
    /// Column index (0-based) that holds the vector.
    #[arg(long, default_value = "1")]
    pub col: usize,
    /// Primary key of the row this vector belongs to.
    #[arg(long)]
    pub pk: u64,
    /// Comma-separated float values, e.g. "0.1,0.2,0.3".
    #[arg(long)]
    pub vec: String,
    /// Data directory.
    #[arg(short, long, default_value = ".")]
    pub dir: PathBuf,
}

#[derive(Args, Debug)]
pub struct VectorSearchArgs {
    /// Table name.
    pub table: String,
    /// Column index to search.
    #[arg(long, default_value = "1")]
    pub col: usize,
    /// Query vector as comma-separated floats.
    #[arg(long)]
    pub vec: String,
    /// Number of neighbours to return.
    #[arg(short, long, default_value = "5")]
    pub k: usize,
    /// Data directory.
    #[arg(short, long, default_value = ".")]
    pub dir: PathBuf,
}

#[derive(Args, Debug)]
pub struct VectorGetArgs {
    /// Table name.
    pub table: String,
    /// Column index.
    #[arg(long, default_value = "1")]
    pub col: usize,
    /// Primary key.
    #[arg(long)]
    pub pk: u64,
    /// Data directory.
    #[arg(short, long, default_value = ".")]
    pub dir: PathBuf,
}

fn parse_vec(s: &str) -> Result<Vec<f32>> {
    s.split(',')
        .map(|t| t.trim().parse::<f32>().context("invalid float in vector"))
        .collect()
}

pub fn run(args: VectorArgs) -> Result<()> {
    match args.command {
        VectorCommand::Insert(a) => {
            let registry = TableRegistry::open(&a.dir, EngineOptions::default())
                .map_err(|e| anyhow::anyhow!("open registry: {e}"))?;
            let registry = Arc::new(registry);
            let engine = registry
                .get(&a.table)
                .map_err(|e| anyhow::anyhow!("table '{}': {e}", a.table))?;
            let vector = parse_vec(&a.vec)?;
            engine
                .insert_vector(a.col, a.pk, vector)
                .map_err(|e| anyhow::anyhow!("insert vector: {e}"))?;
            println!("OK  vector inserted for pk={}", a.pk);
        }
        VectorCommand::Search(a) => {
            let registry = TableRegistry::open(&a.dir, EngineOptions::default())
                .map_err(|e| anyhow::anyhow!("open registry: {e}"))?;
            let registry = Arc::new(registry);
            let engine = registry
                .get(&a.table)
                .map_err(|e| anyhow::anyhow!("table '{}': {e}", a.table))?;
            let query = parse_vec(&a.vec)?;
            let results = engine
                .search_vector(a.col, &query, a.k)
                .map_err(|e| anyhow::anyhow!("search: {e}"))?;
            if results.is_empty() {
                println!("(no results)");
            } else {
                println!("{:<10}  distance", "pk");
                println!("{}", "-".repeat(24));
                for (pk, dist) in &results {
                    println!("{pk:<10}  {dist:.6}");
                }
            }
        }
        VectorCommand::Get(a) => {
            let registry = TableRegistry::open(&a.dir, EngineOptions::default())
                .map_err(|e| anyhow::anyhow!("open registry: {e}"))?;
            let registry = Arc::new(registry);
            let engine = registry
                .get(&a.table)
                .map_err(|e| anyhow::anyhow!("table '{}': {e}", a.table))?;
            match engine.get_vector(a.col, a.pk) {
                Some(v) => {
                    let s: Vec<String> = v.iter().map(|f| format!("{f}")).collect();
                    println!("[{}]", s.join(", "));
                }
                None => println!("(not found)"),
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use vectorvolt_engine::TableSchema;
    use vectorvolt_engine::schema::{ColumnDef, ColumnType};

    fn make_table(dir: &std::path::Path, name: &str) -> Arc<TableRegistry> {
        let r = TableRegistry::open(dir, EngineOptions::default()).unwrap();
        let schema = TableSchema::new(vec![
            ColumnDef::new("id", ColumnType::U64),
            ColumnDef::new("emb", ColumnType::Vector(3)),
        ])
        .unwrap();
        r.create_table_typed(name, schema).unwrap();
        Arc::new(r)
    }

    #[test]
    fn test_insert_and_get() {
        let d = TempDir::new().unwrap();
        let r = make_table(d.path(), "items");
        let engine = r.get("items").unwrap();
        engine
            .insert_vector(1, 42, vec![0.1_f32, 0.2, 0.3])
            .unwrap();
        let got = engine.get_vector(1, 42).unwrap();
        assert_eq!(got, vec![0.1_f32, 0.2, 0.3]);
    }

    #[test]
    fn test_search_returns_nearest() {
        let d = TempDir::new().unwrap();
        let r = make_table(d.path(), "items");
        let engine = r.get("items").unwrap();
        engine.insert_vector(1, 1, vec![1.0_f32, 0.0, 0.0]).unwrap();
        engine.insert_vector(1, 2, vec![0.0_f32, 1.0, 0.0]).unwrap();
        engine.insert_vector(1, 3, vec![0.0_f32, 0.0, 1.0]).unwrap();
        let results = engine.search_vector(1, &[1.0_f32, 0.0, 0.0], 1).unwrap();
        assert_eq!(results[0].0, 1);
    }

    #[test]
    fn test_get_missing() {
        let d = TempDir::new().unwrap();
        let r = make_table(d.path(), "items");
        let engine = r.get("items").unwrap();
        assert!(engine.get_vector(1, 999).is_none());
    }

    #[test]
    fn test_parse_vec_ok() {
        let v = parse_vec("1.0,2.5,3.14").unwrap();
        assert_eq!(v.len(), 3);
        assert!((v[0] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_parse_vec_err() {
        assert!(parse_vec("a,b,c").is_err());
    }
}
