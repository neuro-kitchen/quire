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

    /// The other device has this library but another person's key: one of the user's own
    /// devices whose user key differs, or someone presenting this library's ID.
    #[error(
        "the other device has this library but another user key; if it is yours, give both the same key \
         (`quire user key export` there, `quire user key import` here); otherwise do not sync with it"
    )]
    SameLibraryOtherUser,

    /// The other device is not in this device's list of peers.
    #[error("device {0} is not one of your peers (`quire sync peer add`)")]
    NotAPeer(String),

    /// The hub does not know this person yet: register with a code (the hub's setup code, an
    /// invitation, or the cloud account's link code).
    #[error("the sync hub does not know you yet: add it with its setup or link code")]
    NotRegistered,

    /// The hub refused a request (`code`: `hub::protocol::ErrorCode`).
    #[error("sync hub: {message}")]
    Hub { code: i32, message: String },

    #[error("network: {0}")]
    Network(String),

    /// The library copy failed (database, files).
    #[error("{0}")]
    Replica(String),
}

pub type Result<T> = std::result::Result<T, SyncError>;
