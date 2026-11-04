//! Restart tests for the durable EventOutbox / EventDlq (VV-EVT-008, VV-EVT-012).

use std::io::Write;
use std::sync::atomic::{AtomicBool, Ordering};

use vectorvolt_events::sink::SinkError;
use vectorvolt_events::{DomainEventEnvelope, EventDlq, EventOutbox, EventSink};

fn env(pk: u64) -> DomainEventEnvelope {
    DomainEventEnvelope::new("Ev", "1.0.0", "t", pk, serde_json::json!({ "n": pk }))
}

struct Toggle(AtomicBool);
impl EventSink for Toggle {
    fn send(&self, _: &DomainEventEnvelope) -> Result<(), SinkError> {
        if self.0.load(Ordering::SeqCst) {
            Ok(())
        } else {
            Err(SinkError::ChannelClosed)
        }
    }
}

#[test]
fn pending_outbox_entries_survive_restart() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("outbox.log");
    {
        let ob = EventOutbox::open(&path).unwrap();
        ob.push(env(1));
        ob.push(env(2));
        ob.push(env(3));
        assert_eq!(ob.pending_count(), 3);
        // dropped without flush = crash
    }
    let ob = EventOutbox::open(&path).unwrap();
    assert_eq!(ob.pending_count(), 3);
    let dlq = EventDlq::new();
    ob.flush(&Toggle(AtomicBool::new(true)), &dlq);
    assert_eq!(ob.pending_count(), 0);
    drop(ob);
    // Delivered entries are not resurrected.
    assert_eq!(EventOutbox::open(&path).unwrap().pending_count(), 0);
}

#[test]
fn outbox_failed_flush_moves_to_durable_dlq_and_both_survive() {
    let dir = tempfile::tempdir().unwrap();
    let (op, dp) = (dir.path().join("outbox.log"), dir.path().join("dlq.log"));
    {
        let ob = EventOutbox::open(&op).unwrap();
        let dlq = EventDlq::open(&dp).unwrap();
        ob.push(env(1));
        ob.push(env(2));
        ob.flush(&Toggle(AtomicBool::new(false)), &dlq);
        assert_eq!(ob.pending_count(), 0);
        assert_eq!(dlq.len(), 2);
        ob.push(env(3)); // still pending at "crash"
    }
    let ob = EventOutbox::open(&op).unwrap();
    let dlq = EventDlq::open(&dp).unwrap();
    assert_eq!(ob.pending_count(), 1);
    assert_eq!(dlq.len(), 2);
    assert_eq!(dlq.list()[0].0, "Ev");

    // Replay delivers and the removal is durable.
    dlq.replay(&Toggle(AtomicBool::new(true)));
    assert!(dlq.is_empty());
    drop(dlq);
    assert!(EventDlq::open(&dp).unwrap().is_empty());
}

#[test]
fn dlq_drop_all_is_durable_and_partial_replay_keeps_failures() {
    let dir = tempfile::tempdir().unwrap();
    let dp = dir.path().join("dlq.log");
    {
        let dlq = EventDlq::open(&dp).unwrap();
        dlq.push(env(1), "boom".into());
        dlq.push(env(2), "boom".into());
        dlq.replay(&Toggle(AtomicBool::new(false))); // all fail, all kept
        assert_eq!(dlq.len(), 2);
    }
    let dlq = EventDlq::open(&dp).unwrap();
    assert_eq!(dlq.len(), 2);
    assert_eq!(dlq.list()[1].1, "boom");
    dlq.drop_all();
    drop(dlq);
    assert!(EventDlq::open(&dp).unwrap().is_empty());
}

#[test]
fn torn_trailing_record_is_ignored_and_sequence_continues() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("outbox.log");
    {
        let ob = EventOutbox::open(&path).unwrap();
        ob.push(env(1));
    }
    // Simulate a crash mid-append.
    let mut f = std::fs::OpenOptions::new()
        .append(true)
        .open(&path)
        .unwrap();
    f.write_all(b"{\"op\":\"push\",\"seq\":1,\"env\":{\"event_ty")
        .unwrap();
    drop(f);

    let ob = EventOutbox::open(&path).unwrap();
    assert_eq!(ob.pending_count(), 1);
    ob.push(env(2));
    drop(ob);
    assert_eq!(EventOutbox::open(&path).unwrap().pending_count(), 2);
}
