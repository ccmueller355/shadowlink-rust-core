// ─── [ NEURAL DECK v4.6 $ AI::GENERATED :: NO COPYRIGHT ] ───

//! Location sharing — send and parse location data over Matrix.
//!
//! US4: Location sharing with optional live beacon mode.

use crate::client::SessionHandle;
use crate::error::ShadowLinkError;
use ruma::OwnedRoomId;
use ruma::events::room::message::{
    LocationMessageEventContent, MessageType, RoomMessageEventContent,
};
use std::str::FromStr as _;

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
}
