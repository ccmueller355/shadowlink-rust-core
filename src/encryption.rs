// ─── [ NEURAL DECK v4.6 $ AI::GENERATED :: NO COPYRIGHT ] ───

//! E2EE operations — cross-signing, device verification, key backup.
//!
//! US5: End-to-end encryption key management for Matrix.

use crate::client::SessionHandle;
use crate::error::ShadowLinkError;
use ruma::{OwnedDeviceId, UserId};
use std::path::PathBuf;

/// E2EE trust state for a device (FFI-safe).
#[derive(Clone, Debug)]
pub enum DeviceTrust {
    /// Device has been explicitly verified by the user.
    Verified,
    /// Device is not trusted.
    Unverified,
    /// Device is explicitly blocked.
    Blocked,
    /// Device has been ignored (e.g., a previously unknown device).
    Ignored,
    /// Trust state is unknown (e.g., device not found).
    Unknown,
}

/// Summary of the user's cross-signing status (FFI-safe).
#[derive(Clone, Debug)]
pub struct CrossSigningStatus {
    /// Whether the user has cross-signing set up on their master key.
    pub has_master: bool,
    /// Whether the user has a self-signing key.
    pub has_self_signing: bool,
    /// Whether the user has a user-signing key.
    pub has_user_signing: bool,
}

/// A snapshot of a device and its trust state (FFI-safe).
#[derive(Clone, Debug)]
pub struct DeviceInfo {
    pub user_id: String,
    pub device_id: String,
    pub display_name: Option<String>,
    pub trust: DeviceTrust,
}

/// Fetch information about a specific device belonging to a user.
pub async fn get_device(
    handle: &SessionHandle,
    user_id: &str,
    device_id: &str,
) -> Result<DeviceInfo, ShadowLinkError> {
    let guard = handle.0.lock().await;
    let client = &guard.client;

    let uid = <&UserId>::try_from(user_id).map_err(|_| ShadowLinkError::OperationFailed {
        operation: "get_device".into(),
        detail: format!("Invalid user ID: {user_id}"),
    })?;
    let did = OwnedDeviceId::from(device_id);

    let device = client
        .encryption()
        .get_device(uid, &did)
        .await
        .map_err(|e| ShadowLinkError::OperationFailed {
            operation: "get_device".into(),
            detail: e.to_string(),
        })?
        .ok_or_else(|| ShadowLinkError::OperationFailed {
            operation: "get_device".into(),
            detail: "Device not found".into(),
        })?;

    Ok(DeviceInfo {
        user_id: device.user_id().to_string(),
        device_id: device.device_id().to_string(),
        display_name: device.display_name().map(|s| s.to_owned()),
        trust: local_trust_to_device_trust(device.local_trust_state()),
    })
}

/// Fetch the current user's own device info.
pub async fn get_own_device(handle: &SessionHandle) -> Result<DeviceInfo, ShadowLinkError> {
    let guard = handle.0.lock().await;
    let client = &guard.client;

    let device = client
        .encryption()
        .get_own_device()
        .await
        .map_err(|e| ShadowLinkError::OperationFailed {
            operation: "get_own_device".into(),
            detail: e.to_string(),
        })?
        .ok_or_else(|| ShadowLinkError::OperationFailed {
            operation: "get_own_device".into(),
            detail: "Own device not found".into(),
        })?;

    Ok(DeviceInfo {
        user_id: device.user_id().to_string(),
        device_id: device.device_id().to_string(),
        display_name: device.display_name().map(|s| s.to_owned()),
        trust: local_trust_to_device_trust(device.local_trust_state()),
    })
}

/// List all devices for a given user.
pub async fn get_user_devices(
    handle: &SessionHandle,
    user_id: &str,
) -> Result<Vec<DeviceInfo>, ShadowLinkError> {
    let guard = handle.0.lock().await;
    let client = &guard.client;

    let uid = <&UserId>::try_from(user_id).map_err(|_| ShadowLinkError::OperationFailed {
        operation: "get_user_devices".into(),
        detail: format!("Invalid user ID: {user_id}"),
    })?;

    let user_devices = client
        .encryption()
        .get_user_devices(uid)
        .await
        .map_err(|e| ShadowLinkError::OperationFailed {
            operation: "get_user_devices".into(),
            detail: e.to_string(),
        })?;

    let devices = user_devices
        .devices()
        .map(|device| DeviceInfo {
            user_id: device.user_id().to_string(),
            device_id: device.device_id().to_string(),
            display_name: device.display_name().map(|s| s.to_owned()),
            trust: local_trust_to_device_trust(device.local_trust_state()),
        })
        .collect();

    Ok(devices)
}

