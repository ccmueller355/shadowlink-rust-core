# CLI → Core Interface Contract

**Feature**: 002-cli-integration | **Date**: 2026-06-14

This contract defines the API surface that `shadowlink-cli` consumes from `shadowlink-rust-core`. All functions are called with fully-qualified paths (e.g., `shadowlink_rust_core::client::connect()`).

## Core Modules Consumed

| Module | Purpose | CLI commands using it |
|--------|---------|----------------------|
| `client` | Session lifecycle, sync management | connect, disconnect, all (restore) |
| `rooms` | Room CRUD operations | create-room, list-rooms, invite, accept, leave |
| `messaging` | E2EE messaging | send, get-history, listen |
| `location` | Location sharing | share-location |

## Function Contracts

### `client::connect(homeserver: &str, username: &str, password: &str) -> Result<SessionHandle, ShadowLinkError>`

- **Preconditions**: Homeserver URL is reachable, credentials are valid
- **Postconditions**: Session persisted to `shadowlink_data/session.json`, sync loop started
- **Error cases**: `ConnectionFailed` (unreachable), `AuthenticationFailed` (bad creds)

### `client::restore_session() -> Result<SessionHandle, ShadowLinkError>`

- **Preconditions**: `shadowlink_data/session.json` exists and is valid
- **Postconditions**: Session restored, sync loop started
- **Error cases**: `SessionExpired` (expired tokens), `StorageError` (missing/corrupt file)

### `client::stop_sync(handle: &SessionHandle)`

- **Preconditions**: Valid session handle
- **Postconditions**: Sync loop stopped, session handle invalidated
- **Note**: Called before every command exit to persist sync state. Not the same as disconnect.

### `client::disconnect(handle: SessionHandle) -> Result<(), ShadowLinkError>`

- **Preconditions**: Valid session handle
- **Postconditions**: Session file deleted, SQLite store removed, sync stopped
- **Called by**: `disconnect` command only (not between commands)

### `rooms::list_rooms(handle: &SessionHandle) -> Result<Vec<RoomInfo>, ShadowLinkError>`

- **Returns**: All rooms with state `Joined`, `Invited`, or `Left`
- **RoomInfo fields**: `room_id: String`, `display_name: Option<String>`, `state: RoomState`

### `rooms::create_room(handle: &SessionHandle, name: &str) -> Result<String, ShadowLinkError>`

- **Returns**: Created room ID
- **Postconditions**: Room is E2EE-enabled, user is joined

### `rooms::invite_user(handle: &SessionHandle, room_id: &str, user_id: &str) -> Result<(), ShadowLinkError>`

- **Preconditions**: Caller is a member of `room_id`, `user_id` is a valid Matrix ID
- **Postconditions**: Invitation sent

### `rooms::accept_invite(handle: &SessionHandle, room_id: &str) -> Result<(), ShadowLinkError>`

- **Preconditions**: User has a pending invitation to `room_id`
- **Postconditions**: User joins the room

### `rooms::leave_room(handle: &SessionHandle, room_id: &str) -> Result<(), ShadowLinkError>`

- **Preconditions**: User is a member of `room_id`
- **Postconditions**: User leaves the room

### `messaging::send_text(handle: &SessionHandle, room_id: &str, body: &str) -> Result<String, ShadowLinkError>`

- **Returns**: Event ID of the sent message
- **Postconditions**: Message encrypted and sent via Megolm

### `messaging::get_history(handle: &SessionHandle, room_id: &str, limit: u32) -> Result<Vec<Message>, ShadowLinkError>`

- **Returns**: Up to `limit` most recent messages, newest first
- **Message fields**: `sender: String`, `content: MessageContent`, `timestamp: DateTime`, `room_id: String`

### `messaging::register_message_callback(handle: &SessionHandle, cb: Option<MessageCallback>)`

- **Callback signature**: `Fn(Message) + Send + Sync + 'static`
- **Called by**: Core's sync loop when a new decrypted message arrives
- **CLI usage**: `listen` command registers a callback that prints to stdout

### `location::share_location(handle: &SessionHandle, room_id: &str, lat: f64, lng: f64, accuracy: Option<f64>, timestamp: Option<DateTime>) -> Result<String, ShadowLinkError>`

- **Preconditions**: `lat` ∈ [-90, 90], `lng` ∈ [-180, 180]
- **Returns**: Event ID of the location beacon
- **Validation**: CLI validates lat/lng bounds before calling (FR-019)

## Error Handling Contract

| Core Error Variant | CLI Exit Code | User-Facing Message |
|--------------------|---------------|---------------------|
| `ConnectionFailed` | 1 | "Failed to connect to homeserver: {details}" |
| `AuthenticationFailed` | 1 | "Authentication failed: {details}" |
| `SessionExpired` | 1 | "Session expired. Run `shadowlink connect` to re-authenticate." |
| `StorageError` | 1 | "Storage error: {details}" |
| `RoomNotFound` | 1 | "Room not found: {room_id}" |
| `NotMember` | 1 | "Not a member of room: {room_id}" |
| `EncryptionError` | 1 | "Encryption error: {details}" |
| `InvalidInput` | 1 | "Invalid input: {details}" |
| All others | 1 | "{error}" (Display impl) |

## Version Compatibility

- Core `0.2.x` → CLI `0.1.x`
- Breaking core API changes require CLI update (coordinated via git dep on `main`)
- CLI's `Cargo.toml` pins `branch = "main"` — always tracks latest core

## Concurrency Model

- CLI is single-threaded async (tokio)
- `restore_session()` starts a background sync loop
- `stop_sync()` is called before command exit to flush sync state
- `listen` command blocks on `tokio::signal::ctrl_c()` — callback fires from sync loop
