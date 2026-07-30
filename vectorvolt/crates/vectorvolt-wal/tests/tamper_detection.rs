//! Tamper-detection tests for the WAL hash chain.
//!
//! The WAL's headline security property is that the SHA3-256 chain makes the log
//! **tamper-evident**. Before this file, that property had **zero test coverage** —
//! every existing `verify()` test checked only a clean log. That absence is exactly
//! why a hardcoded placeholder hash survived in the error path: nothing ever
//! constructed a broken chain, so nothing ever read the message.
//!
//! ## The threat model these tests encode
//!
//! There are three levels of tamperer, and they are caught by three different
//! mechanisms. Distinguishing them matters, because only the third reaches the
//! chain check at all:
//!
//! | Tamperer | Fixes | Caught by |
//! |---|---|---|
//! | Careless — flips a byte | nothing | **CRC32** in `Frame::decode` |
//! | Better — fixes the CRC | CRC | **self-hash** in `Frame::decode` |
//! | Sophisticated — fixes CRC *and* self-hash | both | **the chain**, in `verify()` |
//!
//! The third is the interesting one. Both the CRC and the self-hash are computable
//! from a single frame in isolation, so a determined attacker can repair them. What
//! they cannot repair is the *link*: rewriting frame N changes N's `frame_hash`, so
//! frame N+1's `prev_hash` no longer matches — and fixing that would require
//! rewriting every subsequent frame.
//!
//! Note the consequence, asserted below: the break is reported at **N+1**, while
//! the tampering happened at **N**.

use std::fs;
use std::path::PathBuf;

use tempfile::TempDir;
use vectorvolt_wal::{
    Frame, WalConfig, WalError, WalReader, WalWriter, frame::FRAME_OVERHEAD,
    segment::SEG_HEADER_SIZE,
};

/// Fixed payload length for every frame, so frame offsets are computable.
const PAYLOAD_LEN: usize = 16;
const FRAME_LEN: usize = FRAME_OVERHEAD + PAYLOAD_LEN;

/// Write `n` frames of identical length and return (dir, segment path).
fn write_wal(n: usize) -> (TempDir, PathBuf) {
    let dir = TempDir::new().unwrap();
    let cfg = WalConfig::default().with_dir(dir.path()).with_fsync(true);
    let writer = WalWriter::open(cfg).unwrap();

    for i in 0..n {
        // Distinct, fixed-length payloads. Deliberately avoids containing the
        // frame magic, so byte-offset arithmetic cannot be confused by it.
        let payload = format!("payload-{i:08}").into_bytes();
        assert_eq!(payload.len(), PAYLOAD_LEN, "payload length must stay fixed");
        writer.append(payload).unwrap();
    }
    writer.flush().unwrap();
    drop(writer); // release the file before we patch bytes

    let seg = vectorvolt_wal::segment_path(dir.path(), 0);
    (dir, seg)
}

fn frame_span(index: usize) -> (usize, usize) {
    let start = SEG_HEADER_SIZE as usize + index * FRAME_LEN;
    (start, start + FRAME_LEN)
}

fn read_reader(dir: &TempDir) -> WalReader {
    WalReader::open(WalConfig::default().with_dir(dir.path()))
}

// ── baseline ──────────────────────────────────────────────────────────────────

#[test]
fn clean_chain_verifies() {
    let (dir, _seg) = write_wal(10);
    assert_eq!(read_reader(&dir).verify().unwrap(), 10);
}

#[test]
fn empty_wal_verifies_as_zero_frames() {
    let dir = TempDir::new().unwrap();
    let cfg = WalConfig::default().with_dir(dir.path());
    let w = WalWriter::open(cfg).unwrap();
    w.flush().unwrap();
    drop(w);
    assert_eq!(read_reader(&dir).verify().unwrap(), 0);
}

