# Tasks: Family Room Semantics

**Input**: Design documents from `/specs/003-family-room/`

**Prerequisites**: plan.md ✅, spec.md ✅, research.md ✅, data-model.md ✅, contracts/rooms-contract.md ✅, quickstart.md ✅

**Tests**: Spec includes 12 acceptance scenarios across 3 user stories. All tests follow Red-Green-Refactor — write failing tests before implementation.

**Organization**: Tasks grouped by user story. Phase 1 (foundational data model) blocks all stories. Phases 2-4 are independent once Phase 1 is complete.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (US1, US2, US3)
- Exact file paths in descriptions

## Path Conventions

```text
shadowlink-rust-core/
├── src/
│   ├── rooms.rs             # +create_family_room, +set_home_room, +get_home_room
│   ├── client.rs            # +enable_debug_room, +emit_diagnostic, StoredSession.home_room_id
│   ├── ffi.rs               # +ShadowLinkApi wrappers
│   ├── lib.rs               # unchanged (modules already public)
│   └── error.rs             # unchanged (no new variants needed)
├── tests/
│   ├── test_us2_rooms.rs           # +family room tests
│   ├── test_us5_persistence.rs     # +home room ID persistence tests
│   └── integration/mod.rs          # +two-session family room flow
└── specs/003-family-room/   # this directory
```

---

## Phase 1: Foundational — Data Model Extension (Blocks All Stories)

**Purpose**: Extend `RoomInfo`, `StoredSession`, and `Session` structs so all user stories can reference the new fields. No new public functions yet.

**⚠️ CRITICAL**: No user story work can begin until this phase is complete.

- [ ] T001 Extend `RoomInfo` struct in `src/rooms.rs` — add `alias: Option<String>` and `is_home: bool` fields. Both derive `Clone, Debug`. Default: `alias: None`, `is_home: false`.
- [ ] T002 [P] Update `to_room_info()` helper in `src/rooms.rs` to populate `alias` from the room's canonical alias state event (via `room.canonical_alias()`) and set `is_home: false` by default.
- [ ] T003 [P] Extend `StoredSession` struct in `src/client.rs` — add `home_room_id: Option<String>` field. Annotate with `#[serde(skip_serializing_if = "Option::is_none")]` so existing session files load without error.
- [ ] T004 [P] Add `debug_room_id: Option<String>` and `debug_room_enabled: bool` fields to `Session` struct in `src/client.rs`. Default: `None`, `false`.
- [ ] T005 [P] Update all existing `RoomInfo` construction sites in `src/rooms.rs` (create_room, list_rooms, accept_invite, to_room_info) to include new fields with defaults.
- [ ] T006 [P] Update `Session::new()` in `src/client.rs` to initialise new fields with defaults.
- [ ] T007 Verify: `cargo build` — compiles clean with zero warnings. `cargo test` — all existing tests pass unchanged.

**Checkpoint**: ✅ Data model extended — all user stories can now be implemented in parallel.

---

## Phase 2: User Story 1 — Operator Creates the Family Room (Priority: P1) 🎯 MVP

**Goal**: Implement `create_family_room()` with alias derivation, `join_rule: invite` semantics, home room persistence, and `is_home: true` on the returned `RoomInfo`.

**Independent Test**: Connect to local Synapse, call `create_family_room("The Smith Family")`, verify room created with `join_rule: invite`, alias set, `is_home: true`, and `get_home_room()` returns it.

### Tests for User Story 1

- [ ] T008 [P] [US1] Write `test_create_family_room_basic` in `tests/test_us2_rooms.rs` — register ephemeral user, connect, create family room, assert `is_home: true`, alias present, room is in `list_rooms()` with `is_home: true`.
- [ ] T009 [P] [US1] Write `test_create_family_room_alias` in `tests/test_us2_rooms.rs` — verify alias derivation: "The Smith Family" → `#the-smith-family:localhost`, "Hello World!" → `#hello-world:localhost`, "Über Family" → `#ber-family:localhost` (non-ASCII stripped).
- [ ] T010 [P] [US1] Write `test_create_family_room_replaces_previous` in `tests/test_us2_rooms.rs` — create first family room, create second, verify first no longer `is_home`, second is `is_home`.
- [ ] T011 [P] [US1] Write `test_create_family_room_invalid_name` in `tests/test_us2_rooms.rs` — empty string + >255 char name → `OperationFailed`.

