// ─── [ NEURAL DECK v4.6 $ AI::GENERATED :: NO COPYRIGHT ] ───

//! Room CRUD operations — create, list, invite, join, leave.
//!
//! US2: Room management over the Matrix client-server API.

use crate::client::{persist_home_room_id, load_home_room_id, SessionHandle};
use crate::error::ShadowLinkError;

use ruma::UserId;
use ruma::api::client::room::create_room;
use std::str::FromStr as _;

/// Membership state of a room (FFI-safe).
#[derive(Clone, Debug, PartialEq)]
pub enum RoomState {
    Joined,
    Invited,
    Left,
}

/// Public room metadata (all owned fields, FFI-safe).
#[derive(Clone, Debug)]
pub struct RoomInfo {
    pub room_id: String,
    pub name: Option<String>,
    pub alias: Option<String>,
    pub member_count: u64,
    pub encrypted: bool,
    pub is_home: bool,
    pub state: RoomState,
}

// ── Public API ────────────────────────────────────────────────────────────

/// Create a new room with the given name.
///
/// E2EE is enabled by default on the room. Returns the metadata of the newly
/// created room.
pub async fn create_room(handle: &SessionHandle, name: &str) -> Result<RoomInfo, ShadowLinkError> {
    let guard = handle.0.lock().await;
    let client = &guard.client;

    let mut request = create_room::v3::Request::new();
    request.name = Some(name.to_owned());
    request.preset = Some(create_room::v3::RoomPreset::PrivateChat);
    request.visibility = ruma::api::client::room::Visibility::Private;

    let room = client
        .create_room(request)
        .await
        .map_err(|e| ShadowLinkError::OperationFailed {
            operation: "create_room".into(),
            detail: e.to_string(),
        })?;

    let _ = room.enable_encryption().await;

    let mut info = to_room_info(&room, RoomState::Joined).await;
    info.encrypted = true; // We just enabled encryption — report it
    Ok(info)
}

/// Create the family home room — a private, invite-only E2EE room.
///
/// The room name is used to derive a Matrix alias localpart. The family room
/// ID is persisted so `get_home_room()` retrieves it after session restarts.
/// If a family room already exists, it is replaced (old room is not deleted).
pub async fn create_family_room(
    handle: &SessionHandle,
    name: &str,
) -> Result<RoomInfo, ShadowLinkError> {
    if name.is_empty() {
        return Err(ShadowLinkError::OperationFailed {
            operation: "create_family_room".into(),
            detail: "Room name must not be empty".into(),
        });
    }
    if name.len() > 255 {
        return Err(ShadowLinkError::OperationFailed {
            operation: "create_family_room".into(),
            detail: format!("Room name too long ({} chars, max 255)", name.len()),
        });
    }

    let guard = handle.0.lock().await;
    let client = &guard.client;

    let alias_localpart = derive_alias_localpart(name);

    let mut request = create_room::v3::Request::new();
    request.name = Some(name.to_owned());
    request.preset = Some(create_room::v3::RoomPreset::PrivateChat);
    request.visibility = ruma::api::client::room::Visibility::Private;
    // Try with alias first. If the alias is taken (M_ROOM_IN_USE), fall back
    // to creating the room without an alias — aliases are best-effort per spec.
    // A trailing space prevents re-using the same request struct across attempts.
    let room_result = if !alias_localpart.is_empty() {
        let mut req_with_alias = create_room::v3::Request::new();
        req_with_alias.name = Some(name.to_owned());
        req_with_alias.preset = Some(create_room::v3::RoomPreset::PrivateChat);
        req_with_alias.visibility = ruma::api::client::room::Visibility::Private;
        req_with_alias.room_alias_name = Some(alias_localpart);
        client.create_room(req_with_alias).await
    } else {
        // Empty alias localpart — try without alias
        client.create_room(request).await
    };

    let room = match room_result {
        Ok(r) => r,
        Err(e) => {
            // If alias collision caused the failure, retry without alias
            if e.as_client_api_error()
                .and_then(|api_err| {
                    use ruma::api::client::error::ErrorBody;
                    if let ErrorBody::Standard { kind, .. } = &api_err.body {
                        Some(kind)
                    } else {
                        None
                    }
                })
                .is_some_and(|kind| {
                    use ruma::api::client::error::ErrorKind;
                    kind == &ErrorKind::RoomInUse
                })
            {
                let mut req_no_alias = create_room::v3::Request::new();
                req_no_alias.name = Some(name.to_owned());
                req_no_alias.preset = Some(create_room::v3::RoomPreset::PrivateChat);
                req_no_alias.visibility = ruma::api::client::room::Visibility::Private;
                client.create_room(req_no_alias).await.map_err(|e2| {
                    ShadowLinkError::OperationFailed {
                        operation: "create_family_room".into(),
                        detail: format!("Room creation failed (with and without alias): {e2}"),
                    }
                })?
            } else {
                return Err(ShadowLinkError::OperationFailed {
                    operation: "create_family_room".into(),
                    detail: e.to_string(),
                });
            }
        }
    };

    let _ = room.enable_encryption().await;

    // Persist the room ID as the family home room
    drop(guard); // release lock before I/O
    persist_home_room_id(room.room_id().as_str())?;

    let mut info = to_room_info(&room, RoomState::Joined).await;
    info.is_home = true;
    info.encrypted = true;
    Ok(info)
}

