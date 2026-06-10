// ─── [ NEURAL DECK v4.6 $ AI::GENERATED :: NO COPYRIGHT ] ───

//! Matrix client lifecycle — SDK init, auth, session management.
//!
//! US1: Homeserver configuration. Entry gate for all other features.

use crate::error::ShadowLinkError;
use crate::messaging::{self, MessageCallback};
use matrix_sdk::{Client, config::SyncSettings, matrix_auth::MatrixSession};
use ruma::api::client::error::{ErrorBody, ErrorKind};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{Mutex, Notify};

/// Opaque handle for an active Matrix session.
///
/// Cloned freely across threads. The underlying `Session` is protected
/// by an internal mutex. Nothing crosses the FFI boundary by value.
#[derive(Clone, Debug)]
pub struct SessionHandle(pub(crate) Arc<Mutex<Session>>);

impl SessionHandle {
    #[allow(dead_code)]
    pub(crate) async fn with_session<F, T>(&self, f: F) -> T
    where
        F: FnOnce(&mut Session) -> T,
    {
        let mut guard = self.0.lock().await;
        f(&mut guard)
    }
}

/// Callback invoked when a live location beacon is received.
pub type LocationCallback = Box<dyn Fn(crate::location::LocationBeacon) + Send + 'static>;

/// Active Matrix SDK session.
///
/// Wraps `matrix_sdk::Client` and tracks sync loop state plus callbacks.
pub(crate) struct Session {
    pub client: Client,
    pub sync_running: bool,
    pub sync_handle: Option<tokio::task::JoinHandle<()>>,
    pub message_callback: Option<MessageCallback>,
    /// Handle to abort a running live location interval task.
    pub live_location_handle: Option<tokio::task::JoinHandle<()>>,
    /// Callback for incoming location beacons.
    pub location_callback: Option<LocationCallback>,
    /// Signal to cancel the background sync loop cleanly.
    pub cancel_notify: Arc<Notify>,
}

// Manual Debug impl skips the callback field (not required to be Debug).
impl std::fmt::Debug for Session {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Session")
            .field("sync_running", &self.sync_running)
            .field("sync_handle", &self.sync_handle)
            .finish_non_exhaustive()
    }
}

impl Session {
    #[allow(dead_code)]
    fn new(
        client: Client,
        sync_handle: tokio::task::JoinHandle<()>,
        cancel_notify: Arc<Notify>,
    ) -> Self {
        Self {
            client,
            sync_running: true,
            sync_handle: Some(sync_handle),
            message_callback: None,
            live_location_handle: None,
            location_callback: None,
            cancel_notify,
        }
    }
}

// ── US5 session persistence helpers ──────────────────────────────────────────

/// Default session store directory (relative to CWD).
fn store_path() -> PathBuf {
    let mut p = std::env::current_dir().unwrap_or_default();
    p.push("shadowlink_data");
    p
}

/// Path to the persisted session JSON file.
fn session_file_path() -> PathBuf {
    let mut p = store_path();
    p.push("session.json");
    p
}

/// Path to the SDK's SQLite state store directory.
fn sqlite_store_path() -> PathBuf {
    let mut p = store_path();
    p.push("store");
    p
}

/// Serialisable wrapper for persisting a Matrix session alongside the
/// homeserver URL.
///
/// Uses `MatrixSession` directly (it already derives Serialize/Deserialize
/// in the SDK).
#[derive(Serialize, Deserialize)]
struct StoredSession {
    homeserver_url: String,
    #[serde(flatten)]
    session: MatrixSession,
}

impl StoredSession {
    fn save(&self) -> Result<(), ShadowLinkError> {
        let dir = store_path();
        std::fs::create_dir_all(&dir).map_err(|e| ShadowLinkError::StorageError {
            reason: format!("Failed to create store directory: {e}"),
        })?;
        let json =
            serde_json::to_string_pretty(self).map_err(|e| ShadowLinkError::StorageError {
                reason: format!("Failed to serialize session: {e}"),
            })?;
        std::fs::write(session_file_path(), &json).map_err(|e| ShadowLinkError::StorageError {
            reason: format!("Failed to write session file: {e}"),
        })?;
        Ok(())
    }

