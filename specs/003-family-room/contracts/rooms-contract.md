# Interface Contracts: Family Room Semantics

**Feature**: 003-family-room | **Date**: 2026-06-15

These contracts extend the existing FFI contract in `specs/001-shadowlink-core/contracts/ffi-contract.md`
and the CLI contract in `specs/002-cli-integration/contracts/cli-core-contract.md`.

## rooms.rs — Family Room Management (US2 extension)

### `create_family_room`

```rust
/// Create the family's home room — a private, invite-only E2EE room.
///
/// The operator creates this on first setup. The room is created with
/// `join_rule: invite` (via RoomPreset::PrivateChat) and an alias derived
/// from the room name. The family room ID is stored persistently so
/// `get_home_room()` can retrieve it after restarts.
///
/// If a family room already exists, the old room is unpinned (not deleted)
/// and the new room becomes the home room.
///
/// # Parameters
/// - `handle`: Valid session handle.
/// - `name`: Display name for the family room (e.g., "The Smith Family").
///
/// # Returns
/// - `Ok(RoomInfo)`: Newly created room with `is_home: true` and `alias` set.
///
/// # Errors
/// - `OperationFailed`: Room creation rejected by the homeserver (policy,
///   admin restriction). The error detail carries the server's reason.
///
/// # Alias derivation
/// The alias localpart is derived by: lowercasing, replacing spaces with
/// hyphens, stripping characters outside [a-z0-9._=-], truncating to 255
/// characters, and stripping leading/trailing hyphens/dots.
/// If the homeserver rejects the alias, the room still succeeds with
/// `alias: None` — aliases are best-effort.
///
/// # User Story
/// US2 (Room Operations) — extended
pub async fn create_family_room(
    handle: &SessionHandle,
    name: &str,
) -> Result<RoomInfo, ShadowLinkError>;
```

### `set_home_room`

```rust
/// Pin an existing joined room as the family home room.
///
/// Use this when the family room was created externally (e.g., via Element)
/// or when re-pinning after the previous home room was left. Overwrites any
/// previously pinned home room.
///
/// If the target room is not E2EE-encrypted, encryption is enabled before
/// pinning. ShadowLink requires E2EE on the family room.
///
/// # Parameters
/// - `handle`: Valid session handle.
/// - `room_id`: Matrix room ID of the existing room to pin.
///
/// # Returns
/// - `Ok(RoomInfo)`: The room with `is_home: true` and `encrypted: true`.
///
/// # Errors
/// - `NotInRoom`: Local user is not a member of the specified room.
/// - `RoomNotFound`: Room does not exist in the local room list.
/// - `OperationFailed`: E2EE could not be enabled (server rejection).
///
/// # User Story
/// US2 (Room Operations)
pub async fn set_home_room(
    handle: &SessionHandle,
    room_id: &str,
) -> Result<RoomInfo, ShadowLinkError>;
```

### `get_home_room`

```rust
/// Retrieve the pinned family home room, if one is configured.
///
/// Returns `None` if no family room has been created or pinned yet. The
/// Flutter layer uses this to decide whether to show the "set up family
/// room" onboarding flow.
///
/// The returned `RoomInfo` has `is_home: true`. If the persisted home room
/// no longer exists on the server, returns the cached info with the
/// best-known state (may be `Left`).
///
/// # Parameters
/// - `handle`: Valid session handle.
///
/// # Returns
/// - `Ok(Some(RoomInfo))`: The pinned home room.
/// - `Ok(None)`: No family room configured.
///
/// # Errors
/// - `StorageError`: Failed to read persisted home room ID.
///
/// # User Story
/// US2 (Room Operations)
pub async fn get_home_room(
    handle: &SessionHandle,
) -> Result<Option<RoomInfo>, ShadowLinkError>;
```

## client.rs — Debug Room Toggle (US5/Session extension)

### `enable_debug_room`

```rust
/// Enable or disable diagnostic event emission to the debug room.
///
/// When enabled, the core creates a private, invite-only E2EE room named
/// "ShadowLink Debug" (if it doesn't already exist) and begins emitting
/// structured diagnostic events to it. When disabled, emission stops but
/// the room is not deleted.
///
/// The debug room toggle is session-scoped — it is NOT persisted across
/// restarts. The Flutter layer must re-enable it on each `connect()` if
/// desired.
///
/// # Parameters
/// - `handle`: Valid session handle.
/// - `enabled`: `true` to start diagnostics, `false` to stop.
///
/// # Returns
/// - `Ok(())`: Toggle applied.
///
/// # Errors
/// - `OperationFailed`: Debug room creation failed (e.g., server rejected
///   room creation). Diagnostic events will not be emitted.
///
/// # User Story
/// US3 (Diagnostic Debug Room)
pub async fn enable_debug_room(
    handle: &SessionHandle,
    enabled: bool,
) -> Result<(), ShadowLinkError>;
```

## FFI Surface (ShadowLinkApi extension)

The `ShadowLinkApi` struct in `src/ffi.rs` gains the following methods:

```rust
impl ShadowLinkApi {
    // Existing methods unchanged...

    /// Create the family home room. Returns the room ID.
    pub async fn create_family_room(&self, name: &str) -> Result<String, ShadowLinkError> {
        let room_info = crate::rooms::create_family_room(&self.handle, name).await?;
        Ok(room_info.room_id)
    }

    /// Pin an existing room as the family home room.
    pub async fn set_home_room(&self, room_id: &str) -> Result<String, ShadowLinkError> {
        let room_info = crate::rooms::set_home_room(&self.handle, room_id).await?;
        Ok(room_info.room_id)
    }

    /// Retrieve the pinned family room ID, or None.
    pub async fn get_home_room(&self) -> Result<Option<String>, ShadowLinkError> {
        let maybe = crate::rooms::get_home_room(&self.handle).await?;
        Ok(maybe.map(|r| r.room_id))
    }

    /// Toggle diagnostic debug room.
    pub async fn enable_debug_room(&self, enabled: bool) -> Result<(), ShadowLinkError> {
        crate::client::enable_debug_room(&self.handle, enabled).await
    }
}
```

## Contract Compliance

| Contract | Addressed by |
|----------|-------------|
| `specs/001-shadowlink-core/contracts/ffi-contract.md` § `create_family_room` | This contract — identical signature |
| `specs/001-shadowlink-core/contracts/ffi-contract.md` § `set_home_room` | This contract — identical signature |
| `specs/001-shadowlink-core/contracts/ffi-contract.md` § `get_home_room` | This contract — identical signature |
| `specs/002-cli-integration/contracts/cli-core-contract.md` | CLI gains `create-family-room`, `set-home-room`, `get-home-room` subcommands — thin wrappers around these functions |
