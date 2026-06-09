// ─── [ NEURAL DECK v4.6 $ AI::GENERATED :: NO COPYRIGHT ] ───

//! Room CRUD operations — create, list, invite, join, leave.
//!
//! US2: Room management over the Matrix client-server API.

use crate::client::SessionHandle;
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
    pub member_count: u64,
    pub encrypted: bool,
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

    Ok(to_room_info(&room, RoomState::Joined).await)
}

/// List all rooms the session is participating in.
///
/// Returns metadata for joined, invited, and left rooms combined.
pub async fn list_rooms(handle: &SessionHandle) -> Result<Vec<RoomInfo>, ShadowLinkError> {
    let guard = handle.0.lock().await;
    let client = &guard.client;

    let mut results = Vec::new();

    for room in client.joined_rooms() {
        results.push(to_room_info(&room, RoomState::Joined).await);
    }
    for room in client.invited_rooms() {
        results.push(to_room_info(&room, RoomState::Invited).await);
    }
    for room in client.left_rooms() {
        results.push(to_room_info(&room, RoomState::Left).await);
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
        member_count: room.joined_members_count(),
        encrypted,
        state,
    }
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
        drop(b);
        assert_eq!(a, RoomState::Joined);
    }

    // ── RoomInfo ──────────────────────────────────────────────────────

    #[test]
    fn test_room_info_construction() {
        let info = RoomInfo {
            room_id: "!test:example.com".into(),
            name: Some("Family Room".into()),
            member_count: 4,
            encrypted: true,
            state: RoomState::Joined,
        };
        assert_eq!(info.room_id, "!test:example.com");
        assert_eq!(info.name.as_deref(), Some("Family Room"));
        assert_eq!(info.member_count, 4);
        assert!(info.encrypted);
        assert_eq!(info.state, RoomState::Joined);
    }

    #[test]
    fn test_room_info_name_none() {
        let info = RoomInfo {
            room_id: "!unnamed:example.com".into(),
            name: None,
            member_count: 1,
            encrypted: false,
            state: RoomState::Invited,
        };
        assert!(info.name.is_none());
        assert!(!info.encrypted);
        assert_eq!(info.state, RoomState::Invited);
    }

    #[test]
    fn test_room_info_debug_does_not_panic() {
        let info = RoomInfo {
            room_id: "!debug:example.com".into(),
            name: Some("Debug".into()),
            member_count: 0,
            encrypted: true,
            state: RoomState::Left,
        };
        let _ = format!("{:?}", info);
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
