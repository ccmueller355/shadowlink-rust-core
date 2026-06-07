---
title: "4. Solution Strategy"
---

# 4. Solution Strategy

## 4.1 Core Architectural Decisions (Deferred)

The following decisions are identified as critical and will be formally resolved during the
SpecKit Plan phase. They are listed here to establish the decision backlog.

| Decision ID | Topic | Status |
|-------------|-------|--------|
| **ADR-001** | FFI bridge approach: raw `extern "C"` vs. `uniffi` vs. `flutter_rust_bridge` | 🔨 Pending |
| **ADR-002** | Async runtime selection: `tokio` single-threaded vs. `async-std` vs. custom executor | 🔨 Pending |
| **ADR-003** | matrix-rust-sdk version pinning and upgrade strategy | 🔨 Pending |
| **ADR-004** | Error model design: flat error codes vs. structured error types across FFI | 🔨 Pending |
| **ADR-005** | Location event format: Matrix `m.location` (unstable) vs. custom state event | 🔨 Pending |
| **ADR-006** | Session storage: matrix-rust-sdk built-in vs. custom persistence layer | 🔨 Pending |

## 4.2 Development Strategy

### SpecKit-First Workflow

Every component follows a three-phase lifecycle:

1. **Specify** — Write behavioral specifications with formal input bounds, edge scenarios, and
   exit criteria. Documented in the SpecKit contract for the component.
2. **Implement** — Code against the specification. No logic beyond what the spec demands.
   Follow the Karpathy protocol: radical encapsulation, zero abstraction excess.
3. **Verify** — Convert spec assertions into `#[test]` functions in `src/tests.rs`. Use
   `cargo-llvm-cov` for coverage tracking. Spec passes or code doesn't ship.

### arc42 Living Documentation

Architecture documentation is first-class source code. Every structural change is accompanied
by synchronous updates to the corresponding arc42 section. This document set is compiled by
VitePress and deployed to GitHub Pages on every merge to `main`.

### Karpathy Lean Coding Protocol

- **Radical Encapsulation:** Group related types, handlers, and logic in high-density files.
  No fractured directory trees with single-type modules.
- **Surgical Edits:** Touch only what the feature or fix demands. Match existing code style.
- **Zero Abstraction Excess:** No speculative traits, no unused generics, no "future-proofing"
  layers. Core language primitives over framework machinery.

## 4.3 Technology Stack Decisions

| Layer | Choice | Rationale |
|-------|--------|-----------|
| **Language** | Rust 2024 | Memory safety, C-ABI FFI, matrix-rust-sdk ecosystem |
| **Matrix SDK** | matrix-rust-sdk | Canonical Rust SDK, maintained by Matrix.org |
| **Async Runtime** | tokio | Required by matrix-rust-sdk; mobile-optimized configuration |
| **FFI** | C-ABI | Universal interop; supported by Flutter's `dart:ffi` |
| **Testing** | cargo test + llvm-cov | Native tooling, no external test frameworks |
| **CI** | GitHub Actions | Native GitHub integration; gitleaks + clippy gates |
| **Docs** | VitePress + Mermaid.js | Static site with diagram support; deploy to GitHub Pages |
