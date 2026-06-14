// US2: Room Operations — Integration Tests
//
// Prerequisite: `docker compose up -d` with Synapse on localhost:8008
// Run: `cargo test test_us2_ -- --test-threads=1`

mod common;

use common::{cleanup_store, create_ephemeral_user_pair, register_test_user, synapse_available};
use shadowlink_rust_core::client;
use shadowlink_rust_core::rooms::{self, RoomState};
use std::time::Duration;

const HOMESERVER_URL: &str = "http://localhost:8008";

#[tokio::test]
async fn test_create_and_list_rooms() {
    common::init_tracing();
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

    // Create a room with a name
    let room = rooms::create_room(&handle, "Room Test")
        .await
        .expect("create_room() should succeed");

    assert!(!room.room_id.is_empty(), "Room should have a valid room_id");
    assert!(room.encrypted, "Room should have E2EE enabled");
    assert_eq!(
        room.state,
        RoomState::Joined,
        "Creator should be joined state"
    );

    // Allow sync to populate room metadata locally
    tokio::time::sleep(Duration::from_secs(2)).await;

    // List rooms and verify the room appears
    let rooms = rooms::list_rooms(&handle)
        .await
        .expect("list_rooms() should succeed");

    let found = rooms.iter().any(|r| r.room_id == room.room_id);
    assert!(found, "Created room should appear in room list");

    client::disconnect(handle)
        .await
        .expect("disconnect() should succeed");
}

#[tokio::test]
async fn test_accept_invite() {
    common::init_tracing();
    if !synapse_available().await {
        eprintln!("SKIP: Synapse not available");
        return;
    }

    cleanup_store();
    let (user_a, user_b) = create_ephemeral_user_pair()
        .await
        .expect("Failed to register test users");

    // Session A: connect, create room, invite B
    let handle_a = client::connect(HOMESERVER_URL, &user_a.username, &user_a.password)
        .await
        .expect("A: connect() should succeed");

    let room = rooms::create_room(&handle_a, "Invite Test")
        .await
        .expect("A: create_room() should succeed");

    rooms::invite_user(&handle_a, &room.room_id, &user_b.user_id)
        .await
        .expect("A: invite_user() should succeed");

    client::disconnect(handle_a)
        .await
        .expect("A: disconnect() should succeed");
    cleanup_store();

    // Session B: connect, find invite, accept it
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
        .find(|r| r.state == RoomState::Invited)
        .expect("B should have an invited room");

    let joined_room = rooms::accept_invite(&handle_b, &invited.room_id)
        .await
        .expect("B: accept_invite() should succeed");

    assert_eq!(
        joined_room.state,
        RoomState::Joined,
        "Room should be joined after accept"
    );

    // B lists rooms — should now show the room as joined
    let rooms_after = rooms::list_rooms(&handle_b)
        .await
        .expect("B: list_rooms() should succeed after accept");

    let b_state = rooms_after
        .iter()
        .find(|r| r.room_id == invited.room_id)
        .expect("Room should still appear in B's list");

    assert_eq!(
        b_state.state,
        RoomState::Joined,
        "B's room state should be Joined"
    );

    client::disconnect(handle_b)
        .await
        .expect("B: disconnect() should succeed");
}

#[tokio::test]
async fn test_invite_user() {
    common::init_tracing();
    if !synapse_available().await {
        eprintln!("SKIP: Synapse not available");
        return;
    }

    cleanup_store();
    let (user_a, user_b) = create_ephemeral_user_pair()
        .await
        .expect("Failed to register test users");

    let handle_a = client::connect(HOMESERVER_URL, &user_a.username, &user_a.password)
        .await
        .expect("A: connect() should succeed");

    let room = rooms::create_room(&handle_a, "Invite User Test")
        .await
        .expect("A: create_room() should succeed");

    // Invite B by user_id — this is the core assertion
    let result = rooms::invite_user(&handle_a, &room.room_id, &user_b.user_id).await;
    assert!(result.is_ok(), "A: invite_user() to B should succeed");

    // Verify B can see the invite
    client::disconnect(handle_a)
        .await
        .expect("A: disconnect() should succeed");
    cleanup_store();

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
        .find(|r| r.state == RoomState::Invited)
        .expect("B should see the invited room");

    assert_eq!(
        invited.name.as_deref(),
        Some("Invite User Test"),
        "Invited room name should match"
    );

    client::disconnect(handle_b)
        .await
        .expect("B: disconnect() should succeed");
}

#[tokio::test]
async fn test_leave_room() {
    common::init_tracing();
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

    let room = rooms::create_room(&handle, "Leave Test")
        .await
        .expect("create_room() should succeed");

    // Leave the room
    rooms::leave_room(&handle, &room.room_id)
        .await
        .expect("leave_room() should succeed");

    // List rooms — the room should no longer be Joined
    let rooms = rooms::list_rooms(&handle)
        .await
        .expect("list_rooms() should succeed");

    let maybe_room = rooms.iter().find(|r| r.room_id == room.room_id);
    if let Some(r) = maybe_room {
        assert_eq!(
            r.state,
            RoomState::Left,
            "Room should be in Left state after leaving"
        );
    }
    // Room may not appear at all if the SDK drops left rooms from the list;
    // either outcome is acceptable. The key assertion is that it's not Joined.

    client::disconnect(handle)
        .await
        .expect("disconnect() should succeed");
}
