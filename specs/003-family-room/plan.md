# Implementation Plan: Family Room Semantics

**Branch**: `003-family-room` | **Date**: 2026-06-15 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `/specs/003-family-room/spec.md`

## Summary

Implement the family room concept from spec 001-shadowlink-core that was deferred: `create_family_room` with alias generation and `is_home` flag, `set_home_room`/`get_home_room` for pinning/adoption, `enable_debug_room` toggle with structured diagnostic events, and `RoomInfo` struct extension (`is_home: bool`, `alias: Option<String>`). Four new public functions, two new struct fields, zero breaking changes to the existing API.

## Technical Context

**Language/Version**: Rust 2024 edition

**Primary Dependencies**: `matrix-rust-sdk` 0.7 (git), `ruma` (Matrix API types), `tokio` 1 (async runtime), `serde`/`serde_json` (persistence), `tracing` (diagnostics)

**Storage**: `shadowlink_data/session.json` (JSON — extends existing `StoredSession` with `home_room_id: Option<String>`). SDK SQLite store unchanged.

**Testing**: `cargo test` (unit + integration against local Docker Synapse on `localhost:8008`). New tests in `tests/test_us2_rooms.rs` (family room) and `tests/test_us5_persistence.rs` (persistence of home room ID).

**Target Platform**: Linux x86_64 (CI), macOS/Linux (developer). Consumed by Flutter via FFI and by shadowlink-cli via library dep.

**Project Type**: Rust library crate — additive changes only, no new binaries.

**Performance Goals**: `create_family_room` <3s (dominated by room creation + alias setting). `get_home_room` <1ms (in-memory lookup from persisted ID). `enable_debug_room` <3s on first enable (room creation), <1ms on subsequent toggles.

**Constraints**: Zero breaking changes to existing `create_room`, `list_rooms`, `RoomInfo` consumers. All new fields have sensible defaults (`false`, `None`). No new dependencies.

**Scale/Scope**: ~200 lines of new code across `src/rooms.rs`, `src/client.rs`, `src/ffi.rs`. ~100 lines of new tests.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Evidence |
|-----------|--------|----------|
| **I. Clean Separation** | ✅ PASS | All changes internal to `src/`. No UI code, no platform channels. FFI surface is additive. |
| **II. Local-First Privacy** | ✅ PASS | Home room ID stored locally in `session.json`. No server-side state. No telemetry. |
| **III. Minimal API Surface** | ✅ PASS | 4 new public functions justified by spec FR-001 through FR-014. All delegate to existing SDK patterns. |
| **IV. Test-First** | ✅ PASS | Spec exists (14 FRs, 12 acceptance scenarios). Tests written before implementation per Red-Green-Refactor. |
| **V. Battery & Permission** | ✅ PASS | No new background work. No new polling. Debug events are push-based (emitted on error, not polled). |
| **VI. CI Pipeline** | ✅ PASS | Existing CI gates apply. No pipeline changes needed. `cargo test`, `cargo clippy`, `cargo fmt`, `cargo llvm-cov` all unchanged. |

**Gate status**: ✅ PASS. No violations.

## Project Structure

### Documentation (this feature)

```text
specs/003-family-room/
├── plan.md              # This file
├── spec.md              # Feature specification
├── research.md          # Phase 0 — 6 research tasks resolved
├── data-model.md        # Phase 1 — 5 entities, state transitions
├── quickstart.md        # Phase 1 — local + real-server smoke test guide
├── contracts/
│   └── rooms-contract.md  # Phase 1 — 4 function contracts + FFI surface
└── tasks.md             # Phase 2 output (speckit-tasks — NOT created by plan)
```

### Source Code (repository root)

```text
src/
├── rooms.rs             # +create_family_room, +set_home_room, +get_home_room, +alias derivation helper
├── client.rs            # +enable_debug_room, +emit_diagnostic (private), StoredSession.home_room_id, Session.debug_room_*
├── ffi.rs               # +create_family_room, +set_home_room, +get_home_room, +enable_debug_room on ShadowLinkApi
├── lib.rs               # unchanged (modules already public)
├── error.rs             # unchanged (no new error variants needed)
└── encryption.rs        # unchanged

tests/
├── test_us2_rooms.rs           # +family room tests (create, set, get, is_home flag, alias)
├── test_us5_persistence.rs     # +home room ID persistence across restarts
└── integration/mod.rs          # +two-session family room invitation flow
```

**Structure Decision**: Single project — all changes are additive to existing modules. No new modules needed. The feature extends `rooms`, `client`, and `ffi` in-place.

## Implementation Order

### Phase 1: Data model extension (no new functions yet)

1. Add `is_home: bool` and `alias: Option<String>` to `RoomInfo` in `src/rooms.rs`
2. Update `to_room_info()` helper to populate new fields (read alias from SDK room, default `is_home: false`)
3. Extend `StoredSession` with `home_room_id: Option<String>` in `src/client.rs`
4. Add `debug_room_id: Option<String>` and `debug_room_enabled: bool` to `Session` struct
5. Verify: `cargo build` — existing code compiles with new fields (default values)

