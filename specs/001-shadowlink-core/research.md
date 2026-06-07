# Architecture Decision Records

> **Status:** Accepted decisions from the SpecKit Plan phase.
> These ADRs inform the implementation plan in `plan.md` and the data model in
> `data-model.md`. Cross-referenced from arc42 Section 9.

---

## ADR-001: FFI Bridge — flutter_rust_bridge v2

**Status:** Accepted
**Date:** 2026-06-07
**Deciders:** Valerie Decker (Core Architect)

### Context

The Rust crate must expose an API surface callable from the proprietary
ShadowLink Flutter app (Dart). We need a bridge that supports async Rust
functions, passes structured data types bidirectionally, and integrates
cleanly with Flutter's widget tree without hand-written C-ABI glue.

### Decision

Use **flutter_rust_bridge v2** as the FFI layer.

- Flutter Favorite with active maintenance and ~4k GitHub stars.
- One-liner code generation (`flutter_rust_bridge_codegen generate`).
- Native async support — Rust `async fn` maps directly to Dart `Future`.
- Zero-copy buffer passing for media payloads (`Vec<u8>`).
- Automatic generation of Dart wrapper classes from Rust structs/enums.

### Alternatives Considered

| Alternative | Verdict |
|---|---|
| **uniffi-rs** (Mozilla) | No Dart bindings. Python/Kotlin/Swift only. Non-starter for Flutter target. |
| **Raw dart:ffi** | Full manual C-ABI marshalling for every struct, enum, and callback. Error-prone `unsafe` blocks at every boundary. High maintenance burden as API grows. Rejected on cost. |
| **jni-rs style JNI bridge** | Wrong platform; JNI is Android-only, doesn't help with Dart. |

### Consequences

- **Positive:** Async Rust functions are first-class. No manual C bindings.
  Dart code generation keeps FFI surface in sync automatically. Zero-copy
  media transfer avoids heap pressure on mobile.
- **Negative:** Adds a code generation step to the build pipeline.
  flutter_rust_bridge v2 has occasional breaking changes between minor
  versions. Pin to a specific version in the consuming Flutter app's
  `pubspec.yaml`.
- **Note:** The public repo (`shadowlink-rust-core`) defines the Rust side
  of the contract; the private app repo runs code generation against it.

---

## ADR-002: Async Runtime — tokio

**Status:** Accepted
**Date:** 2026-06-07
**Deciders:** Valerie Decker (Core Architect)

### Context

The matrix-rust-sdk internally requires `tokio` as its async runtime. We
need to configure it appropriately for a mobile-targeting crate where the
async executor runs inside a Flutter process (not as a standalone server).

### Decision

Use **tokio** with the `rt-multi-thread` feature for local development and
CI testing, and `rt` (single-threaded) for mobile release builds via a
feature flag (`mobile-single-threaded`).

- `matrix-sdk` hard-depends on `tokio` — no realistic alternative.
- `flutter_rust_bridge` v2 supports async Rust natively; tokio tasks
  integrate without extra bridging.
- Single-threaded runtime on mobile conserves battery and avoids thread
  pool overhead when only one sync loop runs.

### Alternatives Considered

| Alternative | Verdict |
|---|---|
| **async-std / smol** | matrix-rust-sdk does not support them. Forking the SDK to swap runtimes is not viable. |
| **Custom thread pool** | Reinventing what tokio already provides. Adds maintenance burden for zero benefit. |

### Consequences

- **Positive:** Direct compatibility with matrix-rust-sdk. Well-tested
  runtime with strong mobile support (Android/iOS).
- **Negative:** tokio is a large dependency (~200+ crates transitive). We
  accept this because matrix-rust-sdk already pulls it in — no net increase.
- **Note:** Single-threaded mobile mode requires the consuming Flutter app
  to not block the Rust thread. All Dart→Rust calls must be non-blocking.

---

## ADR-003: SDK Version — matrix-sdk 0.7+

**Status:** Accepted
**Date:** 2026-06-07
**Deciders:** Valerie Decker (Core Architect)

### Context

We need to select a specific version and feature set of matrix-rust-sdk
that provides: E2EE (Olm/Megolm), SQLite-backed persistence, media upload,
sync loop, and room management. The SDK version determines available APIs
and stability guarantees.

### Decision

Target **matrix-sdk 0.7+** from crates.io with these features:

- `e2e-encryption` — Olm/Megolm support (core requirement).
- `sqlite` — SQLite-backed state store for session persistence.
- `bundled-sqlite` — Bundled libsqlite3 for Android builds (avoids system
  SQLite version fragmentation).
