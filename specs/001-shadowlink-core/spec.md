# Feature Specification: ShadowLink Rust Core

**Feature Branch**: `001-shadowlink-core`

**Created**: 2026-06-07

**Status**: Draft

**Input**: User description: "Privacy-first Matrix protocol bridge crate consumed by the ShadowLink Flutter app via FFI. Core capabilities: homeserver connection, E2EE room operations, encrypted messaging + media, location sharing, session persistence."

## User Scenarios & Testing

### User Story 1 — Homeserver Configuration (Priority: P1)

A family member launches the ShadowLink app for the first time, enters their
Matrix homeserver URL and credentials, and the Rust core establishes an
authenticated session.

**Why this priority**: Without homeserver connectivity, no other feature
functions. This is the entry gate for the entire application.

**Independent Test**: Start a local Synapse homeserver in Docker, call the FFI
`connect()` function with the homeserver URL and test credentials, verify a
session token is returned and stored via SDK persistence.

**Acceptance Scenarios**:

1. **Given** a valid homeserver URL and credentials, **When** `client::connect()`
   is called, **Then** a session is established and a session ID is returned.
2. **Given** an invalid homeserver URL (unreachable, malformed), **When**
   `client::connect()` is called, **Then** a `ShadowLinkError::ConnectionFailed`
   error is returned with a human-readable message.
3. **Given** valid URL but invalid credentials, **When** `client::connect()` is
   called, **Then** a `ShadowLinkError::AuthenticationFailed` error is returned.
4. **Given** an existing persisted session, **When** `client::connect()` is called
   with the same homeserver, **Then** the session is restored without
   re-authentication.

---

### User Story 2 — Room Operations (Priority: P2)

The **operator** (the family member who sets up ShadowLink) creates a private,
E2EE family room on first launch. It is invite-only — no one can discover or
join without an explicit invite. Other family members join via invite from
the operator. All members can view the list of joined rooms and leave rooms.

**Why this priority**: Rooms are the organizational unit for all family
communication. Chat, media, and location all happen within rooms. The family
room lifecycle is: operator creates → operator invites → members accept.

**Independent Test**: With two authenticated sessions on a local Synapse,
Session A (operator) creates a private E2EE room and invites Session B.
Session B accepts the invite. Both sessions verify the room appears in their
room list. The room is not discoverable via directory search.

**Acceptance Scenarios**:

1. **Given** an authenticated operator session, **When** `rooms::create_family_room(name)`
   is called, **Then** a new E2EE room is created (private, `join_rule: invite`),
   returned as `RoomInfo` with `is_home: true` flag, and persisted as the
   family room.
2. **Given** an authenticated operator session and a family room, **When**
   `rooms::set_home_room(room_id)` is called with an existing room, **Then**
   that room is pinned as the family room.
3. **Given** an authenticated session with a pending invite, **When**
   `rooms::accept_invite(room_id)` is called, **Then** the session joins the
   room and it appears in the room list.
4. **Given** an authenticated session in a room, **When**
   `rooms::invite_user(room_id, user_id)` is called, **Then** an invite is
   sent to the specified user.
5. **Given** an authenticated session with joined rooms, **When**
   `rooms::list_rooms()` is called, **Then** all joined rooms are returned
   with their IDs, names, invite-only status, and a flag indicating which
   room (if any) is the pinned family room.
6. **Given** an authenticated session in a room, **When**
   `rooms::leave_room(room_id)` is called, **Then** the session leaves the
   room and it is removed from the room list.

---

### User Story 3 — E2EE Messaging & Media (Priority: P3)

Family members can send and receive end-to-end encrypted text messages and
media attachments (pictures) within a room.

**Why this priority**: This is the primary communication mechanism. Without
it, the app is a room browser with no utility.

**Independent Test**: Two authenticated sessions in the same E2EE room.
Session A sends a text message and an image. Session B receives both and
verifies decryption. Session B replies; Session A receives the reply.

**Acceptance Scenarios**:

