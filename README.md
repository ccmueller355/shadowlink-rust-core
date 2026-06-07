# ShadowLink Rust Core

[![CI](https://github.com/ccmueller355/shadowlink-rust-core/actions/workflows/ci.yml/badge.svg)](https://github.com/ccmueller355/shadowlink-rust-core/actions/workflows/ci.yml)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE)
![Version](https://img.shields.io/badge/version-0.1.0-blue)

**Privacy-first Matrix protocol bridge** — the public, open-source Rust library
that powers the ShadowLink family communications app. Consumed via FFI by the
proprietary [ShadowLink Flutter application](https://github.com/ccmueller355/shadowlink-app).

## What is ShadowLink?

A lightweight, privacy-first family communication app built on the Matrix
protocol. E2EE chat, media sharing, and location sharing — no cloud infra,
no subscriptions, no surveillance. The Rust core handles:

- **Homeserver connection** — user-provided Matrix homeserver, authenticated session
- **E2EE room operations** — create, join, invite, leave, list encrypted rooms
- **Encrypted messaging** — text messages + media attachments (Olm/Megolm)
- **Location sharing** — static beacons and live location updates
- **Session persistence** — survive app restarts via SDK built-in SQLite store

## Quick Start

### Prerequisites

- Rust 1.75+
- Docker (for integration tests against local Synapse)
- `cargo-llvm-cov` (optional, for code coverage)

### Build & Test

```bash
cargo build
cargo test                         # requires local Synapse on :8008
cargo clippy -- -D warnings
cargo fmt -- --check
```

### Local Synapse (Integration Tests)

```bash
docker run -d --name synapse-test -p 8008:8008 \
  -e SYNAPSE_SERVER_NAME=localhost \
  -e SYNAPSE_REPORT_STATS=no \
  matrixdotorg/synapse:latest
cargo test
docker stop synapse-test && docker rm synapse-test
```

### Coverage

```bash
cargo install cargo-llvm-cov
cargo llvm-cov --all-targets --html
# open target/llvm-cov/html/index.html
```

### Documentation Site

```bash
npm install
npx vitepress build docs          # static site
npx vitepress dev docs             # dev server
```

## Architecture

```
┌─────────────────────────────────────────────┐
│           ShadowLink Flutter App            │
│  (proprietary — private repo)               │
└──────────────────┬──────────────────────────┘
                   │ FFI (flutter_rust_bridge v2)
┌──────────────────▼──────────────────────────┐
│         ShadowLink Rust Core (this repo)    │
│  ┌──────┬──────┬────────┬──────────┬──────┐ │
│  │client│rooms │messaging│location │ ffi  │ │
│  └──┬───┴──┬───┴───┬────┴────┬─────┴──┬───┘ │
│     └──────┴───────┴─────────┴────────┘     │
│                   │ error                   │
└───────────────────┼─────────────────────────┘
                    │ matrix-rust-sdk
┌───────────────────▼─────────────────────────┐
│      User-Provided Matrix Homeserver        │
│  (Synapse, Dendrite, or managed provider)   │
└─────────────────────────────────────────────┘
```

Full arc42 architecture documentation: [`docs/arc42/`](docs/arc42/)

## Implementation Status

| Phase | Status | Description |
|---|---|---|
| Skeleton | ✅ | Crate structure, CI, VitePress, gitleaks |
| Constitution | ✅ | Six architectural principles ratified |
| Specification | ✅ | 5 user stories, 20 functional requirements |
| Plan | ✅ | 8 ADRs, data model, FFI contract, 35 tasks |
| Implementation | ⏸️ | Blocking foundation done — 6/35 tasks complete |

See [`specs/001-shadowlink-core/`](specs/001-shadowlink-core/) for the full
specification, plan, and task list.

## Architecture Decisions (Summary)

| Decision | Choice | Rationale |
|---|---|---|
| FFI bridge | flutter_rust_bridge v2 | Dart bindings, async support, zero-copy |
| Async runtime | tokio | Required by matrix-rust-sdk |
| TLS | rustls-tls + vendored OpenSSL | No system headers needed |
| Error model | thiserror | Derive macros, Display impls |
| Location events | Custom `org.shadowlink.location` | Full control over event schema |
| Media | SDK content repository | No raw HTTP in this crate |

Full ADRs: [`specs/001-shadowlink-core/research.md`](specs/001-shadowlink-core/research.md)

## Project Manifest

See [`docs/PROJECT_MANIFEST.md`](docs/PROJECT_MANIFEST.md) for the full
ShadowLink Family Matrix App specification — tech stack, features, repo
strategy, and monetization model.

## CI Pipeline

8 jobs, all gated before merge:

| Job | Gate |
|---|---|
| Build | `cargo build --release` |
| Test | `cargo test` — zero skipped/ignored |
| Coverage | `cargo llvm-cov --all-targets` |
| Lint (clippy) | `cargo clippy -- -D warnings` |
| Format | `cargo fmt -- --check` |
| Secrets | `gitleaks detect` |
| Docs | `npx vitepress build docs` → GitHub Pages |
| Docs Preview | same build, PR preview |

See [`.github/workflows/ci.yml`](.github/workflows/ci.yml) for details.

## License

**Rust core (this repo):** MIT OR Apache-2.0 — you may use it under the
terms of either license at your option. See [`LICENSE`](LICENSE).

**ShadowLink Flutter app:** Proprietary (all rights reserved). Lives in a
private repository.

## Repository Strategy

- **Public** (`ccmueller355/shadowlink-rust-core`): Rust bridge + SDK integration
- **Private** (`ccmueller355/shadowlink-app`): Full Flutter app with custom UI,
  map styling, paid features. Depends on this crate.
