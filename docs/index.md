---
title: ShadowLink Rust Core
layout: home
hero:
  name: ShadowLink
  text: Rust Core
  tagline: Privacy-first Matrix protocol bridge for family communications
  actions:
    - theme: brand
      text: Architecture Docs
      link: /arc42/
    - theme: alt
      text: Project Manifest
      link: /PROJECT_MANIFEST
features:
  - title: Matrix Protocol Bridge
    details: Full Matrix client integration via matrix-rust-sdk — E2EE chat, media sharing, room operations, and location events exposed through a clean FFI boundary.
  - title: Local-First Privacy
    details: No cloud infrastructure. All data stays local. The only server is the Matrix homeserver the user configures. E2EE ensures only family members can read messages.
  - title: FFI-Ready
    details: Designed from the ground up for Flutter integration via FFI. Clean API surface, structured error model, async-native with mobile battery discipline.
  - title: SpecKit-Verified
    details: Every component defined by behavioral specifications with formal input bounds, edge scenarios, and exit criteria — converted directly to automated test assertions.
  - title: arc42 Architecture
    details: Full 12-section arc42 documentation with Mermaid.js system diagrams, building block views, and cross-cutting concept maps. Living docs, always in sync.
  - title: CI Hardened
    details: GitHub Actions pipeline — build, test, coverage (llvm-cov), clippy, rustfmt, gitleaks secrets scan. All gates must pass before merge.
---
