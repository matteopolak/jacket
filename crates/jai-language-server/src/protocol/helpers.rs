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
    matches!(id, RequestId::Number(_)) || matches!(id, RequestId::String(text) if text.len() <= 128)
}

pub(super) fn decode<T: serde::de::DeserializeOwned>(value: Value) -> Result<T, Failure> {
    serde_json::from_value(value)
        .map_err(|_| fail(ErrorCode::InvalidParams, "Invalid method parameters"))
}

pub(super) fn uri(text: &str) -> Result<DocumentUri, Failure> {
    DocumentUri::parse(text).map_err(domain)
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

pub(super) fn log_error(message: &str) -> Value {
    json!({
        "jsonrpc": "2.0",
        "method": "window/logMessage",
        "params": { "type": 1, "message": message },
    })
}
