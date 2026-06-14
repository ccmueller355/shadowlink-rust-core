# Data Model — CLI Integration

**Feature**: 002-cli-integration | **Date**: 2026-06-14

All entities are defined by `shadowlink-rust-core` and consumed by the CLI. The CLI adds no new types — it wraps core types for display.

## Entity: Session

Represents an authenticated Matrix client session.

| Field | Type | Source | Description |
|-------|------|--------|-------------|
| `homeserver_url` | `String` | CLI input | User-provided Matrix homeserver URL |
| `user_id` | `String` | Core (login response) | Matrix user ID (e.g., `@alice:example.com`) |
| `device_id` | `String` | Core (login response) | Matrix device ID |
| `access_token` | `String` | Core (login response, never displayed) | Opaque auth token |
| `session_file` | `PathBuf` | CLI (CWD-relative) | Path to `shadowlink_data/session.json` |

**Lifecycle**:
1. `connect` → Core authenticates, writes session to disk
2. All other commands → `restore_session()` reads from disk
3. `disconnect` → Deletes session file + SQLite store

**Validation**: None at CLI layer. Core validates credentials against the homeserver.

**Persistence**: `shadowlink_data/session.json` — JSON file written by core's `client::connect()`.

---

## Entity: Room

A Matrix room visible to the authenticated user.

| Field | Type | Source | Description |
|-------|------|--------|-------------|
| `room_id` | `String` | Core (`RoomInfo`) | Matrix room ID (`!localpart:domain`) |
| `display_name` | `Option<String>` | Core (`RoomInfo`) | Human-readable room name |
| `state` | `RoomState` | Core (`RoomState` enum) | `Joined`, `Invited`, or `Left` |
| `encryption` | `bool` | Core (implicit) | Always `true` — all rooms are E2EE |

**Display format** (CLI `list-rooms`):
```
[joined]  Family Chat     (!abc123:example.com)
[invited] Neighborhood    (!def456:example.com)
```

**Operations**: Create, list, invite, accept, leave — all delegated to core.

---

## Entity: Message

An E2EE-decrypted message event.

| Field | Type | Source | Description |
|-------|------|--------|-------------|
| `sender` | `String` | Core (`Message`) | Matrix user ID of sender |
| `content` | `MessageContent` | Core (enum) | `Text`, `Media`, or `Location` |
| `timestamp` | `DateTime` | Core (event origin_server_ts) | When the event was sent |
| `room_id` | `String` | Core (event room_id) | Room the message belongs to |

**Display format** (CLI `listen`):
```
@alice:example.com > Hello, world!
@bob:example.com > [shared a photo: beach.jpg]
```

**MessageContent variants** (consumed from core):

| Variant | CLI display |
|---------|-------------|
| `Text { body }` | `<sender> > <body>` |
| `Media { filename, .. }` | `<sender> > [shared media: <filename>]` |
| `Location { lat, lng, .. }` | `<sender> > [location: <lat>, <lng>]` |

---

## Entity: Location Beacon

A location event sent to a room.

| Field | Type | Source | Description |
|-------|------|--------|-------------|
| `latitude` | `f64` | CLI input | -90.0 .. 90.0 |
| `longitude` | `f64` | CLI input | -180.0 .. 180.0 |
| `accuracy` | `Option<f64>` | CLI input (optional) | Accuracy in meters |
| `timestamp` | `Option<DateTime>` | CLI input (optional) | When the location was captured |

**Validation** (FR-019):
- Latitude: -90.0 ≤ value ≤ 90.0
- Longitude: -180.0 ≤ value ≤ 180.0

Validation happens at CLI layer before calling core's `share_location()`.

---

## Entity Relationships

```
Session ──1:N──> Room
Room   ──1:N──> Message
Room   ──1:N──> Location Beacon
```

- One Session has many Rooms (joined + invited)
- One Room has many Messages (history)
- One Room has many Location Beacons (shared over time)

All relationships are managed by the Matrix homeserver and reflected through core's API surface. The CLI does not maintain its own entity store — it queries core on each command.
