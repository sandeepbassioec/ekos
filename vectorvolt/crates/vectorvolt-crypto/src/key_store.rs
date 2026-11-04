//! VV-SEC-001 / PP-55: Master key storage with OS-level memory protection.
//!
//! The key lives in exactly one place for the lifetime of a [`KeyStore`]:
//!
//! | Platform | Backing | Properties |
//! |---|---|---|
//! | Linux ≥5.14 | `memfd_secret()` mapping | excluded from the kernel direct map; `/proc/<pid>/mem`, ptrace and core dumps cannot read it; never swapped |
//! | other unix / old Linux | private anonymous page, `mlock`ed (+`MADV_DONTDUMP` on Linux) | not swapped, not in core dumps (Linux) |
//! | Windows | `VirtualAlloc` page, `VirtualLock`ed | not paged out while the process runs |
//! | anything else / allocation failure | heap `Box` | zeroized on drop only |
//!
//! [`KeyStore::as_bytes`] borrows directly from that backing — there is no
//! second, long-lived heap copy. Failure to lock (e.g. `RLIMIT_MEMLOCK`
//! exhausted, working-set quota on Windows) is logged once per process and
//! the key is kept unlocked; it is never fatal.
//!
//! The backing is wiped with volatile writes (`zeroize`) before release on
//! every path.
//!
//! # Nonce accounting
//!
//! AES-256-GCM with random 96-bit nonces stays within NIST's 2^-32 collision
//! bound for roughly 2^32 encryptions per key. Every [`KeyStore`] carries an
//! atomic usage counter. Encryptors should call [`KeyStore::record_use`] once
//! per AES-GCM seal (i.e. once per random nonce drawn) and consult
//! [`KeyStore::needs_rotation`]; when it returns `true` the owner should
//! rotate via [`crate::KeyRing::rotate`]. The default threshold is
//! [`DEFAULT_ROTATION_THRESHOLD`] (2^31), leaving a 2x safety margin.
//!
//! The counter is per process: it is not persisted, so a restart resets it.
//! Persisting it alongside the key (or rotating on restart) is the owner's
//! responsibility.

use crate::error::{CryptoError, CryptoResult};
use sha3::{Digest, Sha3_256};
use std::sync::atomic::{AtomicU64, Ordering};
use zeroize::Zeroize;

pub const KEY_LEN: usize = 32; // AES-256

/// Length of a [`KeyId`] in bytes.
pub const KEY_ID_LEN: usize = 8;

/// Public, non-secret identifier of a key: the first 8 bytes of
/// `SHA3-256("vectorvolt-key-id-v1" || key)`. Stored in file / WAL headers so
/// a reader can pick the right key out of a [`crate::KeyRing`].
pub type KeyId = [u8; KEY_ID_LEN];

/// Domain-separation prefix for [`key_id_of`].
pub const KEY_ID_DOMAIN: &[u8] = b"vectorvolt-key-id-v1";

/// Default number of AES-GCM encryptions after which
/// [`KeyStore::needs_rotation`] reports `true` (2^31).
pub const DEFAULT_ROTATION_THRESHOLD: u64 = 1 << 31;

/// Derive the [`KeyId`] for raw key bytes:
/// `SHA3-256("vectorvolt-key-id-v1" || key)[..8]`.
pub fn key_id_of(key: &[u8; KEY_LEN]) -> KeyId {
    let mut h = Sha3_256::new();
    h.update(KEY_ID_DOMAIN);
    h.update(key);
    let digest = h.finalize();
    let mut id = [0u8; KEY_ID_LEN];
    id.copy_from_slice(&digest[..KEY_ID_LEN]);
    id
}

/// How the key bytes of a [`KeyStore`] are protected in memory.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyProtection {
    /// Linux `memfd_secret` mapping (strongest).
    MemfdSecret,
    /// Dedicated page pinned with `mlock` / `VirtualLock`.
    Locked,
    /// Dedicated page or heap allocation that could not be locked; zeroized
    /// on drop only.
    Unlocked,
}

