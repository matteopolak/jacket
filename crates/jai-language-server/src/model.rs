//! Typed source-analysis results. LSP wire numbers and strings live in protocol.rs.
use crate::{Position, Range};

/// The kind of a symbol, which decides its icon.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SymbolKind {
    File,
    Namespace,
    TypeAlias,
    Property,
    Enum,
    Function,
    Variable,
    Constant,
    EnumMember,
    Struct,
    Boolean,
    Event,
}

/// How serious a diagnostic is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DiagnosticSeverity {
    Error,
    Warning,
    /// A fact about the file, not a defect in it.
    Information,
    /// Shown without a squiggle (code faded as unreachable).
    Hint,
}

/// Which part of the toolchain a diagnostic comes from, and its name there.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DiagnosticCode {
    Lexer,
    Parser,
    /// A `#load` target the session has no text for.
    Source,
    /// A token or recursion budget was exceeded.
    Limit,
    /// The type checker's first error in the program.
    Check,
    /// No program entry loads or imports the file, so it is not checked as part of one.
    NotInProgram,
    /// Code no path reaches: the editor fades it (`DiagnosticTag::Unnecessary`).
    Unreachable,
    /// A jailint rule, by name.
    Lint(&'static str),
    /// A problem in a `jacket.toml`.
    Config,
}

impl DiagnosticCode {
    /// The `code` and `source` of the diagnostic on the wire.
    pub(crate) fn wire(self) -> (&'static str, &'static str) {
        match self {
            Self::Lexer => ("lexer", "jaic"),
            Self::Parser => ("parser", "jaic"),
            Self::Source => ("source", "jaic"),
            Self::Limit => ("limit", "jaic"),
            Self::Check => ("check", "jaic"),
            Self::NotInProgram => ("not-in-program", "jaic"),
            Self::Unreachable => ("unreachable", "jaic"),
            Self::Lint(rule) => (rule, "jailint"),
            Self::Config => ("config", "jacket"),
        }
    }
}

/// The kind of a completion item, which decides its icon.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CompletionKind {
    Function,
    #[default]
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
    /// An `#asm` instruction mnemonic.
    Instruction,
    /// A template with placeholders (`insert_text` is a snippet).
    Snippet,
}

/// A problem or note about a range of a document.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Diagnostic {
    pub range: Range,
    pub severity: DiagnosticSeverity,
    pub code: DiagnosticCode,
    pub message: String,
    /// Places the message refers to (`relatedInformation`).
    pub related: Vec<RelatedInformation>,
}

/// A note of a diagnostic that has a place: another location and what it says.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RelatedInformation {
    pub uri: String,
    pub range: Range,
    pub message: String,
}

/// A declaration in a document, with the declarations nested in it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DocumentSymbol {
    pub name: String,
    pub detail: String,
    pub kind: SymbolKind,
    /// The whole declaration.
    pub range: Range,
    /// The name, which is inside `range`.
    pub selection_range: Range,
    pub children: Vec<DocumentSymbol>,
}

/// A range of a document, which `uri` names.
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

/// Text with the markup it is written in.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MarkupContent {
    pub kind: MarkupKind,
    pub value: String,
}

/// What to show for a hovered place, and the range it covers.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Hover {
    pub contents: MarkupContent,
    pub range: Range,
}

/// One entry of a completion list.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CompletionItem {
    pub label: String,
    pub kind: CompletionKind,
    pub detail: String,
    /// Markdown shown next to the list.
    pub documentation: Option<String>,
    /// What to insert instead of the label: an LSP snippet for `CompletionKind::Snippet`.
    pub insert_text: Option<String>,
    /// The range accepting the item replaces (`textEdit`): the whole word being completed,
    /// the part after the cursor included. Without it the client picks the range itself.
    pub replace: Option<Range>,
    /// Shown after the label, dimmed (`labelDetails.description`): an auto-import's module.
    pub label_description: Option<String>,
    /// Sorts the item among the others instead of its label (`sortText`).
    pub sort_text: Option<String>,
    /// Edits elsewhere in the document made when the item is accepted (an auto-import's
    /// `#import`).
    pub additional_edits: Vec<TextEdit>,
    /// Where the item is declared, so its documentation can be read when the client asks
    /// (`completionItem/resolve`) instead of for every item of a long list.
    pub data: Option<CompletionData>,
    /// How a procedure is called: completion inserts the call (`Session::call_snippets`).
    pub call: Option<jaic::ide::IdeCall>,
    /// What the client matches the typed text against instead of the label (`filterText`): a
    /// postfix item replaces the receiver too, so the receiver is part of what was typed.
    pub filter_text: Option<String>,
    /// `insert_text` is a snippet, whatever the kind.
    pub snippet: bool,
    /// Accepting the item asks the client to show signature help (the cursor is inside the
    /// parentheses it inserted).
    pub trigger_signature_help: bool,
    /// The client selects this item when the list opens (`preselect`), whatever it remembers.
    pub preselect: bool,
}

/// The declaration a completion item stands for.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CompletionData {
    /// The document the completion was asked in (its scope resolves the docs' links).
    pub document: String,
    /// The file declaring the item, and the offset of the declaration's name in it.
    pub path: String,
    pub offset: usize,
}

/// The completion items at a place.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CompletionList {
    /// The client asks again as the typed word changes, because `items` was cut short.
    pub is_incomplete: bool,
    pub items: Vec<CompletionItem>,
}

