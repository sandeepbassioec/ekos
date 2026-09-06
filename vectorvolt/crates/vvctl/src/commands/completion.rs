//! `vvctl completion` — generate shell completion scripts via clap_complete (VV-CTL-018).

use anyhow::Result;
use clap::Args;

#[derive(Clone, Debug, clap::ValueEnum)]
pub enum CompletionShell {
    Bash,
    Zsh,
    Fish,
    Powershell,
    Elvish,
}

#[derive(Args)]
pub struct CompletionArgs {
    /// Target shell.
    pub shell: CompletionShell,
}

/// Generate a completion script for `shell` from the given clap `Command`.
///
/// Called from `main` where the full `Cli` command tree is available.
pub fn generate(shell: &CompletionShell, cmd: &mut clap::Command) {
    use clap_complete::Shell;
    use clap_complete::generate;

    let name = cmd.get_name().to_string();
    let mut stdout = std::io::stdout();

    match shell {
        CompletionShell::Bash => generate(Shell::Bash, cmd, name, &mut stdout),
        CompletionShell::Zsh => generate(Shell::Zsh, cmd, name, &mut stdout),
        CompletionShell::Fish => generate(Shell::Fish, cmd, name, &mut stdout),
        CompletionShell::Powershell => generate(Shell::PowerShell, cmd, name, &mut stdout),
        CompletionShell::Elvish => generate(Shell::Elvish, cmd, name, &mut stdout),
    }
}

/// Build the minimal clap `Command` used for completion generation in tests.
///
/// In production, `main` passes `Cli::command()` instead. This function exists so
/// the test crate can call `generate_to_string` without importing the private `Cli`.
#[doc(hidden)]
pub fn stub_command() -> clap::Command {
    clap::Command::new("vvctl")
        .subcommand(clap::Command::new("serve"))
        .subcommand(clap::Command::new("connect"))
        .subcommand(clap::Command::new("table"))
        .subcommand(clap::Command::new("completion"))
        .subcommand(clap::Command::new("install"))
}

/// Generate a completion script to a `String`. Requires a fresh `Cli::command()`.
pub fn generate_to_string(shell: &CompletionShell, cmd: &mut clap::Command) -> String {
    use clap_complete::Shell;
    use clap_complete::generate as clap_generate;

    let name = cmd.get_name().to_string();
    let mut buf = Vec::new();
    match shell {
        CompletionShell::Bash => clap_generate(Shell::Bash, cmd, name, &mut buf),
        CompletionShell::Zsh => clap_generate(Shell::Zsh, cmd, name, &mut buf),
        CompletionShell::Fish => clap_generate(Shell::Fish, cmd, name, &mut buf),
        CompletionShell::Powershell => clap_generate(Shell::PowerShell, cmd, name, &mut buf),
        CompletionShell::Elvish => clap_generate(Shell::Elvish, cmd, name, &mut buf),
    }
    String::from_utf8_lossy(&buf).into_owned()
}

/// Stub — real generation requires the `Cli` command tree from `main`.
///
/// `main` intercepts `Commands::Completion` before calling this and invokes
/// `generate()` directly with a fresh `Cli::command()` instance.
pub fn run(_args: CompletionArgs) -> Result<()> {
    Ok(())
}
