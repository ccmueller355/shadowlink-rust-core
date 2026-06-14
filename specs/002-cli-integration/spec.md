# Feature Specification: CLI Integration

**Feature Branch**: `002-cli-integration`

**Created**: 2026-06-14

**Status**: Draft

**Input**: CLI test client that exercises the `shadowlink-rust-core` library against real Matrix homeservers. Already implemented as a clap-based Rust binary in the `shadowlink-cli` repository — this spec formalizes the completed feature.

## User Scenarios & Testing *(mandatory)*

### User Story 1 — Connect and Authenticate (Priority: P1)

A developer or tester connects the CLI to a Matrix homeserver with credentials, establishing an authenticated session that persists to disk for subsequent commands.

**Why this priority**: Connection is the entry gate — no other command works without a valid session. This is the MVP slice.

**Independent Test**: `shadowlink connect https://matrix.example.com alice mypassword` against a real Synapse instance. Verify session file is written to `shadowlink_data/session.json`. Verify subsequent `list-rooms` command can restore the session without re-authentication.

**Acceptance Scenarios**:

1. **Given** a running Synapse homeserver and valid credentials, **When** the user runs `shadowlink connect <url> <user> <pass>`, **Then** a session is established, persisted to disk, and the CLI reports success with the session file path.
2. **Given** a running Synapse but invalid credentials, **When** the user runs `shadowlink connect`, **Then** the CLI exits with a non-zero status and prints an error message (no session file created).
3. **Given** a running Synapse but an unreachable URL, **When** the user runs `shadowlink connect`, **Then** the CLI exits with a non-zero status and prints a connection-failed error.

---

### User Story 2 — Room Operations (Priority: P2)

A developer or tester creates encrypted rooms, lists joined rooms, invites users, accepts invitations, and leaves rooms — all via CLI subcommands that operate on a restored session.

**Why this priority**: Room management is the foundation for messaging and location sharing. Without rooms, nothing else matters.

**Independent Test**: Connect two ephemeral users. User A creates a room, lists rooms (sees it), invites User B. User B accepts the invite, lists rooms (sees it). User B leaves the room. User A lists rooms (room still visible, but B is gone).

**Acceptance Scenarios**:

1. **Given** an authenticated session, **When** the user runs `shadowlink create-room "Family Chat"`, **Then** a new E2EE room is created and the room ID is printed.
2. **Given** an authenticated session, **When** the user runs `shadowlink list-rooms`, **Then** all joined rooms are listed with their display names and IDs.
3. **Given** an existing room and another user's Matrix ID, **When** the user runs `shadowlink invite <room_id> <user_id>`, **Then** the invitation is sent and confirmed.
4. **Given** a pending invitation to a room, **When** the user runs `shadowlink accept <room_id>`, **Then** the user joins the room.
5. **Given** membership in a room, **When** the user runs `shadowlink leave <room_id>`, **Then** the user leaves the room and the CLI confirms.

---

### User Story 3 — Encrypted Messaging (Priority: P3)

A developer or tester sends text messages to rooms and listens for incoming messages in real time, with all traffic encrypted via Olm/Megolm.

**Why this priority**: Messaging is the core communication primitive. Room operations alone don't demonstrate value — you need to send and receive messages.

**Independent Test**: Two users in the same room. User A runs `shadowlink send <room_id> "hello"`. User B runs `shadowlink listen` and sees the decrypted message appear. User A runs `shadowlink listen` and sees B's reply.

**Acceptance Scenarios**:

1. **Given** an authenticated session and membership in a room, **When** the user runs `shadowlink send <room_id> "Hello, world"`, **Then** the message is sent with E2EE and the event ID is printed.
2. **Given** an authenticated session, **When** the user runs `shadowlink listen` (no room filter), **Then** incoming decrypted messages from all joined rooms are printed to stdout in real time until Ctrl+C.
3. **Given** an authenticated session and a specific room ID, **When** the user runs `shadowlink listen <room_id>`, **Then** only messages from that room are printed.
4. **Given** an active `listen` session, **When** the user presses Ctrl+C, **Then** the sync loop stops, the session is disconnected, and the CLI exits cleanly.

---

### User Story 4 — Location Sharing (Priority: P4)

A developer or tester shares static location beacons (geo URI with lat/lng) to rooms, enabling location-aware features in consuming applications.

**Why this priority**: Location sharing is a differentiated feature of ShadowLink. It depends on room membership but is independently testable.

**Independent Test**: User joins a room, runs `shadowlink share-location <room_id> 37.7749 -122.4194`, and verifies the event ID is returned. A second user running `listen` in the same room sees the location event.

**Acceptance Scenarios**:

1. **Given** membership in a room, **When** the user runs `shadowlink share-location <room_id> 37.7749 -122.4194`, **Then** a location beacon is sent and the event ID is printed.
2. **Given** invalid coordinates (lat > 90 or < -90), **When** the user runs `share-location`, **Then** the CLI exits with an error before sending.
3. **Given** a room the user is not a member of, **When** the user runs `share-location <other_room>`, **Then** the CLI prints a "room not found" or "not in room" error.

---

### User Story 5 — Session Lifecycle (Priority: P5)

A developer or tester manages the full session lifecycle: connect, restore across commands, and disconnect with cleanup.

**Why this priority**: Session management ties all other stories together. The CLI must survive process restarts and clean up after itself.

**Independent Test**: Connect, kill the process. Start a new process, run `list-rooms` — it restores the session without re-authentication. Run `disconnect` — session file and SQLite store are removed. Run `list-rooms` again — error, no session.

**Acceptance Scenarios**:

1. **Given** a previously established session on disk, **When** the user runs any command (list-rooms, send, etc.), **Then** the session is automatically restored from `shadowlink_data/session.json` without re-entering credentials.
2. **Given** an active session on disk, **When** the user runs `shadowlink disconnect`, **Then** the session is dropped, the session file is deleted, the SQLite store is removed, and the CLI confirms cleanup.
3. **Given** no session on disk, **When** the user runs `shadowlink disconnect`, **Then** the CLI reports "no active session" and exits cleanly.

---

### Edge Cases

- **No prior session**: Running `list-rooms` or `send` before `connect` must report a clear error (session file not found / not authenticated).
- **Expired token**: If the session token has expired since the last connect, the CLI must report `SessionExpired` and suggest re-authentication.
- **Corrupt session file**: Invalid JSON in `session.json` must produce a `StorageError` rather than a panic.
- **Missing homeserver**: All commands must time out gracefully (not hang indefinitely) when the homeserver is unreachable.
- **Empty room list**: `list-rooms` on a fresh account with no joined rooms must print a clear "no rooms" message, not an error.
- **Duplicate connect**: Running `connect` when a session already exists must overwrite the old session (not error out).
- **Large message**: Sending a message near the Matrix event size limit (65KB) must succeed or produce a clear error, not truncate silently.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST provide a `connect <url> <username> <password>` subcommand that authenticates against a user-provided Matrix homeserver and persists session state to disk.
- **FR-002**: System MUST provide a `list-rooms` subcommand that displays all joined rooms with their display names and IDs.
- **FR-003**: System MUST provide a `create-room <name>` subcommand that creates a new E2EE-encrypted room and prints the room ID.
- **FR-004**: System MUST provide an `invite <room_id> <user_id>` subcommand that sends a room invitation to the specified Matrix user.
- **FR-005**: System MUST provide an `accept <room_id>` subcommand that accepts a pending room invitation.
- **FR-006**: System MUST provide a `leave <room_id>` subcommand that leaves the specified room.
- **FR-007**: System MUST provide a `send <room_id> <message>` subcommand that sends an E2EE-encrypted text message to the specified room.
- **FR-008**: System MUST provide a `listen [room_id]` subcommand that prints incoming decrypted messages to stdout in real time, blocking until Ctrl+C. Room ID is optional — if omitted, listens to all joined rooms.
- **FR-009**: System MUST provide a `share-location <room_id> <lat> <lng>` subcommand that sends a location beacon to the specified room.
- **FR-010**: System MUST provide a `disconnect` subcommand that tears down the session, deletes the session file, and removes the SQLite store.
- **FR-011**: Every subcommand (except `connect` and `disconnect`) MUST automatically restore the persisted session before executing — users do not reconnect between commands.
- **FR-012**: Session state MUST be persisted to `shadowlink_data/session.json` relative to the current working directory.
- **FR-013**: The SQLite store MUST be persisted to `shadowlink_data/store/` relative to the current working directory and cleaned up on `disconnect`.
- **FR-014**: All subcommands MUST use the `shadowlink-rust-core` library crate as a dependency — no direct matrix-sdk usage.
- **FR-015**: CLI MUST use `clap` with derive macros for argument parsing, with `--version` and `--help` auto-generated.
- **FR-016**: System MUST log structured tracing output at `info` level by default, configurable via `RUST_LOG` environment variable.
- **FR-017**: The `listen` subcommand MUST register a message callback with the core library that prints `<sender> body` for each received text message.
- **FR-018**: All error paths MUST exit with a non-zero status code and print a user-facing error message to stderr.
- **FR-019**: The `share-location` subcommand MUST validate latitude (-90..90) and longitude (-180..180) bounds before calling the core library.

### Key Entities

- **Session**: An authenticated Matrix client session, persisted to `session.json`. Contains the homeserver URL, access token, user ID, and device ID. Restored automatically by all subcommands except `connect`.
- **Room**: A Matrix room identified by its room ID (`!localpart:domain`). Has a display name, membership state (joined/invited/left), and encryption status. Listed by `list-rooms`.
- **Message**: An E2EE-decrypted text event with sender, body, timestamp, and room ID. Printed to stdout by `listen`.
- **Location Beacon**: A custom event (`org.shadowlink.location`) containing latitude, longitude, and optional accuracy/timestamp metadata. Sent by `share-location`.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: A developer can connect to a real Synapse homeserver, create a room, send a message, and see it received by another session — all within 60 seconds of first use.
- **SC-002**: Session restore across commands works with zero re-authentication prompts — running `list-rooms` immediately after `connect` uses the persisted session.
- **SC-003**: The `listen` subcommand displays incoming messages with under 2 seconds of latency from send to display on the same homeserver.
- **SC-004**: 100% of error paths produce a non-zero exit code and a human-readable error message (no panics, no bare stack traces).
- **SC-005**: The CLI compiles and runs against the current `shadowlink-rust-core` library without modification on every CI run.
- **SC-006**: All subcommands print `--help` output that documents every flag, argument, and positional parameter.

## Assumptions

- The user provides their own Matrix homeserver (Synapse, Dendrite, or managed provider) — the CLI does not bundle or manage a homeserver.
- The `shadowlink-rust-core` library is available as a sibling directory (`../shadowlink-rust-core`) at build time.
- The `shadowlink_data/` directory is writable in the current working directory.
- The user has basic familiarity with Matrix concepts (user IDs, room IDs, homeserver URLs).
- Network connectivity to the homeserver is available when running commands — offline operation is out of scope.
- E2EE key backup and cross-signing bootstrap are handled by the core library; the CLI does not expose manual key management commands.