1. **Given** two sessions in the same E2EE room, **When** Session A calls
   `messaging::send_text(room_id, "Hello family!")`, **Then** Session B
   receives the decrypted message via a callback/stream.
2. **Given** two sessions in the same E2EE room, **When** Session A calls
   `messaging::send_media(room_id, image_bytes, "image/jpeg")`, **Then**
   Session B receives the decrypted media with correct MIME type.
3. **Given** an authenticated session, **When** `messaging::get_history(room_id, limit)`
   is called, **Then** the last N messages (text + media metadata) are
   returned in chronological order.
4. **Given** a message send attempt while offline, **When** connectivity
   is restored, **Then** the message is queued and sent automatically
   (via SDK built-in retry).
5. **Given** a message received in an unverified session (first contact
   with a device), **When** the session is later verified, **Then**
   previously received messages remain decryptable.

---

### User Story 4 — Location Sharing (Priority: P4)

Family members can share their current location as a static beacon and
optionally enable live location updates visible to other room members on
a map.

**Why this priority**: Differentiates ShadowLink from generic Matrix clients.
Provides family safety/coordination value without depending on proprietary
location services.

**Independent Test**: Two authenticated sessions in the same room. Session A
sends a static location beacon (lat/lng). Session B receives it and verifies
coordinates. Session A enables live location; Session B receives periodic
updates.

**Acceptance Scenarios**:

1. **Given** an authenticated session in a room, **When**
   `location::send_beacon(room_id, lat, lng, accuracy_m)` is called,
   **Then** a location event is sent to the room with the coordinates.
2. **Given** an authenticated session in a room, **When**
   `location::start_live(room_id, interval_secs)` is called, **Then**
   location updates are sent at the specified interval until stopped.
3. **Given** an authenticated session with active live location, **When**
   `location::stop_live(room_id)` is called, **Then** no further location
   updates are sent.
4. **Given** a session receiving location events, **When** a location event
   arrives, **Then** the coordinates, timestamp, and sender ID are delivered
   via a location callback.
5. **Given** an active live location session, **When** the app is backgrounded,
   **Then** live updates pause to conserve battery (resumable on foreground).

---

### User Story 5 — Session Management (Priority: P5)

The Rust core persists session state (credentials, room memberships, E2EE
keys, sync tokens) via matrix-rust-sdk built-in storage. Sessions survive
app restarts without full re-authentication.

**Why this priority**: Without session persistence, users would re-authenticate
and re-sync on every app launch, degrading the mobile experience.

**Independent Test**: Authenticate a session, send a message, kill the process,
restart, call `connect()`. Verify the session resumes, room list is intact,
and message history is available.

**Acceptance Scenarios**:

1. **Given** a previously authenticated session with SDK persistence enabled,
   **When** `client::connect()` is called after process restart, **Then** the
   session is restored without prompting for credentials.
2. **Given** a restored session, **When** `rooms::list_rooms()` is called,
   **Then** all previously joined rooms are returned.
3. **Given** a restored session, **When** `messaging::get_history(room_id, 20)`
   is called, **Then** cached message history is returned.
4. **Given** a restored session with an expired access token, **When**
   `client::connect()` is called, **Then** a `ShadowLinkError::SessionExpired`
   error is returned and re-authentication is required.
5. **Given** a session with active E2EE sessions, **When** the process
   restarts, **Then** Olm/Megolm key material is loaded from the SDK store
   and message decryption works immediately.

---

### Edge Cases

- What happens when the homeserver is unreachable during `connect()`?
  → Timeout after configurable duration (default 30s), return `ConnectionFailed`.
- What happens when a room invite is received while the app is backgrounded?
  → SDK sync delivers it on next foreground; stored in room list as "invited".
- What happens when a message is sent to a room the sender has already left?
  → SDK returns an error; mapped to `ShadowLinkError::NotInRoom`.
- What happens when media exceeds the homeserver's upload size limit?
  → SDK returns upload error; mapped to `ShadowLinkError::MediaTooLarge` with
    the server-reported limit in the error message.
