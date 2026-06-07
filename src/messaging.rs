// ─── [ NEURAL DECK v4.6 $ AI::GENERATED :: NO COPYRIGHT ] ───

//! Messaging operations — send, receive, history, and callbacks.
//!
//! US3: E2EE messaging & media over the Matrix client-server API.

use crate::client::SessionHandle;
use crate::error::ShadowLinkError;
use matrix_sdk::deserialized_responses::SyncTimelineEvent;
use matrix_sdk::room::MessagesOptions;
use mime::Mime;
use ruma::api::Direction;
use ruma::events::room::message::{MessageType, RoomMessageEventContent};
use ruma::{
    OwnedRoomId,
    events::{AnySyncTimelineEvent, room::MediaSource},
    serde::Raw,
};
use std::str::FromStr as _;
use std::sync::Arc;

/// Shared callback type for incoming messages.
pub type MessageCallback = Arc<dyn Fn(Message) + Send + Sync>;

/// Content variant of a received message (FFI-safe).
#[derive(Clone, Debug)]
pub enum MessageContent {
    /// Plain text body.
    Text { body: String },
    /// Media attachment (image, audio, video, file).
    Media {
        mime_type: String,
        uri: String,
        filename: String,
        size_bytes: u64,
    },
    /// Static or live location pin.
    Location {
        lat: f64,
        lng: f64,
        accuracy_m: Option<f64>,
        live: bool,
    },
}

/// A received or fetched message (FFI-safe).
#[derive(Clone, Debug)]
pub struct Message {
    pub event_id: String,
    pub sender: String,
    pub timestamp: i64,
    pub content: MessageContent,
}

// ── Public API ────────────────────────────────────────────────────────────

/// Send a plain-text message to a room.
///
/// Returns the event ID of the sent message.
pub async fn send_text(
    handle: &SessionHandle,
    room_id: &str,
    body: &str,
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

    let content = RoomMessageEventContent::text_plain(body);
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
            operation: "send_text".into(),
            detail: e.to_string(),
        }
    })?;

    Ok(response.event_id.to_string())
}

/// Send a media attachment (image, file, etc.) to a room.
///
/// Returns the event ID of the sent media event.
pub async fn send_media(
    handle: &SessionHandle,
    room_id: &str,
    data: Vec<u8>,
    mime_type: &str,
    filename: &str,
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

    let mime: Mime = mime_type
        .parse()
        .map_err(|_| ShadowLinkError::OperationFailed {
            operation: "send_media".into(),
            detail: format!("Invalid MIME type: {mime_type}"),
        })?;

    let config = matrix_sdk::attachment::AttachmentConfig::new();
    let data_len = data.len();
    let response = room
        .send_attachment(filename, &mime, data, config)
        .await
        .map_err(|e| {
            if let Some(api_err) = e.as_client_api_error() {
                use ruma::api::client::error::{ErrorBody, ErrorKind};
                if let ErrorBody::Standard { kind, .. } = &api_err.body {
                    if *kind == ErrorKind::Forbidden {
                        return ShadowLinkError::NotInRoom {
                            room_id: room_id.to_owned(),
                        };
                    }
                    if *kind == ErrorKind::TooLarge {
                        return ShadowLinkError::MediaTooLarge {
                            size_bytes: data_len as u64,
                            limit_bytes: 0,
                        };
                    }
                }
            }
            ShadowLinkError::OperationFailed {
                operation: "send_media".into(),
                detail: e.to_string(),
            }
        })?;

    Ok(response.event_id.to_string())
}

/// Fetch message history for a room with a maximum limit.
///
/// Returns messages in reverse-chronological order (newest first).
pub async fn get_history(
    handle: &SessionHandle,
    room_id: &str,
    limit: u32,
) -> Result<Vec<Message>, ShadowLinkError> {
    use ruma::events::AnyTimelineEvent;

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

    let mut opts = MessagesOptions::new(Direction::Backward);
    opts.limit =
        ruma::UInt::try_from(limit as u64).map_err(|_| ShadowLinkError::OperationFailed {
            operation: "get_history".into(),
            detail: "Invalid limit value".into(),
        })?;
    let messages = room
        .messages(opts)
        .await
        .map_err(|e| ShadowLinkError::OperationFailed {
            operation: "get_history".into(),
            detail: e.to_string(),
        })?;

    let result: Vec<Message> = messages
        .chunk
        .iter()
        .filter_map(|tlev| {
            let raw: &Raw<AnyTimelineEvent> = &tlev.event;
            let event: AnyTimelineEvent = raw.deserialize().ok()?;
            extract_message_from_timeline(&event)
        })
        .collect();

    Ok(result)
}

/// Register an optional callback for incoming message events.
///
/// The callback fires on every new message received via sync for any
/// joined room. Pass `None` to unregister.
pub fn register_message_callback(handle: &SessionHandle, callback: Option<MessageCallback>) {
    let mut guard = handle.0.blocking_lock();
    guard.message_callback = callback;
}

// ── Internal (called by sync loop in client.rs) ──────────────────────────

/// Parse sync-timeline events into `Message` structs.
pub(crate) fn dispatch_message_events(events: &[SyncTimelineEvent]) -> Vec<Message> {
    let mut messages = Vec::new();
    for sync_event in events {
        if let Ok(event) = sync_event.event.deserialize()
            && let Some(msg) = extract_message_from_sync(&event)
        {
            messages.push(msg);
        }
    }
    messages
}

