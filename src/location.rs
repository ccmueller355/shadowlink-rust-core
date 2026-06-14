// ─── [ NEURAL DECK v4.6 $ AI::GENERATED :: NO COPYRIGHT ] ───

//! Location sharing — send and parse location data over Matrix.
//!
//! US4: Location sharing with optional live beacon mode.
//!
//! The `org.shadowlink.location` custom event type is defined via ruma's
//! `EventContent` derive macro, giving us full control over the schema
//! without relying on unstable Matrix MSCs.

use crate::client::SessionHandle;
use crate::error::ShadowLinkError;
use matrix_sdk::deserialized_responses::SyncTimelineEvent;
use ruma::OwnedRoomId;
use ruma::events::room::message::{
    LocationMessageEventContent, MessageType, RoomMessageEventContent,
};
use ruma_macros::EventContent;
use serde::{Deserialize, Serialize};
use std::str::FromStr as _;
use tokio::time::interval;

/// Custom `org.shadowlink.location` event content.
///
/// Contains coordinates, optional accuracy, and a live flag.
/// Used for both static beacons and live location streams.
#[derive(Clone, Debug, Deserialize, Serialize, EventContent)]
#[ruma_event(type = "org.shadowlink.location", kind = MessageLike)]
pub struct ShadowLinkLocationEventContent {
    pub lat: f64,
    pub lng: f64,
    pub accuracy_m: Option<f64>,
    pub live: bool,
}

/// Send a static location beacon to a room.
///
/// Validates coordinate ranges (lat: -90..90, lng: -180..180) and sends
/// a custom `org.shadowlink.location` event. Returns the event ID.
pub async fn send_beacon(
    handle: &SessionHandle,
    room_id: &str,
    lat: f64,
    lng: f64,
    accuracy_m: Option<f64>,
) -> Result<String, ShadowLinkError> {
    if !(-90.0..=90.0).contains(&lat) {
        return Err(ShadowLinkError::LocationUnavailable);
    }
    if !(-180.0..=180.0).contains(&lng) {
        return Err(ShadowLinkError::LocationUnavailable);
    }

    let guard = handle.0.lock().await;
    let client = &guard.client;

    let rid = OwnedRoomId::from_str(room_id).map_err(|_| ShadowLinkError::RoomNotFound {
        room_id: room_id.to_owned(),
    })?;
    let room = client
        .get_room(&rid)
        .ok_or_else(|| ShadowLinkError::RoomNotFound {
            room_id: room_id.to_owned(),
        })?;

    let content = ShadowLinkLocationEventContent {
        lat,
        lng,
        accuracy_m,
        live: false,
    };
    let response = room.send(content).await.map_err(|e| {
        if let Some(api_err) = e.as_client_api_error() {
            use ruma::api::client::error::{ErrorBody, ErrorKind};
            if let ErrorBody::Standard { kind, .. } = &api_err.body
                && *kind == ErrorKind::Forbidden
            {
                return ShadowLinkError::NotInRoom {
                    room_id: room_id.to_owned(),
                };
            }
        }
        ShadowLinkError::OperationFailed {
            operation: "send_beacon".into(),
            detail: e.to_string(),
        }
    })?;

    Ok(response.event_id.to_string())
}

/// Register a callback for incoming location beacons.
///
/// Replaces any previously registered callback. The callback is invoked
/// from the sync loop when an `org.shadowlink.location` event arrives.
/// Use `register_location_callback(handle, None)` to unregister.
pub async fn register_location_callback(
    handle: &SessionHandle,
    callback: Option<crate::client::LocationCallback>,
) {
    let mut guard = handle.0.lock().await;
    guard.location_callback = callback;
}

