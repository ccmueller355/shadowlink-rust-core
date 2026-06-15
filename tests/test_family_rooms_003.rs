// 003: Family Room Semantics — Integration Tests
//
// Prerequisite: Synapse running on localhost:8008
// Run: `cargo test test_003_ -- --test-threads=1`

mod common;

use common::{cleanup_store, create_ephemeral_user_pair, register_test_user, synapse_available};
use shadowlink_rust_core::client;
use shadowlink_rust_core::rooms::{self, RoomState};
use std::time::Duration;

const HOMESERVER_URL: &str = "http://localhost:8008";

// ── US1: Create Family Room ───────────────────────────────────────────────────

#[tokio::test]
async fn test_003_create_family_room_basic() {
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

    let room = rooms::create_family_room(&handle, "The Smith Family")
        .await
        .expect("create_family_room() should succeed");

    assert!(!room.room_id.is_empty(), "Room should have a valid room_id");
    assert!(room.is_home, "Family room should have is_home: true");
    assert!(room.encrypted, "Family room should have E2EE enabled");
    assert_eq!(room.state, RoomState::Joined);

    // Alias is best-effort — may be None if taken
    if let Some(ref alias) = room.alias {
        assert!(alias.contains("the-smith-family"), "Alias: {alias}");
    }

    // get_home_room() should return this room
    let home = rooms::get_home_room(&handle)
        .await
        .expect("get_home_room() should succeed")
        .expect("get_home_room() should return Some");
    assert_eq!(home.room_id, room.room_id);
    assert!(home.is_home);

    // list_rooms() should mark the family room
    tokio::time::sleep(Duration::from_secs(2)).await;
    let rooms = rooms::list_rooms(&handle)
        .await
        .expect("list_rooms() should succeed");
    let home_in_list = rooms.iter().find(|r| r.is_home);
    assert!(home_in_list.is_some(), "Family room should be marked is_home in list");
    assert_eq!(home_in_list.unwrap().room_id, room.room_id);

    client::disconnect(handle)
        .await
        .expect("disconnect() should succeed");
}

#[tokio::test]
async fn test_003_create_family_room_replaces_previous() {
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

    let room1 = rooms::create_family_room(&handle, "First Family Room")
        .await
        .expect("First create_family_room() should succeed");
    assert!(room1.is_home, "First room should be is_home: true");

    let room2 = rooms::create_family_room(&handle, "Second Family Room")
        .await
        .expect("Second create_family_room() should succeed");
    assert!(room2.is_home, "Second room should be is_home: true");

    tokio::time::sleep(Duration::from_secs(2)).await;
    let rooms = rooms::list_rooms(&handle)
        .await
        .expect("list_rooms() should succeed");

    let home_rooms: Vec<_> = rooms.iter().filter(|r| r.is_home).collect();
    assert_eq!(home_rooms.len(), 1, "Exactly one room should be is_home: true");
    assert_eq!(home_rooms[0].room_id, room2.room_id);

    client::disconnect(handle)
        .await
        .expect("disconnect() should succeed");
}

#[tokio::test]
async fn test_003_create_family_room_invalid_name() {
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

    // Empty name
    let result = rooms::create_family_room(&handle, "").await;
    assert!(result.is_err(), "Empty name should fail");
    assert!(
        matches!(result.unwrap_err(), shadowlink_rust_core::error::ShadowLinkError::OperationFailed { .. }),
        "Expected OperationFailed"
    );

    // Very long name (>255 chars)
    let long_name = "a".repeat(300);
    let result = rooms::create_family_room(&handle, &long_name).await;
    assert!(result.is_err(), "Overly long name should fail");
    assert!(
        matches!(result.unwrap_err(), shadowlink_rust_core::error::ShadowLinkError::OperationFailed { .. }),
        "Expected OperationFailed"
    );

    client::disconnect(handle)
        .await
        .expect("disconnect() should succeed");
}

// ── US2: Home Room Pinning ───────────────────────────────────────────────────

#[tokio::test]
async fn test_003_set_home_room_basic() {
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

    let room = rooms::create_room(&handle, "Generic Room")
        .await
        .expect("create_room() should succeed");

    let pinned = rooms::set_home_room(&handle, &room.room_id)
        .await
        .expect("set_home_room() should succeed");
    assert!(pinned.is_home, "Pinned room should have is_home: true");
    assert_eq!(pinned.room_id, room.room_id);

    let home = rooms::get_home_room(&handle)
        .await
        .expect("get_home_room() should succeed")
        .expect("get_home_room() should return Some after set_home_room");
    assert_eq!(home.room_id, room.room_id);

    client::disconnect(handle)
        .await
        .expect("disconnect() should succeed");
}

