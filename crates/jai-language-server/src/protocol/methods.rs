//! Method tables for requests, notifications and server commands, and message envelopes.
use super::RequestId;
use super::helpers::{failure, valid_id};
use super::params::Envelope;
use serde_json::Value;

/// JSON-RPC and LSP error codes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ErrorCode {
    ParseError = -32700,
    InvalidRequest = -32600,
    MethodNotFound = -32601,
    InvalidParams = -32602,
    InternalError = -32603,
    ServerNotInitialized = -32002,
    RequestCancelled = -32800,
    ContentModified = -32801,
    RequestFailed = -32803,
}

pub(super) type Failure = (ErrorCode, String);

pub(super) const OUTPUT_LIMIT: &str = "The response is larger than the output limit";

/// The answer to a request: a value, or JSON text already written (a large answer built as a
/// `Value` tree costs tens of times its size in memory).
pub(super) enum Reply {
    Value(Value),
    Raw(String),
}

/// The requests the server answers.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Request {
    Initialize,
    Shutdown,
    SemanticTokens,
    DocumentSymbol,
    Hover,
    Completion,
    CompletionResolve,
    Definition,
    TypeDefinition,
    References,
    PrepareRename,
    Rename,
    DocumentHighlight,
    WorkspaceSymbol,
    DocumentLink,
    FoldingRange,
    InlayHint,
    SignatureHelp,
    CodeAction,
    PrepareCallHierarchy,
    IncomingCalls,
    OutgoingCalls,
    SelectionRange,
    CodeLens,
    ExecuteCommand,
    DocumentDiagnostic,
    WorkspaceDiagnostic,
    /// Non-standard: the generated code of the `#insert`, `#run`, `#if` or macro call at a
    /// position, as a read-only `jai-expansion:` document.
    Expansion,
    /// Non-standard: the text of a file the client has not opened.
    Source,
}

impl Request {
    pub(super) const METHODS: &[(&str, Self)] = &[
        ("initialize", Self::Initialize),
        ("shutdown", Self::Shutdown),
        ("textDocument/semanticTokens/full", Self::SemanticTokens),
        ("textDocument/documentSymbol", Self::DocumentSymbol),
        ("textDocument/hover", Self::Hover),
        ("textDocument/completion", Self::Completion),
        ("completionItem/resolve", Self::CompletionResolve),
        ("textDocument/definition", Self::Definition),
        ("textDocument/typeDefinition", Self::TypeDefinition),
        ("textDocument/references", Self::References),
        ("textDocument/prepareRename", Self::PrepareRename),
        ("textDocument/rename", Self::Rename),
        ("textDocument/documentHighlight", Self::DocumentHighlight),
        ("workspace/symbol", Self::WorkspaceSymbol),
        ("textDocument/documentLink", Self::DocumentLink),
        ("textDocument/foldingRange", Self::FoldingRange),
        ("textDocument/inlayHint", Self::InlayHint),
        ("textDocument/signatureHelp", Self::SignatureHelp),
        ("textDocument/codeAction", Self::CodeAction),
        (
            "textDocument/prepareCallHierarchy",
            Self::PrepareCallHierarchy,
        ),
        ("callHierarchy/incomingCalls", Self::IncomingCalls),
        ("callHierarchy/outgoingCalls", Self::OutgoingCalls),
        ("textDocument/selectionRange", Self::SelectionRange),
        ("textDocument/codeLens", Self::CodeLens),
        ("workspace/executeCommand", Self::ExecuteCommand),
        ("textDocument/diagnostic", Self::DocumentDiagnostic),
        ("workspace/diagnostic", Self::WorkspaceDiagnostic),
        ("jai/expansion", Self::Expansion),
        ("jai/source", Self::Source),
    ];

    pub(super) fn parse(method: &str) -> Option<Self> {
        Self::METHODS
            .iter()
            .find(|(name, _)| *name == method)
            .map(|(_, request)| *request)
    }