- What happens when E2EE key verification fails (potential MITM)?
  → Message decryption fails with `ShadowLinkError::DecryptionFailed`; the
    Flutter layer displays a security warning.
- What happens when location services are disabled on the device?
  → `location::send_beacon()` returns `ShadowLinkError::LocationUnavailable`;
    the Flutter layer prompts the user to enable location.
- What happens when the SDK's SQLite store is corrupted?
  → `client::connect()` returns `ShadowLinkError::StorageError`; the Flutter
    layer can offer to clear and re-sync.
- What happens during initial sync on a large account (100+ rooms)?
  → Sync streams events progressively; `connect()` returns once initial sync
    completes; room list populates incrementally via callback.
- What happens when the debug room is full or gets deleted?
  → The crate re-creates the debug room with a new ID on next `connect()`
    if it no longer exists. Old events are lost — acceptable for diagnostics.
- What happens if the operator declines debug room consent?
  → `connect()` succeeds normally. The debug room is not created. No
    diagnostic events are emitted anywhere. `enable_debug_room(true)` can
    opt in later.
- What happens if no family room is configured?
  → `get_home_room()` returns `None`. The Flutter layer prompts the operator
    to create one. Other members see an empty room list until invited.

## Requirements

### Functional Requirements

- **FR-001**: System MUST provide a `client::connect(url, credentials)` function
  that establishes a Matrix session and returns a session handle.
- **FR-002**: System MUST persist session state (credentials, sync tokens, E2EE
  keys) via matrix-rust-sdk built-in storage.
- **FR-003**: System MUST restore a previous session when `connect()` is called
  without credentials if a persisted session exists.
- **FR-004**: System MUST support E2EE room creation with default encryption
  settings (Olm/Megolm).
- **FR-005**: System MUST support joining rooms via invite acceptance.
- **FR-006**: System MUST provide `messaging::send_text(room_id, text)` for
  sending encrypted text messages.
- **FR-007**: System MUST provide `messaging::send_media(room_id, bytes, mime_type)`
  for sending encrypted media attachments.
- **FR-008**: System MUST deliver incoming messages (text + media) to the Flutter
  layer via a registered callback or stream.
- **FR-009**: System MUST provide `location::send_beacon(room_id, lat, lng, accuracy)`
  for static location sharing.
- **FR-010**: System MUST provide `location::start_live(room_id, interval)` and
  `location::stop_live(room_id)` for live location sharing.
- **FR-011**: System MUST queue outgoing messages when offline and send them when
  connectivity is restored (delegated to SDK).
- **FR-012**: System MUST expose a unified `ShadowLinkError` enum covering all
  failure modes: connection, authentication, encryption, media, location, storage.
- **FR-013**: System MUST support configurable timeouts for network operations
  (default 30s for connect, 60s for media upload).
- **FR-014**: System MUST NOT log plaintext message content, key material, or
  access tokens at any log level.
- **FR-015**: System MUST route all network traffic through the matrix-rust-sdk
  HTTP client (no separate HTTP library).
- **FR-016**: System MUST expose room list as a `Vec<RoomInfo>` with room ID,
  name, member count, and encryption status.
- **FR-017**: System MUST expose message history as a paginated list with
  configurable limit.
- **FR-018**: System MUST verify E2EE device keys before sending messages to
  new devices (TOFU — Trust On First Use — via SDK default behavior).
- **FR-019**: System MUST expose `rooms::invite_user(room_id, user_id)` for
  inviting family members.
- **FR-020**: System MUST expose `rooms::leave_room(room_id)` for leaving rooms.
- **FR-021**: System MUST create a dedicated **debug room** on first session
  `connect()` after user consent — a private, invite-only E2EE room named
  `"ShadowLink Debug"` where the operator is the sole member.
- **FR-022**: System MUST emit structured, human-readable diagnostic events to
  the debug room for: sync status changes, connection failures, E2EE warnings
  (new device, key mismatch), decryption failures, location service transitions,
  and session lifecycle events (login, expire, restore).
