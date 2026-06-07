# Data Model

> **Status:** Accepted — SpecKit Plan phase.
> Cross-referenced from arc42 Section 5 (Building Block View) and Section 8
> (Concepts). These types form the FFI contract surface defined in
> `contracts/ffi-contract.md`.

## Design Principles

1. **FFI-safe by construction:** All types crossing the FFI boundary are
   `Clone`, `Debug`, and contain only owned data (no references, no
   lifetimes visible to Dart).
2. **Opaque handles:** Types that wrap SDK internals (`Session`) are never
   exposed by value. Flutter holds an opaque handle (integer or pointer)
   and passes it back to Rust for every operation.
3. **Algebraic enums over booleans:** Structural states (`RoomState`,
   `MessageContent`) use exhaustive Rust enums. Dart code generation
   produces sealed classes with pattern matching.
4. **Flat over nested:** Deeply nested types cause verbose Dart wrappers.
   Prefer flat structs with optional fields over 3-level nesting.

---

## Core Types

### `Session`

```rust
/// Opaque handle to an authenticated Matrix client session.
///
/// Never exposed to Flutter by value. The FFI layer returns a
/// `SessionHandle` (newtype around `Arc<Mutex<Session>>`) and Flutter
/// passes it back as an opaque token.
///
/// ## Invariants
/// - Created only by `connect()` or `restore_session()`.
/// - Dropped by `disconnect()`, which triggers SDK logout and DB close.
/// - Must not be cloned across threads without explicit `Arc` sharing.
///   flutter_rust_bridge v2 handles this via `StreamSink`.
///
/// ## FFI Safety
/// - This type NEVER crosses the FFI boundary directly.
/// - Raw pointer transmutation is handled by flutter_rust_bridge codegen.
/// - Mutex poisoning: if a Rust panic occurs while holding the lock,
///   the session is poisoned and all subsequent calls return
///   `ShadowLinkError::Internal`.
pub struct Session {
    /// The matrix-rust-sdk Client. Owns the HTTP client, sync token,
    /// E2EE machine, and room list.
    pub(crate) client: matrix_sdk::Client,

    /// Whether the sync loop is currently running.
    /// Controlled by `start_sync()` / `stop_sync()` called internally
    /// after connect/restore.
    pub(crate) sync_running: bool,
}
```

---

### `RoomInfo`

```rust
/// Lightweight room metadata returned to Flutter for display in room lists.
///
/// ## Invariants
/// - `room_id` is a valid Matrix room ID (`!localpart:domain`).
/// - `member_count` includes the local user.
/// - `encrypted` is true if the room has an `m.room.encryption` state event
///   with algorithm `m.megolm.v1.aes-sha2`.
/// - `state` reflects the SDK's internal membership for the local user.
///
/// ## FFI Safety
/// - All fields are owned `String` or primitives. No references.
/// - `Option<String>` maps to Dart `String?`.
/// - `RoomState` is an enum; flutter_rust_bridge generates a Dart enum.
pub struct RoomInfo {
    /// Matrix room ID, e.g. `!abc123:shadowlink.example`.
    pub room_id: String,

    /// Human-readable room name from `m.room.name` state event.
    /// `None` if the room has no name set (canonical alias fallback
    /// not applied — Flutter handles display logic).
    pub name: Option<String>,

    /// Number of joined members.
    pub member_count: u64,

    /// Whether the room is end-to-end encrypted.
    pub encrypted: bool,

    /// Local user's membership state in this room.
    pub state: RoomState,
}

/// Local user's membership state within a room.
///
/// Not to be confused with room-level state (public/private).
/// This enum reflects the SDK's `RoomMember` abstraction.
pub enum RoomState {
    /// User is a joined member and can send/receive messages.
    Joined,

    /// User has a pending invitation and has not yet joined.
    Invited,

    /// User has explicitly left the room. SDK may still hold
    /// limited state for tombstoned rooms.
    Left,
}
```

---

### `Message`

