// Synapse test helpers — ephemeral user registration for integration tests.
//
// Uses the Synapse admin API at `/_synapse/admin/v1/register`.
// Requires `docker compose up` running on `localhost:8008`.
//
// Fields, functions, and imports in this module are unused when running
// `cargo test --lib` (unit tests only). dead_code lint is expected.
#![allow(dead_code, unused_imports)]

use reqwest::Client as HttpClient;
use serde_json::{Value, json};
use std::sync::atomic::{AtomicU64, Ordering};

static USER_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Returns `true` if a Synapse homeserver is reachable at localhost:8008.
/// Integration tests should call this first and `skip` if unavailable.
pub async fn synapse_available() -> bool {
    let http = HttpClient::new();
    http.get("http://localhost:8008/_matrix/client/versions")
        .send()
        .await
        .map(|r| r.status().is_success())
        .unwrap_or(false)
}

/// A disposable test user with a pre-authenticated Matrix session token.
pub struct TestUser {
    pub user_id: String,
    pub password: String,
    pub token: String,
    pub device_id: String,
}

/// Register an ephemeral user via the Synapse admin API.
pub async fn register_test_user() -> Result<TestUser, String> {
    let counter = USER_COUNTER.fetch_add(1, Ordering::SeqCst);
    let username = format!("testuser_{}", counter);
    let password = format!("password_{}", counter);
    let http = HttpClient::new();

    // Step 1: Get nonce
    let resp = http
        .get("http://localhost:8008/_synapse/admin/v1/register")
        .send()
        .await
        .map_err(|e| format!("Failed to get nonce: {}", e))?;

    let nonce: Value = resp
        .json()
        .await
        .map_err(|e| format!("Failed to parse nonce: {}", e))?;
    let nonce_str = nonce["nonce"]
        .as_str()
        .ok_or_else(|| "Missing nonce in response".to_string())?
        .to_string();

    // Step 2: Compute MAC = hmac_sha256(secret, nonce || username || password)
    let mac = hmac_sha256(
        "shadowlink-test-secret",
        &format!("{}{}{}", nonce_str, username, password),
    );

    // Step 3: Register
    let body = json!({
        "nonce": nonce_str,
        "username": username,
        "password": password,
        "admin": false,
        "mac": mac,
    });

    let resp = http
        .post("http://localhost:8008/_synapse/admin/v1/register")
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("Registration request failed: {}", e))?;

    let status = resp.status();
    let data: Value = resp
        .json()
        .await
        .map_err(|e| format!("Failed to parse registration response: {}", e))?;

    if !status.is_success() {
        return Err(format!(
            "Registration failed ({}): {}",
            status,
            data.get("error")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown")
        ));
    }

    let token = data["access_token"]
        .as_str()
        .ok_or_else(|| "Missing access_token".to_string())?
        .to_string();
    let user_id = data["user_id"]
        .as_str()
        .ok_or_else(|| "Missing user_id".to_string())?
        .to_string();
    let device_id = data["device_id"]
        .as_str()
        .ok_or_else(|| "Missing device_id".to_string())?
        .to_string();

    Ok(TestUser {
        user_id,
        password,
        token,
        device_id,
    })
}

/// Register two ephemeral users for two-session tests (US2+).
pub async fn create_ephemeral_user_pair() -> Result<(TestUser, TestUser), String> {
    let a = register_test_user().await?;
    let b = register_test_user().await?;
    Ok((a, b))
}

/// Simple HMAC-SHA256 for Synapse admin nonce registration.
fn hmac_sha256(secret: &str, message: &str) -> String {
    use hmac::{Hmac, Mac};
    use sha2::Sha256;

    type HmacSha256 = Hmac<Sha256>;
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).expect("HMAC key length is valid");
    mac.update(message.as_bytes());
    let result = mac.finalize();
    let code = result.into_bytes();
    hex::encode(code)
}
