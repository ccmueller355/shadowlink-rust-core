# Tasks: ShadowLink Rust Core

**Input**: Design documents from `/specs/001-shadowlink-core/`
**Prerequisites**: spec.md, plan.md, data-model.md, contracts/ffi-contract.md

**Organization**: Tasks are grouped by user story (US1–US5) following the dependency chain:
`error.rs → client.rs → rooms.rs → messaging.rs → location.rs → ffi.rs`

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2)
- All file paths are relative to the repository root

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Project dependencies, Docker test environment, and integration test scaffolding

- [x] T001 Add dependencies to `Cargo.toml` — `matrix-sdk` (0.7, features: e2e-encryption, sqlite, bundled-sqlite, qrcode), `tokio` (1, rt-multi-thread + macros), `thiserror` (1), `flutter_rust_bridge` (2), `ruma` (0.9, features: events, unstable-extensible-events), `serde` (1, derive), `serde_json` (1), `tracing` (0.1); dev-deps: `reqwest` (0.11, json), `tokio` (1, test-util)
- [x] T002 [P] Create `tests/integration/` directory and placeholder `mod.rs` — integration tests run against local Synapse  ⚠️ Tests live in `tests/` root (`tests/test_us{1..5}_*.rs`) — not migrated into `tests/integration/`. Structural cleanup deferred — compiles and runs from current location.
- [x] T003 [P] Create `docker-compose.yml` at repo root — local Synapse service (image: `matrixdotorg/synapse:latest`, port 8008, env vars `SYNAPSE_SERVER_NAME=localhost`, `SYNAPSE_REPORT_STATS=no`)
- [x] T004 [P] Create `tests/common.rs` with Synapse test helpers — `register_test_user()` via `/_synapse/admin/v1/register`, `create_ephemeral_user_pair()` for two-session tests, `teardown_users()` cleanup. `teardown_users()` added to deactivate users via Synapse admin API after test completion.

**Checkpoint**: `cargo build` compiles with all dependencies; `docker compose up -d` starts Synapse on `:8008`

---

## Phase 2: Foundational — Error Type (BLOCKS ALL USER STORIES)

**Purpose**: `ShadowLinkError` enum with `thiserror` — every module depends on it

**⚠️ CRITICAL**: No user story implementation can begin until this phase is complete

- [x] T005 Replace stub in `src/error.rs` with full `ShadowLinkError` enum — `ConnectionFailed { reason }`, `AuthenticationFailed { reason }`, `SessionExpired`, `NotInRoom`, `RoomNotFound`, `DecryptionFailed { event_id }`, `MediaTooLarge { size_bytes, limit_bytes }`, `LocationUnavailable`, `StorageError { reason }`, `Internal { message }` — derive `Debug`, `Clone`, `thiserror::Error`; ensure Display messages match `contracts/ffi-contract.md` error table
- [x] T006 [P] Write unit tests in `src/error.rs` `#[cfg(test)] mod tests` — verify each variant's Display output, verify Clone/Debug round-trips, assert no plaintext credentials in error messages

**Checkpoint**: `cargo build` produces a library with the full error enum; `cargo test` passes error unit tests

---

## Phase 3: User Story 1 — Homeserver Configuration (Priority: P1) 🎯 MVP

**Goal**: Establish and tear down authenticated Matrix sessions. The entry gate for everything.

**Independent Test**: Start Synapse via `docker compose up`, call `client::connect()` with valid URL + test credentials, verify session handle returned; call with invalid URL → `ConnectionFailed`; call with bad password → `AuthenticationFailed`; call `disconnect()` → session dropped.

### Tests for User Story 1

> **NOTE: Write these tests FIRST, ensure they FAIL before implementation**

- [x] T007 [P] [US1] Write integration test `tests/test_us1_connect.rs` — 4 tests: `test_connect_success`, `test_disconnect_double`, `test_connect_invalid_url`, `test_connect_bad_credentials`. Hang in `test_disconnect_double` fixed in commit `dbd91e1` (Notify-based sync loop cancellation).
- [x] T008 [P] [US1] Write integration test `tests/integration/test_us1_connect.rs` — `test_connect_invalid_url`: call `connect()` with `http://nonexistent:9999`, assert `ConnectionFailed`; `test_connect_bad_credentials`: call `connect()` with valid URL but wrong password, assert `AuthenticationFailed`

