# Feature Specification: Family Room Semantics

**Feature Branch**: `003-family-room`

**Created**: 2026-06-15

**Status**: Draft

**Input**: User description: "Implement the family room concept as initially intended in
spec 001-shadowlink-core — create_family_room with is_home flag, room alias
generation, set_home_room, get_home_room, and debug room toggle. The current
create_room is a generic E2EE room with no ShadowLink-specific semantics."

## User Scenarios & Testing *(mandatory)*

### User Story 1 — Operator Creates the Family Room (Priority: P1)

The **operator** (the family member who sets up ShadowLink) runs the app for the first time.
After connecting to their homeserver, they configure the family's home room — a private,
invite-only, E2EE-encrypted room. The room gets a human-readable alias derived from its
display name (e.g. "The Smith Family" → `#the-smith-family:homeserver.tld`). The room is
persisted as *the* family room so the app knows which room to show on startup.

**Why this priority**: Without a dedicated family room concept, the app is a generic Matrix
client. The family room is the root of all family communication — it's what the Flutter
layer uses to decide whether to show onboarding ("set up your family room") or the main
chat view. Every other feature (messaging, location, media) happens inside this room.

**Independent Test**: Start a local Synapse, connect two sessions, call
`create_family_room("The Smith Family")` from session A, verify:
- Room is created with `join_rule: invite` (not discoverable)
- `RoomInfo.is_home` is `true`
- Alias `#the-smith-family:localhost` is set
- `get_home_room()` returns the room
- Session B cannot discover the room without an invite

**Acceptance Scenarios**:

1. **Given** an authenticated operator session with no existing family room, **When**
   `rooms::create_family_room("The Smith Family")` is called, **Then** a private invite-only
   E2EE room is created, returned with `is_home: true`, an alias `#the-smith-family:<domain>`
   is set, and the room ID is persisted so `get_home_room()` returns it.
2. **Given** an authenticated operator session that already has a family room, **When**
   `rooms::create_family_room("Another Room")` is called, **Then** the new room replaces
   the old one as the family room (old room is not deleted, just unpinned).
3. **Given** an authenticated non-operator session (family member), **When**
   `rooms::create_family_room("My Room")` is called, **Then** it succeeds — any
   authenticated user can create a family room, but the semantic intent is operator-only
   (enforced by the Flutter layer, not the core).
4. **Given** an invalid room name (empty string, >255 characters), **When**
   `rooms::create_family_room(name)` is called, **Then** an `OperationFailed` error is
   returned with a descriptive message.
5. **Given** a homeserver where room creation is disabled by admin policy, **When**
   `rooms::create_family_room("The Smith Family")` is called, **Then** an `OperationFailed`
   error is returned with the server's rejection reason. The caller can fall back to
   `set_home_room` with an existing room.

---

### User Story 2 — Family Members View and Navigate to the Home Room (Priority: P2)

Any family member with an authenticated session can see which room is pinned as the family
home room, navigate to it, or re-pin to a different room if needed. The `list_rooms` output
marks the home room so the UI can highlight it.

**Why this priority**: After the family room exists, every user needs to know which room
is "home." Without the `is_home` flag, the Flutter layer has no way to distinguish the
family room from other rooms the user may have joined.

**Independent Test**: Create a family room, join a second generic room, call `list_rooms()`.
Verify exactly one room has `is_home: true` and `get_home_room()` returns that same room.

**Acceptance Scenarios**:

1. **Given** an authenticated session with a pinned family room, **When**
   `rooms::list_rooms()` is called, **Then** the family room has `is_home: true` and all
   other rooms have `is_home: false`.
2. **Given** an authenticated session with a pinned family room, **When**
   `rooms::get_home_room()` is called, **Then** the home room's `RoomInfo` is returned
   (with `is_home: true`).
3. **Given** an authenticated session with no family room configured, **When**
   `rooms::get_home_room()` is called, **Then** `None` is returned — the Flutter layer
   knows to show the "set up family room" onboarding flow.
