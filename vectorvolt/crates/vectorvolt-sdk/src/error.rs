#[derive(Debug, thiserror::Error)]
pub enum SdkError {
    #[error("net: {0}")]
    Net(String),
    #[error("not connected")]
    NotConnected,
    #[error("operation failed: {0}")]
    Op(String),
}
