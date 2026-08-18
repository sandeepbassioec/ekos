//! Terminal UX primitives — VV-CTL-016.
//!
//! Centralises color detection, prefix helpers, and table construction so every
//! command in `vvctl` presents a consistent Spectre.Console-style experience
//! without threading a `&Config` through every function call.
//!
//! ## Initialization
//!
//! Call [`init`] exactly once at the start of `main()`. After that, every
//! helper reads from two `AtomicBool`s and is safe to call from any thread.
//!
//! ## `NO_COLOR`
//!
//! The `NO_COLOR` env var (<https://no-color.org>) is read at init time.
//! Setting it disables all ANSI codes regardless of `--no-color`.

use std::sync::atomic::{AtomicBool, Ordering};

static COLOR_ENABLED: AtomicBool = AtomicBool::new(true);
static QUIET_MODE: AtomicBool = AtomicBool::new(false);

/// Initialise global UI state from CLI flags and environment.
///
/// Must be called before any other function in this module.
pub fn init(no_color: bool, quiet: bool) {
    let no_color_env = std::env::var_os("NO_COLOR").is_some();
    COLOR_ENABLED.store(!no_color && !no_color_env, Ordering::Relaxed);
    QUIET_MODE.store(quiet, Ordering::Relaxed);
}

/// Whether ANSI color codes should be emitted.
pub fn color_enabled() -> bool {
    COLOR_ENABLED.load(Ordering::Relaxed)
}

/// Whether decoration (banners, tables, spinners) should be suppressed.
pub fn quiet_mode() -> bool {
    QUIET_MODE.load(Ordering::Relaxed)
}

/// Print a success line: `✔ <msg>` (green when color enabled).
pub fn print_ok(msg: &str) {
    if quiet_mode() {
        return;
    }
    if color_enabled() {
        use owo_colors::OwoColorize;
        println!("{} {}", "✔".green().bold(), msg);
    } else {
        println!("✔ {msg}");
    }
}

/// Print a warning line: `⚠ <msg>` (yellow when color enabled).
pub fn print_warn(msg: &str) {
    if quiet_mode() {
        return;
    }
    if color_enabled() {
        use owo_colors::OwoColorize;
        println!("{} {}", "⚠".yellow().bold(), msg);
    } else {
        println!("⚠ {msg}");
    }
}

/// Print an error line to stderr: `✖ <msg>` (red when color enabled).
///
/// Unlike `print_ok`/`print_warn`, this is never suppressed by quiet mode
/// because errors must always be visible.
pub fn print_err(msg: &str) {
    if color_enabled() {
        use owo_colors::OwoColorize;
        eprintln!("{} {}", "✖".red().bold(), msg);
    } else {
        eprintln!("✖ {msg}");
    }
}

/// Print a colored section header followed by a separator line.
pub fn section_header(title: &str) {
    if quiet_mode() {
        return;
    }
    if color_enabled() {
        use owo_colors::OwoColorize;
        println!("\n{}", title.cyan().bold());
        println!("{}", "─".repeat(62).dimmed());
    } else {
        println!("\n{title}");
        println!("{}", "─".repeat(62));
    }
}

/// Format a speedup ratio with color coding.
///
/// | Range | Color |
/// |---|---|
/// | ≥ 2.0× | green bold |
/// | ≥ 1.1× | yellow |
/// | < 1.0× | red bold — regression |
pub fn fmt_speedup(x: f64) -> String {
    let s = format!("{x:.2}x");
    if !color_enabled() {
        return s;
    }
    use owo_colors::OwoColorize;
    if x >= 2.0 {
        s.green().bold().to_string()
    } else if x >= 1.1 {
        s.yellow().to_string()
    } else {
        s.red().bold().to_string()
    }
}

/// Build a styled `comfy_table::Table`.
///
/// Uses UTF-8 rounded corners when color is enabled; falls back to plain ASCII
/// borders so output remains readable in CI logs and `--no-color` contexts.
pub fn styled_table() -> comfy_table::Table {
    let mut t = comfy_table::Table::new();
    if color_enabled() {
        t.load_preset(comfy_table::presets::UTF8_FULL_CONDENSED);
        t.apply_modifier(comfy_table::modifiers::UTF8_ROUND_CORNERS);
    } else {
        t.load_preset(comfy_table::presets::ASCII_BORDERS_ONLY_CONDENSED);
    }
    t
}

