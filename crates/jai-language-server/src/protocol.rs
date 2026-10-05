//! Standard JSON parsing belongs at this portable boundary, not in source analysis.
use crate::{
    CompletionKind, CompletionList, Diagnostic, DiagnosticCode, DiagnosticSeverity, DocumentSymbol,
    DocumentUri, Error, Hover, Limits, Location, Position, SemanticToken, SemanticTokenKind,
    Session, SymbolKind, TOKEN_MODIFIERS, TOKEN_TYPES, TextChange,
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::{BTreeSet, VecDeque};

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
    pub fn handle_json(&mut self, input: &str) -> Result<Vec<String>, ProtocolError> {
        if input.len() > self.session.limits().message_bytes {
            return Err(ProtocolError::MessageLimit);
        }
        let value: Value = match serde_json::from_str(input) {
            Ok(value) => value,
            Err(_) => return self.encode(vec![failure(Value::Null, -32700, "Parse error")]),
        };
        let raw_id = value.get("id");
        let id = match raw_id {
            Some(value) => match serde_json::from_value::<RequestId>(value.clone()) {
                Ok(id) if valid_id(&id) => Some(id),
                _ => return self.encode(vec![failure(Value::Null, -32600, "Invalid request id")]),
            },
            None => None,
        };
        let envelope: Envelope = match serde_json::from_value::<Envelope>(value) {
            Ok(e) if e.jsonrpc == "2.0" && !e.method.is_empty() => e,
            _ => return self.encode(vec![failure(Value::Null, -32600, "Invalid request")]),
        };
        let identifier = id.as_ref().map_or(Value::Null, |id| json!(id));
        if envelope.method == "$/cancelRequest" && id.is_none() {
            let result = serde_json::from_value::<CancelParams>(envelope.params)
                .map_err(|_| Error::InvalidEdit("invalid cancellation parameters"))
                .and_then(|params| self.cancel_request(params.id));
            return self.encode(
                result
                    .err()
                    .map(|e| log_error(&e.to_string()))
                    .into_iter()
                    .collect(),
            );
        }
        if envelope.method == "exit" && id.is_none() {
            self.lifecycle = Lifecycle::Exited(if self.lifecycle == Lifecycle::Shutdown {
                0
            } else {
                1
            });
            return Ok(vec![]);
        }
        if let Some(id) = &id
            && self.cancelled.remove(id)
        {
            self.remember(id.clone());
            return self.encode(vec![failure(identifier, -32800, "Request cancelled")]);
        }
        let result = self.dispatch(&envelope.method, envelope.params, id.is_some());
        let output = match result {
            Ok(mut messages) => {
                if let Some(id) = id {
                    self.remember(id);
                    let result = messages.pop().unwrap_or(Value::Null);
                    messages.push(json!({"jsonrpc":"2.0","id":identifier,"result":result}));
                }
                messages
            }
            Err((code, message)) => {
                if let Some(id) = id {
                    self.remember(id);
                    vec![failure(identifier, code, &message)]
                } else if self.lifecycle == Lifecycle::New {
                    vec![]
                } else {
                    vec![log_error(&message)]
                }
            }
        };
        self.encode(output)
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
    fn encode(&self, messages: Vec<Value>) -> Result<Vec<String>, ProtocolError> {
        let mut out = vec![];
        let mut bytes = 0usize;
        for message in messages {
            let text = serde_json::to_string(&message)
                .map_err(|e| ProtocolError::Serialization(e.to_string()))?;
            bytes = bytes
                .checked_add(text.len())
                .filter(|n| *n <= self.session.limits().output_bytes)
                .ok_or(ProtocolError::OutputLimit)?;
            out.push(text);
        }
        Ok(out)
    }
    fn dispatch(
        &mut self,
        method: &str,
        params: Value,
        request: bool,
    ) -> Result<Vec<Value>, (i32, String)> {
        if method == "initialize" && request {
            if self.lifecycle != Lifecycle::New {
                return Err((-32600, "Server is already initialized".into()));
            }
            if !params.is_object() && !params.is_null() {
                return Err((-32602, "Initialize parameters must be an object".into()));
            }
            self.lifecycle = Lifecycle::Running;
            return Ok(vec![
                json!({"capabilities":{"positionEncoding":"utf-16","textDocumentSync":{"openClose":true,"change":2},"hoverProvider":true,
                "completionProvider":{"resolveProvider":false},"definitionProvider":true,"documentSymbolProvider":true,
                "semanticTokensProvider":{"legend":{"tokenTypes":TOKEN_TYPES,"tokenModifiers":TOKEN_MODIFIERS},"full":true},
                "experimental":{"jai":{"analysis":"compiler-source-syntax","compileTimeExecution":false,"typeInference":false,"filesystemReads":false,"moduleSearch":false}}},
                "serverInfo":{"name":"jai-language-server","version":env!("CARGO_PKG_VERSION")}}),
            ]);
        }
        if self.lifecycle == Lifecycle::New {
            return Err((-32002, "Server is not initialized".into()));
        }
        if self.lifecycle != Lifecycle::Running {
            return Err((-32600, "Server has shut down".into()));
        }
        if method == "shutdown" && request {
            self.lifecycle = Lifecycle::Shutdown;
            return Ok(vec![Value::Null]);
        }
        if request {
            let result = match method {
                "textDocument/semanticTokens/full" => {
                    let p: DocumentParams = decode(params)?;
                    let uri = uri(&p.text_document.uri)?;
                    json!({"resultId":self.session.version(&uri).map_err(domain)?.to_string(),"data":semantic_tokens_wire(&self.session.semantic_tokens(&uri).map_err(domain)?)})
                }
                "textDocument/documentSymbol" => {
                    let p: DocumentParams = decode(params)?;
                    let uri = uri(&p.text_document.uri)?;
                    Value::Array(
                        self.session
                            .document_symbols(&uri)
                            .map_err(domain)?
                            .iter()
                            .map(symbol_wire)
                            .collect(),
                    )
                }
                "textDocument/hover" => {
                    let p: PositionParams = decode(params)?;
                    self.session
                        .hover(&uri(&p.text_document.uri)?, p.position)
                        .map_err(domain)?
                        .as_ref()
                        .map_or(Value::Null, hover_wire)
                }
                "textDocument/completion" => {
                    let p: PositionParams = decode(params)?;
                    completion_wire(
                        &self
                            .session
                            .completion(&uri(&p.text_document.uri)?, p.position)
                            .map_err(domain)?,
                    )
                }
                "textDocument/definition" => {
                    let p: PositionParams = decode(params)?;
                    Value::Array(
                        self.session
                            .definition(&uri(&p.text_document.uri)?, p.position)
                            .map_err(domain)?
                            .iter()
                            .map(location_wire)
                            .collect(),
                    )
                }
                _ => return Err((-32601, "Method not supported".into())),
            };
            return Ok(vec![result]);
        }
        match method {
            "initialized" => return Ok(vec![]),
            "textDocument/didOpen" => {
                let p: OpenParams = decode(params)?;
                if p.text_document.language_id != "jai" {
                    return Err((-32602, "Only Jai documents are supported".into()));
                }
                self.session
                    .open(
                        uri(&p.text_document.uri)?,
                        p.text_document.version,
                        p.text_document.text,
                    )
                    .map_err(domain)?;
            }
            "textDocument/didChange" => {
                let p: ChangeParams = decode(params)?;
                self.session
                    .change(
                        &uri(&p.text_document.uri)?,
                        p.text_document.version,
                        &p.content_changes,
                    )
                    .map_err(domain)?;
            }
            "textDocument/didClose" => {
                let p: DocumentParams = decode(params)?;
                let closed = uri(&p.text_document.uri)?;
                self.session.close(&closed).map_err(domain)?;
                let mut publications = self.publications();
                publications.push(json!({"jsonrpc":"2.0","method":"textDocument/publishDiagnostics","params":{"uri":closed.as_str(),"diagnostics":[]}}));
                return Ok(publications);
            }
            // Unknown notifications are ignored as required by JSON-RPC/LSP.
            _ => return Ok(vec![]),
        }
        Ok(self.publications())
    }
    fn publications(&self) -> Vec<Value> {
        self.session.publications().into_iter().map(|(uri,version,diagnostics)|json!({"jsonrpc":"2.0","method":"textDocument/publishDiagnostics","params":{"uri":uri,"version":version,"diagnostics":diagnostics.iter().map(diagnostic_wire).collect::<Vec<_>>()}})).collect()
    }
}
fn valid_id(id: &RequestId) -> bool {
    matches!(id, RequestId::Number(_)) || matches!(id,RequestId::String(text) if text.len()<=128)
}
fn decode<T: serde::de::DeserializeOwned>(value: Value) -> Result<T, (i32, String)> {
    serde_json::from_value(value).map_err(|_| (-32602, "Invalid method parameters".into()))
}
fn uri(text: &str) -> Result<DocumentUri, (i32, String)> {
    DocumentUri::parse(text).map_err(domain)
}
fn domain(error: Error) -> (i32, String) {
    let code = match error {
        Error::Limit(_) => -32803,
        Error::StaleVersion => -32801,
        _ => -32602,
    };
    (code, error.to_string())
}
fn failure(id: Value, code: i32, message: &str) -> Value {
    json!({"jsonrpc":"2.0","id":id,"error":{"code":code,"message":message}})
}
fn log_error(message: &str) -> Value {
    json!({"jsonrpc":"2.0","method":"window/logMessage","params":{"type":1,"message":message}})
}

// Domain enums have no protocol discriminants. Only this JSON boundary maps LSP values.
fn diagnostic_wire(diagnostic: &Diagnostic) -> Value {
    let severity = match diagnostic.severity {
        DiagnosticSeverity::Error => 1,
        DiagnosticSeverity::Warning => 2,
    };
    let code = match diagnostic.code {
        DiagnosticCode::Lexer => "jai-lexer",
        DiagnosticCode::Parser => "jai-parser",
        DiagnosticCode::Source => "jai-source",
        DiagnosticCode::Limit => "jai-limit",
    };
    json!({"range":diagnostic.range,"severity":severity,"code":code,"source":"jai","message":diagnostic.message})
}
fn symbol_wire(symbol: &DocumentSymbol) -> Value {
    let kind = match symbol.kind {
        SymbolKind::Namespace => 2,
        SymbolKind::TypeAlias => 5,
        SymbolKind::Property => 7,
        SymbolKind::Enum => 10,
        SymbolKind::Function => 12,
        SymbolKind::Variable => 13,
        SymbolKind::Constant => 14,
        SymbolKind::EnumMember => 22,
        SymbolKind::Struct => 23,
    };
    let mut value = json!({"name":symbol.name,"detail":symbol.detail,"kind":kind,"range":symbol.range,"selectionRange":symbol.selection_range});
    if !symbol.children.is_empty() {
        value["children"] = Value::Array(symbol.children.iter().map(symbol_wire).collect());
    }
    value
}
fn location_wire(location: &Location) -> Value {
    json!({"uri":location.uri,"range":location.range})
}
fn hover_wire(hover: &Hover) -> Value {
    json!({"contents":{"kind":"plaintext","value":hover.contents.value},"range":hover.range})
}
fn completion_wire(completion: &CompletionList) -> Value {
    let items: Vec<Value> = completion
        .items
        .iter()
        .map(|item| {
            let kind = match item.kind {
                CompletionKind::Function => 3,
                CompletionKind::Variable => 6,
                CompletionKind::TypeAlias => 7,
                CompletionKind::Enum => 13,
                CompletionKind::Keyword => 14,
                CompletionKind::Constant => 21,
                CompletionKind::Struct => 22,
                CompletionKind::Module => 9,
                CompletionKind::Field => 5,
                CompletionKind::EnumMember => 20,
            };
            json!({"label":item.label,"kind":kind,"detail":item.detail})
        })
        .collect();
    json!({"isIncomplete":completion.is_incomplete,"items":items})
}

fn semantic_tokens_wire(tokens: &[SemanticToken]) -> Vec<u32> {
    let mut data = Vec::with_capacity(tokens.len().saturating_mul(5));
    let mut previous = Position {
        line: 0,
        character: 0,
    };
    for token in tokens {
        let kind = match token.kind {
            SemanticTokenKind::Keyword => 0,
            SemanticTokenKind::String => 1,
            SemanticTokenKind::Number => 2,
            SemanticTokenKind::Variable => 3,
            SemanticTokenKind::Function => 4,
            SemanticTokenKind::Type => 5,
            SemanticTokenKind::Property => 6,
            SemanticTokenKind::Parameter => 7,
            SemanticTokenKind::Macro => 8,
            SemanticTokenKind::Operator => 9,
        };
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
            u32::from(token.declaration) | (u32::from(token.readonly) << 1),
        ]);
        previous = token.position;
    }
    data
}