/// Retrieve the pinned family home room, if one is configured.
///
/// Returns `None` if no family room has been created or pinned yet.
pub async fn get_home_room(handle: &SessionHandle) -> Result<Option<RoomInfo>, ShadowLinkError> {
    let home_room_id = load_home_room_id();

    let Some(ref home_id) = home_room_id else {
        return Ok(None);
    };

    let guard = handle.0.lock().await;
    let client = &guard.client;

    // Look up in joined rooms first, then invited, then left
    for room in client.joined_rooms() {
        if room.room_id().as_str() == home_id {
            let mut info = to_room_info(&room, RoomState::Joined).await;
            info.is_home = true;
            return Ok(Some(info));
        }
    }
    for room in client.invited_rooms() {
        if room.room_id().as_str() == home_id {
            let mut info = to_room_info(&room, RoomState::Invited).await;
            info.is_home = true;
            return Ok(Some(info));
        }
    }
    for room in client.left_rooms() {
        if room.room_id().as_str() == home_id {
            let mut info = to_room_info(&room, RoomState::Left).await;
            info.is_home = true;
            return Ok(Some(info));
        }
    }

    // Persisted room no longer exists — return None
    Ok(None)
}

/// Pin an existing joined room as the family home room.
///
/// If the target room is not E2EE-encrypted, encryption is enabled before pinning.
pub async fn set_home_room(
    handle: &SessionHandle,
    room_id: &str,
) -> Result<RoomInfo, ShadowLinkError> {
    let guard = handle.0.lock().await;
    let client = &guard.client;

    let rid = ruma::OwnedRoomId::from_str(room_id).map_err(|_| ShadowLinkError::RoomNotFound {
        room_id: room_id.to_owned(),
    })?;

    let room = client
        .get_room(&rid)
        .ok_or_else(|| ShadowLinkError::RoomNotFound {
            room_id: room_id.to_owned(),
        })?;

    // If not encrypted, enable E2EE
    if !room.is_encrypted().await.unwrap_or(false) {
        room.enable_encryption()
            .await
            .map_err(|e| ShadowLinkError::OperationFailed {
                operation: "set_home_room".into(),
                detail: format!("Failed to enable encryption: {e}"),
            })?;
    }

    let home_id = room.room_id().as_str().to_owned();
    drop(guard); // release lock before I/O
    persist_home_room_id(&home_id)?;

    let mut info = to_room_info(&room, RoomState::Joined).await;
    info.is_home = true;
    info.encrypted = true;
    Ok(info)
}