4. **Given** an authenticated session in an existing room, **When**
   `rooms::set_home_room(existing_room_id)` is called, **Then** that room is pinned as
   the family room, its `is_home` flag becomes `true`, and any previously pinned room
   loses its `is_home` flag.
5. **Given** an authenticated session in an existing room created outside ShadowLink
   (e.g., via Element) that lacks E2EE, **When** `set_home_room(room_id)` is called,
   **Then** encryption is enabled on the room before pinning, and the returned
   `RoomInfo` has `encrypted: true` and `is_home: true`.
6. **Given** an authenticated session in an existing room without E2EE, and the
   homeserver rejects the encryption request, **When** `set_home_room(room_id)` is
   called, **Then** the call fails with `OperationFailed` — ShadowLink requires E2EE
   on the family room.

---

### User Story 3 — Operator Enables Diagnostic Debug Room (Priority: P3)

The operator (or any authenticated user) can opt into a diagnostic debug room named
`"ShadowLink Debug"`. When enabled, the core emits structured diagnostic events — sync
status changes, connection failures, E2EE warnings, decryption failures, location service
transitions, and session lifecycle events — as human-readable messages with machine-parseable
JSON metadata appended. The debug room is private, invite-only, with the operator as the
sole member. It can be toggled on/off at any time without re-authenticating.

**Why this priority**: Diagnostic visibility is critical for troubleshooting real-world
Matrix deployments. Without it, debugging federation issues, E2EE key problems, or sync
failures requires external logging infrastructure. The debug room provides in-band,
E2EE-encrypted diagnostics that travel with the session.

**Independent Test**: Connect a session, call `enable_debug_room(true)`, trigger a
controlled failure (e.g., send to nonexistent room), verify a diagnostic event appears
in the `"ShadowLink Debug"` room with both human-readable text and JSON metadata.

**Acceptance Scenarios**:

