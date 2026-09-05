//! Layer 85 â€” `vvctl tune` (VV-CFG-006).
//!
//! Tests the `DerivedConfig` probe + TOML emission path without requiring a
//! live interactive terminal.

use vvctl::commands::tune::{DerivedConfig, measure_fsync_ns};

// â”€â”€ probe basics â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

#[test]
fn probe_succeeds() {
    // Must not panic on any platform.
    let _ = DerivedConfig::probe(64);
}

#[test]
fn derived_flush_threshold_is_nonzero() {
    let cfg = DerivedConfig::probe(64);
    assert!(cfg.flush_threshold > 0, "flush_threshold must be > 0");
}

#[test]
fn derived_flush_threshold_is_within_clamp_bounds() {
    let cfg = DerivedConfig::probe(64);
    assert!(
        cfg.flush_threshold >= 1_024,
        "flush_threshold below minimum 1024: {}",
        cfg.flush_threshold
    );
    assert!(
        cfg.flush_threshold <= 1_048_576,
        "flush_threshold above maximum 1048576: {}",
        cfg.flush_threshold
    );
}

#[test]
fn derived_durability_is_valid_string() {
    let cfg = DerivedConfig::probe(64);
    assert!(
        cfg.durability == "fsync" || cfg.durability == "buffered",
        "unexpected durability: {}",
        cfg.durability
    );
}

#[test]
fn derived_max_partitions_is_positive() {
    let cfg = DerivedConfig::probe(64);
    assert!(
        cfg.max_partitions > 0,
        "max_partitions must be > 0; got {}",
        cfg.max_partitions
    );
}

#[test]
fn derived_max_partitions_is_within_clamp_bounds() {
    let cfg = DerivedConfig::probe(64);
    assert!(cfg.max_partitions >= 4, "max_partitions below minimum 4");
    assert!(
        cfg.max_partitions <= 256,
        "max_partitions above maximum 256"
    );
}

#[test]
fn derived_writer_count_is_positive() {
    let cfg = DerivedConfig::probe(64);
    assert!(cfg.writer_count >= 1, "writer_count must be >= 1");
}

#[test]
fn topo_summary_is_nonempty() {
    let cfg = DerivedConfig::probe(64);
    assert!(
        !cfg.topo_summary.is_empty(),
        "topo_summary must not be empty"
    );
}

// â”€â”€ fsync calibration â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

#[test]
fn fsync_calibration_produces_plausible_latency() {
    // On any real disk the roundtrip must be > 0 ns and < 30 s.
    let ns = measure_fsync_ns();
    assert!(
        ns < 30_000_000_000,
        "fsync latency implausibly large: {ns} ns"
    );
}

// â”€â”€ TOML emission â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

#[test]
fn toml_output_is_valid_toml() {
    let cfg = DerivedConfig::probe(64);
    let text = cfg.to_annotated_toml(false);
    let parsed: Result<toml::Value, _> = toml::from_str(&text);
    assert!(
        parsed.is_ok(),
        "emitted TOML is not valid:\n{text}\nerror: {:?}",
        parsed.err()
    );
}

#[test]
fn toml_output_contains_flush_threshold() {
    let cfg = DerivedConfig::probe(64);
    let text = cfg.to_annotated_toml(false);
    assert!(
        text.contains("flush_threshold"),
        "TOML missing flush_threshold:\n{text}"
    );
}

#[test]
fn toml_output_contains_durability() {
    let cfg = DerivedConfig::probe(64);
    let text = cfg.to_annotated_toml(false);
    assert!(
        text.contains("durability"),
        "TOML missing durability:\n{text}"
    );
}

#[test]
fn toml_output_contains_max_partitions() {
    let cfg = DerivedConfig::probe(64);
    let text = cfg.to_annotated_toml(false);
    assert!(
        text.contains("max_partitions"),
        "TOML missing max_partitions:\n{text}"
    );
}

#[test]
fn explain_output_contains_comments() {
    let cfg = DerivedConfig::probe(64);
    let text = cfg.to_annotated_toml(true);
    let comment_lines = text.lines().filter(|l| l.starts_with('#')).count();
    assert!(
        comment_lines >= 3,
        "explain mode must have at least 3 comment lines; got {comment_lines}:\n{text}"
    );
}

#[test]
fn explain_mentions_cache_flush_threshold_and_fsync() {
    let cfg = DerivedConfig::probe(64);
    let text = cfg.to_annotated_toml(true);
    assert!(
        text.contains("cache") || text.contains("flush"),
        "explain must mention cache or flush context:\n{text}"
    );
    assert!(
        text.contains("fsync"),
        "explain must mention fsync:\n{text}"
    );
}

// ── diff_toml (VV-CFG-006 / S15) ─────────────────────────────────────────────

#[test]
fn diff_toml_no_drift_when_toml_matches_derived() {
    let cfg = DerivedConfig::probe(64);
    let toml = cfg.to_annotated_toml(false);
    let diffs = cfg.diff_toml(&toml);
    assert!(
        diffs.is_empty(),
        "expected no drift against self-generated TOML; got: {diffs:?}"
    );
}

#[test]
fn diff_toml_detects_flush_threshold_drift() {
    let cfg = DerivedConfig::probe(64);
    let wrong_threshold = if cfg.flush_threshold == 1_024 {
        2_048
    } else {
        1_024
    };
    let toml = format!(
        "flush_threshold = {wrong_threshold}\ndurability = \"{}\"\nmax_partitions = {}\n",
        cfg.durability, cfg.max_partitions
    );
    let diffs = cfg.diff_toml(&toml);
    let field_names: Vec<_> = diffs.iter().map(|(f, _, _)| *f).collect();
    assert!(
        field_names.contains(&"flush_threshold"),
        "flush_threshold drift should be reported; diffs: {diffs:?}"
    );
}

#[test]
fn diff_toml_parse_error_returns_error_sentinel() {
    let cfg = DerivedConfig::probe(64);
    let bad_toml = "this is [not valid toml !!";
    let diffs = cfg.diff_toml(bad_toml);
    assert!(
        !diffs.is_empty(),
        "parse error should produce an error sentinel entry"
    );
    assert_eq!(
        diffs[0].0, "(parse error)",
        "first diff entry should be the parse-error sentinel"
    );
}

#[test]
fn toml_output_roundtrips_through_engine_config() {
    let cfg = DerivedConfig::probe(64);
    let text = cfg.to_annotated_toml(false);
    let engine_cfg = vectorvolt_engine::config::EngineConfig::from_toml(&text);
    assert!(
        engine_cfg.is_ok(),
        "EngineConfig::from_toml rejected the emitted TOML: {:?}\n{text}",
        engine_cfg.err()
    );
    let ec = engine_cfg.unwrap();
    assert_eq!(ec.flush_threshold, Some(cfg.flush_threshold));
    assert_eq!(ec.max_partitions, Some(cfg.max_partitions));
    assert_eq!(ec.durability.as_deref(), Some(cfg.durability));
}