/// A 32-byte master key held in protected memory.
///
/// On drop the memory is zeroed before release regardless of platform.
pub struct KeyStore {
    backing: Backing,
    key_id: KeyId,
    uses: AtomicU64,
    rotation_threshold: u64,
}

impl KeyStore {
    /// Generate a fresh random key.
    pub fn generate() -> CryptoResult<Self> {
        use rand::RngCore;
        let mut key = [0u8; KEY_LEN];
        rand::thread_rng().fill_bytes(&mut key);
        Self::from_bytes(key)
    }

    /// Load a key from a 64-char hex string (e.g. from `VECTORVOLT_MASTER_KEY` env var).
    pub fn from_hex(hex: &str) -> CryptoResult<Self> {
        if hex.len() != KEY_LEN * 2 {
            return Err(CryptoError::KeyStore(format!(
                "expected {} hex chars, got {}",
                KEY_LEN * 2,
                hex.len()
            )));
        }
        let mut key = [0u8; KEY_LEN];
        for (i, chunk) in hex.as_bytes().chunks(2).enumerate() {
            let parsed = std::str::from_utf8(chunk)
                .map_err(|_| CryptoError::KeyStore("invalid hex".into()))
                .and_then(|s| {
                    u8::from_str_radix(s, 16)
                        .map_err(|_| CryptoError::KeyStore("invalid hex char".into()))
                });
            match parsed {
                Ok(b) => key[i] = b,
                Err(e) => {
                    key.zeroize();
                    return Err(e);
                }
            }
        }
        Self::from_bytes(key)
    }

    /// Load from the `VECTORVOLT_MASTER_KEY` environment variable.
    pub fn from_env() -> CryptoResult<Self> {
        let mut hex = std::env::var("VECTORVOLT_MASTER_KEY")
            .map_err(|_| CryptoError::KeyStore("VECTORVOLT_MASTER_KEY not set".into()))?;
        let r = Self::from_hex(hex.trim());
        hex.zeroize();
        r
    }

    /// Move raw key bytes into protected memory.
    ///
    /// The argument array is zeroized after it has been copied. Because
    /// `[u8; N]` is `Copy`, callers must not keep their own copy around
    /// (zeroize it themselves if they do).
    pub fn from_bytes(mut key: [u8; KEY_LEN]) -> CryptoResult<Self> {
        let key_id = key_id_of(&key);
        let backing = Backing::new(&key);
        key.zeroize();
        Ok(Self {
            backing,
            key_id,
            uses: AtomicU64::new(0),
            rotation_threshold: DEFAULT_ROTATION_THRESHOLD,
        })
    }

    /// Override the [`needs_rotation`](Self::needs_rotation) threshold
    /// (number of encryptions). Mainly for tests and conservative deployments.
    pub fn with_rotation_threshold(mut self, threshold: u64) -> Self {
        self.rotation_threshold = threshold;
        self
    }

    /// Return a reference to the raw key bytes, borrowed from the protected
    /// backing (no copy is made).
    pub fn as_bytes(&self) -> &[u8; KEY_LEN] {
        self.backing.bytes()
    }

    /// Hex-encode the key (for `vvctl key gen` output only — never log this).
    pub fn to_hex(&self) -> String {
        self.as_bytes().iter().map(|b| format!("{b:02x}")).collect()
    }

    /// Non-secret key identifier: `SHA3-256("vectorvolt-key-id-v1" || key)[..8]`.
    pub fn key_id(&self) -> KeyId {
        self.key_id
    }

    /// How the key bytes are protected in memory on this host.
    pub fn protection(&self) -> KeyProtection {
        self.backing.protection()
    }

