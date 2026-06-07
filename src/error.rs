// Unified error type for the ShadowLink Rust Core.
//
// Every fallible public API returns `Result<T, ShadowLinkError>`.
// Variants carry human-readable context for the Flutter layer.

use thiserror::Error;

/// Top-level error enum covering all subsystem failures.
///
/// Each variant maps to a specific failure mode. The Flutter layer
/// receives these as typed exceptions via flutter_rust_bridge.
#[derive(Error, Debug)]
pub enum ShadowLinkError {
    /// Homeserver unreachable, DNS failure, or timeout.
    #[error("connection failed: {reason}")]
    ConnectionFailed { reason: String },

    /// Valid homeserver, but credentials rejected.
    #[error("authentication failed: {reason}")]
    AuthenticationFailed { reason: String },

    /// Previously persisted session has expired and requires re-auth.
    #[error("session expired — re-authentication required")]
    SessionExpired,

    /// Attempted operation on a room the session is not a member of.
    #[error("not a member of this room")]
    NotInRoom,

    /// Referenced room does not exist or is not visible to this session.
    #[error("room not found")]
    RoomNotFound,

    /// E2EE decryption failed — possible key mismatch or verification issue.
    #[error("decryption failed for event {event_id}")]
    DecryptionFailed { event_id: String },

    /// Media upload exceeded the homeserver's size limit.
    #[error("media too large: {size_bytes} bytes (limit: {limit_bytes} bytes)")]
    MediaTooLarge { size_bytes: u64, limit_bytes: u64 },

    /// Location services unavailable on the device.
    #[error("location services unavailable")]
    LocationUnavailable,

    /// SDK persistence layer error (SQLite I/O, corruption, migration).
    #[error("storage error: {reason}")]
    StorageError { reason: String },

    /// Catch-all for unexpected internal errors.
    /// The Flutter layer should display a generic error message.
    #[error("internal error: {message}")]
    Internal { message: String },
}
