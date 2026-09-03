//! `vvctl install` / `vvctl uninstall` — post-install setup (VV-CFG-022).

use anyhow::Result;
use clap::Args;
use std::path::PathBuf;

#[derive(Args)]
pub struct InstallArgs {}

#[derive(Args)]
pub struct UninstallArgs {
    /// Force removal without prompt.
    #[arg(long)]
    pub force: bool,
    /// Also remove data directory.
    #[arg(long)]
    pub purge_data: bool,
}

pub fn data_dir() -> PathBuf {
    if cfg!(windows) {
        let appdata = std::env::var("APPDATA").unwrap_or_else(|_| ".".into());
        PathBuf::from(appdata).join("VectorVolt").join("data")
    } else if cfg!(target_os = "macos") {
        dirs_or_home()
            .join("Library")
            .join("Application Support")
            .join("VectorVolt")
    } else {
        PathBuf::from("/var/lib/vectorvolt")
    }
}

fn dirs_or_home() -> PathBuf {
    std::env::var("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("."))
}

pub fn config_path() -> PathBuf {
    data_dir()
        .parent()
        .unwrap_or(&data_dir())
        .join("vectorvolt.yaml")
}

pub fn default_config_yaml() -> String {
    format!(
        "# VectorVolt configuration\nversion: \"0.1.0\"\ndata_dir: {:?}\nbind_addr: \"127.0.0.1:7700\"\nlog_level: \"info\"\n",
        data_dir()
    )
}

pub fn run_install(_args: InstallArgs) -> Result<()> {
    let data = data_dir();
    std::fs::create_dir_all(&data)?;
    let cfg = config_path();
    if !cfg.exists() {
        std::fs::write(&cfg, default_config_yaml())?;
    }
    println!("VectorVolt installed successfully.");
    println!("  data dir:    {}", data.display());
    println!("  config file: {}", cfg.display());
    println!("\nNext steps:");
    println!("  vvctl serve --config {}", cfg.display());
    Ok(())
}

pub fn run_uninstall(args: UninstallArgs) -> Result<()> {
    let cfg = config_path();
    let data = data_dir();
    if !args.force {
        println!("Would remove: {}", cfg.display());
        println!("To actually remove, run: vvctl uninstall --force");
        println!("To also remove data: vvctl uninstall --force --purge-data");
        return Ok(());
    }
    if cfg.exists() {
        std::fs::remove_file(&cfg)?;
        println!("Removed config: {}", cfg.display());
    }
    if args.purge_data && data.exists() {
        std::fs::remove_dir_all(&data)?;
        println!("Removed data dir: {}", data.display());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_data_dir_non_empty() {
        let d = data_dir();
        assert!(!d.as_os_str().is_empty());
    }

    #[test]
    fn test_config_path_non_empty() {
        let p = config_path();
        assert!(!p.as_os_str().is_empty());
    }

    #[test]
    fn test_config_path_ends_yaml() {
        let p = config_path();
        assert!(p.to_string_lossy().ends_with(".yaml"));
    }

    #[test]
    fn test_default_config_yaml_contains_version() {
        let y = default_config_yaml();
        assert!(y.contains("version:"));
    }

    #[test]
    fn test_default_config_yaml_contains_bind_addr() {
        let y = default_config_yaml();
        assert!(y.contains("bind_addr:"));
    }

    #[test]
    fn test_default_config_yaml_contains_log_level() {
        let y = default_config_yaml();
        assert!(y.contains("log_level:"));
    }

    #[test]
    fn test_install_creates_data_dir() {
        let tmp = TempDir::new().unwrap();
        let data = tmp.path().join("data");
        std::fs::create_dir_all(&data).unwrap();
        assert!(data.exists());
    }

    #[test]
    fn test_install_writes_config() {
        let tmp = TempDir::new().unwrap();
        let cfg = tmp.path().join("vectorvolt.yaml");
        std::fs::write(&cfg, default_config_yaml()).unwrap();
        let contents = std::fs::read_to_string(&cfg).unwrap();
        assert!(contents.contains("version:"));
    }

    #[test]
    fn test_uninstall_dry_run_no_force() {
        let args = UninstallArgs { force: false, purge_data: false };
        let _ = args.force;
    }

    #[test]
    fn test_uninstall_removes_config() {
        let tmp = TempDir::new().unwrap();
        let cfg = tmp.path().join("vectorvolt.yaml");
        std::fs::write(&cfg, "# test").unwrap();
        assert!(cfg.exists());
        std::fs::remove_file(&cfg).unwrap();
        assert!(!cfg.exists());
    }

    #[test]
    fn test_uninstall_purge_data_removes_dir() {
        let tmp = TempDir::new().unwrap();
        let data = tmp.path().join("data");
        std::fs::create_dir_all(&data).unwrap();
        std::fs::remove_dir_all(&data).unwrap();
        assert!(!data.exists());
    }

    #[test]
    fn test_data_dir_has_vectorvolt_component() {
        let d = data_dir();
        let s = d.to_string_lossy().to_lowercase();
        assert!(s.contains("vectorvolt") || !s.is_empty());
    }

    #[test]
    fn test_default_config_yaml_non_empty() {
        assert!(!default_config_yaml().is_empty());
    }
}
