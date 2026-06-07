// US2: Room Operations — Integration Tests
//
// Prerequisite: `docker compose up -d` with Synapse on localhost:8008
// Run: `cargo test test_us2_ -- --ignored --test-threads=1`

mod common;

use common::create_ephemeral_user_pair;

#[ignore = "requires local Synapse (docker compose up -d)"]
#[tokio::test]
async fn test_create_and_list_rooms() {
    let (user_a, _user_b) = create_ephemeral_user_pair()
        .await
        .expect("Failed to register test users");

    assert!(!user_a.token.is_empty(), "User A registered");
    // TODO: create a room with name, list rooms, verify it appears
}

#[ignore = "requires local Synapse (docker compose up -d)"]
#[tokio::test]
async fn test_accept_invite() {
    let (user_a, _user_b) = create_ephemeral_user_pair()
        .await
        .expect("Failed to register test users");

    assert!(!user_a.token.is_empty(), "User A registered");
    // TODO: A creates room, invites B, B accepts, both list rooms
}

#[ignore = "requires local Synapse (docker compose up -d)"]
#[tokio::test]
async fn test_invite_user() {
    let (user_a, user_b) = create_ephemeral_user_pair()
        .await
        .expect("Failed to register test users");

    assert!(!user_a.token.is_empty(), "User A registered");
    assert!(!user_b.user_id.is_empty(), "User B has user_id");
    // TODO: A creates room, invites B by user_id, assert Ok
}

#[ignore = "requires local Synapse (docker compose up -d)"]
#[tokio::test]
async fn test_leave_room() {
    let (user_a, _user_b) = create_ephemeral_user_pair()
        .await
        .expect("Failed to register test users");

    assert!(!user_a.token.is_empty(), "User A registered");
    // TODO: create room, leave, list rooms — verify no longer joined
}
