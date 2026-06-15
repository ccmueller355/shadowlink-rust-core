# Data Model: Family Room Semantics

**Feature**: 003-family-room | **Date**: 2026-06-15

## Entities

### RoomInfo (extended)

Extended with two new fields. Existing fields unchanged.

| Field | Type | Source | Description |
|-------|------|--------|-------------|
| `room_id` | `String` | Matrix API | Opaque room ID (`!localpart:domain`) |
| `name` | `Option<String>` | Matrix API (`m.room.name`) | Human-readable display name |
| `alias` | `Option<String>` | **NEW** — Matrix API (`m.room.canonical_alias`) | Canonical alias (`#localpart:domain`). `None` if no alias set or alias creation failed. |
| `member_count` | `u64` | Matrix API (`m.room.member`) | Number of joined members |
| `encrypted` | `bool` | Matrix API (`m.room.encryption`) | Whether E2EE is enabled on the room |
| `is_home` | `bool` | **NEW** — client-side | `true` if this room is the pinned family home room. Exactly one room per session may have this. |
| `state` | `RoomState` | Matrix API | `Joined`, `Invited`, or `Left` |

**Validation rules**:
- `is_home` is `true` for at most one room in `list_rooms()` output
- `is_home` is `false` by default for rooms created via generic `create_room()`
- `alias` is populated from the room's canonical alias state event, or `None`

### FamilyRoom (persisted)

Stored in `shadowlink_data/session.json` as part of `StoredSession`.

| Field | Type | Persistence | Description |
|-------|------|-------------|-------------|
| `home_room_id` | `Option<String>` | `session.json` → `StoredSession.home_room_id` | The room ID of the pinned family room. `None` if no family room configured. |

**Validation rules**:
- Set by `create_family_room()` and `set_home_room()`
- Cleared if the home room is left or deleted (best-effort detection)
- Survives session restarts — loaded alongside the Matrix session

### StoredSession (extended)

Adds one field to the existing persistence struct in `src/client.rs`.

```rust
struct StoredSession {
    homeserver_url: String,
    #[serde(flatten)]
    session: MatrixSession,
    home_room_id: Option<String>,  // NEW
}
```

### DiagnosticEvent

Emitted to the `"ShadowLink Debug"` room when debug mode is enabled.

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `event` | `String` | Yes | Event type: `sync_error`, `connection_failed`, `e2ee_warning`, `decryption_failed`, `location_transition`, `session_lifecycle` |
| `module` | `String` | Yes | Source module: `client`, `rooms`, `messaging`, `location`, `encryption` |
| `severity` | `String` | Yes | `info`, `warn`, `error` |
| `timestamp` | `String` | Yes | ISO 8601 timestamp (UTC) |
| `error_code` | `Option<String>` | No | `ShadowLinkError` variant name, if applicable |
| `detail` | `String` | Yes | Human-readable summary (no PII) |

**Format**: Sent as an `m.room.message` with `msgtype: m.text`. Body is `{detail}\n\n```json\n{json_block}\n````.

**PII constraints** (per spec FR-013):
- MUST NOT include message body content
- MUST NOT include coordinates
- MUST NOT include access tokens or key material
- MUST NOT include user IDs beyond the operator's own

### DebugRoom (session-scoped)

| Field | Type | Persistence | Description |
|-------|------|-------------|-------------|
| `debug_room_id` | `Option<String>` | Session memory only (not persisted) | Room ID of the `"ShadowLink Debug"` room |
| `debug_room_enabled` | `bool` | Session memory only | Whether diagnostic events are being emitted |

**Lifecycle**:
- Created on first `enable_debug_room(true)` per session
- Re-created if deleted externally (next diagnostic event triggers creation)
- Never deleted by the core — toggle only stops emission
- Not persisted across restarts (Flutter layer re-enables on each connect)

## State Transitions

### Family Room Lifecycle

```
[No family room] ──create_family_room()──→ [Room created, is_home=true, persisted]
[No family room] ──set_home_room(id)─────→ [Existing room pinned, is_home=true, persisted]
[Has family room] ──create_family_room()──→ [Old room unpinned, new room pinned]
[Has family room] ──set_home_room(id)─────→ [Old room unpinned, new room pinned]
[Has family room] ──leave_room(home_id)───→ [Room left, is_home cleared from persistence]
```

### Debug Room Lifecycle

```
[Disabled] ──enable_debug_room(true)──→ [Room created, enabled=true, emitting]
[Enabled]  ──enable_debug_room(false)─→ [Enabled=false, room not deleted, not emitting]
[Enabled, room deleted externally] ──next diagnostic──→ [Room re-created, emitting resumes]
```
