---
title: "11. Risks & Technical Debt"
---

# 11. Risks & Technical Debt

## 11.1 Risk Matrix

| ID | Risk | Probability | Impact | Risk Level |
|----|------|-------------|--------|------------|
| **R1** | matrix-rust-sdk API instability | Medium | High | 🔴 Critical |
| **R2** | FFI complexity and memory bugs | Medium | High | 🔴 Critical |
| **R3** | Mobile battery/permission quirks | High | Medium | 🟡 High |
| **R4** | Matrix spec evolution (MSC churn) | Medium | Medium | 🟡 High |
| **R5** | E2EE session corruption | Low | Critical | 🟡 High |
| **R6** | Flutter FFI binding maintenance | Medium | Low | 🟢 Medium |
| **R7** | Cross-compilation toolchain breakage | Low | Medium | 🟢 Medium |
| **R8** | Single maintainer bus factor | Medium | Medium | 🟡 High |
| **R9** | Dependency supply chain | Low | High | 🟡 High |
| **R10** | iOS platform divergence | Low | Medium | 🟢 Medium |

## 11.2 Risk Details & Mitigations

### R1: matrix-rust-sdk API Instability

- **Description:** The `matrix-rust-sdk` crate is under active development by the Matrix.org
  Foundation. Breaking API changes occur between minor versions, requiring adaptation in the
  Rust Core's integration layer.
- **Mitigation:** Pin SDK version in `Cargo.toml` with exact semver. Wrap SDK types behind
  our own domain types to limit blast radius of upstream changes. Track SDK changelog
  proactively.
- **Contingency:** If a critical upstream change breaks integration, fork the SDK at the last
  compatible commit and maintain a patch branch until upstream stabilizes.

### R2: FFI Complexity and Memory Bugs

- **Description:** Cross-language memory management between Rust and Dart via C-ABI is
  inherently error-prone. Use-after-free, double-free, and ownership confusion are classic
  FFI failure modes.
- **Mitigation:** Design FFI ownership contracts explicitly (caller-owns vs. callee-owns).
  Use `#[repr(C)]` structs with clear lifetime documentation. Fuzz the FFI boundary.
  Run under sanitizers (ASan, UBSan) in CI.
- **Contingency:** If raw FFI proves unsustainable, evaluate `uniffi` or `flutter_rust_bridge`
  as migration paths (ADR-001).

### R3: Mobile Battery & Permission Quirks

- **Description:** Android Doze, App Standby, and vendor-specific battery optimizations can
  kill or throttle background sync. Location permissions are increasingly restricted per
  Android version.
- **Mitigation:** Use Android's `WorkManager` for sync scheduling (Flutter side). Respect
  `onLowMemory` and background execution limits. Use fused location provider with adaptive
  intervals. Test on physical devices across multiple OEMs.
- **Contingency:** Provide user-facing battery optimization guidance in the Flutter app.
  Accept degraded sync frequency under battery constraints.

### R4: Matrix Spec Evolution (MSC Churn)

- **Description:** The Matrix protocol evolves through Matrix Spec Change (MSC) proposals.
  Features like location events (`m.location`) may not yet be stable in the spec.
- **Mitigation:** Target stable Matrix spec endpoints where possible. Isolate unstable/MSC
  code paths behind feature flags. Monitor MSC status for location and other planned features.
- **Contingency:** If an MSC we depend on is rejected, fall back to custom state events or
  alternative encodings within the Matrix event model.

### R5: E2EE Session Corruption

- **Description:** Corrupted Olm/Megolm session state can cause permanent decryption failures
  for affected rooms, effectively breaking E2EE for those conversations.
- **Mitigation:** Rely on matrix-rust-sdk's built-in session management and key backup.
  Implement session health checks. Surface decryption errors to Flutter as actionable
  events (e.g., "verify device" prompt).
- **Contingency:** If session corruption is detected, trigger automatic key re-request or
  guide user through interactive verification recovery.

### R6: Flutter FFI Binding Maintenance

- **Description:** The Dart-side FFI bindings must stay synchronized with the Rust-side API.
  Breaking changes in the Rust API require corresponding Dart binding updates.
- **Mitigation:** Version the FFI API explicitly. Test FFI bindings in CI with a minimal
  Dart test harness. Document breaking changes in changelog.
- **Contingency:** Generate Dart bindings from Rust source using code generation (if manual
  maintenance overhead grows too large).

### R7: Cross-Compilation Toolchain Breakage

- **Description:** NDK updates, cargo-ndk changes, or LLVM version bumps can break Android
  cross-compilation.
- **Mitigation:** Pin NDK version. Use Docker-based reproducible builds. Cache build artifacts
  in CI.
- **Contingency:** Maintain a `rust-toolchain.toml` with exact toolchain version. CI matrix
  tests multiple NDK versions.

### R8: Single Maintainer Bus Factor

- **Description:** The project currently has limited contributor breadth, creating a key-person
  dependency risk.
- **Mitigation:** Comprehensive documentation (this arc42 set). Clean, conventional codebase.
  Public issue tracker. Explicit contribution guide (`CONTRIBUTING.md`).
- **Contingency:** Open-source visibility via GitHub. The MIT/Apache 2.0 license ensures the
  project can be forked and continued by the community.

### R9: Dependency Supply Chain

- **Description:** Transitive dependency compromise, typo-squatting, or unmaintained crates
  in the dependency tree.
- **Mitigation:** `cargo audit` in CI. `cargo-deny` for license compliance. Minimal dependency
  footprint (Karpathy protocol). Prefer crates maintained by well-known organizations.
- **Contingency:** Vendor critical dependencies. Have a rapid response plan for CVE remediation
  (patch, test, release within 48 hours).

### R10: iOS Platform Divergence

- **Description:** iOS background execution model differs significantly from Android. Platform-
  specific code may be required for sync scheduling, location, and memory limits.
- **Mitigation:** Platform-specific modules behind `#[cfg(target_os)]` gates. Design abstractions
  that accommodate both Android and iOS background models from the start.
- **Contingency:** If iOS support proves architecturally incompatible, delay iOS launch and
  focus on Android-first delivery. The core library remains iOS-compatible at the Rust level.

## 11.3 Technical Debt Tracking

No known technical debt at project inception. Technical debt will be tracked as:

- `// TODO(tech-debt):` comments in source code linked to GitHub issues.
- ADR amendments (Section 9) when architectural shortcuts are taken.
- arc42 section updates when documentation drifts from implementation.
