// ─── [ NEURAL DECK v4.6 $ AI::GENERATED :: NO COPYRIGHT ] ───

//! FFI bridge — flutter_rust_bridge v2 integration surface.
//!
//! This module exposes the ShadowLink Rust core API to Dart via
//! flutter_rust_bridge v2 codegen. All public functions in the sibling
//! modules (`client`, `rooms`, `messaging`, `location`, `encryption`)
//! are automatically picked up by the codegen when placed in a
//! flutter_rust_bridge v2 project.
//!
//! This file provides:
//!  - The `ShadowLinkApi` struct (the main FFI entry point).
//!  - `init_app()` for Tokio runtime initialisation.

use crate::client::SessionHandle;
use crate::encryption::{CrossSigningStatus, DeviceInfo};
use crate::error::ShadowLinkError;
use crate::location::LiveLocationConfig;
use crate::messaging::Message;

// ── FFI convenience wrappers ────────────────────────────────────────────

// ── FFI convenience wrappers ────────────────────────────────────────────

/// High-level FFI API struct for flutter_rust_bridge v2 codegen.
///
/// Each method forwards to the corresponding module-level function.
pub struct ShadowLinkApi {
    pub(crate) handle: SessionHandle,
}

impl ShadowLinkApi {
    /// Connect to a Matrix homeserver and return an API instance.
    pub async fn connect(
        homeserver_url: String,
        username: String,
        password: String,
    ) -> Result<Self, ShadowLinkError> {
        let handle = crate::client::connect(&homeserver_url, &username, &password).await?;
        Ok(Self { handle })
    }

    /// Disconnect from the server and clean up.
    pub async fn disconnect(self) -> Result<(), ShadowLinkError> {
        crate::client::disconnect(self.handle).await
    }

    /// Restore a persisted session from the local SQLite store.
    pub async fn restore_session() -> Result<Self, ShadowLinkError> {
        let handle = crate::client::restore_session().await?;
        Ok(Self { handle })
    }

    // ── Rooms ──────────────────────────────────────────────────────────

    pub async fn create_room(&self, name: &str) -> Result<String, ShadowLinkError> {
        let room_info = crate::rooms::create_room(&self.handle, name).await?;
        Ok(room_info.room_id)
    }

    pub async fn list_rooms(&self) -> Result<Vec<crate::rooms::RoomInfo>, ShadowLinkError> {
        crate::rooms::list_rooms(&self.handle).await
    }

    pub async fn accept_invite(&self, room_id: &str) -> Result<String, ShadowLinkError> {
        let room_info = crate::rooms::accept_invite(&self.handle, room_id).await?;
        Ok(room_info.room_id)
    }

    pub async fn invite_user(&self, room_id: &str, user_id: &str) -> Result<(), ShadowLinkError> {
        crate::rooms::invite_user(&self.handle, room_id, user_id).await
    }

    pub async fn leave_room(&self, room_id: &str) -> Result<(), ShadowLinkError> {
        crate::rooms::leave_room(&self.handle, room_id).await
    }

    /// Create the family home room. Returns the room ID.
    pub async fn create_family_room(&self, name: &str) -> Result<String, ShadowLinkError> {
        let room_info = crate::rooms::create_family_room(&self.handle, name).await?;
        Ok(room_info.room_id)
    }

    /// Pin an existing room as the family home room.
    pub async fn set_home_room(&self, room_id: &str) -> Result<String, ShadowLinkError> {
        let room_info = crate::rooms::set_home_room(&self.handle, room_id).await?;
        Ok(room_info.room_id)
    }

    /// Retrieve the pinned family room ID, or None.
    pub async fn get_home_room(&self) -> Result<Option<String>, ShadowLinkError> {
        let maybe = crate::rooms::get_home_room(&self.handle).await?;
        Ok(maybe.map(|r| r.room_id))
    }

    // ── Messaging ───────────────────────────────────────────────────────

    pub async fn send_text(&self, room_id: &str, body: &str) -> Result<String, ShadowLinkError> {
        crate::messaging::send_text(&self.handle, room_id, body).await
    }

    pub async fn send_media(
        &self,
        room_id: &str,
        data: Vec<u8>,
        mime_type: &str,
        filename: &str,
    ) -> Result<String, ShadowLinkError> {
        crate::messaging::send_media(&self.handle, room_id, data, mime_type, filename).await
    }

