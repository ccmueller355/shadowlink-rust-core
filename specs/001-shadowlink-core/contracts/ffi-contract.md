# FFI Contract

> **Status:** Accepted — SpecKit Plan phase.
> These are the public API signatures the proprietary ShadowLink Flutter app
> calls via flutter_rust_bridge v2 code generation. Every function listed
> here maps to a generated Dart method.

## Conventions

- **`SessionHandle`:** Opaque token returned by `connect()` /
  `restore_session()`. Flutter stores it and passes it to every other
  function. Internally wraps `Arc<Mutex<Session>>`.
- **Return type:** All functions return `Result<T, ShadowLinkError>`.
  flutter_rust_bridge converts `Err` to a Dart exception with the error
  message from `Display`.
- **Async:** All functions are `async`. Callers `await` in Dart.
- **Callbacks:** Functions with `_callback` suffix register a Dart function
  that Rust calls when new data arrives. The Dart function must be
  `static` or top-level (flutter_rust_bridge requirement).
- **Memory:** `Vec<u8>` for media data is transferred zero-copy via
  flutter_rust_bridge. Dart owns the buffer after the call completes.
- **Debug Room:** An opt-in diagnostics room (ADR-009). All diagnostic
  events contain a human-readable prefix + JSON metadata suffix. No PII
  is ever emitted. Toggleable at runtime.

---

## client.rs — Session Lifecycle (US1, US5)

### `connect`

