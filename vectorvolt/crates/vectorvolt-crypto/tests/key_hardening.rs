//! PP-55 / C5: KeyStore hardening, nonce accounting, key ids and rotation.

use sha3::{Digest, Sha3_256};
use vectorvolt_crypto::{
    DEFAULT_ROTATION_THRESHOLD, KEY_LEN, KeyProtection, KeyRing, KeyStore, key_id_of,
};

fn expected_id(key: &[u8; KEY_LEN]) -> [u8; 8] {
    let mut h = Sha3_256::new();
    h.update(b"vectorvolt-key-id-v1");
    h.update(key);
    let d = h.finalize();
    let mut id = [0u8; 8];
    id.copy_from_slice(&d[..8]);
    id
}

#[test]
fn from_bytes_as_bytes_roundtrip() {
    let raw: [u8; KEY_LEN] = std::array::from_fn(|i| i as u8);
    let ks = KeyStore::from_bytes(raw).unwrap();
    assert_eq!(ks.as_bytes(), &raw);
    let ks2 = KeyStore::from_hex(&ks.to_hex()).unwrap();
    assert_eq!(ks2.as_bytes(), &raw);
}

#[test]
fn protection_is_reported() {
    let ks = KeyStore::generate().unwrap();
    let p = ks.protection();
    // Locking may legitimately fail (RLIMIT_MEMLOCK) — that must not be fatal.
    assert!(matches!(
        p,
        KeyProtection::MemfdSecret | KeyProtection::Locked | KeyProtection::Unlocked
    ));
    // Debug never prints key material.
    let dbg = format!("{ks:?}");
    assert!(!dbg.contains(&ks.to_hex()));
}

#[test]
fn many_keys_can_coexist_and_drop() {
    // Exercises lock/unlock/free paths repeatedly (and lock-failure fallback
    // once RLIMIT_MEMLOCK is exhausted on small limits).
    let keys: Vec<KeyStore> = (0..256).map(|_| KeyStore::generate().unwrap()).collect();
    for k in &keys {
        assert_eq!(k.as_bytes().len(), KEY_LEN);
    }
    drop(keys);
}

#[test]
fn key_id_matches_formula_and_is_stable() {
    let raw = [0x42u8; KEY_LEN];
    let a = KeyStore::from_bytes(raw).unwrap();
    let b = KeyStore::from_bytes(raw).unwrap();
    assert_eq!(a.key_id(), expected_id(&raw));
    assert_eq!(a.key_id(), key_id_of(&raw));
    assert_eq!(a.key_id(), b.key_id());
    let c = KeyStore::from_bytes([0x43u8; KEY_LEN]).unwrap();
    assert_ne!(a.key_id(), c.key_id());
}

#[test]
fn key_id_known_answer_for_zero_key() {
    // Pinned so the WAL header's independent implementation can cross-check.
    // Value computed independently: Python hashlib.sha3_256.
    let raw = [0u8; KEY_LEN];
    assert_eq!(
        key_id_of(&raw),
        [0xd0, 0x3f, 0xed, 0xb3, 0x99, 0x64, 0xc3, 0x1d]
    );
    assert_eq!(key_id_of(&raw), expected_id(&raw));
}

#[test]
fn usage_counter_and_rotation_threshold() {
    let ks = KeyStore::generate().unwrap();
    assert_eq!(ks.rotation_threshold(), DEFAULT_ROTATION_THRESHOLD);
    assert_eq!(DEFAULT_ROTATION_THRESHOLD, 1u64 << 31);

    let ks = ks.with_rotation_threshold(3);
    assert_eq!(ks.uses(), 0);
    assert!(!ks.needs_rotation());
    assert_eq!(ks.record_use(), 1);
    assert_eq!(ks.record_use(), 2);
    assert!(!ks.needs_rotation());
    assert_eq!(ks.record_use(), 3);
    assert!(ks.needs_rotation());
    assert_eq!(ks.uses(), 3);
}

#[test]
fn usage_counter_is_thread_safe() {
    let ks = std::sync::Arc::new(KeyStore::generate().unwrap());
    let handles: Vec<_> = (0..8)
        .map(|_| {
            let ks = ks.clone();
            std::thread::spawn(move || {
                for _ in 0..1000 {
                    ks.record_use();
                }
            })
        })
        .collect();
    for h in handles {
        h.join().unwrap();
    }
    assert_eq!(ks.uses(), 8000);
}

#[test]
fn key_ring_rotate_and_get() {
    let k1 = KeyStore::from_bytes([1; KEY_LEN]).unwrap();
    let id1 = k1.key_id();
    let mut ring = KeyRing::new(k1);
    assert_eq!(ring.active_id(), id1);
    assert_eq!(ring.len(), 1);

    let k2 = KeyStore::from_bytes([2; KEY_LEN]).unwrap();
    let id2 = k2.key_id();
    assert_eq!(ring.rotate(k2).unwrap(), id1);
    assert_eq!(ring.active_id(), id2);
    assert_eq!(ring.active().as_bytes(), &[2; KEY_LEN]);
    assert_eq!(ring.get(&id1).unwrap().as_bytes(), &[1; KEY_LEN]);
    assert_eq!(ring.get(&id2).unwrap().as_bytes(), &[2; KEY_LEN]);
    assert!(ring.get(&[0xFF; 8]).is_none());
    assert_eq!(ring.len(), 2);
    assert_eq!(ring.retired_ids().copied().collect::<Vec<_>>(), vec![id1]);

    // Re-adding a known key is rejected (would reuse its nonce budget).
    let dup = KeyStore::from_bytes([1; KEY_LEN]).unwrap();
    assert!(ring.rotate(dup).is_err());

    // Active key cannot be retired; retired one can.
    assert!(ring.retire(&id2).is_err());
    assert!(ring.retire(&id1).unwrap().is_some());
    assert!(ring.get(&id1).is_none());
    assert_eq!(ring.len(), 1);
}

#[test]
fn key_ring_active_arc_survives_rotation() {
    let mut ring = KeyRing::new(KeyStore::from_bytes([9; KEY_LEN]).unwrap());
    let held = ring.active_arc();
    ring.rotate(KeyStore::generate().unwrap()).unwrap();
    assert_eq!(held.as_bytes(), &[9; KEY_LEN]);
}

#[test]
fn encrypt_decrypt_still_work_with_hardened_store() {
    let key = KeyStore::generate().unwrap();
    let ct = vectorvolt_crypto::encrypt(&key, b"hello").unwrap();
    assert_eq!(vectorvolt_crypto::decrypt(&key, &ct).unwrap(), b"hello");
}
