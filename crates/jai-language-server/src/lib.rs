//! Bounded source analysis shared by native LSP and the actual WebAssembly bridge.
mod analysis;
mod asm;
mod auto_import;
mod document;
mod exports;
pub(crate) mod features;
pub(crate) mod format;
pub mod framing;
mod here_string;
mod hierarchy;
pub(crate) mod hover;
mod imports;
#[cfg(unix)]
pub mod large_alloc;
mod links;
pub mod lints;
mod model;
mod modules;
mod position;
mod project;
mod protocol;
mod refactor;
mod selection;
mod semantic;
mod session;

pub use document::{DocumentUri, TextChange, VirtualSources};
pub use model::{
    CallHierarchyCall, CallHierarchyItem, CodeAction, CodeLens, Command, CompletionItem,
    CompletionKind, CompletionList, Diagnostic, DiagnosticCode, DiagnosticSeverity, DocumentSymbol,
    Expansion, FoldingRange, Hover, InlayHint, InlayHintKind, Location, MarkupContent, MarkupKind,
    SelectionRange, SemanticToken, SemanticTokenKind, SignatureHelp, SignatureInformation,
    SymbolInformation, SymbolKind, TextEdit,
};
pub use position::{Position, Range};
pub use protocol::{JsonSession, ProtocolError, RequestId, message_too_large};
pub use semantic::Environment;
pub use session::Session;

const MIB: usize = 1024 * 1024;

/// Safety caps, not working limits: a document or message past one is refused and the user is
/// told, and the server itself never stops for it. The defaults are far above any source file
/// (generated bindings of several MiB are routine); `Limits::browser` is for the playground, which
/// shares one wasm heap with the page.
#[derive(Clone, Copy, Debug)]
pub struct Limits {
    pub documents: usize,
    pub document_bytes: usize,
    pub workspace_bytes: usize,
    pub message_bytes: usize,
    pub output_bytes: usize,
    pub tokens: usize,
    pub recursive_tokens: usize,
    /// Most results of one completion or workspace-symbol query.
    pub symbols: usize,
    /// Declarations recorded per document for navigation (symbols, rename, highlights).
    pub rows: usize,
    pub edits: usize,
    pub cancelled_requests: usize,
}

impl Default for Limits {
    fn default() -> Self {
        Self {
            documents: 512,
            document_bytes: 32 * MIB,
            workspace_bytes: 256 * MIB,
            message_bytes: 64 * MIB,
            output_bytes: 128 * MIB,
            tokens: 64_000_000,
            recursive_tokens: 96,
            symbols: 1024,
            rows: 4_000_000,
            edits: 4096,
            cancelled_requests: 128,
        }
    }
}

impl Limits {
    /// Smaller caps for the browser build.
    pub fn browser() -> Self {
        Self {
            documents: 64,
            document_bytes: 4 * MIB,
            workspace_bytes: 32 * MIB,
            message_bytes: 16 * MIB,
            output_bytes: 32 * MIB,
            tokens: 4_000_000,
            rows: 500_000,
            ..Self::default()
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