```rust
/// Establish a Matrix session on the given homeserver.
///
/// Performs homeserver discovery, login with username/password,
/// initial E2EE key upload, and starts the sync loop.
/// If a persisted session exists in the SDK store, restores it
/// instead of re-authenticating (US5).
///
/// # Parameters
/// - `homeserver_url`: Matrix homeserver URL (e.g., `https://matrix.example.com`).
///   Must include scheme. Well-known delegation is followed automatically by the SDK.
/// - `username`: Matrix user ID localpart (e.g., `alice`), or full MXID
///   (e.g., `@alice:example.com`).
/// - `password`: Account password. Discarded after login — never stored or logged.
///
/// # Returns
/// - `Ok(SessionHandle)`: Opaque session token. Store and pass to all other functions.
///
/// # Errors
/// - `ConnectionFailed`: Homeserver unreachable or DNS resolution failed.
/// - `AuthenticationFailed`: Invalid credentials or homeserver rejected login.
/// - `SessionExpired`: Persisted session exists but token is expired.
/// - `StorageError`: SDK persistence layer failure (corrupt DB, disk full).
///
/// # User Story
/// US1 (Homeserver Configuration), US5 (Session Persistence)
pub async fn connect(
    homeserver_url: String,
    username: String,
    password: String,
) -> Result<SessionHandle, ShadowLinkError>;
```

### `restore_session`

```rust
/// Attempt to restore a previously persisted session.
///
/// Does not take credentials — only works if a valid persisted session
/// exists in the SDK's SQLite store.
///
/// # Returns
/// - `Ok(SessionHandle)`: Session restored. Sync loop started.
///
/// # Errors
/// - `SessionExpired`: Persisted session found but access token is expired.
/// - `StorageError`: No persisted session found or store is corrupted.
///
/// # User Story
/// US5 (Session Persistence)
pub async fn restore_session() -> Result<SessionHandle, ShadowLinkError>;
```

### `disconnect`

```rust
/// Terminate the session: stop sync loop, log out from homeserver,
/// close the SDK store, and drop the session handle.
///
/// After this call, the `SessionHandle` is invalid. Any subsequent
/// use of the handle will return `ShadowLinkError::Internal`.
///
/// # Parameters
/// - `handle`: SessionHandle from `connect()` or `restore_session()`.
///
/// # User Story
/// US1 (Homeserver Configuration)
pub async fn disconnect(handle: SessionHandle);
```

---

## rooms.rs — Room Operations (US2)

### `create_room`

```rust
/// Create a new E2EE-encrypted Matrix room.
///
/// The room is created with default encryption settings
/// (Megolm, algorithm `m.megolm.v1.aes-sha2`).
///
/// # Parameters
/// - `handle`: Valid session handle.
/// - `name`: Human-readable room name (set as `m.room.name` state event).
///
/// # Returns
/// - `Ok(RoomInfo)`: The newly created room's metadata.
///
/// # Errors
/// - `ConnectionFailed`: Network error during room creation request.
/// - `Internal`: SDK error during room creation.
///
/// # User Story
/// US2 (Room Operations)
pub async fn create_room(
    handle: SessionHandle,
    name: String,
) -> Result<RoomInfo, ShadowLinkError>;
```

### `accept_invite`

```rust
/// Accept a pending room invitation and join the room.
///
/// # Parameters
/// - `handle`: Valid session handle.
/// - `room_id`: Full Matrix room ID (e.g., `!abc123:example.com`).
///
/// # Returns
/// - `Ok(RoomInfo)`: The joined room's metadata.
///
/// # Errors
/// - `RoomNotFound`: No pending invite for the given room ID.
/// - `ConnectionFailed`: Network error during join request.
///
/// # User Story
/// US2 (Room Operations)
pub async fn accept_invite(
    handle: SessionHandle,
    room_id: String,
) -> Result<RoomInfo, ShadowLinkError>;
```

### `invite_user`

```rust
/// Invite a user to a room the local user is a member of.
///
/// # Parameters
/// - `handle`: Valid session handle.
/// - `room_id`: Matrix room ID.
/// - `user_id`: Full Matrix user ID to invite (e.g., `@bob:example.com`).
///
/// # Returns
/// - `Ok(())`: Invitation sent successfully.
///
/// # Errors
/// - `NotInRoom`: Local user is not a member of the specified room.
/// - `RoomNotFound`: Room does not exist in the local room list.
/// - `ConnectionFailed`: Network error during invite request.
///
/// # User Story
/// US2 (Room Operations)
pub async fn invite_user(
    handle: SessionHandle,
    room_id: String,
    user_id: String,
) -> Result<(), ShadowLinkError>;
```

### `list_rooms`

```rust
/// Retrieve all rooms the local user has joined or been invited to.
///
/// # Parameters
/// - `handle`: Valid session handle.
///
/// # Returns
/// - `Ok(Vec<RoomInfo>)`: List of rooms with their metadata and state.
///
/// # Errors
/// - `Internal`: SDK room list access failed (mutex poison, etc.).
///
/// # User Story
/// US2 (Room Operations)
pub async fn list_rooms(
    handle: SessionHandle,
) -> Result<Vec<RoomInfo>, ShadowLinkError>;
```

### `leave_room`

```rust
/// Leave a room the local user is currently a member of.
///
/// After leaving, the room will no longer appear in `list_rooms()`
/// with `RoomState::Joined`.
///
/// # Parameters
/// - `handle`: Valid session handle.
/// - `room_id`: Matrix room ID to leave.
///
/// # Returns
/// - `Ok(())`: Successfully left the room.
///
/// # Errors
/// - `NotInRoom`: Local user is not a member of the specified room.
/// - `RoomNotFound`: Room does not exist in the local room list.
///
/// # User Story
/// US2 (Room Operations)
pub async fn leave_room(
    handle: SessionHandle,
    room_id: String,
) -> Result<(), ShadowLinkError>;
```

### `create_family_room`

```rust
/// Create the family's home room — a private, invite-only E2EE room.
///
/// The operator creates this on first setup. The room is created with
/// `join_rule: invite` and an alias derived from the room name. The
/// family room ID is stored persistently so `get_home_room()` can
/// retrieve it after restarts.
///
/// # Parameters
/// - `handle`: Valid session handle.
/// - `name`: Display name for the family room (e.g., "The Smith Family").
///
/// # Returns
/// - `Ok(RoomInfo)`: Newly created room with `is_home: true`.
///
/// # Errors
/// - `Internal`: Room creation failed at the SDK level.
///
/// # User Story
/// US2 (Room Operations)
pub async fn create_family_room(
    handle: SessionHandle,
    name: String,
) -> Result<RoomInfo, ShadowLinkError>;
```

### `set_home_room`

```rust
/// Pin an existing room as the family home room.
///
/// Use this if the family room was created externally or needs to be
/// re-pinned after deletion. Overwrites any previously pinned home room.
///
/// # Parameters
/// - `handle`: Valid session handle.
/// - `room_id`: Matrix room ID of the existing room to pin.
///
/// # Returns
/// - `Ok(RoomInfo)`: The same room with `is_home: true`.
///
/// # Errors
/// - `NotInRoom`: Local user is not a member of the specified room.
/// - `RoomNotFound`: Room does not exist in the local room list.
///
/// # User Story
/// US2 (Room Operations)
pub async fn set_home_room(
    handle: SessionHandle,
    room_id: String,
) -> Result<RoomInfo, ShadowLinkError>;
```

### `get_home_room`

```rust
/// Retrieve the pinned family home room ID, if one is configured.
///
/// Returns `None` if no family room has been created or pinned yet.
/// The Flutter layer uses this to decide whether to show the
/// "set up family room" onboarding flow.
///
/// # Parameters
/// - `handle`: Valid session handle.
///
/// # Returns
/// - `Ok(Option<RoomInfo>)`: The pinned home room, or `None`.
///
/// # Errors
/// - `StorageError`: Failed to read persisted home room ID.
///
/// # User Story
/// US2 (Room Operations)
pub async fn get_home_room(
    handle: SessionHandle,
) -> Result<Option<RoomInfo>, ShadowLinkError>;
```

---

## messaging.rs — E2EE Messaging & Media (US3)

### `send_text`

```rust
/// Send an end-to-end encrypted text message to a room.
///
/// The message is encrypted via Megolm before transmission.
/// The SDK handles queueing if offline.
///
/// # Parameters
/// - `handle`: Valid session handle.
/// - `room_id`: Target Matrix room ID.
/// - `body`: Plaintext message body. Must not be empty.
///
/// # Returns
/// - `Ok(String)`: The Matrix event ID of the sent message.
///
/// # Errors
/// - `NotInRoom`: Local user is not a member of the specified room.
/// - `RoomNotFound`: Room does not exist in the local room list.
/// - `ConnectionFailed`: Network error; message queued for retry.
///
/// # User Story
/// US3 (E2EE Messaging & Media)
pub async fn send_text(
    handle: SessionHandle,
    room_id: String,
    body: String,
) -> Result<String, ShadowLinkError>;
```

### `send_media`

```rust
/// Send an end-to-end encrypted media attachment to a room.
///
/// The SDK encrypts the file, uploads ciphertext to the homeserver's
/// media repository, and sends an `m.room.message` event with the
/// `mxc://` URI and decryption metadata.
///
/// # Parameters
/// - `handle`: Valid session handle.
/// - `room_id`: Target Matrix room ID.
/// - `data`: Raw file bytes (cleartext). The SDK handles encryption.
/// - `mime_type`: MIME type, e.g., `image/jpeg`, `image/png`.
/// - `filename`: Original filename for the receiver's display.
///
/// # Returns
/// - `Ok(String)`: The Matrix event ID of the sent media message.
///
/// # Errors
/// - `NotInRoom`: Local user is not a member of the specified room.
/// - `RoomNotFound`: Room does not exist in the local room list.
/// - `MediaTooLarge`: File exceeds homeserver's upload size limit.
/// - `ConnectionFailed`: Network error during upload.
///
/// # User Story
/// US3 (E2EE Messaging & Media)
pub async fn send_media(
    handle: SessionHandle,
    room_id: String,
    data: Vec<u8>,
    mime_type: String,
    filename: String,
) -> Result<String, ShadowLinkError>;
```

### `get_history`

```rust
/// Retrieve the most recent messages from a room's timeline.
///
/// Returns decrypted messages. The SDK handles Megolm decryption
/// using cached session keys. Messages that cannot be decrypted
/// (missing keys) are omitted from results — no partial/encrypted
/// messages are returned.
///
/// # Parameters
/// - `handle`: Valid session handle.
/// - `room_id`: Matrix room ID.
/// - `limit`: Maximum number of messages to return (1–1000).
///
/// # Returns
/// - `Ok(Vec<Message>)`: Messages in reverse chronological order
///   (newest first). May be fewer than `limit` if room has less history.
///
/// # Errors
/// - `NotInRoom`: Local user is not a member of the specified room.
/// - `RoomNotFound`: Room does not exist in the local room list.
///
/// # User Story
/// US3 (E2EE Messaging & Media)
pub async fn get_history(
    handle: SessionHandle,
    room_id: String,
    limit: u32,
) -> Result<Vec<Message>, ShadowLinkError>;
```

### `register_message_callback`

```rust
/// Register a Dart callback to receive incoming messages in real time.
///
/// The callback is invoked on the Rust async runtime's thread.
/// flutter_rust_bridge handles dispatching to the Dart main isolate.
///
/// Only one callback can be registered per session. Calling this
/// again replaces the previous callback.
///
/// # Parameters
/// - `handle`: Valid session handle.
/// - `callback`: Dart function with signature `void Function(Message)`.
///
/// # User Story
/// US3 (E2EE Messaging & Media)
pub async fn register_message_callback(
    handle: SessionHandle,
    callback: impl Fn(Message) + Send + 'static,
);
```

---

## location.rs — Location Sharing (US4)

### `send_beacon`

```rust
/// Send a one-shot location beacon to a room.
///
/// Transmitted as a custom `org.shadowlink.location` Matrix event.
/// Not part of a live stream — use `start_live_location()` for
/// ongoing tracking.
///
/// # Parameters
/// - `handle`: Valid session handle.
/// - `room_id`: Target Matrix room ID.
/// - `lat`: Latitude in decimal degrees (WGS84). Valid range: [-90, 90].
/// - `lng`: Longitude in decimal degrees (WGS84). Valid range: [-180, 180].
/// - `accuracy_m`: Horizontal accuracy radius in meters. `None` if unknown.
///
/// # Returns
/// - `Ok(String)`: The Matrix event ID of the sent location event.
///
/// # Errors
/// - `NotInRoom`: Local user is not a member of the specified room.
/// - `RoomNotFound`: Room does not exist in the local room list.
/// - `LocationUnavailable`: Device reports location services disabled.
/// - `ConnectionFailed`: Network error; event queued for retry.
///
/// # User Story
/// US4 (Location Sharing)
pub async fn send_beacon(
    handle: SessionHandle,
    room_id: String,
    lat: f64,
    lng: f64,
    accuracy_m: Option<f64>,
) -> Result<String, ShadowLinkError>;
```

### `start_live_location`

```rust
/// Start a live location stream in a room.
///
/// Begins sending periodic location updates at the specified interval.
/// The stream continues until `stop_live_location()` is called or the
/// session disconnects. Updates are sent as `org.shadowlink.location`
/// events with `live: true`.
///
/// # Parameters
/// - `handle`: Valid session handle.
/// - `room_id`: Target Matrix room ID.
/// - `interval_secs`: Seconds between updates. Must be ≥ 5.
///
/// # Returns
/// - `Ok(())`: Live stream started.
///
/// # Errors
/// - `NotInRoom`: Local user is not a member of the specified room.
/// - `RoomNotFound`: Room does not exist in the local room list.
/// - `LocationUnavailable`: Device reports location services disabled.
/// - `Internal`: Live stream already active for this room, or interval < 5.
///
/// # User Story
/// US4 (Location Sharing)
pub async fn start_live_location(
    handle: SessionHandle,
    room_id: String,
    interval_secs: u64,
) -> Result<(), ShadowLinkError>;
```

### `stop_live_location`

```rust
/// Stop an active live location stream in a room.
///
/// No further location updates are sent after this call.
/// A final event may be sent indicating the stream has ended.
///
/// # Parameters
/// - `handle`: Valid session handle.
/// - `room_id`: Target Matrix room ID.
///
/// # Returns
/// - `Ok(())`: Live stream stopped.
///
/// # Errors
/// - `NotInRoom`: Local user is not a member of the specified room.
/// - `RoomNotFound`: Room does not exist in the local room list.
/// - `Internal`: No active live stream for this room.
///
/// # User Story
/// US4 (Location Sharing)
pub async fn stop_live_location(
    handle: SessionHandle,
    room_id: String,
) -> Result<(), ShadowLinkError>;
```

### `register_location_callback`

```rust
/// Register a Dart callback to receive incoming location events.
///
/// The callback fires for both static beacons and live location
/// updates from other room members. Flutter distinguishes them
/// via the event metadata (sender, timestamp) for map rendering.
///
/// Only one callback can be registered per session. Calling this
/// again replaces the previous callback.
///
/// # Parameters
/// - `handle`: Valid session handle.
/// - `callback`: Dart function with signature `void Function(LocationBeacon)`.
///
/// # User Story
/// US4 (Location Sharing)
pub async fn register_location_callback(
    handle: SessionHandle,
    callback: impl Fn(LocationBeacon) + Send + 'static,
);
```

---

## Error Contract Summary

All FFI functions return `Result<T, ShadowLinkError>`. The Flutter app
handles errors by matching on the error variant name (available via
codegen) and displaying the `Display` message to the user.

| Error Variant | Category | Flutter Action |
|---|---|---|
| `ConnectionFailed` | Retryable | Show "Connection lost" toast with retry button |
| `AuthenticationFailed` | Fatal | Show login screen with error message |
| `SessionExpired` | Fatal | Clear stored session, show login screen |
| `NotInRoom` | User | Show "You've left this room" message |
| `RoomNotFound` | User | Refresh room list, retry operation |
| `DecryptionFailed` | Retryable | Show "Unable to decrypt message" placeholder |
| `MediaTooLarge` | User | Resize/compress image, retry upload |
| `LocationUnavailable` | User | Prompt to enable GPS/location permissions |
| `StorageError` | System | Offer "Clear and re-sync" option |
| `Internal` | System | Show generic error, offer "Send diagnostics" |

---

## Debug Room (ADR-009)

### `enable_debug_room`

```rust
/// Enable or disable the debug room at runtime.
///
/// When enabled and the debug room does not yet exist, it is created
/// as a private, invite-only E2EE room named "ShadowLink Debug" with
/// the local user as the sole member. When disabled, diagnostic event
/// emission stops immediately (the room is NOT deleted).
///
/// # Parameters
/// - `handle`: Valid session handle.
/// - `enabled`: `true` to enable, `false` to disable.
///
/// # Returns
/// - `Ok(bool)`: The new state (matches `enabled` on success).
///
/// # Errors
/// - `ConnectionFailed`: Could not create the debug room.
///
/// # User Story
/// US2 (Room Operations), ADR-009
pub async fn enable_debug_room(
    handle: SessionHandle,
    enabled: bool,
) -> Result<bool, ShadowLinkError>;
```

### `is_debug_room_enabled`

```rust
/// Check whether the debug room is currently active.
///
/// # Parameters
/// - `handle`: Valid session handle.
///
/// # Returns
/// - `Ok(bool)`: `true` if debug room is enabled and emitting events.
///
/// # Errors
/// - `Internal`: Failed to read persisted debug room state.
///
/// # User Story
/// US2 (Room Operations), ADR-009
pub async fn is_debug_room_enabled(
    handle: SessionHandle,
) -> Result<bool, ShadowLinkError>;
```

### `get_debug_room_id`

```rust
/// Retrieve the debug room's Matrix room ID for navigation purposes.
///
/// The Flutter layer uses this to provide a direct deep-link to the
/// debug room. Returns `None` if the debug room is disabled or not
/// yet created.
///
/// # Parameters
/// - `handle`: Valid session handle.
///
/// # Returns
/// - `Ok(Option<String>)`: Debug room ID, or `None`.
///
/// # User Story
/// ADR-009
pub async fn get_debug_room_id(
    handle: SessionHandle,
) -> Result<Option<String>, ShadowLinkError>;
```
