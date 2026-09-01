//! Sink error types.

#[derive(Debug, thiserror::Error)]
pub enum SinkError {
    #[error("sink I/O error: {0}")]
    Io(String),

    #[error("sink serialization error: {0}")]
    Serialization(String),

    #[error("sink channel closed")]
    ChannelClosed,

    #[error("sink not connected")]
    NotConnected,

    #[error("sink error: {0}")]
    Other(String),
}

pub type SinkResult<T> = Result<T, SinkError>;
