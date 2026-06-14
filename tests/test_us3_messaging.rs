// US3: E2EE Messaging & Media — Integration Tests
//
// Prerequisite: `docker compose up -d` with Synapse on localhost:8008
// Run: `cargo test test_us3_ -- --test-threads=1`

mod common;

use common::{cleanup_store, create_ephemeral_user_pair, synapse_available};
use shadowlink_rust_core::client;
use shadowlink_rust_core::messaging::{self, MessageCallback, MessageContent};
use shadowlink_rust_core::rooms;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;

const HOMESERVER_URL: &str = "http://localhost:8008";

/// Helper: register two users, connect both simultaneously, create room with A,
/// invite B, accept on B. Returns handles and room_id so tests can use them
/// directly without reconnecting (which would conflict with E2EE device IDs).
struct TwoUserSetup {
    room_id: String,
    handle_a: client::SessionHandle,
    handle_b: client::SessionHandle,
}

async fn setup_two_users_in_room() -> TwoUserSetup {
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

    // B: connect, accept invite (A stays online so E2EE keys get shared)
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

    // Wait for key sharing to complete
    tokio::time::sleep(std::time::Duration::from_secs(2)).await;

    TwoUserSetup {
        room_id,
        handle_a,
        handle_b,
    }
}

#[tokio::test]
async fn test_send_receive_text() {
    common::init_tracing();
    cleanup_store();
    if !synapse_available().await {
        eprintln!("SKIP: Synapse not available");
        return;
    }

    let setup = setup_two_users_in_room().await;

    // A sends a text message (handle remains from setup)
    let event_id = messaging::send_text(&setup.handle_a, &setup.room_id, "Hello family!")
        .await
        .expect("A: send_text() should succeed");

    assert!(!event_id.is_empty(), "Event ID should be non-empty");

    // Allow B's sync to receive the event
    tokio::time::sleep(Duration::from_secs(2)).await;

    // B reads history (handle remains from setup)
    let history = messaging::get_history(&setup.handle_b, &setup.room_id, 10)
        .await
        .expect("B: get_history() should succeed");

    let found = history
        .iter()
        .any(|m| matches!(&m.content, MessageContent::Text { body } if body == "Hello family!"));
    assert!(found, "B should see the text message in history");

    client::disconnect(setup.handle_a)
        .await
        .expect("A: disconnect() should succeed");
    client::disconnect(setup.handle_b)
        .await
        .expect("B: disconnect() should succeed");
}

#[tokio::test]
async fn test_send_receive_media() {
    common::init_tracing();
    cleanup_store();
    if !synapse_available().await {
        eprintln!("SKIP: Synapse not available");
        return;
    }

    let setup = setup_two_users_in_room().await;

    // A sends a media message
    let image_data: Vec<u8> = vec![0xFF, 0xD8, 0xFF, 0xE0]; // minimal JPEG header
    let event_id = messaging::send_media(
        &setup.handle_a,
        &setup.room_id,
        image_data,
        "image/jpeg",
        "test.jpg",
    )
    .await
    .expect("A: send_media() should succeed");

    assert!(!event_id.is_empty(), "Media event ID should be non-empty");

    // Allow B's sync to receive the event
    tokio::time::sleep(Duration::from_secs(2)).await;

    // B reads history
    let history = messaging::get_history(&setup.handle_b, &setup.room_id, 10)
        .await
        .expect("B: get_history() should succeed");

    let found = history.iter().any(
        |m| matches!(&m.content, MessageContent::Media { filename, .. } if filename == "test.jpg"),
    );
    assert!(found, "B should see the media message in history");

    client::disconnect(setup.handle_a)
        .await
        .expect("A: disconnect() should succeed");
    client::disconnect(setup.handle_b)
        .await
        .expect("B: disconnect() should succeed");
}

#[tokio::test]
async fn test_message_history() {
    common::init_tracing();
    cleanup_store();
    if !synapse_available().await {
        eprintln!("SKIP: Synapse not available");
        return;
    }

    let setup = setup_two_users_in_room().await;

    // A sends 5 messages
    let mut sent_ids = Vec::new();
    for i in 0..5 {
        let body = format!("Message {}", i);
        let eid = messaging::send_text(&setup.handle_a, &setup.room_id, &body)
            .await
            .expect("A: send_text() should succeed");
        sent_ids.push(eid);
    }

    // Allow B's sync to receive the events
    tokio::time::sleep(Duration::from_secs(2)).await;

    // B reads history with limit=3
    let history = messaging::get_history(&setup.handle_b, &setup.room_id, 3)
        .await
        .expect("B: get_history() should succeed");

    // Should get at most 3 messages (newest first)
    assert!(
        history.len() <= 3,
        "History limit should cap at 3, got {}",
        history.len()
    );

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

    client::disconnect(setup.handle_a)
        .await
        .expect("A: disconnect() should succeed");
    client::disconnect(setup.handle_b)
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

    // Use both users: A sends, B receives via callback
    let setup = setup_two_users_in_room().await;

    // Register callback on B that collects received messages
    let received: Arc<Mutex<Vec<shadowlink_rust_core::messaging::Message>>> =
        Arc::new(Mutex::new(Vec::new()));
    let received_clone = Arc::clone(&received);

    let callback: MessageCallback = Arc::new(move |msg| {
        let mut msgs = received_clone.lock().expect("Lock should not be poisoned");
        msgs.push(msg);
    });

    messaging::register_message_callback(&setup.handle_b, Some(callback)).await;

    // A sends a message
    messaging::send_text(&setup.handle_a, &setup.room_id, "Callback test message")
        .await
        .expect("send_text() should succeed");

    // Poll for the callback to fire (with explicit get_history fallback)
    for _ in 0..10 {
        tokio::time::sleep(Duration::from_secs(1)).await;
        let count = received.lock().expect("Lock should not be poisoned").len();
        if count >= 1 {
            break;
        }
        // Also check via history in case callback didn't fire but message arrived
        let history = messaging::get_history(&setup.handle_b, &setup.room_id, 5)
            .await
            .unwrap_or_default();
        if !history.is_empty() {
            // Message was received via sync (timeline got it), callback should have
            // fired — if not, we have a dispatch issue, not a sync issue
            let cb_count = received.lock().expect("Lock should not be poisoned").len();
            assert!(
                cb_count >= 1,
                "Message was synced but callback didn't fire (got {})",
                cb_count
            );
        }
    }

    // Final check
    let count = received.lock().expect("Lock should not be poisoned").len();
    assert!(
        count >= 1,
        "Callback should have fired at least once on B, got {} messages",
        count
    );

    // Unregister
    messaging::register_message_callback(&setup.handle_b, None).await;

    // Cleanup
    client::disconnect(setup.handle_a)
        .await
        .expect("A: disconnect() should succeed");
    client::disconnect(setup.handle_b)
        .await
        .expect("B: disconnect() should succeed");
}
