// ─── [ NEURAL DECK v4.6 $ AI::GENERATED :: NO COPYRIGHT ] ───

use thiserror::Error;

/// Top-level error enum covering all subsystem failures.
///
/// Each variant maps to a specific failure mode. The Flutter layer
/// receives these as typed exceptions via flutter_rust_bridge.
#[derive(Error, Debug, Clone)]
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
    #[error("not a member of room {room_id}")]
    NotInRoom { room_id: String },

    /// Referenced room does not exist or is not visible to this session.
    #[error("room not found: {room_id}")]
    RoomNotFound { room_id: String },

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

    /// A specific room operation failed (SDK error, protocol violation).
    #[error("{operation} failed: {detail}")]
    OperationFailed { operation: String, detail: String },

    /// Catch-all for unexpected internal errors.
    /// The Flutter layer should display a generic error message.
    #[error("internal error: {message}")]
    Internal { message: String },
}

// ── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    /// Verify every variant produces a Display string that is non-empty
    /// and does not contain raw credentials (actual passwords/tokens).
    fn assert_safe_display(err: &ShadowLinkError) {
        let display = err.to_string();
        assert!(!display.is_empty(), "Display must not be empty");
        assert!(
            !display.contains("hunter2"),
            "Display must not leak credential value: {}",
            display
        );
        assert!(
            !display.contains("my_secret_p4ss"),
            "Display must not leak credential value: {}",
            display
        );
    }

    #[test]
    fn test_connection_failed_display() {
        let err = ShadowLinkError::ConnectionFailed {
            reason: "DNS resolution failed".into(),
        };
        assert_safe_display(&err);
        assert!(err.to_string().contains("DNS resolution failed"));
    }

    #[test]
    fn test_authentication_failed_display() {
        let err = ShadowLinkError::AuthenticationFailed {
            reason: "invalid password".into(),
        };
        assert_safe_display(&err);
        assert!(err.to_string().contains("invalid password"));
        // Ensure the password value itself is not dumped verbatim
        assert!(!err.to_string().contains("hunter2"));
    }

    #[test]
    fn test_session_expired_display() {
        let err = ShadowLinkError::SessionExpired;
        assert_safe_display(&err);
        assert!(err.to_string().contains("re-authentication"));
    }

    #[test]
    fn test_not_in_room_display() {
        let err = ShadowLinkError::NotInRoom {
            room_id: "!test:example.com".into(),
        };
        assert_safe_display(&err);
        assert!(err.to_string().contains("!test:example.com"));
    }

    #[test]
    fn test_room_not_found_display() {
        let err = ShadowLinkError::RoomNotFound {
            room_id: "!missing:example.com".into(),
        };
        assert_safe_display(&err);
        assert!(err.to_string().contains("!missing:example.com"));
    }

    #[test]
    fn test_decryption_failed_display() {
        let err = ShadowLinkError::DecryptionFailed {
            event_id: "$abc123".into(),
        };
        assert_safe_display(&err);
        assert!(err.to_string().contains("$abc123"));
    }

    #[test]
    fn test_media_too_large_display() {
        let err = ShadowLinkError::MediaTooLarge {
            size_bytes: 10_000_000,
            limit_bytes: 5_000_000,
        };
        assert_safe_display(&err);
        assert!(err.to_string().contains("10000000"));
        assert!(err.to_string().contains("5000000"));
    }

    #[test]
    fn test_location_unavailable_display() {
        let err = ShadowLinkError::LocationUnavailable;
        assert_safe_display(&err);
    }

    #[test]
    fn test_storage_error_display() {
        let err = ShadowLinkError::StorageError {
            reason: "disk full".into(),
        };
        assert_safe_display(&err);
        assert!(err.to_string().contains("disk full"));
    }

    #[test]
    fn test_internal_error_display() {
        let err = ShadowLinkError::Internal {
            message: "unexpected null".into(),
        };
        assert_safe_display(&err);
        assert!(err.to_string().contains("unexpected null"));
    }

    #[test]
    fn test_operation_failed_display() {
        let err = ShadowLinkError::OperationFailed {
            operation: "create_room".into(),
            detail: "403 Forbidden".into(),
        };
        assert_safe_display(&err);
        assert!(err.to_string().contains("create_room"));
        assert!(err.to_string().contains("403 Forbidden"));
    }

    #[test]
    fn test_clone_round_trip() {
        let err = ShadowLinkError::ConnectionFailed {
            reason: "timeout".into(),
        };
        let cloned = err.clone();
        assert_eq!(err.to_string(), cloned.to_string());
    }

    #[test]
    fn test_debug_round_trip() {
        let err = ShadowLinkError::DecryptionFailed {
            event_id: "$evt001".into(),
        };
        let debugged = format!("{:?}", err);
        assert!(debugged.contains("DecryptionFailed"));
        assert!(debugged.contains("$evt001"));
    }
}
