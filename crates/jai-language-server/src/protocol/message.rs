//! The library owns normal messages; this adapter validates reserved envelope metadata.
use lsp_server::{ErrorCode, Message, ResponseError};
use serde::de::{self, MapAccess, Visitor};
use serde::{Deserialize, Serialize};
use serde_json::value::RawValue;
use std::fmt;

/// JSON-RPC requires a null ID when an invalid request cannot be identified. The normal
/// library `Response` deliberately requires `RequestId`, so only this error edge is separate.
#[derive(Debug)]
pub(crate) struct UnidentifiedError(pub ResponseError);

impl Serialize for UnidentifiedError {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        #[derive(Serialize)]
        struct ErrorEnvelope<'a> {
            jsonrpc: &'static str,
            id: (),
            error: &'a ResponseError,
        }
        ErrorEnvelope {
            jsonrpc: "2.0",
            id: (),
            error: &self.0,
        }
        .serialize(serializer)
    }
}

/// A rejected complete body stays ordered with normal incoming messages.
#[derive(Debug)]
pub struct RejectedMessage {
    pub id: Option<lsp_server::RequestId>,
    pub error: ResponseError,
}

/// Error outputs selected by the shared semantic owner, not the reader thread.
pub enum RejectedReply {
    Response(lsp_server::Response),
    Unidentified(ResponseError),
}

/// Decode complete, size-checked input without exposing payload-bearing library errors.
/// Example: `decode(r#"{"jsonrpc":"2.0","id":1,"method":"shutdown"}"#)`.
pub(crate) fn decode(input: &str) -> Result<Option<Message>, RejectedMessage> {
    let MetadataKind::Message(id) = validate_metadata(input)? else {
        return Ok(None);
    };
    serde_json::from_str(input)
        .map(Some)
        .map_err(|_| rejected(id, ErrorCode::InvalidRequest, "Invalid request"))
}

pub(crate) fn rejected(
    id: Option<lsp_server::RequestId>,
    code: ErrorCode,
    text: &str,
) -> RejectedMessage {
    RejectedMessage {
        id,
        error: ResponseError {
            code: code as i32,
            message: text.into(),
            data: None,
        },
    }
}

/// Validate only envelope metadata; native `Message::read` remains the body/schema decoder.
pub(crate) fn validate_metadata(input: &str) -> Result<MetadataKind, RejectedMessage> {
    let metadata = serde_json::from_str::<Metadata>(input).map_err(|_| {
        if serde_json::from_str::<de::IgnoredAny>(input).is_err() {
            rejected(None, ErrorCode::ParseError, "Parse error")
        } else {
            rejected(None, ErrorCode::InvalidRequest, "Invalid request")
        }
    })?;
    if !metadata.valid {
        return Err(rejected(
            metadata.id,
            ErrorCode::InvalidRequest,
            "Invalid request",
        ));
    }
    Ok(if metadata.unidentified_response {
        MetadataKind::UnidentifiedResponse
    } else {
        MetadataKind::Message(metadata.id)
    })
}

/// Normal messages remain library-owned; a null-ID response cannot identify a pending request.
pub(crate) enum MetadataKind {
    Message(Option<lsp_server::RequestId>),
    UnidentifiedResponse,
}

struct Metadata {
    id: Option<lsp_server::RequestId>,
    valid: bool,
    unidentified_response: bool,
}

impl<'de> Deserialize<'de> for Metadata {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct Envelope;
        impl<'de> Visitor<'de> for Envelope {
            type Value = Metadata;
            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str("a JSON-RPC message object")
            }
            fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Metadata, A::Error> {
                let mut seen = 0u8;
                let mut valid = true;
                let mut id = None;
                let mut duplicate_id = false;
                let mut null_id = false;
                let mut error = None;
                while let Some(key) = map.next_key::<String>()? {
                    let bit = match key.as_str() {
                        "jsonrpc" => 1,
                        "id" => 2,
                        "method" => 4,
                        "params" => 8,
                        "result" => 16,
                        "error" => 32,
                        _ => {
                            map.next_value::<de::IgnoredAny>()?;
                            continue;
                        }
                    };
                    if seen & bit != 0 {
                        valid = false;
                        duplicate_id |= key == "id";
                    }
                    seen |= bit;
                    let raw = map.next_value::<&RawValue>()?.get();
                    valid &= match key.as_str() {
                        "jsonrpc" => {
                            serde_json::from_str::<String>(raw).ok().as_deref() == Some("2.0")
                        }
                        "id" => {
                            id = serde_json::from_str::<lsp_server::RequestId>(raw).ok();
                            null_id = raw == "null";
                            id.is_some() || null_id
                        }
                        "method" => {
                            serde_json::from_str::<String>(raw).is_ok_and(|s| !s.is_empty())
                        }
                        "params" => {
                            raw == "null" || matches!(raw.as_bytes().first(), Some(b'{' | b'['))
                        }
                        "error" => {
                            error = Some(raw);
                            true
                        }
                        _ => true,
                    };
                }
                let request = seen & 4 != 0;
                valid &= seen & 1 != 0
                    && if request {
                        seen & (16 | 32) == 0 && !null_id
                    } else {
                        seen & 2 != 0 && matches!(seen & (16 | 32), 16 | 32) && seen & 8 == 0
                    };
                if null_id && seen & 32 != 0 {
                    valid &=
                        error.is_some_and(|raw| serde_json::from_str::<ResponseError>(raw).is_ok());
                }
                if duplicate_id {
                    id = None;
                }
                Ok(Metadata {
                    id,
                    valid,
                    unidentified_response: !request && null_id,
                })
            }
        }
        deserializer.deserialize_map(Envelope)
    }
}