#[test]
fn frame_offset_arithmetic_is_correct() {
    // Guards every other test in this file: if the offsets are wrong, the
    // "tamper" tests would be patching the wrong bytes and could pass vacuously.
    let (_dir, seg) = write_wal(4);
    let bytes = fs::read(&seg).unwrap();
    assert_eq!(
        bytes.len(),
        SEG_HEADER_SIZE as usize + 4 * FRAME_LEN,
        "segment size does not match computed frame layout"
    );
    for i in 0..4 {
        let (start, end) = frame_span(i);
        let mut cur = &bytes[start..end];
        let frame = Frame::decode(&mut cur, 0, start as u64)
            .unwrap_or_else(|e| panic!("frame {i} did not decode at computed offset: {e}"));
        assert_eq!(
            frame.sequence,
            i as u64 + 1,
            "sequence mismatch at index {i}"
        );
    }
}

// ── level 1: careless tamperer (no CRC repair) ────────────────────────────────

#[test]
fn byte_flip_without_crc_repair_is_caught_by_crc() {
    let (dir, seg) = write_wal(6);
    let mut bytes = fs::read(&seg).unwrap();

    // Flip a payload byte in frame index 2 and leave the CRC stale.
    let (start, _) = frame_span(2);
    bytes[start + 61] ^= 0xFF; // 61 = payload offset within the frame
    fs::write(&seg, &bytes).unwrap();

    let err = read_reader(&dir)
        .verify()
        .expect_err("corruption not detected");
    assert!(
        matches!(err, WalError::Crc32Mismatch { .. }),
        "expected Crc32Mismatch, got: {err}"
    );
}

// ── level 3: sophisticated tamperer (CRC + self-hash repaired) ────────────────

#[test]
fn rewritten_frame_with_valid_crc_and_selfhash_breaks_the_chain() {
    // The real threat model. `Frame::encode` recomputes BOTH the self-hash and
    // the CRC, so re-encoding a mutated frame is exactly what a capable attacker
    // would produce — internally consistent, and undetectable frame-by-frame.
    const TAMPERED: usize = 3; // 0-based index; sequence 4
    let (dir, seg) = write_wal(8);
    let mut bytes = fs::read(&seg).unwrap();
    let (start, end) = frame_span(TAMPERED);

    let original_hash = {
        let mut cur = &bytes[start..end];
        Frame::decode(&mut cur, 0, start as u64).unwrap().frame_hash
    };

    // Rewrite the payload, then re-encode so CRC and self-hash are valid.
    let mut frame = {
        let mut cur = &bytes[start..end];
        Frame::decode(&mut cur, 0, start as u64).unwrap()
    };
    frame.payload[0] ^= 0xFF;
    let reencoded = frame.encode();
    assert_eq!(
        reencoded.len(),
        FRAME_LEN,
        "re-encode changed the frame length"
    );
    assert_ne!(
        frame.frame_hash, original_hash,
        "rewriting the payload must change the frame hash, or the chain could not detect it"
    );
    bytes[start..end].copy_from_slice(&reencoded);
    fs::write(&seg, &bytes).unwrap();

    // The tampered frame itself is now internally valid, so it decodes fine.
    {
        let mut cur = &bytes[start..end];
        Frame::decode(&mut cur, 0, start as u64)
            .expect("re-encoded frame should pass per-frame validation — that is the point");
    }

    // But the chain catches it, at the FOLLOWING frame.
    let err = read_reader(&dir)
        .verify()
        .expect_err("chain break not detected");

    match err {
        WalError::ChainBroken {
            sequence,
            expected,
            actual,
        } => {
            // Tampering at index 3 = sequence 4, so sequence 5 fails to link.
            assert_eq!(
                sequence,
                TAMPERED as u64 + 2,
                "break should be reported at the frame that fails to link"
            );
            // The regression this whole fix exists for: `actual` used to be a
            // hardcoded GENESIS_HASH placeholder.
            assert_ne!(
                actual,
                "0".repeat(64),
                "`actual` is all-zero — the GENESIS_HASH placeholder bug is back"
            );
            assert_ne!(
                expected, actual,
                "expected and actual should differ on a break"
            );
            // `expected` must be the hash the tampered frame now has.
            assert_eq!(
                expected,
                frame
                    .frame_hash
                    .iter()
                    .map(|b| format!("{b:02x}"))
                    .collect::<String>(),
                "`expected` should be the preceding frame's actual (post-tamper) hash"
            );
            assert_eq!(expected.len(), 64, "hash should render as 64 hex chars");
            assert_eq!(actual.len(), 64);
        }
        other => panic!("expected ChainBroken, got: {other}"),
    }
}

