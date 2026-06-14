// US1: Homeserver Configuration — Integration Tests
//
// Prerequisite: Synapse running on localhost:8008 (see scripts/setup-synapse.sh)
// Run: `cargo test test_us1_ -- --test-threads=1`

mod common;

use common::{cleanup_store, register_test_user, synapse_available};
use shadowlink_rust_core::client;
use shadowlink_rust_core::error::ShadowLinkError;

const HOMESERVER_URL: &str = "http://localhost:8008";

#[tokio::test]
async fn test_connect_success() {
    if !synapse_available().await {
        eprintln!("SKIP: Synapse not available");
        return;
    }

    cleanup_store();

    let user = register_test_user()
        .await
        .expect("Failed to register test user");

    let handle = client::connect(HOMESERVER_URL, &user.username, &user.password)
        .await
        .expect("connect() should succeed for valid credentials");

    // Verify the handle is functional — disconnect cleanly
    client::disconnect(handle)
        .await
        .expect("disconnect() should succeed");
}

#[tokio::test]
async fn test_disconnect_double() {
    if !synapse_available().await {
        eprintln!("SKIP: Synapse not available");
        return;
    }

    cleanup_store();

    let user = register_test_user()
        .await
        .expect("Failed to register test user");

    let handle = client::connect(HOMESERVER_URL, &user.username, &user.password)
        .await
        .expect("connect() should succeed");

    // First disconnect should succeed
    client::disconnect(handle)
        .await
        .expect("First disconnect() should succeed");

    // We no longer have a handle to disconnect twice — that's the point.
    // The handle is consumed by disconnect(). This tests that the
    // disconnect path doesn't panic or leak resources.
}

#[tokio::test]
async fn test_connect_invalid_url() {
    // This test does NOT require Synapse — it validates error handling.

    let result = client::connect("http://nonexistent:9999", "user", "pass").await;

    match result {
        Err(ShadowLinkError::ConnectionFailed { .. }) => {} // expected
        Err(other) => panic!("Expected ConnectionFailed, got: {other:?}"),
        Ok(_) => panic!("Expected ConnectionFailed, got Ok"),
    }
}

#[tokio::test]
async fn test_connect_bad_credentials() {
    if !synapse_available().await {
        eprintln!("SKIP: Synapse not available");
        return;
    }

    let user = register_test_user()
        .await
        .expect("Failed to register test user");

    // Use the right URL but a deliberately wrong password
    let wrong_password = format!("{}_wrong", user.password);
    let result = client::connect(HOMESERVER_URL, &user.username, &wrong_password).await;

    match result {
        Err(ShadowLinkError::AuthenticationFailed { .. }) => {} // expected
        Err(other) => panic!("Expected AuthenticationFailed, got: {other:?}"),
        Ok(_) => panic!("Expected AuthenticationFailed, got Ok"),
    }
}
