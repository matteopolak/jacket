//! Typed source-analysis results. LSP wire numbers and strings live in protocol.rs.
use crate::{Position, Range};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SymbolKind {
    Namespace,
    TypeAlias,
    Property,
    Enum,
    Function,
    Variable,
    Constant,
    EnumMember,
    Struct,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DiagnosticSeverity {
    Error,
    Warning,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DiagnosticCode {
    Lexer,
    Parser,
    Source,
    Limit,
    /// A format string disagrees with the arguments of its print-family call.
    Format,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CompletionKind {
    Function,
    Variable,
    TypeAlias,
    Enum,
    Keyword,
    Constant,
    Struct,
    Module,
    Field,
    EnumMember,
    File,
    Folder,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Diagnostic {
    pub range: Range,
    pub severity: DiagnosticSeverity,
    pub code: DiagnosticCode,
    pub message: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DocumentSymbol {
    pub name: String,
    pub detail: String,
    pub kind: SymbolKind,
    pub range: Range,
    pub selection_range: Range,
    pub children: Vec<DocumentSymbol>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Location {
    pub uri: String,
    pub range: Range,
}

/// How hover text is written. Clients list what they render in
/// `textDocument.hover.contentFormat`; Markdown is used when it is listed.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum MarkupKind {
    #[default]
    PlainText,
    Markdown,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MarkupContent {
    pub kind: MarkupKind,
    pub value: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Hover {
    pub contents: MarkupContent,
    pub range: Range,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CompletionItem {
    pub label: String,
    pub kind: CompletionKind,
    pub detail: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CompletionList {
    pub is_incomplete: bool,
    pub items: Vec<CompletionItem>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SemanticTokenKind {
    Keyword,
    String,
    Number,
    Variable,
    Function,
    Type,
    Property,
    Parameter,
    Macro,
    Operator,
    /// A module (`Basic :: #import "Basic"`, or used through such a name).
    Namespace,
    /// A polymorphic parameter `$T` and its uses in the procedure.
    TypeParameter,
    EnumMember,
    /// A note, `@name`.
    Decorator,
    /// `%`, `%N` in the format string of a print-family call.
    FormatSpecifier,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SemanticToken {
    pub position: Position,
    pub length: u32,
    pub kind: SemanticTokenKind,
    pub declaration: bool,
    pub readonly: bool,
    /// A procedure declared `#expand` (a macro).
    pub expand: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InlayHintKind {
    Type,
    Parameter,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InlayHint {
    pub position: Position,
    pub label: String,
    pub kind: Option<InlayHintKind>,
    pub padding_left: bool,
    pub padding_right: bool,
    pub tooltip: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TextEdit {
    pub range: Range,
    pub new_text: String,
}

/// A command the client runs with `workspace/executeCommand`; `target` is its argument
/// `{uri, position}`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Command {
    pub title: String,
    pub command: String,
    pub target: Option<(String, Position)>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CodeAction {
    pub title: String,
    pub kind: Option<&'static str>,
    /// Edits of one document.
    pub edit: Option<(String, Vec<TextEdit>)>,
    pub command: Option<Command>,
}

/// Generated code of an `#insert`, `#run`, `#if` or macro call, shown as a read-only document.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Expansion {
    /// `jai-expansion:///path/of/file.jai?LINE:CHARACTER`.
    pub uri: String,
    /// The directive or call the code came from.
    pub source: Location,
    /// `insert`, `run`, `if` or `macro`.
    pub kind: &'static str,
    pub text: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SignatureInformation {
    pub label: String,
    /// Each parameter's text, a substring of `label`.
    pub parameters: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SignatureHelp {
    pub signatures: Vec<SignatureInformation>,
    pub active_signature: usize,
    pub active_parameter: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FoldingRange {
    pub start_line: u32,
    pub end_line: u32,
    /// `imports` for a run of `#import`/`#load` lines.
    pub imports: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CodeLens {
    pub range: Range,
    pub command: Command,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SymbolInformation {
    pub name: String,
    pub kind: SymbolKind,
    pub location: Location,
    pub container: Option<String>,
}