### Implementation for User Story 1

- [x] T009 [P] [US1] Implement `Session` struct and `SessionHandle` newtype in `src/client.rs` — `Session` wraps `matrix_sdk::Client` + `sync_running: bool`; `SessionHandle` wraps `Arc<Mutex<Session>>`; nothing crosses FFI boundary by value
- [x] T010 [US1] Implement `client::connect()` in `src/client.rs` — accept `homeserver_url: String, username: String, password: String` → `Result<SessionHandle, ShadowLinkError>`; use `Client::builder().server_name_or_homeserver_url()` for discovery; `login_username()` for auth; `encryption()` to enable E2EE; spawn sync loop in tokio task; return handle (depends on T009)
- [x] T011 [US1] Implement `client::disconnect()` in `src/client.rs` — accept `SessionHandle`; stop sync loop; call `client.logout()`; drop the handle; no-op if already disconnected
- [x] T012 [US1] Write unit tests in `src/client.rs` `#[cfg(test)] mod tests` — test `SessionHandle` Cloning increments Arc count; test double-disconnect is safe; test error mapping from SDK errors to `ShadowLinkError` variants

**Checkpoint**: `cargo test test_us1_connect` passes against local Synapse with `docker compose up`

---

## Phase 4: User Story 2 — Room Operations (Priority: P2)

**Goal**: Create E2EE rooms, invite family members, accept invites, list joined rooms, leave rooms.

**Independent Test**: Two ephemeral sessions on local Synapse. Session A creates a room, invites Session B. Session B accepts. Both call `list_rooms()` and verify membership. Session B leaves the room.

### Tests for User Story 2

- [x] T013 [P] [US2] Write integration test `tests/test_us2_rooms.rs` — 4 tests: `test_create_and_list_rooms`, `test_accept_invite`, `test_invite_user`, `test_leave_room`. All use real room ops against local Synapse.

### Implementation for User Story 2

- [x] T014 [P] [US2] Define `RoomInfo` struct and `RoomState` enum in `src/rooms.rs` — `RoomInfo { room_id: String, name: Option<String>, member_count: u64, encrypted: bool, state: RoomState }`; `RoomState { Joined, Invited, Left }`; derive Clone, Debug; ensure all fields are owned (FFI-safe)
- [x] T015 [US2] Implement `rooms::create_room()` and `rooms::list_rooms()` in `src/rooms.rs` — `create_room(handle, name)` → `Result<RoomInfo>`: use `client.create_room()`, set E2EE default, set room name, sync room, return RoomInfo; `list_rooms(handle)` → `Result<Vec<RoomInfo>>`: iterate SDK room list, project each room's metadata and membership state (depends on T014)
- [x] T016 [US2] Implement `rooms::accept_invite()` and `rooms::invite_user()` in `src/rooms.rs` — `accept_invite(handle, room_id)` → `Result<RoomInfo>`: find invited room by id, call `.join()`, sync; `invite_user(handle, room_id, user_id)` → `Result<()>`: find joined room by id, call `.invite_user_by_id()`, map errors to NotInRoom/RoomNotFound
- [x] T017 [US2] Implement `rooms::leave_room()` in `src/rooms.rs` — `leave_room(handle, room_id)` → `Result<()>`: find joined room, call `.leave()`, verify membership updated; map errors to NotInRoom/RoomNotFound

**Checkpoint**: `cargo test test_us2_rooms` passes with two ephemeral sessions on local Synapse

---

## Phase 5: User Story 3 — E2EE Messaging & Media (Priority: P3)

**Goal**: Send and receive encrypted text messages and media attachments within rooms. Message history with pagination. Real-time message callbacks.

**Independent Test**: Two sessions in the same E2EE room. Session A sends text + image. Session B receives both via callback. Session B reads history and confirms both messages present.

### Tests for User Story 3

