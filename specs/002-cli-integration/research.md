# Research: CLI Integration

**Feature**: 002-cli-integration | **Date**: 2026-06-14

## Decision 1: Repo Strategy — Separate Repository

**Decision**: CLI lives in its own public repository (`shadowlink-cli`), not as a workspace member in `shadowlink-rust-core`.

**Rationale**:
- **Constitution I (Clean Separation)**: The core is a library crate consumed by multiple clients (Flutter via FFI, CLI via direct Rust dependency). Mixing a binary into the core repo creates an ambiguous boundary.
- **CI independence**: CLI CI can run without core CI — it pulls the core from `main` branch via git dep.
- **License clarity**: Core is MIT/Apache 2.0; CLI is also MIT/Apache 2.0. Separate repos keep licensing unambiguous.
- **Release cadence**: CLI can release independently of core.

**Alternatives considered**:
- **Workspace member**: Rejected — would require core repo to carry a binary, violating Clean Separation. Also complicates CI (one repo's tests depend on the other's source).
- **Monorepo with `crates/`**: Rejected — overkill for a two-crate project. Adds Cargo workspace complexity for no benefit.

**Status**: ✅ Implemented. Repo exists at `github.com/ccmueller355/shadowlink-cli`.

---

## Decision 2: Integration Pattern — Git Dependency

**Decision**: CLI depends on core via Cargo git dependency: `shadowlink-rust-core = { git = "https://github.com/ccmueller355/shadowlink-rust-core", branch = "main" }`.

**Rationale**:
- **Always current**: Pulls latest `main` on `cargo update`, ensuring CLI tests against the latest core.
- **No publish overhead**: Core is not published to crates.io yet (pre-1.0).
- **CI-friendly**: GitHub Actions fetches the dep via HTTPS — no SSH keys or auth required since both repos are public.
- **Path dep fallback**: Local development can override with `[patch]` section in `Cargo.toml` for testing against uncommitted core changes.

**Alternatives considered**:
- **Path dependency** (`path = "../shadowlink-rust-core"`): Rejected — requires sibling directory layout on disk. Breaks CI (GitHub Actions doesn't clone sibling repos automatically). OK for local dev via `[patch]`.
- **crates.io**: Rejected — core is pre-1.0, not yet published. Future option when core stabilizes at 1.0.

**Status**: ✅ Implemented. `Cargo.toml` uses git dep.

---

## Decision 3: CI Cross-Repo Strategy

**Decision**: CLI CI is self-contained — no cross-repo orchestration. Core CI is the gate for core changes.

**Rationale**:
- CLI CI tests against core `main` branch (via git dep). If core `main` is broken, CLI CI fails — this is the desired behavior.
- No integration tests requiring a live Synapse in CLI CI (yet). All CLI tests are unit tests that mock the core interface.
- Core CI runs full integration tests against a local Synapse Docker container — that's where E2EE correctness is validated.

**Gap identified**: CLI has NO integration tests against a real Synapse. The spec's acceptance scenarios (connect, create room, send message, receive) are manual-only. Automated E2E tests for CLI would require a Synapse service in CLI CI.

**Future consideration**: Add a Synapse service to CLI CI for automated integration tests (post-MVP).

**Alternatives considered**:
- **Cross-repo CI trigger**: Rejected — adds complexity (webhook management, token sharing) without proportional benefit. Simpler: if core `main` breaks CLI, fix core first.

**Status**: ✅ Current approach sufficient for MVP. Integration test gap documented as future work.

---

## Decision 4: Session Persistence Path Convention

**Decision**: CLI uses `shadowlink_data/session.json` and `shadowlink_data/store/` relative to CWD — matching core's default path convention.

**Rationale**:
- **No config needed**: User runs commands from any directory; session data lands in `./shadowlink_data/`.
- **Core compatibility**: Core's `client::connect()` writes to CWD-relative paths by default. CLI's `restore_session()` reads from the same location.
- **Disconnect cleanup**: `cmd_disconnect()` removes both `session.json` and the entire `shadowlink_data/store/` directory.

**Trade-off**: Session data is per-directory, not per-user. Running from different directories creates separate sessions. This is intentional — the CLI is a test tool, not a user-facing application.

**Alternatives considered**:
- **XDG_DATA_HOME**: Rejected — over-engineering for a test client. Adds platform-specific path logic.
- **Explicit `--data-dir` flag**: Considered as future enhancement. Would allow sharing sessions across directories.

**Status**: ✅ Implemented.

---

## Decision 5: Error Handling Strategy

**Decision**: CLI matches on `shadowlink_rust_core::ShadowLinkError` and exits non-zero with user-facing messages. No panics, no bare stack traces.

**Rationale**:
- **FR-018**: All error paths must exit non-zero with human-readable messages.
- **Core already provides structured errors**: `ShadowLinkError` enum covers all failure modes (ConnectionFailed, AuthenticationFailed, RoomNotFound, etc.).
- **CLI is thin**: It translates core errors to exit messages — no error logic of its own.

**Status**: ✅ Implemented. All command functions use `match` on core results with `eprintln!` + `std::process::exit(1)` on error.

---

## Decision 6: CLI Structure — Single File

**Decision**: The entire CLI lives in `src/main.rs` (382 lines). No module split.

**Rationale**:
- **Karpathy Protocol (Street-Lean Coding)**: Radical encapsulation. A 382-line CLI doesn't need a module tree.
- **All commands are thin wrappers**: Each command is ~20 lines — connect to core, call one function, print result. Splitting into files would add boilerplate without improving readability.
- **Future refactor boundary**: If CLI grows beyond ~500 lines or gains shared logic (e.g., middleware, config management), split into `commands/` and `cli.rs`.

**Status**: ✅ Implemented. Single-file is appropriate for current scope.
