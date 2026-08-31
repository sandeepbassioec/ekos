use std::io::Write;
use tempfile::NamedTempFile;
use vvctl::commands::config_audit::run;

#[test]
fn missing_config_returns_ok() {
    let result = run(Some(std::path::PathBuf::from("/nonexistent/path/vv.toml")));
    assert!(result.is_ok());
}

#[test]
fn default_path_missing_returns_ok() {
    // Run with None — will use "vv.toml" which doesn't exist in test CWD
    let result = run(None);
    assert!(result.is_ok());
}

#[test]
fn existing_config_clean_returns_ok() {
    let mut f = NamedTempFile::new().unwrap();
    writeln!(f, "port = 7777").unwrap();
    let result = run(Some(f.path().to_path_buf()));
    assert!(result.is_ok());
}

#[test]
fn config_with_deprecated_key_returns_ok() {
    let mut f = NamedTempFile::new().unwrap();
    writeln!(f, "deprecated_key = true").unwrap();
    let result = run(Some(f.path().to_path_buf()));
    assert!(result.is_ok());
}

#[test]
fn empty_config_returns_ok() {
    let f = NamedTempFile::new().unwrap();
    let result = run(Some(f.path().to_path_buf()));
    assert!(result.is_ok());
}

#[test]
fn config_with_multiple_keys_returns_ok() {
    let mut f = NamedTempFile::new().unwrap();
    writeln!(f, "host = \"127.0.0.1\"\nport = 9000").unwrap();
    let result = run(Some(f.path().to_path_buf()));
    assert!(result.is_ok());
}

#[test]
fn config_with_deprecated_in_value_returns_ok() {
    let mut f = NamedTempFile::new().unwrap();
    writeln!(f, "name = \"deprecated_key\"").unwrap();
    let result = run(Some(f.path().to_path_buf()));
    assert!(result.is_ok());
}

#[test]
fn config_large_file_returns_ok() {
    let mut f = NamedTempFile::new().unwrap();
    for i in 0..100 {
        writeln!(f, "key_{i} = {i}").unwrap();
    }
    let result = run(Some(f.path().to_path_buf()));
    assert!(result.is_ok());
}

#[test]
fn config_with_toml_sections_returns_ok() {
    let mut f = NamedTempFile::new().unwrap();
    writeln!(f, "[server]\nhost = \"localhost\"\nport = 7777").unwrap();
    let result = run(Some(f.path().to_path_buf()));
    assert!(result.is_ok());
}

#[test]
fn path_none_uses_default() {
    // None triggers default path lookup — should not panic
    assert!(run(None).is_ok());
}

#[test]
fn path_to_directory_returns_ok() {
    let dir = tempfile::tempdir().unwrap();
    // Pointing at a directory — it won't exist as a file
    let result = run(Some(dir.path().join("nonexistent.toml")));
    assert!(result.is_ok());
}

#[test]
fn unicode_content_returns_ok() {
    let mut f = NamedTempFile::new().unwrap();
    writeln!(f, "# 配置文件\nhost = \"本地\"").unwrap();
    let result = run(Some(f.path().to_path_buf()));
    assert!(result.is_ok());
}

#[test]
fn config_only_comments_returns_ok() {
    let mut f = NamedTempFile::new().unwrap();
    writeln!(f, "# just a comment\n# another comment").unwrap();
    let result = run(Some(f.path().to_path_buf()));
    assert!(result.is_ok());
}