/// Browser JSON has no Content-Length header; use the same version wrapper as `Message::write`.
#[derive(Serialize)]
pub(crate) struct Versioned<'a> {
    pub jsonrpc: &'static str,
    #[serde(flatten)]
    pub message: &'a Message,
}

impl<'a> Versioned<'a> {
    pub fn new(message: &'a Message) -> Self {
        Self {
            jsonrpc: "2.0",
            message,
        }
    }
}

/// Count the library's versioned body without allocating an extra serialized String.
pub(crate) fn check_output(messages: &[Message], limit: usize) -> Result<(), super::ProtocolError> {
    check_output_with_lengths(messages, limit, |_| {})
}

/// Reuse the required cap walk for optional body-byte observations, without encoding again.
pub(crate) fn check_output_with_lengths(
    messages: &[Message],
    limit: usize,
    mut body: impl FnMut(usize),
) -> Result<(), super::ProtocolError> {
    struct Counter {
        bytes: usize,
        limit: usize,
    }
    impl std::io::Write for Counter {
        fn write(&mut self, data: &[u8]) -> std::io::Result<usize> {
            self.bytes = self
                .bytes
                .checked_add(data.len())
                .filter(|n| *n <= self.limit)
                .ok_or_else(|| std::io::Error::other("output limit"))?;
            Ok(data.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    let mut counter = Counter {
        bytes: 0,
        limit,
    };
    for message in messages {
        let before = counter.bytes;
        serde_json::to_writer(&mut counter, &Versioned::new(message))
            .map_err(|_| super::ProtocolError::OutputLimit)?;
        body(counter.bytes - before);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn observed_output_lengths_match_library_frames_and_the_aggregate_cap() {
        let messages: [Message; 2] = [
            lsp_server::Notification::new("test/notice".into(), serde_json::json!({"text":"😀é"}))
                .into(),
            lsp_server::Response::new_ok(
                lsp_server::RequestId::from(7),
                serde_json::json!([1, 2, 3]),
            )
            .into(),
        ];
        let mut actual = Vec::new();
        for message in &messages {
            let mut frame = Vec::new();
            message.write(&mut frame).unwrap();
            let body = frame
                .windows(4)
                .position(|bytes| bytes == b"\r\n\r\n")
                .unwrap()
                + 4;
            actual.push(frame.len() - body);
        }
        let total = actual.iter().sum();
        let mut observed = Vec::new();
        check_output_with_lengths(&messages, total, |bytes| observed.push(bytes)).unwrap();
        assert_eq!(observed, actual);
        assert!(matches!(
            check_output(&messages, total - 1),
            Err(super::super::ProtocolError::OutputLimit)
        ));
    }

    #[test]
    fn null_error_edge_and_standard_response_are_distinct() {
        let error = decode("{").unwrap_err();
        assert_eq!(
            serde_json::to_value(UnidentifiedError(error.error)).unwrap(),
            serde_json::json!({
                "jsonrpc":"2.0", "id":null, "error":{"code":-32700,"message":"Parse error"}
            })
        );
        let response = decode(r#"{"jsonrpc":"2.0","id":1,"result":null}"#).unwrap();
        assert!(matches!(response, Some(Message::Response(_))));
    }

    #[test]
    fn null_peer_response_edge_requires_a_complete_valid_response() {
        for input in [
            r#"{"jsonrpc":"2.0","id":null,"result":null}"#,
            r#"{"jsonrpc":"2.0","id":null,"error":{"code":-32700,"message":"peer"}}"#,
        ] {
            assert!(decode(input).unwrap().is_none());
        }
        for input in [
            r#"{"jsonrpc":"2.0","result":null}"#,
            r#"{"jsonrpc":"2.0","id":null,"id":null,"result":null}"#,
            r#"{"jsonrpc":"2.0","id":null,"method":"exit","result":null}"#,
            r#"{"jsonrpc":"2.0","id":null,"params":{},"result":null}"#,
            r#"{"jsonrpc":"2.0","id":null,"error":{"code":"bad","message":"peer"}}"#,
            r#"{"jsonrpc":"2.0","id":null,"error":null}"#,
            r#"{"jsonrpc":"2.0","id":null,"result":null} {}"#,
        ] {
            assert!(decode(input).is_err(), "{input}");
        }
    }

    #[test]
    fn reserved_metadata_is_checked_before_untagged_fallback() {
        for input in [
            r#"{"jsonrpc":"2.0","id":null,"method":"exit"}"#,
            r#"{"jsonrpc":"2.0","id":7,"method":"shutdown","params":123}"#,
            r#"{"jsonrpc":"2.0","id":1.0,"method":"exit"}"#,
            r#"{"jsonrpc":"2.0","id":2147483648,"method":"exit"}"#,
            r#"{"jsonrpc":"2.0","id":1,"result":null,"error":{"code":1,"message":"x"}}"#,
            r#"{"jsonrpc":"2.0","method":"exit","method":"exit"}"#,
        ] {
            assert!(decode(input).is_err(), "{input}");
        }
    }
}
