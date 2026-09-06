//! Tests for vvctl completion command (VV-CFG-023).
use vvctl::commands::completion::{CompletionShell, generate_to_string, stub_command};

fn bash_completion() -> String {
    generate_to_string(&CompletionShell::Bash, &mut stub_command())
}
fn zsh_completion() -> String {
    generate_to_string(&CompletionShell::Zsh, &mut stub_command())
}
fn fish_completion() -> String {
    generate_to_string(&CompletionShell::Fish, &mut stub_command())
}
fn powershell_completion() -> String {
    generate_to_string(&CompletionShell::Powershell, &mut stub_command())
}

#[test]
fn test_bash_completion_non_empty() {
    assert!(!bash_completion().is_empty());
}

#[test]
fn test_zsh_completion_non_empty() {
    assert!(!zsh_completion().is_empty());
}

#[test]
fn test_fish_completion_non_empty() {
    assert!(!fish_completion().is_empty());
}

#[test]
fn test_powershell_completion_non_empty() {
    assert!(!powershell_completion().is_empty());
}

#[test]
fn test_bash_contains_vvctl() {
    assert!(bash_completion().contains("vvctl"));
}

#[test]
fn test_zsh_contains_vvctl() {
    assert!(zsh_completion().contains("vvctl"));
}

#[test]
fn test_bash_contains_table() {
    assert!(bash_completion().contains("table"));
}

#[test]
fn test_zsh_contains_serve() {
    assert!(zsh_completion().contains("serve"));
}

#[test]
fn test_fish_contains_connect() {
    assert!(fish_completion().contains("connect"));
}

#[test]
fn test_powershell_contains_register_argument_completer() {
    assert!(powershell_completion().contains("Register-ArgumentCompleter"));
}

#[test]
fn test_all_shells_different() {
    let b = bash_completion();
    let z = zsh_completion();
    let f = fish_completion();
    let p = powershell_completion();
    assert_ne!(b, z);
    assert_ne!(b, f);
    assert_ne!(b, p);
    assert_ne!(z, f);
}

#[test]
fn test_completion_valid_utf8() {
    for shell in [
        CompletionShell::Bash,
        CompletionShell::Zsh,
        CompletionShell::Fish,
        CompletionShell::Powershell,
    ] {
        let s = generate_to_string(&shell, &mut stub_command());
        assert!(std::str::from_utf8(s.as_bytes()).is_ok());
    }
}

#[test]
fn test_completion_length_gt_100() {
    for shell in [
        CompletionShell::Bash,
        CompletionShell::Zsh,
        CompletionShell::Fish,
        CompletionShell::Powershell,
    ] {
        let s = generate_to_string(&shell, &mut stub_command());
        assert!(
            s.len() > 100,
            "completion for {:?} too short: {}",
            shell,
            s.len()
        );
    }
}
