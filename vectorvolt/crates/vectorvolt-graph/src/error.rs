use thiserror::Error;

#[derive(Debug, Error)]
pub enum GraphError {
    #[error("node {0} not found")]
    NodeNotFound(u64),
    #[error("edge ({0}, {1}) not found")]
    EdgeNotFound(u64, u64),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("graph contains a cycle")]
    CycleDetected,
}

pub type GraphResult<T> = Result<T, GraphError>;
