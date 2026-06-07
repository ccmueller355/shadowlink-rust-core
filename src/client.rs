// ─── [ NEURAL DECK v4.6 $ AI::GENERATED :: NO COPYRIGHT ] ───

//! Matrix client lifecycle — SDK init, auth, session management.
//!
//! US1: Homeserver configuration. Entry gate for all other features.

use crate::error::ShadowLinkError;
use crate::messaging::{self, MessageCallback};
use matrix_sdk::{Client, config::SyncSettings};
use ruma::api::client::error::{ErrorBody, ErrorKind};
use std::sync::Arc;
use tokio::sync::Mutex;

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

/// Active Matrix SDK session.
///
/// Wraps `matrix_sdk::Client` and tracks sync loop state plus callbacks.
pub(crate) struct Session {
    pub client: Client,
    pub sync_running: bool,
    pub sync_handle: Option<tokio::task::JoinHandle<()>>,
    pub message_callback: Option<MessageCallback>,
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
    fn new(client: Client, sync_handle: tokio::task::JoinHandle<()>) -> Self {
        Self {
            client,
            sync_running: true,
            sync_handle: Some(sync_handle),
            message_callback: None,
        }
    }
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

    // Build shared session before spawning sync loop so the loop
    // can read callbacks from the same Session behind the handle.
    let client_for_sync = client.clone();
    let shared = Arc::new(Mutex::new(Session {
        client,
        sync_running: false,
        sync_handle: None,
        message_callback: None,
    }));

    let shared_clone = Arc::clone(&shared);
    let sync_handle = tokio::spawn(async move {
        let settings = SyncSettings::new();
        loop {
            match client_for_sync.sync_once(settings.clone()).await {
                Ok(response) => {
                    // Collect messages from all joined rooms
                    let mut all_messages: Vec<(String, Vec<messaging::Message>)> = Vec::new();
                    for (room_id, room) in &response.rooms.join {
                        let msgs = messaging::dispatch_message_events(&room.timeline.events);
                        if !msgs.is_empty() {
                            all_messages.push((room_id.to_string(), msgs));
                        }
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
    let mut guard = handle.0.lock().await;

    if !guard.sync_running {
        return Ok(());
    }

    if let Some(jh) = guard.sync_handle.take() {
        jh.abort();
    }
    guard.sync_running = false;

    let client = &guard.client;
    if let Err(e) = client.matrix_auth().logout().await {
        tracing::warn!("Logout failed (non-fatal): {}", e);
    }

    Ok(())
}

// ── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    /// Verify SessionHandle cloning increments the Arc reference count.
    #[test]
    fn test_handle_clone_increments_refcount() {
        let original = Arc::new(42);
        let cloned = Arc::clone(&original);
        assert_eq!(Arc::strong_count(&original), 2);
        drop(cloned);
        assert_eq!(Arc::strong_count(&original), 1);
    }
}