### Phase 2: Family room creation

6. Implement `create_family_room()` in `src/rooms.rs`:
   - Derive alias localpart from name
   - Set `room_alias_name` on create request
   - Persist room ID as `home_room_id`
   - Return `RoomInfo` with `is_home: true`
7. Implement alias derivation helper in `src/rooms.rs` (private function)
8. Add FFI wrapper `ShadowLinkApi::create_family_room()` in `src/ffi.rs`
9. Write tests: `test_create_family_room_basic`, `test_create_family_room_alias`, `test_create_family_room_replaces_previous`

### Phase 3: Home room pinning and retrieval

10. Implement `set_home_room()` in `src/rooms.rs`:
    - Verify user is member
    - Enable E2EE if not encrypted
    - Persist as home room
11. Implement `get_home_room()` in `src/rooms.rs`:
    - Read `home_room_id` from stored session
    - Look up room in joined rooms
    - Return `RoomInfo` with `is_home: true` or `None`
12. Add FFI wrappers in `src/ffi.rs`
13. Update `list_rooms()` to mark the home room with `is_home: true`
14. Write tests: `test_set_home_room`, `test_get_home_room_none`, `test_list_rooms_marks_home`

### Phase 4: Debug room

15. Implement `enable_debug_room()` in `src/client.rs`:
    - Create `"ShadowLink Debug"` room on first enable
    - Toggle emission flag
    - Re-create if deleted externally
16. Implement `emit_diagnostic()` private function in `src/client.rs`:
    - Check `debug_room_enabled` flag
    - Format diagnostic event (text + JSON metadata)
    - Send to debug room
17. Wire diagnostic emission into sync error paths, connection failure handlers
18. Add FFI wrapper in `src/ffi.rs`
19. Write tests: `test_debug_room_creation`, `test_debug_room_toggle`, `test_debug_room_no_pii`

### Phase 5: Integration and concurrency

20. Wire `get_home_room()` into session restore path — populate on `restore_session()`
21. Handle edge case: home room deleted externally → `get_home_room()` returns cached info
22. Handle edge case: `set_home_room` on non-encrypted room → enable E2EE before pinning
23. Integration test: two-session family room flow (operator creates, invites, member accepts, pins)
24. Verify: `cargo test` — all gates green. `cargo clippy`, `cargo fmt`, `cargo llvm-cov`.

### Phase 6: Real-server smoke test (manual)

25. Create `@shadowlink-test-familyop:matrix.org` account (user action)
26. Run through the real-server verification checklist from `quickstart.md`
27. Fix any real-world issues found (TLS, rate limiting, alias rejection)
28. Document results in CHANGELOG

## Dependencies

```
Phase 1 (data model) ──── Phase 2 (create_family_room) ──── Phase 3 (set/get_home)
                                    │                              │
                                    └── Phase 4 (debug room) ──────┘
                                                      │
                                              Phase 5 (integration)
                                                      │
                                              Phase 6 (smoke test)
```

- Phase 1 blocks everything (struct changes needed before any function can compile)
- Phases 2, 3, 4 are independent once Phase 1 is done — can be parallelized
- Phase 5 depends on Phases 2-4
- Phase 6 is independent (manual, post-implementation)

## Risk Register

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| Homeserver rejects alias, room creation fails entirely (not just alias) | Low | Medium | Already handled: alias is best-effort, room still created. If server rejects room because of alias, error is surfaced. |
| `enable_encryption()` on non-encrypted room fails silently | Low | Medium | SDK returns error for encryption failures. Contract requires `OperationFailed` on E2EE rejection. |
| Home room ID stored but room later deleted | Medium | Low | `get_home_room()` returns cached `RoomInfo` with best-known state. Flutter layer handles the UX. |
| Debug room creation races with sync events | Low | Low | `emit_diagnostic()` is called from sync handler (serial). No concurrent debug room creation. |
| Existing tests break due to `RoomInfo` field additions | Low | Low | New fields have defaults. All existing `RoomInfo` construction sites updated. `cargo test` catches any misses. |

## Verification Plan

1. **Unit tests**: Every new function has a test against local Synapse
2. **Integration tests**: Two-session family room flow (connect → create → invite → accept → pin → list)
3. **Persistence tests**: Create family room, disconnect, restore session, verify `get_home_room()` returns it
4. **Edge case tests**: Invalid names, missing rooms, non-encrypted rooms, debug room re-creation
5. **CI gates**: `cargo test` (all green), `cargo clippy -- -D warnings` (zero), `cargo fmt -- --check` (clean), `cargo llvm-cov --all-targets` (coverage generated)
6. **Manual smoke test**: Real homeserver verification per `spec.md` § Real-World Homeserver Verification