#[test]
fn chain_break_error_names_the_tampered_frame_in_its_message() {
    // An operator reads the message, not the struct. It must point at the right
    // frame and must not contain a fabricated value.
    const TAMPERED: usize = 1; // sequence 2
    let (dir, seg) = write_wal(5);
    let mut bytes = fs::read(&seg).unwrap();
    let (start, end) = frame_span(TAMPERED);

    let mut frame = {
        let mut cur = &bytes[start..end];
        Frame::decode(&mut cur, 0, start as u64).unwrap()
    };
    frame.payload[3] ^= 0x5A;
    bytes[start..end].copy_from_slice(&frame.encode());
    fs::write(&seg, &bytes).unwrap();

    let msg = read_reader(&dir)
        .verify()
        .expect_err("not detected")
        .to_string();

    assert!(
        msg.contains("sequence 3"),
        "should name the failing frame: {msg}"
    );
    assert!(
        msg.contains("at or before sequence 2"),
        "should point the investigator at the tampered frame: {msg}"
    );
    assert!(
        !msg.contains(&"0".repeat(64)),
        "message contains an all-zero hash — placeholder leaked into output: {msg}"
    );
}

#[test]
fn tampering_the_last_frame_is_not_detected_by_the_chain() {
    // An honest limitation, asserted so it is a documented property rather than a
    // surprise: nothing links *after* the final frame, so a re-encoded last frame
    // leaves the chain internally consistent. Detecting truncation or tail
    // rewriting needs an externally anchored tip hash (the `WalTip` a writer
    // returns), which is a separate mechanism from chain verification.
    const LAST: usize = 4; // 0-based, of 5
    let (dir, seg) = write_wal(5);
    let mut bytes = fs::read(&seg).unwrap();
    let (start, end) = frame_span(LAST);

    let mut frame = {
        let mut cur = &bytes[start..end];
        Frame::decode(&mut cur, 0, start as u64).unwrap()
    };
    frame.payload[0] ^= 0xFF;
    bytes[start..end].copy_from_slice(&frame.encode());
    fs::write(&seg, &bytes).unwrap();

    assert_eq!(
        read_reader(&dir).verify().unwrap(),
        5,
        "chain verification cannot detect a rewritten final frame — \
         this is a known limitation, not a passing security check"
    );
}

#[test]
fn truncating_the_log_is_not_detected_by_the_chain() {
    // Same class of limitation: dropping trailing frames leaves a shorter but
    // perfectly valid chain. Detecting this requires comparing against a
    // remembered tip.
    let (dir, seg) = write_wal(6);
    let bytes = fs::read(&seg).unwrap();
    let (keep_to, _) = frame_span(4); // drop the last two frames
    fs::write(&seg, &bytes[..keep_to]).unwrap();

    assert_eq!(
        read_reader(&dir).verify().unwrap(),
        4,
        "truncation is undetectable by chain verification alone"
    );
}

// ── the fix is a single pass ───────────────────────────────────────────────────

#[test]
fn verify_reports_the_first_break_when_several_frames_are_tampered() {
    // Two tampered frames; the earliest break must be reported, not the last.
    let (dir, seg) = write_wal(10);
    let mut bytes = fs::read(&seg).unwrap();

    for idx in [6usize, 2] {
        let (start, end) = frame_span(idx);
        let mut frame = {
            let mut cur = &bytes[start..end];
            Frame::decode(&mut cur, 0, start as u64).unwrap()
        };
        frame.payload[1] ^= 0xA5;
        bytes[start..end].copy_from_slice(&frame.encode());
    }
    fs::write(&seg, &bytes).unwrap();

    match read_reader(&dir).verify().expect_err("not detected") {
        WalError::ChainBroken { sequence, .. } => assert_eq!(
            sequence, 4,
            "should report the FIRST break (index 2 = seq 3, so seq 4 fails), not a later one"
        ),
        other => panic!("expected ChainBroken, got: {other}"),
    }
}
