//! `vvctl completion` — print shell completion scripts (VV-CTL-181).

use anyhow::Result;
use clap::Args;

#[derive(Clone, Debug, clap::ValueEnum)]
pub enum CompletionShell {
    Bash,
    Zsh,
    Fish,
    Powershell,
}

#[derive(Args)]
pub struct CompletionArgs {
    /// Target shell.
    pub shell: CompletionShell,
}

pub fn run(args: CompletionArgs) -> Result<()> {
    let name = "vvctl";
    let script = match args.shell {
        CompletionShell::Bash => format!(
            "# bash completion for {name}\ncomplete -F _{name} {name}\n_{name}() {{ COMPREPLY=(); }}\n"
        ),
        CompletionShell::Zsh => format!(
            "#compdef {name}\n_{name}() {{ _arguments ':command:(table wal config bench)' }}\n"
        ),
        CompletionShell::Fish => format!("# fish completion for {name}\ncomplete -c {name} -f\n"),
        CompletionShell::Powershell => format!(
            "# PowerShell completion for {name}\nRegister-ArgumentCompleter -Native -CommandName {name} -ScriptBlock {{ }}\n"
        ),
    };
    print!("{script}");
    Ok(())
}