### Implementation for User Story 1

- [ ] T012 [US1] Implement `derive_alias_localpart(name: &str) -> String` private helper in `src/rooms.rs` — lowercase, spaces→hyphens, strip `[^a-z0-9._=-]`, truncate 255, strip leading/trailing hyphens/dots. Add `#[cfg(test)] mod tests` with unit tests for the derivation.
- [ ] T013 [US1] Implement `create_family_room(handle, name)` in `src/rooms.rs` — validate name (non-empty, ≤255 chars), derive alias localpart, build `create_room::v3::Request` with `RoomPreset::PrivateChat`, `Visibility::Private`, `room_alias_name`, enable E2EE, persist `home_room_id` via `StoredSession`, return `RoomInfo` with `is_home: true`. Alias is best-effort — if server rejects alias, room still succeeds with `alias: None`.
- [ ] T014 [US1] Implement `persist_home_room_id(room_id)` and `load_home_room_id() -> Option<String>` private helpers in `src/client.rs` — read/write the `StoredSession.home_room_id` field. Re-use existing `StoredSession::save()`/`load()` paths.
- [ ] T015 [US1] Add FFI wrapper `ShadowLinkApi::create_family_room(&self, name: &str) -> Result<String, ShadowLinkError>` in `src/ffi.rs`.
- [ ] T016 [US1] Run `cargo test test_create_family_room` — all four tests pass.

**Checkpoint**: ✅ Operator can create a family room with alias and `is_home` flag. `create_room` unchanged.

---

## Phase 3: User Story 2 — View and Navigate Home Room (Priority: P2)

**Goal**: Implement `set_home_room()`, `get_home_room()`, and update `list_rooms()` to mark the home room. Enable the "adopt existing room" flow with E2EE enforcement.

**Independent Test**: Create a generic room, call `set_home_room(id)`, verify `is_home: true`, `get_home_room()` returns it, `list_rooms()` marks exactly one room.

### Tests for User Story 2

- [ ] T017 [P] [US2] Write `test_set_home_room_basic` in `tests/test_us2_rooms.rs` — create generic room, call `set_home_room`, assert `is_home: true`, `get_home_room()` returns it.
- [ ] T018 [P] [US2] Write `test_set_home_room_unpins_previous` in `tests/test_us2_rooms.rs` — create family room, create generic room, `set_home_room(generic_id)`, verify family room no longer `is_home`, generic is `is_home`.
- [ ] T019 [P] [US2] Write `test_get_home_room_none` in `tests/test_us2_rooms.rs` — fresh session, `get_home_room()` returns `None`.
- [ ] T020 [P] [US2] Write `test_list_rooms_marks_home` in `tests/test_us2_rooms.rs` — create family room + 2 generic rooms, `list_rooms()` returns exactly one with `is_home: true`.
- [ ] T021 [P] [US2] Write `test_set_home_room_not_member` in `tests/test_us2_rooms.rs` — `set_home_room` with a room the user is not in → `NotInRoom`.

### Implementation for User Story 2

- [ ] T022 [US2] Implement `set_home_room(handle, room_id)` in `src/rooms.rs` — find room in joined rooms (error: `NotInRoom`/`RoomNotFound`), check E2EE: if not encrypted, call `room.enable_encryption().await` (error: `OperationFailed` if rejected), persist `home_room_id`, return `RoomInfo` with `is_home: true` and `encrypted: true`.
- [ ] T023 [US2] Implement `get_home_room(handle)` in `src/rooms.rs` — load `home_room_id` from stored session, if `Some`, look up room in joined rooms, return `RoomInfo` with `is_home: true`. If room not found in joined list (left/deleted), fall back to invited rooms, then left rooms. If `None`, return `Ok(None)`.
- [ ] T024 [US2] Update `list_rooms()` in `src/rooms.rs` — after collecting all rooms, load `home_room_id` and set `is_home: true` on the matching room.
- [ ] T025 [US2] Add FFI wrappers `ShadowLinkApi::set_home_room(&self, room_id: &str) -> Result<String, ShadowLinkError>` and `ShadowLinkApi::get_home_room(&self) -> Result<Option<String>, ShadowLinkError>` in `src/ffi.rs`.
- [ ] T026 [US2] Run `cargo test test_set_home_room test_get_home_room test_list_rooms` — all five tests pass.

