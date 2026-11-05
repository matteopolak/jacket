//! Decoding helpers and error replies.
use super::methods::{ErrorCode, Failure};
use super::params::{DocumentParams, PositionParams};
use crate::{DocumentUri, Error, Position};
use lsp_types::notification::Notification as _;
use serde_json::{Value, json};

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
    if !is_file_uri(p.text_document.uri.as_str()) {
        return Ok(None);
    }
    uri(p.text_document.uri.as_str()).map(Some)
}

/// The document of a `{ textDocument }` request.
pub(super) fn document(params: Value) -> Result<DocumentUri, Failure> {
    let p: DocumentParams = decode(params)?;
    uri(p.text_document.uri.as_str())
}

/// The document and position of a `{ textDocument, position }` request.
pub(super) fn at(params: Value) -> Result<(DocumentUri, Position), Failure> {
    let p: PositionParams = decode(params)?;
    Ok((uri(p.text_document.uri.as_str())?, p.position))
}

pub(super) fn array<I: IntoIterator>(items: I, wire: impl FnMut(I::Item) -> Value) -> Value {
    Value::Array(items.into_iter().map(wire).collect())
}

pub(super) fn fail(code: ErrorCode, message: impl Into<String>) -> Failure {
    Failure {
        code: code as i32,
        message: message.into(),
        data: None,
    }
}

pub(super) fn internal(error: serde_json::Error) -> Failure {
    fail(ErrorCode::InternalError, error.to_string())
}

pub(super) fn domain(error: Error) -> Failure {
    let code = match error {
        Error::Limit(_) => ErrorCode::RequestFailed,
        Error::StaleVersion => ErrorCode::ContentModified,
        Error::DiagnosticsUnavailable => ErrorCode::ServerCancelled,
        _ => ErrorCode::InvalidParams,
    };
    let mut result = fail(code, error.to_string());
    if matches!(error, Error::DiagnosticsUnavailable) {
        result.data = Some(super::wire::standard_value(
            lsp_types::DiagnosticServerCancellationData {
                retrigger_request: false,
            },
        ));
    }
    result
}


pub(super) fn log_error(message: &str) -> Value {
    json!({
        "jsonrpc": "2.0",
        "method": lsp_types::notification::LogMessage::METHOD,
        "params": lsp_types::LogMessageParams { typ: lsp_types::MessageType::ERROR, message: message.into() },
    })
}