    /// Record one encryption (one nonce drawn) under this key and return the
    /// new total. Call once per AES-GCM seal.
    pub fn record_use(&self) -> u64 {
        self.uses.fetch_add(1, Ordering::Relaxed).saturating_add(1)
    }

    /// Number of encryptions recorded with [`record_use`](Self::record_use).
    pub fn uses(&self) -> u64 {
        self.uses.load(Ordering::Relaxed)
    }

    /// The configured rotation threshold.
    pub fn rotation_threshold(&self) -> u64 {
        self.rotation_threshold
    }

    /// `true` once [`uses`](Self::uses) has reached the rotation threshold:
    /// the owner should rotate to a fresh key before encrypting more data.
    pub fn needs_rotation(&self) -> bool {
        self.uses() >= self.rotation_threshold
    }
}

impl std::fmt::Debug for KeyStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("KeyStore")
            .field("key_id", &self.key_id)
            .field("protection", &self.protection())
            .field("uses", &self.uses())
            .finish_non_exhaustive()
    }
}

// ── backing storage ───────────────────────────────────────────────────────────

enum Backing {
    #[cfg(target_os = "linux")]
    Memfd(MemfdSecret),
    #[cfg(any(unix, windows))]
    Page(LockedPage),
    Heap(Box<[u8; KEY_LEN]>),
}

impl Backing {
    fn new(key: &[u8; KEY_LEN]) -> Self {
        #[cfg(target_os = "linux")]
        if let Ok(mut m) = MemfdSecret::new() {
            m.bytes_mut().copy_from_slice(key);
            return Backing::Memfd(m);
        }
        #[cfg(any(unix, windows))]
        if let Some(mut p) = LockedPage::new() {
            p.bytes_mut().copy_from_slice(key);
            return Backing::Page(p);
        }
        warn_unlocked_once("no dedicated key page available; using plain heap");
        Backing::Heap(Box::new(*key))
    }

    fn bytes(&self) -> &[u8; KEY_LEN] {
        match self {
            #[cfg(target_os = "linux")]
            Backing::Memfd(m) => m.bytes(),
            #[cfg(any(unix, windows))]
            Backing::Page(p) => p.bytes(),
            Backing::Heap(b) => b,
        }
    }

    fn bytes_mut(&mut self) -> &mut [u8; KEY_LEN] {
        match self {
            #[cfg(target_os = "linux")]
            Backing::Memfd(m) => m.bytes_mut(),
            #[cfg(any(unix, windows))]
            Backing::Page(p) => p.bytes_mut(),
            Backing::Heap(b) => b,
        }
    }

    fn protection(&self) -> KeyProtection {
        match self {
            #[cfg(target_os = "linux")]
            Backing::Memfd(_) => KeyProtection::MemfdSecret,
            #[cfg(any(unix, windows))]
            Backing::Page(p) if p.locked => KeyProtection::Locked,
            #[cfg(any(unix, windows))]
            Backing::Page(_) => KeyProtection::Unlocked,
            Backing::Heap(_) => KeyProtection::Unlocked,
        }
    }

    /// Wipe the key bytes in place (volatile writes).
    fn wipe(&mut self) {
        self.bytes_mut().zeroize();
    }
}

impl Drop for Backing {
    fn drop(&mut self) {
        // Wipe before the variant's own Drop releases the memory.
        self.wipe();
    }
}

/// Log (once per process) that key memory could not be locked.
fn warn_unlocked_once(reason: &str) {
    static WARNED: std::sync::Once = std::sync::Once::new();
    WARNED.call_once(|| {
        tracing::warn!(
            reason,
            "vectorvolt-crypto: master key memory is not locked; it may be swapped to disk"
        );
    });
}

// ── dedicated locked page (unix: mmap + mlock, Windows: VirtualAlloc + VirtualLock) ──

#[cfg(any(unix, windows))]
struct LockedPage {
    ptr: *mut u8,
    len: usize,
    locked: bool,
}

