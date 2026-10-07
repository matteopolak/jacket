//! Bounded source analysis shared by native LSP and the actual WebAssembly bridge.
mod analysis;
mod document;
pub(crate) mod features;
pub(crate) mod format;
pub mod framing;
pub(crate) mod hover;
mod imports;
mod links;
pub mod lints;
mod model;
mod position;
mod protocol;
mod semantic;
mod session;

pub use document::{DocumentUri, TextChange, VirtualSources};
pub use model::{
    CodeAction, CodeLens, Command, CompletionItem, CompletionKind, CompletionList, Diagnostic,
    DiagnosticCode, DiagnosticSeverity, DocumentSymbol, Expansion, FoldingRange, Hover, InlayHint,
    InlayHintKind, Location, MarkupContent, MarkupKind, SemanticToken, SemanticTokenKind,
    SignatureHelp, SignatureInformation, SymbolInformation, SymbolKind, TextEdit,
};
pub use position::{Position, Range};
pub use protocol::{JsonSession, ProtocolError, RequestId};
pub use semantic::Environment;
pub use session::Session;

#[derive(Clone, Copy, Debug)]
pub struct Limits {
    pub documents: usize,
    pub document_bytes: usize,
    pub workspace_bytes: usize,
    pub message_bytes: usize,
    pub output_bytes: usize,
    pub tokens: usize,
    pub recursive_tokens: usize,
    pub symbols: usize,
    pub edits: usize,
    pub cancelled_requests: usize,
}

impl Default for Limits {
    fn default() -> Self {
        Self {
            documents: 32,
            document_bytes: 256 * 1024,
            workspace_bytes: 4 * 1024 * 1024,
            message_bytes: 1024 * 1024,
            output_bytes: 2 * 1024 * 1024,
            tokens: 8192,
            recursive_tokens: 96,
            symbols: 1024,
            edits: 128,
            cancelled_requests: 128,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Error {
    Uri(&'static str),
    Position(&'static str),
    Limit(&'static str),
    MissingDocument,
    AlreadyOpen,
    StaleVersion,
    InvalidEdit(&'static str),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let text = match self {
            Self::Uri(t) | Self::Position(t) | Self::Limit(t) | Self::InvalidEdit(t) => *t,
            Self::MissingDocument => "document is not open in this session",
            Self::AlreadyOpen => "document is already open",
            Self::StaleVersion => "document version must strictly increase",
        };
        f.write_str(text)
    }
}

impl std::error::Error for Error {
}

pub const TOKEN_TYPES: &[&str] = &[
    "keyword",
    "string",
    "number",
    "variable",
    "function",
    "type",
    "property",
    "parameter",
    "macro",
    "operator",
    "namespace",
    "typeParameter",
    "enumMember",
    "decorator",
    "formatSpecifier",
];

/// `macro` marks a procedure declared `#expand`.
pub const TOKEN_MODIFIERS: &[&str] = &["declaration", "readonly", "macro"];

/// Commands `workspace/executeCommand` runs.
pub const COMMANDS: &[&str] = &["jai.showExpansion", "jai.showPolymorphs"];
