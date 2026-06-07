// ShadowLink Rust Core — Integration tests
//
// All integration tests run against a local Synapse homeserver
// started via `docker compose up`. See docker-compose.yml at repo root.
//
// Test flow:
//   1. Docker: `docker compose up -d` (runs Synapse on :8008)
//   2. Admin API: register ephemeral test users
//   3. Run tests: `cargo test --test-threads=1`
//   4. Cleanup: `docker compose down`