- `qrcode` — QR code generation for device verification (future use).

We follow crates.io releases, not git main. If a critical fix is needed
before a release, we pin a specific git rev with a comment explaining why.

### Alternatives Considered

| Alternative | Verdict |
|---|---|
| **matrix-sdk 0.6.x** | Missing stable sliding sync support and several E2EE fixes. 0.7 is the current stable line. |
| **matrix-sdk git main** | Unstable API surface. Suitable for development but not for a crate consumed by a shipping app. |
| **Custom Matrix client (HTTP + ruma directly)** | Reimplements sync loop, E2EE state machine, room state tracking, Olm session management. Thousands of lines of security-critical code. Rejected outright. |

### Consequences

- **Positive:** Battle-tested SDK from the Matrix.org Foundation.
  E2EE handled by experts. Active maintenance.
- **Negative:** SDK API evolves rapidly. We must track release notes and
  test upgrades in CI before bumping.
- **Note:** The `bundled-sqlite` feature is Android-only but harmless on
  desktop — we enable it unconditionally for simplicity.

---

## ADR-004: Error Model — thiserror + ShadowLinkError

**Status:** Accepted
**Date:** 2026-06-07
**Deciders:** Valerie Decker (Core Architect)

### Context

The crate needs a unified error taxonomy that spans SDK errors, I/O
failures, FFI boundary conditions, and domain-specific failures (e2ee
decryption, location unavailable). Errors must be serializable across the
FFI boundary as structured codes the Flutter layer can act on.

### Decision

Use **thiserror** derive macros on a single `ShadowLinkError` enum.

- Each variant carries context (e.g., `ConnectionFailed { reason: String }`).
- `thiserror` auto-generates `Display` and `Error` impls, avoiding manual
  boilerplate.
- The enum is `#[derive(Debug, Clone)]` for FFI transfer.
- Flutter receives error codes as strings (variant name) with the
  human-readable `Display` message for user-facing alerts.

Error classification taxonomy:
- **Retryable:** `ConnectionFailed`, `DecryptionFailed` (transient key delay)
- **Fatal:** `AuthenticationFailed`, `SessionExpired`, `RoomNotFound`
- **User-correctable:** `MediaTooLarge`, `LocationUnavailable`, `NotInRoom`

### Alternatives Considered

| Alternative | Verdict |
|---|---|
| **anyhow** | Great for applications, not for libraries. Opaque error chains leak implementation details across FFI. |
| **Manual Display/Error impls** | Boilerplate that `thiserror` eliminates. No advantage for a library crate. |
| **Separate error types per module** | Fragments error handling at the FFI boundary. The Flutter layer would need N different catch clauses. Unified enum is cleaner. |

### Consequences

- **Positive:** Single `Result<T, ShadowLinkError>` type across the entire
  FFI surface. Flutter catches one error type. `thiserror` keeps impls DRY.
- **Negative:** The enum will grow with new features. Must ensure exhaustive
  matching in tests so no variant is accidentally ignored.
- **Note:** The existing placeholder `ShadowLinkError::Unimplemented` in
  `src/error.rs` will be replaced with the full enum during implementation.

---

## ADR-005: Media Pipeline — SDK Delegation

**Status:** Accepted
**Date:** 2026-06-07
**Deciders:** Valerie Decker (Core Architect)

### Context

The app must send and receive encrypted media attachments (photos). Matrix
media upload involves: encrypting the file, uploading ciphertext to the
homeserver's media repository, and sending an `m.room.message` event with
the `mxc://` URI and decryption metadata (key, IV, hashes). Receiving
involves the reverse: download ciphertext, decrypt.

### Decision

Delegate the entire media pipeline to the SDK's built-in attachment API:
`matrix_sdk::room::Joined::send_attachment()`.

- The SDK handles encryption, upload, event construction, and thumbnail
  generation (if configured).
- The crate exposes a single FFI function `send_media(handle, room_id, data,
  mime_type, filename) -> Result<String, ShadowLinkError>` where data is the
  raw (cleartext) file bytes.
- Received media is delivered via the message callback with an `mxc://` URI
  and decryption metadata; the Flutter layer constructs the download URL.
- No raw HTTP handling in this crate — no `reqwest`, no manual upload logic.

### Alternatives Considered

