//! Catalog watch — poll-based change detection (VV-CTL-011).
//!
//! [`watch_catalog`] exports the live catalog at a configurable interval and
//! emits a [`WatchEvent`] whenever the schema set diverges from the last
//! observed state.  Polling is used deliberately (no OS file-watcher
//! dependency) so the same code path runs on Linux, macOS, and Windows.
//!
//! The function accepts a callback that receives each event.  It blocks until
//! the callback returns [`WatchControl::Stop`] or the optional `max_iterations`
//! limit is reached — the latter is used exclusively in tests to avoid infinite
//! loops.

use std::path::Path;
use std::thread;
use std::time::Duration;

use crate::catalog_diff::{CatalogDiff, diff_catalogs};
use crate::catalog_export::{CatalogExport, export_catalog};
use crate::error::EngineResult;

// ── Public types ──────────────────────────────────────────────────────────────

/// A single event emitted by [`watch_catalog`].
#[derive(Debug)]
pub enum WatchEvent {
    /// Baseline snapshot taken on the first poll.
    Initial { export: CatalogExport },
    /// The catalog changed since the last poll.
    Changed {
        diff: CatalogDiff,
        current: CatalogExport,
    },
    /// The catalog is identical to the previous snapshot.
    Unchanged,
}

/// Returned by the user callback to control the watch loop.
#[derive(Debug, PartialEq)]
pub enum WatchControl {
    /// Continue polling.
    Continue,
    /// Stop the watch loop cleanly.
    Stop,
}

// ── watch ─────────────────────────────────────────────────────────────────────

/// Poll the catalog at `interval` and call `on_event` for every tick.
///
/// - `registry_dir`: directory containing `.schema` files.
/// - `interval`: time between polls.
/// - `max_iterations`: when `Some(n)`, stop after `n` calls to `on_event`
///   regardless of what the callback returns (test helper).
/// - `on_event`: called with each [`WatchEvent`]; returning [`WatchControl::Stop`]
///   ends the loop.
pub fn watch_catalog<F>(
    registry_dir: impl AsRef<Path>,
    interval: Duration,
    max_iterations: Option<usize>,
    mut on_event: F,
) -> EngineResult<()>
where
    F: FnMut(WatchEvent) -> WatchControl,
{
    let registry_dir = registry_dir.as_ref();
    let mut prev: Option<CatalogExport> = None;
    let mut iterations = 0usize;

    loop {
        let current = export_catalog(registry_dir)?;

        let event = match &prev {
            None => WatchEvent::Initial {
                export: current.clone(),
            },
            Some(p) => {
                let diff = diff_catalogs(p, &current);
                if diff.is_empty() {
                    WatchEvent::Unchanged
                } else {
                    WatchEvent::Changed {
                        diff,
                        current: current.clone(),
                    }
                }
            }
        };

        prev = Some(current);
        iterations += 1;

        let ctrl = on_event(event);
        if ctrl == WatchControl::Stop {
            break;
        }
        if max_iterations.is_some_and(|max| iterations >= max) {
            break;
        }

        thread::sleep(interval);
    }

    Ok(())
}

// ── Unit tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use tempfile::TempDir;

    use super::*;
    use crate::schema::{ColumnDef, TableSchema};
    use crate::schema_registry::SchemaRegistry;

    fn reg(dir: &TempDir) -> std::path::PathBuf {
        dir.path().join("registry")
    }

    fn add_table(reg_path: &std::path::Path, name: &str) {
        let registry = SchemaRegistry::new(reg_path);
        let schema = TableSchema::new(vec![ColumnDef::u64("pk"), ColumnDef::u64("val")]).unwrap();
        registry.save(name, &schema).unwrap();
    }

    #[test]
    fn first_event_is_initial() {
        let dir = TempDir::new().unwrap();
        let reg_path = reg(&dir);
        add_table(&reg_path, "orders");
        let mut count = 0u32;
        watch_catalog(&reg_path, Duration::ZERO, Some(1), |event| {
            if let WatchEvent::Initial { .. } = event {
                count += 1;
            }
            WatchControl::Stop
        })
        .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn empty_registry_initial_event_has_no_tables() {
        let dir = TempDir::new().unwrap();
        let reg_path = reg(&dir);
        std::fs::create_dir_all(&reg_path).unwrap();
        let mut table_count = None;
        watch_catalog(&reg_path, Duration::ZERO, Some(1), |event| {
            if let WatchEvent::Initial { export } = event {
                table_count = Some(export.tables.len());
            }
            WatchControl::Stop
        })
        .unwrap();
        assert_eq!(table_count, Some(0));
    }

    #[test]
    fn unchanged_after_two_polls_with_no_change() {
        let dir = TempDir::new().unwrap();
        let reg_path = reg(&dir);
        add_table(&reg_path, "t");
        let events: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
        let ev = events.clone();
        watch_catalog(&reg_path, Duration::ZERO, Some(2), |event| {
            let label = match &event {
                WatchEvent::Initial { .. } => "initial",
                WatchEvent::Unchanged => "unchanged",
                WatchEvent::Changed { .. } => "changed",
            };
            ev.lock().unwrap().push(label.to_string());
            WatchControl::Continue
        })
        .unwrap();
        let ev = events.lock().unwrap();
        assert_eq!(ev[0], "initial");
        assert_eq!(ev[1], "unchanged");
    }

    #[test]
    fn stop_control_halts_loop() {
        let dir = TempDir::new().unwrap();
        let reg_path = reg(&dir);
        add_table(&reg_path, "t");
        let mut calls = 0u32;
        watch_catalog(&reg_path, Duration::ZERO, None, |_| {
            calls += 1;
            WatchControl::Stop
        })
        .unwrap();
        assert_eq!(calls, 1);
    }

    #[test]
    fn max_iterations_limits_calls() {
        let dir = TempDir::new().unwrap();
        let reg_path = reg(&dir);
        add_table(&reg_path, "t");
        let mut calls = 0u32;
        watch_catalog(&reg_path, Duration::ZERO, Some(3), |_| {
            calls += 1;
            WatchControl::Continue
        })
        .unwrap();
        assert_eq!(calls, 3);
    }
}
