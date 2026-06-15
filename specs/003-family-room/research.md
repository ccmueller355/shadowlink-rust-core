# Research: Family Room Semantics

**Feature**: 003-family-room | **Date**: 2026-06-15

## Research Tasks

### RT-001: `join_rule: invite` semantics vs current `RoomPreset::PrivateChat`

**Verdict**: No change needed. `RoomPreset::PrivateChat` already sets `join_rule: invite`
per the Matrix spec. The current `create_room` and the new `create_family_room` use the
same underlying room creation API. The difference is semantic — `create_family_room` adds
alias generation, home room persistence, and the `is_home: true` flag.

**Matrix spec behaviour for `RoomPreset::PrivateChat`**:
- `join_rule`: `invite`
- `history_visibility`: `shared`
- `guest_access`: `forbidden`

**Alternatives considered**:
- `RoomPreset::TrustedPrivateChat` — adds `invite: [inviter_mxid]` power level restriction.
  Rejected. Family members should be able to invite additional members (grandparents, etc.).
- Manual `initial_state` with explicit `m.room.join_rules` — overkill. The preset handles it.

### RT-002: Room alias setting via `create_room::v3::Request`

**Verdict**: `create_room::v3::Request` already exposes `room_alias_name: Option<String>`.
This field sets the localpart of a room alias (`#<localpart>:<homeserver>`). The homeserver
generates the full alias.

**Implementation**:
```rust
request.room_alias_name = Some(alias_localpart);
```
If the homeserver rejects the alias (collision, namespace restriction), room creation may
fail or the alias may be rejected silently depending on server policy. Spec handles this:
alias is best-effort, room still succeeds with `alias: None`.

**Alias derivation rules** (per Matrix spec localpart constraints `[a-z0-9._=-]`):
- Lowercase the display name
- Replace spaces with hyphens
- Strip characters outside `[a-z0-9._=-]`
- Truncate to 255 characters
- Strip leading/trailing hyphens and dots

Example: `"The Smith Family"` → `"the-smith-family"`

### RT-003: Home room ID persistence

**Verdict**: Extend the existing `StoredSession` struct with an `Option<String>` field.
The session JSON already persists between restarts — no new I/O path needed.

**Implementation**:
```rust
struct StoredSession {
    homeserver_url: String,
    #[serde(flatten)]
    session: MatrixSession,
    home_room_id: Option<String>,  // NEW
}
```

**Alternatives considered**:
- Separate `home_room.json` file — adds I/O complexity for a single string. Rejected.
- Stored in SQLite via SDK — no SDK API for arbitrary key-value storage. Rejected.

### RT-004: Debug room lifecycle

**Verdict**: The debug room toggle is session-scoped state. Store the room ID and enabled
flag on the `Session` struct. The debug room is created on first `enable_debug_room(true)`
call per session — it is NOT persisted across restarts (per spec assumption: opt-in per
session).

**State**:
```rust
pub(crate) struct Session {
    // ... existing fields ...
    pub debug_room_id: Option<String>,    // NEW
    pub debug_room_enabled: bool,         // NEW
}
```

**Diagnostic event emission**: Add a private `emit_diagnostic()` function that checks
`debug_room_enabled`, re-creates the debug room if needed (deleted externally), formats
the JSON metadata block, and sends the message. Called from sync error paths, connection
failure handlers, and E2EE warning hooks.

### RT-005: `RoomInfo` struct extension — FFI safety

**Verdict**: Add two owned fields (`is_home: bool`, `alias: Option<String>`). Both are
FFI-safe (bool is primitive, `Option<String>` is an owned String). No breaking change
to existing `RoomInfo` consumers — new fields are additive and have sensible defaults
(`false`, `None`).

```rust
pub struct RoomInfo {
    pub room_id: String,
    pub name: Option<String>,
    pub alias: Option<String>,   // NEW
    pub member_count: u64,
    pub encrypted: bool,
    pub is_home: bool,           // NEW
    pub state: RoomState,
}
```

### RT-006: `set_home_room` E2EE enforcement

**Verdict**: Call `room.enable_encryption().await` before pinning if `!room.is_encrypted()`.
The SDK's `BaseRoom::enable_encryption()` sends the necessary state events. If the
homeserver rejects it, surface `OperationFailed`.

**Note**: `enable_encryption()` is idempotent — calling it on an already-encrypted room
is a no-op per the SDK. Safe to call unconditionally, but checking first avoids an
unnecessary network round-trip.

## Constitution Impact

| Principle | Impact |
|-----------|--------|
| I. Clean Separation | None — all changes are internal to `src/rooms.rs`, `src/client.rs`, `src/ffi.rs` |
| II. Local-First Privacy | None — home room ID persisted locally, no server-side state |
| III. Minimal API Surface | Adds 3 public functions (`create_family_room`, `set_home_room`, `get_home_room`) and 1 toggle (`enable_debug_room`). Justified by the spec. |
| IV. Test-First | All functions will have SpecKit-derived tests before implementation |
| V. Battery & Permission | None — no new background work, no new network patterns |
| VI. CI Pipeline | No pipeline changes needed — existing gates apply |

No constitution violations.
