use vectorvolt_engine::EngineError;

#[derive(Debug, thiserror::Error)]
pub enum QueryError {
    #[error("syntax error: {0}")]
    Syntax(String),

    #[error("unexpected end of input")]
    UnexpectedEof,

    #[error("engine: {0}")]
    Engine(#[from] EngineError),
}

pub type QueryResult<T> = Result<T, QueryError>;
