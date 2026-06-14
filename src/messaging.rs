// ─── [ NEURAL DECK v4.6 $ AI::GENERATED :: NO COPYRIGHT ] ───

//! Messaging operations — send, receive, history, and callbacks.
//!
//! US3: E2EE messaging & media over the Matrix client-server API.

use crate::client::SessionHandle;
use crate::error::ShadowLinkError;
use matrix_sdk::Client;
use matrix_sdk::deserialized_responses::SyncTimelineEvent;
use matrix_sdk::room::MessagesOptions;
use mime::Mime;
use ruma::api::Direction;
use ruma::events::room::message::{MessageType, RoomMessageEventContent};
use ruma::{
    OwnedRoomId,
    events::{AnySyncMessageLikeEvent, AnySyncTimelineEvent, room::MediaSource},
    serde::Raw,
};
use std::str::FromStr as _;
use std::sync::Arc;

/// Shared callback type for incoming messages.
pub type MessageCallback = Arc<dyn Fn(Message) + Send + Sync>;

/// Content variant of a received message (FFI-safe).
#[derive(Clone)]
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

// Manual Debug — omits message body text per FR-014 (no plaintext logging)
impl std::fmt::Debug for MessageContent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Text { body: _ } => f.debug_struct("Text").field("body", &"<redacted>").finish(),
            Self::Media {
                mime_type,
                uri,
                filename,
                size_bytes,
            } => f
                .debug_struct("Media")
                .field("mime_type", mime_type)
                .field("uri", uri)
                .field("filename", filename)
                .field("size_bytes", size_bytes)
                .finish(),
            Self::Location {
                lat,
                lng,
                accuracy_m,
                live,
            } => f
                .debug_struct("Location")
                .field("lat", lat)
                .field("lng", lng)
                .field("accuracy_m", accuracy_m)
                .field("live", live)
                .finish(),
        }
    }
}

/// A received or fetched message (FFI-safe).
#[derive(Clone)]
pub struct Message {
    pub event_id: String,
    pub sender: String,
    pub timestamp: i64,
    pub content: MessageContent,
}

// Manual Debug — delegates to `MessageContent::Debug` which omits body text
impl std::fmt::Debug for Message {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Message")
            .field("event_id", &self.event_id)
            .field("sender", &self.sender)
            .field("timestamp", &self.timestamp)
            .field("content", &self.content)
            .finish()
    }
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
pub async fn register_message_callback(handle: &SessionHandle, callback: Option<MessageCallback>) {
    let mut guard = handle.0.lock().await;
    guard.message_callback = callback;
}

// ── Internal (called by sync loop in client.rs) ──────────────────────────

