# Implementation Plan

> **Status:** Accepted — SpecKit Plan phase.
> Bridges the feature spec (`spec.md`) to executable work items (tasks phase).
> Cross-referenced from arc42 Section 5 (Building Block View).

## Overview

The ShadowLink Rust Core implements 5 user stories (US1–US5) across 6 Rust
source modules, all exposed via a single FFI surface. The dependency chain
is strictly linear: each user story builds on the one before it.

```
US1 (Homeserver) → US2 (Rooms) → US3 (Messaging) → US4 (Location) → US5 (Persistence)
```

US5 (session persistence) is implemented incrementally — basic persistence
is wired in US1, verified after each story, and fully validated at the end.

## Module Design

Each `src/` file maps to one or more user stories. No module stands alone;
each depends on the modules to its left.

```
src/
├── lib.rs          # Crate root — module declarations, re-exports
├── error.rs        # ShadowLinkError enum (all stories)
├── client.rs       # US1: connect, restore_session, disconnect
├── rooms.rs        # US2: create_room, accept_invite, invite_user,
│                   #      list_rooms, leave_room
├── messaging.rs    # US3: send_text, send_media, get_history,
│                   #      register_message_callback
├── location.rs     # US4: send_beacon, start_live_location,
│                   #      stop_live_location, register_location_callback
└── ffi.rs          # FFI boundary — wire flutter_rust_bridge annotations,
                    # callback dispatch, memory handoff
```

### Module Dependency Rules

1. **`error.rs`** — Zero upstream dependencies. Every other module depends
   on it. Must be implemented first.
2. **`client.rs`** — Depends on `error`. Creates and owns the `Session`.
   Exposes `connect()`, `restore_session()`, `disconnect()`.
3. **`rooms.rs`** — Depends on `client` (needs `Session` handle) and
   `error`. Exposes room CRUD operations.
4. **`messaging.rs`** — Depends on `rooms` (needs room membership) and
   `error`. Exposes message send/receive and media.
5. **`location.rs`** — Depends on `rooms` and `error`. Implements
   `org.shadowlink.location` custom event type via ruma macros.
6. **`ffi.rs`** — Depends on all modules. Contains only
   `flutter_rust_bridge` annotations, callback type definitions, and
   memory handoff helpers. No business logic.

### What Does NOT Get Its Own Module

Following the Karpathy Protocol (radical encapsulation):
- **No `sync.rs`:** Sync loop lives inside `client.rs`. It's ~80 lines of
  tokio task spawning, not a separate subsystem.
- **No `storage.rs`:** Persistence is SDK-managed. The crate only calls
  `Client::builder().sqlite_store(path, passphrase)`. No custom storage
  layer.
- **No `config.rs`:** Configuration is passed as function parameters
  (homeserver URL, credentials). No persistent config file beyond what
  the SDK stores.

## FFI Contract Design

The FFI surface is defined in `contracts/ffi-contract.md`. Key design
decisions:

1. **SessionHandle pattern:** `connect()` returns an opaque handle.
   Every other function takes it as the first parameter. Flutter never
   inspects the handle — it's a token to pass back.

2. **Callbacks via StreamSink:** flutter_rust_bridge v2's `StreamSink<T>`
   is the mechanism for pushing events (incoming messages, location beacons)
   from Rust to Dart. Flutter registers a callback once per session; Rust
   holds a weak reference.

3. **Return types:** All FFI functions return `Result<T, ShadowLinkError>`.
   flutter_rust_bridge converts `Err` to a Dart exception, `Ok` to the
   return value.

4. **Async all the way:** Every FFI function is `async fn`. Even operations
   that appear synchronous (like `list_rooms()`) may need to await the
   internal Mutex lock.

## Test Strategy

### Local Synapse via Docker

All integration tests run against a local Synapse homeserver:

```bash
docker run -d --name synapse-test \
  -v synapse-data:/data \
  -e SYNAPSE_SERVER_NAME=localhost \
  -e SYNAPSE_REPORT_STATS=no \
  -p 8008:8008 \
  matrixdotorg/synapse:latest
```

Test registration creates disposable users via the Matrix admin API
(`/_synapse/admin/v1/register`). Tests run sequentially to avoid
concurrent room state conflicts.

### Test Organization

Tests live in `src/tests.rs` (integration tests at the crate level) and
as `#[cfg(test)]` modules within each source file for unit tests.

| Test Layer | Scope | Location |
|---|---|---|
| Unit tests | Pure logic, no network. Error variant formatting, enum conversion, type invariants. | `#[cfg(test)] mod tests` in each `src/*.rs` |
| Integration tests | Real SDK against local Synapse. Full FFI flow per user story. | `src/tests.rs` |
| Contract tests | Verify FFI function signatures match `contracts/ffi-contract.md`. | `src/tests.rs` — `ffi_contract_*` |

### Per-Story Integration Tests

| Story | Test Names | What It Validates |
|---|---|---|
| US1 | `test_connect_success`, `test_connect_invalid_url`, `test_connect_bad_credentials`, `test_disconnect` | Homeserver auth, error mapping, session lifecycle |
| US2 | `test_create_and_list_rooms`, `test_accept_invite`, `test_invite_user`, `test_leave_room` | Room CRUD with two sessions |
| US3 | `test_send_receive_text`, `test_send_receive_media`, `test_message_history`, `test_callback_registration` | E2EE message round-trip, media upload, pagination |
| US4 | `test_send_beacon`, `test_live_location_start_stop`, `test_location_callback` | Custom event send/receive, live toggle |
| US5 | `test_session_persistence_across_restart`, `test_restored_session_room_list`, `test_restored_session_history` | SDK persistence, restore without re-auth |