/// What a semantic token is, which decides how the editor colors it.
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
    /// `%`, `%N` in the format string of a `@PrintLike` call.
    FormatSpecifier,
}

impl SemanticTokenKind {
    /// Every kind, in the order of the legend: a token's wire type is `kind as u32`.
    pub const ALL: [Self; 15] = [
        Self::Keyword,
        Self::String,
        Self::Number,
        Self::Variable,
        Self::Function,
        Self::Type,
        Self::Property,
        Self::Parameter,
        Self::Macro,
        Self::Operator,
        Self::Namespace,
        Self::TypeParameter,
        Self::EnumMember,
        Self::Decorator,
        Self::FormatSpecifier,
    ];

    /// The LSP token type name in the legend.
    pub fn name(self) -> &'static str {
        match self {
            Self::Keyword => "keyword",
            Self::String => "string",
            Self::Number => "number",
            Self::Variable => "variable",
            Self::Function => "function",
            Self::Type => "type",
            Self::Property => "property",
            Self::Parameter => "parameter",
            Self::Macro => "macro",
            Self::Operator => "operator",
            Self::Namespace => "namespace",
            Self::TypeParameter => "typeParameter",
            Self::EnumMember => "enumMember",
            Self::Decorator => "decorator",
            Self::FormatSpecifier => "formatSpecifier",
        }
    }
}

/// A classified run of text on one line.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SemanticToken {
    pub position: Position,
    /// In UTF-16 code units.
    pub length: u32,
    pub kind: SemanticTokenKind,
    pub declaration: bool,
    pub readonly: bool,
    /// A procedure declared `#expand` (a macro).
    pub expand: bool,
}

/// What an inlay hint shows.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InlayHintKind {
    Type,
    Parameter,
}

/// Text the editor draws inline at `position`, such as an inferred type or a parameter name.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InlayHint {
    pub position: Position,
    pub label: String,
    pub kind: Option<InlayHintKind>,
    /// The editor leaves a space before or after the hint.
    pub padding_left: bool,
    pub padding_right: bool,
    pub tooltip: Option<String>,
}

/// A replacement of `range` with `new_text`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TextEdit {
    pub range: Range,
    pub new_text: String,
}

/// A command the client runs (`workspace/executeCommand` for the server's own, else one the
/// client registered), called with `arguments`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Command {
    pub title: String,
    pub command: String,
    pub arguments: Vec<serde_json::Value>,
}

impl Command {
    /// A command of the server that works on the place `{uri, position}`.
    pub fn at(title: String, command: &str, uri: &str, position: Position) -> Self {
        Self {
            title,
            command: command.into(),
            arguments: vec![serde_json::json!({ "uri": uri, "position": position })],
        }
    }
}

/// A fix or refactoring the editor offers.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CodeAction {
    pub title: String,
    /// The LSP code action kind (`quickfix`), if any.
    pub kind: Option<&'static str>,
    /// Edits of one document.
    pub edit: Option<(String, Vec<TextEdit>)>,
    pub command: Option<Command>,
    /// The diagnostics the action resolves.
    pub diagnostics: Vec<Diagnostic>,
    /// The action to take when several fix the same thing (a lint's safe fix).
    pub is_preferred: bool,
    /// The jailint rule whose fix this is (`data.rule` on the wire).
    pub rule: Option<&'static str>,
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

/// One overload of a procedure, for signature help.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SignatureInformation {
    pub label: String,
    /// Each parameter's text, a substring of `label`.
    pub parameters: Vec<String>,
    /// The procedure's doc comment.
    pub documentation: Option<String>,
    /// What each parameter's doc comment says (parallel to `parameters`).
    pub parameter_docs: Vec<Option<String>>,
}

/// The overloads of the procedure being called, and which of them and which parameter the
/// cursor is on.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SignatureHelp {
    pub signatures: Vec<SignatureInformation>,
    /// Index into `signatures`.
    pub active_signature: usize,
    /// Index into the active signature's `parameters`.
    pub active_parameter: usize,
}

/// A range of whole lines the editor can fold.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FoldingRange {
    pub start_line: u32,
    pub end_line: u32,
    /// `imports` for a run of `#import`/`#load` lines.
    pub imports: bool,
}

/// A command shown above a line, for the code in `range`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CodeLens {
    pub range: Range,
    pub command: Command,
}

/// A declaration found by a workspace-wide search.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SymbolInformation {
    pub name: String,
    pub kind: SymbolKind,
    pub location: Location,
    /// The name of the declaration it is nested in.
    pub container: Option<String>,
}

/// A procedure in the call hierarchy.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CallHierarchyItem {
    pub name: String,
    /// The procedure's header, `(a: int) -> int`.
    pub detail: String,
    pub kind: SymbolKind,
    pub uri: String,
    /// The whole declaration.
    pub range: Range,
    /// The name.
    pub selection_range: Range,
}

/// A type in the type hierarchy: the same shape as a procedure of the call hierarchy.
pub type TypeHierarchyItem = CallHierarchyItem;

/// A caller or a callee, with the places in the caller where the call is written.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CallHierarchyCall {
    pub item: CallHierarchyItem,
    pub from_ranges: Vec<Range>,
}

/// A range to select, and the larger one around it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SelectionRange {
    pub range: Range,
    pub parent: Option<Box<SelectionRange>>,
}
