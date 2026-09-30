//! vv-pam-enroll — enroll a user for VectorVolt TOTP MFA
//!
//! Usage: sudo vv-pam-enroll <username>
//!        sudo vv-pam-enroll --regenerate-recovery-codes <username>
//!
//! Generates a TOTP secret, saves it to /etc/vectorvolt/totp/<username>.secret,
//! and prints the otpauth:// URI + QR code instructions.
//!
//! `--regenerate-recovery-codes` issues a fresh set of break-glass recovery
//! codes without touching the TOTP secret — use this after a user burns
//! through their original codes, or to push new codes to another node
//! (recovery codes are per-node, same as the TOTP secret; copy the resulting
//! `/etc/vectorvolt/totp/<user>.recovery` file to other nodes yourself if you
//! want the same codes valid cluster-wide).

fn print_recovery_codes(username: &str, codes: &[String]) {
    println!();
    println!("⚠ SAVE THESE RECOVERY CODES NOW — shown only once. Each works ONE");
    println!("  time if the user loses access to their authenticator app:");
    println!();
    for c in codes {
        println!("    {c}");
    }
    println!();
    println!("Any previously issued codes for {username} are now invalid.");
    println!("Store these somewhere safe (password manager, printed copy).");
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() == 3 && args[1] == "--regenerate-recovery-codes" {
        let username = &args[2];
        match vectorvolt_pam::generate_recovery_codes(username) {
            Ok(codes) => {
                print_recovery_codes(username, &codes);
            }
            Err(e) => {
                eprintln!("Error generating recovery codes: {e} (try running as root)");
                std::process::exit(1);
            }
        }
        return;
    }

    if args.len() != 2 {
        eprintln!("Usage: vv-pam-enroll <username>");
        eprintln!("       vv-pam-enroll --regenerate-recovery-codes <username>");
        std::process::exit(1);
    }
    let username = &args[1];

    let (b32, uri) = vectorvolt_pam::enroll_totp(username);

    match vectorvolt_pam::save_totp_secret(username, &b32) {
        Ok(_) => {}
        Err(e) => {
            eprintln!("Error saving secret: {} (try running as root)", e);
            std::process::exit(1);
        }
    }

    println!();
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║     VectorVolt MFA Enrollment — {}", username);
    println!("╠══════════════════════════════════════════════════════════════╣");
    println!("║                                                              ║");
    println!("║  1. Open Google Authenticator / Authy / any TOTP app        ║");
    println!("║  2. Tap '+' → 'Scan QR code' or 'Enter setup key'           ║");
    println!("║  3. Use the URI below or enter the secret manually           ║");
    println!("║                                                              ║");
    println!("╚══════════════════════════════════════════════════════════════╝");
    println!();
    println!("Secret (Base32): {}", b32);
    println!();
    println!("otpauth URI:");
    println!("{}", uri);
    println!();
    println!("To generate a QR code image, run:");
    println!("  qrencode -t ANSI '{}'", uri);
    println!("  or paste the URI at: https://www.qr-code-generator.com");
    println!();
    println!("✓ Secret saved to /etc/vectorvolt/totp/{}.secret", username);
    println!("  Next SSH login will require your 6-digit OTP.");

    match vectorvolt_pam::generate_recovery_codes(username) {
        Ok(codes) => print_recovery_codes(username, &codes),
        Err(e) => {
            eprintln!(
                "\nWarning: could not generate recovery codes ({e}) — no break-glass path exists for this user."
            );
        }
    }
}
