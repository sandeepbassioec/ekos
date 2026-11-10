use std::collections::{BTreeMap, HashMap, HashSet};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use vectorvolt_cluster::saga::*;
use vectorvolt_cluster::versionstamp::{Versionstamp, VersionstampClock};

#[derive(Clone, Copy)]
enum Mode {
    FailPermanent,
    Hang,
}

#[derive(Default)]
struct Fake {
    // (shard, key) -> value
    data: Mutex<BTreeMap<(u32, String), i64>>,
    applied: Mutex<HashSet<(String, Phase)>>,
    behaviors: Mutex<HashMap<String, Mode>>,
    dup_hits: AtomicU64,
    stamps: Mutex<Vec<Versionstamp>>,
}

fn parse(p: &[u8]) -> (String, i64) {
    let s = std::str::from_utf8(p).unwrap();
    let (k, v) = s.split_once('=').unwrap();
    (k.to_string(), v.parse().unwrap())
}

#[async_trait::async_trait]
impl SagaStepExecutor for Fake {
    async fn execute(&self, req: &StepRequest) -> Result<(), StepError> {
        let mode = self.behaviors.lock().unwrap().get(&req.step_id).copied();
        if req.phase == Phase::Forward {
            match mode {
                Some(Mode::FailPermanent) => return Err(StepError::Permanent("boom".into())),
                Some(Mode::Hang) => {
                    tokio::time::sleep(std::time::Duration::from_secs(60)).await;
                }
                None => {}
            }
        }
        let (key, phase) = req.idempotency_key();
        let mut applied = self.applied.lock().unwrap();
        if applied.contains(&(key.clone(), phase)) {
            self.dup_hits.fetch_add(1, Ordering::SeqCst);
            return Ok(());
        }
        let (k, d) = parse(&req.payload);
        match phase {
            Phase::Forward => {
                if applied.contains(&(key.clone(), Phase::Compensate)) {
                    return Err(StepError::Permanent("already compensated".into()));
                }
                *self.data.lock().unwrap().entry((req.shard, k)).or_default() += d;
            }
            Phase::Compensate => {
                // no-op unless the forward applied
                if applied.contains(&(key.clone(), Phase::Forward)) {
                    *self.data.lock().unwrap().entry((req.shard, k)).or_default() += d;
                }
            }
        }
        applied.insert((key, phase));
        self.stamps.lock().unwrap().push(req.stamp);
        Ok(())
    }
}

fn clock() -> Arc<VersionstampClock> {
    let t = Arc::new(AtomicU64::new(1_000));
    Arc::new(
        VersionstampClock::open(1, 1, None, Arc::new(move || t.load(Ordering::SeqCst))).unwrap(),
    )
}

fn def() -> SagaDef {
    let mk = |id: &str, shard: u32, k: &str, d: i64| {
        let mut s = StepDef::new(
            id,
            shard,
            format!("{k}={d}").as_bytes(),
            format!("{k}={}", -d).as_bytes(),
        );
        s.timeout_ms = 30;
        s.backoff_ms = 1;
        s.max_attempts = 2;
        s
    };
    SagaDef {
        saga_id: "s1".into(),
        steps: vec![
            mk("debit", 1, "a", -10),
            mk("credit", 2, "b", 10),
            mk("audit", 3, "c", 1),
        ],
    }
}

fn snapshot(f: &Fake) -> BTreeMap<(u32, String), i64> {
    f.data
        .lock()
        .unwrap()
        .iter()
        .filter(|(_, v)| **v != 0)
        .map(|(k, v)| (k.clone(), *v))
        .collect()
}

#[tokio::test]
async fn all_steps_succeed() {
    let f = Arc::new(Fake::default());
    let dir = tempfile::tempdir().unwrap();
    let mut log = SagaLog::open(&dir.path().join("s.log")).unwrap();
    let c = SagaCoordinator::new(f.clone(), clock());
    assert_eq!(
        c.run(&mut log, Some(def())).await.unwrap(),
        SagaOutcome::Committed
    );
    assert_eq!(snapshot(&f).len(), 3);
    let st = f.stamps.lock().unwrap().clone();
    assert!(st.windows(2).all(|w| w[0] < w[1]));
    // re-run on a finished log is a no-op
    assert_eq!(c.run(&mut log, None).await.unwrap(), SagaOutcome::Committed);
}

#[tokio::test]
async fn middle_failure_compensates_in_reverse() {
    let f = Arc::new(Fake::default());
    f.behaviors
        .lock()
        .unwrap()
        .insert("credit".into(), Mode::FailPermanent);
    let dir = tempfile::tempdir().unwrap();
    let mut log = SagaLog::open(&dir.path().join("s.log")).unwrap();
    let c = SagaCoordinator::new(f.clone(), clock());
    assert_eq!(
        c.run(&mut log, Some(def())).await.unwrap(),
        SagaOutcome::Aborted
    );
    assert!(snapshot(&f).is_empty());
    // reverse order: failed step (credit) first, then debit; audit never started
    let comps: Vec<usize> = log
        .records
        .iter()
        .filter_map(|r| match r {
            Record::Done {
                idx,
                phase: Phase::Compensate,
            } => Some(*idx),
            _ => None,
        })
        .collect();
    assert_eq!(comps, vec![1, 0]);
}