1. **Given** an authenticated session with debug room disabled (default), **When**
   `enable_debug_room(true)` is called, **Then** a private invite-only E2EE room named
   `"ShadowLink Debug"` is created (if it doesn't exist), and subsequent diagnostic
   events are emitted to it.
2. **Given** an authenticated session with debug room enabled, **When** a sync failure
   occurs, **Then** a message is sent to the debug room containing: a human-readable
   summary (e.g. "Sync failed: connection refused") and a JSON metadata block with
   `{"event": "sync_error", "module": "client", "severity": "error", "timestamp": ...,
   "error_code": "ConnectionFailed"}`.
3. **Given** an authenticated session with debug room enabled, **When**
   `enable_debug_room(false)` is called, **Then** diagnostic events stop. The debug room
   itself is not deleted — it remains for historical reference.
4. **Given** an authenticated session with debug room enabled and the debug room has been
   deleted externally, **When** the next diagnostic event fires, **Then** the debug room
   is re-created with a new room ID. Old events in the deleted room are lost (acceptable).
5. **Given** an authenticated session with debug room enabled, **When** a diagnostic event
   is emitted, **Then** the event MUST NOT contain PII — no message content, no coordinates,
   no access tokens, no key material. Metadata is limited to event type, module, severity,
   timestamp, and error code.

---

## Real-World Homeserver Verification *(mandatory)*

The local Docker Synapse used in CI validates protocol correctness but cannot test
real-world behaviors: TLS certificate chains, federation, homeserver-specific room
creation policies, alias namespace restrictions, rate limiting, or anti-abuse heuristics.
A real homeserver test is required to surface these. However, public servers (notably
`matrix.org`) aggressively rate-limit and auto-ban clients that exhibit bot-like behavior —
creating rooms in rapid succession, sending rapid-fire messages, or cycling sessions.
The verification strategy must produce useful data without triggering abuse detection.

### Test Homeserver

| Option | Risk | Recommendation |
|--------|------|----------------|
| `matrix.org` | High — auto-bans bots, strict rate limits | **Not for automated CI**. Manual one-shot tests only. |
| Self-hosted Synapse on VPS | None — full control | **Preferred** for CI. A $5 VPS with Docker Compose Synapse. |
| `conduwuit` (lightweight Rust homeserver) | None — full control | Alternative for CI diversity. Tests Matrix spec compliance against a non-Synapse implementation. |
| Managed provider (etke.cc, Element One) | Low — paid accounts have better tolerance | Acceptable for manual testing with a real federated account. |

**Recommendation**: Self-hosted VPS Synapse for automated CI (full control, real TLS, real
federation if desired). `matrix.org` for one-shot manual verification only.

### Anti-Ban Protocol

When testing against any public or federated homeserver, the test suite MUST:

1. **Use dedicated test accounts**. Account localparts MUST be prefixed `shadowlink-test-`
   (e.g., `@shadowlink-test-familyop:matrix.org`). This identifies the traffic as a test
   suite and lets homeserver admins contact the developer before banning.
2. **Set a descriptive User-Agent**. The SDK's HTTP client MUST identify itself as
   `ShadowLinkTestSuite/0.2` (not the default `matrix-rust-sdk` UA). This is configured
   via `Client::builder().user_agent()`.
3. **Space room creation apart**. No more than 1 room creation per 30 seconds. Tests that
   create rooms MUST include a `tokio::time::sleep(Duration::from_secs(30))` between
   creation calls.
4. **Clean up after every test run**. Every test MUST leave all rooms it joined and delete
   all rooms it created. Best-effort cleanup — if the homeserver is unreachable during
   teardown, document the orphaned rooms.
5. **Do not send to public rooms**. All test rooms are private (`join_rule: invite`).
   Never send test messages to `#general:matrix.org` or any public alias.
6. **Rate-limit message sends**. No more than 1 message per 5 seconds per room. Batch
   message tests accordingly.
7. **Never test federation without explicit intent**. If testing alias behavior or room
   discovery across servers, use two self-hosted homeservers that federate with each
   other — never test federation against `matrix.org` without prior arrangement.

### Behaviors to Verify Against a Real Server

These behaviors are invisible against a local Docker Synapse but critical for real-world
correctness:

| Behavior | Why it matters | How to verify |
|----------|---------------|---------------|
| **TLS handshake** | Local Synapse runs plaintext. Real servers require TLS 1.3 with valid certs. | `connect()` to a real homeserver; verify no `ConnectionFailed` from TLS errors. |
| **Room creation policy rejection** | Some servers disable room creation or restrict alias namespaces. | Call `create_family_room` on a server with restricted room creation; verify `OperationFailed` carries the server's rejection reason (not a generic error). |
| **Alias namespace enforcement** | Homeservers may reject aliases outside their namespace or with reserved prefixes. | Call `create_family_room` with a name that maps to a restricted alias (e.g., `#admin-*`); verify the room still succeeds, alias is `None`, no panic. |
| **Rate-limit response (429)** | Real servers return HTTP 429 with `retry_after_ms`. The SDK must propagate this. | Trigger a rate limit (rapid room creation); verify the error is surfaced, not swallowed or retried into a ban. |
| **Session expiry across restarts** | Real tokens expire. `restore_session()` must handle `SessionExpired` gracefully. | Connect, wait >24h (or use a short-lived token), call `restore_session()`; verify `SessionExpired` is returned, not a panic or hang. |
| **E2EE key upload against real server** | Key upload failures are invisible on localhost. Real servers may reject large key counts. | Bootstrap cross-signing against a real server; verify `bootstrap_cross_signing()` succeeds or surfaces a clear error. |
| **Debug room diagnostics with real latency** | Diagnostic events must not block the sync loop. Real network latency exposes timing bugs. | Enable debug room, trigger a sync error (disconnect network briefly); verify diagnostic event arrives within 10s and sync recovers without hanging. |

### CI Integration

- Self-hosted VPS Synapse is the **CI target** for automated real-server tests.
- Tests that require a real server are gated behind a `#[cfg(feature = "real-server")]`
  feature flag (or `#[ignore]` with an env var `SHADOWLINK_REAL_SERVER=1`).
- These tests do **not** run on every PR — they run on a schedule (nightly) or on-demand
  via workflow dispatch.
- Manual `matrix.org` tests are documented in `specs/003-family-room/quickstart.md` with
  step-by-step instructions and anti-ban warnings.

---

### Edge Cases

- What happens when `create_family_room` is called with a name containing characters
  invalid for a Matrix alias (spaces, uppercase, Unicode)?
  → The alias is derived by lowercasing, replacing spaces with hyphens, and stripping
  characters outside `[a-z0-9._=-]`. The display name is stored as-is.
- What happens when the homeserver rejects an alias (collision, policy)?
  → `create_family_room` succeeds with the room but logs a warning; the alias is `None`.
  The room is still the family room — aliases are best-effort.
- What happens when `set_home_room` is called with a room the user is not a member of?
  → Returns `NotInRoom` error.
- What happens when `set_home_room` is called with a room that doesn't exist?
  → Returns `RoomNotFound` error.
- What happens when the debug room's JSON metadata exceeds the Matrix event size limit?
  → Metadata is truncated to fit within the 65536-byte event limit. A `truncated: true`
  flag is added to the JSON.
- What happens during session restore — is the family room ID preserved?
  → Yes. The family room ID is persisted alongside the session (in `session.json` or an
  adjacent file). `get_home_room()` returns it immediately after `restore_session()`.
- What happens if the persisted home room no longer exists on the server?
  → `get_home_room()` returns the cached `RoomInfo` with `state: Left` (or similar).
  The Flutter layer can prompt the operator to create or pin a new one.
- What happens when the homeserver rejects room creation entirely (admin-disabled,
  invite-only server, or policy restriction)?
  → `create_family_room` returns `OperationFailed` with the server's rejection reason.
  The caller can fall back to `set_home_room` with an existing room, or switch
  homeservers. The core does not retry — policy decisions are surfaced immediately.
- What happens when the user already has a suitable room from outside ShadowLink
  (e.g., created via Element, or a pre-existing family chat)?
  → The user calls `set_home_room(existing_room_id)` to adopt it. No re-creation
  needed. `get_home_room()` returns it with `is_home: true`. The core checks that
  the user is a member (`Joined`) and that E2EE is enabled — if encryption is
  missing, it is enabled during adoption.
- What happens when `set_home_room` is called on a room that is not E2EE-encrypted?
  → The core attempts to enable encryption on the room before pinning it. If
  encryption cannot be enabled (server rejection), the call fails with
  `OperationFailed` — ShadowLink requires E2EE on the family room.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST provide `rooms::create_family_room(handle, name) -> Result<RoomInfo>`
  that creates a private, invite-only (`join_rule: invite`), E2EE-encrypted room and returns
  it with `is_home: true`.
- **FR-002**: System MUST derive a Matrix room alias from the display name — lowercase,
  spaces→hyphens, strip invalid characters — and set it on the room via
  `create_room::v3::Request::room_alias_name`. Alias setting is best-effort (non-fatal on
  failure).
- **FR-003**: System MUST persist the family room ID so `get_home_room()` survives session
  restarts.
- **FR-004**: System MUST provide `rooms::set_home_room(handle, room_id) -> Result<RoomInfo>`
  that pins an existing joined room as the family room, unpinning any previous home room.
  If the target room is not E2EE-encrypted, encryption MUST be enabled before pinning
  (fail with `OperationFailed` if the server rejects encryption).
- **FR-004a**: When `create_family_room` fails because the homeserver rejected room creation
  (policy, admin restriction), the error MUST carry the server's rejection reason. The
  system MUST surface this immediately — no retry, no fallback room creation. The caller
  is expected to use `set_home_room` with an existing room or switch homeservers.
- **FR-005**: System MUST provide `rooms::get_home_room(handle) -> Result<Option<RoomInfo>>`
  that returns the pinned family room or `None`.
- **FR-006**: The `RoomInfo` struct MUST include an `is_home: bool` field indicating whether
  the room is the pinned family room.
- **FR-007**: The `RoomInfo` struct MUST include an `alias: Option<String>` field for the
  room's canonical alias (e.g. `#the-smith-family:localhost`).
- **FR-008**: `rooms::list_rooms()` MUST set `is_home: true` on the pinned family room and
  `false` on all others.
- **FR-009**: System MUST provide `enable_debug_room(handle, enabled: bool) -> Result<()>`
  to toggle diagnostic event emission at runtime.
- **FR-010**: When debug room is enabled and no `"ShadowLink Debug"` room exists, the system
  MUST create it — private, invite-only, E2EE-encrypted, with the operator as sole member.
- **FR-011**: Diagnostic events MUST be structured as text messages with a JSON metadata
  block appended, containing `event` (type), `module` (source), `severity` (info/warn/error),
  `timestamp` (ISO 8601), and `error_code` (where applicable).
- **FR-012**: Diagnostic events MUST cover: sync status changes, connection failures, E2EE
  warnings (new/unverified device, key mismatch), decryption failures, location service
  transitions, and session lifecycle events (login, expiry, restore).
- **FR-013**: Diagnostic events MUST NOT contain PII — no message body content, no
  coordinates, no access tokens, no key material, no user IDs beyond the operator.
- **FR-014**: The `create_room` function (existing generic creator) MUST continue to work
  unchanged — `create_family_room` is additive, not a replacement.

### Key Entities

- **Family Room**: A private, invite-only E2EE Matrix room serving as the family's home
  room. Distinguished by `is_home: true` in `RoomInfo`. Has an optional canonical alias.
  Persisted across sessions. Created by the operator, joined by family members via invite.
- **RoomInfo (enhanced)**: Extended with `is_home: bool` and `alias: Option<String>` fields
  alongside existing `room_id`, `name`, `member_count`, `encrypted`, `state`.
- **Diagnostic Event**: A structured message sent to the `"ShadowLink Debug"` room. Contains
  a human-readable summary line followed by a JSON metadata block (`event`, `module`,
  `severity`, `timestamp`, `error_code`). Never contains PII. Only emitted when debug room
  is enabled.
- **Debug Room**: A private, invite-only E2EE room named `"ShadowLink Debug"` serving as the
  operator's diagnostic channel. Sole member is the operator. Created on first enable, re-created
  if deleted, toggleable at runtime.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: A developer can call `create_family_room("The Smith Family")` and receive a
  `RoomInfo` with `is_home: true` and an alias matching `#the-smith-family:<domain>`.
- **SC-002**: After session disconnect and `restore_session()`, `get_home_room()` returns the
  previously pinned family room (not `None`).
- **SC-003**: `list_rooms()` returns exactly one room with `is_home: true` when a family room
  is configured, and zero when none is configured.
- **SC-004**: Calling `enable_debug_room(true)` followed by a sync failure produces a
  diagnostic event in the `"ShadowLink Debug"` room within 10 seconds of the error.
- **SC-005**: All existing tests (US1–US5, spec_contracts, integration) continue to pass
  without modification — the `create_room` generic function is unchanged.
- **SC-006**: `cargo clippy -- -D warnings` and `cargo fmt -- --check` pass with zero
  diagnostics.

## Assumptions

- The family room ID is persisted in `shadowlink_data/session.json` (alongside existing
  session data) rather than a separate file, minimizing I/O and keeping the session
  self-contained.
- Alias generation follows Matrix spec rules: localpart is `[a-z0-9._=-]` only, lowercase.
  Unicode characters and emoji are stripped. Spaces become hyphens. The result is truncated
  to 255 characters if needed.
- The debug room toggle applies to the current session only — it is not persisted across
  restarts (the Flutter layer can re-enable it on each `connect()` if desired). This keeps
  the debug room opt-in per session.
- `set_home_room` only accepts rooms the user is currently a member of (`state: Joined`).
  Invited or left rooms cannot be pinned.
- The generic `create_room` function retains its current semantics (`RoomPreset::PrivateChat`) —
  `create_family_room` is a separate function with distinct semantics (`join_rule: invite`,
  alias generation, persistence).
- The `RoomInfo` struct is FFI-safe (all owned fields, no lifetimes) — the new `alias` and
  `is_home` fields follow the same pattern.
