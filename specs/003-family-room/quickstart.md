# Quickstart: Family Room Semantics

**Feature**: 003-family-room | **Date**: 2026-06-15

## Prerequisites

- Rust 1.75+
- Docker (for local Synapse integration tests)
- `cargo build` passes from the repo root
- Local Synapse running on `localhost:8008` (see [README](../../README.md))

## Quick Verification (Local Synapse)

### 1. Start Synapse

```bash
docker compose up -d
# Wait for Synapse to be ready
curl -s http://localhost:8008/_matrix/client/versions | head -c 100
```

### 2. Build

```bash
cargo build
```

### 3. Run family room tests

```bash
# Run all tests (existing + new family room tests)
cargo test

# Run only family room tests
cargo test family_room
```

### 4. Manual smoke test via the debug binary

If a `shadowlink-cli` binary or test binary is available:

```bash
# Connect as operator
shadowlink connect http://localhost:8008 alice pass123

# Create the family room
shadowlink create-family-room "The Smith Family"
# Expected output: Room created: !abc123:localhost (alias: #the-smith-family:localhost)

# Verify it's pinned
shadowlink get-home-room
# Expected output: Home room: !abc123:localhost (The Smith Family)

# List rooms — home room marked
shadowlink list-rooms
# Expected output shows is_home: true on one room

# Pin an existing room instead
shadowlink set-home-room !xyz789:localhost
# Expected output: Home room set: !xyz789:localhost

# Enable debug room
shadowlink enable-debug-room true
# Expected output: Debug room enabled: !debug456:localhost

# Trigger a diagnostic (send to nonexistent room)
shadowlink send !nonexistent:localhost "test"
# Check the "ShadowLink Debug" room for the error event
```

### 5. Two-session test (operator + family member)

```bash
# Terminal 1: Operator
shadowlink connect http://localhost:8008 alice pass123
shadowlink create-family-room "Family HQ"
# Note the room ID: !room123:localhost

# Terminal 2: Family member
shadowlink connect http://localhost:8008 bob pass456
# Bob has no home room yet
shadowlink get-home-room
# Expected: No home room configured.

# Operator invites Bob (Terminal 1)
shadowlink invite !room123:localhost @bob:localhost

# Bob accepts (Terminal 2)
shadowlink accept !room123:localhost
shadowlink set-home-room !room123:localhost
shadowlink get-home-room
# Expected: Home room: !room123:localhost (Family HQ)
```

## Real Homeserver Smoke Test

> **WARNING**: Follow the Anti-Ban Protocol from `spec.md` § Real-World Homeserver Verification.
> Never run automated tests against `matrix.org`. Manual one-shot tests only.

### Prerequisites

1. Create a test account on `matrix.org` via https://app.element.io
   - Username: `shadowlink-test-familyop`
   - Save the password

2. Set the environment:
   ```bash
   export SHADOWLINK_HOMESERVER=https://matrix-client.matrix.org
   export SHADOWLINK_USER=shadowlink-test-familyop
   export SHADOWLINK_PASSWORD=<your-password>
   ```

### Manual Verification Steps

```bash
# 1. Connect (TLS verified)
shadowlink connect $SHADOWLINK_HOMESERVER $SHADOWLINK_USER $SHADOWLINK_PASSWORD

# 2. Create family room — wait 30s between creation calls
shadowlink create-family-room "ShadowLink Test Family"
# Verify: room created, alias set, is_home: true

# 3. Verify persistence
shadowlink disconnect
sleep 5
shadowlink restore-session
shadowlink get-home-room
# Expected: same room, is_home: true

# 4. Enable debug room
shadowlink enable-debug-room true

# 5. Clean up
shadowlink leave-room <room_id>
shadowlink disconnect
```

## API Usage (Rust)

```rust
use shadowlink_rust_core::client;
use shadowlink_rust_core::rooms;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Connect
    let handle = client::connect("http://localhost:8008", "alice", "pass123").await?;

    // Create family room
    let room = rooms::create_family_room(&handle, "The Smith Family").await?;
    assert!(room.is_home);
    assert_eq!(room.alias, Some("#the-smith-family:localhost".to_string()));

    // Retrieve it after restart
    let home = rooms::get_home_room(&handle).await?;
    assert_eq!(home.unwrap().room_id, room.room_id);

    // Pin an existing room
    let existing = rooms::create_room(&handle, "General Chat").await?;
    let pinned = rooms::set_home_room(&handle, &existing.room_id).await?;
    assert!(pinned.is_home);
    // Old family room is no longer home
    let home = rooms::get_home_room(&handle).await?;
    assert_eq!(home.unwrap().room_id, existing.room_id);

    // Enable debug room
    client::enable_debug_room(&handle, true).await?;

    // Disconnect
    client::disconnect(handle).await?;
    Ok(())
}
```
