//! The error type of `quire-sync`.

#[derive(Debug, thiserror::Error)]
pub enum SyncError {
    #[error("connection: {0}")]
    Io(#[from] std::io::Error),

    #[error("sync protocol: {0}")]
    Protocol(String),

    #[error("a message or file is too large")]
    TooLarge,

    /// Both copies have their own data and are different libraries.
    #[error("the other device has a different library (only a new, empty copy can join another)")]
    DifferentLibrary,

    /// The other device is not in this device's list of peers.
    #[error("device {0} is not one of your peers (`quire sync peer add`)")]
    NotAPeer(String),

    #[error("network: {0}")]
    Network(String),

    /// The library copy failed (database, files).
    #[error("{0}")]
    Replica(String),
}

pub type Result<T> = std::result::Result<T, SyncError>;