/// Parse sync-timeline events into `Message` structs.
pub(crate) async fn dispatch_message_events(
    events: &[SyncTimelineEvent],
    client: &Client,
    room_id: &ruma::RoomId,
) -> Vec<Message> {
    eprintln!("[DISPATCH] called with {} events", events.len());
    let mut messages = Vec::new();
    for sync_event in events {
        // Try to get the decrypted event; if encrypted, attempt decryption.
        let event: Option<AnySyncTimelineEvent> = match sync_event.event.deserialize() {
            Ok(AnySyncTimelineEvent::MessageLike(AnySyncMessageLikeEvent::RoomEncrypted(_e))) => {
                tracing::debug!(
                    "dispatch: found RoomEncrypted event — attempting decrypt via room.decrypt_event()"
                );
                if let Some(room) = client.get_room(room_id) {
                    match room.decrypt_event(sync_event.event.cast_ref()).await {
                        Ok(decrypted) => {
                            tracing::debug!("dispatch: decrypt succeeded");
                            let cast: &Raw<AnySyncTimelineEvent> = decrypted.event.cast_ref();
                            cast.deserialize().ok()
                        }
                        Err(err) => {
                            tracing::warn!("dispatch: decrypt failed: {err}");
                            None
                        }
                    }
                } else {
                    tracing::warn!("dispatch: room {room_id} not found in client store");
                    None
                }
            }
            Ok(AnySyncTimelineEvent::MessageLike(AnySyncMessageLikeEvent::RoomMessage(msg))) => {
                tracing::debug!("dispatch: already-decrypted RoomMessage event");
                Some(AnySyncTimelineEvent::MessageLike(
                    AnySyncMessageLikeEvent::RoomMessage(msg),
                ))
            }
            Ok(AnySyncTimelineEvent::MessageLike(other)) => {
                tracing::debug!("dispatch: non-message event variant received");
                Some(AnySyncTimelineEvent::MessageLike(other))
            }
            Ok(other) => {
                tracing::debug!("dispatch: non-MessageLike event");
                Some(other)
            }
            Err(err) => {
                tracing::warn!("dispatch: deserialize failed: {err}");
                None
            }
        };

        if let Some(ev) = event
            && let Some(msg) = extract_message_from_sync(&ev)
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
    use ruma::events::AnyTimelineEvent;
    use ruma::events::room::message::{
        AudioMessageEventContent, EmoteMessageEventContent, FileMessageEventContent,
        ImageMessageEventContent, LocationMessageEventContent, VideoMessageEventContent,
    };
    use ruma::events::room::{
        EncryptedFileInit, ImageInfo, JsonWebKey, JsonWebKeyInit, MediaSource,
    };
    use ruma::serde::{Base64, Raw};

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

    // ── message_content_from_msgtype ─────────────────────────────────────

    #[test]
    fn test_msg_content_from_text() {
        let raw = RoomMessageEventContent::text_plain("Hello world");
        let content = message_content_from_msgtype(&raw.msgtype);
        match &content {
            MessageContent::Text { body } => assert_eq!(body, "Hello world"),
            other => panic!("Expected Text, got {other:?}"),
        }
    }

    #[test]
    fn test_msg_content_from_image() {
        let mut info = ImageInfo::new();
        info.mimetype = Some("image/jpeg".into());
        info.size = Some(1024_u32.into());
        let source = MediaSource::Plain("mxc://example.com/img".into());
        let img =
            ImageMessageEventContent::new("photo.jpg".into(), source).info(Some(Box::new(info)));
        let content = message_content_from_msgtype(&MessageType::Image(img));
        match &content {
            MessageContent::Media {
                mime_type,
                uri,
                filename,
                size_bytes,
            } => {
                assert_eq!(mime_type, "image/jpeg");
                assert_eq!(uri, "mxc://example.com/img");
                assert_eq!(filename, "photo.jpg");
                assert_eq!(*size_bytes, 1024);
            }
            other => panic!("Expected Media, got {other:?}"),
        }
    }

    #[test]
    fn test_msg_content_from_audio() {
        let mut info = ruma::events::room::message::AudioInfo::new();
        info.mimetype = Some("audio/ogg".into());
        let source = MediaSource::Plain("mxc://example.com/audio".into());
        let audio = AudioMessageEventContent::new("recording.ogg".into(), source)
            .info(Some(Box::new(info)));
        let content = message_content_from_msgtype(&MessageType::Audio(audio));
        match &content {
            MessageContent::Media {
                mime_type,
                filename,
                ..
            } => {
                assert_eq!(mime_type, "audio/ogg");
                assert_eq!(filename, "recording.ogg");
            }
            other => panic!("Expected Media, got {other:?}"),
        }
    }

    #[test]
    fn test_msg_content_from_video() {
        let mut info = ruma::events::room::message::VideoInfo::new();
        info.mimetype = Some("video/mp4".into());
        let source = MediaSource::Plain("mxc://example.com/vid".into());
        let video =
            VideoMessageEventContent::new("clip.mp4".into(), source).info(Some(Box::new(info)));
        let content = message_content_from_msgtype(&MessageType::Video(video));
        match &content {
            MessageContent::Media {
                mime_type,
                filename,
                ..
            } => {
                assert_eq!(mime_type, "video/mp4");
                assert_eq!(filename, "clip.mp4");
            }
            other => panic!("Expected Media, got {other:?}"),
        }
    }

    #[test]
    fn test_msg_content_from_file() {
        let mut info = ruma::events::room::message::FileInfo::new();
        info.mimetype = Some("application/pdf".into());
        let source = MediaSource::Plain("mxc://example.com/doc".into());
        let file =
            FileMessageEventContent::new("report.pdf".into(), source).info(Some(Box::new(info)));
        let content = message_content_from_msgtype(&MessageType::File(file));
        match &content {
            MessageContent::Media {
                mime_type,
                filename,
                ..
            } => {
                assert_eq!(mime_type, "application/pdf");
                assert_eq!(filename, "report.pdf");
            }
            other => panic!("Expected Media, got {other:?}"),
        }
    }

    #[test]
    fn test_msg_content_from_location() {
        let body = "Office".to_owned();
        let geo_uri = "geo:48.8566,2.3522".to_owned();
        let loc_content = LocationMessageEventContent::new(body, geo_uri);
        let msg = MessageType::Location(loc_content);
        let content = message_content_from_msgtype(&msg);
        match &content {
            MessageContent::Location { lat, lng, .. } => {
                assert!((*lat - 48.8566).abs() < 1e-4);
                assert!((*lng - 2.3522).abs() < 1e-4);
            }
            other => panic!("Expected Location, got {other:?}"),
        }
    }

    #[test]
    fn test_msg_content_unknown_fallback_to_text() {
        // Emote is not matched explicitly in our function,
        // so it should fall through to the catch-all Text arm.
        let emote = EmoteMessageEventContent::plain("waves hello");
        let msgtype = MessageType::Emote(emote);
        let content = message_content_from_msgtype(&msgtype);
        match &content {
            MessageContent::Text { body } => {
                assert!(body.contains("waves hello"), "body = {body}");
            }
            other => panic!("Expected Text fallback, got {other:?}"),
        }
    }

    // ── media_source_uri ─────────────────────────────────────────────

    #[test]
    fn test_media_source_uri_plain() {
        let source = MediaSource::Plain("mxc://example.com/file".into());
        assert_eq!(media_source_uri(&source), "mxc://example.com/file");
    }

    #[test]
    fn test_media_source_uri_encrypted() {
        let init = EncryptedFileInit {
            url: "mxc://example.com/enc".into(),
            key: JsonWebKey::from(JsonWebKeyInit {
                kty: "oct".into(),
                key_ops: vec!["decrypt".into()],
                alg: "A256CTR".into(),
                k: Base64::new(b"base64+key".to_vec()),
                ext: true,
            }),
            hashes: [("sha256".to_owned(), Base64::new(b"base64+hash".to_vec()))].into(),
            iv: Base64::new(b"base64+iv".to_vec()),
            v: "v2".into(),
        };
        let file: ruma::events::room::EncryptedFile = init.into();
        let source = MediaSource::Encrypted(Box::new(file));
        assert_eq!(media_source_uri(&source), "mxc://example.com/enc");
    }

    // ── extract_message_from_timeline ────────────────────────────────

    #[test]
    fn test_extract_non_message_event() {
        // Construct a non-message event (room power levels) — should return None.
        let json = serde_json::json!({
            "content": { "ban": 50, "events": {} },
            "event_id": "$nonmsg:example.com",
            "origin_server_ts": 1_700_000_000u64,
            "sender": "@admin:example.com",
            "room_id": "!roomid:example.com",
            "type": "m.room.power_levels",
            "state_key": "",
        });
        let raw: Raw<AnyTimelineEvent> = Raw::from_json_string(json.to_string()).unwrap();
        let event: AnyTimelineEvent = raw.deserialize().unwrap();
        assert!(extract_message_from_timeline(&event).is_none());
    }

    // ── dispatch_message_events ──────────────────────────────────────
    // Integration testing of dispatch_message_events with real sync responses
    // lives in tests/test_us3_messaging.rs (test_callback_registration).
}
