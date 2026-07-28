# MFA Session Mandate
## VectorVolt + EKOS — SSH MFA Enforcement at Binary Level

**Status:** Architecture Capture (implement at Phase 2 production hardening)  
**Captured:** 2026-07-07  
**Applies to:** `ekosd`, `vvctl`, and any future VectorVolt/EKOS binaries on Linux

---

## 1. Requirement

> Any Linux server running VectorVolt or EKOS binaries must enforce SSH login via TOTP/MFA.  
> If a session was not authenticated with MFA, the binary refuses to start and prints a clear mandate message.  
> If a running binary detects its session's MFA has expired or been revoked, it shuts down gracefully.

This protection is **baked into the binary itself** — not enforced by external tooling, systemd units, or ops procedures that can be bypassed.

---

## 2. How Linux Knows About MFA — PAM

SSH on Linux uses PAM (Pluggable Authentication Modules) for authentication.  
`pam_google_authenticator` (or `pam_oath`) handles TOTP.

```
User SSH → sshd → PAM stack:
  pam_unix        (password / key check)
  pam_google_authenticator  (TOTP check)
  pam_exec        (runs our hook script on success)  ← we add this
  pam_systemd     (creates session)
```

When PAM completes successfully (all modules passed including TOTP), `pam_exec` runs a script we control. That script writes a **signed session token** to tmpfs. Our binary reads and verifies it.

---

## 3. Session Token Design

### Token Location

```
/run/ekos-auth/<uid>/<pam_session_id>.token
```

- `/run/` is tmpfs — survives only until reboot, never touches disk
- Per-uid subdirectory — one user cannot read another's token
- Per-session filename — each SSH session gets its own token

### Token Format

```
token = base64( uid || session_id || issued_at_unix || valid_until_unix || HMAC-SHA256(above, SERVER_SECRET) )
```

Fields:
- `uid` — Linux user ID (u32)
- `session_id` — PAM session ID from `$PAM_SESSION_SERVICE` (32 bytes)
- `issued_at_unix` — Unix timestamp when token was written
- `valid_until_unix` — `issued_at + session_ttl` (default: 8 hours)
- `HMAC-SHA256` — keyed with `SERVER_SECRET` (32-byte key stored at `/etc/ekos/session.key`, root-only read)

### SERVER_SECRET

```bash
# Generated once during server setup, root-only
install -m 600 -o root /dev/null /etc/ekos/session.key
dd if=/dev/urandom bs=32 count=1 | base64 > /etc/ekos/session.key
```

Only `root` and the `ekos` service user (via group) can read it.  
The PAM exec script runs as root — it can read the key.  
The Rust binary runs as the `ekos` user — it can also read the key.

---

## 4. PAM Configuration

### /etc/pam.d/sshd (additions)

```pam
# Existing TOTP line (already present if pam_google_authenticator installed):
auth required pam_google_authenticator.so

# Add AFTER the TOTP line — runs our token writer on session open:
session optional pam_exec.so /usr/local/bin/ekos-mfa-token-writer
```

### /usr/local/bin/ekos-mfa-token-writer

```bash
#!/bin/bash
# Called by PAM after successful MFA login.
# Writes a signed session token to tmpfs.
# Environment provided by PAM: PAM_USER, PAM_SERVICE, PAM_TYPE

set -euo pipefail

[[ "$PAM_TYPE" != "open_session" ]] && exit 0

UID_VAL=$(id -u "$PAM_USER")
SESSION_ID=$(cat /proc/self/sessionid 2>/dev/null || echo "unknown-$$")
TOKEN_DIR="/run/ekos-auth/$UID_VAL"
SERVER_SECRET=$(cat /etc/ekos/session.key)

ISSUED=$(date +%s)
VALID_UNTIL=$(( ISSUED + 28800 ))   # 8 hours

PAYLOAD="${UID_VAL}|${SESSION_ID}|${ISSUED}|${VALID_UNTIL}"
HMAC=$(echo -n "$PAYLOAD" | openssl dgst -sha256 -hmac "$SERVER_SECRET" -binary | base64)
TOKEN=$(echo "${PAYLOAD}|${HMAC}" | base64)

install -m 700 -o "$PAM_USER" -d "$TOKEN_DIR"
echo "$TOKEN" > "${TOKEN_DIR}/${SESSION_ID}.token"
chmod 600 "${TOKEN_DIR}/${SESSION_ID}.token"
```

