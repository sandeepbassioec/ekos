//! `vvctl query` — ad-hoc SQL queries against a local table directory (S24-B).

use std::path::PathBuf;
use std::sync::Arc;

use anyhow::{Context, Result};
use clap::Args;
use vectorvolt_engine::{EngineOptions, TableRegistry};
use vectorvolt_query::sql::executor::{QueryResult, SqlExecutor};

#[derive(Debug, Args)]
pub struct QueryArgs {
    /// SQL statement to execute (SELECT, INSERT, DELETE, CREATE TABLE, DROP TABLE, EXPLAIN SELECT).
    #[arg(long, short)]
    pub sql: String,

    /// Table data directory (default: current directory).
    #[arg(long, short, default_value = ".")]
    pub dir: PathBuf,
}

pub fn run(args: QueryArgs) -> Result<()> {
    let registry = Arc::new(
        TableRegistry::open(&args.dir, EngineOptions::default())
            .with_context(|| format!("opening registry at {}", args.dir.display()))?,
    );
    let executor = SqlExecutor::new(registry);
    let result = executor
        .execute(&args.sql)
        .with_context(|| format!("executing: {}", args.sql))?;

    match result {
        QueryResult::Rows { columns, rows } => {
            if rows.is_empty() {
                println!("(0 rows)");
                return Ok(());
            }
            // Column widths
            let widths: Vec<usize> = columns
                .iter()
                .enumerate()
                .map(|(i, h)| {
                    rows.iter()
                        .map(|r| r.get(i).map(|s| s.len()).unwrap_or(0))
                        .max()
                        .unwrap_or(0)
                        .max(h.len())
                })
                .collect();
            let header: Vec<String> = columns
                .iter()
                .enumerate()
                .map(|(i, h)| format!("{:width$}", h, width = widths[i]))
                .collect();
            println!("{}", header.join(" | "));
            println!(
                "{}",
                widths
                    .iter()
                    .map(|w| "-".repeat(*w))
                    .collect::<Vec<_>>()
                    .join("-+-")
            );
            for row in &rows {
                let cells: Vec<String> = row
                    .iter()
                    .enumerate()
                    .map(|(i, v)| format!("{:width$}", v, width = widths[i]))
                    .collect();
                println!("{}", cells.join(" | "));
            }
            println!(
                "({} row{})",
                rows.len(),
                if rows.len() == 1 { "" } else { "s" }
            );
        }
        QueryResult::Affected(n) => println!("Affected {n} row(s)."),
        QueryResult::Created => println!("Table created."),
        QueryResult::Dropped => println!("Table dropped."),
        QueryResult::TxBegin => println!("BEGIN"),
        QueryResult::TxCommit => println!("COMMIT"),
        QueryResult::TxRollback => println!("ROLLBACK"),
        QueryResult::ExplainPlan(lines) => {
            println!("Query Plan");
            println!("----------");
            for (i, line) in lines.iter().enumerate() {
                let prefix = if i == 0 { "→" } else { "  └─" };
                println!("{prefix} {line}");
            }
        }
        QueryResult::Error(msg) => eprintln!("ERROR  {msg}"),
        QueryResult::Notice(msg) => println!("NOTICE {msg}"),
        QueryResult::Ok(msg) => println!("{msg}"),
        QueryResult::WithNotice { result, notice } => {
            println!("NOTICE {notice}");
            match *result {
                QueryResult::Rows { columns, rows } => {
                    println!("{}", columns.join(" | "));
                    for row in &rows {
                        println!("{}", row.join(" | "));
                    }
                    println!("({} row{})", rows.len(), if rows.len() == 1 { "" } else { "s" });
                }
                QueryResult::Affected(n) => println!("Affected {n} row(s)."),
                other => println!("{other:?}"),
            }
        }
    }
    Ok(())
}