**Checkpoint**: ✅ Family room can be pinned/unpinned, adopted from existing rooms, and listed.

---

## Phase 4: User Story 3 — Diagnostic Debug Room (Priority: P3)

**Goal**: Implement `enable_debug_room()` toggle and `emit_diagnostic()` private function. Wire diagnostic emission into sync error, connection failure, and E2EE warning paths.

**Independent Test**: Connect session, enable debug room, trigger a sync failure, verify diagnostic event appears in `"ShadowLink Debug"` room with text + JSON metadata.

### Tests for User Story 3

- [ ] T027 [P] [US3] Write `test_debug_room_creation` in `tests/test_us2_rooms.rs` — enable debug room, verify `"ShadowLink Debug"` room created, private, invite-only, E2EE, user is sole member.
- [ ] T028 [P] [US3] Write `test_debug_room_toggle` in `tests/test_us2_rooms.rs` — enable, verify emission flag true, disable, verify emission flag false, room not deleted.
- [ ] T029 [P] [US3] Write `test_debug_room_recreate` in `tests/test_us2_rooms.rs` — enable, leave debug room, trigger diagnostic, verify new debug room created with different ID.
- [ ] T030 [P] [US3] Write `test_debug_room_no_pii` in `tests/test_us2_rooms.rs` — trigger diagnostic event, read message from debug room, assert no access token, no key material, no message content in JSON metadata.

### Implementation for User Story 3

- [ ] T031 [US3] Implement `emit_diagnostic(event_type, module, severity, detail, error_code)` private async function in `src/client.rs` — check `debug_room_enabled`, if enabled: ensure debug room exists (create if missing), format message body as `{detail}\n\n```json\n{json}\n````, send via SDK `room.send()`. JSON block contains: `event`, `module`, `severity`, `timestamp` (ISO 8601 UTC), `error_code` (optional). PII constraint: never include message content, coordinates, tokens, or non-operator user IDs.
- [ ] T032 [US3] Implement `enable_debug_room(handle, enabled)` in `src/client.rs` — set `debug_room_enabled` flag, if `true` and no debug room exists: create private invite-only E2EE room named `"ShadowLink Debug"`, store `debug_room_id`. If creation fails, leave flag false and return `OperationFailed`. If `false`, just set flag (room not deleted).
- [ ] T033 [US3] Wire `emit_diagnostic()` into sync error path in `src/client.rs` — in the sync loop's error handler, emit `sync_error`/`client`/`error` with the error detail.
- [ ] T034 [P] [US3] Wire `emit_diagnostic()` into connection failure path in `src/client.rs` — in `connect()` error handling, emit `connection_failed`/`client`/`error`.
- [ ] T035 [P] [US3] Wire `emit_diagnostic()` into session lifecycle events in `src/client.rs` — on session restore success (`session_lifecycle`/`client`/`info`/`"Session restored"`), on session expired (`session_lifecycle`/`client`/`warn`/`"Session expired"`).
- [ ] T036 [US3] Add FFI wrapper `ShadowLinkApi::enable_debug_room(&self, enabled: bool) -> Result<(), ShadowLinkError>` in `src/ffi.rs`.
- [ ] T037 [US3] Run `cargo test test_debug_room` — all four tests pass.

**Checkpoint**: ✅ Debug room toggles on/off, emits structured diagnostics, survives external deletion.

---

## Phase 5: Integration & Persistence

**Purpose**: Wire everything together — session restore integration, cross-story edge cases, two-session integration test.

### Integration Tests

- [ ] T038 [P] Write `test_home_room_persistence_across_restarts` in `tests/test_us5_persistence.rs` — create family room, disconnect, restore session, verify `get_home_room()` returns same room with `is_home: true`.
- [ ] T039 [P] Write `test_home_room_cleared_on_leave` in `tests/test_us5_persistence.rs` — create family room, leave it, restore session, verify `get_home_room()` returns `None` or cached info with `state: Left`.
- [ ] T040 [P] Write `test_two_session_family_room_flow` in `tests/integration/mod.rs` — operator creates family room, invites member, member accepts, member calls `set_home_room`, both verify `is_home: true` in `list_rooms()`.

### Implementation

