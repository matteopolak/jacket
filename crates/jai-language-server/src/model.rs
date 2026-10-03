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
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MarkupContent {
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
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SemanticToken {
    pub position: Position,
    pub length: u32,
    pub kind: SemanticTokenKind,
    pub declaration: bool,
    pub readonly: bool,
}
