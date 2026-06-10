// US3: E2EE Messaging & Media — Integration Tests
//
// Prerequisite: `docker compose up -d` with Synapse on localhost:8008
// Run: `cargo test test_us3_ -- --test-threads=1`

mod common;

use common::{cleanup_store, create_ephemeral_user_pair, synapse_available};
use shadowlink_rust_core::client;
use shadowlink_rust_core::messaging::{self, MessageCallback, MessageContent};
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;
use shadowlink_rust_core::rooms;

const HOMESERVER_URL: &str = "http://localhost:8008";

/// Helper: register two users, connect A, create room, invite B, disconnect A,
/// connect B, accept invite, disconnect B. Returns (room_id, user_a, user_b).
async fn setup_two_users_in_room() -> (String, String, String) {
    let (user_a, user_b) = create_ephemeral_user_pair()
        .await
        .expect("Failed to register test users");

    // A: connect, create room, invite B
    let handle_a = client::connect(HOMESERVER_URL, &user_a.username, &user_a.password)
        .await
        .expect("A: connect() should succeed");

    let room = rooms::create_room(&handle_a, "Messaging Test")
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

    // B: connect, accept invite
    let handle_b = client::connect(HOMESERVER_URL, &user_b.username, &user_b.password)
        .await
        .expect("B: connect() should succeed");

    // Allow sync to process the pending invite
    tokio::time::sleep(std::time::Duration::from_secs(2)).await;

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

    client::disconnect(handle_b)
        .await
        .expect("B: disconnect() should succeed");
    cleanup_store();

    (room_id, user_a.username, user_b.username)
}

#[tokio::test]
async fn test_send_receive_text() {
    common::init_tracing();
    if !synapse_available().await {
        eprintln!("SKIP: Synapse not available");
        return;
    }

    let (room_id, user_a_name, user_b_name) = setup_two_users_in_room().await;

    // A: connect and send a text message
    let handle_a = client::connect(HOMESERVER_URL, &user_a_name, &user_a_name)
        .await
        .expect("A: connect() should succeed");

    let event_id = messaging::send_text(&handle_a, &room_id, "Hello family!")
        .await
        .expect("A: send_text() should succeed");

    assert!(!event_id.is_empty(), "Event ID should be non-empty");

    client::disconnect(handle_a)
        .await
        .expect("A: disconnect() should succeed");
    cleanup_store();

    // B: connect and fetch history — should contain A's message
    let handle_b = client::connect(HOMESERVER_URL, &user_b_name, &user_b_name)
        .await
        .expect("B: connect() should succeed");

    // Allow sync to pick up the room state
    tokio::time::sleep(Duration::from_secs(2)).await;

    let history = messaging::get_history(&handle_b, &room_id, 10)
        .await
        .expect("B: get_history() should succeed");

    let found = history.iter().any(|m| {
        matches!(&m.content, MessageContent::Text { body } if body == "Hello family!")
    });
    assert!(found, "B should see the text message in history");

    client::disconnect(handle_b)
        .await
        .expect("B: disconnect() should succeed");
}

#[tokio::test]
async fn test_send_receive_media() {
    common::init_tracing();
    if !synapse_available().await {
        eprintln!("SKIP: Synapse not available");
        return;
    }

    let (room_id, user_a_name, user_b_name) = setup_two_users_in_room().await;

    // A: connect and send a tiny media blob
    let handle_a = client::connect(HOMESERVER_URL, &user_a_name, &user_a_name)
        .await
        .expect("A: connect() should succeed");

    let image_data: Vec<u8> = vec![0xFF, 0xD8, 0xFF, 0xE0]; // minimal JPEG header
    let event_id = messaging::send_media(&handle_a, &room_id, image_data, "image/jpeg", "test.jpg")
        .await
        .expect("A: send_media() should succeed");

    assert!(!event_id.is_empty(), "Media event ID should be non-empty");

    client::disconnect(handle_a)
        .await
        .expect("A: disconnect() should succeed");
    cleanup_store();

    // B: connect and fetch history — should contain the media message
    let handle_b = client::connect(HOMESERVER_URL, &user_b_name, &user_b_name)
        .await
        .expect("B: connect() should succeed");

    tokio::time::sleep(Duration::from_secs(2)).await;

    let history = messaging::get_history(&handle_b, &room_id, 10)
        .await
        .expect("B: get_history() should succeed");

    let found = history.iter().any(|m| {
        matches!(&m.content, MessageContent::Media { filename, .. } if filename == "test.jpg")
    });
    assert!(found, "B should see the media message in history");

    client::disconnect(handle_b)
        .await
        .expect("B: disconnect() should succeed");
}

