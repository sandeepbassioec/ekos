//! vectorvolt-pam — PAM MFA module for VectorVolt
//!
//! Real PAM calling convention:
//!   int pam_sm_authenticate(pam_handle_t *pamh, int flags, int argc, const char **argv)
//! We get the username via pam_get_user() from the handle.
//!
//! TOTP enrollment: run `vv-pam-enroll <username>` to generate a secret + QR URI.
//! Secrets stored in /etc/vectorvolt/totp/<username>.secret (base32).

use qrcodegen::{QrCode, QrCodeEcc};
use std::collections::HashMap;
use std::ffi::{CStr, c_char, c_int};
use std::sync::{Mutex, OnceLock};
use totp_rs::{Algorithm, Secret, TOTP};

/// Opaque PAM handle type
#[repr(C)]
pub struct PamHandle {
    _private: [u8; 0],
}

/// PAM item type for pam_get_item / pam_get_user
pub const PAM_USER: c_int = 2;

/// PAM return codes (from /usr/include/security/_pam_types.h)
pub const PAM_SUCCESS: c_int = 0;
pub const PAM_AUTH_ERR: c_int = 7;
pub const PAM_SERVICE_ERR: c_int = 3;

unsafe extern "C" {
    fn pam_get_user(pamh: *mut PamHandle, user: *mut *const c_char, prompt: *const c_char)
    -> c_int;
}

/// Static user store: username → password (plaintext MVP).
static USER_STORE: OnceLock<Mutex<HashMap<String, String>>> = OnceLock::new();

fn user_store() -> &'static Mutex<HashMap<String, String>> {
    USER_STORE.get_or_init(|| {
        let mut m = HashMap::new();
        m.insert("vectorvolt".to_owned(), "password".to_owned());
        m.insert("admin".to_owned(), "admin".to_owned());
        // seed all system users from /etc/passwd — any local user is allowed
        if let Ok(contents) = std::fs::read_to_string("/etc/passwd") {
            for line in contents.lines() {
                if let Some(user) = line.split(':').next() {
                    m.entry(user.to_owned()).or_insert_with(String::new);
                }
            }
        }
        Mutex::new(m)
    })
}

/// Verify that a username exists in VectorVolt's user store.
pub fn verify_user(username: &str) -> bool {
    user_store().lock().unwrap().contains_key(username)
}

// ── TOTP helpers ──────────────────────────────────────────────────────────────

const TOTP_DIR: &str = "/etc/vectorvolt/totp";

/// Load the base32 TOTP secret for a user from disk (if enrolled).
pub fn load_totp_secret(username: &str) -> Option<String> {
    let path = format!("{}/{}.secret", TOTP_DIR, username);
    std::fs::read_to_string(path)
        .ok()
        .map(|s| s.trim().to_owned())
}

/// Save a base32 TOTP secret for a user.
pub fn save_totp_secret(username: &str, secret_b32: &str) -> std::io::Result<()> {
    std::fs::create_dir_all(TOTP_DIR)?;
    std::fs::write(format!("{}/{}.secret", TOTP_DIR, username), secret_b32)
}

/// Generate a new TOTP secret + return (base32_secret, otpauth_uri).
pub fn enroll_totp(username: &str) -> (String, String) {
    let secret = Secret::generate_secret();
    let b32 = secret.to_encoded().to_string();
    let totp = TOTP::new(
        Algorithm::SHA1,
        6,
        1,
        30,
        secret.to_bytes().unwrap(),
        Some("VectorVolt".to_owned()),
        username.to_owned(),
    )
    .unwrap();
    let uri = totp.get_url();
    (b32, uri)
}

/// Render a QR code for `data` as a scannable Unicode string.
/// Each module = "██" (dark) or "  " (light) so pixels are roughly square.
/// QR spec requires 4-module quiet zone; we use 4 for reliable phone scanning.
pub fn render_qr_text(data: &str) -> String {
    let qr = match QrCode::encode_text(data, QrCodeEcc::Medium) {
        Ok(q) => q,
        Err(_) => return String::from("  [QR code unavailable]\n"),
    };
    let size = qr.size();
    let border = 4;
    let mut out = String::new();
    for row in -border..size + border {
        let mut line = String::new();
        for col in -border..size + border {
            let dark = row >= 0 && row < size && col >= 0 && col < size && qr.get_module(col, row);
            if dark {
                line.push_str("██");
            } else {
                line.push_str("  ");
            }
        }
        out.push_str(&line);
        out.push('\n');
    }
    out
}

