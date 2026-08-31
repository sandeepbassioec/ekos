//! Integration tests for Layer 178: version banner (VV-CTL-178).

#[test]
fn version_const_nonempty() {
    let v = env!("CARGO_PKG_VERSION");
    assert!(!v.is_empty());
}

#[test]
fn version_has_dot() {
    let v = env!("CARGO_PKG_VERSION");
    assert!(v.contains('.'), "expected semver with dot, got {v}");
}

#[test]
fn hal_describe_nonempty() {
    let d = vectorvolt_hal::describe();
    assert!(!d.is_empty());
}

#[test]
fn hal_describe_contains_simd_or_scalar() {
    let d = vectorvolt_hal::describe().to_lowercase();
    let has_simd = d.contains("avx2")
        || d.contains("avx")
        || d.contains("scalar")
        || d.contains("sse")
        || d.contains("simd")
        || d.contains("neon");
    assert!(has_simd, "hal describe should mention SIMD tier: {d}");
}

#[test]
fn version_major_is_numeric() {
    let v = env!("CARGO_PKG_VERSION");
    let major = v.split('.').next().unwrap_or("0");
    assert!(
        major.parse::<u32>().is_ok(),
        "major version not numeric: {major}"
    );
}

#[test]
fn version_not_0_0_0() {
    let v = env!("CARGO_PKG_VERSION");
    assert_ne!(v, "0.0.0", "version should be set");
}

#[test]
fn hal_describe_length_reasonable() {
    let d = vectorvolt_hal::describe();
    assert!(d.len() < 500, "hal describe suspiciously long: {}", d.len());
}

#[test]
fn pkg_name_is_vvctl() {
    let name = env!("CARGO_PKG_NAME");
    assert_eq!(name, "vvctl");
}

#[test]
fn hal_describe_not_error_string() {
    let d = vectorvolt_hal::describe().to_lowercase();
    assert!(!d.contains("panic"), "hal describe contains panic: {d}");
}

#[test]
fn version_parts_count_at_least_2() {
    let v = env!("CARGO_PKG_VERSION");
    let parts: Vec<&str> = v.split('.').collect();
    assert!(parts.len() >= 2, "expected at least major.minor: {v}");
}

#[test]
fn hal_describe_ascii() {
    let d = vectorvolt_hal::describe();
    assert!(d.is_ascii(), "hal describe should be ASCII: {d}");
}

#[test]
fn version_minor_is_numeric() {
    let v = env!("CARGO_PKG_VERSION");
    let parts: Vec<&str> = v.split('.').collect();
    if parts.len() >= 2 {
        assert!(
            parts[1].parse::<u32>().is_ok(),
            "minor version not numeric: {}",
            parts[1]
        );
    }
}

#[test]
fn hal_describe_two_calls_consistent() {
    let a = vectorvolt_hal::describe();
    let b = vectorvolt_hal::describe();
    assert_eq!(a, b, "hal describe should be deterministic");
}
