//! `vectorvolt-crypto` — data-at-rest security for VectorVolt (VV-SEC-001..004).
//!
//! | Layer | Module | What |
//! |---|---|---|
//! | 1 | `encryptor` + `encrypted_io` | AES-256-GCM file encryption |
//! | 2 | `key_store` | `memfd_secret` / mlock / VirtualLock key isolation, nonce accounting |
//! | 2 | `rotation` | `KeyRing`: active + retired keys by `KeyId` |
//! | 3 | `dac` | chmod 700/600 + chown vectorvolt |
//! | 4 | `hardening` | Linux capability drop + no-new-privs |

pub mod dac;
pub mod encrypted_io;
pub mod encryptor;
pub mod error;
pub mod hardening;
pub mod key_store;
pub mod rotation;

pub use dac::{DacReport, ensure_system_user, harden_data_dir};
pub use encrypted_io::{EncryptedReader, EncryptedWriter};
pub use encryptor::{decrypt, encrypt, is_encrypted};
pub use error::{CryptoError, CryptoResult};
pub use hardening::harden;
pub use key_store::{
    DEFAULT_ROTATION_THRESHOLD, KEY_ID_DOMAIN, KEY_ID_LEN, KEY_LEN, KeyId, KeyProtection, KeyStore,
    key_id_of,
};
pub use rotation::KeyRing;