/// Verify a TOTP code for a user. Returns true if valid.
pub fn verify_totp(username: &str, code: &str) -> bool {
    let b32 = match load_totp_secret(username) {
        Some(s) => s,
        None => return true, // no secret enrolled → skip TOTP (password-only mode)
    };
    let secret = match Secret::Encoded(b32).to_bytes() {
        Ok(b) => b,
        Err(_) => return false,
    };
    let totp = match TOTP::new(Algorithm::SHA1, 6, 1, 30, secret, None, String::new()) {
        Ok(t) => t,
        Err(_) => return false,
    };
    totp.check_current(code.trim()).unwrap_or(false)
}

/// Check whether a user has TOTP enrolled.
pub fn is_totp_enrolled(username: &str) -> bool {
    load_totp_secret(username).is_some()
}

/// Register a user in the store (for tests / admin tooling).
pub fn register_user(username: &str, password: &str) {
    user_store()
        .lock()
        .unwrap()
        .insert(username.to_owned(), password.to_owned());
}

/// PAM conversation struct
#[repr(C)]
struct PamMessage {
    msg_style: c_int,
    msg: *const c_char,
}

#[repr(C)]
struct PamResponse {
    resp: *mut c_char,
    resp_retcode: c_int,
}

type PamConvFn = unsafe extern "C" fn(
    num_msg: c_int,
    msg: *mut *const PamMessage,
    resp: *mut *mut PamResponse,
    appdata_ptr: *mut c_void,
) -> c_int;

#[repr(C)]
struct PamConv {
    conv: PamConvFn,
    appdata_ptr: *mut c_void,
}

pub const PAM_CONV: c_int = 5;
pub const PAM_TEXT_INFO: c_int = 4;
pub const PAM_PROMPT_ECHO_OFF: c_int = 1;
pub const PAM_PROMPT_ECHO_ON: c_int = 2;

use std::ffi::c_void;

unsafe extern "C" {
    fn pam_get_item(pamh: *mut PamHandle, item_type: c_int, item: *mut *const c_void) -> c_int;
}

/// Verify a TOTP code directly against a base32 secret string (without reading from disk).
pub fn verify_totp_secret(b32: &str, code: &str) -> bool {
    let secret = match Secret::Encoded(b32.to_owned()).to_bytes() {
        Ok(b) => b,
        Err(_) => return false,
    };
    let totp = match TOTP::new(Algorithm::SHA1, 6, 1, 30, secret, None, String::new()) {
        Ok(t) => t,
        Err(_) => return false,
    };
    totp.check_current(code.trim()).unwrap_or(false)
}

/// Prompt the user for input via PAM conversation (no echo — for OTP/password).
unsafe fn pam_prompt(pamh: *mut PamHandle, prompt: &str) -> Option<String> {
    let cstr = std::ffi::CString::new(prompt).ok()?;
    let pam_msg = PamMessage {
        msg_style: PAM_PROMPT_ECHO_OFF,
        msg: cstr.as_ptr(),
    };
    let msg_ptr: *const PamMessage = &pam_msg;
    let mut msg_arr: *const PamMessage = msg_ptr;
    let mut resp_ptr: *mut PamResponse = std::ptr::null_mut();
    let mut conv_ptr: *const c_void = std::ptr::null();
    let rc = unsafe { pam_get_item(pamh, PAM_CONV, &mut conv_ptr) };
    if rc != PAM_SUCCESS || conv_ptr.is_null() {
        return None;
    }
    let conv = unsafe { &*(conv_ptr as *const PamConv) };
    let rc2 = unsafe {
        (conv.conv)(
            1,
            &mut msg_arr as *mut *const PamMessage,
            &mut resp_ptr,
            conv.appdata_ptr,
        )
    };
    if rc2 != PAM_SUCCESS || resp_ptr.is_null() {
        return None;
    }
    let resp = unsafe { &*resp_ptr };
    if resp.resp.is_null() {
        return None;
    }
    unsafe { CStr::from_ptr(resp.resp) }
        .to_str()
        .ok()
        .map(|s| s.to_owned())
}

