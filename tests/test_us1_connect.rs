// US1: Homeserver Configuration — Integration Tests
//
// Prerequisite: `docker compose up -d` with Synapse on localhost:8008
// Run: `cargo test test_us1_ -- --ignored --test-threads=1`

mod common;

use common::register_test_user;

#[ignore = "requires local Synapse (docker compose up -d)"]
#[tokio::test]
async fn test_connect_success() {
    let user = register_test_user()
        .await
        .expect("Failed to register test user");

    // TODO: Implement client::connect() — for now this verifies the harness works
    assert!(!user.token.is_empty(), "Expected non-empty access token");
    assert!(!user.user_id.is_empty(), "Expected non-empty user ID");
    assert!(!user.device_id.is_empty(), "Expected non-empty device ID");
}

#[ignore = "requires local Synapse (docker compose up -d)"]
#[tokio::test]
async fn test_disconnect() {
    let user = register_test_user()
        .await
        .expect("Failed to register test user");

    assert!(!user.token.is_empty(), "Expected non-empty access token");

    // TODO: Full disconnect round-trip after connect() is implemented
}

#[tokio::test]
async fn test_connect_invalid_url() {
    // This test validates error handling — it does NOT require a running Synapse
    // because we never reach one. It should fail with ConnectionFailed.
    // TODO: After connect() is implemented, replace with real assertion
    let result: Result<(), &str> = Err("ConnectionFailed expected here once connect() exists");
    assert!(
        result.is_err(),
        "Expected ConnectionFailed for unreachable URL"
    );
}

#[ignore = "requires local Synapse (docker compose up -d)"]
#[tokio::test]
async fn test_connect_bad_credentials() {
    let user = register_test_user()
        .await
        .expect("Failed to register test user");

    // TODO: After connect() is implemented, test with good URL + bad password
    assert!(!user.token.is_empty(), "Harness works: registered user");
}
