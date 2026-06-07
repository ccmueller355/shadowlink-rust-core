---
title: "12. Glossary"
---

# 12. Glossary

## Core Terms

| Term | Definition |
|------|------------|
| **arc42** | A standardized template for documenting software architectures, originating from German
engineering practice. Provides 12 structured sections covering goals, constraints, building blocks,
runtime, deployment, concepts, decisions, quality, risks, and glossary. |
| **C-ABI** | The C Application Binary Interface — the stable, platform-defined calling convention used
for cross-language function calls. The Rust Core exposes functions via `extern "C"` for Flutter's
`dart:ffi` to consume. |
| **cargo-llvm-cov** | A Rust tool that uses LLVM's source-based code coverage to measure which
lines and branches are exercised by tests. Used in the SpecKit verification pipeline. |
| **Conventional Commits** | A commit message format convention (`type(scope): description`) that
enables automated changelog generation and semantic versioning. Enforced in CI for this project. |
| **Dart FFI** | Flutter's foreign function interface (`dart:ffi`) for calling native C libraries from
Dart code. The consumption point for the Rust Core's C-ABI exports. |
| **E2EE** | End-to-End Encryption. In the Matrix context, implemented via the Olm (one-to-one) and
Megolm (group) ratchet protocols. Ensures only intended recipients can decrypt messages. |
| **FFI** | Foreign Function Interface — the mechanism by which code written in one language calls
functions written in another. In this project: Rust (producer) ↔ Dart/Flutter (consumer) via C-ABI. |
| **Gitleaks** | A static analysis tool that scans git repositories for secrets (API keys, tokens,
credentials). Runs as a CI gate to prevent accidental secret exposure. |
| **Homeserver** | A Matrix server that stores user accounts, room state, and message history. Users
provide their own homeserver URL (e.g., matrix.org, self-hosted Synapse, Conduit). |
| **Karpathy Protocol** | A lean coding philosophy (named after Andrej Karpathy) emphasizing radical
encapsulation, surgical edits, and zero abstraction excess. Maximizes context locality and minimizes
speculative complexity. |
| **Matrix** | An open, decentralized communication protocol for real-time messaging, VoIP, and IoT.
Defines the Client-Server API, Server-Server API, and E2EE primitives used by this project. |
| **matrix-rust-sdk** | The official Rust SDK for the Matrix protocol, maintained by the Matrix.org
Foundation. Provides client, sync, E2EE, room state, and media operations. |
| **Megolm** | The Matrix group messaging encryption ratchet. Extends Olm's double-ratchet for
efficient group communication where each sender maintains an outbound session shared with room
members. |
| **MSC** | Matrix Spec Change — a proposal process for evolving the Matrix protocol specification.
Features not yet in the stable spec are tracked as MSCs (e.g., location events). |
| **Olm** | The Matrix one-to-one encryption protocol, an implementation of the Double Ratchet
Algorithm. Provides forward secrecy for direct messages and device-to-device key exchange. |
| **Semantic Versioning (SemVer)** | A versioning scheme (`MAJOR.MINOR.PATCH`) where MAJOR changes
indicate breaking API changes. The FFI surface is the public API for semver purposes. |
| **SpecKit** | A behavioral specification framework used in this project. Components are defined by
formal input bounds, edge scenarios, and exit criteria, then verified through automated test
assertions. |
| **sqlx / SQLite** | matrix-rust-sdk uses SQLite for local persistence of sync state, room data,
and E2EE sessions. The Rust Core interacts with this store indirectly through the SDK. |
| **VitePress** | A static site generator built on Vite and Vue, used to compile the arc42
documentation and Mermaid diagrams into a searchable documentation site deployed to GitHub Pages. |
| **Synapse / Dendrite / Conduit** | Popular Matrix homeserver implementations. Synapse (Python,
reference), Dendrite (Go, efficient), Conduit (Rust, lightweight). The Rust Core is homeserver-
agnostic and works with any spec-compliant homeserver. |

## Abbreviations

| Abbreviation | Expansion |
|-------------|-----------|
| **ABI** | Application Binary Interface |
| **ADR** | Architecture Decision Record |
| **API** | Application Programming Interface |
| **APK** | Android Package |
| **ASan** | Address Sanitizer |
| **CDN** | Content Delivery Network |
| **CI** | Continuous Integration |
| **CRUD** | Create, Read, Update, Delete |
| **CVE** | Common Vulnerabilities and Exposures |
| **IPA** | iOS App Store Package |
| **NDK** | Native Development Kit (Android) |
| **OSS** | Open Source Software |
| **PBF** | Protocolbuffer Binary Format (MapLibre tiles) |
| **SAS** | Short Authentication String (E2EE verification) |
| **SDK** | Software Development Kit |
| **UBSan** | Undefined Behavior Sanitizer |
| **URI** | Uniform Resource Identifier |
| **WSS** | WebSocket Secure |