#[tokio::test]
async fn test_message_history() {
    common::init_tracing();
    if !synapse_available().await {
        eprintln!("SKIP: Synapse not available");
        return;
    }

    let (room_id, user_a_name, user_b_name) = setup_two_users_in_room().await;

    // A: connect and send 5 messages
    let handle_a = client::connect(HOMESERVER_URL, &user_a_name, &user_a_name)
        .await
        .expect("A: connect() should succeed");

    let mut sent_ids = Vec::new();
    for i in 0..5 {
        let body = format!("Message {}", i);
        let eid = messaging::send_text(&handle_a, &room_id, &body)
            .await
            .expect("A: send_text() should succeed");
        sent_ids.push(eid);
    }

    client::disconnect(handle_a)
        .await
        .expect("A: disconnect() should succeed");
    cleanup_store();

    // B: connect and fetch history with limit=3
    let handle_b = client::connect(HOMESERVER_URL, &user_b_name, &user_b_name)
        .await
        .expect("B: connect() should succeed");

    tokio::time::sleep(Duration::from_secs(2)).await;

    let history = messaging::get_history(&handle_b, &room_id, 3)
        .await
        .expect("B: get_history() should succeed");

    // Should get at most 3 messages (newest first)
    assert!(history.len() <= 3, "History limit should cap at 3, got {}", history.len());

    // The newest messages should be "Message 4", "Message 3", "Message 2"
    let bodies: Vec<&str> = history
        .iter()
        .filter_map(|m| match &m.content {
            MessageContent::Text { body } => Some(body.as_str()),
            _ => None,
        })
        .collect();

    if !bodies.is_empty() {
        assert_eq!(bodies[0], "Message 4", "Newest message should be first");
    }

    client::disconnect(handle_b)
        .await
        .expect("B: disconnect() should succeed");
}

#[tokio::test]
async fn test_callback_registration() {
    common::init_tracing();
    if !synapse_available().await {
        eprintln!("SKIP: Synapse not available");
        return;
    }

    cleanup_store();

    let (user_a, _user_b) = create_ephemeral_user_pair()
        .await
        .expect("Failed to register test users");

    let handle_a = client::connect(HOMESERVER_URL, &user_a.username, &user_a.password)
        .await
        .expect("A: connect() should succeed");

    // Register a callback that collects received messages
    let received: Arc<Mutex<Vec<shadowlink_rust_core::messaging::Message>>> =
        Arc::new(Mutex::new(Vec::new()));
    let received_clone = Arc::clone(&received);

    let callback: MessageCallback = Arc::new(move |msg| {
        let mut msgs = received_clone.lock().expect("Lock should not be poisoned");
        msgs.push(msg);
    });

    messaging::register_message_callback(&handle_a, Some(callback));

    // Create a room and send a message — the sync loop should echo it back
    let room = rooms::create_room(&handle_a, "Callback Test")
        .await
        .expect("create_room() should succeed");

    messaging::send_text(&handle_a, &room.room_id, "Callback test message")
        .await
        .expect("send_text() should succeed");

    // Wait for sync to process the echo
    tokio::time::sleep(Duration::from_secs(3)).await;

    // Check if the callback fired
    let count = received.lock().expect("Lock should not be poisoned").len();
    assert!(
        count >= 1,
        "Callback should have fired at least once, got {} messages",
        count
    );

    // Unregister
    messaging::register_message_callback(&handle_a, None);

    client::disconnect(handle_a)
        .await
        .expect("disconnect() should succeed");
}