- **FR-023**: Debug room events MUST contain both a human-readable text summary
  AND machine-parseable metadata (event type, module, severity, timestamp,
  error code). Metadata is appended as a JSON block after the text.
- **FR-024**: Debug room events MUST NOT contain PII — no message content, no
  coordinates, no access tokens, no key material. Operator consent is required
  before the debug room is created, and it can be disabled at any time.
- **FR-025**: System MUST expose `get_home_room(handle)` to retrieve the pinned
  family room ID (or `None` if not configured).
- **FR-026**: System MUST expose `enable_debug_room(handle, enabled)` to toggle
  the debug room at runtime without re-authenticating.

### Key Entities

- **Session**: Represents an authenticated Matrix client session. Contains
  homeserver URL, user ID, access token, device ID. Persisted via SDK store.
- **Room**: An E2EE-encrypted Matrix room. Contains room ID, name, member list,
  encryption state. Joined rooms are tracked by the SDK.
- **Message**: An encrypted communication unit within a room. Contains sender
  ID, timestamp, body (text or media metadata), and event ID. Decrypted by
  the SDK before delivery to the Flutter layer.
- **LocationEvent**: A specialized event containing latitude, longitude,
  accuracy radius, timestamp, and a flag indicating static vs. live.
  Transmitted as a Matrix custom event type (`org.shadowlink.location`).
- **MediaAttachment**: An encrypted file uploaded to the homeserver's media
  repository. Contains MIME type, file size, decryption metadata (key, IV,
  hashes), and a content URI (`mxc://`).

## Success Criteria

### Measurable Outcomes

- **SC-001**: A developer can clone the repo, run `cargo build`, and compile
  the crate with zero errors on the first attempt.
- **SC-002**: `client::connect()` against a local Synapse homeserver completes
  in under 5 seconds (excluding initial E2EE key upload).
- **SC-003**: A text message sent by Session A is delivered to Session B in
  under 2 seconds on localhost.
- **SC-004**: Session state survives a process restart — 100% of previously
  joined rooms and cached messages are accessible without re-authentication.
- **SC-005**: All 5 user stories are independently testable against a local
  Synapse homeserver with `cargo test`.
- **SC-006**: `cargo llvm-cov` reports at least 80% line coverage for the
  crate (excluding FFI boilerplate).
- **SC-007**: `cargo clippy -- -D warnings` reports zero warnings.
- **SC-008**: `gitleaks detect` reports zero findings.

## Constraints

- **CO-001**: The crate is **dual-licensed MIT OR Apache-2.0** — all contributions
  must be compatible with both licenses. A single `LICENSE` file is replaced by
  `LICENSE-MIT` and `LICENSE-APACHE` per Rust ecosystem convention.
- **CO-002**: The consuming Flutter app is proprietary (all rights reserved) and
  lives in a separate private repository. No Flutter code ever enters this repo.
- **CO-003**: Breaking FFI changes require a MAJOR SemVer bump. MINOR and PATCH
  bumps must preserve backward compatibility at the FFI boundary.

## Assumptions

- The consuming Flutter app handles UI rendering, map display, and platform
  permissions — the Rust crate is a pure data/state layer.
- The user provides their own Matrix homeserver (self-hosted Synapse or
  managed provider). The crate ships with zero hardcoded server URLs.
- matrix-rust-sdk 0.7+ is the target SDK version, providing built-in E2EE
  (Olm/Megolm), sync, and persistence.
- `tokio` is the async runtime, exposed through FFI entry points managed
  by flutter_rust_bridge or raw dart:ffi (decision in Plan phase).
- Media uploads delegate to the SDK's media content repository API; the
  crate does not implement its own HTTP upload logic.
- Location events use a custom Matrix event type (`org.shadowlink.location`)
  for interoperability; standard `m.location` (MSC3488) is evaluated in Plan.
- Initial development and testing target Linux (local Synapse via Docker);
  Android cross-compilation is deferred to Phase 6+.
- iOS support is out of scope for the initial Rust crate release — the FFI
  layer design accommodates it but does not validate.
