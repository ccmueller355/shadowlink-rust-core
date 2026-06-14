# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [0.2.3] — 2026-06-14

### Added

- **Graceful sync shutdown**: `stop_sync()` for stopping the sync loop
  without disconnecting or logging out
- **CodeWhale instructions**: MemPalace mining protocol (never mine root,
  always scope to dirs), skills reference table

### Changed

- **VERSION**: Bumped from 0.1.0 → 0.2.2 to match Cargo.toml

## [0.2.2] — 2026-06-10

### Added

- **Integration tests US2–US5**: 13 new tests across 4 files — room operations
  (create/list/invite/accept/leave), messaging (text/media/history/callback),
  location sharing (beacon/invalid coords), session persistence (restore/rooms/
  history).
- **Tracing subscriber**: `tracing-subscriber` with env-filter for debug
  visibility during integration test runs. Configurable via `RUST_LOG`.
- **Common test infra**: `init_tracing()` helper in `tests/common.rs` for
  one-time subscriber initialization per test binary.

### Changed

- **Cargo.toml**: Added `tracing-subscriber` dev-dependency (v0.3, env-filter).

## [0.2.1] — 2026-06-09

### Added

- **Unit test suite**: 33 new tests across 5 modules — `messaging.rs` (10),
  `encryption.rs` (6), `client.rs` (3), `location.rs` (6), `rooms.rs` (8).
  All pure-logic, zero Synapse dependency. Coverage 18.14% → 45.57%.
- **Coverage gate**: CI coverage job now enforces `--fail-under-lines 40` —
  pipeline fails if total line coverage drops below 40%.

### Fixed

- **CI coverage job**: Removed `--all-targets` flag (fragmented llvm-cov
  instrumentation on bench targets), split `--html --lcov` into separate
  `cargo llvm-cov --lcov` and `cargo llvm-cov report --html` steps to
  match cargo-llvm-cov 0.8.7+ API.

## [0.2.0] — 2026-06-07

### Added

- **Client module** (`src/client.rs`): `SessionHandle` with ref-counted
  `connect(address, user_id, password)` / `disconnect()` / sync loop with
  message callback dispatch.
- **Room management** (`src/rooms.rs`): `create_room(name, topic)` /
  `list_rooms()` / `invite_user(room_id, user_id)` / `accept_invite(room_id)` /
  `leave_room(room_id)`.
- **Messaging** (`src/messaging.rs`): `send_text(room_id, body)` /
  `send_media(room_id, path, mime)` / `get_history(room_id, limit)` /
  `register_message_callback(id, callback)` with sync event extraction and
  dispatch.
- **Location sharing** (`src/location.rs`): `share_location(room_id, lat, long,
  desc)` with geo URI encoding, `parse_location_content(event)` for both
  `m.location` and `m.space` fallback.
- **E2EE** (`src/encryption.rs`): `get_device(user_id, device_id)` /
  `get_own_device()` / `get_user_devices()` / `bootstrap_cross_signing()` /
  `cross_signing_status()` / `export_room_keys(path, passphrase)` /
  `import_room_keys(path, passphrase)`. DeviceInfo, DeviceTrust,
  CrossSigningStatus FFI-safe types.
- **FFI bridge** (`src/ffi.rs`): `ShadowLinkApi` struct wrapping all public
  API methods for flutter_rust_bridge v2 codegen. `register_message_callback`
  bridge via `Rust2DartSender`.
- **Error model expansion** (`src/error.rs`): Expanded to 14 variants —
  `DecryptionFailed`, `KeysNotExported`, `CrossSigningNotBootstrapped`,
  `MediaTooLarge`.
- **Integration test harness**: `tests/common.rs` (Synapse admin API
  registration), `tests/test_us1_connect.rs`, `tests/test_us2_rooms.rs`.
  8 tests, 7 `#[ignore]`'d (require `docker compose up`).
- **Docker Compose**: `docker-compose.yml` — Synapse homeserver for local
  integration testing.
- **Coverage integration**: HTML coverage report embedded in VitePress docs
  at `/coverage/` via `docs/public/coverage/`.
- **Copyright headers**: SPDX license identifiers (MIT OR Apache-2.0) on all
  Rust source files — REUSE-compliant.

### Changed

