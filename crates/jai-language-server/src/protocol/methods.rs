//! Method tables for requests, notifications and server commands, and message envelopes.


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
        (
            gen_lsp_types::LspRequestMethod::Initialize.as_str(),
            Self::Initialize,
        ),
        (
            gen_lsp_types::LspRequestMethod::Shutdown.as_str(),
            Self::Shutdown,
        ),
        (
            gen_lsp_types::LspRequestMethod::TextDocumentSemanticTokensFull.as_str(),
            Self::SemanticTokens,
        ),
        (
            gen_lsp_types::LspRequestMethod::TextDocumentSemanticTokensRange.as_str(),
            Self::SemanticTokensRange,
        ),
        (
            gen_lsp_types::LspRequestMethod::TextDocumentSemanticTokensFullDelta.as_str(),
            Self::SemanticTokensDelta,
        ),
        (
            gen_lsp_types::LspRequestMethod::TextDocumentDocumentSymbol.as_str(),
            Self::DocumentSymbol,
        ),
        (
            gen_lsp_types::LspRequestMethod::TextDocumentHover.as_str(),
            Self::Hover,
        ),
        (
            gen_lsp_types::LspRequestMethod::TextDocumentCompletion.as_str(),
            Self::Completion,
        ),
        (
            gen_lsp_types::LspRequestMethod::CompletionItemResolve.as_str(),
            Self::CompletionResolve,
        ),
        (
            gen_lsp_types::LspRequestMethod::TextDocumentDefinition.as_str(),
            Self::Definition,
        ),
        (
            gen_lsp_types::LspRequestMethod::TextDocumentTypeDefinition.as_str(),
            Self::TypeDefinition,
        ),
        (
            gen_lsp_types::LspRequestMethod::TextDocumentDeclaration.as_str(),
            Self::Declaration,
        ),
        (
            gen_lsp_types::LspRequestMethod::TextDocumentPrepareTypeHierarchy.as_str(),
            Self::PrepareTypeHierarchy,
        ),
        (
            gen_lsp_types::LspRequestMethod::TypeHierarchySupertypes.as_str(),
            Self::Supertypes,
        ),
        (
            gen_lsp_types::LspRequestMethod::TypeHierarchySubtypes.as_str(),
            Self::Subtypes,
        ),
        (
            gen_lsp_types::LspRequestMethod::TextDocumentLinkedEditingRange.as_str(),
            Self::LinkedEditingRange,
        ),
        (
            gen_lsp_types::LspRequestMethod::TextDocumentImplementation.as_str(),
            Self::Implementation,
        ),
        (
            gen_lsp_types::LspRequestMethod::TextDocumentFormatting.as_str(),
            Self::Formatting,
        ),
        (
            gen_lsp_types::LspRequestMethod::TextDocumentRangeFormatting.as_str(),
            Self::RangeFormatting,
        ),
        (
            gen_lsp_types::LspRequestMethod::TextDocumentReferences.as_str(),
            Self::References,
        ),
        (
            gen_lsp_types::LspRequestMethod::TextDocumentPrepareRename.as_str(),
            Self::PrepareRename,
        ),
        (
            gen_lsp_types::LspRequestMethod::TextDocumentRename.as_str(),
            Self::Rename,
        ),
        (
            gen_lsp_types::LspRequestMethod::TextDocumentDocumentHighlight.as_str(),
            Self::DocumentHighlight,
        ),
        (
            gen_lsp_types::LspRequestMethod::WorkspaceSymbol.as_str(),
            Self::WorkspaceSymbol,
        ),
        (
            gen_lsp_types::LspRequestMethod::TextDocumentDocumentLink.as_str(),
            Self::DocumentLink,
        ),
        (
            gen_lsp_types::LspRequestMethod::TextDocumentFoldingRange.as_str(),
            Self::FoldingRange,
        ),
        (
            gen_lsp_types::LspRequestMethod::TextDocumentInlayHint.as_str(),
            Self::InlayHint,
        ),
        (
            gen_lsp_types::LspRequestMethod::TextDocumentSignatureHelp.as_str(),
            Self::SignatureHelp,
        ),
        (
            gen_lsp_types::LspRequestMethod::TextDocumentCodeAction.as_str(),
            Self::CodeAction,
        ),
        (
            gen_lsp_types::LspRequestMethod::TextDocumentPrepareCallHierarchy.as_str(),
            Self::PrepareCallHierarchy,
        ),
        (
            gen_lsp_types::LspRequestMethod::CallHierarchyIncomingCalls.as_str(),
            Self::IncomingCalls,
        ),
        (
            gen_lsp_types::LspRequestMethod::CallHierarchyOutgoingCalls.as_str(),
            Self::OutgoingCalls,
        ),
        (
            gen_lsp_types::LspRequestMethod::TextDocumentSelectionRange.as_str(),
            Self::SelectionRange,
        ),
        (
            gen_lsp_types::LspRequestMethod::TextDocumentCodeLens.as_str(),
            Self::CodeLens,
        ),
        (
            gen_lsp_types::LspRequestMethod::WorkspaceExecuteCommand.as_str(),
            Self::ExecuteCommand,
        ),
        (
            gen_lsp_types::LspRequestMethod::TextDocumentDiagnostic.as_str(),
            Self::DocumentDiagnostic,
        ),
        (
            gen_lsp_types::LspRequestMethod::WorkspaceDiagnostic.as_str(),
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
            gen_lsp_types::LspNotificationMethod::Initialized.as_str(),
            Self::Initialized,
        ),
        (
            gen_lsp_types::LspNotificationMethod::Exit.as_str(),
            Self::Exit,
        ),
        (
            gen_lsp_types::LspNotificationMethod::CancelRequest.as_str(),
            Self::Cancel,
        ),
        (
            gen_lsp_types::LspNotificationMethod::TextDocumentDidOpen.as_str(),
            Self::DidOpen,
        ),
        (
            gen_lsp_types::LspNotificationMethod::TextDocumentDidChange.as_str(),
            Self::DidChange,
        ),
        (
            gen_lsp_types::LspNotificationMethod::TextDocumentDidSave.as_str(),
            Self::DidSave,
        ),
        (
            gen_lsp_types::LspNotificationMethod::TextDocumentDidClose.as_str(),
            Self::DidClose,
        ),
        (
            gen_lsp_types::LspNotificationMethod::WorkspaceDidChangeConfiguration.as_str(),
            Self::DidChangeConfiguration,
        ),
        (
            gen_lsp_types::LspNotificationMethod::WorkspaceDidChangeWorkspaceFolders.as_str(),
            Self::DidChangeWorkspaceFolders,
        ),
        (
            gen_lsp_types::LspNotificationMethod::WorkspaceDidChangeWatchedFiles.as_str(),
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