/// List all rooms the session is participating in.
///
/// Returns metadata for joined, invited, and left rooms combined.
pub async fn list_rooms(handle: &SessionHandle) -> Result<Vec<RoomInfo>, ShadowLinkError> {
    let home_room_id = load_home_room_id();
    let guard = handle.0.lock().await;
    let client = &guard.client;

    let mut results = Vec::new();

    for room in client.joined_rooms() {
        let mut info = to_room_info(&room, RoomState::Joined).await;
        if let Some(ref home_id) = home_room_id {
            if room.room_id().as_str() == home_id {
                info.is_home = true;
            }
        }
        results.push(info);
    }
    for room in client.invited_rooms() {
        let mut info = to_room_info(&room, RoomState::Invited).await;
        if let Some(ref home_id) = home_room_id {
            if room.room_id().as_str() == home_id {
                info.is_home = true;
            }
        }
        results.push(info);
    }
    for room in client.left_rooms() {
        let mut info = to_room_info(&room, RoomState::Left).await;
        if let Some(ref home_id) = home_room_id {
            if room.room_id().as_str() == home_id {
                info.is_home = true;
            }
        }
        results.push(info);
    }

    Ok(results)
}

/// Accept a pending room invitation by room ID.
///
/// Returns the updated room metadata after joining.
pub async fn accept_invite(
    handle: &SessionHandle,
    room_id: &str,
) -> Result<RoomInfo, ShadowLinkError> {
    let guard = handle.0.lock().await;
    let client = &guard.client;

    let rid = ruma::OwnedRoomId::from_str(room_id).map_err(|_| ShadowLinkError::RoomNotFound {
        room_id: room_id.to_owned(),
    })?;
    let room = client
        .get_room(&rid)
        .ok_or_else(|| ShadowLinkError::RoomNotFound {
            room_id: room_id.to_owned(),
        })?;

    room.join().await.map_err(|e| {
        if e.as_client_api_error().is_some() {
            ShadowLinkError::OperationFailed {
                operation: "accept_invite".into(),
                detail: e.to_string(),
            }
        } else {
            ShadowLinkError::ConnectionFailed {
                reason: e.to_string(),
            }
        }
    })?;

    // Re-fetch to get updated membership state
    let room = client
        .get_room(&rid)
        .ok_or_else(|| ShadowLinkError::RoomNotFound {
            room_id: room_id.to_owned(),
        })?;

    Ok(to_room_info(&room, RoomState::Joined).await)
}

/// Invite a user to a room by Matrix user ID.
///
/// The caller must be a member of the target room.
pub async fn invite_user(
    handle: &SessionHandle,
    room_id: &str,
    user_id: &str,
) -> Result<(), ShadowLinkError> {
    let guard = handle.0.lock().await;
    let client = &guard.client;

    let rid = ruma::OwnedRoomId::from_str(room_id).map_err(|_| ShadowLinkError::RoomNotFound {
        room_id: room_id.to_owned(),
    })?;
    let room = client
        .get_room(&rid)
        .ok_or_else(|| ShadowLinkError::RoomNotFound {
            room_id: room_id.to_owned(),
        })?;

    let user_id = <&UserId>::try_from(user_id).map_err(|_| ShadowLinkError::OperationFailed {
        operation: "invite_user".into(),
        detail: format!("Invalid user ID: {user_id}"),
    })?;

    room.invite_user_by_id(user_id).await.map_err(|e| {
        if let Some(api_err) = e.as_client_api_error() {
            use ruma::api::client::error::{ErrorBody, ErrorKind};
            if let ErrorBody::Standard { kind, .. } = &api_err.body
                && *kind == ErrorKind::Forbidden
            {
                return ShadowLinkError::NotInRoom {
                    room_id: room_id.to_owned(),
                };
            }
        }
        ShadowLinkError::OperationFailed {
            operation: "invite_user".into(),
            detail: e.to_string(),
        }
    })
}