/// Render a styled version banner.
///
/// Returns a multi-line string — print it with `println!("{}", ui::banner())`.
pub fn banner() -> String {
    let ver = env!("CARGO_PKG_VERSION");
    if color_enabled() {
        use owo_colors::OwoColorize;
        format!(
            "{}\n{}\n{}",
            " VectorVolt ".on_blue().white().bold(),
            format!("  v{ver}  ").cyan().bold(),
            "  ultra-low-latency kernel-bypass data engine  ".dimmed(),
        )
    } else {
        format!("VectorVolt v{ver}\nultra-low-latency kernel-bypass data engine")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn reset() {
        COLOR_ENABLED.store(true, Ordering::Relaxed);
        QUIET_MODE.store(false, Ordering::Relaxed);
    }

    // 1 — color disabled by direct store
    #[test]
    fn color_disabled_by_store() {
        reset();
        COLOR_ENABLED.store(false, Ordering::Relaxed);
        assert!(!color_enabled());
        reset();
    }

    // 2 — color enabled when not suppressed
    #[test]
    fn color_enabled_when_reset() {
        reset();
        assert!(color_enabled());
    }

    // 3 — quiet mode can be set
    #[test]
    fn quiet_mode_settable() {
        reset();
        QUIET_MODE.store(true, Ordering::Relaxed);
        assert!(quiet_mode());
        reset();
    }

    // 4 — quiet mode is off after reset
    #[test]
    fn quiet_mode_off_after_reset() {
        reset();
        assert!(!quiet_mode());
    }

    // 5 — fmt_speedup good path (no color)
    #[test]
    fn fmt_speedup_good_no_color() {
        reset();
        COLOR_ENABLED.store(false, Ordering::Relaxed);
        assert_eq!(fmt_speedup(3.5), "3.50x");
        reset();
    }

    // 6 — fmt_speedup mediocre path (no color)
    #[test]
    fn fmt_speedup_mediocre_no_color() {
        reset();
        COLOR_ENABLED.store(false, Ordering::Relaxed);
        assert_eq!(fmt_speedup(1.5), "1.50x");
        reset();
    }

    // 7 — fmt_speedup regression path (no color)
    #[test]
    fn fmt_speedup_regression_no_color() {
        reset();
        COLOR_ENABLED.store(false, Ordering::Relaxed);
        assert_eq!(fmt_speedup(0.9), "0.90x");
        reset();
    }

    // 8 — fmt_speedup exactly 1.0 (no color — edge of regression zone)
    #[test]
    fn fmt_speedup_one_no_color() {
        reset();
        COLOR_ENABLED.store(false, Ordering::Relaxed);
        assert_eq!(fmt_speedup(1.0), "1.00x");
        reset();
    }

    // 9 — fmt_speedup with color produces non-empty string
    #[test]
    fn fmt_speedup_with_color_nonempty() {
        reset();
        COLOR_ENABLED.store(true, Ordering::Relaxed);
        assert!(!fmt_speedup(4.0).is_empty());
        assert!(!fmt_speedup(1.3).is_empty());
        assert!(!fmt_speedup(0.8).is_empty());
        reset();
    }

    // 10 — styled_table no-color variant does not panic
    #[test]
    fn styled_table_no_color_no_panic() {
        reset();
        COLOR_ENABLED.store(false, Ordering::Relaxed);
        let t = styled_table();
        let _ = t.to_string();
        reset();
    }

    // 11 — styled_table color variant does not panic
    #[test]
    fn styled_table_color_no_panic() {
        reset();
        COLOR_ENABLED.store(true, Ordering::Relaxed);
        let t = styled_table();
        let _ = t.to_string();
        reset();
    }

    // 12 — banner contains the crate version string
    #[test]
    fn banner_contains_version() {
        reset();
        COLOR_ENABLED.store(false, Ordering::Relaxed);
        let b = banner();
        assert!(
            b.contains(env!("CARGO_PKG_VERSION")),
            "banner missing version: {b}"
        );
        reset();
    }

    // 13 — banner is non-empty in color mode
    #[test]
    fn banner_with_color_nonempty() {
        reset();
        COLOR_ENABLED.store(true, Ordering::Relaxed);
        assert!(!banner().is_empty());
        reset();
    }

    // 14 — init with no_color=true disables color (env-var-safe)
    #[test]
    fn init_no_color_flag_disables_color() {
        reset();
        if std::env::var_os("NO_COLOR").is_some() {
            // env already disables color; flag test is redundant but still valid
            init(true, false);
            assert!(!color_enabled());
            reset();
            return;
        }
        init(true, false);
        assert!(
            !color_enabled(),
            "color should be disabled after init(no_color=true)"
        );
        reset();
    }

    // 15 — init with quiet=true enables quiet mode
    #[test]
    fn init_quiet_flag_enables_quiet() {
        reset();
        let no_color_env = std::env::var_os("NO_COLOR").is_some();
        init(no_color_env, true);
        assert!(quiet_mode(), "quiet mode should be enabled");
        reset();
    }
}
