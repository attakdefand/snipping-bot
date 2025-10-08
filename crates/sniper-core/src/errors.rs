use thiserror::Error;

#[derive(Debug, Error)]
pub enum SniperError {
    #[error("config error: {0}")]
    Config(String),
    #[error("bus error: {0}")]
    Bus(String),
    #[error("io error: {0}")]
    Io(String),
    #[error("other: {0}")]
    Other(String),
}
