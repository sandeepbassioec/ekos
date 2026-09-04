//! Domain types for Workflow Engine.

use serde::{Deserialize, Serialize};

/// State of a single workflow step.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum StepState {
    Pending,
    Running,
    Completed,
    Failed(String),
}

/// A single step in a workflow DAG.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowStep {
    pub id: u64,
    pub name: String,
    pub state: StepState,
    /// IDs of steps that must be `Completed` before this step can run.
    pub depends_on: Vec<u64>,
}

/// High-level state of a workflow.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum WorkflowState {
    Created,
    Running,
    Completed,
    Failed,
}

/// A workflow with a set of dependent steps.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workflow {
    pub id: u64,
    pub name: String,
    pub steps: Vec<WorkflowStep>,
    pub state: WorkflowState,
}