---

## 5. Rust Binary Implementation

### `ekos-security` crate (shared by ekosd + vvctl)

```rust
// ekos/crates/ekos-security/src/mfa.rs

use std::time::{SystemTime, UNIX_EPOCH};
use hmac::{Hmac, Mac};
use sha2::Sha256;

#[derive(Debug, thiserror::Error)]
pub enum MfaError {
    #[error("Security mandate not followed: SSH session requires MFA/TOTP authentication.\n\
             Connect via SSH with TOTP enabled to use EKOS on this server.")]
    MfaRequired,

    #[error("Security mandate not followed: MFA session has expired. Re-authenticate via SSH.")]
    SessionExpired,

    #[error("Security mandate not followed: Session token is invalid or tampered.")]
    InvalidToken,

    #[error("Security mandate not followed: Cannot determine current session identity.")]
    SessionUnknown,
}

pub struct MfaGuard {
    valid_until: u64,
    token_path: std::path::PathBuf,
}

impl MfaGuard {
    /// Call at binary startup. Returns Err if MFA was not performed.
    pub fn verify_startup() -> Result<Self, MfaError> {
        let uid = unsafe { libc::getuid() };
        let session_id = Self::current_session_id()?;
        let token_path = format!("/run/ekos-auth/{}/{}.token", uid, session_id);
        let server_secret = std::fs::read_to_string("/etc/ekos/session.key")
            .map_err(|_| MfaError::MfaRequired)?;

        let raw = std::fs::read_to_string(&token_path)
            .map_err(|_| MfaError::MfaRequired)?;

        let decoded = base64::decode(raw.trim())
            .map_err(|_| MfaError::InvalidToken)?;

        let parts: Vec<&str> = std::str::from_utf8(&decoded)
            .map_err(|_| MfaError::InvalidToken)?
            .split('|')
            .collect();

        if parts.len() != 5 {
            return Err(MfaError::InvalidToken);
        }

        let payload = format!("{}|{}|{}|{}", parts[0], parts[1], parts[2], parts[3]);
        let claimed_hmac = parts[4];
        Self::verify_hmac(&payload, &server_secret, claimed_hmac)?;

        let valid_until: u64 = parts[3].parse().map_err(|_| MfaError::InvalidToken)?;
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH).unwrap().as_secs();

        if now > valid_until {
            return Err(MfaError::SessionExpired);
        }

        Ok(MfaGuard {
            valid_until,
            token_path: token_path.into(),
        })
    }

    /// Call periodically (e.g. every 5 minutes) to detect expired/revoked sessions.
    pub fn recheck(&self) -> Result<(), MfaError> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH).unwrap().as_secs();

        // Token file deleted = session revoked by admin
        if !self.token_path.exists() {
            return Err(MfaError::MfaRequired);
        }

        if now > self.valid_until {
            return Err(MfaError::SessionExpired);
        }

        Ok(())
    }

    fn current_session_id() -> Result<String, MfaError> {
        // Read from /proc/self/sessionid
        std::fs::read_to_string("/proc/self/sessionid")
            .map(|s| s.trim().to_string())
            .map_err(|_| MfaError::SessionUnknown)
    }

    fn verify_hmac(payload: &str, secret: &str, claimed: &str) -> Result<(), MfaError> {
        type HmacSha256 = Hmac<Sha256>;
        let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
            .map_err(|_| MfaError::InvalidToken)?;
        mac.update(payload.as_bytes());
        let computed = base64::encode(mac.finalize().into_bytes());
        if computed != claimed {
            return Err(MfaError::InvalidToken);
        }
        Ok(())
    }
}
```

### Integration in ekosd main.rs

