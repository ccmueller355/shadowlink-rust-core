# ShadowLink Rust Core Constitution

The ShadowLink Rust Core is the public, open-source Matrix protocol bridge library
that powers the ShadowLink family communications application. This constitution
defines the non-negotiable architectural principles and quality gates governing
all contributions to this crate.

## Core Principles

### I. Clean Separation

The Rust core is an independent library crate — it has zero knowledge of Flutter,
UI, or platform-specific concerns. It is consumed via FFI by the proprietary
ShadowLink Flutter app. No UI code, no widget logic, no platform channels shall
ever live in this crate.

* Crate exposes a minimal, stable public API surface
* FFI boundary is the only consumer contract — internal refactors must not break it
* All platform-specific logic lives in the consuming app, not here

### II. Local-First Privacy

ShadowLink's value proposition is privacy without cloud dependency. The only
external infrastructure is the user's own Matrix homeserver.

* No hardcoded server URLs, no telemetry, no analytics
* All data (messages, media, location) flows through the user-provided homeserver
* Matrix-rust-sdk built-in persistence is the sole storage layer
* No third-party cloud dependencies — no Firebase, no AWS, no CDN

### III. Minimal API Surface

Expose only what the Flutter app needs. Internal complexity — async runtimes,
SDK session management, E2EE key handling — is hidden behind clean, versioned
FFI entry points.

* Every public function must justify its existence in the FFI contract
* Prefer coarse-grained, high-value functions over many tiny helpers
* Internal modules may be refactored freely as long as FFI contract is maintained
* Breaking FFI changes require a MAJOR version bump (per SemVer)

### IV. Test-First (NON-NEGOTIABLE)

SpecKit contracts precede implementation. No code is written without a failing
SpecKit behavioral specification defining its input bounds, edge cases, and
exit criteria.

* SpecKit spec → test assertions → implementation (Red-Green-Refactor)
* Every user story must be independently testable against a local Synapse homeserver
* Integration tests for all FFI entry points
* `cargo test` must pass with zero ignored/skipped tests before merge

### V. Battery & Permission Discipline

The consuming app runs on mobile devices. This crate must not contain patterns
that cause excessive wake locks, background CPU churn, or unnecessary network
requests.

* Async operations must yield when idle — no busy-wait loops
* Background sync must be batched and throttled
* No hardcoded polling intervals; all timers configurable from the Flutter layer
* Location services must respect OS-level permission models (implemented in Flutter,
  but the Rust API must support permission-aware call patterns)

### VI. CI Pipeline Discipline

All quality gates must pass before merge. No exceptions, no bypass commits.

* `cargo build --release` — clean compilation
* `cargo test` — all tests green, zero skipped/ignored
* `cargo llvm-cov --all-targets` — coverage report generated, no regression
* `cargo clippy -- -D warnings` — zero warnings
* `cargo fmt -- --check` — formatting clean
* `gitleaks detect` — no secrets in source

## Security Requirements

E2EE is the core value prop. The Rust crate owns Matrix's Olm/Megolm encryption
via matrix-rust-sdk.

* Key material never leaves the SDK's encrypted store
* No key export or plaintext key logging, even in debug builds
* FFI boundary must not expose raw key material
* Session tokens stored exclusively via SDK persistence
* All network traffic routed through the SDK's built-in HTTP client (no raw reqwest)

## Development Workflow

* **Branch strategy**: Feature branches from `main`, merged via PR
* **Commit format**: [Conventional Commits](https://www.conventionalcommits.org/)
  strictly enforced — `feat(scope):`, `fix(scope):`, `chore(scope):`, `docs(scope):`
* **Versioning**: [SemVer](https://semver.org/) tracked in `Cargo.toml`
* **SpecKit pipeline**: constitution → specify → plan → tasks → implement
* **Documentation**: arc42 living docs updated in lockstep with structural changes
* **Review requirements**: At least one approving review before merge; CI must be green

## Governance

This constitution supersedes all other development practices for this crate.
Amendments require:

1. Documented proposal with rationale
2. Impact analysis on existing FFI contracts and consuming app
3. Approval by project maintainer
4. Migration plan for any breaking changes

All PRs and code reviews must verify compliance with these principles.
Complexity that violates these principles must be explicitly justified in
the PR description.

**Version**: 1.0.0 | **Ratified**: 2026-06-07 | **Last Amended**: 2026-06-07