- [x] T018 [P] [US3] Write integration test `tests/test_us3_messaging.rs` — 4 tests: `test_send_receive_text`, `test_send_receive_media`, `test_message_history`, `test_callback_registration`.

### Implementation for User Story 3

- [x] T019 [P] [US3] Define `Message` struct and `MessageContent` enum in `src/messaging.rs` — `Message { event_id: String, sender: String, timestamp: i64, content: MessageContent }`; `MessageContent { Text { body }, Media { mime_type, uri, filename, size_bytes }, Location { lat, lng, accuracy_m, live } }`; implement Debug that omits body (FR-014); all fields owned
- [x] T020 [US3] Implement `messaging::send_text()` and `messaging::send_media()` in `src/messaging.rs` — `send_text(handle, room_id, body)` → `Result<String>`: use SDK `Joined::send()` with plaintext; `send_media(handle, room_id, data, mime_type, filename)` → `Result<String>`: use SDK `Joined::send_attachment()`, build content info from mime_type/filename; map SDK errors to NotInRoom/ConnectionFailed/MediaTooLarge (depends on T019)
- [x] T021 [US3] Implement `messaging::get_history()` in `src/messaging.rs` — `get_history(handle, room_id, limit: u32)` → `Result<Vec<Message>>`: find joined room, call `.messages()` builder with limit, await batch, decrypt via SDK, skip undecryptable events, return in reverse-chron order; map errors to NotInRoom/RoomNotFound
- [x] T022 [US3] Implement `messaging::register_message_callback()` and wire sync event dispatcher in `src/messaging.rs` — `register_message_callback(handle, callback: impl Fn(Message) + Send + 'static)`: store callback ref in session; wire sync handler in `client.rs` sync loop: on `SyncEvent::Message`, decrypt, construct `Message`, invoke stored callback; handle callback replacement on re-registration

**Checkpoint**: `cargo test test_us3_messaging` passes — E2EE round-trip verified, callbacks fire

---

## Phase 6: User Story 4 — Location Sharing (Priority: P4)

**Goal**: Send static location beacons and live location streams. Receive location events from family members via callback.

**Independent Test**: Two sessions in same room. Session A sends a static beacon (lat/lng). Session B receives it via callback. Session A starts live location; Session B receives periodic updates. Session A stops live location; updates cease.

### Tests for User Story 4

- [x] T023 [P] [US4] Write integration test `tests/test_us4_location.rs` — 3 tests: `test_send_beacon`, `test_live_location_start_stop`, `test_location_unavailable`. All use real Synapse sessions and verify beacon delivery via history fallback.

### Implementation for User Story 4

- [x] T024 [P] [US4] Define `LocationBeacon` and `LiveLocationConfig` types in `src/location.rs` — `LocationBeacon { lat: f64, lng: f64, accuracy_m: Option<f64> }`; `LiveLocationConfig { interval_secs: u64, accuracy_m: Option<f64> }`; all Clone + Debug; validate `interval_secs >= 5`
- [x] T025 [P] [US4] Define `org.shadowlink.location` custom ruma event type in `src/location.rs` — use ruma `ExtensibleEventContent` derive macro; fields: `lat: f64`, `lng: f64`, `accuracy_m: Option<f64>`, `live: bool`; implement `Serialize`/`Deserialize`; register event type for SDK event parsing
- [x] T026 [US4] Implement `location::send_beacon()` in `src/location.rs` — `send_beacon(handle, room_id, lat, lng, accuracy_m)` → `Result<String>`: validate coordinate ranges (lat: -90..90, lng: -180..180), construct custom event, send via SDK room send; map errors to NotInRoom/RoomNotFound/LocationUnavailable (depends on T024, T025)
- [x] T027 [US4] Implement `location::start_live_location()`, `stop_live_location()`, and `register_location_callback()` in `src/location.rs` — `start_live`: spawn tokio interval task sending `org.shadowlink.location` events with `live:true` every interval_secs, store `JoinHandle` in session, reject duplicate starts; `stop_live`: abort the interval task, send final event; `register_location_callback`: store callback in session (replaces previous); wire sync handler to parse incoming `org.shadowlink.location` events and invoke callback with `LocationBeacon`