// ── Helpers ───────────────────────────────────────────────────────────────

fn extract_message_from_timeline(event: &ruma::events::AnyTimelineEvent) -> Option<Message> {
    use ruma::events::AnyTimelineEvent;

    let (event_id, sender, ts, msg_content) = match event {
        AnyTimelineEvent::MessageLike(ruma::events::AnyMessageLikeEvent::RoomMessage(orig)) => {
            let o = orig.as_original()?;
            (
                o.event_id.as_str(),
                o.sender.as_str(),
                o.origin_server_ts.get(),
                &o.content,
            )
        }
        _ => return None,
    };

    Some(Message {
        event_id: event_id.to_owned(),
        sender: sender.to_owned(),
        timestamp: u64::from(ts) as i64,
        content: message_content_from_msgtype(&msg_content.msgtype),
    })
}

fn extract_message_from_sync(event: &AnySyncTimelineEvent) -> Option<Message> {
    use ruma::events::AnySyncTimelineEvent;

    let (event_id, sender, ts, msg_content) = match event {
        AnySyncTimelineEvent::MessageLike(ruma::events::AnySyncMessageLikeEvent::RoomMessage(
            orig,
        )) => {
            let o = orig.as_original()?;
            (
                o.event_id.as_str(),
                o.sender.as_str(),
                o.origin_server_ts.get(),
                &o.content,
            )
        }
        _ => return None,
    };

    Some(Message {
        event_id: event_id.to_owned(),
        sender: sender.to_owned(),
        timestamp: u64::from(ts) as i64,
        content: message_content_from_msgtype(&msg_content.msgtype),
    })
}

fn media_source_uri(source: &MediaSource) -> String {
    match source {
        MediaSource::Plain(uri) => uri.to_string(),
        MediaSource::Encrypted(file) => file.url.to_string(),
    }
}

fn message_content_from_msgtype(msgtype: &MessageType) -> MessageContent {
    match msgtype {
        MessageType::Text(txt) => MessageContent::Text {
            body: txt.body.clone(),
        },
        MessageType::Image(img) => MessageContent::Media {
            mime_type: img
                .info
                .as_ref()
                .and_then(|i| i.mimetype.clone())
                .unwrap_or_default(),
            uri: media_source_uri(&img.source),
            filename: img.body.clone(),
            size_bytes: img
                .info
                .as_ref()
                .and_then(|i| i.size)
                .map(u64::from)
                .unwrap_or(0),
        },
        MessageType::Audio(audio) => MessageContent::Media {
            mime_type: audio
                .info
                .as_ref()
                .and_then(|i| i.mimetype.clone())
                .unwrap_or_default(),
            uri: media_source_uri(&audio.source),
            filename: audio.body.clone(),
            size_bytes: audio
                .info
                .as_ref()
                .and_then(|i| i.size)
                .map(u64::from)
                .unwrap_or(0),
        },
        MessageType::Video(video) => MessageContent::Media {
            mime_type: video
                .info
                .as_ref()
                .and_then(|i| i.mimetype.clone())
                .unwrap_or_default(),
            uri: media_source_uri(&video.source),
            filename: video.body.clone(),
            size_bytes: video
                .info
                .as_ref()
                .and_then(|i| i.size)
                .map(u64::from)
                .unwrap_or(0),
        },
        MessageType::File(file) => MessageContent::Media {
            mime_type: file
                .info
                .as_ref()
                .and_then(|i| i.mimetype.clone())
                .unwrap_or_default(),
            uri: media_source_uri(&file.source),
            filename: file.body.clone(),
            size_bytes: file
                .info
                .as_ref()
                .and_then(|i| i.size)
                .map(u64::from)
                .unwrap_or(0),
        },
        MessageType::Location(loc) => {
            let (lat, lng) = parse_geo_uri(&loc.geo_uri).unwrap_or((0.0, 0.0));
            MessageContent::Location {
                lat,
                lng,
                accuracy_m: None,
                live: false,
            }
        }
        _ => MessageContent::Text {
            body: msgtype.body().to_owned(),
        },
    }
}

/// Rudimentary geo URI parser — "geo:48.8566,2.3522" → (lat, lng).
fn parse_geo_uri(uri: &str) -> Option<(f64, f64)> {
    let coords = uri.strip_prefix("geo:")?;
    let (lat_str, lng_str) = coords.split_once(',')?;
    let lat = lat_str.parse::<f64>().ok()?;
    let lng = lng_str.parse::<f64>().ok()?;
    Some((lat, lng))
}

// ── Tests ─────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_geo_uri_valid() {
        let (lat, lng) = parse_geo_uri("geo:48.8566,2.3522").unwrap();
        assert!((lat - 48.8566).abs() < 1e-4);
        assert!((lng - 2.3522).abs() < 1e-4);
    }

    #[test]
    fn test_parse_geo_uri_invalid() {
        assert!(parse_geo_uri("geo:48.8566").is_none());
        assert!(parse_geo_uri("48.8566,2.3522").is_none());
        assert!(parse_geo_uri("").is_none());
    }
}
