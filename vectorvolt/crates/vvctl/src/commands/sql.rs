//! `vvctl sql` — interactive SQL REPL and one-shot exec (VV-QRY-020).

use std::io::{BufRead, Write};
use std::path::PathBuf;
use std::sync::Arc;

use anyhow::Result;
use clap::Args;
use vectorvolt_engine::{EngineOptions, TableRegistry};
use vectorvolt_query::sql::executor::{QueryResult, SqlExecutor};

#[derive(Args, Debug)]
pub struct SqlArgs {
    /// Data directory that holds VectorVolt tables.
    #[arg(short, long, default_value = ".")]
    pub dir: PathBuf,

    /// Execute a single SQL statement then exit (non-interactive).
    #[arg(short = 'e', long)]
    pub exec: Option<String>,

    /// Output format: `table` (default) or `csv`.
    #[arg(short, long, default_value = "table")]
    pub output: String,
}

pub fn run(args: SqlArgs) -> Result<()> {
    let registry = TableRegistry::open(&args.dir, EngineOptions::default())
        .map_err(|e| anyhow::anyhow!("open registry: {e}"))?;
    let executor = SqlExecutor::new(Arc::new(registry));
    let csv_mode = args.output.to_lowercase() == "csv";

    if let Some(sql) = args.exec {
        exec_and_print(&executor, sql.trim(), csv_mode);
        return Ok(());
    }

    // Interactive REPL
    let stdin = std::io::stdin();
    let mut buf = String::new();
    print!("> ");
    std::io::stdout().flush()?;
    for line in stdin.lock().lines() {
        let line = line?;
        let trimmed = line.trim();
        if trimmed.eq_ignore_ascii_case("quit") || trimmed.eq_ignore_ascii_case("exit") {
            break;
        }
        if trimmed.is_empty() {
            print!("> ");
            std::io::stdout().flush()?;
            continue;
        }
        buf.push_str(trimmed);
        if trimmed.ends_with(';') {
            exec_and_print(&executor, buf.trim_end_matches(';').trim(), csv_mode);
            buf.clear();
            print!("> ");
        } else {
            buf.push(' ');
            print!("... ");
        }
        std::io::stdout().flush()?;
    }
    Ok(())
}

fn exec_and_print(executor: &SqlExecutor, sql: &str, csv: bool) {
    match executor.execute(sql) {
        Ok(QueryResult::Rows { columns, rows }) => {
            if csv {
                println!("{}", columns.join(","));
                for row in &rows {
                    println!("{}", row.join(","));
                }
            } else {
                print_table(&columns, &rows);
            }
        }
        Ok(QueryResult::Affected(n)) => println!("OK  {n} row(s) affected"),
        Ok(QueryResult::Created) => println!("OK  table created"),
        Ok(QueryResult::Dropped) => println!("OK  table dropped"),
        Ok(QueryResult::TxBegin) => println!("BEGIN"),
        Ok(QueryResult::TxCommit) => println!("COMMIT"),
        Ok(QueryResult::TxRollback) => println!("ROLLBACK"),
        Ok(QueryResult::ExplainPlan(lines)) => {
            for line in &lines {
                println!("{line}");
            }
        }
        Err(e) => eprintln!("ERROR  {e}"),
    }
}

fn print_table(columns: &[String], rows: &[Vec<String>]) {
    // Column widths
    let mut widths: Vec<usize> = columns.iter().map(|c| c.len()).collect();
    for row in rows {
        for (i, cell) in row.iter().enumerate() {
            if i < widths.len() {
                widths[i] = widths[i].max(cell.len());
            }
        }
    }

    let sep: String = widths
        .iter()
        .map(|&w| "-".repeat(w + 2))
        .collect::<Vec<_>>()
        .join("+");
    println!("+{sep}+");

    let header: String = columns
        .iter()
        .zip(&widths)
        .map(|(c, &w)| format!(" {c:<w$} "))
        .collect::<Vec<_>>()
        .join("|");
    println!("|{header}|");
    println!("+{sep}+");

    for row in rows {
        let cells: String = row
            .iter()
            .zip(&widths)
            .map(|(c, &w)| format!(" {c:<w$} "))
            .collect::<Vec<_>>()
            .join("|");
        println!("|{cells}|");
    }
    println!("+{sep}+");
    println!("{} row(s)", rows.len());
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn make_executor(dir: &std::path::Path) -> SqlExecutor {
        let r = TableRegistry::open(dir, EngineOptions::default()).unwrap();
        SqlExecutor::new(Arc::new(r))
    }

    #[test]
    fn test_create_and_select() {
        let d = TempDir::new().unwrap();
        let e = make_executor(d.path());
        exec_and_print(&e, "CREATE TABLE t (id u64, val u64)", false);
        exec_and_print(&e, "INSERT INTO t (id, val) VALUES (1, 100)", false);
        exec_and_print(&e, "SELECT * FROM t", false);
    }

    #[test]
    fn test_csv_output() {
        let d = TempDir::new().unwrap();
        let e = make_executor(d.path());
        exec_and_print(&e, "CREATE TABLE t (id u64)", false);
        exec_and_print(&e, "INSERT INTO t (id) VALUES (1)", false);
        exec_and_print(&e, "SELECT * FROM t", true);
    }

    #[test]
    fn test_error_output() {
        let d = TempDir::new().unwrap();
        let e = make_executor(d.path());
        exec_and_print(&e, "SELECT * FROM nonexistent", false);
    }
}
