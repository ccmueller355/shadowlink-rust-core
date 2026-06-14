# Tasks: CLI Integration

**Input**: Design documents from `/specs/002-cli-integration/`

**Prerequisites**: plan.md ✅, spec.md ✅, research.md ✅, data-model.md ✅, contracts/ ✅, quickstart.md ✅

**Tests**: Spec includes acceptance scenarios (US1–US4 manual integration tests). Automated E2E tests are a future task.

**Organization**: Tasks grouped by user story (SpecKit spec has US1–US4), followed by remaining work. CLI is already fully implemented (382 lines, 11 commands) — Phase 1–6 tasks are retroactively marked done. Phase 7 has pending items.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (US1, US2, US3, US4)
- Exact file paths in descriptions — CLI is a single-file binary in separate repo

## Path Conventions

CLI lives in separate repository: `github.com/ccmueller355/shadowlink-cli`

```
shadowlink-cli/
├── Cargo.toml             # git dep on shadowlink-rust-core
├── src/main.rs            # 382 lines — all CLI logic
├── .github/workflows/
│   └── ci.yml             # build, test, clippy, fmt, gitleaks
└── tests/                 # unit tests (future — currently none)
```

---

## Phase 1: Setup (Project Scaffold)

**Purpose**: Repository, build system, CI foundation

- [x] T001 Create shadowlink-cli repository with Cargo project structure
- [x] T002 [P] Configure Cargo.toml: git dependency on shadowlink-rust-core (main branch), clap 4 (derive), tokio, tracing
- [x] T003 [P] Configure CI pipeline — `ci.yml`: build, test, clippy, fmt, gitleaks jobs
- [x] T004 [P] Add .gitignore (target/, shadowlink_data/) and .gitleaks.toml
- [x] T005 [P] Add CodeWhale instructions in .codewhale/instructions.md
- [x] T006 Add LICENSE (MIT OR Apache-2.0) and README.md

**Checkpoint**: ✅ Repository scaffolded and CI green on main

---

## Phase 2: Foundational (Blocks All Stories)

**Purpose**: Core dependency integration, session restore pattern, error handling

- [x] T007 Establish CLI→Core integration pattern in `Cargo.toml` — git dep resolving to all core modules (client, rooms, messaging, location)
- [x] T008 Implement session persistence path convention in `src/main.rs` — `session_dir()` / `session_file()` functions, CWD-relative `shadowlink_data/`
- [x] T009 Implement `main()` entry point in `src/main.rs` — tokio runtime, tracing subscriber with `RUST_LOG` env filter, clap derive parse + subcommand dispatch
- [x] T010 [P] Define Cli struct with all subcommands in `src/main.rs` — clap Parser + Subcommand derive, all 11 variants
- [x] T011 Implement session restore pattern — `restore_session()` → `stop_sync()` wrapper used by all commands except connect/disconnect
- [x] T012 Implement error handling contract — match on `ShadowLinkError` variants, non-zero exit, user-facing stderr messages

**Checkpoint**: ✅ Foundation ready — core dep resolves, session restore works, error handling in place

---

## Phase 3: User Story 1 — Connect and Authenticate (Priority: P1) 🎯 MVP

**Goal**: Developer connects CLI to a Matrix homeserver, establishes authenticated session persisted to disk

**Independent Test**: `shadowlink connect <url> <user> <pass>` → verify `session.json` written, `list-rooms` uses restored session

### Implementation for User Story 1

- [x] T013 [US1] Implement `cmd_connect()` in `src/main.rs` — call `client::connect()`, persist session, print success with file path
- [x] T014 [US1] Handle connect error cases in `src/main.rs` — ConnectionFailed (unreachable URL), AuthenticationFailed (bad creds)
- [x] T015 [US1] Implement `cmd_disconnect()` in `src/main.rs` — call `client::disconnect()`, remove session file + SQLite store
- [x] T016 [US1] Validate FR-011 (auto session restore) — all commands except connect/disconnect restore session first

**Checkpoint**: ✅ User Story 1 complete — connect, disconnect, session restore all work

---

## Phase 4: User Story 2 — Room Operations (Priority: P2)

**Goal**: Developer creates/manages E2EE rooms via CLI subcommands on a restored session

**Independent Test**: Two users — User A creates room, invites B. B accepts, lists rooms, leaves. A lists rooms (sees B gone).

### Implementation for User Story 2

- [x] T017 [US2] Implement `cmd_create_room()` in `src/main.rs` — call `rooms::create_room()`, print room ID
- [x] T018 [US2] Implement `cmd_list_rooms()` in `src/main.rs` — call `rooms::list_rooms()`, display [state] name (room_id) for all rooms
- [x] T019 [US2] Map `RoomState` enum for display — Joined → "[joined]", Invited → "[invited]", Left → "[left]"
- [x] T020 [US2] Implement `cmd_invite()` in `src/main.rs` — call `rooms::invite_user()`, confirm invitation sent
- [x] T021 [US2] Implement `cmd_accept()` in `src/main.rs` — call `rooms::accept_invite()`, confirm joined
- [x] T022 [US2] Implement `cmd_leave()` in `src/main.rs` — call `rooms::leave_room()`, confirm left