### CI Pipeline Integration

Tests requiring Synapse run in a CI job with a Docker service container.
Unit tests (no network) run in a separate, faster job.

```yaml
# .github/workflows/ci.yml (relevant section)
jobs:
  test-integration:
    runs-on: ubuntu-latest
    services:
      synapse:
        image: matrixdotorg/synapse:latest
        env:
          SYNAPSE_SERVER_NAME: localhost
          SYNAPSE_REPORT_STATS: no
        ports:
          - 8008:8008
    steps:
      - uses: actions/checkout@v4
      - run: cargo test -- --test-threads=1
```

## Rollout Order

### Phase 4a — US1: Homeserver Configuration (P1)

1. Replace stub `ShadowLinkError` in `src/error.rs` with full `thiserror` enum.
2. Implement `client::connect()` with homeserver discovery, login, sync start.
3. Implement `client::restore_session()` using SDK's `Client::builder().sqlite_store()`.
4. Implement `client::disconnect()` with logout and sync stop.
5. Write integration tests: `test_connect_*`, `test_disconnect`.
6. **Gate:** `cargo test test_connect` passes against local Synapse.

### Phase 4b — US2: Room Operations (P2)

1. Implement `rooms::create_room()` with E2EE default settings.
2. Implement `rooms::accept_invite()`, `rooms::invite_user()`, `rooms::leave_room()`.
3. Implement `rooms::list_rooms()` projecting SDK room list to `Vec<RoomInfo>`.
4. Write integration tests: `test_create_and_list_rooms`, `test_accept_invite`,
   `test_invite_user`, `test_leave_room`.
5. **Gate:** All room tests pass with two sessions on local Synapse.

### Phase 4c — US3: E2EE Messaging & Media (P3)

1. Implement `messaging::send_text()` using `Joined::send()`.
2. Implement `messaging::send_media()` delegating to `Joined::send_attachment()`.
3. Implement `messaging::get_history()` with configurable limit.
4. Implement `messaging::register_message_callback()` via `StreamSink`.
5. Wire sync event handler to dispatch incoming messages to the callback.
6. Write integration tests: `test_send_receive_text`, `test_send_receive_media`,
   `test_message_history`.
7. **Gate:** Message round-trip works, callbacks fire, history is paginated.

### Phase 4d — US4: Location Sharing (P4)

1. Define `org.shadowlink.location` event content type via ruma's
   `ExtensibleEventContent` derive macro.
2. Implement `location::send_beacon()` constructing and sending the custom event.
3. Implement `location::start_live_location()` spawning a tokio interval task.
4. Implement `location::stop_live_location()` cancelling the interval task.
5. Implement `location::register_location_callback()` for incoming location events.
6. Wire sync event handler to parse `org.shadowlink.location` events.
7. Write integration tests: `test_send_beacon`, `test_live_location_start_stop`.
8. **Gate:** Location events round-trip, live stream starts/stops cleanly.

### Phase 4e — US5: Session Persistence (P5)

1. Verify SDK persistence is wired correctly from US1 implementation.
2. Write the "process restart" integration test:
   - Authenticate, send a message, drop the session.
   - Create a new session with `restore_session()`.
   - Verify room list and message history are intact.
3. Test edge case: expired token → `SessionExpired` error.
4. Test edge case: corrupted store → `StorageError`.
5. **Gate:** `test_session_persistence_across_restart` passes.

### Phase 4f — Polish & CI Gates

1. Run `cargo clippy -- -D warnings` — fix all issues.
2. Run `cargo fmt -- --check` — ensure formatting.
3. Run `cargo llvm-cov --all-targets --html` — verify ≥80% line coverage.
4. Run `gitleaks detect` — verify zero findings.
5. Update `README.md` with final quick-start.
6. Tag `v0.1.0` and publish to crates.io (if ready).

## Dependencies & Cargo.toml

Final dependency list (to be added incrementally as modules are built):

```toml
[dependencies]
matrix-sdk = { version = "0.7", features = [
    "e2e-encryption",
    "sqlite",
    "bundled-sqlite",
    "qrcode",
] }
tokio = { version = "1", features = ["rt-multi-thread", "macros"] }
thiserror = "1"
flutter_rust_bridge = "2"
ruma = { version = "0.9", features = ["events", "unstable-extensible-events"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
tracing = "0.1"

[dev-dependencies]
tokio = { version = "1", features = ["rt-multi-thread", "macros", "test-util"] }
reqwest = { version = "0.11", features = ["json"] }  # for Synapse admin API in tests
```

## Open Questions

1. **Device verification UX:** The crate provides E2EE; device verification
   (emoji comparison, QR scan) is a UX flow handled by Flutter. Does the
   crate need to expose verification state, or can Flutter poll it?
   → Deferred: expose `get_verification_state(handle, user_id) -> VerificationState`
   as a P2 addition after US5.

2. **Sliding sync:** matrix-sdk 0.7 includes `experimental-sliding-sync`.
   Should we enable it from day 1 or fall back to `/sync`?
   → Deferred: start with `/sync` v2 (stable). Evaluate sliding sync in
   Phase 6 when the feature stabilizes in the SDK.

3. **Media thumbnail generation:** The SDK can generate thumbnails during
   upload. Should we request a specific size for mobile?
   → Deferred: accept SDK defaults. Flutter can resize before upload if
   needed (reduces bandwidth, not a Rust concern).
