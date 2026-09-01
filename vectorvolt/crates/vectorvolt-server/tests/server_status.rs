use vectorvolt_server::server_status;

#[test]
fn server_status_returns_string() {
    let s = server_status();
    assert!(!s.is_empty());
}

#[test]
fn server_status_contains_ok() {
    let s = server_status();
    assert!(s.contains("ok"));
}

#[test]
fn server_status_contains_version() {
    let s = server_status();
    assert!(s.contains("version"));
}

#[test]
fn server_status_is_json_like() {
    let s = server_status();
    assert!(s.starts_with('{'));
    assert!(s.ends_with('}'));
}

#[test]
fn server_status_contains_status_field() {
    let s = server_status();
    assert!(s.contains("\"status\""));
}

#[test]
fn server_status_contains_version_field() {
    let s = server_status();
    assert!(s.contains("\"version\""));
}

#[test]
fn server_status_version_value() {
    let s = server_status();
    assert!(s.contains("0.1.0"));
}

#[test]
fn server_status_idempotent() {
    let a = server_status();
    let b = server_status();
    assert_eq!(a, b);
}

#[test]
fn server_status_no_newlines() {
    let s = server_status();
    assert!(!s.contains('\n'));
}

#[test]
fn server_status_len_reasonable() {
    let s = server_status();
    assert!(s.len() < 256);
}

#[test]
fn server_status_status_ok_value() {
    let s = server_status();
    assert!(s.contains("\"ok\""));
}

#[test]
fn server_status_not_empty_object() {
    let s = server_status();
    assert_ne!(s, "{}");
}

#[test]
fn server_status_call_multiple() {
    for _ in 0..3 {
        let s = server_status();
        assert!(!s.is_empty());
    }
}
