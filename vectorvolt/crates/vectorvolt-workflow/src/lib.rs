//! `vectorvolt-workflow` — DAG-based workflow engine reference domain.

pub mod engine;
pub mod error;
mod persist;
pub mod workflow;

pub use engine::WorkflowEngine;
pub use error::{WorkflowError, WorkflowResult};
pub use workflow::{StepState, Workflow, WorkflowState, WorkflowStep};
