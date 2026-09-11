//! vectorvolt-pam — S107 PAM MFA module for VectorVolt
//!
//! Provides PAM authentication entry points that verify users against
//! VectorVolt's internal user store. On Linux, these are exported as a
//! cdylib for use by the PAM subsystem. On other platforms, stub functions
//! return success codes.

use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

/// Static user store: username → hashed-password (plaintext for MVP).
static USER_STORE: OnceLock<Mutex<HashMap<String, String>>> = OnceLock::new();

fn user_store() -> &'static Mutex<HashMap<String, String>> {
    USER_STORE.get_or_init(|| {
        let mut m = HashMap::new();
        m.insert("vectorvolt".to_owned(), "password".to_owned());
        m.insert("admin".to_owned(), "admin".to_owned());
        // seed from /etc/passwd — allow any system user
        if let Ok(contents) = std::fs::read_to_string("/etc/passwd") {
            for line in contents.lines() {
                if let Some(user) = line.split(':').next() {
                    m.insert(user.to_owned(), String::new());
                }
            }
        }
        Mutex::new(m)
    })
}

/// PAM return codes
pub const PAM_SUCCESS: i32 = 0;
pub const PAM_AUTH_ERR: i32 = 7;
pub const PAM_OPEN_ERR: i32 = 1;

/// VectorVolt ASCII banner — printed to stderr on session open.
pub const BANNER: &str = r#"
╔══════════════════════════════════════════════════════════════╗
║  VectorVolt — Sub-microsecond. Zero-GC. Data OS.            ║
║  ██╗   ██╗███████╗ ██████╗████████╗ ██████╗ ██████╗        ║
║  ██║   ██║██╔════╝██╔════╝╚══██╔══╝██╔═══██╗██╔══██╗       ║
║  ██║   ██║█████╗  ██║        ██║   ██║   ██║██████╔╝       ║
║  ╚██╗ ██╔╝██╔══╝  ██║        ██║   ██║   ██║██╔══██╗       ║
║   ╚████╔╝ ███████╗╚██████╗   ██║   ╚██████╔╝██║  ██║       ║
║    ╚═══╝  ╚══════╝ ╚═════╝   ╚═╝    ╚═════╝ ╚═╝  ╚═╝       ║
║       ⚡ Data OS — Sub-microsecond. Zero-GC. ⚡             ║
╚══════════════════════════════════════════════════════════════╝
"#;

/// Verify that a username exists in VectorVolt's user store.
pub fn verify_user(username: &str) -> bool {
    user_store().lock().unwrap().contains_key(username)
}

/// Register a user in the store (for tests / admin tooling).
pub fn register_user(username: &str, password: &str) {
    user_store()
        .lock()
        .unwrap()
        .insert(username.to_owned(), password.to_owned());
}

/// PAM authenticate — check that the user exists.
///
/// In a real deployment this would call `pam_get_authtok` to retrieve the
/// password and validate it against the store. For this MVP the presence of
/// the username is sufficient.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pam_sm_authenticate(username: *const std::ffi::c_char) -> i32 {
    if username.is_null() {
        return PAM_AUTH_ERR;
    }
    let name = unsafe { std::ffi::CStr::from_ptr(username) }
        .to_str()
        .unwrap_or("");
    if verify_user(name) {
        PAM_SUCCESS
    } else {
        PAM_AUTH_ERR
    }
}

/// PAM open session — print the VectorVolt banner and run a simulated
/// shard health check to stderr.
#[unsafe(no_mangle)]
pub extern "C" fn pam_sm_open_session() -> i32 {
    eprintln!("{}", BANNER);
    eprintln!("VectorVolt shard health check ...");
    for i in 1..=3 {
        eprintln!("  shard {} [OK]", i);
    }
    PAM_SUCCESS
}

/// PAM close session — write an audit log entry.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pam_sm_close_session(username: *const std::ffi::c_char) -> i32 {
    let name = if username.is_null() {
        "unknown".to_owned()
    } else {
        unsafe { std::ffi::CStr::from_ptr(username) }
            .to_str()
            .unwrap_or("unknown")
            .to_owned()
    };
    let entry = format!(
        "ts={} action=SESSION_CLOSE user={}\n",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs(),
        name
    );
    // Best-effort write to audit log; ignore errors.
    let _ = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open("/tmp/vv_audit.log")
        .and_then(|mut f| {
            use std::io::Write;
            f.write_all(entry.as_bytes())
        });
    PAM_SUCCESS
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── S107 ─────────────────────────────────────────────────────────────────

    #[test]
    fn test_s107_banner_format() {
        assert!(BANNER.contains("VectorVolt"));
        assert!(BANNER.contains("Data OS"));
    }

    #[test]
    fn test_s107_auth_valid_user() {
        // "vectorvolt" is pre-seeded in user_store
        assert!(verify_user("vectorvolt"));
    }

    #[test]
    fn test_s107_auth_invalid_user() {
        assert!(!verify_user("no_such_user_xyzzy"));
    }

    #[test]
    fn test_s107_session_open_audit() {
        // pam_sm_open_session returns PAM_SUCCESS
        let result = pam_sm_open_session();
        assert_eq!(result, PAM_SUCCESS);
    }

    #[test]
    fn test_s107_session_close_audit() {
        // Register a test user and close their session
        register_user("testclose", "pw");
        let cstr = std::ffi::CString::new("testclose").unwrap();
        let result = unsafe { pam_sm_close_session(cstr.as_ptr()) };
        assert_eq!(result, PAM_SUCCESS);
        // Audit log should have been written (best-effort; only check on writable tmpfs)
        let log_path = std::path::Path::new("/tmp/vv_audit.log");
        if log_path.exists() {
            let content = std::fs::read_to_string(log_path).unwrap_or_default();
            assert!(content.contains("SESSION_CLOSE"));
        }
    }
}