// Safety: LockedPage exclusively owns its mapping; access goes through &/&mut.
#[cfg(any(unix, windows))]
unsafe impl Send for LockedPage {}
#[cfg(any(unix, windows))]
unsafe impl Sync for LockedPage {}

#[cfg(any(unix, windows))]
impl LockedPage {
    fn bytes(&self) -> &[u8; KEY_LEN] {
        // Safety: ptr is a live, page-aligned mapping of len >= KEY_LEN bytes.
        unsafe { &*(self.ptr as *const [u8; KEY_LEN]) }
    }

    fn bytes_mut(&mut self) -> &mut [u8; KEY_LEN] {
        // Safety: as above, and &mut self guarantees exclusivity.
        unsafe { &mut *(self.ptr as *mut [u8; KEY_LEN]) }
    }
}

#[cfg(unix)]
impl LockedPage {
    fn new() -> Option<Self> {
        let page = unsafe { libc::sysconf(libc::_SC_PAGESIZE) };
        let len = if page > 0 { page as usize } else { 4096 }.max(KEY_LEN);
        // Safety: anonymous private mapping, no fd involved.
        let ptr = unsafe {
            libc::mmap(
                std::ptr::null_mut(),
                len,
                libc::PROT_READ | libc::PROT_WRITE,
                libc::MAP_PRIVATE | libc::MAP_ANONYMOUS,
                -1,
                0,
            )
        };
        if ptr == libc::MAP_FAILED {
            return None;
        }
        // Keep the key out of core dumps. Best effort.
        // Safety: ptr/len describe the mapping just created.
        #[cfg(target_os = "linux")]
        unsafe {
            libc::madvise(ptr, len, libc::MADV_DONTDUMP);
        }
        // Safety: as above.
        let locked = unsafe { libc::mlock(ptr, len) } == 0;
        if !locked {
            warn_unlocked_once("mlock failed (RLIMIT_MEMLOCK?)");
        }
        Some(Self {
            ptr: ptr as *mut u8,
            len,
            locked,
        })
    }
}

#[cfg(unix)]
impl Drop for LockedPage {
    fn drop(&mut self) {
        self.bytes_mut().zeroize();
        // Safety: ptr/len describe our own live mapping.
        unsafe {
            if self.locked {
                libc::munlock(self.ptr as *const libc::c_void, self.len);
            }
            libc::munmap(self.ptr as *mut libc::c_void, self.len);
        }
    }
}

#[cfg(windows)]
impl LockedPage {
    fn new() -> Option<Self> {
        use windows_sys::Win32::System::Memory::{
            MEM_COMMIT, MEM_RESERVE, PAGE_READWRITE, VirtualAlloc, VirtualLock,
        };
        let len = 4096usize.max(KEY_LEN);
        // Safety: fresh committed allocation; the OS picks the address.
        let ptr = unsafe {
            VirtualAlloc(
                std::ptr::null(),
                len,
                MEM_COMMIT | MEM_RESERVE,
                PAGE_READWRITE,
            )
        };
        if ptr.is_null() {
            return None;
        }
        // Safety: ptr/len describe the allocation just made.
        let locked = unsafe { VirtualLock(ptr, len) } != 0;
        if !locked {
            warn_unlocked_once("VirtualLock failed (working-set quota?)");
        }
        Some(Self {
            ptr: ptr as *mut u8,
            len,
            locked,
        })
    }
}

#[cfg(windows)]
impl Drop for LockedPage {
    fn drop(&mut self) {
        use windows_sys::Win32::System::Memory::{MEM_RELEASE, VirtualFree, VirtualUnlock};
        self.bytes_mut().zeroize();
        // Safety: ptr/len describe our own live allocation.
        unsafe {
            if self.locked {
                VirtualUnlock(self.ptr as *const core::ffi::c_void, self.len);
            }
            VirtualFree(self.ptr as *mut core::ffi::c_void, 0, MEM_RELEASE);
        }
    }
}

