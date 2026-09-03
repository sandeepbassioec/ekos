//! `vvctl support-bundle` — collect diagnostics into a bundle directory (VV-CTL-200).

use anyhow::Result;
use clap::Args;
use std::path::PathBuf;

#[derive(Args)]
pub struct SupportBundleArgs {
    /// Output directory for the bundle files.
    #[arg(long, default_value = "vvctl-support-bundle")]
    pub output: PathBuf,
}

pub fn sanitize_config(yaml: &str) -> String {
    let mut out = String::new();
    for line in yaml.lines() {
        let lower = line.to_lowercase();
        if lower.contains("token")
            || lower.contains("password")
            || lower.contains("secret")
            || lower.contains("key")
        {
            if let Some(colon) = line.find(':') {
                out.push_str(&line[..=colon]);
                out.push_str(" \"***\"\n");
            } else {
                out.push_str("***\n");
            }
        } else {
            out.push_str(line);
            out.push('\n');
        }
    }
    out
}

pub fn system_profile_json() -> String {
    serde_json::json!({
        "os": std::env::consts::OS,
        "arch": std::env::consts::ARCH,
        "version": env!("CARGO_PKG_VERSION"),
        "family": std::env::consts::FAMILY,
    })
    .to_string()
}

pub fn version_txt() -> String {
    format!("vectorvolt {}", env!("CARGO_PKG_VERSION"))
}

pub fn run(args: SupportBundleArgs) -> Result<()> {
    let out = &args.output;
    std::fs::create_dir_all(out)?;

    let config_content = default_config_content();
    let cfg_path = out.join("config.yaml");
    std::fs::write(&cfg_path, sanitize_config(&config_content))?;
    println!("  {}", cfg_path.display());

    let prof_path = out.join("system_profile.json");
    std::fs::write(&prof_path, system_profile_json())?;
    println!("  {}", prof_path.display());

    let ver_path = out.join("version.txt");
    std::fs::write(&ver_path, version_txt())?;
    println!("  {}", ver_path.display());

    println!("Support bundle written to: {}", out.display());
    Ok(())
}

fn default_config_content() -> String {
    "# VectorVolt configuration\nversion: \"0.1.0\"\nbind_addr: \"127.0.0.1:7700\"\nlog_level: \"info\"\ntoken: \"secret-token\"\n".into()
}