```rust
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // MFA check is FIRST — before loading config, before opening KoStore
    let mfa_guard = ekos_security::MfaGuard::verify_startup()
        .map_err(|e| {
            eprintln!("\n{}\n", e);
            std::process::exit(1);
        })?;

    // Spawn periodic recheck — graceful shutdown if session expires mid-run
    let guard_clone = mfa_guard.clone();
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_secs(300)).await;  // 5 min
            if let Err(e) = guard_clone.recheck() {
                eprintln!("\n[EKOS] {}\nInitiating graceful shutdown.\n", e);
                std::process::exit(1);
            }
        }
    });

    // Normal startup continues...
    let config = Config::load()?;
    // ...
}
```

### Integration in vvctl main.rs

```rust
fn main() -> anyhow::Result<()> {
    // Same check — vvctl also refuses without valid MFA session
    ekos_security::MfaGuard::verify_startup()
        .map_err(|e| {
            eprintln!("\n{}\n", e);
            std::process::exit(1);
        })?;

    // Normal CLI parsing continues...
}
```

---

## 6. What the User Sees

**Without MFA:**
```
$ ekosd --config /etc/ekos/ekosd.toml

Security mandate not followed: SSH session requires MFA/TOTP authentication.
Connect via SSH with TOTP enabled to use EKOS on this server.
```
Exit code: 1. No further output. No partial startup.

**With MFA, session expired mid-run:**
```
[EKOS] Security mandate not followed: MFA session has expired. Re-authenticate via SSH.
Initiating graceful shutdown.
```
ekosd performs graceful shutdown (drains in-flight requests, flushes WAL, closes KoStore). Then exits.

**Admin revokes session (deletes token file):**
```
[EKOS] Security mandate not followed: SSH session requires MFA/TOTP authentication.
Initiating graceful shutdown.
```
Same graceful path. Admin can force-shutdown any session by deleting its token file from `/run/ekos-auth/<uid>/`.

---

## 7. Admin Operations

### Force-revoke a session
```bash
# Find all active ekos sessions for uid 1001
ls /run/ekos-auth/1001/

# Revoke specific session (ekosd detects within 5 minutes)
rm /run/ekos-auth/1001/<session_id>.token

# Revoke all sessions for a user (emergency)
rm -rf /run/ekos-auth/1001/
```

### Extend a session (re-issue token)
```bash
# Re-run the token writer manually as root
PAM_USER=ekosuser PAM_TYPE=open_session /usr/local/bin/ekos-mfa-token-writer
```

### Disable on non-SSH environments (e.g., systemd service on dedicated server)
```toml
# ekosd.toml
[security]
mfa_enforcement = "disabled"   # or "enabled" (default) or "warn-only"
mfa_token_path  = "/run/ekos-auth"
session_recheck_interval_secs = 300
```

`disabled` is valid for dedicated servers where no human SSH access exists (e.g., a cloud instance that only accepts connections from an internal service mesh with mutual TLS). The config is explicit — not a silent bypass.

---

## 8. Crate Structure

```
ekos/crates/ekos-security/
  src/
    lib.rs
    mfa.rs        ← MfaGuard (above)
    session.rs    ← session ID detection, token path resolution
  Cargo.toml
```

```toml
[dependencies]
hmac      = "0.12"
sha2      = "0.10"
base64    = "0.22"
libc      = "0.2"
thiserror = "2.0"
```

Used by both `ekosd` and `vvctl` (and any future binary).

---

## 9. What This Does NOT Cover

- **Non-SSH access** (e.g., systemd service on a locked-down server) — use `mfa_enforcement = "disabled"` with explicit config, secured by mTLS at the network layer instead
- **Windows** — PAM is Linux-only; Windows equivalent is a separate requirement (Smart Card / Windows Hello for Business)
- **Container environments** — if EKOS runs in a container, the PAM approach doesn't apply; use Kubernetes admission webhooks + service account tokens instead (future requirement)

---

## 10. Phase Placement

```
Phase 1:    Done ✅ (no security hardening required for local dev)
Phase 2:    Add ekos-security crate + MfaGuard
            Integrate into ekosd startup
            Integrate into vvctl startup
            Write PAM exec script + installation docs
Phase 3+:   MFA guard is always-on in all new binaries
```

---

*This is a binary-level mandate — not a deployment policy.*  
*The binary refuses to run. There is no ops workaround short of modifying the binary.*