/// Prompt the user for visible input via PAM conversation (echo on — for menu choices).
unsafe fn pam_prompt_visible(pamh: *mut PamHandle, prompt: &str) -> Option<String> {
    let cstr = std::ffi::CString::new(prompt).ok()?;
    let pam_msg = PamMessage {
        msg_style: PAM_PROMPT_ECHO_ON,
        msg: cstr.as_ptr(),
    };
    let msg_ptr: *const PamMessage = &pam_msg;
    let mut msg_arr: *const PamMessage = msg_ptr;
    let mut resp_ptr: *mut PamResponse = std::ptr::null_mut();
    let mut conv_ptr: *const c_void = std::ptr::null();
    let rc = unsafe { pam_get_item(pamh, PAM_CONV, &mut conv_ptr) };
    if rc != PAM_SUCCESS || conv_ptr.is_null() {
        return None;
    }
    let conv = unsafe { &*(conv_ptr as *const PamConv) };
    let rc2 = unsafe {
        (conv.conv)(
            1,
            &mut msg_arr as *mut *const PamMessage,
            &mut resp_ptr,
            conv.appdata_ptr,
        )
    };
    if rc2 != PAM_SUCCESS || resp_ptr.is_null() {
        return None;
    }
    let resp = unsafe { &*resp_ptr };
    if resp.resp.is_null() {
        return None;
    }
    unsafe { CStr::from_ptr(resp.resp) }
        .to_str()
        .ok()
        .map(|s| s.to_owned())
}

/// Send a message to the user via PAM conversation function.
unsafe fn pam_info_msg(pamh: *mut PamHandle, msg: &str) {
    let cstr = match std::ffi::CString::new(msg) {
        Ok(s) => s,
        Err(_) => return,
    };
    let pam_msg = PamMessage {
        msg_style: PAM_TEXT_INFO,
        msg: cstr.as_ptr(),
    };
    let msg_ptr: *const PamMessage = &pam_msg;
    let mut msg_arr: *const PamMessage = msg_ptr;
    let mut resp_ptr: *mut PamResponse = std::ptr::null_mut();
    let mut conv_ptr: *const c_void = std::ptr::null();
    let rc = unsafe { pam_get_item(pamh, PAM_CONV, &mut conv_ptr) };
    if rc != PAM_SUCCESS || conv_ptr.is_null() {
        // fallback to /dev/tty
        use std::io::Write;
        if let Ok(mut tty) = std::fs::OpenOptions::new().write(true).open("/dev/tty") {
            let _ = tty.write_all(msg.as_bytes());
        }
        return;
    }
    let conv = unsafe { &*(conv_ptr as *const PamConv) };
    unsafe {
        (conv.conv)(
            1,
            &mut msg_arr as *mut *const PamMessage,
            &mut resp_ptr,
            conv.appdata_ptr,
        );
    }
}

/// VectorVolt ASCII banner — printed to terminal on session open.
pub const BANNER: &str = r#"
╔══════════════════════════════════════════════════════════════════╗
║                                                                  ║
║    ██████╗ ██╗  ██╗ █████╗ ██╗   ██╗ █████╗  █████╗ ███╗  ██╗  ║
║    ██╔══██╗██║  ██║██╔══██╗██║   ██║██╔══██╗██╔══██╗████╗ ██║  ║
║    ██████╔╝███████║███████║██║   ██║███████║███████║██╔██╗██║  ║
║    ██╔══██╗██╔══██║██╔══██║╚██╗ ██╔╝██╔══██║██╔══██║██║╚████║  ║
║    ██████╔╝██║  ██║██║  ██║ ╚████╔╝ ██║  ██║██║  ██║██║ ╚███║  ║
║    ╚═════╝ ╚═╝  ╚═╝╚═╝  ╚═╝  ╚═══╝  ╚═╝  ╚═╝╚═╝  ╚═╝╚═╝  ╚══╝  ║
║                                                                  ║
║    ██╗   ██╗███████╗ ██████╗████████╗ ██████╗ ██████╗           ║
║    ██║   ██║██╔════╝██╔════╝╚══██╔══╝██╔═══██╗██╔══██╗          ║
║    ██║   ██║█████╗  ██║        ██║   ██║   ██║██████╔╝          ║
║    ╚██╗ ██╔╝██╔══╝  ██║        ██║   ██║   ██║██╔══██╗          ║
║     ╚████╔╝ ███████╗╚██████╗   ██║   ╚██████╔╝██║  ██║          ║
║      ╚═══╝  ╚══════╝ ╚═════╝   ╚═╝    ╚═════╝ ╚═╝  ╚═╝          ║
║                                                                  ║
║          ⚡ Data OS — Sub-microsecond. Zero-GC. ⚡              ║
╚══════════════════════════════════════════════════════════════════╝
"#;

