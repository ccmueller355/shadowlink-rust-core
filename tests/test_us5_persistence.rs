// US5: Session Persistence — Integration Tests
//
// Prerequisite: `docker compose up -d` with Synapse on localhost:8008
// Run: `cargo test test_us5_ -- --test-threads=1`

mod common;

use common::{cleanup_store, register_test_user, synapse_available};
use shadowlink_rust_core::client;
use shadowlink_rust_core::messaging::{self, MessageContent};
use shadowlink_rust_core::rooms;
use std::time::Duration;

const HOMESERVER_URL: &str = "http://localhost:8008";

#[tokio::test]
async fn test_session_persistence_across_restart() {
    common::init_tracing();
    if !synapse_available().await {
        eprintln!("SKIP: Synapse not available");
        return;
    }

    cleanup_store();

    let user = register_test_user()
        .await
        .expect("Failed to register test user");

    // ── Phase 1: Connect, create room, send message, disconnect ──
    let handle = client::connect(HOMESERVER_URL, &user.username, &user.password)
        .await
        .expect("Phase 1: connect() should succeed");

    let room = rooms::create_room(&handle, "Persistence Test")
        .await
        .expect("Phase 1: create_room() should succeed");

    let _event_id = messaging::send_text(&handle, &room.room_id, "Persist this message")
        .await
        .expect("Phase 1: send_text() should succeed");

    client::disconnect(handle)
        .await
        .expect("Phase 1: disconnect() should succeed");

    // ── Phase 2: Restore session via persisted token ──
    // Note: restore_session() reads from shadowlink_data/session.json
    // which was saved during connect() in Phase 1.
    let restored = client::restore_session()
        .await
        .expect("Phase 2: restore_session() should succeed");

    // Allow sync to catch up
    tokio::time::sleep(Duration::from_secs(2)).await;

    // Verify the restored session can list rooms
    let rooms_list = rooms::list_rooms(&restored)
        .await
        .expect("Phase 2: list_rooms() should succeed after restore");

    let found = rooms_list
        .iter()
        .any(|r| r.room_id == room.room_id);
    assert!(
        found,
        "Restored session should see the previously created room"
    );

    // Verify restored session can fetch history
    let history = messaging::get_history(&restored, &room.room_id, 10)
        .await
        .expect("Phase 2: get_history() should succeed after restore");

    let persisted_msg = history.iter().any(|m| {
        matches!(&m.content, MessageContent::Text { body } if body == "Persist this message")
    });
    assert!(
        persisted_msg,
        "Restored session should retrieve the persisted message"
    );

    client::disconnect(restored)
        .await
        .expect("Phase 2: disconnect() should succeed");
}

#[tokio::test]
async fn test_restored_session_room_list() {
    common::init_tracing();
    if !synapse_available().await {
        eprintln!("SKIP: Synapse not available");
        return;
    }

    cleanup_store();

    let user = register_test_user()
        .await
        .expect("Failed to register test user");

    // Connect and create multiple rooms
    let handle = client::connect(HOMESERVER_URL, &user.username, &user.password)
        .await
        .expect("connect() should succeed");

    let room_a = rooms::create_room(&handle, "Room Alpha")
        .await
        .expect("create_room Alpha should succeed");

    let room_b = rooms::create_room(&handle, "Room Beta")
        .await
        .expect("create_room Beta should succeed");

    client::disconnect(handle)
        .await
        .expect("disconnect() should succeed");

    // Restore and verify both rooms appear
    let restored = client::restore_session()
        .await
        .expect("restore_session() should succeed");

    tokio::time::sleep(Duration::from_secs(2)).await;

    let rooms_list = rooms::list_rooms(&restored)
        .await
        .expect("list_rooms() should succeed after restore");

    let found_a = rooms_list.iter().any(|r| r.room_id == room_a.room_id);
    let found_b = rooms_list.iter().any(|r| r.room_id == room_b.room_id);
    assert!(found_a, "Restored session should see Room Alpha");
    assert!(found_b, "Restored session should see Room Beta");

    client::disconnect(restored)
        .await
        .expect("disconnect() should succeed");
}

#[tokio::test]
async fn test_restored_session_history() {
    common::init_tracing();
    if !synapse_available().await {
        eprintln!("SKIP: Synapse not available");
        return;
    }

    cleanup_store();

    let user = register_test_user()
        .await
        .expect("Failed to register test user");

    // Connect, create room, send a message
    let handle = client::connect(HOMESERVER_URL, &user.username, &user.password)
        .await
        .expect("connect() should succeed");

    let room = rooms::create_room(&handle, "History Test")
        .await
        .expect("create_room() should succeed");

    messaging::send_text(&handle, &room.room_id, "Message A")
        .await
        .expect("send_text A should succeed");

    messaging::send_text(&handle, &room.room_id, "Message B")
        .await
        .expect("send_text B should succeed");

    client::disconnect(handle)
        .await
        .expect("disconnect() should succeed");

    // Restore and check history
    let restored = client::restore_session()
        .await
        .expect("restore_session() should succeed");

    tokio::time::sleep(Duration::from_secs(2)).await;

    let history = messaging::get_history(&restored, &room.room_id, 10)
        .await
        .expect("get_history() should succeed after restore");

    let bodies: Vec<&str> = history
        .iter()
        .filter_map(|m| match &m.content {
            MessageContent::Text { body } => Some(body.as_str()),
            _ => None,
        })
        .collect();

    assert!(
        bodies.contains(&"Message A"),
        "History should contain Message A, got: {:?}",
        bodies
    );
    assert!(
        bodies.contains(&"Message B"),
        "History should contain Message B, got: {:?}",
        bodies
    );

    client::disconnect(restored)
        .await
        .expect("disconnect() should succeed");
}
