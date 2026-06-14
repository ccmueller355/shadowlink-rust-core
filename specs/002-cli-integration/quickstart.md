# Quickstart — ShadowLink CLI

**Feature**: 002-cli-integration | **Date**: 2026-06-14

## Prerequisites

- Rust 1.75+
- A running Matrix homeserver (Synapse, Dendrite, or managed provider)
- Matrix user account on that homeserver

## Build

```bash
git clone https://github.com/ccmueller355/shadowlink-cli
cd shadowlink-cli
cargo build --release
```

## Local Development (against uncommitted core changes)

To test against a local checkout of `shadowlink-rust-core`:

```bash
# 1. Add a [patch] section to CLI's Cargo.toml:
# [patch."https://github.com/ccmueller355/shadowlink-rust-core"]
# shadowlink-rust-core = { path = "../shadowlink-rust-core" }

# 2. Build with local core:
cargo build

# 3. Remove [patch] before committing.
```

## Usage

All commands operate relative to the current working directory. Session data is stored in `./shadowlink_data/`.

### Connect

```bash
shadowlink connect https://matrix.example.com alice mypassword
# ✅ Connected! Session saved to shadowlink_data/session.json
```

### Room Operations

```bash
# Create a room
shadowlink create-room "Family Chat"
# ✅ Room created: !abc123:example.com

# List joined rooms
shadowlink list-rooms
# [joined]  Family Chat     (!abc123:example.com)

# Invite another user
shadowlink invite !abc123:example.com @bob:example.com
# ✅ Invited @bob:example.com

# Accept an invitation (run as bob)
shadowlink accept !abc123:example.com
# ✅ Joined room

# Leave a room
shadowlink leave !abc123:example.com
# ✅ Left room
```

### Messaging

```bash
# Send a text message
shadowlink send !abc123:example.com "Hello, family!"
# ✅ Sent: $event123

# Get message history
shadowlink get-history !abc123:example.com
# @alice:example.com: Hello, family!
# @bob:example.com: Hi Alice!

# Listen for incoming messages (blocks until Ctrl+C)
shadowlink listen
# @bob:example.com > How's everyone doing?
# ^C
```

### Location Sharing

```bash
shadowlink share-location !abc123:example.com 51.5074 -0.1278
# ✅ Location shared
```

### Disconnect

```bash
shadowlink disconnect
# ✅ Disconnected. Session cleaned up.
```

## Integration Test (Manual)

```bash
# Terminal 1 — Alice
cd /tmp/alice-test
shadowlink connect https://localhost:8008 alice password
shadowlink listen

# Terminal 2 — Bob
cd /tmp/bob-test
shadowlink connect https://localhost:8008 bob password
shadowlink create-room "Test"
# Note the room ID
shadowlink invite <room_id> @alice:localhost

# Terminal 1 — Alice accepts
shadowlink accept <room_id>

# Terminal 2 — Bob sends
shadowlink send <room_id> "Hello from Bob!"

# Terminal 1 — Alice sees
# @bob:localhost > Hello from Bob!
```

## CI

CLI CI runs on every push/PR to `main`:
- `cargo build --release`
- `cargo test`
- `cargo clippy -- -D warnings`
- `cargo fmt -- --check`
- `gitleaks detect`

No integration tests against a live Synapse in CI (yet). Manual testing per above.
