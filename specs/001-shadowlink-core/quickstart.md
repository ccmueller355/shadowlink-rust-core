# Quickstart — ShadowLink Rust Core

Developer setup guide for building, testing, and contributing to the
`shadowlink-rust-core` crate.

## Prerequisites

| Tool | Version | Purpose |
|---|---|---|
| **Rust** | 1.75+ | Compiler + Cargo. Install via [rustup](https://rustup.rs). |
| **Docker** | 24+ | Local Synapse homeserver for integration tests. |
| **cargo-llvm-cov** | latest | Code coverage reports. `cargo install cargo-llvm-cov` |
| **gitleaks** | latest | Secrets scanning. `brew install gitleaks` or `go install github.com/gitleaks/gitleaks/v8@latest` |
| **Node.js** | 18+ | VitePress documentation preview (optional). |

## Build

```bash
# Debug build (fast compile, no optimizations)
cargo build

# Release build (LTO + opt-level=3)
cargo build --release
```

## Test

Local Synapse is required for integration tests. Unit tests run without it.

### Start local Synapse

```bash
docker run -d --name synapse-test \
  -v synapse-data:/data \
  -e SYNAPSE_SERVER_NAME=localhost \
  -e SYNAPSE_REPORT_STATS=no \
  -p 8008:8008 \
  matrixdotorg/synapse:latest

# Wait for Synapse to be ready
until curl -s http://localhost:8008/_matrix/client/versions > /dev/null; do
  echo "Waiting for Synapse..."
  sleep 2
done
echo "Synapse ready."
```

### Run tests

```bash
# Unit tests only (no Docker needed)
cargo test --lib

# All tests including integration (requires Synapse)
cargo test -- --test-threads=1

# Specific test
cargo test test_connect_success
```

### Stop Synapse

```bash
docker stop synapse-test && docker rm synapse-test
```

## Lint & Format

```bash
# Clippy (strict — warnings are errors in CI)
cargo clippy -- -D warnings

# Format check
cargo fmt -- --check

# Auto-format
cargo fmt
```

## Coverage

```bash
# Generate HTML coverage report
cargo llvm-cov --all-targets --html

# Text summary (CI)
cargo llvm-cov --all-targets --summary-only

# Open HTML report
open target/llvm-cov/html/index.html
```

Coverage target: **≥80% line coverage** (excluding FFI boilerplate).

## Secrets Scanning

```bash
gitleaks detect --source . --config .gitleaks.toml --verbose
```

Must report zero findings before merge.

## CI Pipeline

GitHub Actions runs on every push and PR:

```bash
# View CI status
gh ci status

# View CI run details
gh ci view
```

Gates (all must pass):
1. **Build** — `cargo build` (debug and release)
2. **Test** — `cargo test` (unit + integration against Synapse service container)
3. **Lint** — `cargo clippy -- -D warnings`
4. **Format** — `cargo fmt -- --check`
5. **Coverage** — `cargo llvm-cov --all-targets --summary-only` (≥80%)
6. **Secrets** — `gitleaks detect`

## Documentation

```bash
# VitePress dev server (arc42 architecture docs)
npm run docs:dev

# Build static site
npm run docs:build
```

Arc42 documentation lives in `docs/arc42/`. SpecKit specs and plans in
`specs/001-shadowlink-core/`.

## Conventional Commits

All commits must follow [Conventional Commits](https://www.conventionalcommits.org/):

```
feat(client): add homeserver connection with auto-discovery
fix(error): map SDK HTTP timeout to ConnectionFailed
docs(arc42): update building block view with session handle pattern
test(rooms): add two-session invite acceptance test
```

Allowed scopes: `client`, `rooms`, `messaging`, `location`, `ffi`, `error`,
`ci`, `docs`, `specs`.

## Project Structure

```
shadowlink-rust-core/
├── src/
│   ├── lib.rs          # Crate root
│   ├── client.rs       # US1 — Session lifecycle
│   ├── rooms.rs        # US2 — Room operations
│   ├── messaging.rs    # US3 — E2EE messaging & media
│   ├── location.rs     # US4 — Location sharing
│   ├── error.rs        # ShadowLinkError enum
│   └── ffi.rs          # FFI boundary (flutter_rust_bridge)
├── specs/
│   └── 001-shadowlink-core/
│       ├── spec.md              # Feature specification
│       ├── plan.md              # Implementation plan (this phase)
│       ├── research.md          # Architecture Decision Records
│       ├── data-model.md        # Rust type definitions
│       ├── quickstart.md        # This file
│       └── contracts/
│           └── ffi-contract.md  # FFI function signatures
├── docs/
│   └── arc42/                   # arc42 architecture documentation
├── Cargo.toml
├── README.md
└── LICENSE
```