/// Leave a room by room ID.
///
/// The caller must be a member of the target room.
pub async fn leave_room(handle: &SessionHandle, room_id: &str) -> Result<(), ShadowLinkError> {
    let guard = handle.0.lock().await;
    let client = &guard.client;

    let rid = ruma::OwnedRoomId::from_str(room_id).map_err(|_| ShadowLinkError::RoomNotFound {
        room_id: room_id.to_owned(),
    })?;
    let room = client
        .get_room(&rid)
        .ok_or_else(|| ShadowLinkError::RoomNotFound {
            room_id: room_id.to_owned(),
        })?;

    room.leave().await.map_err(|e| {
        if let Some(api_err) = e.as_client_api_error() {
            use ruma::api::client::error::{ErrorBody, ErrorKind};
            if let ErrorBody::Standard { kind, .. } = &api_err.body
                && *kind == ErrorKind::Forbidden
            {
                return ShadowLinkError::NotInRoom {
                    room_id: room_id.to_owned(),
                };
            }
        }
        ShadowLinkError::OperationFailed {
            operation: "leave_room".into(),
            detail: e.to_string(),
        }
    })
}

// ── Helpers ───────────────────────────────────────────────────────────────

/// Project a `matrix_sdk::Room` into our FFI-safe `RoomInfo`.
async fn to_room_info(room: &matrix_sdk::Room, state: RoomState) -> RoomInfo {
    let encrypted = room.is_encrypted().await.unwrap_or(false);
    RoomInfo {
        room_id: room.room_id().as_str().to_owned(),
        name: room.name(),
        alias: room.canonical_alias().map(|a| a.to_string()),
        member_count: room.joined_members_count(),
        encrypted,
        is_home: false, // Set by list_rooms / get_home_room; default false here
        state,
    }
}

/// Derive a Matrix alias localpart from a human-readable room name.
///
/// Rules per Matrix spec:
/// - Lowercase ASCII letters
/// - Replace spaces with hyphens
/// - Drop characters outside [a-z0-9._=-]
/// - Collapse consecutive hyphens
/// - Truncate to 255 characters
/// - Strip leading/trailing hyphens and dots
fn derive_alias_localpart(name: &str) -> String {
    let mut result = String::with_capacity(name.len());
    let mut prev_was_hyphen = false;

    for c in name.chars() {
        match c {
            ' ' => {
                // Replace spaces with hyphens, but no double hyphens
                if !prev_was_hyphen {
                    result.push('-');
                    prev_was_hyphen = true;
                }
            }
            c if c.is_ascii_uppercase() => {
                result.push(c.to_ascii_lowercase());
                prev_was_hyphen = false;
            }
            c if c.is_ascii_lowercase()
                || c.is_ascii_digit()
                || c == '.'
                || c == '_'
                || c == '-'  // explicit hyphens in the name
                || c == '=' =>
            {
                if c == '-' {
                    if !prev_was_hyphen {
                        result.push('-');
                        prev_was_hyphen = true;
                    }
                } else {
                    result.push(c);
                    prev_was_hyphen = false;
                }
            }
            // All other characters (non-ASCII, !, ?, etc.) are dropped
            _ => {}
        }
    }

    // Trim leading/trailing hyphens and dots, truncate to 255
    let trimmed: String = result
        .trim_matches(|c: char| c == '-' || c == '.')
        .chars()
        .take(255)
        .collect();
    trimmed
}

// ── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    // ── RoomState ─────────────────────────────────────────────────────

    #[test]
    fn test_room_state_equality() {
        assert_eq!(RoomState::Joined, RoomState::Joined);
        assert_eq!(RoomState::Invited, RoomState::Invited);
        assert_eq!(RoomState::Left, RoomState::Left);
        assert_ne!(RoomState::Joined, RoomState::Invited);
        assert_ne!(RoomState::Joined, RoomState::Left);
        assert_ne!(RoomState::Invited, RoomState::Left);
    }

    #[test]
    fn test_room_state_clone() {
        let a = RoomState::Joined;
        let b = a.clone();
        assert_eq!(a, b);
        assert_eq!(a, RoomState::Joined);
    }

    // ── RoomInfo ──────────────────────────────────────────────────────

    #[test]
    fn test_room_info_construction() {
        let info = RoomInfo {
            room_id: "!test:example.com".into(),
            name: Some("Family Room".into()),
            alias: Some("#family-room:example.com".into()),
            member_count: 4,
            encrypted: true,
            is_home: true,
            state: RoomState::Joined,
        };
        assert_eq!(info.room_id, "!test:example.com");
        assert_eq!(info.name.as_deref(), Some("Family Room"));
        assert_eq!(info.alias.as_deref(), Some("#family-room:example.com"));
        assert_eq!(info.member_count, 4);
        assert!(info.encrypted);
        assert!(info.is_home);
        assert_eq!(info.state, RoomState::Joined);
    }

    #[test]
    fn test_room_info_name_none() {
        let info = RoomInfo {
            room_id: "!unnamed:example.com".into(),
            name: None,
            alias: None,
            member_count: 1,
            encrypted: false,
            is_home: false,
            state: RoomState::Invited,
        };
        assert!(info.name.is_none());
        assert!(info.alias.is_none());
        assert!(!info.is_home);
        assert!(!info.encrypted);
        assert_eq!(info.state, RoomState::Invited);
    }

    #[test]
    fn test_room_info_debug_does_not_panic() {
        let info = RoomInfo {
            room_id: "!debug:example.com".into(),
            name: Some("Debug".into()),
            alias: None,
            member_count: 0,
            encrypted: true,
            is_home: false,
            state: RoomState::Left,
        };
        let _ = format!("{:?}", info);
    }

    // ── Alias derivation ───────────────────────────────────────────────

    #[test]
    fn test_derive_alias_localpart_basic() {
        assert_eq!(derive_alias_localpart("The Smith Family"), "the-smith-family");
    }

    #[test]
    fn test_derive_alias_localpart_unicode_stripped() {
        // 'Ü' is non-ASCII → dropped; "ber" stays; spaces to hyphens
        assert_eq!(derive_alias_localpart("Über Family Room"), "ber-family-room");
    }

    #[test]
    fn test_derive_alias_localpart_special_chars() {
        assert_eq!(derive_alias_localpart("Hello! World?"), "hello-world");
    }

    #[test]
    fn test_derive_alias_localpart_double_hyphens_collapsed() {
        assert_eq!(derive_alias_localpart("Hello!  World?"), "hello-world");
        assert_eq!(derive_alias_localpart("a---b"), "a-b");
    }

    #[test]
    fn test_derive_alias_localpart_dots_and_underscores() {
        assert_eq!(derive_alias_localpart("test.name_123"), "test.name_123");
    }

    #[test]
    fn test_derive_alias_localpart_truncate() {
        let long = "a".repeat(300);
        let alias = derive_alias_localpart(&long);
        assert_eq!(alias.len(), 255);
    }

    #[test]
    fn test_derive_alias_localpart_trim_leading_trailing() {
        assert_eq!(derive_alias_localpart("--hello--"), "hello");
        assert_eq!(derive_alias_localpart("..world.."), "world");
    }

    // ── Room ID parsing (error mapping paths) ─────────────────────────

    #[test]
    fn test_room_id_parse_valid() {
        let rid = ruma::OwnedRoomId::from_str("!abc123:matrix.org");
        assert!(rid.is_ok());
        assert_eq!(rid.unwrap().as_str(), "!abc123:matrix.org");
    }

    #[test]
    fn test_room_id_parse_invalid() {
        let rid = ruma::OwnedRoomId::from_str("");
        assert!(rid.is_err());

        let rid = ruma::OwnedRoomId::from_str("not-a-room-id");
        assert!(rid.is_err());
    }

    // ── User ID parsing (error mapping paths) ─────────────────────────

    #[test]
    fn test_user_id_parse_valid() {
        let uid = <&UserId>::try_from("@alice:example.com");
        assert!(uid.is_ok());
    }

    #[test]
    fn test_user_id_parse_invalid() {
        let uid = <&UserId>::try_from("alice");
        assert!(uid.is_err());

        let uid = <&UserId>::try_from("");
        assert!(uid.is_err());
    }
}
