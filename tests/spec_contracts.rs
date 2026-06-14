// ─── [ NEURAL DECK v4.6 ] ───
//! Contract compilation tests — verify every FFI function signature from
//! `contracts/ffi-contract.md` compiles.
//!
//! These tests do NOT require a running Synapse. They are purely
//! compile-time + lightweight runtime assertions.

use shadowlink_rust_core::client::SessionHandle;
use shadowlink_rust_core::error::ShadowLinkError;
use shadowlink_rust_core::location::LiveLocationConfig;
use shadowlink_rust_core::location::LocationBeacon;
use shadowlink_rust_core::rooms::RoomInfo;

// ── Assertions that types exist with expected traits ────────────────────

#[test]
fn test_session_handle_is_send() {
    fn assert_send<T: Send>() {}
    assert_send::<SessionHandle>();
}

#[test]
fn test_shadow_link_error_is_send() {
    fn assert_send<T: Send>() {}
    assert_send::<ShadowLinkError>();
}

#[test]
fn test_location_beacon_traits() {
    fn assert_send<T: Send>() {}
    fn assert_clone<T: Clone>() {}
    fn assert_debug<T: std::fmt::Debug>() {}
    assert_send::<LocationBeacon>();
    assert_clone::<LocationBeacon>();
    assert_debug::<LocationBeacon>();
}

#[test]
fn test_room_info_traits() {
    fn assert_clone<T: Clone>() {}
    fn assert_debug<T: std::fmt::Debug>() {}
    assert_clone::<RoomInfo>();
    assert_debug::<RoomInfo>();
}

#[test]
fn test_live_location_config_traits() {
    fn assert_clone<T: Clone>() {}
    fn assert_debug<T: std::fmt::Debug>() {}
    assert_clone::<LiveLocationConfig>();
    assert_debug::<LiveLocationConfig>();
}

// ── Compile-time function existence checks ──────────────────────────────
// Each test simply references the function to verify it exists with the
// correct module path. The type system ensures the function is accessible
// with its expected name.

#[test]
fn test_connect_exists() {
    let _ = shadowlink_rust_core::client::connect;
}

#[test]
fn test_disconnect_exists() {
    let _ = shadowlink_rust_core::client::disconnect;
}

#[test]
fn test_restore_session_exists() {
    let _ = shadowlink_rust_core::client::restore_session;
}

#[test]
fn test_create_room_exists() {
    let _ = shadowlink_rust_core::rooms::create_room;
}

#[test]
fn test_list_rooms_exists() {
    let _ = shadowlink_rust_core::rooms::list_rooms;
}

#[test]
fn test_accept_invite_exists() {
    let _ = shadowlink_rust_core::rooms::accept_invite;
}

#[test]
fn test_invite_user_exists() {
    let _ = shadowlink_rust_core::rooms::invite_user;
}

#[test]
fn test_leave_room_exists() {
    let _ = shadowlink_rust_core::rooms::leave_room;
}

#[test]
fn test_send_text_exists() {
    let _ = shadowlink_rust_core::messaging::send_text;
}

#[test]
fn test_send_media_exists() {
    let _ = shadowlink_rust_core::messaging::send_media;
}

#[test]
fn test_get_history_exists() {
    let _ = shadowlink_rust_core::messaging::get_history;
}

#[test]
fn test_register_message_callback_exists() {
    let _ = shadowlink_rust_core::messaging::register_message_callback;
}

#[test]
fn test_send_beacon_exists() {
    let _ = shadowlink_rust_core::location::send_beacon;
}

#[test]
fn test_start_live_location_exists() {
    let _ = shadowlink_rust_core::location::start_live_location;
}

#[test]
fn test_stop_live_location_exists() {
    let _ = shadowlink_rust_core::location::stop_live_location;
}

#[test]
fn test_register_location_callback_exists() {
    let _ = shadowlink_rust_core::location::register_location_callback;
}

// ── Future type coherence checks ────────────────────────────────────────
// These verify the return types are compatible with the contract.

#[test]
fn test_connect_future_send() {
    fn assert_fut_send<T: Send>(_fut: &T) {}
    let fut = shadowlink_rust_core::client::connect("http://localhost", "u", "p");
    assert_fut_send(&fut);
}

#[test]
fn test_restore_session_future_send() {
    fn assert_fut_send<T: Send>(_fut: &T) {}
    let fut = shadowlink_rust_core::client::restore_session();
    assert_fut_send(&fut);
}

#[test]
fn test_message_type_send() {
    use shadowlink_rust_core::messaging::Message;
    fn assert_send<T: Send>() {}
    assert_send::<Message>();
}

#[test]
fn test_message_type_debug() {
    use shadowlink_rust_core::messaging::Message;
    fn assert_debug<T: std::fmt::Debug>() {}
    assert_debug::<Message>();
}
