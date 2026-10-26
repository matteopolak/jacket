//! Encoding results into the protocol's JSON shapes.
use super::helpers::array;
use super::methods::ServerCommand;
use crate::{
    CallHierarchyCall, CallHierarchyItem, CodeAction, CodeLens, Command, Diagnostic,
    DiagnosticCode, DiagnosticSeverity, DocumentSymbol, Expansion, FoldingRange, Hover, InlayHint,
    InlayHintKind, Location, MarkupKind, Position, SelectionRange, SemanticToken,
    SemanticTokenKind, SignatureHelp, SymbolInformation, SymbolKind, TOKEN_MODIFIERS,
};
use serde_json::{Value, json};

// Domain enums have no protocol discriminants. Only this JSON boundary maps LSP values.
/// A document's diagnostics for a pull: the report is `unchanged` when the client's last
/// result id still describes them. `uri` is set for the items of a workspace report.
pub(super) fn document_report(
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

pub(super) fn diagnostic_wire(diagnostic: &Diagnostic) -> Value {
    let severity = match diagnostic.severity {
        DiagnosticSeverity::Error => 1,
        DiagnosticSeverity::Warning => 2,
        DiagnosticSeverity::Hint => 4,
    };
    let (code, source) = diagnostic.code.wire();
    let mut value = json!({
        "range": diagnostic.range,
        "severity": severity,
        "code": code,
        "source": source,
        "message": diagnostic.message,
    });
    if !diagnostic.related.is_empty() {
        value["relatedInformation"] = diagnostic
            .related
            .iter()
            .map(|r| json!({"location": {"uri": r.uri, "range": r.range}, "message": r.message}))
            .collect();
    }
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
pub(super) struct SymbolsWire<'a>(pub(super) &'a [DocumentSymbol]);

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

pub(super) fn hierarchy_item_wire(item: &CallHierarchyItem) -> Value {
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
pub(super) fn hierarchy_call_wire(call: &CallHierarchyCall, side: &str) -> Value {
    json!({ side: hierarchy_item_wire(&call.item), "fromRanges": call.from_ranges })
}

pub(super) fn selection_wire(selection: &SelectionRange) -> Value {
    let mut value = json!({ "range": selection.range });
    if let Some(parent) = &selection.parent {
        value["parent"] = selection_wire(parent);
    }
    value
}

fn location_wire(location: &Location) -> Value {
    json!({ "uri": location.uri, "range": location.range })
}

pub(super) fn locations_wire(locations: &[Location]) -> Value {
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

pub(super) fn symbol_information_wire(symbol: &SymbolInformation) -> Value {
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

pub(super) fn folding_wire(range: &FoldingRange) -> Value {
    let mut value = json!({ "startLine": range.start_line, "endLine": range.end_line });
    if range.imports {
        value["kind"] = json!("imports");
    }
    value
}

pub(super) fn inlay_wire(hint: &InlayHint) -> Value {
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

pub(super) fn signature_wire(help: &SignatureHelp, kind: MarkupKind) -> Value {
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
    json!({ "title": command.title, "command": command.command, "arguments": command.arguments })
}

pub(super) fn code_action_wire(action: &CodeAction) -> Value {
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

pub(super) fn code_lens_wire(lens: &CodeLens) -> Value {
    json!({ "range": lens.range, "command": command_wire(&lens.command) })
}

pub(super) fn expansion_wire(expansion: &Expansion) -> Value {
    json!({
        "uri": expansion.uri,
        "kind": expansion.kind,
        "text": expansion.text,
        "source": location_wire(&expansion.source),
    })
}

/// The documentation format a client asked for signature help.
pub(super) fn signature_kind(params: &Value) -> MarkupKind {
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
pub(super) fn capabilities(pull_diagnostics: bool) -> Value {
    let token_types: Vec<&str> = SemanticTokenKind::ALL.iter().map(|k| k.name()).collect();
    let commands: Vec<&str> = ServerCommand::NAMES.iter().map(|(name, _)| *name).collect();
    let mut capabilities = json!({
        "positionEncoding": "utf-16",
        "textDocumentSync": { "openClose": true, "change": 2, "save": true },
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

pub(super) fn workspace_edit_wire(documents: crate::features::RenameEdits) -> Value {
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

pub(super) fn hover_kind(params: &Value) -> MarkupKind {
    let formats = params
        .pointer("/capabilities/textDocument/hover/contentFormat")
        .and_then(Value::as_array);
    match formats {
        Some(formats) if formats.iter().any(|f| f == "markdown") => MarkupKind::Markdown,
        _ => MarkupKind::PlainText,
    }
}

pub(super) fn hover_wire(hover: &Hover) -> Value {
    let kind = match hover.contents.kind {
        MarkupKind::PlainText => "plaintext",
        MarkupKind::Markdown => "markdown",
    };
    json!({
        "contents": { "kind": kind, "value": hover.contents.value },
        "range": hover.range,
    })
}

pub(super) fn semantic_tokens_wire(tokens: &[SemanticToken]) -> Vec<u32> {
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
