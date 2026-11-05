//! Method tables for requests, notifications and server commands, and message envelopes.


use lsp_types::notification::Notification as _;
use lsp_types::request::Request as _;

pub(super) use lsp_server::{ErrorCode, ResponseError as Failure};

pub(super) const OUTPUT_LIMIT: &str = "The response is larger than the output limit";

/// The requests the server answers.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Request {
    Initialize,
    Shutdown,
    SemanticTokens,
    SemanticTokensRange,
    SemanticTokensDelta,
    DocumentSymbol,
    Hover,
    Completion,
    CompletionResolve,
    Definition,
    TypeDefinition,
    Declaration,
    PrepareTypeHierarchy,
    Supertypes,
    Subtypes,
    LinkedEditingRange,
    Implementation,
    Formatting,
    RangeFormatting,
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
        (lsp_types::request::Initialize::METHOD, Self::Initialize),
        (lsp_types::request::Shutdown::METHOD, Self::Shutdown),
        (
            lsp_types::request::SemanticTokensFullRequest::METHOD,
            Self::SemanticTokens,
        ),
        (
            lsp_types::request::SemanticTokensRangeRequest::METHOD,
            Self::SemanticTokensRange,
        ),
        (
            lsp_types::request::SemanticTokensFullDeltaRequest::METHOD,
            Self::SemanticTokensDelta,
        ),
        (
            lsp_types::request::DocumentSymbolRequest::METHOD,
            Self::DocumentSymbol,
        ),
        (lsp_types::request::HoverRequest::METHOD, Self::Hover),
        (lsp_types::request::Completion::METHOD, Self::Completion),
        (
            lsp_types::request::ResolveCompletionItem::METHOD,
            Self::CompletionResolve,
        ),
        (lsp_types::request::GotoDefinition::METHOD, Self::Definition),
        (
            lsp_types::request::GotoTypeDefinition::METHOD,
            Self::TypeDefinition,
        ),
        (
            lsp_types::request::GotoDeclaration::METHOD,
            Self::Declaration,
        ),
        (
            lsp_types::request::TypeHierarchyPrepare::METHOD,
            Self::PrepareTypeHierarchy,
        ),
        (
            lsp_types::request::TypeHierarchySupertypes::METHOD,
            Self::Supertypes,
        ),
        (
            lsp_types::request::TypeHierarchySubtypes::METHOD,
            Self::Subtypes,
        ),
        (
            lsp_types::request::LinkedEditingRange::METHOD,
            Self::LinkedEditingRange,
        ),
        (
            lsp_types::request::GotoImplementation::METHOD,
            Self::Implementation,
        ),
        (lsp_types::request::Formatting::METHOD, Self::Formatting),
        (
            lsp_types::request::RangeFormatting::METHOD,
            Self::RangeFormatting,
        ),
        (lsp_types::request::References::METHOD, Self::References),
        (
            lsp_types::request::PrepareRenameRequest::METHOD,
            Self::PrepareRename,
        ),
        (lsp_types::request::Rename::METHOD, Self::Rename),
        (
            lsp_types::request::DocumentHighlightRequest::METHOD,
            Self::DocumentHighlight,
        ),
        (
            lsp_types::request::WorkspaceSymbolRequest::METHOD,
            Self::WorkspaceSymbol,
        ),
        (
            lsp_types::request::DocumentLinkRequest::METHOD,
            Self::DocumentLink,
        ),
        (
            lsp_types::request::FoldingRangeRequest::METHOD,
            Self::FoldingRange,
        ),
        (
            lsp_types::request::InlayHintRequest::METHOD,
            Self::InlayHint,
        ),
        (
            lsp_types::request::SignatureHelpRequest::METHOD,
            Self::SignatureHelp,
        ),
        (
            lsp_types::request::CodeActionRequest::METHOD,
            Self::CodeAction,
        ),
        (
            lsp_types::request::CallHierarchyPrepare::METHOD,
            Self::PrepareCallHierarchy,
        ),
        (
            lsp_types::request::CallHierarchyIncomingCalls::METHOD,
            Self::IncomingCalls,
        ),
        (
            lsp_types::request::CallHierarchyOutgoingCalls::METHOD,
            Self::OutgoingCalls,
        ),
        (
            lsp_types::request::SelectionRangeRequest::METHOD,
            Self::SelectionRange,
        ),
        (lsp_types::request::CodeLensRequest::METHOD, Self::CodeLens),
        (
            lsp_types::request::ExecuteCommand::METHOD,
            Self::ExecuteCommand,
        ),
        (
            lsp_types::request::DocumentDiagnosticRequest::METHOD,
            Self::DocumentDiagnostic,
        ),
        (
            lsp_types::request::WorkspaceDiagnosticRequest::METHOD,
            Self::WorkspaceDiagnostic,
        ),
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
    DidSave,
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
        (
            lsp_types::notification::Initialized::METHOD,
            Self::Initialized,
        ),
        (lsp_types::notification::Exit::METHOD, Self::Exit),
        (lsp_types::notification::Cancel::METHOD, Self::Cancel),
        (
            lsp_types::notification::DidOpenTextDocument::METHOD,
            Self::DidOpen,
        ),
        (
            lsp_types::notification::DidChangeTextDocument::METHOD,
            Self::DidChange,
        ),
        (
            lsp_types::notification::DidSaveTextDocument::METHOD,
            Self::DidSave,
        ),
        (
            lsp_types::notification::DidCloseTextDocument::METHOD,
            Self::DidClose,
        ),
        (
            lsp_types::notification::DidChangeConfiguration::METHOD,
            Self::DidChangeConfiguration,
        ),
        (
            lsp_types::notification::DidChangeWorkspaceFolders::METHOD,
            Self::DidChangeWorkspaceFolders,
        ),
        (
            lsp_types::notification::DidChangeWatchedFiles::METHOD,
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
    Expansion,
    Polymorphs,
    CompileTimeCosts,
}

impl ServerCommand {
    pub(super) const NAMES: &[(&str, Self)] = &[
        ("jai.showExpansion", Self::Expansion),
        ("jai.showPolymorphs", Self::Polymorphs),
        ("jai.showCompileTimeCosts", Self::CompileTimeCosts),
    ];

    pub(super) fn parse(name: &str) -> Option<Self> {
        Self::NAMES
            .iter()
            .find(|(known, _)| *known == name)
            .map(|(_, command)| *command)
    }
}

/// Debug builds panic on `jai/debugPanic`, to test that a panic is one message's error.
pub(super) fn debug_panic(method: &str) {
    #[cfg(debug_assertions)]
    assert!(method != "jai/debugPanic", "requested by jai/debugPanic");
    #[cfg(not(debug_assertions))]
    let _ = method;
}