- **Cargo.toml**: Added `mime = "0.3"` direct dependency.
- **README.md**: Full implementation status table, updated architecture
  diagram with E2EE module.
- **CI pipeline**: Coverage report copied into `docs/public/coverage/` before
  VitePress build (works locally + deployed).

### Fixed

- **9 clippy warnings**: Collapsible `if` expressions, unused imports, dead
  code annotations.
- **3 lib warnings**: Unused variables, unnecessary casts.
- **Integration tests**: Gated with `#[ignore = "requires local Synapse"]`
  so `cargo test` passes in CI without a running Synapse.

## [0.1.0] — 2026-06-07

### Added

- **Project skeleton**: Cargo crate `shadowlink-rust-core` (edition 2024,
  dual-licensed MIT OR Apache-2.0).
- **Module stubs**: `src/` — `lib.rs`, `client.rs`, `rooms.rs`, `messaging.rs`,
  `location.rs`, `ffi.rs`, `error.rs`.
- **Error model**: `ShadowLinkError` enum with 10 structured variants via
  `thiserror` — covers connection, authentication, session, room, decryption,
  media, location, and storage failures.
- **Dependency resolution**: `matrix-sdk 0.7` (E2EE, SQLite, QR verification,
  rustls-tls), `tokio 1` (rt-multi-thread + macros), `reqwest 0.11` (rustls-tls),
  `ruma 0.9`, `serde` + `serde_json`, `tracing`, vendored OpenSSL.
- **CI pipeline**: GitHub Actions — build, test, coverage (llvm-cov), clippy,
  fmt, gitleaks secrets scan, VitePress pages deployment. 8 jobs, all gated.
- **Arc42 documentation**: 12-section architecture docs (891 lines) with
  Mermaid diagrams — introduction, constraints, context, strategy, building
  blocks, runtime, deployment, concepts, ADRs, quality requirements, risks,
  glossary.
- **VitePress site**: Mermaid-enabled documentation site at `docs/`. Builds
  to GitHub Pages.
- **Gitleaks secrets scanning**: `.gitleaks.toml` + pre-commit hook.
- **SpecKit pipeline**: constitution → specify → plan → tasks → implement.
  5 phases complete, Phase 6 in progress.
- **Constitution**: 6 architectural principles (Clean Separation, Local-First
  Privacy, Minimal API Surface, Test-First, Battery Discipline, CI Pipeline).
- **Feature specification**: 5 user stories (US1-US5), 20 functional requirements,
  8 success criteria, 15 acceptance scenarios.
- **Architecture decisions**: 8 ADRs covering FFI bridge (flutter_rust_bridge v2),
  async runtime (tokio), SDK version, error model, media pipeline, location
  events, TUF/cross-signing, iOS readiness.
- **Implementation plan**: 35 tasks across 9 phases, module→story mapping,
  dependency chain, test strategy.
- **FFI contract**: 15 documented function signatures with params, returns,
  error conditions, and story mapping.
- **Debug room specification**: FR-021–FR-026 — opt-in E2EE diagnostics room
  with structured events (human-readable + JSON metadata), PII filter,
  runtime toggle.
- **Family room lifecycle**: `create_family_room`, `set_home_room`,
  `get_home_room` — private, invite-only, operator-created.
- **Debuggability strategy (ADR-009)**: Two-layer approach — Debug Room
  (Matrix diagnostics channel) + tracing crate with PII-filtering subscriber,
  forwarded to Flutter via FFI callback.
- **FFI contract extension**: 6 new functions — `create_family_room`,
  `set_home_room`, `get_home_room`, `enable_debug_room`,
  `is_debug_room_enabled`, `get_debug_room_id`.
- **Rust edition bump**: 2021 → 2024 (matching installed rustc 1.95.0).
- **Business context in arc42 §1**: ShadowLink pricing model and paid app
  intention documented in Project Overview.

[0.2.2]: https://github.com/ccmueller355/shadowlink-rust-core/releases/tag/v0.2.2
[0.2.1]: https://github.com/ccmueller355/shadowlink-rust-core/releases/tag/v0.2.1
[0.2.0]: https://github.com/ccmueller355/shadowlink-rust-core/releases/tag/v0.2.0
[0.1.0]: https://github.com/ccmueller355/shadowlink-rust-core/releases/tag/v0.1.0
