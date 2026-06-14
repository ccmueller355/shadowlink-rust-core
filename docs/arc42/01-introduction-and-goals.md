---
title: "1. Introduction & Goals"
---

# 1. Introduction & Goals

## 1.1 Project Overview

**ShadowLink Rust Core** is the public, open-source Rust library that powers the ShadowLink family
communications application. It functions as a Matrix protocol bridge — wrapping
[matrix-rust-sdk](https://github.com/matrix-org/matrix-rust-sdk) into a clean API surface consumed
by Flutter applications (via FFI) and the ShadowLink CLI (via direct Rust dependency).

The core library handles all Matrix protocol operations: session management, room discovery and
membership, end-to-end encrypted messaging, media sharing, and location event exchange. It exposes
no UI, no network infrastructure, and no map rendering — those belong to the consuming layer.

The library powers **ShadowLink**, a one-time-purchase ($3–8) Play Store application for private
family communications. The Rust crate is deliberately open-source (MIT / Apache 2.0) to enable
security auditing and community contribution; consuming applications handle UI and theming.

**License:** MIT / Apache 2.0 (dual-licensed).

## 1.2 Top-Level Quality Goals

| Priority | Goal | Rationale |
|----------|------|-----------|
| **Q1** | **Privacy** | E2EE by default, local-first architecture. No plaintext secrets on disk. All communication secured through Matrix's Olm/Megolm protocol. The only server involved is the Matrix homeserver the user configures. |
| **Q2** | **Portability** | FFI-ready API surface designed from the ground up for Flutter consumption. C-ABI compatible, async-native, mobile-battery-disciplined. Android-first, iOS-ready. |
| **Q3** | **Correctness** | SpecKit-verified behavioral specifications for every component. Formal input bounds, edge scenario enumeration, and exit criteria converted directly into automated test assertions. |

## 1.3 Stakeholders

| Role | Stakeholder | Involvement |
|------|-------------|-------------|
| **Consumer Devs** | Flutter / CLI developers | Primary API consumers. Require stable FFI contracts (Flutter) or Rust API stability (CLI), clear error semantics, semver discipline. |
| **End Users** | Families using ShadowLink | Indirect beneficiaries. Depend on E2EE correctness, battery efficiency, reliable sync. |
| **OSS Community** | External contributors, auditors | Interested in protocol compliance, security review, code quality. MIT/Apache 2.0 licensing supports broad reuse. |

## 1.4 Business Context

ShadowLink is a one-time-purchase ($3–8) Play Store application targeting families who want
private, E2EE communication without server infrastructure beyond their own Matrix homeserver.
The Rust core is deliberately open-source to enable security auditing and community contribution,
while consuming applications handle UI and theming.
