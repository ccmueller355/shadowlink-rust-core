---
title: "2. Architecture Constraints"
---

# 2. Architecture Constraints

## 2.1 Technical Constraints

| Constraint | Detail | Rationale |
|------------|--------|-----------|
| **Language** | Rust 2024 edition | Safety guarantees, zero-cost abstractions, FFI compatibility. |
| **SDK** | [matrix-rust-sdk](https://github.com/matrix-org/matrix-rust-sdk) | Canonical Rust Matrix client implementation. Provides sync, E2EE (Olm/Megolm), room state, and event handling. |
| **FFI** | C-ABI via `extern "C"` | Flutter interop requires C-compatible calling conventions. No higher-level RPC (gRPC/Cap'n Proto) — direct FFI for minimal latency. |
| **Async Runtime** | `tokio` (single-threaded, mobile-optimized) | matrix-rust-sdk depends on tokio. Mobile context demands bounded thread pools and battery-aware scheduling. |
| **Build Target** | Android (arm64-v8a, armeabi-v7a) via `cargo-ndk` | Android-first deployment. iOS (aarch64) planned as follow-on. |
| **Test Framework** | `cargo test` + `cargo-llvm-cov` | Native Rust test runner with LLVM source-based coverage for SpecKit verification. |

## 2.2 Organizational Constraints

| Constraint | Detail |
|------------|--------|
| **License** | MIT / Apache 2.0 dual-license. All contributions must be compatible. |
| **Commit Convention** | [Conventional Commits](https://www.conventionalcommits.org/) — `feat(module):`, `fix(module):`, `docs:`, etc. Enforced in CI. |
| **Versioning** | [Semantic Versioning 2.0](https://semver.org/). Breaking FFI changes require major version bump. |
| **Workflow** | SpecKit behavioral specification → implementation → automated verification. No code ships without passing SpecKit-derived tests. |
| **CI Gates** | Build, test, coverage, `clippy` (strict), `rustfmt`, `gitleaks` (secrets scan). All must be green before merge. |
| **Repo Split** | Public core (`shadowlink-rust-core`, this repo) + public CLI (`shadowlink-cli`) + consuming Flutter applications. No mixed-license monorepo. |

## 2.3 Platform Constraints

| Constraint | Detail |
|------------|--------|
| **Primary Platform** | Android (API 26+) via Flutter. |
| **Future Platform** | iOS (15+) via Flutter. Architecture must not preclude iOS. |
| **No Cloud Infra** | Zero server-side components. The Rust core is a library, not a service. Users provide their own Matrix homeserver. |
| **Offline Capability** | Matrix sync state persisted locally (matrix-rust-sdk built-in storage). Graceful degradation when offline. |
| **Battery** | Background sync must respect Android Doze/App Standby. Location updates use fused provider with adaptive intervals. |