/// Bootstrap cross-signing for the current user.
///
/// If cross-signing is already set up and keys are available, calling this
/// is a no-op.
pub async fn bootstrap_cross_signing(handle: &SessionHandle) -> Result<(), ShadowLinkError> {
    let guard = handle.0.lock().await;
    let client = &guard.client;

    client
        .encryption()
        .bootstrap_cross_signing(None)
        .await
        .map_err(|e| ShadowLinkError::OperationFailed {
            operation: "bootstrap_cross_signing".into(),
            detail: e.to_string(),
        })
}

/// Check the current cross-signing status of the user.
pub async fn cross_signing_status(
    handle: &SessionHandle,
) -> Result<CrossSigningStatus, ShadowLinkError> {
    let guard = handle.0.lock().await;
    let client = &guard.client;

    let status = client.encryption().cross_signing_status().await;
    Ok(CrossSigningStatus {
        has_master: status.as_ref().map(|s| s.has_master).unwrap_or(false),
        has_self_signing: status.as_ref().map(|s| s.has_self_signing).unwrap_or(false),
        has_user_signing: status.as_ref().map(|s| s.has_user_signing).unwrap_or(false),
    })
}

/// Export room keys to a file (E2EE key backup).
///
/// The keys are written to the given `path` encrypted with the
/// `passphrase`. The path should be writable by the application.
pub async fn export_room_keys(
    handle: &SessionHandle,
    path: &str,
    passphrase: &str,
) -> Result<(), ShadowLinkError> {
    let guard = handle.0.lock().await;
    let client = &guard.client;

    let encryption = client.encryption();

    // Wait for E2EE to be fully initialised before exporting.
    encryption.wait_for_e2ee_initialization_tasks().await;

    let export_path = PathBuf::from(path);

    encryption
        .export_room_keys(export_path, passphrase, |_| true)
        .await
        .map_err(|e| ShadowLinkError::OperationFailed {
            operation: "export_room_keys".into(),
            detail: e.to_string(),
        })
}

/// Import room keys from a previously exported backup file.
///
/// The file at `path` should contain an encrypted room key export (as
/// produced by `export_room_keys`), and `passphrase` must match the one
/// used during export.
pub async fn import_room_keys(
    handle: &SessionHandle,
    path: &str,
    passphrase: &str,
) -> Result<(), ShadowLinkError> {
    let guard = handle.0.lock().await;
    let client = &guard.client;

    let encryption = client.encryption();
    encryption.wait_for_e2ee_initialization_tasks().await;

    let import_path = PathBuf::from(path);

    encryption
        .import_room_keys(import_path, passphrase)
        .await
        .map_err(|e| ShadowLinkError::OperationFailed {
            operation: "import_room_keys".into(),
            detail: e.to_string(),
        })?;

    Ok(())
}

/// Wait for E2EE initialisation tasks to complete.
///
/// Call this after `connect()` if you plan to immediately perform E2EE
/// operations like key export or device listing.
pub async fn wait_for_e2ee(handle: &SessionHandle) -> Result<(), ShadowLinkError> {
    let guard = handle.0.lock().await;
    let client = &guard.client;
    client
        .encryption()
        .wait_for_e2ee_initialization_tasks()
        .await;
    Ok(())
}

// ── Helpers ───────────────────────────────────────────────────────────────

fn local_trust_to_device_trust(trust: matrix_sdk::crypto::LocalTrust) -> DeviceTrust {
    use matrix_sdk::crypto::LocalTrust;
    match trust {
        LocalTrust::Verified => DeviceTrust::Verified,
        LocalTrust::BlackListed => DeviceTrust::Blocked,
        LocalTrust::Unset => DeviceTrust::Unverified,
        LocalTrust::Ignored => DeviceTrust::Ignored,
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_local_trust_conversion() {
        use matrix_sdk::crypto::LocalTrust;
        assert!(matches!(
            local_trust_to_device_trust(LocalTrust::Verified),
            DeviceTrust::Verified,
        ));
        assert!(matches!(
            local_trust_to_device_trust(LocalTrust::BlackListed),
            DeviceTrust::Blocked,
        ));
        assert!(matches!(
            local_trust_to_device_trust(LocalTrust::Unset),
            DeviceTrust::Unverified,
        ));
        assert!(matches!(
            local_trust_to_device_trust(LocalTrust::Ignored),
            DeviceTrust::Ignored,
        ));
    }
}
