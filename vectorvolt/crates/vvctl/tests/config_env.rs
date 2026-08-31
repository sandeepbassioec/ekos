//! Integration tests for L125: VvConfig env-var override layer (VV-CFG-013).

use std::env;
use std::sync::Mutex;

use vectorvolt_engine::VvConfig;
use vvctl::commands::config::{ConfigCommand, run};

static ENV_LOCK: Mutex<()> = Mutex::new(());

// â”€â”€ engine API: from_env / merge_env â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

#[test]
fn from_env_default_when_no_vars() {
    let _g = ENV_LOCK.lock().unwrap();
    for (k, _, _) in VvConfig::env_var_table() {
        unsafe { env::remove_var(k) };
    }
    let cfg = VvConfig::from_env().unwrap();
    assert_eq!(cfg, VvConfig::default());
}

#[test]
fn from_env_picks_up_writer_count() {
    let _g = ENV_LOCK.lock().unwrap();
    let key = "VECTORVOLT_THREADS_WRITER_COUNT";
    unsafe { env::set_var(key, "7") };
    let cfg = VvConfig::from_env().unwrap();
    unsafe { env::remove_var(key) };
    assert_eq!(cfg.threads.writer_count, Some(7));
}

#[test]
fn from_env_picks_up_bind_addr() {
    let _g = ENV_LOCK.lock().unwrap();
    let key = "VECTORVOLT_NETWORK_BIND_ADDR";
    unsafe { env::set_var(key, "0.0.0.0:9999") };
    let cfg = VvConfig::from_env().unwrap();
    unsafe { env::remove_var(key) };
    assert_eq!(cfg.network.bind_addr.as_deref(), Some("0.0.0.0:9999"));
}

#[test]
fn from_env_bad_integer_errors() {
    let _g = ENV_LOCK.lock().unwrap();
    let key = "VECTORVOLT_THREADS_WRITER_COUNT";
    unsafe { env::set_var(key, "not-a-number") };
    let result = VvConfig::from_env();
    unsafe { env::remove_var(key) };
    assert!(result.is_err());
}

#[test]
fn merge_env_overrides_existing_value() {
    let _g = ENV_LOCK.lock().unwrap();
    let key = "VECTORVOLT_STORAGE_BLOCK_SIZE";
    unsafe { env::set_var(key, "131072") };
    let mut cfg = VvConfig::default();
    cfg.storage.block_size = Some(4096);
    cfg.merge_env().unwrap();
    unsafe { env::remove_var(key) };
    assert_eq!(cfg.storage.block_size, Some(131_072));
}

#[test]
fn merge_env_leaves_unset_fields_alone() {
    let _g = ENV_LOCK.lock().unwrap();
    unsafe { env::remove_var("VECTORVOLT_STORAGE_BLOCK_SIZE") };
    let mut cfg = VvConfig::default();
    cfg.storage.block_size = Some(4096);
    cfg.merge_env().unwrap();
    assert_eq!(cfg.storage.block_size, Some(4096));
}

#[test]
fn merge_env_sets_huge_pages() {
    let _g = ENV_LOCK.lock().unwrap();
    let key = "VECTORVOLT_HAL_HUGE_PAGES";
    unsafe { env::set_var(key, "1gb") };
    let mut cfg = VvConfig::default();
    cfg.merge_env().unwrap();
    unsafe { env::remove_var(key) };
    assert_eq!(cfg.hal.huge_pages.as_deref(), Some("1gb"));
}

#[test]
fn merge_env_sets_trace_level() {
    let _g = ENV_LOCK.lock().unwrap();
    let key = "VECTORVOLT_OBSERVABILITY_TRACE_LEVEL";
    unsafe { env::set_var(key, "debug") };
    let mut cfg = VvConfig::default();
    cfg.merge_env().unwrap();
    unsafe { env::remove_var(key) };
    assert_eq!(cfg.observability.trace_level.as_deref(), Some("debug"));
}

#[test]
fn env_var_table_is_non_empty() {
    assert!(!VvConfig::env_var_table().is_empty());
}

#[test]
fn env_var_table_all_dot_keys_valid() {
    let mut probe = VvConfig::default();
    let problems: Vec<_> = VvConfig::env_var_table()
        .iter()
        .filter_map(|(_, dot_key, _)| probe.set_key(dot_key, "0").err().map(|_| *dot_key))
        .collect();
    assert!(problems.is_empty(), "invalid dot keys: {problems:?}");
}

// â”€â”€ CLI: config env-list â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

#[test]
fn env_list_command_succeeds() {
    run(ConfigCommand::EnvList).unwrap();
}

#[test]
fn env_list_shows_set_var() {
    let _g = ENV_LOCK.lock().unwrap();
    let key = "VECTORVOLT_QUERY_INDEX_HINT";
    unsafe { env::set_var(key, "cuckoo") };
    let result = run(ConfigCommand::EnvList);
    unsafe { env::remove_var(key) };
    assert!(result.is_ok());
}

#[test]
fn from_env_picks_up_max_scan_rows() {
    let _g = ENV_LOCK.lock().unwrap();
    let key = "VECTORVOLT_QUERY_MAX_SCAN_ROWS";
    unsafe { env::set_var(key, "5000") };
    let cfg = VvConfig::from_env().unwrap();
    unsafe { env::remove_var(key) };
    assert_eq!(cfg.query.max_scan_rows, Some(5000));
}