#[tokio::test]
async fn test_003_get_home_room_none() {
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

    let home = rooms::get_home_room(&handle)
        .await
        .expect("get_home_room() should succeed");
    assert!(home.is_none(), "get_home_room() should return None when no family room");

    client::disconnect(handle)
        .await
        .expect("disconnect() should succeed");
}

#[tokio::test]
async fn test_003_set_home_room_not_member() {
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

    let result = rooms::set_home_room(&handle, "!nonexistent-room:localhost").await;
    assert!(result.is_err(), "set_home_room on nonexistent room should fail");
    assert!(
        matches!(result.unwrap_err(), shadowlink_rust_core::error::ShadowLinkError::RoomNotFound { .. }),
        "Expected RoomNotFound"
    );

    client::disconnect(handle)
        .await
        .expect("disconnect() should succeed");
}

// ── US3: Debug Room Toggle ──────────────────────────────────────────────────

#[tokio::test]
async fn test_003_debug_room_toggle() {
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

    client::enable_debug_room(&handle, true)
        .await
        .expect("enable_debug_room(true) should succeed");

    client::enable_debug_room(&handle, true)
        .await
        .expect("enable_debug_room(true) twice should succeed");

    client::enable_debug_room(&handle, false)
        .await
        .expect("enable_debug_room(false) should succeed");

    client::enable_debug_room(&handle, true)
        .await
        .expect("enable_debug_room(true) after false should succeed");

    client::disconnect(handle)
        .await
        .expect("disconnect() should succeed");
}

// ── Two-Session Family Room Flow (T040) ─────────────────────────────────────

#[tokio::test]
async fn test_003_two_session_family_room_flow() {
    common::init_tracing();
    if !synapse_available().await {
        eprintln!("SKIP: Synapse not available");
        return;
    }
    cleanup_store();

    let (user_a, user_b) = create_ephemeral_user_pair()
        .await
        .expect("Failed to register test users");

    // Session A (operator): connect, create family room, invite B
    let handle_a = client::connect(HOMESERVER_URL, &user_a.username, &user_a.password)
        .await
        .expect("A: connect() should succeed");

    let room = rooms::create_family_room(&handle_a, "Family HQ")
        .await
        .expect("A: create_family_room() should succeed");
    assert!(room.is_home, "A: Room should be is_home: true");

    rooms::invite_user(&handle_a, &room.room_id, &user_b.user_id)
        .await
        .expect("A: invite_user() should succeed");

    client::disconnect(handle_a)
        .await
        .expect("A: disconnect() should succeed");
    cleanup_store();

    // Session B (member): connect, accept invite, pin as home room
    let handle_b = client::connect(HOMESERVER_URL, &user_b.username, &user_b.password)
        .await
        .expect("B: connect() should succeed");

    tokio::time::sleep(Duration::from_secs(2)).await;

    let b_home = rooms::get_home_room(&handle_b)
        .await
        .expect("B: get_home_room() should succeed");
    assert!(b_home.is_none(), "B: No family room before accepting invite");

    let joined = rooms::accept_invite(&handle_b, &room.room_id)
        .await
        .expect("B: accept_invite() should succeed");
    assert_eq!(joined.state, RoomState::Joined);

    let pinned = rooms::set_home_room(&handle_b, &room.room_id)
        .await
        .expect("B: set_home_room() should succeed");
    assert!(pinned.is_home);

    let b_home = rooms::get_home_room(&handle_b)
        .await
        .expect("B: get_home_room() should succeed")
        .expect("B: get_home_room() should return Some");
    assert_eq!(b_home.room_id, room.room_id);

    tokio::time::sleep(Duration::from_secs(2)).await;
    let b_rooms = rooms::list_rooms(&handle_b)
        .await
        .expect("B: list_rooms() should succeed");
    let b_home_in_list = b_rooms.iter().find(|r| r.is_home);
    assert!(b_home_in_list.is_some(), "B: list_rooms() should show home room");
    assert_eq!(b_home_in_list.unwrap().room_id, room.room_id);

    client::disconnect(handle_b)
        .await
        .expect("B: disconnect() should succeed");
}