**Checkpoint**: ✅ User Story 2 complete — all 5 room operations work

---

## Phase 5: User Story 3 — Encrypted Messaging (Priority: P3)

**Goal**: Developer sends/receives E2EE messages via CLI

**Independent Test**: Two users in same room — User A sends text, User B receives via `listen`. User A runs `get-history` to see B's reply.

### Implementation for User Story 3

- [x] T023 [US3] Implement `cmd_send()` in `src/main.rs` — call `messaging::send_text()`, print event ID
- [x] T024 [US3] Implement `cmd_get_history()` in `src/main.rs` — call `messaging::get_history()`, print messages with sender + content
- [x] T025 [US3] Map `MessageContent` enum for display — Text → body, Media → "[shared media: {filename}]", Location → "[location: {lat}, {lng}]"
- [x] T026 [US3] Implement `cmd_listen()` in `src/main.rs` — register message callback via `messaging::register_message_callback()`, block on Ctrl+C
- [x] T027 [US3] Support optional room filter on listen — `shadowlink listen [room_id]` filters to one room, omitting listens to all
- [x] T028 [US3] Handle listen shutdown — on Ctrl+C, stop sync and exit cleanly

**Checkpoint**: ✅ User Story 3 complete — send, get-history, listen all work with E2EE

---

## Phase 6: User Story 4 — Location Sharing (Priority: P4)

**Goal**: Developer shares location beacons to rooms via CLI

**Independent Test**: Share location to room, verify event appears in message history

### Implementation for User Story 4

- [x] T029 [US4] Implement lat/lng validation in `src/main.rs` — latitude -90..90, longitude -180..180 (FR-019)
- [x] T030 [US4] Implement `cmd_share_location()` in `src/main.rs` — validate bounds, call `location::share_location()`, print event ID

**Checkpoint**: ✅ User Story 4 complete — location sharing with bounds validation works

---

## Phase 7: Polish & Remaining Work

**Purpose**: Cross-cutting concerns, CI hardening, future enhancements

### Documentation (Completed)

- [x] T031 [P] Write plan.md — retroactive plan for already-implemented feature in `specs/002-cli-integration/plan.md`
- [x] T032 [P] Write research.md — 6 architectural decisions in `specs/002-cli-integration/research.md`
- [x] T033 [P] Write data-model.md — 4 entities in `specs/002-cli-integration/data-model.md`
- [x] T034 [P] Write cli-core-contract.md — 14 function signatures in `specs/002-cli-integration/contracts/cli-core-contract.md`
- [x] T035 [P] Write quickstart.md — build, usage, manual integration test guide in `specs/002-cli-integration/quickstart.md`
- [x] T036 [P] Write tasks.md — this file
- [x] T037 Update SPECKIT markers to point to 002 plan in `.codewhale/instructions.md` and `.github/copilot-instructions.md`

### Automated E2E Testing (Pending)

- [x] T038 [P] Add Synapse service container to CLI `.github/workflows/ci.yml` — Docker service on port 8008
- [x] T039 Add `get-history` integration test — send message, fetch history, verify message appears
- [x] T040 Add connect→disconnect integration test — connect, verify session.json, disconnect, verify cleanup

### Enhancements (Pending)

- [x] T041 [P] Add `--data-dir` flag for shared session directory — override CWD-relative default

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies — started immediately ✅
- **Foundational (Phase 2)**: Depends on Setup — blocks all user stories ✅
- **User Stories (Phase 3–6)**: All depend on Foundational. Stories are independent of each other and could be parallel.
- **Polish (Phase 7)**: Pending items depend on all stories being complete. E2E tests (T038–T040) depend on CI availability; `--data-dir` (T041) is standalone.

### User Story Dependencies

- **US1 (Connect)**: Foundation only — no other story deps ✅
- **US2 (Rooms)**: Foundation only — no other story deps ✅
- **US3 (Messaging)**: Foundation only — no other story deps ✅
- **US4 (Location)**: Foundation only — no other story deps ✅

All four user stories are independently testable per SpecKit spec.

### Within Each User Story

- Core function call before error handling
- Error handling before display formatting
- Story complete before next priority (per spec)

### Parallel Opportunities

- All 11 commands are in one file (`src/main.rs`) — serial within the file, but all four stories are independent
- T038, T041 can run in parallel (different concerns)
- T039, T040 can run in parallel (different test scenarios)

---

## Summary

| Phase | Tasks | Status |
|-------|-------|--------|
| Phase 1 — Setup | T001–T006 (6) | ✅ All done |
| Phase 2 — Foundation | T007–T012 (6) | ✅ All done |
| Phase 3 — US1: Connect | T013–T016 (4) | ✅ All done |
| Phase 4 — US2: Rooms | T017–T022 (6) | ✅ All done |
| Phase 5 — US3: Messaging | T023–T028 (6) | ✅ All done |
| Phase 6 — US4: Location | T029–T030 (2) | ✅ All done |
| Phase 7 — Polish | T031–T041 (11) | ✅ All done |

**Total**: 41 tasks — **41 completed ✅**
