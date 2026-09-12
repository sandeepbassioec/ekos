//! vv-pam-enroll — enroll a user for VectorVolt TOTP MFA
//!
//! Usage: sudo vv-pam-enroll <username>
//!
//! Generates a TOTP secret, saves it to /etc/vectorvolt/totp/<username>.secret,
//! and prints the otpauth:// URI + QR code instructions.

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: vv-pam-enroll <username>");
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
}