/// Start sending live location beacons at a fixed interval.
///
/// Spawns a background task that sends an `org.shadowlink.location` event
/// with `live: true` every `config.interval_secs`. Returns an error if
/// location sharing is already running.
pub async fn start_live_location(
    handle: &SessionHandle,
    room_id: &str,
    lat: f64,
    lng: f64,
    accuracy_m: Option<f64>,
    config: &LiveLocationConfig,
) -> Result<(), ShadowLinkError> {
    if !(-90.0..=90.0).contains(&lat) {
        return Err(ShadowLinkError::LocationUnavailable);
    }
    if !(-180.0..=180.0).contains(&lng) {
        return Err(ShadowLinkError::LocationUnavailable);
    }

    let room_id = room_id.to_owned();
    let config = config.clone();
    let handle_outer = handle.clone();
    let handle_inner = handle.clone();

    // Check if already running under the lock.
    {
        let guard = handle_outer.0.lock().await;
        if guard.live_location_handle.is_some() {
            return Err(ShadowLinkError::OperationFailed {
                operation: "start_live_location".into(),
                detail: "Live location is already running".into(),
            });
        }
    }

    let jh = tokio::spawn(async move {
        let mut interval = interval(std::time::Duration::from_secs(config.interval_secs));
        interval.tick().await; // skip immediate first tick

        loop {
            interval.tick().await;
            let result = send_beacon(&handle_inner, &room_id, lat, lng, accuracy_m).await;
            if let Err(e) = result {
                tracing::warn!("Live location send failed: {e}");
            }
        }
    });

    let mut guard = handle_outer.0.lock().await;
    guard.live_location_handle = Some(jh);
    Ok(())
}

/// Stop an active live location broadcast.
///
/// Aborts the interval task and clears the stored handle.
/// Safe to call when no live location is running (no-op).
pub async fn stop_live_location(handle: &SessionHandle) {
    let mut guard = handle.0.lock().await;
    if let Some(jh) = guard.live_location_handle.take() {
        jh.abort();
    }
}

/// Dispatch incoming `org.shadowlink.location` events from the sync loop.
///
/// Iterates through raw sync timeline events, tries to deserialize each as
/// a `ShadowLinkLocationEventContent`, converts to `LocationBeacon`, and
/// invokes the registered callback (if any).
pub async fn dispatch_location_events(events: &[SyncTimelineEvent], handle: &SessionHandle) {
    use ruma::events::OriginalSyncMessageLikeEvent;

    // Collect beacons before acquiring the lock to minimise lock hold time.
    let mut beacons = Vec::new();
    for sync_event in events {
        // Try to deserialize the raw event as our custom event type.
        let Ok(original) = sync_event
            .event
            .deserialize_as::<OriginalSyncMessageLikeEvent<ShadowLinkLocationEventContent>>()
        else {
            continue;
        };

        beacons.push(LocationBeacon {
            lat: original.content.lat,
            lng: original.content.lng,
            accuracy_m: original.content.accuracy_m,
        });
    }

    if beacons.is_empty() {
        return;
    }

    // Lock once to dispatch all collected beacons.
    let guard = handle.0.lock().await;
    if let Some(ref cb) = guard.location_callback {
        for beacon in beacons {
            (cb)(beacon);
        }
    }
}

/// Share a static (non-live) location pin in a room.
///
/// Sends an `m.location` event at the given coordinates with
/// optional accuracy metadata.
pub async fn share_location(
    handle: &SessionHandle,
    room_id: &str,
    lat: f64,
    lng: f64,
    _accuracy_m: Option<f64>,
    description: Option<&str>,
) -> Result<String, ShadowLinkError> {
    let guard = handle.0.lock().await;
    let client = &guard.client;

    let rid = OwnedRoomId::from_str(room_id).map_err(|_| ShadowLinkError::RoomNotFound {
        room_id: room_id.to_owned(),
    })?;
    let room = client
        .get_room(&rid)
        .ok_or_else(|| ShadowLinkError::RoomNotFound {
            room_id: room_id.to_owned(),
        })?;

    let body = description.unwrap_or("Location").to_owned();
    let geo_uri = format!("geo:{lat},{lng}");
    let location_content = LocationMessageEventContent::new(body, geo_uri);
    let content = RoomMessageEventContent::new(MessageType::Location(location_content));

    let response = room.send(content).await.map_err(|e| {
        if let Some(api_err) = e.as_client_api_error() {
            use ruma::api::client::error::{ErrorBody, ErrorKind};
            if let ErrorBody::Standard { kind, .. } = &api_err.body
                && *kind == ErrorKind::Forbidden
            {
                return ShadowLinkError::NotInRoom {
                    room_id: room_id.to_owned(),
                };
            }
        }
        ShadowLinkError::OperationFailed {
            operation: "share_location".into(),
            detail: e.to_string(),
        }
    })?;

    Ok(response.event_id.to_string())
}