#[tokio::test]
async fn crash_at_every_record_boundary_converges() {
    for fail in [false, true] {
        let setup = || {
            let f = Arc::new(Fake::default());
            if fail {
                f.behaviors
                    .lock()
                    .unwrap()
                    .insert("credit".into(), Mode::FailPermanent);
            }
            f
        };
        let rf = setup();
        let dir = tempfile::tempdir().unwrap();
        let mut log = SagaLog::open(&dir.path().join("ref.log")).unwrap();
        let ref_outcome = SagaCoordinator::new(rf.clone(), clock())
            .run(&mut log, Some(def()))
            .await
            .unwrap();
        let total = log.records.len();
        let ref_snap = snapshot(&rf);

        for n in 0..=total {
            let f = setup();
            let p = dir.path().join(format!("c{fail}-{n}.log"));
            let mut log = SagaLog::open(&p).unwrap();
            log.crash_after(n);
            let c = SagaCoordinator::new(f.clone(), clock());
            let first = c.run(&mut log, Some(def())).await;
            if n < total {
                assert!(matches!(first, Err(SagaError::Crashed)), "n={n}");
            }
            drop(log);
            // restart: fresh coordinator + clock; definition comes from the log
            let mut log = SagaLog::open(&p).unwrap();
            let c2 = SagaCoordinator::new(f.clone(), clock());
            let out = if n == 0 {
                c2.run(&mut log, Some(def())).await.unwrap() // crashed before Begin
            } else {
                c2.run(&mut log, None).await.unwrap()
            };
            assert_eq!(out, ref_outcome, "n={n} fail={fail}");
            assert_eq!(snapshot(&f), ref_snap, "n={n} fail={fail}");
        }
    }
}

#[tokio::test]
async fn torn_tail_is_truncated_and_resumed() {
    let f = Arc::new(Fake::default());
    let dir = tempfile::tempdir().unwrap();
    let p = dir.path().join("s.log");
    {
        let mut log = SagaLog::open(&p).unwrap();
        log.crash_after(3);
        let _ = SagaCoordinator::new(f.clone(), clock())
            .run(&mut log, Some(def()))
            .await;
    }
    use std::io::Write;
    std::fs::OpenOptions::new()
        .append(true)
        .open(&p)
        .unwrap()
        .write_all(&[9, 0, 0, 0, 1, 2])
        .unwrap();
    let mut log = SagaLog::open(&p).unwrap();
    assert_eq!(log.records.len(), 3);
    let out = SagaCoordinator::new(f.clone(), clock())
        .run(&mut log, None)
        .await
        .unwrap();
    assert_eq!(out, SagaOutcome::Committed);
    assert_eq!(SagaLog::open(&p).unwrap().records.len(), log.records.len());
}

#[tokio::test]
async fn duplicate_delivery_does_not_double_apply() {
    let f = Arc::new(Fake::default());
    let dir = tempfile::tempdir().unwrap();
    let p = dir.path().join("s.log");
    let mut log = SagaLog::open(&p).unwrap();
    // Begin, Started0, Done0 then crash before Started1.
    log.crash_after(3);
    let c = SagaCoordinator::new(f.clone(), clock());
    let _ = c.run(&mut log, Some(def())).await;
    drop(log);
    // Lost ack: executor applied "credit" twice (duplicate delivery) but the
    // coordinator never logged it.
    let req = StepRequest {
        saga_id: "s1".into(),
        step_id: "credit".into(),
        shard: 2,
        phase: Phase::Forward,
        stamp: Versionstamp::ZERO,
        payload: b"b=10".to_vec(),
    };
    f.execute(&req).await.unwrap();
    f.execute(&req).await.unwrap();
    let mut log = SagaLog::open(&p).unwrap();
    c.run(&mut log, None).await.unwrap();
    let snap = snapshot(&f);
    assert_eq!(snap[&(1, "a".to_string())], -10);
    assert_eq!(snap[&(2, "b".to_string())], 10);
    assert_eq!(snap[&(3, "c".to_string())], 1);
    assert!(f.dup_hits.load(Ordering::SeqCst) >= 2);
}

#[tokio::test]
async fn step_timeout_triggers_compensation() {
    let f = Arc::new(Fake::default());
    f.behaviors
        .lock()
        .unwrap()
        .insert("audit".into(), Mode::Hang);
    let dir = tempfile::tempdir().unwrap();
    let mut log = SagaLog::open(&dir.path().join("s.log")).unwrap();
    let c = SagaCoordinator::new(f.clone(), clock());
    assert_eq!(
        c.run(&mut log, Some(def())).await.unwrap(),
        SagaOutcome::Aborted
    );
    assert!(snapshot(&f).is_empty());
    assert!(
        log.records
            .iter()
            .any(|r| matches!(r, Record::Failed { idx: 2, reason } if reason.contains("timeout")))
    );
}