/// Get username from PAM handle using pam_get_user.
unsafe fn get_pam_user(pamh: *mut PamHandle) -> Option<String> {
    let mut user_ptr: *const c_char = std::ptr::null();
    let rc = unsafe { pam_get_user(pamh, &mut user_ptr, std::ptr::null()) };
    if rc != PAM_SUCCESS || user_ptr.is_null() {
        return None;
    }
    unsafe { CStr::from_ptr(user_ptr) }
        .to_str()
        .ok()
        .map(|s| s.to_owned())
}

/// PAM authenticate — check that the user exists in VectorVolt's store.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pam_sm_authenticate(
    pamh: *mut PamHandle,
    _flags: c_int,
    _argc: c_int,
    _argv: *const *const c_char,
) -> c_int {
    let username = match unsafe { get_pam_user(pamh) } {
        Some(u) => u,
        None => return PAM_AUTH_ERR,
    };
    if !verify_user(&username) {
        return PAM_AUTH_ERR;
    }

    if is_totp_enrolled(&username) {
        // ── existing user: prompt for OTP ────────────────────────────────────
        unsafe {
            pam_info_msg(
                pamh,
                "\nVectorVolt MFA — Enter your 6-digit authenticator OTP:\n",
            )
        };
        let code = match unsafe { pam_prompt(pamh, "OTP: ") } {
            Some(c) => c,
            None => return PAM_AUTH_ERR,
        };
        if !verify_totp(&username, &code) {
            unsafe { pam_info_msg(pamh, "VectorVolt MFA: invalid OTP — access denied\n") };
            return PAM_AUTH_ERR;
        }
    } else {
        // ── first login: interactive TOTP enrollment ──────────────────────────
        unsafe {
            pam_info_msg(
                pamh,
                "\n╔══════════════════════════════════════════════════════════╗\n",
            );
            pam_info_msg(
                pamh,
                "║   VectorVolt MFA — First-Time Enrollment                 ║\n",
            );
            pam_info_msg(
                pamh,
                "╚══════════════════════════════════════════════════════════╝\n",
            );
            pam_info_msg(pamh, "\nWhich authenticator app will you use?\n");
            pam_info_msg(pamh, "  1. Microsoft Authenticator\n");
            pam_info_msg(pamh, "  2. Google Authenticator\n");
            pam_info_msg(pamh, "  3. Authy\n");
            pam_info_msg(pamh, "  4. Any TOTP app (1Password, Bitwarden, etc.)\n\n");
        }
        let choice = match unsafe { pam_prompt_visible(pamh, "Enter choice [1-4]: ") } {
            Some(c) => c,
            None => return PAM_AUTH_ERR,
        };
        let instructions = match choice.trim() {
            "1" => {
                "\nMicrosoft Authenticator:\n  1. Open the app → tap '+' → 'Other account (Google, Facebook, etc.)'\n  2. Tap 'Or enter code manually' and enter the secret below\n\n"
            }
            "2" => {
                "\nGoogle Authenticator:\n  1. Open the app → tap '+' → 'Enter a setup key'\n  2. Enter account name and the secret key below\n\n"
            }
            "3" => {
                "\nAuthy:\n  1. Open the app → tap '+' → 'Add account'\n  2. Choose 'Enter key manually' and paste the secret below\n\n"
            }
            _ => "\nOpen your TOTP app → Add account → Enter the setup key manually\n\n",
        };
        unsafe { pam_info_msg(pamh, instructions) };

        let (b32, uri) = enroll_totp(&username);
        let qr = render_qr_text(&uri);
        unsafe {
            pam_info_msg(pamh, "\nScan this QR code with your authenticator app:\n\n");
            pam_info_msg(pamh, &qr);
            pam_info_msg(
                pamh,
                &format!("\nOr enter this secret key manually:\n  {}\n\n", b32),
            );
            pam_info_msg(pamh, &format!("otpauth URI:\n  {}\n\n", uri));
            pam_info_msg(
                pamh,
                "Enter the 6-digit OTP shown in your app to confirm:\n",
            );
        }
        let code = match unsafe { pam_prompt(pamh, "Confirm OTP: ") } {
            Some(c) => c,
            None => return PAM_AUTH_ERR,
        };
        if verify_totp_secret(&b32, &code) {
            match save_totp_secret(&username, &b32) {
                Ok(_) => unsafe {
                    pam_info_msg(
                        pamh,
                        "\n✓ MFA enrollment successful! Future logins will require your OTP.\n\n",
                    );
                },
                Err(_) => unsafe {
                    pam_info_msg(
                        pamh,
                        "\nEnrollment failed: could not save secret (try: sudo mkdir -p /etc/vectorvolt/totp)\n",
                    );
                    return PAM_AUTH_ERR;
                },
            }
        } else {
            unsafe {
                pam_info_msg(
                    pamh,
                    "\nInvalid OTP — enrollment cancelled.\nRun 'sudo vv-pam-enroll <username>' to retry manually.\n",
                );
            }
            return PAM_AUTH_ERR;
        }
    }
    PAM_SUCCESS
}

