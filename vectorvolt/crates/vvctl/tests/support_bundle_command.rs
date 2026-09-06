//! Tests for vvctl support-bundle (VV-CFG-024).
use tempfile::TempDir;
use vvctl::commands::support_bundle::{
    SupportBundleArgs, run, sanitize_config, system_profile_json, version_txt,
};

#[test]
fn test_sanitize_config_hides_token() {
    let yaml = "token: my-secret-token\nbind_addr: 127.0.0.1:7700\n";
    let result = sanitize_config(yaml);
    assert!(!result.contains("my-secret-token"));
    assert!(result.contains("\"***\""));
}

#[test]
fn test_sanitize_config_preserves_other_fields() {
    let yaml = "bind_addr: 127.0.0.1:7700\nlog_level: info\n";
    let result = sanitize_config(yaml);
    assert!(result.contains("bind_addr"));
    assert!(result.contains("127.0.0.1:7700"));
}

#[test]
fn test_sanitize_config_hides_password() {
    let yaml = "password: hunter2\nhost: localhost\n";
    let result = sanitize_config(yaml);
    assert!(!result.contains("hunter2"));
}

#[test]
fn test_system_profile_json_is_valid_json() {
    let j = system_profile_json();
    let v: serde_json::Value = serde_json::from_str(&j).unwrap();
    assert!(v.is_object());
}

#[test]
fn test_system_profile_json_has_os() {
    let j = system_profile_json();
    let v: serde_json::Value = serde_json::from_str(&j).unwrap();
    assert!(v["os"].is_string());
}

#[test]
fn test_system_profile_json_has_arch() {
    let j = system_profile_json();
    let v: serde_json::Value = serde_json::from_str(&j).unwrap();
    assert!(v["arch"].is_string());
}

#[test]
fn test_system_profile_json_has_version() {
    let j = system_profile_json();
    let v: serde_json::Value = serde_json::from_str(&j).unwrap();
    assert!(v["version"].is_string());
}

#[test]
fn test_version_txt_contains_vectorvolt() {
    let v = version_txt();
    assert!(v.contains("vectorvolt"));
}

#[test]
fn test_run_creates_zip_file() {
    let tmp = TempDir::new().unwrap();
    let out = tmp.path().join("bundle.zip");
    run(SupportBundleArgs {
        output: out.clone(),
        data_dir: None,
    })
    .unwrap();
    assert!(out.exists());
}

#[test]
fn test_run_zip_contains_system_info() {
    let tmp = TempDir::new().unwrap();
    let out = tmp.path().join("bundle.zip");
    run(SupportBundleArgs {
        output: out.clone(),
        data_dir: None,
    })
    .unwrap();
    let bytes = std::fs::read(&out).unwrap();
    let reader = std::io::Cursor::new(bytes);
    let mut archive = zip::ZipArchive::new(reader).unwrap();
    let names: Vec<String> = (0..archive.len())
        .map(|i| archive.by_index(i).unwrap().name().to_string())
        .collect();
    assert!(names.contains(&"system_info.json".to_string()));
}

#[test]
fn test_run_zip_contains_env() {
    let tmp = TempDir::new().unwrap();
    let out = tmp.path().join("bundle.zip");
    run(SupportBundleArgs {
        output: out.clone(),
        data_dir: None,
    })
    .unwrap();
    let bytes = std::fs::read(&out).unwrap();
    let reader = std::io::Cursor::new(bytes);
    let mut archive = zip::ZipArchive::new(reader).unwrap();
    let names: Vec<String> = (0..archive.len())
        .map(|i| archive.by_index(i).unwrap().name().to_string())
        .collect();
    assert!(names.contains(&"env.txt".to_string()));
}

#[test]
fn test_run_zip_nonzero_size() {
    let tmp = TempDir::new().unwrap();
    let out = tmp.path().join("bundle.zip");
    run(SupportBundleArgs {
        output: out.clone(),
        data_dir: None,
    })
    .unwrap();
    let len = std::fs::metadata(&out).unwrap().len();
    assert!(len > 0);
}

#[test]
fn test_config_yaml_sanitized() {
    // sanitize_config masks sensitive keys
    let yaml = "token: secret-token\nbind_addr: 127.0.0.1:7700\n";
    let result = sanitize_config(yaml);
    assert!(!result.contains("secret-token"));
}