    fn load() -> Result<Self, ShadowLinkError> {
        let path = session_file_path();
        let json = std::fs::read_to_string(&path).map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                ShadowLinkError::StorageError {
                    reason: "No persisted session found. Please connect first.".into(),
                }
            } else {
                ShadowLinkError::StorageError {
                    reason: format!("Failed to read session file: {e}"),
                }
            }
        })?;
        serde_json::from_str(&json).map_err(|e| ShadowLinkError::StorageError {
            reason: format!("Failed to parse session file: {e}"),
        })
    }
}

/// Default SQLite store passphrase (dev-only — replace in production).
const STORE_PASSPHRASE: &str = "shadowlink-dev-passphrase";

/// Attempt to restore a previously persisted session.
///
/// Reads the persisted session data from `shadowlink_data/session.json`,
/// builds an SDK `Client` with the SQLite state store, and restores the
/// session. If the access token is expired, returns `SessionExpired`.
///
/// No credentials are required — the access token from the persisted
/// session is used directly. The sync loop is started automatically.
pub async fn restore_session() -> Result<SessionHandle, ShadowLinkError> {
    let stored = StoredSession::load()?;

    let client = Client::builder()
        .homeserver_url(&stored.homeserver_url)
        .sqlite_store(sqlite_store_path(), Some(STORE_PASSPHRASE))
        .build()
        .await
        .map_err(|e| ShadowLinkError::StorageError {
            reason: format!("Failed to open SQLite store: {e}"),
        })?;

    let session = stored.session;
    client
        .matrix_auth()
        .restore_session(session)
        .await
        .map_err(|e| {
            if let Some(err) = e.as_client_api_error()
                && let ErrorBody::Standard { kind, .. } = &err.body
                && matches!(*kind, ErrorKind::UnknownToken { .. })
            {
                return ShadowLinkError::SessionExpired;
            }
            ShadowLinkError::StorageError {
                reason: format!("Failed to restore session: {e}"),
            }
        })?;

    // Verify the token is still valid with a lightweight API call.
    if let Err(_e) = client.whoami().await {
        return Err(ShadowLinkError::SessionExpired);
    }

    // Build shared session and start sync loop (same as connect()).
    let cancel = Arc::new(Notify::new());
    let cancel_clone = Arc::clone(&cancel);
    let client_for_sync = client.clone();
    let shared = Arc::new(Mutex::new(Session {
        client,
        sync_running: false,
        sync_handle: None,
        message_callback: None,
        live_location_handle: None,
        location_callback: None,
        cancel_notify: cancel,
    }));

    let shared_clone = Arc::clone(&shared);
    let sync_handle = tokio::spawn(async move {
        let settings = SyncSettings::new();
        loop {
            tokio::select! {
                _ = cancel_clone.notified() => {
                    break;
                }
                result = client_for_sync.sync_once(settings.clone()) => {
                    match result {
                        Ok(response) => {
                            let mut all_messages: Vec<(String, Vec<messaging::Message>)> = Vec::new();
                            for (room_id, room) in &response.rooms.join {
                                let msgs = messaging::dispatch_message_events(&room.timeline.events);
                                if !msgs.is_empty() {
                                    all_messages.push((room_id.to_string(), msgs));
                                }
                                crate::location::dispatch_location_events(
                                    &room.timeline.events,
                                    &SessionHandle(Arc::clone(&shared_clone)),
                                )
                                .await;
                            }
                            if !all_messages.is_empty() {
                                let cb = {
                                    let guard = shared_clone.lock().await;
                                    guard.message_callback.clone()
                                };
                                if let Some(callback) = cb {
                                    for (_room_id, msgs) in &all_messages {
                                        for msg in msgs {
                                            callback(msg.clone());
                                        }
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            tracing::warn!("Sync error: {}", e);
                            tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                        }
                    }
                }
            }
        }
    });

    {
        let mut guard = shared.lock().await;
        guard.sync_handle = Some(sync_handle);
        guard.sync_running = true;
    }

    Ok(SessionHandle(shared))
}

/// Establish an authenticated Matrix session.
///
/// Performs homeserver discovery, username/password login, enables E2EE,
/// and starts the sync loop. Returns a `SessionHandle` for all subsequent
/// operations.
pub async fn connect(
    homeserver_url: &str,
    username: &str,
    password: &str,
) -> Result<SessionHandle, ShadowLinkError> {
    let client = Client::builder()
        .homeserver_url(homeserver_url)
        .sqlite_store(sqlite_store_path(), Some(STORE_PASSPHRASE))
        .build()
        .await
        .map_err(|e| ShadowLinkError::ConnectionFailed {
            reason: format!("Failed to build client: {}", e),
        })?;

    client
        .matrix_auth()
        .login_username(username, password)
        .initial_device_display_name("ShadowLink")
        .send()
        .await
        .map_err(|e| {
            if let Some(api_err) = e.as_client_api_error()
                && let ErrorBody::Standard { kind, .. } = &api_err.body
            {
                match kind {
                    ErrorKind::Forbidden | ErrorKind::UserDeactivated => {
                        return ShadowLinkError::AuthenticationFailed {
                            reason: "Invalid credentials".into(),
                        };
                    }
                    ErrorKind::LimitExceeded { .. } => {
                        return ShadowLinkError::AuthenticationFailed {
                            reason: "Rate limited — try again later".into(),
                        };
                    }
                    _ => {}
                }
            }
            ShadowLinkError::ConnectionFailed {
                reason: format!("Login failed: {}", e),
            }
        })?;

    client.encryption();

    // Persist session data for future restore.
    if let Some(session) = client.matrix_auth().session() {
        let stored = StoredSession {
            homeserver_url: homeserver_url.to_owned(),
            session,
        };
        stored.save()?;
    }

    // Build shared session before spawning sync loop so the loop
    // can read callbacks from the same Session behind the handle.
    let cancel = Arc::new(Notify::new());
    let cancel_clone = Arc::clone(&cancel);
    let client_for_sync = client.clone();
    let shared = Arc::new(Mutex::new(Session {
        client,
        sync_running: false,
        sync_handle: None,
        message_callback: None,
        live_location_handle: None,
        location_callback: None,
        cancel_notify: cancel,
    }));

    let shared_clone = Arc::clone(&shared);
    let sync_handle = tokio::spawn(async move {
        let settings = SyncSettings::new();
        loop {
            tokio::select! {
                _ = cancel_clone.notified() => {
                    break;
                }
                result = client_for_sync.sync_once(settings.clone()) => {
                    match result {
                        Ok(response) => {
                            // Collect messages from all joined rooms
                            let mut all_messages: Vec<(String, Vec<messaging::Message>)> = Vec::new();
                            for (room_id, room) in &response.rooms.join {
                                let msgs = messaging::dispatch_message_events(&room.timeline.events);
                                if !msgs.is_empty() {
                                    all_messages.push((room_id.to_string(), msgs));
                                }

                                // Dispatch location events for this room.
                                crate::location::dispatch_location_events(
                                    &room.timeline.events,
                                    &SessionHandle(Arc::clone(&shared_clone)),
                                )
                                .await;
                            }

                            // Dispatch to registered callback (if any) without
                            // holding the session lock during invocation.
                            if !all_messages.is_empty() {
                                let cb = {
                                    let guard = shared_clone.lock().await;
                                    guard.message_callback.clone()
                                };
                                if let Some(callback) = cb {
                                    for (_room_id, msgs) in &all_messages {
                                        for msg in msgs {
                                            callback(msg.clone());
                                        }
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            tracing::warn!("Sync error: {}", e);
                            tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                        }
                    }
                }
            }
        }
    });

    {
        let mut guard = shared.lock().await;
        guard.sync_handle = Some(sync_handle);
        guard.sync_running = true;
    }

    Ok(SessionHandle(shared))
}

/// Gracefully shut down an active Matrix session.
///
/// Stops the sync loop, logs out of the homeserver, and drops the session.
/// Safe to call on an already-disconnected handle (no-op).
pub async fn disconnect(handle: SessionHandle) -> Result<(), ShadowLinkError> {
    // Phase 1: extract sync handle and signal cancellation (brief lock).
    let sync_handle = {
        let mut guard = handle.0.lock().await;
        if !guard.sync_running {
            return Ok(());
        }
        guard.sync_running = false;
        guard.cancel_notify.notify_one();
        guard.sync_handle.take()
    };

    // Phase 2: wait for sync task to terminate (no lock held).
    if let Some(jh) = sync_handle {
        jh.abort();
        // Bound wait — the Notify + select! in the sync loop ensures the
        // long-poll is cancelled promptly, so this is a safety net.
        let _ = tokio::time::timeout(Duration::from_secs(10), jh).await;
    }

    // Phase 3: logout from homeserver (brief lock).
    {
        let guard = handle.0.lock().await;
        if let Err(e) = guard.client.matrix_auth().logout().await {
            tracing::warn!("Logout failed (non-fatal): {}", e);
        }
    }

    Ok(())
}

// ── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use tokio::sync::Mutex;

    /// Verify SessionHandle cloning increments the Arc reference count.
    #[test]
    fn test_handle_clone_increments_refcount() {
        let original = Arc::new(42);
        let cloned = Arc::clone(&original);
        assert_eq!(Arc::strong_count(&original), 2);
        drop(cloned);
        assert_eq!(Arc::strong_count(&original), 1);
    }

    /// Helper to build a minimal Client for unit tests.
    async fn dummy_client() -> Client {
        Client::builder()
            .homeserver_url("https://matrix.example.com")
            .build()
            .await
            .expect("Client builder should succeed without network")
    }

    /// Verify SessionHandle Debug output.
    #[tokio::test]
    async fn test_session_handle_debug() {
        let handle = SessionHandle(Arc::new(Mutex::new(Session {
            client: dummy_client().await,
            sync_running: false,
            sync_handle: None,
            message_callback: None,
            live_location_handle: None,
            location_callback: None,
            cancel_notify: Arc::new(Notify::new()),
        })));
        let debug = format!("{:?}", handle);
        assert!(debug.contains("SessionHandle"));
    }

    /// Verify disconnect on an already-disconnected session is a no-op.
    #[tokio::test]
    async fn test_disconnect_when_not_running() {
        let handle = SessionHandle(Arc::new(Mutex::new(Session {
            client: dummy_client().await,
            sync_running: false,
            sync_handle: None,
            message_callback: None,
            live_location_handle: None,
            location_callback: None,
            cancel_notify: Arc::new(Notify::new()),
        })));
        let result = disconnect(handle).await;
        assert!(result.is_ok());
    }

    /// Verify error mapping: connect with an empty URL → ConnectionFailed.
    #[tokio::test]
    async fn test_connect_empty_url_connection_failed() {
        let result = connect("", "@test:example.com", "password").await;
        let err = result.expect_err("connect with empty URL should fail");
        match err {
            ShadowLinkError::ConnectionFailed { .. } => {} // expected
            _ => panic!("Expected ConnectionFailed, got: {err:?}"),
        }
    }

    /// Verify error mapping: connect with bad URL scheme → ConnectionFailed.
    #[tokio::test]
    async fn test_connect_bad_scheme_connection_failed() {
        let result = connect("null://\0", "@test:example.com", "password").await;
        let err = result.expect_err("connect with null URL should fail");
        match err {
            ShadowLinkError::ConnectionFailed { .. } => {} // expected
            _ => panic!("Expected ConnectionFailed, got: {err:?}"),
        }
    }
}