**Checkpoint**: `cargo test test_us4_location` passes — beacons and live streams round-trip

---

## Phase 7: User Story 5 — Session Persistence (Priority: P5)

**Goal**: Sessions survive process restarts. SDK SQLite store persists credentials, room memberships, E2EE keys, and sync tokens. Expired tokens produce clear errors.

**Independent Test**: Connect, send a message, drop session object. Call `restore_session()` and verify room list, message history, and E2EE session keys are intact without re-authentication.

### Tests for User Story 5

- [x] T028 [P] [US5] Write integration test `tests/test_us5_persistence.rs` — 3 tests: `test_session_persistence_across_restart`, `test_restored_session_room_list`, `test_restored_session_history`. Verify SQLite store round-trips across disconnect/reconnect cycles.

### Implementation for User Story 5

- [x] T029 [US5] Implement `client::restore_session()` in `src/client.rs` — `restore_session() -> Result<SessionHandle, ShadowLinkError>`: use `Client::builder().sqlite_store(path, passphrase)` to open existing store, call `client.restore_session()`, verify token validity → SessionExpired if invalid, start sync loop, return handle
- [x] T030 [US5] Wire SDK SQLite persistence path in `connect()` and `restore_session()` in `src/client.rs` — define store path relative to a configurable base directory (default: `shadowlink_data/` next to binary); pass passphrase from environment or hard-coded dev default; ensure store is closed on disconnect
- [x] T031 [US5] Handle expired token → `SessionExpired` and corrupt store → `StorageError` in `src/client.rs` — catch SDK `HttpError::Unauthorized` on restore → SessionExpired; catch SDK `StoreError::OpenStore` failures → StorageError with reason string; ensure error messages recommend re-authentication or store reset

**Checkpoint**: `cargo test test_us5_persistence` passes — session survives process lifecycle

---

## Phase 8: FFI Surface — flutter_rust_bridge Wire-Up

**Purpose**: Expose all core functions to Flutter via `flutter_rust_bridge` annotations with `StreamSink` callbacks

- [x] T032 [P] [US1-US5] Write contract test `tests/spec_contracts.rs` — verify every FFI function signature from `contracts/ffi-contract.md` compiles: `connect`, `restore_session`, `disconnect`, `create_room`, `accept_invite`, `invite_user`, `list_rooms`, `leave_room`, `send_text`, `send_media`, `get_history`, `register_message_callback`, `send_beacon`, `start_live_location`, `stop_live_location`, `register_location_callback`; use `#[frb]` attribute compilation check
- [x] T033 [US1-US5] Implement all FFI wrappers in `src/ffi.rs` — annotate each function with `flutter_rust_bridge` macros (`#[frb]`); wrap `SessionHandle` as opaque pointer; implement `StreamSink<Message>` for message callback dispatch; implement `StreamSink<LocationBeacon>` for location callback dispatch; all functions are `pub async fn` → `Result<T, ShadowLinkError>`; no business logic — delegate to `src/{client,rooms,messaging,location}.rs` functions

**Checkpoint**: `cargo test test_spec_contracts` compiles and passes; `flutter_rust_bridge_codegen generate` produces Dart bindings (external tool, verify syntax only)

---

## Phase 9: Polish & CI Gates

**Purpose**: Quality gates, security scan, and documentation update

- [x] T034 Run `cargo clippy -- -D warnings` and fix all lints — ensure zero warnings across all `src/` and `tests/` modules
- [x] T035 [P] Run `cargo fmt -- --check` to verify formatting; run `cargo llvm-cov --all-targets --html` → verify ≥80% line coverage; run `gitleaks detect` → verify zero findings; update `README.md` with quick-start instructions (docker compose up, cargo test, FFI codegen)

**Checkpoint**: All CI gates green; crate ready for `v0.1.0` tag

---

## Dependencies & Execution Order

### Phase Dependencies

```
Phase 1 (Setup) ──► Phase 2 (Error) ──► Phase 3 (US1) ──► Phase 4 (US2) ──► Phase 5 (US3) ──► Phase 6 (US4) ──► Phase 7 (US5) ──► Phase 8 (FFI) ──► Phase 9 (Polish)
```

