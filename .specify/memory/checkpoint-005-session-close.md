# Checkpoint 005 — Session Close

**Date:** 2026-06-07
**Branch:** `001-shadowlink-core`
**Commits:** `36fb7a3` → `2fb98ce`

## Summary

Completed all 9 phases of the ShadowLink Rust Core implementation.
Finalized coverage integration into VitePress docs. Session closed.

## What was built

| Module | File | Lines | Status |
|---|---|---|---|
| Client | `src/client.rs` | ~209 | Handles, connect, disconnect, sync loop |
| Rooms | `src/rooms.rs` | ~212 | Create, list, invite, accept, leave |
| Messaging | `src/messaging.rs` | ~397 | Send text/media, history, callbacks |
| Location | `src/location.rs` | ~131 | Geo URI share/parse |
| E2EE | `src/encryption.rs` | ~286 | Device verification, cross-signing, key export |
| FFI | `src/ffi.rs` | ~185 | ShadowLinkApi for flutter_rust_bridge v2 |
| Error | `src/error.rs` | ~157 | 14+ variants, FFI-safe |
| Integration harness | `tests/` | 3 files | Synapse-gated, `#[ignore]` by default |

## CI Pipeline

7 jobs: build → test → coverage → clippy → fmt → gitleaks → pages
Coverage HTML integrated into VitePress via `docs/public/coverage/`

## Coverage

| Module | Coverage |
|---|---|
| error.rs | 100% |
| location.rs | 41.84% |
| messaging.rs | 10.93% |
| client.rs | 9.68% |
| encryption.rs | 6.85% |
| ffi.rs | 3.65% |
| rooms.rs | 0.00% |
| **Total** | **18.14%** |

Low coverage expected — 82% of surface area requires live Synapse.

## Pending

- [ ] `e2e-integration-tests` — Synapse-based round-trip tests (connect → rooms → messaging → E2EE) with CI service container
- [ ] Deploy docs to GitHub Pages (merge to `main`)

## Key Technical Decisions

- **matrix-rust-sdk v0.7.1** — SyncTimelineEvent from deserialized_responses, UserId via `<&UserId>::try_from()`, DeviceId via `OwnedDeviceId::from()`
- **flutter_rust_bridge v2** — Rust2DartSender at `flutter_rust_bridge::rust2dart::sender`, StreamSink codegen-generated
- **Dead code pattern** — Integration tests `#[ignore = "requires local Synapse"]`, common.rs `#![allow(dead_code)]`