    /// Answered only for a client that pulls diagnostics.
    pub(super) fn pulls_diagnostics(self) -> bool {
        matches!(self, Self::DocumentDiagnostic | Self::WorkspaceDiagnostic)
    }
}

/// The notifications the server acts on; any other is ignored.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Notification {
    Initialized,
    Exit,
    Cancel,
    DidOpen,
    DidChange,
    DidClose,
    DidChangeConfiguration,
    DidChangeWorkspaceFolders,
    DidChangeWatchedFiles,
    /// The client's answer to workspace trust: `{ "trusted": true }` allows the workspace's
    /// compile-time code to run, `false` withholds it.
    SetTrusted,
}

impl Notification {
    pub(super) const METHODS: &[(&str, Self)] = &[
        ("initialized", Self::Initialized),
        ("exit", Self::Exit),
        ("$/cancelRequest", Self::Cancel),
        ("textDocument/didOpen", Self::DidOpen),
        ("textDocument/didChange", Self::DidChange),
        ("textDocument/didClose", Self::DidClose),
        (
            "workspace/didChangeConfiguration",
            Self::DidChangeConfiguration,
        ),
        (
            "workspace/didChangeWorkspaceFolders",
            Self::DidChangeWorkspaceFolders,
        ),
        (
            "workspace/didChangeWatchedFiles",
            Self::DidChangeWatchedFiles,
        ),
        ("jai/setTrusted", Self::SetTrusted),
    ];

    pub(super) fn parse(method: &str) -> Option<Self> {
        Self::METHODS
            .iter()
            .find(|(name, _)| *name == method)
            .map(|(_, notification)| *notification)
    }
}

/// The commands `workspace/executeCommand` runs.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ServerCommand {
    ShowExpansion,
    ShowPolymorphs,
}

impl ServerCommand {
    pub(super) const NAMES: &[(&str, Self)] = &[
        ("jai.showExpansion", Self::ShowExpansion),
        ("jai.showPolymorphs", Self::ShowPolymorphs),
    ];

    pub(super) fn parse(name: &str) -> Option<Self> {
        Self::NAMES
            .iter()
            .find(|(known, _)| *known == name)
            .map(|(_, command)| *command)
    }
}

/// A well-formed request (with an id) or notification.
pub(super) struct Incoming {
    pub(super) id: Option<RequestId>,
    pub(super) method: String,
    pub(super) params: Value,
}

/// The message in `input`; `None` for the client's answer to a request of ours, and the error
/// response for a malformed one.
pub(super) fn incoming(input: &str) -> Result<Option<Incoming>, Value> {
    let invalid = |message| failure(Value::Null, ErrorCode::InvalidRequest, message);
    let value: Value = serde_json::from_str(input)
        .map_err(|_| failure(Value::Null, ErrorCode::ParseError, "Parse error"))?;
    let answer = value.get("result").is_some() || value.get("error").is_some();
    if value.get("method").is_none() && value.get("id").is_some() && answer {
        return Ok(None);
    }
    let id = match value.get("id") {
        None => None,
        Some(id) => match serde_json::from_value::<RequestId>(id.clone()) {
            Ok(id) if valid_id(&id) => Some(id),
            _ => return Err(invalid("Invalid request id")),
        },
    };
    match serde_json::from_value::<Envelope>(value) {
        Ok(e) if e.jsonrpc == "2.0" && !e.method.is_empty() => Ok(Some(Incoming {
            id,
            method: e.method,
            params: e.params,
        })),
        _ => Err(invalid("Invalid request")),
    }
}

/// Debug builds panic on `jai/debugPanic`, to test that a panic is one message's error.
pub(super) fn debug_panic(method: &str) {
    #[cfg(debug_assertions)]
    if method == "jai/debugPanic" {
        panic!("requested by jai/debugPanic");
    }
    #[cfg(not(debug_assertions))]
    let _ = method;
}
