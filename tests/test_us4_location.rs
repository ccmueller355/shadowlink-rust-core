// US4: Location Sharing — Integration Tests
//
// Prerequisite: `docker compose up -d` with Synapse on localhost:8008
// Run: `cargo test test_us4_ -- --test-threads=1`

mod common;

use common::{cleanup_store, create_ephemeral_user_pair, synapse_available};
use shadowlink_rust_core::client;
use shadowlink_rust_core::error::ShadowLinkError;
use shadowlink_rust_core::location;
use shadowlink_rust_core::rooms;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;

const HOMESERVER_URL: &str = "http://localhost:8008";

/// Helper: register user, connect, create room, return handle + room_id.
async fn setup_connected_user() -> (shadowlink_rust_core::client::SessionHandle, String) {
    let user = common::register_test_user()
        .await
        .expect("Failed to register test user");

    let handle = client::connect(HOMESERVER_URL, &user.username, &user.password)
        .await
        .expect("connect() should succeed");

    let room = rooms::create_room(&handle, "Location Test")
        .await
        .expect("create_room() should succeed");

    (handle, room.room_id)
}

#[tokio::test]
async fn test_send_beacon() {
    common::init_tracing();
    if !synapse_available().await {
        eprintln!("SKIP: Synapse not available");
        return;
    }

    cleanup_store();
    let (user_a, user_b) = create_ephemeral_user_pair()
        .await
        .expect("Failed to register test users");

    // A: connect, create room, invite B
    let handle_a = client::connect(HOMESERVER_URL, &user_a.username, &user_a.password)
        .await
        .expect("A: connect() should succeed");

    let room = rooms::create_room(&handle_a, "Beacon Test")
        .await
        .expect("A: create_room() should succeed");
    let room_id = room.room_id.clone();

    rooms::invite_user(&handle_a, &room_id, &user_b.user_id)
        .await
        .expect("A: invite_user() should succeed");

    client::disconnect(handle_a)
        .await
        .expect("A: disconnect() should succeed");
    cleanup_store();

    // B: connect, accept invite, register location callback
    let handle_b = client::connect(HOMESERVER_URL, &user_b.username, &user_b.password)
        .await
        .expect("B: connect() should succeed");

    // Allow sync to process the pending invite
    tokio::time::sleep(Duration::from_secs(2)).await;

    let rooms_list = rooms::list_rooms(&handle_b)
        .await
        .expect("B: list_rooms() should succeed");

    let invited = rooms_list
        .iter()
        .find(|r| matches!(r.state, shadowlink_rust_core::rooms::RoomState::Invited))
        .expect("B should have an invited room");

    rooms::accept_invite(&handle_b, &invited.room_id)
        .await
        .expect("B: accept_invite() should succeed");

    // Register location callback on B
    let received: Arc<Mutex<Vec<shadowlink_rust_core::location::LocationBeacon>>> =
        Arc::new(Mutex::new(Vec::new()));
    let received_clone = Arc::clone(&received);

    let callback: shadowlink_rust_core::client::LocationCallback = Box::new(move |beacon| {
        let mut beacons = received_clone.lock().expect("Lock should not be poisoned");
        beacons.push(beacon);
    });

    location::register_location_callback(&handle_b, Some(callback));

    client::disconnect(handle_b)
        .await
        .expect("B: disconnect() should succeed");
    cleanup_store();

    // A: reconnect, send a beacon
    let handle_a2 = client::connect(HOMESERVER_URL, &user_a.username, &user_a.password)
        .await
        .expect("A: reconnect() should succeed");

    let event_id = location::send_beacon(&handle_a2, &room_id, 48.8566, 2.3522, Some(10.0))
        .await
        .expect("A: send_beacon() should succeed");

    assert!(!event_id.is_empty(), "Beacon event ID should be non-empty");

    client::disconnect(handle_a2)
        .await
        .expect("A: disconnect() should succeed");
    cleanup_store();

    // B: reconnect, verify beacon arrived via history
    let handle_b2 = client::connect(HOMESERVER_URL, &user_b.username, &user_b.password)
        .await
        .expect("B: reconnect() should succeed");

    tokio::time::sleep(Duration::from_secs(2)).await;

    // Verify the callback fired (synced from initial pending events)
    // or check message history for the location event
    let count = received.lock().expect("Lock should not be poisoned").len();
    if count == 0 {
        // Fallback: try fetching message history
        let history = shadowlink_rust_core::messaging::get_history(&handle_b2, &room_id, 10)
            .await
            .expect("B: get_history() should succeed");

        let found_location = history.iter().any(|m| {
            matches!(&m.content, shadowlink_rust_core::messaging::MessageContent::Location { lat, lng, .. }
                if (*lat - 48.8566).abs() < 0.001 && (*lng - 2.3522).abs() < 0.001)
        });
        assert!(
            found_location,
            "B should find the location beacon in message history"
        );
    }

    client::disconnect(handle_b2)
        .await
        .expect("B: disconnect() should succeed");
}

#[tokio::test]
async fn test_location_unavailable() {
    common::init_tracing();
    if !synapse_available().await {
        eprintln!("SKIP: Synapse not available");
        return;
    }

    cleanup_store();
    let (handle, room_id) = setup_connected_user().await;

    // Invalid latitude (out of range)
    let result = location::send_beacon(&handle, &room_id, 100.0, 0.0, None).await;
    assert!(
        matches!(result, Err(ShadowLinkError::LocationUnavailable)),
        "Invalid latitude should return LocationUnavailable, got: {:?}",
        result
    );

    // Invalid longitude (out of range)
    let result = location::send_beacon(&handle, &room_id, 0.0, 200.0, None).await;
    assert!(
        matches!(result, Err(ShadowLinkError::LocationUnavailable)),
        "Invalid longitude should return LocationUnavailable, got: {:?}",
        result
    );

    // Valid coordinates should succeed
    let result = location::send_beacon(&handle, &room_id, 48.8566, 2.3522, None).await;
    assert!(result.is_ok(), "Valid coords should succeed, got: {:?}", result);

    client::disconnect(handle)
        .await
        .expect("disconnect() should succeed");
}
