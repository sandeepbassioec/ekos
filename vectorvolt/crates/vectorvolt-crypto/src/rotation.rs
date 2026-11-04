//! PP-55: key rotation.
//!
//! A [`KeyRing`] holds one *active* key (used for every new encryption) plus
//! any number of *retired* keys, all addressable by [`KeyId`]. Encrypted
//! files / WAL segments record the `KeyId` of the key that sealed them; on
//! read, look it up with [`KeyRing::get`].
//!
//! Typical flow:
//!
//! 1. Encryptor seals with `ring.active()` and calls `record_use()`.
//! 2. When `ring.active().needs_rotation()` (nonce budget spent) or on an
//!    operator request, call `ring.rotate(KeyStore::generate()?)`.
//! 3. Old data stays readable through the retired key.
//! 4. **Integration point (not implemented here):** compaction should
//!    re-encrypt segments whose header `KeyId != ring.active_id()` with the
//!    active key; once no data references a retired key, drop it with
//!    [`KeyRing::retire`].
//!
//! `KeyRing` is not internally synchronized; share it behind an `RwLock` (or
//! clone the `Arc<KeyStore>` returned by [`KeyRing::active_arc`] so in-flight
//! encryptions keep their key across a concurrent rotation).

use crate::error::{CryptoError, CryptoResult};
use crate::key_store::{KeyId, KeyStore};
use std::collections::HashMap;
use std::sync::Arc;

/// Active key plus retired keys, indexed by [`KeyId`].
#[derive(Debug)]
pub struct KeyRing {
    active: Arc<KeyStore>,
    retired: HashMap<KeyId, Arc<KeyStore>>,
}

impl KeyRing {
    /// Create a ring whose active key is `active`.
    pub fn new(active: KeyStore) -> Self {
        Self {
            active: Arc::new(active),
            retired: HashMap::new(),
        }
    }

    /// The key to use for new encryptions.
    pub fn active(&self) -> &KeyStore {
        &self.active
    }

    /// A shareable handle to the active key.
    pub fn active_arc(&self) -> Arc<KeyStore> {
        Arc::clone(&self.active)
    }

    /// [`KeyId`] of the active key.
    pub fn active_id(&self) -> KeyId {
        self.active.key_id()
    }

    /// Make `new_key` active; the previous active key is retired but kept for
    /// decryption. Returns the previous active key's id.
    ///
    /// Errors if `new_key` is already present in the ring (same `KeyId`):
    /// re-activating an old key would reuse its spent nonce budget.
    pub fn rotate(&mut self, new_key: KeyStore) -> CryptoResult<KeyId> {
        let new_id = new_key.key_id();
        if self.contains(&new_id) {
            return Err(CryptoError::KeyStore(format!(
                "key {} is already in the key ring",
                hex_id(&new_id)
            )));
        }
        let old = std::mem::replace(&mut self.active, Arc::new(new_key));
        let old_id = old.key_id();
        self.retired.insert(old_id, old);
        Ok(old_id)
    }

    /// Look up a key (active or retired) by id.
    pub fn get(&self, id: &KeyId) -> Option<&KeyStore> {
        if self.active.key_id() == *id {
            Some(&self.active)
        } else {
            self.retired.get(id).map(|k| &**k)
        }
    }

    /// `true` if a key with this id is in the ring.
    pub fn contains(&self, id: &KeyId) -> bool {
        self.get(id).is_some()
    }

    /// Ids of retired (non-active) keys, in no particular order.
    pub fn retired_ids(&self) -> impl Iterator<Item = &KeyId> {
        self.retired.keys()
    }

    /// Total number of keys (active + retired).
    pub fn len(&self) -> usize {
        1 + self.retired.len()
    }

    /// Always `false`: a ring always has an active key.
    pub fn is_empty(&self) -> bool {
        false
    }

    /// Remove a retired key once no data references it any more (after
    /// compaction re-encrypted everything under the active key). The active
    /// key cannot be retired. Returns the removed key, if it was present.
    pub fn retire(&mut self, id: &KeyId) -> CryptoResult<Option<Arc<KeyStore>>> {
        if self.active.key_id() == *id {
            return Err(CryptoError::KeyStore(
                "cannot retire the active key; rotate first".into(),
            ));
        }
        Ok(self.retired.remove(id))
    }
}

fn hex_id(id: &KeyId) -> String {
    id.iter().map(|b| format!("{b:02x}")).collect()
}