| Alternative | Verdict |
|---|---|
| **Manual encrypt + upload via reqwest** | Reimplements Matrix content repo protocol, encryption envelope construction, and thumbnail spec. Duplicates SDK code. Security risk. |
| **Separate media crate** | Over-engineering for a single function call through the SDK. No value add. |

### Consequences

- **Positive:** Zero custom media protocol code. SDK handles encryption
  correctly. One function call on the Rust side.
- **Negative:** We depend on the SDK's attachment API not changing
  significantly between 0.7.x releases. Media upload progress reporting
  (bytes sent) may require SDK feature requests if not already exposed.
- **Note:** Maximum upload size is homeserver-configured. The SDK returns
  an error we map to `ShadowLinkError::MediaTooLarge`.

---

## ADR-006: Location Events — Custom `org.shadowlink.location`

**Status:** Accepted
**Date:** 2026-06-07
**Deciders:** Valerie Decker (Core Architect)

### Context

Family location sharing is a core differentiator for ShadowLink. We need a
Matrix event format for transmitting geographic coordinates, accuracy
radius, and a live/static flag. Two options exist: the proposed MSC3488
(`m.location`) or a custom extensible event type.

### Decision

Use a custom Matrix event type **`org.shadowlink.location`** via ruma's
extensible events system.

Rationale:
- Full control over the event schema — fields we need (live flag, beacon ID,
  accuracy granularity) without waiting for MSC stabilization.
- MSC3488 (`m.location`) is still unstable and may change before acceptance.
- Custom event types are a first-class Matrix extensibility mechanism.
  No homeserver modifications needed — the event is treated as a room
  message with custom content.
- ruma's `ExtensibleEventContent` derive macro makes custom event
  definition a few lines of Rust.

### Alternatives Considered

| Alternative | Verdict |
|---|---|
| **MSC3488 `m.location`** | Unstable MSC. Field set may not cover our needs (live tracking flag). Could adopt later as a secondary format if it stabilizes with feature parity. |
| **Custom state event** | Locations are ephemeral messages, not persistent room state. `m.room.message` with custom content is the correct event kind. |

### Consequences

- **Positive:** Schema matches our exact requirements. No dependency on MSC
  timeline. Easy to extend (battery level, speed, heading in future).
- **Negative:** Non-standard event type won't render in Element or other
  generic Matrix clients. Acceptable — ShadowLink is a custom client by
  design; interoperability with generic clients is not a goal.
- **Note:** We document the event schema publicly so other clients *could*
  implement support if desired.

---

## ADR-007: Logging & Observability (Deferred)

**Status:** Proposed
**Date:** 2026-06-07
**Deciders:** Valerie Decker (Core Architect)

### Context

The crate needs structured logging for debugging and crash diagnostics,
but must never log plaintext message content, key material, or access
tokens (FR-014). The FFI layer needs a way to forward logs to Flutter's
logging system.

### Decision

Deferred to implementation phase. Tentative approach:
- `tracing` crate with structured spans.
- `tracing-subscriber` with a custom layer that filters PII.
- An FFI callback for forwarding `WARN` and `ERROR` events to Flutter
  (e.g., Crashlytics).
- Log levels: `TRACE` (dev, full sync details), `DEBUG` (CI, function
  entry/exit), `INFO` (release, session lifecycle), `WARN` (retryable
  errors), `ERROR` (fatal errors).

### Alternatives Considered

| Alternative | Verdict |
|---|---|
| **log + env_logger** | No structured spans. Harder to filter PII programmatically. tracing is the Rust ecosystem standard for async code. |
| **No logging** | Unacceptable for debugging production issues on mobile devices. |

### Consequences

- **Positive:** tracing integrates with tokio's async span model.
- **Negative:** Adds a dependency and requires PII-filtering layer
  (security-critical code). Deferred to implementation to avoid
  over-engineering before the core API is functional.

---

## ADR-008: Cross-Platform Build Toolchain (Deferred)

**Status:** Proposed
**Date:** 2026-06-07
**Deciders:** Valerie Decker (Core Architect)

### Context

The crate must compile for Android (arm64, x86_64 emulator) and eventually
iOS. Cross-compilation of Rust with native dependencies (OpenSSL for TLS,
SQLite) requires platform-specific toolchains and NDK configuration.

### Decision

Deferred to Phase 6+. Initial development and CI target Linux x86_64 only.
Android build configuration (NDK, cargo-ndk, linker settings) will be
documented in a `BUILD_ANDROID.md` guide. The `bundled-sqlite` feature
eliminates the system SQLite dependency. For TLS, matrix-rust-sdk uses
`rustls` by default (no OpenSSL dependency on mobile).

