//! Workflow execution engine.

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use parking_lot::RwLock;

use crate::error::{WorkflowError, WorkflowResult};
use crate::workflow::{StepState, Workflow, WorkflowState, WorkflowStep};

/// Persistent workflow engine.
pub struct WorkflowEngine {
    path: PathBuf,
    store: Arc<RwLock<std::collections::HashMap<u64, Workflow>>>,
    next_id: AtomicU64,
}

impl WorkflowEngine {
    /// Open (or create) the workflow store at `dir`.
    pub fn open(dir: &Path) -> WorkflowResult<Self> {
        std::fs::create_dir_all(dir)?;
        let path = dir.join("workflows.json");
        let store: std::collections::HashMap<u64, Workflow> = if path.exists() {
            let raw = std::fs::read_to_string(&path)?;
            serde_json::from_str(&raw)?
        } else {
            std::collections::HashMap::new()
        };
        let next_id = store.keys().copied().max().unwrap_or(0) + 1;
        Ok(Self {
            path,
            store: Arc::new(RwLock::new(store)),
            next_id: AtomicU64::new(next_id),
        })
    }

    fn persist(&self) -> WorkflowResult<()> {
        let raw = serde_json::to_string(&*self.store.read())?;
        std::fs::write(&self.path, raw)?;
        Ok(())
    }

    /// Create a workflow.  `steps` is `(name, depends_on_ids)` pairs.
    /// Returns the workflow id.
    pub fn create(&self, name: &str, steps: Vec<(String, Vec<u64>)>) -> WorkflowResult<u64> {
        let id = self.next_id.fetch_add(1, Ordering::Relaxed);
        let wf_steps: Vec<WorkflowStep> = steps
            .into_iter()
            .enumerate()
            .map(|(i, (sname, deps))| WorkflowStep {
                id: i as u64,
                name: sname,
                state: StepState::Pending,
                depends_on: deps,
            })
            .collect();
        let wf = Workflow {
            id,
            name: name.to_string(),
            steps: wf_steps,
            state: WorkflowState::Created,
        };
        self.store.write().insert(id, wf);
        self.persist()?;
        Ok(id)
    }

    /// Transition a workflow from `Created` to `Running`.
    pub fn start(&self, id: u64) -> WorkflowResult<()> {
        let mut store = self.store.write();
        let wf = store.get_mut(&id).ok_or(WorkflowError::NotFound(id))?;
        if wf.state != WorkflowState::Created {
            return Err(WorkflowError::InvalidTransition);
        }
        wf.state = WorkflowState::Running;
        drop(store);
        self.persist()
    }

    /// Mark step `step_id` in workflow `workflow_id` as `Completed`.
    /// If all steps are completed, the workflow transitions to `Completed`.
    pub fn complete_step(&self, workflow_id: u64, step_id: u64) -> WorkflowResult<()> {
        let mut store = self.store.write();
        let wf = store
            .get_mut(&workflow_id)
            .ok_or(WorkflowError::NotFound(workflow_id))?;
        let step = wf
            .steps
            .iter_mut()
            .find(|s| s.id == step_id)
            .ok_or(WorkflowError::StepNotFound(step_id))?;
        step.state = StepState::Completed;
        let all_done = wf.steps.iter().all(|s| s.state == StepState::Completed);
        if all_done {
            wf.state = WorkflowState::Completed;
        }
        drop(store);
        self.persist()
    }

    /// Mark step `step_id` as `Failed`.  The workflow also transitions to `Failed`.
    pub fn fail_step(&self, workflow_id: u64, step_id: u64, reason: &str) -> WorkflowResult<()> {
        let mut store = self.store.write();
        let wf = store
            .get_mut(&workflow_id)
            .ok_or(WorkflowError::NotFound(workflow_id))?;
        let step = wf
            .steps
            .iter_mut()
            .find(|s| s.id == step_id)
            .ok_or(WorkflowError::StepNotFound(step_id))?;
        step.state = StepState::Failed(reason.to_string());
        wf.state = WorkflowState::Failed;
        drop(store);
        self.persist()
    }

    /// Fetch a workflow by id.
    pub fn get(&self, id: u64) -> WorkflowResult<Option<Workflow>> {
        Ok(self.store.read().get(&id).cloned())
    }

    /// Return the ids of steps whose dependencies are all `Completed`.
    pub fn ready_steps(&self, id: u64) -> WorkflowResult<Vec<u64>> {
        let store = self.store.read();
        let wf = store.get(&id).ok_or(WorkflowError::NotFound(id))?;
        let completed: std::collections::HashSet<u64> = wf
            .steps
            .iter()
            .filter(|s| s.state == StepState::Completed)
            .map(|s| s.id)
            .collect();
        let ready = wf
            .steps
            .iter()
            .filter(|s| {
                s.state == StepState::Pending
                    && s.depends_on.iter().all(|dep| completed.contains(dep))
            })
            .map(|s| s.id)
            .collect();
        Ok(ready)
    }

    /// Return `true` when the workflow has reached `Completed`.
    pub fn is_complete(&self, id: u64) -> WorkflowResult<bool> {
        let store = self.store.read();
        let wf = store.get(&id).ok_or(WorkflowError::NotFound(id))?;
        Ok(wf.state == WorkflowState::Completed)
    }
}