```rust
/// A decrypted message delivered to Flutter via callback or history query.
///
/// ## Invariants
/// - `event_id` is the Matrix event ID (`$unique:domain`).
/// - `sender` is a fully qualified Matrix user ID (`@user:domain`).
/// - `timestamp` is Unix milliseconds since epoch (SDK origin_server_ts).
/// - `content` discriminates between text, media, and location events.
///
/// ## FFI Safety
/// - Fully owned — no references to SDK-internal buffers.
/// - `MessageContent` enum generates Dart sealed class hierarchy.
/// - Messages are NEVER logged (FR-014). Debug formatting omits body.
pub struct Message {
    /// Matrix event ID. Unique per event within the room.
    pub event_id: String,

    /// Fully qualified MXID of the sender.
    pub sender: String,

    /// Unix timestamp in milliseconds since epoch.
    pub timestamp: i64,

    /// Discriminated content payload.
    pub content: MessageContent,
}

/// Content of a decrypted message.
///
/// The SDK decrypts Megolm before constructing this enum.
/// Plaintext body is available only for `Text` variant.
/// Media and Location variants carry metadata, not raw bytes.
pub enum MessageContent {
    /// Plaintext message body.
    Text {
        /// The decrypted text. NEVER logged.
        body: String,
    },

    /// Encrypted media attachment metadata.
    /// Flutter uses `uri` to construct the download URL:
    /// `{homeserver}/_matrix/media/v3/download/{server}/{media_id}`
    Media {
        /// MIME type as reported by sender, e.g. `image/jpeg`.
        mime_type: String,

        /// `mxc://` content URI. Flutter constructs HTTPS download URL.
        uri: String,

        /// Original filename from the sender.
        filename: String,

        /// File size in bytes (from the Matrix event, may be approximate
        /// if thumbnail was sent).
        size_bytes: u64,
    },

    /// Location beacon from `org.shadowlink.location` event.
    Location {
        /// Latitude in decimal degrees (WGS84).
        lat: f64,

        /// Longitude in decimal degrees (WGS84).
        lng: f64,

        /// Horizontal accuracy radius in meters. `None` if the sender
        /// did not provide accuracy (e.g., GPS fix unavailable).
        accuracy_m: Option<f64>,

        /// `true` if this is a live location update (part of an ongoing
        /// stream). `false` for a one-shot beacon.
        live: bool,
    },
}
```

---

### Location Types

```rust
/// A static location beacon delivered to Flutter via callback.
///
/// Sent when another user calls `send_beacon()` or when a live
/// location stream delivers an update.
///
/// ## Invariants
/// - Coordinates are WGS84 decimal degrees.
/// - `accuracy_m`, if present, is positive and represents a
///   1-sigma horizontal confidence radius in meters.
pub struct LocationBeacon {
    pub lat: f64,
    pub lng: f64,
    pub accuracy_m: Option<f64>,
}

/// Configuration for starting a live location stream.
///
/// Passed from Flutter to `start_live_location()`.
///
/// ## Invariants
/// - `interval_secs` must be ≥ 5 (enforced in Rust, returns error
///   if violated — prevents excessive event flooding).
/// - `accuracy_m`, if set, is passed through to location events
///   but does not control the device GPS (that's Flutter's job).
pub struct LiveLocationConfig {
    /// Interval between location updates in seconds.
    pub interval_secs: u64,

    /// Desired accuracy in meters (informational).
    pub accuracy_m: Option<f64>,
}
```

---

### Error Enum

```rust
/// Unified error type for all FFI-exposed functions.
///
/// Each variant carries human-readable context for display in Flutter
/// and a machine-readable variant name for programmatic handling.
///
/// ## FFI Safety
/// - `Clone + Debug` — flutter_rust_bridge generates Dart exception class.
/// - The `Display` impl produces user-facing messages.
/// - Flutter maps variant names to localized error UI.
/// - NEVER includes plaintext message bodies, keys, or tokens.
///
/// ## Error Classification
/// | Category | Variants |
/// |----------|----------|
/// | Retryable | `ConnectionFailed`, `DecryptionFailed` |
/// | Fatal | `AuthenticationFailed`, `SessionExpired`, `RoomNotFound` |
/// | User-correctable | `MediaTooLarge`, `LocationUnavailable`, `NotInRoom` |
/// | System | `StorageError`, `Internal` |
#[derive(Debug, Clone, thiserror::Error)]
pub enum ShadowLinkError {
    /// Network error during homeserver connection or sync.
    /// Retryable — Flutter may offer a "Try Again" button.
    #[error("connection failed: {reason}")]
    ConnectionFailed {
        reason: String,
    },