    pub async fn get_history(
        &self,
        room_id: &str,
        limit: u32,
    ) -> Result<Vec<Message>, ShadowLinkError> {
        crate::messaging::get_history(&self.handle, room_id, limit).await
    }

    // ── Location ────────────────────────────────────────────────────────

    pub async fn share_location(
        &self,
        room_id: &str,
        lat: f64,
        lng: f64,
        accuracy_m: Option<f64>,
        description: Option<String>,
    ) -> Result<String, ShadowLinkError> {
        crate::location::share_location(
            &self.handle,
            room_id,
            lat,
            lng,
            accuracy_m,
            description.as_deref(),
        )
        .await
    }

    pub async fn send_beacon(
        &self,
        room_id: &str,
        lat: f64,
        lng: f64,
        accuracy_m: Option<f64>,
    ) -> Result<String, ShadowLinkError> {
        crate::location::send_beacon(&self.handle, room_id, lat, lng, accuracy_m).await
    }

    pub async fn start_live_location(
        &self,
        room_id: &str,
        lat: f64,
        lng: f64,
        accuracy_m: Option<f64>,
        interval_secs: u64,
    ) -> Result<(), ShadowLinkError> {
        let config = LiveLocationConfig {
            interval_secs,
            accuracy_m,
        };
        crate::location::start_live_location(&self.handle, room_id, lat, lng, accuracy_m, &config)
            .await
    }

    pub async fn stop_live_location(&self) -> Result<(), ShadowLinkError> {
        crate::location::stop_live_location(&self.handle).await;
        Ok(())
    }

    // ── E2EE ────────────────────────────────────────────────────────────

    pub async fn get_device(
        &self,
        user_id: &str,
        device_id: &str,
    ) -> Result<DeviceInfo, ShadowLinkError> {
        crate::encryption::get_device(&self.handle, user_id, device_id).await
    }

    pub async fn get_own_device(&self) -> Result<DeviceInfo, ShadowLinkError> {
        crate::encryption::get_own_device(&self.handle).await
    }

    pub async fn get_user_devices(
        &self,
        user_id: &str,
    ) -> Result<Vec<DeviceInfo>, ShadowLinkError> {
        crate::encryption::get_user_devices(&self.handle, user_id).await
    }

    pub async fn bootstrap_cross_signing(&self) -> Result<(), ShadowLinkError> {
        crate::encryption::bootstrap_cross_signing(&self.handle).await
    }

    pub async fn cross_signing_status(&self) -> Result<CrossSigningStatus, ShadowLinkError> {
        crate::encryption::cross_signing_status(&self.handle).await
    }

    pub async fn export_room_keys(
        &self,
        path: String,
        passphrase: String,
    ) -> Result<(), ShadowLinkError> {
        crate::encryption::export_room_keys(&self.handle, &path, &passphrase).await
    }

    pub async fn import_room_keys(
        &self,
        path: String,
        passphrase: String,
    ) -> Result<(), ShadowLinkError> {
        crate::encryption::import_room_keys(&self.handle, &path, &passphrase).await
    }

    /// Toggle diagnostic debug room.
    pub async fn enable_debug_room(&self, enabled: bool) -> Result<(), ShadowLinkError> {
        crate::client::enable_debug_room(&self.handle, enabled).await
    }
}

/// Register a message callback connected to the Flutter stream.
///
/// This bridges the sync loop's internal callback pattern to the
/// flutter_rust_bridge v2 `StreamSink` mechanism. Call this once after
/// `connect()` with a Dart-provided stream sink.
///
/// The `handle` must be an active session returned from `connect()`.
/// Pass `None` for `callback` to unregister.
pub async fn register_message_callback(
    handle: &SessionHandle,
    callback: Option<crate::messaging::MessageCallback>,
) {
    crate::messaging::register_message_callback(handle, callback).await;
}

/// Register (or unregister) a location beacon callback connected to the Flutter stream.
///
/// Call this once after `connect()` or `restore_session()` with a
/// Dart-provided stream sink. Pass `None` for `callback` to unregister.
pub async fn register_location_callback(
    handle: &SessionHandle,
    callback: Option<crate::client::LocationCallback>,
) {
    crate::location::register_location_callback(handle, callback).await;
}

// ── Tests ─────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_struct_is_send_safe() {
        fn assert_send<T: Send>() {}
        assert_send::<ShadowLinkApi>();
    }
}