### Alternatives Considered

| Alternative | Verdict |
|---|---|
| **Pre-build all platforms from day 1** | Premature. The SDK and FFI contract can be validated entirely on Linux. Android-specific issues (JNI, NDK linking) are orthogonal to core logic. |

### Consequences

- **Positive:** Faster development iteration. CI is simple (Linux only).
- **Negative:** Android bugs (e.g., file path differences for SQLite store)
  will surface later. Mitigated by testing on Android emulator in Phase 6.

---

## ADR-009: Debuggability — Debug Room + Structured Logging

**Status:** Accepted
**Date:** 2026-06-07
**Deciders:** Valerie Decker (Core Architect), Operator

### Context

The crate runs on mobile devices with no direct console access. When things
go wrong — connection drops, E2EE key mismatches, sync stalls — the operator
needs to see what happened and where. Debugging via adb logcat or Xcode
console is impractical for a shipped family app. We need an observability
channel that works on production devices, is opt-in for privacy, and
integrates with the Matrix protocol itself.

### Decision

Implement a two-layer debuggability strategy:

**Layer 1: Debug Room (visible to operator)**
A private, invite-only E2EE Matrix room (`"ShadowLink Debug"`) is created
on first `connect()` after explicit user consent. The crate emits structured
diagnostic events as normal Matrix messages into this room:

```
[14:21:03] [SYNC] Full sync completed (142 events, 3 rooms)
            {"event_type":"sync_complete","module":"client","severity":"info",
             "elapsed_ms":1420,"room_count":3}

[14:21:05] [E2EE] Decryption failed for event $abc123
            {"event_type":"decryption_failed","module":"messaging",
             "severity":"error","event_id":"$abc123",
             "error_code":"key_mismatch","retryable":true}
```

- Human-readable prefix for eyeball scanning in any Matrix client
- JSON metadata suffix for automated parsing
- No PII: message content, coordinates, tokens, key material never emitted
- Operator consent required at setup; toggleable at runtime via
  `enable_debug_room()`
- If the debug room is deleted, it is re-created on next `connect()`

**Layer 2: Tracing (on-device, forwarded to Flutter)**
`tracing` crate with structured spans and events. A custom PII-filtering
layer sits between the `tracing` subscriber and the output sink:

| Log level | Enabled in | Content |
|---|---|---|
| `ERROR` | Release | Fatal errors (crash diagnostics) |
| `WARN` | Release | Retryable failures, degraded modes |
| `INFO` | Debug builds | Session lifecycle, sync progress |
| `DEBUG` | Debug builds | Function entry/exit |
| `TRACE` | Dev only | Full sync event dump, raw SDK state |

Logs are forwarded to Flutter via an FFI callback for display in a debug
overlay (Flutter's `debugFillProperties`) or crash reporter (Crashlytics
etc.). The PII filter strips message content, coordinates, tokens, and
key bytes at the `tracing` subscriber level — they never reach the output.

### Alternatives Considered

| Alternative | Verdict |
|---|---|
| **adb logcat / Xcode console only** | Useless on production devices. Operator can't access them. |
| **Third-party crash reporter (Crashlytics, Sentry)** | Adds vendor lock-in and privacy concerns. Data leaves the Matrix ecosystem. Complement, not replace, the debug room. |
| **HTTP endpoint for live logs** | Contradicts no-cloud-infra principle. Requires backend. Debug room uses the existing Matrix connection. |
| **Log file on device** | Hard to retrieve on mobile. No UX for the operator to read it. Debug room is self-serve. |

### Consequences

- **Positive:** The debug room is zero-infrastructure — uses the existing
  Matrix homeserver. Works on production devices. Operator reads it from
  any Matrix client. Consent-based, PII-filtered, toggleable.
- **Positive:** tracing crate integrates with tokio's async spans and
  provides structured diagnostics that can feed both the debug room and
  on-device crash reporters.
- **Negative:** Debug room events consume homeserver storage and bandwidth
  (acceptable — text-only, low volume, opt-in). The PII filter is
  security-critical code that must be reviewed and tested.
- **Negative:** Debug room requires the operator to monitor a separate
  room. Mitigated by the Flutter app showing a debug badge/banner when
  unread events exist in the debug room.
- **Note:** The debug room is NOT a user-facing feature. It is an operator
  diagnostics tool. The Flutter layer should only expose it via a hidden
  gesture (e.g., 5 taps on the version number).