- [ ] T041 Wire `get_home_room()` into `restore_session()` path in `src/client.rs` — after session restore, populate `home_room_id` from `StoredSession` into in-memory state (already available via `load_home_room_id()`). No behaviour change needed — `get_home_room()` reads from stored session directly.
- [ ] T042 Handle edge case: `set_home_room` on non-encrypted room in `src/rooms.rs` — verify `room.enable_encryption()` is called and error propagated. Already implemented in T022; verify with test.
- [ ] T043 Handle edge case: `get_home_room()` when persisted room is deleted in `src/rooms.rs` — if room not found in joined/invited/left lists, return `None` (silently clear stale home room ID from persistence).
- [ ] T044 Run full test suite: `cargo test` — all existing tests + all new tests pass. `cargo clippy -- -D warnings` — zero warnings. `cargo fmt -- --check` — clean.

**Checkpoint**: ✅ All stories integrated. Session persistence verified. Two-session flow works.

---

## Phase 6: Polish & Documentation

**Purpose**: Final cleanup, documentation, real-server smoke test prep.

- [ ] T045 [P] Update `CHANGELOG.md` with 003-family-room entries under Unreleased — new functions, `RoomInfo` fields, debug room.
- [ ] T046 [P] Bump `VERSION` file to `0.3.0` (MINOR bump — new functionality, no breaking changes).
- [ ] T047 [P] Update `Cargo.toml` version to `0.3.0`.
- [ ] T048 [P] Update `README.md` implementation status table — add 003 Family Room Semantics row.
- [ ] T049 Run `cargo llvm-cov --all-targets` — verify coverage report generated, no regression.
- [ ] T050 Run gitleaks: `gitleaks detect --no-git` — zero secrets.

**Checkpoint**: ✅ Ready for merge and real-server smoke test (Phase 6 from plan.md — manual).

---

## Dependencies & Execution Order

### Phase Dependencies

- **Phase 1 (Foundational)**: No dependencies — starts immediately. BLOCKS all user stories.
- **Phase 2 (US1 — create_family_room)**: Depends on Phase 1. No other story deps.
- **Phase 3 (US2 — set/get/list home)**: Depends on Phase 1. No other story deps.
- **Phase 4 (US3 — debug room)**: Depends on Phase 1. No other story deps.
- **Phase 5 (Integration)**: Depends on Phases 2, 3, 4.
- **Phase 6 (Polish)**: Depends on Phase 5.

### User Story Dependencies

- **US1 (Create Family Room)**: Foundation only — no other story deps.
- **US2 (View/Pin Home Room)**: Foundation only — no other story deps. Does NOT require US1 (uses `set_home_room` with generic rooms for its tests).
- **US3 (Debug Room)**: Foundation only — no other story deps.

All three user stories are independently testable per spec. They can be implemented in parallel after Phase 1.

### Within Each User Story

- Tests written first, verified FAILING before implementation
- Private helper functions before public API functions
- Module-level implementation before FFI wrapper
- Story complete (all tests green) before moving to next

### Parallel Opportunities

```
Phase 1 ──┬── Phase 2 (T008-T016) ──┐
          ├── Phase 3 (T017-T026) ──┼── Phase 5 (T038-T044) ── Phase 6 (T045-T050)
          └── Phase 4 (T027-T037) ──┘
```

- T001-T006: All [P] — different files or non-overlapping struct fields
- T008-T011: All [P] — independent test functions
- T012-T015: T012 (alias helper) blocks T013 (create_family_room); T013 blocks T015 (FFI)
- T017-T021: All [P] — independent test functions
- T027-T030: All [P] — independent test functions
- T033-T035: All [P] — different error paths, no shared state
- T038-T040: All [P] — independent test functions
- T045-T048: All [P] — different files
- Phases 2/3/4 can run concurrently if multiple developers (or parallel sub-agents)

---

## Summary

| Phase | Tasks | Story |
|-------|-------|-------|
| Phase 1 — Foundational | T001–T007 (7) | — |
| Phase 2 — US1: Create Family Room | T008–T016 (9) | US1 |
| Phase 3 — US2: View/Pin Home Room | T017–T026 (10) | US2 |
| Phase 4 — US3: Debug Room | T027–T037 (11) | US3 |
| Phase 5 — Integration | T038–T044 (7) | — |
| Phase 6 — Polish | T045–T050 (6) | — |

**Total**: 50 tasks across 6 phases.
