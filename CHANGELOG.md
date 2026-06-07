# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

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

[0.1.0]: https://github.com/ccmueller355/shadowlink-rust-core/releases/tag/v0.1.0
