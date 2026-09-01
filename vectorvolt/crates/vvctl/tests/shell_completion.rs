//! Integration tests for Layer 181: vvctl completion command (VV-CTL-181).

use vvctl::commands::completion::{CompletionArgs, CompletionShell, run};

#[test]
fn completion_bash_ok() {
    assert!(
        run(CompletionArgs {
            shell: CompletionShell::Bash
        })
        .is_ok()
    );
}

#[test]
fn completion_zsh_ok() {
    assert!(
        run(CompletionArgs {
            shell: CompletionShell::Zsh
        })
        .is_ok()
    );
}

#[test]
fn completion_fish_ok() {
    assert!(
        run(CompletionArgs {
            shell: CompletionShell::Fish
        })
        .is_ok()
    );
}

#[test]
fn completion_powershell_ok() {
    assert!(
        run(CompletionArgs {
            shell: CompletionShell::Powershell
        })
        .is_ok()
    );
}

#[test]
fn completion_bash_twice_ok() {
    assert!(
        run(CompletionArgs {
            shell: CompletionShell::Bash
        })
        .is_ok()
    );
    assert!(
        run(CompletionArgs {
            shell: CompletionShell::Bash
        })
        .is_ok()
    );
}

#[test]
fn completion_zsh_twice_ok() {
    assert!(
        run(CompletionArgs {
            shell: CompletionShell::Zsh
        })
        .is_ok()
    );
    assert!(
        run(CompletionArgs {
            shell: CompletionShell::Zsh
        })
        .is_ok()
    );
}

#[test]
fn completion_fish_twice_ok() {
    assert!(
        run(CompletionArgs {
            shell: CompletionShell::Fish
        })
        .is_ok()
    );
    assert!(
        run(CompletionArgs {
            shell: CompletionShell::Fish
        })
        .is_ok()
    );
}

#[test]
fn completion_all_four_shells_ok() {
    for shell in [
        CompletionShell::Bash,
        CompletionShell::Zsh,
        CompletionShell::Fish,
        CompletionShell::Powershell,
    ] {
        assert!(run(CompletionArgs { shell }).is_ok());
    }
}

#[test]
fn completion_bash_neverr() {
    let result = run(CompletionArgs {
        shell: CompletionShell::Bash,
    });
    assert!(result.is_ok(), "bash completion should not error");
}

#[test]
fn completion_zsh_neverr() {
    let result = run(CompletionArgs {
        shell: CompletionShell::Zsh,
    });
    assert!(result.is_ok());
}

#[test]
fn completion_fish_neverr() {
    let result = run(CompletionArgs {
        shell: CompletionShell::Fish,
    });
    assert!(result.is_ok());
}

#[test]
fn completion_powershell_neverr() {
    let result = run(CompletionArgs {
        shell: CompletionShell::Powershell,
    });
    assert!(result.is_ok());
}

#[test]
fn completion_is_fast() {
    let start = std::time::Instant::now();
    run(CompletionArgs {
        shell: CompletionShell::Bash,
    })
    .unwrap();
    assert!(start.elapsed().as_secs() < 2);
}
