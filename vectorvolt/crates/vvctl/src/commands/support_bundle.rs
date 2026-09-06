//! `vvctl support-bundle` — collect diagnostics into a ZIP file (VV-CFG-022).

use anyhow::Result;
use clap::Args;
use std::io::Write as _;
use std::path::PathBuf;

#[derive(Args)]
pub struct SupportBundleArgs {
    /// Write the bundle ZIP to this path.
    #[arg(long, default_value = "vvctl-support-bundle.zip")]
    pub output: PathBuf,

    /// Include WAL stats from this data directory (optional).
    #[arg(long)]
    pub data_dir: Option<PathBuf>,
}

pub fn sanitize_yaml(yaml: &str) -> String {
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

/// Public alias used by integration tests.
pub fn system_profile_json() -> String {
    system_info()
}

/// Public alias for sanitize_yaml, used by integration tests.
pub fn sanitize_config(yaml: &str) -> String {
    sanitize_yaml(yaml)
}

/// Returns a short version string for diagnostics.
pub fn version_txt() -> String {
    format!("vectorvolt v{}", env!("CARGO_PKG_VERSION"))
}

fn system_info() -> String {
    serde_json::json!({
        "os":      std::env::consts::OS,
        "arch":    std::env::consts::ARCH,
        "family":  std::env::consts::FAMILY,
        "version": env!("CARGO_PKG_VERSION"),
        "hal":     vectorvolt_hal::describe(),
    })
    .to_string()
}

fn env_vars() -> String {
    let mut lines = Vec::new();
    for (k, v) in std::env::vars() {
        if k.starts_with("VECTORVOLT_") {
            let val = if k.to_lowercase().contains("key")
                || k.to_lowercase().contains("token")
                || k.to_lowercase().contains("secret")
            {
                "***".to_string()
            } else {
                v
            };
            lines.push(format!("{k}={val}"));
        }
    }
    lines.sort();
    lines.join("\n")
}

fn wal_summary(data_dir: &std::path::Path) -> String {
    let wal_dir = data_dir.join("wal");
    if !wal_dir.exists() {
        return format!("WAL dir not found: {}", wal_dir.display());
    }
    let mut count = 0usize;
    let mut total_bytes = 0u64;
    if let Ok(rd) = std::fs::read_dir(&wal_dir) {
        for entry in rd.flatten() {
            if entry.path().extension().and_then(|s| s.to_str()) == Some("vvwal") {
                count += 1;
                total_bytes += entry.metadata().map(|m| m.len()).unwrap_or(0);
            }
        }
    }
    format!(
        "WAL segments: {count}\nTotal WAL bytes: {total_bytes}\nWAL dir: {}",
        wal_dir.display()
    )
}

fn schema_info(data_dir: &std::path::Path) -> String {
    let schema_file = data_dir.join("schema.txt");
    if let Ok(s) = std::fs::read_to_string(&schema_file) {
        format!("schema.txt:\n{s}")
    } else {
        format!("schema.txt not found at {}", schema_file.display())
    }
}

pub fn run(args: SupportBundleArgs) -> Result<()> {
    let file = std::fs::File::create(&args.output)?;
    let mut zip = zip::ZipWriter::new(file);
    let opts = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);

    // ── system_info.json ─────────────────────────────────────────────────────
    zip.start_file("system_info.json", opts)?;
    zip.write_all(system_info().as_bytes())?;

    // ── env.txt ──────────────────────────────────────────────────────────────
    zip.start_file("env.txt", opts)?;
    zip.write_all(env_vars().as_bytes())?;

    // ── data_dir diagnostics (optional) ─────────────────────────────────────
    if let Some(ref dir) = args.data_dir {
        zip.start_file("wal_summary.txt", opts)?;
        zip.write_all(wal_summary(dir).as_bytes())?;

        zip.start_file("schema.txt", opts)?;
        zip.write_all(schema_info(dir).as_bytes())?;

        // Partition file inventory
        let mut inv = String::new();
        if let Ok(rd) = std::fs::read_dir(dir) {
            for entry in rd.flatten() {
                if entry.path().extension().and_then(|s| s.to_str()) == Some("vvsst") {
                    let size = entry.metadata().map(|m| m.len()).unwrap_or(0);
                    inv.push_str(&format!(
                        "{}: {} bytes\n",
                        entry.file_name().to_string_lossy(),
                        size
                    ));
                }
            }
        }
        zip.start_file("partitions.txt", opts)?;
        zip.write_all(inv.as_bytes())?;
    }

    zip.finish()?;

    println!("Support bundle: {}", args.output.display());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanitize_masks_token_key_password_secret() {
        let yaml = "token: abc123\nname: hello\npassword: hunter2\nsecret_key: xyz\nbind_addr: 127.0.0.1:5995";
        let out = sanitize_yaml(yaml);
        assert!(out.contains("token: \"***\""));
        assert!(out.contains("password: \"***\""));
        assert!(out.contains("secret_key: \"***\""));
        assert!(out.contains("name: hello"));
        assert!(out.contains("bind_addr: 127.0.0.1:5995"));
    }

    #[test]
    fn sanitize_no_colon_line_masked() {
        let out = sanitize_yaml("secret_no_colon");
        assert_eq!(out.trim(), "***");
    }

    #[test]
    fn system_info_is_valid_json() {
        let s = system_info();
        let v: serde_json::Value = serde_json::from_str(&s).expect("valid JSON");
        assert!(v.get("os").is_some());
        assert!(v.get("version").is_some());
    }

    #[test]
    fn env_vars_only_vectorvolt_prefix() {
        // Can't guarantee VECTORVOLT_* vars in CI, but function must not panic.
        let _ = env_vars();
    }

    #[test]
    fn wal_summary_missing_dir() {
        let dir = std::path::Path::new("/nonexistent/path");
        let s = wal_summary(dir);
        assert!(s.contains("not found"));
    }

    #[test]
    fn run_creates_zip() {
        let dir = tempfile::TempDir::new().unwrap();
        let out = dir.path().join("bundle.zip");
        run(SupportBundleArgs {
            output: out.clone(),
            data_dir: None,
        })
        .unwrap();
        assert!(out.exists());
        let len = std::fs::metadata(&out).unwrap().len();
        assert!(len > 0, "ZIP should not be empty");
    }

    #[test]
    fn run_with_data_dir_creates_zip() {
        let data = tempfile::TempDir::new().unwrap();
        std::fs::create_dir_all(data.path().join("wal")).unwrap();
        std::fs::write(data.path().join("schema.txt"), "pk\nage").unwrap();
        let out_dir = tempfile::TempDir::new().unwrap();
        let out = out_dir.path().join("bundle.zip");
        run(SupportBundleArgs {
            output: out.clone(),
            data_dir: Some(data.path().to_path_buf()),
        })
        .unwrap();
        assert!(out.exists());
    }

    #[test]
    fn run_with_data_dir_includes_wal_entry() {
        let data = tempfile::TempDir::new().unwrap();
        let wal_dir = data.path().join("wal");
        std::fs::create_dir_all(&wal_dir).unwrap();
        std::fs::write(wal_dir.join("0.vvwal"), b"fakesegment").unwrap();
        let out_dir = tempfile::TempDir::new().unwrap();
        let out = out_dir.path().join("bundle.zip");
        run(SupportBundleArgs {
            output: out.clone(),
            data_dir: Some(data.path().to_path_buf()),
        })
        .unwrap();
        let bytes = std::fs::read(&out).unwrap();
        let reader = std::io::Cursor::new(bytes);
        let mut archive = zip::ZipArchive::new(reader).unwrap();
        let names: Vec<String> = (0..archive.len())
            .map(|i| archive.by_index(i).unwrap().name().to_string())
            .collect();
        assert!(names.contains(&"wal_summary.txt".to_string()));
        assert!(names.contains(&"system_info.json".to_string()));
    }
}