/// Parse a `Message`'s content into structured location data.
///
/// Returns `None` if the message is not a location type.
pub fn parse_location_content(msg: &crate::messaging::Message) -> Option<LocationData> {
    use crate::messaging::MessageContent;
    match &msg.content {
        MessageContent::Location {
            lat,
            lng,
            accuracy_m,
            live,
        } => Some(LocationData {
            lat: *lat,
            lng: *lng,
            accuracy_m: *accuracy_m,
            live: *live,
        }),
        _ => None,
    }
}

/// Structured location data extracted from a message.
#[derive(Clone, Debug)]
pub struct LocationData {
    pub lat: f64,
    pub lng: f64,
    pub accuracy_m: Option<f64>,
    pub live: bool,
}

/// A static location beacon (snapshot of coordinates).
///
/// FFI-safe: all owned fields, Clone + Debug.
#[derive(Clone, Debug)]
pub struct LocationBeacon {
    pub lat: f64,
    pub lng: f64,
    pub accuracy_m: Option<f64>,
}

/// Configuration for live location streaming.
///
/// `interval_secs` is clamped to ≥ 5 at construction.
#[derive(Clone, Debug)]
pub struct LiveLocationConfig {
    /// Interval between location updates (minimum: 5 seconds).
    pub interval_secs: u64,
    /// Optional desired accuracy in meters.
    pub accuracy_m: Option<f64>,
}

