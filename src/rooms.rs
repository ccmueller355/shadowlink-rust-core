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
