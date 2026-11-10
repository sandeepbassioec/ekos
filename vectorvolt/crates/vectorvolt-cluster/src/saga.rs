//! Durable cross-shard saga coordinator (gap F8).
//!
//! A saga is an ordered list of [`StepDef`]s, each with a forward and a
//! compensating payload executed against shard-local transactions through a
//! pluggable [`SagaStepExecutor`].  Progress is recorded in an append-only,
//! CRC-framed, fsynced [`SagaLog`] (torn tails are truncated on open), so a
//! coordinator restart resumes: unfinished forward steps continue, or
//! compensations run in reverse after a failure.
//!
//! Idempotency: every execution carries `saga_id/step_id`, the phase and a
//! versionstamp that is logged *before* the first attempt and reused on every
//! retry/resume, so executors can dedupe duplicate deliveries.
//!
//! Executors MUST treat a compensation for a forward that never applied as a
//! no-op (and should refuse a late forward afterwards), because a crash can
//! leave a step "started" but not "done".

use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::versionstamp::{StampError, Versionstamp, VersionstampClock};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Phase {
    Forward,
    Compensate,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct StepDef {
    pub id: String,
    pub shard: u32,
    pub forward: Vec<u8>,
    pub compensate: Vec<u8>,
    pub timeout_ms: u64,
    pub max_attempts: u32,
    pub backoff_ms: u64,
}

impl StepDef {
    pub fn new(id: &str, shard: u32, forward: &[u8], compensate: &[u8]) -> Self {
        Self {
            id: id.into(),
            shard,
            forward: forward.to_vec(),
            compensate: compensate.to_vec(),
            timeout_ms: 5_000,
            max_attempts: 3,
            backoff_ms: 50,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SagaDef {
    pub saga_id: String,
    pub steps: Vec<StepDef>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SagaOutcome {
    Committed,
    Aborted,
}

/// One execution request handed to the executor.
#[derive(Clone, Debug)]
pub struct StepRequest {
    pub saga_id: String,
    pub step_id: String,
    pub shard: u32,
    pub phase: Phase,
    pub stamp: Versionstamp,
    pub payload: Vec<u8>,
}

impl StepRequest {
    /// Stable idempotency key for dedupe: `saga/step` + phase.
    pub fn idempotency_key(&self) -> (String, Phase) {
        (format!("{}/{}", self.saga_id, self.step_id), self.phase)
    }
}

#[derive(Debug, Clone, thiserror::Error)]
pub enum StepError {
    #[error("transient: {0}")]
    Transient(String),
    #[error("permanent: {0}")]
    Permanent(String),
}

#[async_trait::async_trait]
pub trait SagaStepExecutor: Send + Sync {
    /// Run the step as a shard-local transaction. Must be idempotent per
    /// [`StepRequest::idempotency_key`].
    async fn execute(&self, req: &StepRequest) -> Result<(), StepError>;
}

#[derive(Debug, thiserror::Error)]
pub enum SagaError {
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("stamp: {0}")]
    Stamp(#[from] StampError),
    #[error("log corrupt: {0}")]
    Corrupt(String),
    #[error("no saga definition and none in log")]
    NoDefinition,
    #[error("compensation of step {0} exhausted retries")]
    CompensationStuck(String),
    #[error("simulated coordinator crash")]
    Crashed,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Record {
    Begin(SagaDef),
    Started {
        idx: usize,
        phase: Phase,
        stamp: Versionstamp,
    },
    Done {
        idx: usize,
        phase: Phase,
    },
    Failed {
        idx: usize,
        reason: String,
    },
    Finished(SagaOutcome),
}

/// Append-only log. Frame: `len:u32le | crc32(payload):u32le | json payload`.
pub struct SagaLog {
    file: File,
    pub records: Vec<Record>,
    appends_until_crash: Option<usize>,
}

impl SagaLog {
    pub fn open(path: &Path) -> Result<Self, SagaError> {
        let mut file = OpenOptions::new()
            .read(true)
            .append(true)
            .create(true)
            .open(path)?;
        let mut buf = Vec::new();
        file.read_to_end(&mut buf)?;
        let mut records = Vec::new();
        let mut off = 0usize;
        while off + 8 <= buf.len() {
            let len = u32::from_le_bytes(buf[off..off + 4].try_into().unwrap()) as usize;
            let crc = u32::from_le_bytes(buf[off + 4..off + 8].try_into().unwrap());
            let end = off + 8 + len;
            if end > buf.len() || crc32fast::hash(&buf[off + 8..end]) != crc {
                break; // torn / corrupt tail
            }
            match serde_json::from_slice::<Record>(&buf[off + 8..end]) {
                Ok(r) => records.push(r),
                Err(_) => break,
            }
            off = end;
        }
        if off < buf.len() {
            // Drop the torn tail so new appends follow the last good frame.
            drop(file);
            let f = OpenOptions::new().write(true).open(path)?;
            f.set_len(off as u64)?;
            f.sync_all()?;
            file = OpenOptions::new().read(true).append(true).open(path)?;
        }
        Ok(Self {
            file,
            records,
            appends_until_crash: None,
        })
    }

    /// Test hook: the `n`-th next append fails with `Crashed` without writing.
    pub fn crash_after(&mut self, n: usize) {
        self.appends_until_crash = Some(n);
    }

    pub fn append(&mut self, rec: Record) -> Result<(), SagaError> {
        if let Some(n) = self.appends_until_crash.as_mut() {
            if *n == 0 {
                return Err(SagaError::Crashed);
            }
            *n -= 1;
        }
        let payload = serde_json::to_vec(&rec).map_err(|e| SagaError::Corrupt(e.to_string()))?;
        let mut frame = Vec::with_capacity(payload.len() + 8);
        frame.extend_from_slice(&(payload.len() as u32).to_le_bytes());
        frame.extend_from_slice(&crc32fast::hash(&payload).to_le_bytes());
        frame.extend_from_slice(&payload);
        self.file.write_all(&frame)?;
        self.file.sync_data()?;
        self.records.push(rec);
        Ok(())
    }
}

/// State derived by folding the log.
#[derive(Default, Debug)]
struct SagaState {
    def: Option<SagaDef>,
    fwd_stamp: Vec<Option<Versionstamp>>,
    comp_stamp: Vec<Option<Versionstamp>>,
    fwd_done: Vec<bool>,
    comp_done: Vec<bool>,
    failed: Option<usize>,
    finished: Option<SagaOutcome>,
}

impl SagaState {
    fn fold(records: &[Record]) -> Self {
        let mut s = SagaState::default();
        for r in records {
            match r {
                Record::Begin(d) => {
                    let n = d.steps.len();
                    s.fwd_stamp = vec![None; n];
                    s.comp_stamp = vec![None; n];
                    s.fwd_done = vec![false; n];
                    s.comp_done = vec![false; n];
                    s.def = Some(d.clone());
                }
                Record::Started { idx, phase, stamp } => match phase {
                    Phase::Forward => s.fwd_stamp[*idx] = Some(*stamp),
                    Phase::Compensate => s.comp_stamp[*idx] = Some(*stamp),
                },
                Record::Done { idx, phase } => match phase {
                    Phase::Forward => s.fwd_done[*idx] = true,
                    Phase::Compensate => s.comp_done[*idx] = true,
                },
                Record::Failed { idx, .. } => s.failed = Some(*idx),
                Record::Finished(o) => s.finished = Some(*o),
            }
        }
        s
    }
}

pub struct SagaCoordinator<E: SagaStepExecutor> {
    exec: Arc<E>,
    clock: Arc<VersionstampClock>,
}

impl<E: SagaStepExecutor> SagaCoordinator<E> {
    pub fn new(exec: Arc<E>, clock: Arc<VersionstampClock>) -> Self {
        Self { exec, clock }
    }

    /// Run `def` to completion, or resume the saga already recorded in `log`
    /// (then `def` may be `None`; a supplied one is ignored).
    pub async fn run(
        &self,
        log: &mut SagaLog,
        def: Option<SagaDef>,
    ) -> Result<SagaOutcome, SagaError> {
        if !log.records.iter().any(|r| matches!(r, Record::Begin(_))) {
            let d = def.ok_or(SagaError::NoDefinition)?;
            log.append(Record::Begin(d))?;
        }
        let st = SagaState::fold(&log.records);
        if let Some(o) = st.finished {
            return Ok(o);
        }
        let def = st.def.clone().ok_or(SagaError::NoDefinition)?;
        let mut failed = st.failed;

        if failed.is_none() {
            for idx in 0..def.steps.len() {
                if st.fwd_done[idx] {
                    continue;
                }
                let stamp = match st.fwd_stamp[idx] {
                    Some(s) => s,
                    None => {
                        let s = self.clock.next()?;
                        log.append(Record::Started {
                            idx,
                            phase: Phase::Forward,
                            stamp: s,
                        })?;
                        s
                    }
                };
                match self.attempt(&def, idx, Phase::Forward, stamp).await {
                    Ok(()) => log.append(Record::Done {
                        idx,
                        phase: Phase::Forward,
                    })?,
                    Err(reason) => {
                        log.append(Record::Failed { idx, reason })?;
                        failed = Some(idx);
                        break;
                    }
                }
            }
            if failed.is_none() {
                log.append(Record::Finished(SagaOutcome::Committed))?;
                return Ok(SagaOutcome::Committed);
            }
        }

        // Compensate in reverse every step that was started (the failed step
        // may have partially applied, so it is compensated too).
        let st = SagaState::fold(&log.records);
        for idx in (0..def.steps.len()).rev() {
            if st.fwd_stamp[idx].is_none() || st.comp_done[idx] {
                continue;
            }
            let stamp = match st.comp_stamp[idx] {
                Some(s) => s,
                None => {
                    let s = self.clock.next()?;
                    log.append(Record::Started {
                        idx,
                        phase: Phase::Compensate,
                        stamp: s,
                    })?;
                    s
                }
            };
            self.attempt(&def, idx, Phase::Compensate, stamp)
                .await
                .map_err(|_| SagaError::CompensationStuck(def.steps[idx].id.clone()))?;
            log.append(Record::Done {
                idx,
                phase: Phase::Compensate,
            })?;
        }
        log.append(Record::Finished(SagaOutcome::Aborted))?;
        Ok(SagaOutcome::Aborted)
    }

    async fn attempt(
        &self,
        def: &SagaDef,
        idx: usize,
        phase: Phase,
        stamp: Versionstamp,
    ) -> Result<(), String> {
        let step = &def.steps[idx];
        let req = StepRequest {
            saga_id: def.saga_id.clone(),
            step_id: step.id.clone(),
            shard: step.shard,
            phase,
            stamp,
            payload: match phase {
                Phase::Forward => step.forward.clone(),
                Phase::Compensate => step.compensate.clone(),
            },
        };
        // Compensations are retried harder: giving up leaves the saga stuck.
        let attempts = match phase {
            Phase::Forward => step.max_attempts.max(1),
            Phase::Compensate => step.max_attempts.max(5),
        };
        let mut last = String::new();
        for a in 0..attempts {
            if a > 0 {
                let ms = step.backoff_ms.saturating_mul(1u64 << (a - 1).min(10));
                tokio::time::sleep(Duration::from_millis(ms.min(5_000))).await;
            }
            let fut = self.exec.execute(&req);
            match tokio::time::timeout(Duration::from_millis(step.timeout_ms), fut).await {
                Ok(Ok(())) => return Ok(()),
                Ok(Err(StepError::Permanent(m))) => return Err(m),
                Ok(Err(StepError::Transient(m))) => last = m,
                Err(_) => last = format!("timeout after {}ms", step.timeout_ms),
            }
        }
        Err(last)
    }
}