// ── memfd_secret wrapper (Linux ≥5.14) ────────────────────────────────────────

#[cfg(target_os = "linux")]
struct MemfdSecret {
    fd: libc::c_int,
    ptr: *mut u8,
}

#[cfg(target_os = "linux")]
impl MemfdSecret {
    fn new() -> Result<Self, ()> {
        #[cfg(target_arch = "x86_64")]
        const SYS_MEMFD_SECRET: libc::c_long = 447;
        #[cfg(target_arch = "aarch64")]
        const SYS_MEMFD_SECRET: libc::c_long = 423;
        #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
        return Err(()); // unsupported arch

        let fd = unsafe { libc::syscall(SYS_MEMFD_SECRET, 0u32) } as libc::c_int;
        if fd < 0 {
            return Err(());
        }
        let ret = unsafe { libc::ftruncate(fd, KEY_LEN as libc::off_t) };
        if ret != 0 {
            unsafe { libc::close(fd) };
            return Err(());
        }
        let ptr = unsafe {
            libc::mmap(
                std::ptr::null_mut(),
                KEY_LEN,
                libc::PROT_READ | libc::PROT_WRITE,
                libc::MAP_SHARED,
                fd,
                0,
            )
        };
        if ptr == libc::MAP_FAILED {
            unsafe { libc::close(fd) };
            return Err(());
        }
        Ok(Self {
            fd,
            ptr: ptr as *mut u8,
        })
    }

    fn bytes(&self) -> &[u8; KEY_LEN] {
        // Safety: ptr is a live mapping of KEY_LEN bytes.
        unsafe { &*(self.ptr as *const [u8; KEY_LEN]) }
    }

    fn bytes_mut(&mut self) -> &mut [u8; KEY_LEN] {
        // Safety: as above, and &mut self guarantees exclusivity.
        unsafe { &mut *(self.ptr as *mut [u8; KEY_LEN]) }
    }
}

#[cfg(target_os = "linux")]
impl Drop for MemfdSecret {
    fn drop(&mut self) {
        self.bytes_mut().zeroize();
        unsafe {
            libc::munmap(self.ptr as *mut libc::c_void, KEY_LEN);
            libc::close(self.fd);
        }
    }
}

// Safety: MemfdSecret exclusively owns its mapping; access goes through &/&mut.
#[cfg(target_os = "linux")]
unsafe impl Send for MemfdSecret {}
#[cfg(target_os = "linux")]
unsafe impl Sync for MemfdSecret {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wipe_zeroes_backing_in_place() {
        // Drop calls exactly this wipe before releasing memory; reading freed
        // memory after drop would be UB, so test the wipe itself.
        let mut ks = KeyStore::from_bytes([0xAB; KEY_LEN]).unwrap();
        assert_eq!(ks.as_bytes(), &[0xAB; KEY_LEN]);
        ks.backing.wipe();
        assert_eq!(ks.as_bytes(), &[0u8; KEY_LEN]);
    }

    #[test]
    fn as_bytes_borrows_from_backing_not_a_copy() {
        let ks = KeyStore::from_bytes([7; KEY_LEN]).unwrap();
        assert_eq!(ks.as_bytes().as_ptr(), ks.backing.bytes().as_ptr());
        // Stable address across calls: served from one place.
        assert_eq!(ks.as_bytes().as_ptr(), ks.as_bytes().as_ptr());
    }

    #[cfg(any(unix, windows))]
    #[test]
    fn uses_dedicated_page_on_unix_and_windows() {
        let ks = KeyStore::from_bytes([1; KEY_LEN]).unwrap();
        match &ks.backing {
            #[cfg(target_os = "linux")]
            Backing::Memfd(_) => {}
            Backing::Page(p) => assert_eq!(p.ptr as usize % 4096, 0),
            Backing::Heap(_) => panic!("expected a dedicated page"),
        }
    }
}
