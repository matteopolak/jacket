//! Standard JSON parsing belongs at this portable boundary, not in source analysis.
use crate::{
    CallHierarchyCall, CallHierarchyItem, CodeAction, CodeLens, Command, CompletionKind,
    CompletionList, Diagnostic, DiagnosticCode, DiagnosticSeverity, DocumentSymbol, DocumentUri,
    Error, Expansion, FoldingRange, Hover, InlayHint, InlayHintKind, Limits, Location, MarkupKind,
    Position, Range, SelectionRange, SemanticToken, SemanticTokenKind, Session, SignatureHelp,
    SymbolInformation, SymbolKind, TOKEN_MODIFIERS, TextChange,
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::{BTreeSet, VecDeque};
use std::path::PathBuf;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(untagged)]
pub enum RequestId {
    Number(i32),
    String(String),
}

#[derive(Clone, Debug)]
pub enum ProtocolError {
    MessageLimit,
    OutputLimit,
    Serialization(String),
}

impl std::fmt::Display for ProtocolError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MessageLimit => f.write_str("LSP message byte limit exceeded"),
            Self::OutputLimit => f.write_str("LSP response byte limit exceeded"),
            Self::Serialization(e) => f.write_str(e),
        }
    }
}

impl std::error::Error for ProtocolError {
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Lifecycle {
    New,
    Running,
    Shutdown,
    Exited(u8),
}

pub struct JsonSession {
    session: Session,
    lifecycle: Lifecycle,
    cancelled: BTreeSet<RequestId>,
    completed: VecDeque<RequestId>,
    /// Markdown when the client listed it in `textDocument.hover.contentFormat`.
    hover_kind: MarkupKind,
    /// Markdown when the client listed it in the signature help `documentationFormat`.
    signature_kind: MarkupKind,
    /// The client pulls diagnostics (`textDocument/diagnostic`): none are pushed to it.
    pull_diagnostics: bool,
    /// It can also be told to pull again (`workspace/diagnostic/refresh`).
    refresh_diagnostics: bool,
    refreshes: u64,
    /// Diagnostics are pushed by `flush_publications`, not with the answer to each edit.
    defer_publications: bool,
    /// An edit since the last flush has diagnostics to publish.
    publications_pending: bool,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct DocumentItem {
    uri: String,
    language_id: String,
    version: i32,
    text: String,
}

#[derive(Deserialize)]
struct DocumentIdentifier {
    uri: String,
}

#[derive(Deserialize)]
struct VersionedIdentifier {
    uri: String,
    version: i32,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct OpenParams {
    text_document: DocumentItem,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct DocumentParams {
    text_document: DocumentIdentifier,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ChangeParams {
    text_document: VersionedIdentifier,
    content_changes: Vec<TextChange>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct PositionParams {
    text_document: DocumentIdentifier,
    position: Position,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct SelectionParams {
    text_document: DocumentIdentifier,
    positions: Vec<Position>,
}

/// The call hierarchy item a client sends back; only where it points matters.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct HierarchyItem {
    name: String,
    #[serde(default)]
    detail: Option<String>,
    uri: String,
    range: Range,
    selection_range: Range,
}

impl HierarchyItem {
    fn into_item(self) -> CallHierarchyItem {
        CallHierarchyItem {
            name: self.name,
            detail: self.detail.unwrap_or_default(),
            kind: SymbolKind::Function,
            uri: self.uri,
            range: self.range,
            selection_range: self.selection_range,
        }
    }
}

#[derive(Deserialize)]
struct HierarchyParams {
    item: HierarchyItem,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RangeParams {
    text_document: DocumentIdentifier,
    range: Range,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct CodeActionParams {
    text_document: DocumentIdentifier,
    range: Range,
    #[serde(default)]
    context: Option<CodeActionContext>,
}

#[derive(Deserialize, Default)]
struct CodeActionContext {
    #[serde(default)]
    diagnostics: Vec<WireDiagnostic>,
    #[serde(default)]
    only: Option<Vec<String>>,
}

/// The parts of a client's diagnostic that identify a lint.
#[derive(Deserialize)]
struct WireDiagnostic {
    range: Range,
    #[serde(default)]
    code: Option<Value>,
    #[serde(default)]
    source: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ReferenceContext {
    include_declaration: bool,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ReferenceParams {
    text_document: DocumentIdentifier,
    position: Position,
    context: Option<ReferenceContext>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RenameParams {
    text_document: DocumentIdentifier,
    position: Position,
    new_name: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct DiagnosticParams {
    text_document: DocumentIdentifier,
    #[serde(default)]
    previous_result_id: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct WorkspaceDiagnosticParams {
    #[serde(default)]
    previous_result_ids: Vec<PreviousResult>,
}

#[derive(Deserialize)]
struct PreviousResult {
    uri: String,
    value: String,
}

#[derive(Deserialize)]
struct WorkspaceSymbolParams {
    query: String,
}

#[derive(Deserialize)]
struct CommandTarget {
    uri: String,
    position: Position,
}

#[derive(Deserialize)]
struct CommandParams {
    command: String,
    #[serde(default)]
    arguments: Vec<CommandTarget>,
}

#[derive(Deserialize)]
struct CancelParams {
    id: RequestId,
}

#[derive(Deserialize)]
struct Envelope {
    jsonrpc: String,
    method: String,
    #[serde(default)]
    params: Value,
}

/// JSON-RPC and LSP error codes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ErrorCode {
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

type Failure = (ErrorCode, String);

const OUTPUT_LIMIT: &str = "The response is larger than the output limit";

/// The answer to a request: a value, or JSON text already written (a large answer built as a
/// `Value` tree costs tens of times its size in memory).
enum Reply {
    Value(Value),
    Raw(String),
}

/// The requests the server answers.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Request {
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
    const METHODS: &[(&str, Self)] = &[
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

    fn parse(method: &str) -> Option<Self> {
        Self::METHODS
            .iter()
            .find(|(name, _)| *name == method)
            .map(|(_, request)| *request)
    }

    /// Answered only for a client that pulls diagnostics.
    fn pulls_diagnostics(self) -> bool {
        matches!(self, Self::DocumentDiagnostic | Self::WorkspaceDiagnostic)
    }
}

/// The notifications the server acts on; any other is ignored.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Notification {
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
    const METHODS: &[(&str, Self)] = &[
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

    fn parse(method: &str) -> Option<Self> {
        Self::METHODS
            .iter()
            .find(|(name, _)| *name == method)
            .map(|(_, notification)| *notification)
    }
}

/// The commands `workspace/executeCommand` runs.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ServerCommand {
    ShowExpansion,
    ShowPolymorphs,
}

impl ServerCommand {
    const NAMES: &[(&str, Self)] = &[
        ("jai.showExpansion", Self::ShowExpansion),
        ("jai.showPolymorphs", Self::ShowPolymorphs),
    ];

    fn parse(name: &str) -> Option<Self> {
        Self::NAMES
            .iter()
            .find(|(known, _)| *known == name)
            .map(|(_, command)| *command)
    }
}

/// A well-formed request (with an id) or notification.
struct Incoming {
    id: Option<RequestId>,
    method: String,
    params: Value,
}

/// The message in `input`; `None` for the client's answer to a request of ours, and the error
/// response for a malformed one.
fn incoming(input: &str) -> Result<Option<Incoming>, Value> {
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
fn debug_panic(method: &str) {
    #[cfg(debug_assertions)]
    if method == "jai/debugPanic" {
        panic!("requested by jai/debugPanic");
    }
    #[cfg(not(debug_assertions))]
    let _ = method;
}

impl JsonSession {
    pub fn new(limits: Limits) -> Self {
        Self::with_session(Session::new(limits))
    }

    /// A session that type-checks the open documents (see `Session::with_environment`).
    pub fn with_environment(limits: Limits, environment: crate::Environment) -> Self {
        Self::with_session(Session::with_environment(limits, environment))
    }

    fn with_session(session: Session) -> Self {
        Self {
            session,
            lifecycle: Lifecycle::New,
            cancelled: BTreeSet::new(),
            completed: VecDeque::new(),
            hover_kind: MarkupKind::PlainText,
            signature_kind: MarkupKind::PlainText,
            pull_diagnostics: false,
            defer_publications: false,
            publications_pending: false,
            refresh_diagnostics: false,
            refreshes: 0,
        }
    }

    pub fn session(&self) -> &Session {
        &self.session
    }

    pub fn exit_status(&self) -> Option<u8> {
        if let Lifecycle::Exited(code) = self.lifecycle {
            Some(code)
        } else {
            None
        }
    }

    /// Hold back `publishDiagnostics` after `didOpen`, `didChange` and the like until
    /// `flush_publications`, so a transport can answer requests that are already waiting and
    /// publish once typing pauses (see `DIAGNOSTICS_DEBOUNCE` in `main.rs`). Off by default:
    /// `handle_json` then returns the publications with each notification's answer.
    pub fn defer_publications(&mut self, on: bool) {
        self.defer_publications = on;
    }

    /// Withhold (`false`) or allow running the workspace's compile-time code; see
    /// `Session::set_trusted`. A server for an untrusted workspace calls this before serving.
    pub fn set_trusted(&mut self, trusted: bool) {
        self.session.set_trusted(trusted);
    }

    /// (compiles so far, compiles alive); see `Session::compile_counts`.
    pub fn compile_counts(&self) -> (usize, usize) {
        self.session.compile_counts()
    }

    /// Work that makes later requests faster, for a transport to do while it waits for input
    /// (see `Session::warm_auto_import`).
    pub fn warm_up(&self) {
        self.session.warm_auto_import();
    }

    /// An edit is waiting for `flush_publications`.
    pub fn publications_pending(&self) -> bool {
        self.publications_pending
    }

    /// The diagnostics of every open document as they are now (one compile at most), for the
    /// edits since the last flush. Empty for a client that pulls.
    pub fn flush_publications(&mut self) -> Result<Vec<String>, ProtocolError> {
        if !std::mem::take(&mut self.publications_pending) {
            return Ok(Vec::new());
        }
        let messages = self.publications_now();
        self.encode(messages, None)
    }

    pub fn handle_json(&mut self, input: &str) -> Result<Vec<String>, ProtocolError> {
        if input.len() > self.session.limits().message_bytes {
            return Err(ProtocolError::MessageLimit);
        }
        match incoming(input) {
            Err(error) => self.encode(vec![error], None),
            // The client's answer to a request of ours (a diagnostics refresh): nothing to do.
            Ok(None) => Ok(vec![]),
            Ok(Some(Incoming {
                id: Some(id),
                method,
                params,
            })) => self.handle_request(id, &method, params),
            Ok(Some(Incoming {
                id: None,
                method,
                params,
            })) => self.handle_notification(&method, params),
        }
    }

    fn handle_request(
        &mut self,
        id: RequestId,
        method: &str,
        params: Value,
    ) -> Result<Vec<String>, ProtocolError> {
        let identifier = json!(id);
        if self.cancelled.remove(&id) {
            self.remember(id);
            let cancelled = failure(identifier, ErrorCode::RequestCancelled, "Request cancelled");
            return self.encode(vec![cancelled], None);
        }
        let result = self.guarded(method, |this| {
            debug_panic(method);
            this.request(method, params)
        });
        self.remember(id);
        let (mut output, tail) = match result {
            Ok(Reply::Value(result)) => (
                vec![json!({ "jsonrpc": "2.0", "id": identifier, "result": result })],
                None,
            ),
            Ok(Reply::Raw(raw)) => (
                vec![],
                Some(format!(
                    "{{\"jsonrpc\":\"2.0\",\"id\":{identifier},\"result\":{raw}}}"
                )),
            ),
            Err((code, message)) => (vec![failure(identifier.clone(), code, &message)], None),
        };
        output.extend(self.shown_messages());
        match self.encode(output, tail) {
            // A response over the output cap is that request's error, not the end of the session.
            Err(ProtocolError::OutputLimit) => self.encode(
                vec![failure(identifier, ErrorCode::RequestFailed, OUTPUT_LIMIT)],
                None,
            ),
            other => other,
        }
    }

    fn handle_notification(
        &mut self,
        method: &str,
        params: Value,
    ) -> Result<Vec<String>, ProtocolError> {
        let notification = Notification::parse(method);
        match notification {
            Some(Notification::Cancel) => {
                let result = serde_json::from_value::<CancelParams>(params)
                    .map_err(|_| Error::InvalidEdit("invalid cancellation parameters"))
                    .and_then(|params| self.cancel_request(params.id));
                let errors = result.err().map(|e| log_error(&e.to_string()));
                return self.encode(errors.into_iter().collect(), None);
            }
            Some(Notification::Exit) => {
                let code = u8::from(self.lifecycle != Lifecycle::Shutdown);
                self.lifecycle = Lifecycle::Exited(code);
                return Ok(vec![]);
            }
            _ => {}
        }
        let result = self.guarded(method, |this| {
            debug_panic(method);
            this.notification(notification, params)
        });
        let mut output = match result {
            Ok(messages) => messages,
            // Nothing is logged before `initialize`.
            Err(_) if self.lifecycle == Lifecycle::New => vec![],
            Err((_, message)) => vec![log_error(&message)],
        };
        output.extend(self.shown_messages());
        match self.encode(output, None) {
            Err(ProtocolError::OutputLimit) => self.encode(vec![log_error(OUTPUT_LIMIT)], None),
            other => other,
        }
    }

    /// Run `handler`. A panic in it is that message's error; the server and its documents live on.
    fn guarded<T>(
        &mut self,
        method: &str,
        handler: impl FnOnce(&mut Self) -> Result<T, Failure>,
    ) -> Result<T, Failure> {
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| handler(self))).unwrap_or_else(
            |payload| {
                let what = payload
                    .downcast_ref::<&str>()
                    .map(|s| s.to_string())
                    .or_else(|| payload.downcast_ref::<String>().cloned())
                    .unwrap_or_else(|| "unknown panic".into());
                eprintln!("jailsp: internal error in `{method}`: {what}");
                self.session.reset_caches();
                Err(fail(
                    ErrorCode::InternalError,
                    format!("Internal error in `{method}`: {what}"),
                ))
            },
        )
    }

    /// The session's messages for the user, as `window/showMessage` warnings, and its log lines
    /// as `window/logMessage` infos.
    fn shown_messages(&self) -> Vec<Value> {
        let logs = self.session.take_logs().into_iter().map(|message| {
            json!({
                "jsonrpc": "2.0",
                "method": "window/logMessage",
                "params": { "type": 3, "message": message },
            })
        });
        self.session
            .take_messages()
            .into_iter()
            .map(|message| {
                json!({
                    "jsonrpc": "2.0",
                    "method": "window/showMessage",
                    "params": { "type": 2, "message": message },
                })
            })
            .chain(logs)
            .collect()
    }

    pub fn cancel_request(&mut self, id: RequestId) -> Result<(), Error> {
        if !valid_id(&id) {
            return Err(Error::InvalidEdit("invalid cancellation id"));
        }
        if self.completed.contains(&id) {
            return Ok(());
        }
        if self.cancelled.len() >= self.session.limits().cancelled_requests
            && !self.cancelled.contains(&id)
        {
            return Err(Error::Limit("cancellation queue exceeded"));
        }
        self.cancelled.insert(id);
        Ok(())
    }

    fn remember(&mut self, id: RequestId) {
        let limit = self.session.limits().cancelled_requests;
        if limit == 0 {
            return;
        }
        if self.completed.len() >= limit {
            self.completed.pop_front();
        }
        self.completed.push_back(id);
    }

    /// The messages as JSON text, then `tail` (already text: the response to a large request).
    fn encode(
        &self,
        messages: Vec<Value>,
        tail: Option<String>,
    ) -> Result<Vec<String>, ProtocolError> {
        let mut out = vec![];
        let mut bytes = 0usize;
        let texts = messages
            .into_iter()
            .map(|message| {
                serde_json::to_string(&message)
                    .map_err(|e| ProtocolError::Serialization(e.to_string()))
            })
            .chain(tail.map(Ok));
        for text in texts {
            let text = text?;
            bytes = bytes
                .checked_add(text.len())
                .filter(|n| *n <= self.session.limits().output_bytes)
                .ok_or(ProtocolError::OutputLimit)?;
            out.push(text);
        }
        Ok(out)
    }

    fn ensure_running(&self) -> Result<(), Failure> {
        match self.lifecycle {
            Lifecycle::Running => Ok(()),
            Lifecycle::New => Err(fail(
                ErrorCode::ServerNotInitialized,
                "Server is not initialized",
            )),
            Lifecycle::Shutdown | Lifecycle::Exited(_) => {
                Err(fail(ErrorCode::InvalidRequest, "Server has shut down"))
            }
        }
    }

    fn request(&mut self, method: &str, params: Value) -> Result<Reply, Failure> {
        let request = Request::parse(method);
        if request != Some(Request::Initialize) {
            self.ensure_running()?;
        }
        let request = request
            .filter(|r| self.pull_diagnostics || !r.pulls_diagnostics())
            .ok_or_else(|| fail(ErrorCode::MethodNotFound, "Method not supported"))?;
        let result = match request {
            Request::Initialize => self.initialize(params)?,
            Request::Shutdown => {
                self.lifecycle = Lifecycle::Shutdown;
                Value::Null
            }
            Request::SemanticTokens => return self.semantic_tokens(params),
            Request::DocumentSymbol => return self.document_symbols(params),
            Request::Hover => {
                let (document, position) = at(params)?;
                let hover = self.session.hover_as(&document, position, self.hover_kind);
                hover
                    .map_err(domain)?
                    .as_ref()
                    .map_or(Value::Null, hover_wire)
            }
            Request::Completion => {
                let (document, position) = at(params)?;
                let list = self.session.completion(&document, position);
                completion_wire(&list.map_err(domain)?)
            }
            Request::CompletionResolve => self.resolve_completion(params),
            Request::Definition => {
                let (document, position) = at(params)?;
                locations_wire(
                    &self
                        .session
                        .definition(&document, position)
                        .map_err(domain)?,
                )
            }
            Request::TypeDefinition => {
                let (document, position) = at(params)?;
                let found = self.session.type_definition(&document, position);
                locations_wire(&found.map_err(domain)?)
            }
            Request::References => {
                let p: ReferenceParams = decode(params)?;
                let declaration = p.context.is_none_or(|c| c.include_declaration);
                let document = uri(&p.text_document.uri)?;
                let found = self.session.references(&document, p.position, declaration);
                locations_wire(&found.map_err(domain)?)
            }
            Request::PrepareRename => {
                let (document, position) = at(params)?;
                let range = self.session.prepare_rename(&document, position);
                range
                    .map_err(domain)?
                    .map_or(Value::Null, |range| json!(range))
            }
            Request::Rename => {
                let p: RenameParams = decode(params)?;
                let document = uri(&p.text_document.uri)?;
                let edits = self.session.rename(&document, p.position, &p.new_name);
                edits
                    .map_err(domain)?
                    .map_or(Value::Null, workspace_edit_wire)
            }
            Request::DocumentHighlight => {
                let (document, position) = at(params)?;
                let ranges = self.session.document_highlights(&document, position);
                array(
                    ranges.map_err(domain)?,
                    |range| json!({ "range": range, "kind": 1 }),
                )
            }
            Request::WorkspaceSymbol => {
                let p: WorkspaceSymbolParams = decode(params)?;
                array(
                    &self.session.workspace_symbols(&p.query),
                    symbol_information_wire,
                )
            }
            Request::DocumentLink => {
                let links = self.session.document_links(&document(params)?);
                array(
                    links.map_err(domain)?,
                    |(range, target)| json!({ "range": range, "target": target }),
                )
            }
            Request::FoldingRange => {
                let ranges = self.session.folding_ranges(&document(params)?);
                array(&ranges.map_err(domain)?, folding_wire)
            }
            Request::InlayHint => {
                let p: RangeParams = decode(params)?;
                let hints = self
                    .session
                    .inlay_hints(&uri(&p.text_document.uri)?, p.range);
                array(&hints.map_err(domain)?, inlay_wire)
            }
            Request::SignatureHelp => {
                let (document, position) = at(params)?;
                let kind = self.signature_kind;
                let help = self.session.signature_help_as(&document, position, kind);
                help.map_err(domain)?
                    .as_ref()
                    .map_or(Value::Null, |help| signature_wire(help, kind))
            }
            Request::CodeAction => self.code_actions(params)?,
            Request::PrepareCallHierarchy => {
                let (document, position) = at(params)?;
                let items = self.session.prepare_call_hierarchy(&document, position);
                match items.map_err(domain)? {
                    items if items.is_empty() => Value::Null,
                    items => array(&items, hierarchy_item_wire),
                }
            }
            Request::IncomingCalls => {
                let p: HierarchyParams = decode(params)?;
                let calls = self.session.incoming_calls(&p.item.into_item());
                array(&calls.map_err(domain)?, |c| hierarchy_call_wire(c, "from"))
            }
            Request::OutgoingCalls => {
                let p: HierarchyParams = decode(params)?;
                let calls = self.session.outgoing_calls(&p.item.into_item());
                array(&calls.map_err(domain)?, |c| hierarchy_call_wire(c, "to"))
            }
            Request::SelectionRange => {
                let p: SelectionParams = decode(params)?;
                let ranges = self
                    .session
                    .selection_ranges(&uri(&p.text_document.uri)?, &p.positions);
                array(&ranges.map_err(domain)?, selection_wire)
            }
            Request::CodeLens => {
                let lenses = self.session.code_lenses(&document(params)?);
                array(&lenses.map_err(domain)?, code_lens_wire)
            }
            Request::ExecuteCommand => self.execute_command(params)?,
            Request::Expansion => {
                let (document, position) = at(params)?;
                let expansion = self.session.expansion(&document, position);
                expansion
                    .map_err(domain)?
                    .as_ref()
                    .map_or(Value::Null, expansion_wire)
            }
            Request::DocumentDiagnostic => {
                let p: DiagnosticParams = decode(params)?;
                let target = uri(&p.text_document.uri)?;
                let (version, diagnostics) = self.pulled(&target);
                document_report(None, version, &diagnostics, p.previous_result_id.as_deref())
            }
            Request::WorkspaceDiagnostic => self.workspace_diagnostics(params)?,
            Request::Source => self.source(params)?,
        };
        Ok(Reply::Value(result))
    }

    fn initialize(&mut self, params: Value) -> Result<Value, Failure> {
        if self.lifecycle != Lifecycle::New {
            return Err(fail(
                ErrorCode::InvalidRequest,
                "Server is already initialized",
            ));
        }
        if !params.is_object() && !params.is_null() {
            return Err(fail(
                ErrorCode::InvalidParams,
                "Initialize parameters must be an object",
            ));
        }
        self.lifecycle = Lifecycle::Running;
        self.hover_kind = hover_kind(&params);
        self.signature_kind = signature_kind(&params);
        self.pull_diagnostics = params
            .pointer("/capabilities/textDocument/diagnostic")
            .is_some_and(Value::is_object);
        self.refresh_diagnostics = self.pull_diagnostics
            && params
                .pointer("/capabilities/workspace/diagnostics/refreshSupport")
                .and_then(Value::as_bool)
                == Some(true);
        self.configure(&params["initializationOptions"]);
        // A server started untrusted (`--untrusted`) stays so unless the client grants
        // trust later; initialization options can only withhold it.
        if params["initializationOptions"]["trusted"].as_bool() == Some(false) {
            self.session.set_trusted(false);
        }
        let folders: Vec<PathBuf> = match params["workspaceFolders"].as_array() {
            Some(folders) => folders
                .iter()
                .filter_map(|f| folder_path(&f["uri"]))
                .collect(),
            None => folder_path(&params["rootUri"]).into_iter().collect(),
        };
        self.session.set_workspace_folders(folders);
        Ok(json!({
            "capabilities": capabilities(self.pull_diagnostics),
            "serverInfo": {
                "name": "jai-language-server",
                "version": env!("CARGO_PKG_VERSION"),
            },
        }))
    }

    fn semantic_tokens(&mut self, params: Value) -> Result<Reply, Failure> {
        let document = document(params)?;
        let version = self.session.version(&document).map_err(domain)?;
        let tokens = self.session.semantic_tokens(&document).map_err(domain)?;
        let data = semantic_tokens_wire(&tokens);
        drop(tokens);
        let data = serde_json::to_string(&data).map_err(internal)?;
        Ok(Reply::Raw(format!(
            "{{\"resultId\":\"{version}\",\"data\":{data}}}"
        )))
    }

    fn document_symbols(&mut self, params: Value) -> Result<Reply, Failure> {
        let symbols = self
            .session
            .document_symbols(&document(params)?)
            .map_err(domain)?;
        let text = serde_json::to_string(&SymbolsWire(&symbols)).map_err(internal)?;
        Ok(Reply::Raw(text))
    }

    /// The documentation of an item whose list left it out.
    fn resolve_completion(&self, mut item: Value) -> Value {
        let data = item.get("data").and_then(|d| {
            Some(crate::CompletionData {
                document: d.get("document")?.as_str()?.to_owned(),
                path: d.get("path")?.as_str()?.to_owned(),
                offset: d.get("offset")?.as_u64()? as usize,
            })
        });
        if let Some(doc) = data.and_then(|d| self.session.completion_documentation(&d)) {
            item["documentation"] = json!({ "kind": "markdown", "value": doc });
        }
        item
    }

    fn code_actions(&mut self, params: Value) -> Result<Value, Failure> {
        let p: CodeActionParams = decode(params)?;
        let context = p.context.unwrap_or_default();
        let context = crate::lints::ActionContext {
            only: context.only,
            diagnostics: context
                .diagnostics
                .into_iter()
                .filter(|d| matches!(d.source.as_deref(), Some("jailint" | "jaic")))
                .filter_map(|d| Some((d.code?.as_str()?.to_string(), d.range)))
                .collect(),
        };
        let actions = self
            .session
            .code_actions_in(&uri(&p.text_document.uri)?, p.range, &context);
        Ok(array(&actions.map_err(domain)?, code_action_wire))
    }

    fn execute_command(&mut self, params: Value) -> Result<Value, Failure> {
        let p: CommandParams = decode(params)?;
        let Some(target) = p.arguments.first() else {
            return Err(fail(
                ErrorCode::InvalidParams,
                "The command needs a {uri, position} argument",
            ));
        };
        let document = uri(&target.uri)?;
        match ServerCommand::parse(&p.command) {
            Some(ServerCommand::ShowExpansion) => {
                let expansion = self.session.expansion(&document, target.position);
                Ok(expansion
                    .map_err(domain)?
                    .as_ref()
                    .map_or(Value::Null, expansion_wire))
            }
            Some(ServerCommand::ShowPolymorphs) => {
                Ok(json!(self.session.polymorphs(&document, target.position)))
            }
            None => Err(fail(ErrorCode::InvalidParams, "Unknown command")),
        }
    }

    fn workspace_diagnostics(&mut self, params: Value) -> Result<Value, Failure> {
        let p: WorkspaceDiagnosticParams = decode(params)?;
        let items: Vec<Value> = self
            .session
            .publications()
            .into_iter()
            .map(|(target, version, diagnostics)| {
                let previous = p
                    .previous_result_ids
                    .iter()
                    .find(|r| r.uri == target)
                    .map(|r| r.value.as_str());
                document_report(Some(&target), Some(version), &diagnostics, previous)
            })
            .collect();
        Ok(json!({ "items": items }))
    }

    /// The text of a definition's file the client has not opened (a module or stdlib file), so a
    /// browser editor can show it read-only; or of a `jai-expansion:` document.
    fn source(&mut self, params: Value) -> Result<Value, Failure> {
        let p: DocumentIdentifier = decode(params)?;
        let text = if p.uri.starts_with(crate::features::EXPANSION_SCHEME) {
            self.session.expansion_source(&p.uri)
        } else {
            self.session.source(&uri(&p.uri)?)
        };
        Ok(text.map_or(Value::Null, Value::String))
    }

    fn notification(
        &mut self,
        notification: Option<Notification>,
        params: Value,
    ) -> Result<Vec<Value>, Failure> {
        self.ensure_running()?;
        // Unknown notifications are ignored as required by JSON-RPC/LSP.
        let Some(notification) = notification else {
            return Ok(vec![]);
        };
        match notification {
            // `$/cancelRequest` and `exit` are handled before the lifecycle is checked.
            Notification::Initialized | Notification::Cancel | Notification::Exit => Ok(vec![]),
            Notification::DidOpen => self.did_open(params),
            Notification::DidChange => self.did_change(params),
            Notification::DidClose => self.did_close(params),
            Notification::DidChangeConfiguration => Ok(self.did_change_configuration(&params)),
            Notification::SetTrusted => {
                let Some(on) = params["trusted"].as_bool() else {
                    return Err(fail(
                        ErrorCode::InvalidParams,
                        "jai/setTrusted needs a boolean `trusted`",
                    ));
                };
                self.session.set_trusted(on);
                Ok(self.edited(true))
            }
            Notification::DidChangeWorkspaceFolders => {
                self.did_change_workspace_folders(&params["event"]);
                Ok(vec![])
            }
            Notification::DidChangeWatchedFiles => {
                // Files changed on disk: 1 created, 2 changed, 3 deleted.
                for change in params["changes"].as_array().into_iter().flatten() {
                    if let Some(path) = folder_path(&change["uri"]) {
                        let changed = change["type"].as_u64() == Some(2);
                        self.session.file_changed(&path, !changed);
                    }
                }
                Ok(self.refresh())
            }
        }
    }

    /// The publications after an edit, and a pull request when a settings file changed.
    fn edited(&mut self, settings_touched: bool) -> Vec<Value> {
        let mut messages = self.publications();
        if settings_touched {
            messages.extend(self.refresh());
        }
        messages
    }

    fn did_open(&mut self, params: Value) -> Result<Vec<Value>, Failure> {
        let OpenParams {
            text_document: item,
        } = decode(params)?;
        let settings = crate::project::CONFIG_NAMES
            .iter()
            .any(|name| item.uri.ends_with(&format!("/{name}")));
        if item.language_id != "jai" && !settings {
            return Err(fail(
                ErrorCode::InvalidParams,
                "Only Jai documents are supported",
            ));
        }
        let size = item.text.len();
        let target = uri(&item.uri)?;
        if let Err(error) = self.session.open(target, item.version, item.text) {
            // Never drop a document without telling the user.
            if let Error::Limit(what) = &error {
                self.session.notify(format!(
                    "jailsp did not open {} ({:.1} MiB): {what}. Its diagnostics, hover and completion are off.",
                    item.uri,
                    size as f64 / (1024.0 * 1024.0)
                ));
            }
            return Err(domain(error));
        }
        Ok(self.edited(settings))
    }

    fn did_change(&mut self, params: Value) -> Result<Vec<Value>, Failure> {
        let p: ChangeParams = decode(params)?;
        let target = uri(&p.text_document.uri)?;
        let settings = crate::lints::is_config(&target);
        let changed = self
            .session
            .change(&target, p.text_document.version, &p.content_changes);
        if let Err(error) = changed {
            if let Error::Limit(what) = &error {
                self.session.notify(format!(
                    "jailsp did not apply an edit to {}: {what}.",
                    p.text_document.uri
                ));
            }
            return Err(domain(error));
        }
        Ok(self.edited(settings))
    }

    fn did_close(&mut self, params: Value) -> Result<Vec<Value>, Failure> {
        let closed = document(params)?;
        self.session.close(&closed).map_err(domain)?;
        if self.pull_diagnostics {
            return Ok(if crate::lints::is_config(&closed) {
                self.refresh()
            } else {
                Vec::new()
            });
        }
        let mut publications = self.publications();
        publications.push(json!({
            "jsonrpc": "2.0",
            "method": "textDocument/publishDiagnostics",
            "params": { "uri": closed.as_str(), "diagnostics": [] },
        }));
        Ok(publications)
    }

    fn did_change_configuration(&mut self, params: &Value) -> Vec<Value> {
        let settings = &params["settings"]["jai"];
        let trusted = self.session.trusted();
        self.configure(settings);
        if let Some(on) = settings["trusted"].as_bool() {
            self.session.set_trusted(on);
        }
        // Trust decides what the compile may run, so every document's diagnostics change.
        let mut messages = if trusted != self.session.trusted() {
            self.publications()
        } else {
            Vec::new()
        };
        messages.extend(self.refresh());
        messages
    }

    fn did_change_workspace_folders(&mut self, event: &Value) {
        let folders = |key: &str| -> Vec<PathBuf> {
            event[key]
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(|f| folder_path(&f["uri"]))
                .collect()
        };
        let removed = folders("removed");
        let mut kept: Vec<PathBuf> = self
            .session
            .workspace_folders
            .iter()
            .filter(|f| !removed.contains(f))
            .cloned()
            .collect();
        kept.extend(folders("added"));
        self.session.set_workspace_folders(kept);
    }


    /// The version and diagnostics of `target` for a pull (none for a document that is not open).
    fn pulled(&self, target: &DocumentUri) -> (Option<i32>, Vec<Diagnostic>) {
        (
            self.session.version(target).ok(),
            self.session.diagnostics(target).unwrap_or_default(),
        )
    }

    /// A request for the client to pull again, when it supports being asked: settings files
    /// changed the diagnostics of documents that did not.
    fn refresh(&mut self) -> Vec<Value> {
        if !self.refresh_diagnostics {
            return Vec::new();
        }
        self.refreshes += 1;
        vec![json!({
            "jsonrpc": "2.0",
            "id": format!("jai-diagnostic-refresh-{}", self.refreshes),
            "method": "workspace/diagnostic/refresh",
        })]
    }

    fn publications(&mut self) -> Vec<Value> {
        if self.pull_diagnostics {
            return Vec::new();
        }
        if self.defer_publications {
            self.publications_pending = true;
            return Vec::new();
        }
        self.publications_now()
    }

    fn publications_now(&self) -> Vec<Value> {
        self.session
            .publications()
            .into_iter()
            .map(|(uri, version, diagnostics)| {
                let diagnostics: Vec<Value> = diagnostics.iter().map(diagnostic_wire).collect();
                json!({
                    "jsonrpc": "2.0",
                    "method": "textDocument/publishDiagnostics",
                    "params": { "uri": uri, "version": version, "diagnostics": diagnostics },
                })
            })
            .collect()
    }
}

impl JsonSession {
    /// Apply the settings in `settings` (`initializationOptions`, or the `jai` section of
    /// `workspace/didChangeConfiguration`): `{ "completion": { "autoImport": false } }`.
    fn configure(&mut self, settings: &Value) {
        if let Some(on) = settings["completion"]["autoImport"].as_bool() {
            self.session.set_auto_import(on);
        }
    }
}

/// The path of a `file://` URI value.
fn folder_path(uri: &Value) -> Option<PathBuf> {
    DocumentUri::parse(uri.as_str()?)
        .ok()
        .map(|u| PathBuf::from(u.path()))
}

fn valid_id(id: &RequestId) -> bool {
    matches!(id, RequestId::Number(_)) || matches!(id, RequestId::String(text) if text.len() <= 128)
}

fn decode<T: serde::de::DeserializeOwned>(value: Value) -> Result<T, Failure> {
    serde_json::from_value(value)
        .map_err(|_| fail(ErrorCode::InvalidParams, "Invalid method parameters"))
}

fn uri(text: &str) -> Result<DocumentUri, Failure> {
    DocumentUri::parse(text).map_err(domain)
}

/// The document of a `{ textDocument }` request.
fn document(params: Value) -> Result<DocumentUri, Failure> {
    let p: DocumentParams = decode(params)?;
    uri(&p.text_document.uri)
}

/// The document and position of a `{ textDocument, position }` request.
fn at(params: Value) -> Result<(DocumentUri, Position), Failure> {
    let p: PositionParams = decode(params)?;
    Ok((uri(&p.text_document.uri)?, p.position))
}

fn array<I: IntoIterator>(items: I, wire: impl FnMut(I::Item) -> Value) -> Value {
    Value::Array(items.into_iter().map(wire).collect())
}

fn fail(code: ErrorCode, message: impl Into<String>) -> Failure {
    (code, message.into())
}

fn internal(error: serde_json::Error) -> Failure {
    fail(ErrorCode::InternalError, error.to_string())
}

fn domain(error: Error) -> Failure {
    let code = match error {
        Error::Limit(_) => ErrorCode::RequestFailed,
        Error::StaleVersion => ErrorCode::ContentModified,
        _ => ErrorCode::InvalidParams,
    };
    (code, error.to_string())
}

fn failure(id: Value, code: ErrorCode, message: &str) -> Value {
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

fn log_error(message: &str) -> Value {
    json!({
        "jsonrpc": "2.0",
        "method": "window/logMessage",
        "params": { "type": 1, "message": message },
    })
}

// Domain enums have no protocol discriminants. Only this JSON boundary maps LSP values.
/// A document's diagnostics for a pull: the report is `unchanged` when the client's last
/// result id still describes them. `uri` is set for the items of a workspace report.
fn document_report(
    uri: Option<&str>,
    version: Option<i32>,
    diagnostics: &[Diagnostic],
    previous: Option<&str>,
) -> Value {
    let items: Vec<Value> = diagnostics.iter().map(diagnostic_wire).collect();
    let result_id = {
        use std::hash::{Hash, Hasher};
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        Value::Array(items.clone()).to_string().hash(&mut hasher);
        format!("{:016x}", hasher.finish())
    };
    let mut report = if previous == Some(result_id.as_str()) {
        json!({ "kind": "unchanged", "resultId": result_id })
    } else {
        json!({ "kind": "full", "resultId": result_id, "items": items })
    };
    if let Some(uri) = uri {
        report["uri"] = json!(uri);
        report["version"] = json!(version);
    }
    report
}

fn diagnostic_wire(diagnostic: &Diagnostic) -> Value {
    let severity = match diagnostic.severity {
        DiagnosticSeverity::Error => 1,
        DiagnosticSeverity::Warning => 2,
        DiagnosticSeverity::Hint => 4,
    };
    let (code, source) = match diagnostic.code {
        DiagnosticCode::Lexer => ("lexer", "jaic"),
        DiagnosticCode::Parser => ("parser", "jaic"),
        DiagnosticCode::Source => ("source", "jaic"),
        DiagnosticCode::Limit => ("limit", "jaic"),
        DiagnosticCode::Check => ("check", "jaic"),
        DiagnosticCode::Unreachable => ("unreachable", "jaic"),
        DiagnosticCode::Lint(rule) => (rule, "jailint"),
    };
    let mut value = json!({
        "range": diagnostic.range,
        "severity": severity,
        "code": code,
        "source": source,
        "message": diagnostic.message,
    });
    if diagnostic.code == DiagnosticCode::Unreachable {
        // `DiagnosticTag.Unnecessary`
        value["tags"] = json!([1]);
    }
    if let DiagnosticCode::Lint(rule) = diagnostic.code {
        value["codeDescription"] = json!({ "href": crate::lints::rule_url(rule) });
    }
    value
}

/// The symbols of a document as the protocol writes them, serialized straight to text.
struct SymbolsWire<'a>(&'a [DocumentSymbol]);

impl serde::Serialize for SymbolsWire<'_> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_seq(self.0.iter().map(SymbolWire))
    }
}

struct SymbolWire<'a>(&'a DocumentSymbol);

impl serde::Serialize for SymbolWire<'_> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeMap;
        let symbol = self.0;
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("name", &symbol.name)?;
        map.serialize_entry("detail", &symbol.detail)?;
        map.serialize_entry("kind", &symbol_kind_wire(symbol.kind))?;
        map.serialize_entry("range", &symbol.range)?;
        map.serialize_entry("selectionRange", &symbol.selection_range)?;
        if !symbol.children.is_empty() {
            map.serialize_entry("children", &SymbolsWire(&symbol.children))?;
        }
        map.end()
    }
}

fn hierarchy_item_wire(item: &CallHierarchyItem) -> Value {
    json!({
        "name": item.name,
        "kind": symbol_kind_wire(item.kind),
        "detail": item.detail,
        "uri": item.uri,
        "range": item.range,
        "selectionRange": item.selection_range,
    })
}

/// `side` is `from` for an incoming call and `to` for an outgoing one.
fn hierarchy_call_wire(call: &CallHierarchyCall, side: &str) -> Value {
    json!({ side: hierarchy_item_wire(&call.item), "fromRanges": call.from_ranges })
}

fn selection_wire(selection: &SelectionRange) -> Value {
    let mut value = json!({ "range": selection.range });
    if let Some(parent) = &selection.parent {
        value["parent"] = selection_wire(parent);
    }
    value
}

fn location_wire(location: &Location) -> Value {
    json!({ "uri": location.uri, "range": location.range })
}

fn locations_wire(locations: &[Location]) -> Value {
    Value::Array(locations.iter().map(location_wire).collect())
}

fn symbol_kind_wire(kind: SymbolKind) -> u32 {
    match kind {
        SymbolKind::Namespace => 2,
        SymbolKind::TypeAlias => 5,
        SymbolKind::Property => 7,
        SymbolKind::Enum => 10,
        SymbolKind::Function => 12,
        SymbolKind::Variable => 13,
        SymbolKind::Constant => 14,
        SymbolKind::EnumMember => 22,
        SymbolKind::Struct => 23,
    }
}

fn symbol_information_wire(symbol: &SymbolInformation) -> Value {
    let mut value = json!({
        "name": symbol.name,
        "kind": symbol_kind_wire(symbol.kind),
        "location": location_wire(&symbol.location),
    });
    if let Some(container) = &symbol.container {
        value["containerName"] = json!(container);
    }
    value
}

fn folding_wire(range: &FoldingRange) -> Value {
    let mut value = json!({ "startLine": range.start_line, "endLine": range.end_line });
    if range.imports {
        value["kind"] = json!("imports");
    }
    value
}

fn inlay_wire(hint: &InlayHint) -> Value {
    let mut value = json!({
        "position": hint.position,
        "label": hint.label,
        "paddingLeft": hint.padding_left,
        "paddingRight": hint.padding_right,
    });
    if let Some(kind) = hint.kind {
        value["kind"] = json!(match kind {
            InlayHintKind::Type => 1,
            InlayHintKind::Parameter => 2,
        });
    }
    if let Some(tooltip) = &hint.tooltip {
        value["tooltip"] = json!(tooltip);
    }
    value
}

/// Documentation as the client asked for it: a `MarkupContent`, or a plain string.
fn documentation_wire(kind: MarkupKind, text: &str) -> Value {
    match kind {
        MarkupKind::Markdown => json!({ "kind": "markdown", "value": text }),
        MarkupKind::PlainText => json!(text),
    }
}

fn signature_wire(help: &SignatureHelp, kind: MarkupKind) -> Value {
    let signatures: Vec<Value> = help
        .signatures
        .iter()
        .map(|s| {
            let parameters: Vec<Value> = s
                .parameters
                .iter()
                .enumerate()
                .map(|(i, p)| {
                    let mut wire = json!({ "label": p });
                    if let Some(Some(doc)) = s.parameter_docs.get(i) {
                        wire["documentation"] = documentation_wire(kind, doc);
                    }
                    wire
                })
                .collect();
            let mut wire = json!({ "label": s.label, "parameters": parameters });
            if let Some(doc) = &s.documentation {
                wire["documentation"] = documentation_wire(kind, doc);
            }
            wire
        })
        .collect();
    json!({
        "signatures": signatures,
        "activeSignature": help.active_signature,
        "activeParameter": help.active_parameter,
    })
}

fn command_wire(command: &Command) -> Value {
    let arguments: Vec<Value> = command
        .target
        .iter()
        .map(|(uri, position)| json!({ "uri": uri, "position": position }))
        .collect();
    json!({ "title": command.title, "command": command.command, "arguments": arguments })
}

fn code_action_wire(action: &CodeAction) -> Value {
    let mut value = json!({ "title": action.title });
    if let Some(kind) = action.kind {
        value["kind"] = json!(kind);
    }
    if let Some((uri, edits)) = &action.edit {
        let edits: Vec<Value> = edits
            .iter()
            .map(|e| json!({ "range": e.range, "newText": e.new_text }))
            .collect();
        value["edit"] = json!({ "changes": { uri.clone(): edits } });
    }
    if let Some(command) = &action.command {
        value["command"] = command_wire(command);
    }
    if !action.diagnostics.is_empty() {
        value["diagnostics"] =
            Value::Array(action.diagnostics.iter().map(diagnostic_wire).collect());
    }
    if action.is_preferred {
        value["isPreferred"] = json!(true);
    }
    if let Some(rule) = action.rule {
        value["data"] = json!({ "rule": rule });
    }
    value
}

fn code_lens_wire(lens: &CodeLens) -> Value {
    json!({ "range": lens.range, "command": command_wire(&lens.command) })
}

fn expansion_wire(expansion: &Expansion) -> Value {
    json!({
        "uri": expansion.uri,
        "kind": expansion.kind,
        "text": expansion.text,
        "source": location_wire(&expansion.source),
    })
}

/// The documentation format a client asked for signature help.
fn signature_kind(params: &Value) -> MarkupKind {
    let formats = params
        .pointer(
            "/capabilities/textDocument/signatureHelp/signatureInformation/documentationFormat",
        )
        .and_then(Value::as_array);
    match formats {
        Some(formats) if formats.iter().any(|f| f == "markdown") => MarkupKind::Markdown,
        _ => MarkupKind::PlainText,
    }
}

/// The hover format a client asked for in its `initialize` capabilities.
/// What the server offers; diagnostics are pulled only from a client that asked to.
fn capabilities(pull_diagnostics: bool) -> Value {
    let token_types: Vec<&str> = SemanticTokenKind::ALL.iter().map(|k| k.name()).collect();
    let commands: Vec<&str> = ServerCommand::NAMES.iter().map(|(name, _)| *name).collect();
    let mut capabilities = json!({
        "positionEncoding": "utf-16",
        "textDocumentSync": { "openClose": true, "change": 2 },
        "hoverProvider": true,
        "completionProvider": {
            "resolveProvider": true,
            "triggerCharacters": [".", "#", "\"", "/"],
        },
        "definitionProvider": true,
        "typeDefinitionProvider": true,
        "referencesProvider": true,
        "renameProvider": { "prepareProvider": true },
        "documentHighlightProvider": true,
        "documentSymbolProvider": true,
        "workspaceSymbolProvider": true,
        "foldingRangeProvider": true,
        "documentLinkProvider": { "resolveProvider": false },
        "inlayHintProvider": true,
        "signatureHelpProvider": {
            "triggerCharacters": ["(", ","],
            "retriggerCharacters": [","],
        },
        "codeActionProvider": {
            "codeActionKinds": [
                "quickfix",
                "refactor.extract",
                "refactor.inline",
                "refactor.rewrite",
                crate::lints::FIX_ALL_KIND,
            ]
        },
        "codeLensProvider": { "resolveProvider": false },
        "callHierarchyProvider": true,
        "selectionRangeProvider": true,
        "executeCommandProvider": { "commands": commands },
        "semanticTokensProvider": {
            "legend": {
                "tokenTypes": token_types,
                "tokenModifiers": TOKEN_MODIFIERS,
            },
            "full": true,
        },
        "experimental": {
            "jai": {
                "analysis": "compiler-source-syntax",
                "compileTimeExecution": false,
                "typeInference": false,
                "filesystemReads": false,
                "moduleSearch": false,
                // `jai/expansion` and `jai-expansion:` documents.
                "expansions": true,
            },
        },
    });
    if pull_diagnostics {
        capabilities["diagnosticProvider"] = json!({
            "identifier": "jai",
            "interFileDependencies": true,
            "workspaceDiagnostics": true,
        });
    }
    capabilities
}

fn workspace_edit_wire(documents: crate::features::RenameEdits) -> Value {
    let changes: serde_json::Map<String, Value> = documents
        .into_iter()
        .map(|(target, edits)| {
            let edits = array(
                &edits,
                |e| json!({ "range": e.range, "newText": e.new_text }),
            );
            (target, edits)
        })
        .collect();
    json!({ "changes": changes })
}

fn hover_kind(params: &Value) -> MarkupKind {
    let formats = params
        .pointer("/capabilities/textDocument/hover/contentFormat")
        .and_then(Value::as_array);
    match formats {
        Some(formats) if formats.iter().any(|f| f == "markdown") => MarkupKind::Markdown,
        _ => MarkupKind::PlainText,
    }
}

fn hover_wire(hover: &Hover) -> Value {
    let kind = match hover.contents.kind {
        MarkupKind::PlainText => "plaintext",
        MarkupKind::Markdown => "markdown",
    };
    json!({
        "contents": { "kind": kind, "value": hover.contents.value },
        "range": hover.range,
    })
}

/// The LSP `CompletionItemKind` of a completion.
fn completion_kind_wire(kind: CompletionKind) -> u32 {
    match kind {
        CompletionKind::Function => 3,
        CompletionKind::Field => 5,
        CompletionKind::Variable => 6,
        CompletionKind::TypeAlias => 7,
        CompletionKind::Module => 9,
        CompletionKind::Enum => 13,
        CompletionKind::Keyword => 14,
        CompletionKind::Snippet => 15,
        CompletionKind::File => 17,
        CompletionKind::Folder => 19,
        CompletionKind::EnumMember => 20,
        CompletionKind::Constant => 21,
        CompletionKind::Struct => 22,
        // An `#asm` mnemonic is an operation, not a language keyword.
        CompletionKind::Instruction => 24,
    }
}

fn completion_wire(completion: &CompletionList) -> Value {
    let items: Vec<Value> = completion
        .items
        .iter()
        .map(|item| {
            let kind = completion_kind_wire(item.kind);
            let mut wire = json!({ "label": item.label, "kind": kind, "detail": item.detail });
            if let Some(doc) = &item.documentation {
                wire["documentation"] = json!({ "kind": "markdown", "value": doc });
            }
            if let Some(data) = &item.data {
                wire["data"] = json!({
                    "document": data.document,
                    "path": data.path,
                    "offset": data.offset,
                });
            }
            if let Some(description) = &item.label_description {
                wire["labelDetails"] = json!({ "description": description });
            }
            if let Some(sort) = &item.sort_text {
                wire["sortText"] = json!(sort);
            }
            if !item.additional_edits.is_empty() {
                wire["additionalTextEdits"] = item
                    .additional_edits
                    .iter()
                    .map(|e| json!({ "range": e.range, "newText": e.new_text }))
                    .collect();
            }
            // `textEdit` replaces the whole word; `insertText` is for items without a range.
            let text = item.insert_text.as_deref();
            if let Some(range) = &item.replace {
                wire["textEdit"] =
                    json!({ "range": range, "newText": text.unwrap_or(&item.label) });
            } else if let Some(text) = text {
                wire["insertText"] = json!(text);
            }
            if text.is_some() {
                // 2: snippet syntax (`${1:name}`), 1: plain text.
                wire["insertTextFormat"] = json!(if item.kind == CompletionKind::Snippet {
                    2
                } else {
                    1
                });
            }
            wire
        })
        .collect();
    json!({ "isIncomplete": completion.is_incomplete, "items": items })
}

fn semantic_tokens_wire(tokens: &[SemanticToken]) -> Vec<u32> {
    let mut data = Vec::with_capacity(tokens.len().saturating_mul(5));
    let mut previous = Position {
        line: 0,
        character: 0,
    };
    for token in tokens {
        let kind = token.kind as u32;
        let line_delta = token.position.line - previous.line;
        data.extend([
            line_delta,
            if line_delta == 0 {
                token.position.character - previous.character
            } else {
                token.position.character
            },
            token.length,
            kind,
            u32::from(token.declaration)
                | (u32::from(token.readonly) << 1)
                | (u32::from(token.expand) << 2),
        ]);
        previous = token.position;
    }
    data
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn token_types_are_listed_in_wire_order() {
        for (index, kind) in SemanticTokenKind::ALL.iter().enumerate() {
            assert_eq!(*kind as usize, index, "{kind:?} is out of legend order");
        }
        let names: BTreeSet<&str> = SemanticTokenKind::ALL.iter().map(|k| k.name()).collect();
        assert_eq!(names.len(), SemanticTokenKind::ALL.len());
    }

    #[test]
    fn instructions_are_not_keywords() {
        assert_ne!(
            completion_kind_wire(CompletionKind::Instruction),
            completion_kind_wire(CompletionKind::Keyword)
        );
    }

    #[test]
    fn method_tables_have_no_duplicates() {
        let requests: BTreeSet<&str> = Request::METHODS.iter().map(|(name, _)| *name).collect();
        assert_eq!(requests.len(), Request::METHODS.len());
        let notifications: BTreeSet<&str> = Notification::METHODS
            .iter()
            .map(|(name, _)| *name)
            .collect();
        assert_eq!(notifications.len(), Notification::METHODS.len());
        assert!(requests.is_disjoint(&notifications));
    }
}
