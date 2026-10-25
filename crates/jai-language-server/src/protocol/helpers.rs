//! Decoding helpers and error replies.
use super::RequestId;
use super::methods::{ErrorCode, Failure};
use super::params::{DocumentParams, PositionParams};
use crate::{DocumentUri, Error, Position};
use serde_json::{Value, json};
use std::path::PathBuf;

/// The path of a `file://` URI value.
pub(super) fn folder_path(uri: &Value) -> Option<PathBuf> {
    DocumentUri::parse(uri.as_str()?)
        .ok()
        .map(|u| PathBuf::from(u.path()))
}

pub(super) fn valid_id(id: &RequestId) -> bool {
    !matches!(id, RequestId::String(text) if text.len() > 128)
}

pub(super) fn decode<T: serde::de::DeserializeOwned>(value: Value) -> Result<T, Failure> {
    serde_json::from_value(value)
        .map_err(|_| fail(ErrorCode::InvalidParams, "Invalid method parameters"))
}

pub(super) fn uri(text: &str) -> Result<DocumentUri, Failure> {
    DocumentUri::parse(text).map_err(domain)
}

/// Whether `text` names a local file; other schemes (untitled buffers, remote files) have no
/// path to analyse.
pub(super) fn is_file_uri(text: &str) -> bool {
    text.get(..5)
        .is_some_and(|scheme| scheme.eq_ignore_ascii_case("file:"))
}

/// The document of a `{ textDocument }` notification, `None` when it is not a local file.
pub(super) fn file_document(params: Value) -> Result<Option<DocumentUri>, Failure> {
    let p: DocumentParams = decode(params)?;
    if !is_file_uri(&p.text_document.uri) {
        return Ok(None);
    }
    uri(&p.text_document.uri).map(Some)
}

/// The document of a `{ textDocument }` request.
pub(super) fn document(params: Value) -> Result<DocumentUri, Failure> {
    let p: DocumentParams = decode(params)?;
    uri(&p.text_document.uri)
}

/// The document and position of a `{ textDocument, position }` request.
pub(super) fn at(params: Value) -> Result<(DocumentUri, Position), Failure> {
    let p: PositionParams = decode(params)?;
    Ok((uri(&p.text_document.uri)?, p.position))
}

pub(super) fn array<I: IntoIterator>(items: I, wire: impl FnMut(I::Item) -> Value) -> Value {
    Value::Array(items.into_iter().map(wire).collect())
}

pub(super) fn fail(code: ErrorCode, message: impl Into<String>) -> Failure {
    (code, message.into())
}

pub(super) fn internal(error: serde_json::Error) -> Failure {
    fail(ErrorCode::InternalError, error.to_string())
}

pub(super) fn domain(error: Error) -> Failure {
    let code = match error {
        Error::Limit(_) => ErrorCode::RequestFailed,
        Error::StaleVersion => ErrorCode::ContentModified,
        _ => ErrorCode::InvalidParams,
    };
    (code, error.to_string())
}

pub(super) fn failure(id: Value, code: ErrorCode, message: &str) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "error": { "code": code as i32, "message": message },
    })
}

/// The `window/showMessage` that tells the user a message of `length` bytes was over the
/// `limit` and was ignored.
pub fn message_too_large(length: usize, limit: usize) -> String {
    let mib = |n: usize| n as f64 / (1024.0 * 1024.0);
    json!({
        "jsonrpc": "2.0",
        "method": "window/showMessage",
        "params": {
            "type": 1,
            "message": format!(
                "jailsp ignored a {:.1} MiB message: the limit is {:.0} MiB. A document that large is not analysed.",
                mib(length),
                mib(limit)
            ),
        },
    })
    .to_string()
}

/// What to send for a message that was over the limit and skipped: an error response when its
/// request id could be read from the start of the body, and the notice for the user.
pub fn oversized_replies(message: &crate::framing::Oversized, limit: usize) -> Vec<String> {
    let mut replies = vec![];
    if let Some(id) = leading_id(&message.head) {
        let error = failure(
            json!(id),
            ErrorCode::RequestFailed,
            &format!(
                "The request is larger than the {} MiB message limit",
                limit / (1024 * 1024)
            ),
        );
        replies.push(error.to_string());
    }
    replies.push(message_too_large(message.length, limit));
    replies
}

/// The `id` member of the JSON object that `head` starts, if it comes within `head`.
fn leading_id(head: &[u8]) -> Option<RequestId> {
    let (mut depth, mut at) = (0usize, 0);
    while at < head.len() {
        match head[at] {
            b'{' | b'[' => depth += 1,
            b'}' | b']' => depth = depth.checked_sub(1)?,
            b'"' => {
                let start = at + 1;
                at = start;
                while at < head.len() && head[at] != b'"' {
                    at += if head[at] == b'\\' {
                        2
                    } else {
                        1
                    };
                }
                let key = head.get(start..at)?;
                let after = head.get(at + 1..)?;
                let after_colon = after
                    .iter()
                    .position(|b| !b.is_ascii_whitespace())
                    .filter(|&n| after[n] == b':');
                if depth == 1
                    && key == b"id"
                    && let Some(n) = after_colon
                {
                    let mut values = serde_json::Deserializer::from_slice(&after[n + 1..])
                        .into_iter::<RequestId>();
                    return values.next()?.ok();
                }
            }
            _ => {}
        }
        at += 1;
    }
    None
}

pub(super) fn log_error(message: &str) -> Value {
    json!({
        "jsonrpc": "2.0",
        "method": "window/logMessage",
        "params": { "type": 1, "message": message },
    })
}