- **Phase 1 (Setup)**: No dependencies — run immediately
- **Phase 2 (Error)**: Depends on Setup (T001 for `thiserror` dep) — BLOCKS all user stories
- **Phase 3 (US1)**: Depends on Phase 2 — foundation for all subsequent stories
- **Phase 4 (US2)**: Depends on US1 (needs `SessionHandle` from `client.rs`)
- **Phase 5 (US3)**: Depends on US2 (needs room membership from `rooms.rs`)
- **Phase 6 (US4)**: Depends on US2 (needs room access) + US3 (if location events reuse message callback pattern)
- **Phase 7 (US5)**: Depends on US1 (persistence is wired in `client.rs`); validated against US2+US3 behavior after restore
- **Phase 8 (FFI)**: Depends on all stories — wraps every `src/` module function
- **Phase 9 (Polish)**: Depends on all implementation complete

### Within Each User Story

- Tests (T00X [P]) MUST be written and FAIL before implementation
- Types/structs before functions that use them
- Core operations before callback wiring
- Story complete (tests pass) before moving to next priority

### Parallel Opportunities

- T002, T003, T004 (setup scaffolding): all different files — run in parallel
- T005, T006 (error.rs + tests): same file but different concerns — unit tests can run first
- T007, T008 (US1 tests): same file `test_us1_connect.rs` but independent test functions — write together
- T009 (Session struct) can be written in parallel with T007/T008 tests
- T024, T025 (US4 types + event): different files — run in parallel
- T034, T035 (clippy + fmt/coverage/leaks): independent tools — run in parallel

---

## Implementation Strategy

### MVP First (US1 Only)

1. Complete Phase 1: Setup (`docker compose up`, `cargo build`)
2. Complete Phase 2: Error type (`cargo test` green)
3. Complete Phase 3: User Story 1 (`cargo test test_us1_connect`)
4. **STOP and VALIDATE**: Connect to real Synapse, verify session lifecycle
5. Demo: `connect()` → `disconnect()` works end-to-end

### Incremental Delivery

1. Setup + Error → Foundation ready
2. US1: Homeserver connect → **MVP shipped**
3. US2: Room operations → Two-session rooms
4. US3: E2EE messaging → Family chat works
5. US4: Location sharing → Beacons + live tracking
6. US5: Session persistence → App restart resilient
7. FFI + Polish → Crate publishable

### Parallel Team Strategy (3 developers)

1. Team completes Setup + Error together
2. Once Error is done:
   - Dev A: US1 (client.rs)
   - Dev B: Write US2 tests while US1 wraps up
   - Dev C: Wire docker-compose + CI config (dev-deps)
3. Once US1 done:
   - Dev A: US2 (rooms.rs)
   - Dev B: US3 (messaging.rs) — can start with rooms.rs stubs
   - Dev C: US4 types (T024, T025) — independent files
4. Once US2 done:
   - Dev A: US5 (client.rs persistence)
   - Dev B: US4 (location.rs functions)
   - Dev C: FFI wrappers (Phase 8)

---

## Notes

- [P] tasks = different files, no data dependencies — launch in parallel
- [Story] label maps task to specific user story for traceability
- All module stubs in `src/` already exist — tasks **replace** stub content, do not create new files
- `tests/` directory and `tests/integration/` do NOT exist — tasks create them
- `src/tests.rs` does NOT exist — not needed; unit tests live in `#[cfg(test)]` in each module
- `docker-compose.yml` at repo root starts Synapse for all integration tests
- Commit after each task or logical group with Conventional Commits format
- Stop at any checkpoint to validate story independently
- FR-014 (no plaintext logging): all Debug impls for message types omit body
- Co-authored-by trailer required on all commits: `Co-authored-by: Valerie Decker <neural-deck@v4.7.0>`
- Contract tests (T032) verify FFI signatures compile — they do NOT require a running Synapse
- Integration tests (T007, T008, T013, T018, T023, T028) require `docker compose up` running Synapse on `:8008`
