//! Integration tests for L133: vvctl profile (VV-CFG-001 CLI surface).

use vvctl::commands::profile::{ProfileArgs, ProfileOutput, run};

// ── JSON output (default) ─────────────────────────────────────────────────────

#[test]
fn profile_json_succeeds() {
    run(ProfileArgs {
        output: ProfileOutput::Json,
        recommendations: false,
    })
    .unwrap();
}

#[test]
fn profile_json_recommendations_succeeds() {
    run(ProfileArgs {
        output: ProfileOutput::Json,
        recommendations: true,
    })
    .unwrap();
}

// ── Table output ──────────────────────────────────────────────────────────────

#[test]
fn profile_table_succeeds() {
    run(ProfileArgs {
        output: ProfileOutput::Table,
        recommendations: false,
    })
    .unwrap();
}

#[test]
fn profile_table_recommendations_succeeds() {
    run(ProfileArgs {
        output: ProfileOutput::Table,
        recommendations: true,
    })
    .unwrap();
}

// ── Underlying SystemProfile invariants ──────────────────────────────────────

#[test]
fn profile_logical_cores_at_least_one() {
    use vectorvolt_engine::SystemProfile;
    let p = SystemProfile::probe();
    assert!(p.logical_cores >= 1);
}

#[test]
fn profile_physical_cores_at_least_one() {
    use vectorvolt_engine::SystemProfile;
    let p = SystemProfile::probe();
    assert!(p.physical_cores >= 1);
}

#[test]
fn profile_numa_node_count_at_least_one() {
    use vectorvolt_engine::SystemProfile;
    let p = SystemProfile::probe();
    assert!(p.numa_node_count >= 1);
}

#[test]
fn profile_simd_tier_name_is_known() {
    use vectorvolt_engine::SystemProfile;
    let p = SystemProfile::probe();
    let name = p.simd_tier.name();
    assert!(["scalar", "avx2", "avx512", "neon"].contains(&name));
}

#[test]
fn profile_os_page_size_power_of_two() {
    use vectorvolt_engine::SystemProfile;
    let p = SystemProfile::probe();
    assert!(p.os_page_size >= 4096);
    assert_eq!(p.os_page_size & (p.os_page_size - 1), 0);
}

#[test]
fn profile_recommended_flush_threshold_in_bounds() {
    use vectorvolt_engine::SystemProfile;
    let p = SystemProfile::probe();
    assert!(p.recommended_flush_threshold >= 1_024);
    assert!(p.recommended_flush_threshold <= 1_048_576);
}

#[test]
fn profile_recommended_partition_count_in_bounds() {
    use vectorvolt_engine::SystemProfile;
    let p = SystemProfile::probe();
    assert!(p.recommended_partition_count >= 1);
    assert!(p.recommended_partition_count <= 64);
}

#[test]
fn profile_two_probes_agree() {
    use vectorvolt_engine::SystemProfile;
    let a = SystemProfile::probe();
    let b = SystemProfile::probe();
    assert_eq!(a.logical_cores, b.logical_cores);
    assert_eq!(a.recommended_partition_count, b.recommended_partition_count);
}

#[test]
fn profile_summary_contains_simd() {
    use vectorvolt_engine::SystemProfile;
    assert!(SystemProfile::probe().summary().contains("SIMD="));
}