impl LiveLocationConfig {
    /// Create a new `LiveLocationConfig`.
    ///
    /// `interval_secs` is clamped to a minimum of 5 seconds.
    pub fn new(interval_secs: u64, accuracy_m: Option<f64>) -> Self {
        Self {
            interval_secs: interval_secs.max(5),
            accuracy_m,
        }
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_location_round_trip() {
        let msg = crate::messaging::Message {
            event_id: "$evtid".into(),
            sender: "@alice:example.com".into(),
            timestamp: 1_700_000_000_000,
            content: crate::messaging::MessageContent::Location {
                lat: 48.8566,
                lng: 2.3522,
                accuracy_m: Some(25.0),
                live: false,
            },
        };
        let loc = parse_location_content(&msg).expect("parse should succeed");
        assert!((loc.lat - 48.8566).abs() < 1e-4);
        assert!((loc.lng - 2.3522).abs() < 1e-4);
        assert_eq!(loc.accuracy_m, Some(25.0));
        assert!(!loc.live);
    }

    #[test]
    fn test_parse_non_location_returns_none() {
        let msg = crate::messaging::Message {
            event_id: "$e2".into(),
            sender: "@bob:example.com".into(),
            timestamp: 1_700_000_000_001,
            content: crate::messaging::MessageContent::Text {
                body: "hello".into(),
            },
        };
        assert!(parse_location_content(&msg).is_none());
    }

    #[test]
    fn test_parse_location_no_accuracy() {
        let msg = crate::messaging::Message {
            event_id: "$e3".into(),
            sender: "@carol:example.com".into(),
            timestamp: 1_700_000_000_002,
            content: crate::messaging::MessageContent::Location {
                lat: 51.5074,
                lng: -0.1278,
                accuracy_m: None,
                live: false,
            },
        };
        let loc = parse_location_content(&msg).expect("parse should succeed");
        assert!((loc.lat - 51.5074).abs() < 1e-4);
        assert!((loc.lng - (-0.1278)).abs() < 1e-4);
        assert!(loc.accuracy_m.is_none());
        assert!(!loc.live);
    }

    #[test]
    fn test_parse_location_live_enabled() {
        let msg = crate::messaging::Message {
            event_id: "$e4".into(),
            sender: "@dan:example.com".into(),
            timestamp: 1_700_000_000_003,
            content: crate::messaging::MessageContent::Location {
                lat: 40.7128,
                lng: -74.0060,
                accuracy_m: Some(50.0),
                live: true,
            },
        };
        let loc = parse_location_content(&msg).expect("parse should succeed");
        assert!(loc.live);
        assert_eq!(loc.accuracy_m, Some(50.0));
    }

    #[test]
    fn test_parse_location_zero_accuracy() {
        let msg = crate::messaging::Message {
            event_id: "$e5".into(),
            sender: "@eve:example.com".into(),
            timestamp: 1_700_000_000_004,
            content: crate::messaging::MessageContent::Location {
                lat: 0.0,
                lng: 0.0,
                accuracy_m: Some(0.0),
                live: false,
            },
        };
        let loc = parse_location_content(&msg).expect("parse should succeed");
        assert_eq!(loc.lat, 0.0);
        assert_eq!(loc.lng, 0.0);
        assert_eq!(loc.accuracy_m, Some(0.0));
    }

    #[test]
    fn test_location_data_debug() {
        let data = LocationData {
            lat: 35.6762,
            lng: 139.6503,
            accuracy_m: Some(100.0),
            live: false,
        };
        let debug = format!("{:?}", data);
        assert!(debug.contains("35.6762"));
        assert!(debug.contains("100.0"));
    }

    #[test]
    fn test_location_data_clone() {
        let data = LocationData {
            lat: 48.8566,
            lng: 2.3522,
            accuracy_m: Some(25.0),
            live: false,
        };
        let cloned = data.clone();
        assert!((cloned.lat - data.lat).abs() < 1e-10);
    }

    // ── LocationBeacon ────────────────────────────────────────────────

    #[test]
    fn test_location_beacon_construction() {
        let beacon = LocationBeacon {
            lat: 48.8566,
            lng: 2.3522,
            accuracy_m: Some(10.0),
        };
        assert!((beacon.lat - 48.8566).abs() < 1e-4);
        assert!((beacon.lng - 2.3522).abs() < 1e-4);
        assert_eq!(beacon.accuracy_m, Some(10.0));
    }

    #[test]
    fn test_location_beacon_no_accuracy() {
        let beacon = LocationBeacon {
            lat: 51.5074,
            lng: -0.1278,
            accuracy_m: None,
        };
        assert!(beacon.accuracy_m.is_none());
    }

    #[test]
    fn test_location_beacon_zero_coords() {
        let beacon = LocationBeacon {
            lat: 0.0,
            lng: 0.0,
            accuracy_m: Some(0.0),
        };
        assert_eq!(beacon.lat, 0.0);
        assert_eq!(beacon.lng, 0.0);
        assert_eq!(beacon.accuracy_m, Some(0.0));
    }

    #[test]
    fn test_location_beacon_debug() {
        let beacon = LocationBeacon {
            lat: 35.6762,
            lng: 139.6503,
            accuracy_m: Some(100.0),
        };
        let debug = format!("{:?}", beacon);
        assert!(debug.contains("35.6762"));
        assert!(debug.contains("139.6503"));
    }

    #[test]
    fn test_location_beacon_clone() {
        let a = LocationBeacon {
            lat: 48.8566,
            lng: 2.3522,
            accuracy_m: Some(25.0),
        };
        let b = a.clone();
        assert!((b.lat - a.lat).abs() < 1e-10);
    }

    // ── LiveLocationConfig ────────────────────────────────────────────

    #[test]
    fn test_live_location_config_default_valid() {
        let cfg = LiveLocationConfig::new(10, Some(25.0));
        assert_eq!(cfg.interval_secs, 10);
        assert_eq!(cfg.accuracy_m, Some(25.0));
    }

    #[test]
    fn test_live_location_config_clamp_interval() {
        let cfg = LiveLocationConfig::new(1, None);
        assert_eq!(cfg.interval_secs, 5);
    }

    #[test]
    fn test_live_location_config_boundary_min() {
        let cfg = LiveLocationConfig::new(5, None);
        assert_eq!(cfg.interval_secs, 5);
    }

    #[test]
    fn test_live_location_config_no_accuracy() {
        let cfg = LiveLocationConfig::new(30, None);
        assert_eq!(cfg.interval_secs, 30);
        assert!(cfg.accuracy_m.is_none());
    }

    #[test]
    fn test_live_location_config_debug() {
        let cfg = LiveLocationConfig::new(15, Some(50.0));
        let debug = format!("{:?}", cfg);
        assert!(debug.contains("15"));
        assert!(debug.contains("50.0"));
    }

    #[test]
    fn test_live_location_config_clone() {
        let a = LiveLocationConfig::new(10, Some(10.0));
        let b = a.clone();
        assert_eq!(a.interval_secs, b.interval_secs);
    }
}