    /// Invalid credentials (wrong password, expired token before
    /// session expiry detection).
    #[error("authentication failed: {reason}")]
    AuthenticationFailed {
        reason: String,
    },

    /// Previously valid session token has expired and cannot be
    /// refreshed. Requires full re-authentication.
    #[error("session expired — please log in again")]
    SessionExpired,

    /// User attempted an operation (send message, invite) in a room
    /// from which they have left.
    #[error("you are no longer a member of this room")]
    NotInRoom,

    /// The specified room was not found in the local room list.
    /// May indicate session desync — suggest refresh.
    #[error("room not found")]
    RoomNotFound,

    /// E2EE decryption failed for a specific event.
    /// Retryable in some cases (key arrives later via room key sharing).
    /// Fatal if the sender's device is unverified and TOFU policy blocks it.
    #[error("decryption failed for event {event_id}")]
    DecryptionFailed {
        event_id: String,
    },

    /// Media file exceeds the homeserver's configured upload size limit.
    /// Flutter should resize/compress before retrying.
    #[error("media too large: {size_bytes} bytes (limit: {limit_bytes} bytes)")]
    MediaTooLarge {
        size_bytes: u64,
        limit_bytes: u64,
    },

    /// Location services unavailable on the device.
    /// Flutter should prompt the user to enable GPS/location permissions.
    #[error("location services unavailable")]
    LocationUnavailable,

    /// SDK persistence layer error (corrupt DB, disk full, migration failure).
    #[error("storage error: {reason}")]
    StorageError {
        reason: String,
    },

    /// Internal error not attributable to user action.
    /// Includes the underlying error message for debugging.
    /// Flutter should display a generic "something went wrong" message
    /// and offer to send diagnostics.
    #[error("internal error: {message}")]
    Internal {
        message: String,
    },
}
```

---

## FFI Type Mapping

flutter_rust_bridge v2 generates Dart classes automatically. The mapping is:

| Rust Type | Dart Type | Notes |
|---|---|---|
| `String` | `String` | UTF-8, owned |
| `Vec<String>` | `List<String>` | |
| `Vec<u8>` | `Uint8List` | Zero-copy via `frb` |
| `Option<T>` | `T?` | |
| `u64` | `int` | Dart `int` is arbitrary-precision, safe |
| `i64` | `int` | |
| `f64` | `double` | |
| `bool` | `bool` | |
| `enum` (simple) | `enum` | Dart enum with same variants |
| `enum` (data-carrying) | `sealed class` | Freezed-style pattern matching |
| `struct` | `class` | All fields as final properties |
| `Result<T, E>` | `Future<T>` | Err becomes Dart exception |
| `SessionHandle` | Opaque `int`/`ptr` | Passed back to Rust unmodified |

## Memory Ownership

- **Rust allocates, Dart reads:** structs returned by value are copied
  (or moved) to Dart-managed memory by flutter_rust_bridge.
- **Dart allocates, Rust reads:** `Vec<u8>` for media is zero-copy
  borrowed by Rust during the FFI call; ownership transfers back to
  Dart after the async operation completes.
- **Opaque handles:** `SessionHandle` is a newtype around `Arc<Mutex<Session>>`.
  Rust manages the reference count. Dart holds an opaque pointer.
  `disconnect()` drops the Arc, triggering cleanup.
- **Callbacks:** `extern "C" fn(Message)` is registered via
  flutter_rust_bridge's `StreamSink`. The Rust side holds a weak reference
  to avoid pinning Dart isolates beyond their lifetime.