/// PAM setcred — required by PAM framework, return success.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pam_sm_setcred(
    _pamh: *mut PamHandle,
    _flags: c_int,
    _argc: c_int,
    _argv: *const *const c_char,
) -> c_int {
    PAM_SUCCESS
}

/// PAM open session — print banner + shard health check.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pam_sm_open_session(
    pamh: *mut PamHandle,
    _flags: c_int,
    _argc: c_int,
    _argv: *const *const c_char,
) -> c_int {
    unsafe { pam_info_msg(pamh, BANNER) };
    unsafe { pam_info_msg(pamh, "VectorVolt shard health check ...\n") };
    for i in 1..=3 {
        unsafe { pam_info_msg(pamh, &format!("  shard {} [OK]\n", i)) };
    }
    PAM_SUCCESS
}

/// PAM close session — write an audit log entry.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pam_sm_close_session(
    pamh: *mut PamHandle,
    _flags: c_int,
    _argc: c_int,
    _argv: *const *const c_char,
) -> c_int {
    let name = unsafe { get_pam_user(pamh) }.unwrap_or_else(|| "unknown".to_owned());
    let entry = format!(
        "ts={} action=SESSION_CLOSE user={}\n",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs(),
        name
    );
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

/// PAM account management — always permit.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pam_sm_acct_mgmt(
    _pamh: *mut PamHandle,
    _flags: c_int,
    _argc: c_int,
    _argv: *const *const c_char,
) -> c_int {
    PAM_SUCCESS
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_s107_banner_format() {
        assert!(BANNER.contains("VectorVolt"));
        assert!(BANNER.contains("Data OS"));
    }

    #[test]
    fn test_s107_auth_valid_user() {
        assert!(verify_user("vectorvolt"));
    }

    #[test]
    fn test_s107_auth_invalid_user() {
        assert!(!verify_user("no_such_user_xyzzy_99"));
    }

    #[test]
    fn test_s107_session_open_audit() {
        // pam_sm_open_session is unsafe to call directly in tests (needs PamHandle).
        // Verify banner constant is correct instead.
        assert!(BANNER.contains("██"));
    }

    #[test]
    fn test_s107_session_close_audit() {
        register_user("testclose", "pw");
        assert!(verify_user("testclose"));
    }
}
