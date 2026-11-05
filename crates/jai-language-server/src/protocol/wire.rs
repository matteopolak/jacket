//! Encoding results into the protocol's JSON shapes.
use super::methods::ServerCommand;
use crate::{
    CallHierarchyCall, CallHierarchyItem, CodeAction, CodeLens, Command, Diagnostic,
    DiagnosticCode, DiagnosticSeverity, DocumentSymbol, Expansion, FoldingRange, Hover, InlayHint,
    InlayHintKind, Location, MarkupKind, Position, SelectionRange, SemanticToken,
    SemanticTokenKind, SignatureHelp, SymbolInformation, SymbolKind, TOKEN_MODIFIERS,
};
use gen_lsp_types as lsp;
use serde_json::{Value, json};

// Domain enums have no protocol discriminants. Only this JSON boundary maps LSP values.
/// Standard full/unchanged document report with a content-derived opaque result id.
pub(super) fn document_report(
    diagnostics: &[Diagnostic],
    previous: Option<&str>,
) -> lsp::DocumentDiagnosticReport {
    let items: Vec<_> = diagnostics.iter().map(diagnostic_model).collect();
    let result_id = {
        use std::hash::{Hash, Hasher};
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        standard_value(&items).to_string().hash(&mut hasher);
        format!("{:016x}", hasher.finish())
    };
    if previous == Some(result_id.as_str()) {
        lsp::RelatedUnchangedDocumentDiagnosticReport {
            related_documents: None,
            unchanged_document_diagnostic_report: lsp::UnchangedDocumentDiagnosticReport {
                result_id,
            },
        }
        .into()
    } else {
        lsp::RelatedFullDocumentDiagnosticReport {
            related_documents: None,
            full_document_diagnostic_report: lsp::FullDocumentDiagnosticReport {
                result_id: Some(result_id),
                items,
            },
        }
        .into()
    }
}

pub(super) fn workspace_document_report(
    uri: &str,
    version: Option<i32>,
    diagnostics: &[Diagnostic],
    previous: Option<&str>,
) -> lsp::WorkspaceDocumentDiagnosticReport {
    match document_report(diagnostics, previous) {
        lsp::DocumentDiagnosticReport::RelatedFullDocumentDiagnosticReport(report) => {
            lsp::WorkspaceFullDocumentDiagnosticReport {
                uri: standard_uri(uri),
                version,
                full_document_diagnostic_report: report.full_document_diagnostic_report,
            }
            .into()
        }
        lsp::DocumentDiagnosticReport::RelatedUnchangedDocumentDiagnosticReport(report) => {
            lsp::WorkspaceUnchangedDocumentDiagnosticReport {
                uri: standard_uri(uri),
                version,
                unchanged_document_diagnostic_report: report.unchanged_document_diagnostic_report,
            }
            .into()
        }
    }
}

/// Standard diagnostic publication, including the version only when it is known.
pub(super) fn diagnostics_notification(
    uri: &str,
    version: Option<i32>,
    diagnostics: &[Diagnostic],
) -> Value {
    json!({"jsonrpc":"2.0", "method":lsp::LspNotificationMethod::TextDocumentPublishDiagnostics.as_str(),
        "params":lsp::PublishDiagnosticsParams { uri: standard_uri(uri), version,
            diagnostics: diagnostics.iter().map(diagnostic_model).collect() }})
}

#[cfg(test)]
fn diagnostic_wire(diagnostic: &Diagnostic) -> Value {
    standard_value(diagnostic_model(diagnostic))
}

fn diagnostic_model(diagnostic: &Diagnostic) -> lsp::Diagnostic {
    let severity = match diagnostic.severity {
        DiagnosticSeverity::Error => lsp::DiagnosticSeverity::Error,
        DiagnosticSeverity::Warning => lsp::DiagnosticSeverity::Warning,
        DiagnosticSeverity::Hint => lsp::DiagnosticSeverity::Hint,
    };
    let (code, source) = diagnostic.code.wire();
    lsp::Diagnostic {
        range: diagnostic.range,
        severity: Some(severity),
        code: Some(lsp::Code::String(code.into())),
        source: Some(source.into()),
        message: diagnostic.message.clone().into(),
        related_information: (!diagnostic.related.is_empty()).then(|| {
            diagnostic
                .related
                .iter()
                .map(|related| lsp::DiagnosticRelatedInformation {
                    location: lsp::Location {
                        uri: standard_uri(&related.uri),
                        range: related.range,
                    },
                    message: related.message.clone(),
                })
                .collect()
        }),
        tags: (diagnostic.code == DiagnosticCode::Unreachable)
            .then(|| vec![lsp::DiagnosticTag::Unnecessary]),
        code_description: match diagnostic.code {
            DiagnosticCode::Lint(rule) => Some(lsp::CodeDescription {
                href: standard_uri(&jailint::rule_url(rule)),
            }),
            _ => None,
        },
        data: None,
    }
}

/// Typed standard payloads serialize through the existing bounded response envelope.
pub(super) fn standard_value(value: impl serde::Serialize) -> Value {
    serde_json::to_value(value).expect("standard wire schemas serialize to JSON")
}

pub(super) fn standard_uri(text: &str) -> lsp::Uri {
    text.parse()
        .expect("domain locations contain valid document URIs")
}

/// Consume domain symbols so standard wire types reuse their owned strings.
#[expect(
    deprecated,
    reason = "the standard deprecated field is intentionally absent"
)]
pub(super) fn document_symbol_wire(symbol: DocumentSymbol) -> gen_lsp_types::DocumentSymbol {
    gen_lsp_types::DocumentSymbol {
        name: symbol.name,
        detail: Some(symbol.detail),
        kind: symbol_kind_wire(symbol.kind),
        range: symbol.range,
        selection_range: symbol.selection_range,
        children: (!symbol.children.is_empty()).then(|| {
            symbol
                .children
                .into_iter()
                .map(document_symbol_wire)
                .collect()
        }),
        tags: None,
        deprecated: None,
    }
}

pub(super) fn hierarchy_item_wire(item: &CallHierarchyItem) -> Value {
    standard_value(hierarchy_item_model(item))
}

fn hierarchy_item_model(item: &CallHierarchyItem) -> lsp::CallHierarchyItem {
    lsp::CallHierarchyItem {
        name: item.name.clone(),
        kind: symbol_kind_wire(item.kind),
        detail: Some(item.detail.clone()),
        uri: standard_uri(&item.uri),
        range: item.range,
        selection_range: item.selection_range,
        tags: None,
        data: None,
    }
}

/// `side` is `from` for an incoming call and `to` for an outgoing one.
pub(super) fn hierarchy_call_wire(call: &CallHierarchyCall, side: &str) -> Value {
    let item = hierarchy_item_model(&call.item);
    if side == "from" {
        standard_value(lsp::CallHierarchyIncomingCall {
            from: item,
            from_ranges: call.from_ranges.clone(),
        })
    } else {
        standard_value(lsp::CallHierarchyOutgoingCall {
            to: item,
            from_ranges: call.from_ranges.clone(),
        })
    }
}

pub(super) fn selection_wire(selection: &SelectionRange) -> Value {
    standard_value(selection_model(selection))
}

fn selection_model(selection: &SelectionRange) -> lsp::SelectionRange {
    lsp::SelectionRange {
        range: selection.range,
        parent: selection
            .parent
            .as_ref()
            .map(|parent| Box::new(selection_model(parent))),
    }
}

fn location_wire(location: &Location) -> Value {
    standard_value(location_model(location))
}

fn location_model(location: &Location) -> lsp::Location {
    lsp::Location {
        uri: standard_uri(&location.uri),
        range: location.range,
    }
}

pub(super) fn locations_wire(locations: &[Location]) -> Value {
    standard_value(locations.iter().map(location_model).collect::<Vec<_>>())
}

fn symbol_kind_wire(kind: SymbolKind) -> gen_lsp_types::SymbolKind {
    match kind {
        SymbolKind::File => gen_lsp_types::SymbolKind::File,
        SymbolKind::Namespace => gen_lsp_types::SymbolKind::Module,
        SymbolKind::TypeAlias => gen_lsp_types::SymbolKind::Class,
        SymbolKind::Property => gen_lsp_types::SymbolKind::Property,
        SymbolKind::Enum => gen_lsp_types::SymbolKind::Enum,
        SymbolKind::Function => gen_lsp_types::SymbolKind::Function,
        SymbolKind::Variable => gen_lsp_types::SymbolKind::Variable,
        SymbolKind::Constant => gen_lsp_types::SymbolKind::Constant,
        SymbolKind::EnumMember => gen_lsp_types::SymbolKind::EnumMember,
        SymbolKind::Struct => gen_lsp_types::SymbolKind::Struct,
        SymbolKind::Boolean => gen_lsp_types::SymbolKind::Boolean,
        SymbolKind::Event => gen_lsp_types::SymbolKind::Event,
    }
}

#[expect(
    deprecated,
    reason = "the standard deprecated field is intentionally absent"
)]
pub(super) fn symbol_information_wire(symbol: &SymbolInformation) -> Value {
    standard_value(lsp::SymbolInformation {
        base_symbol_information: lsp::BaseSymbolInformation {
            name: symbol.name.clone(),
            kind: symbol_kind_wire(symbol.kind),
            container_name: symbol.container.clone(),
            tags: None,
        },
        location: location_model(&symbol.location),
        deprecated: None,
    })
}

pub(super) fn folding_wire(range: &FoldingRange) -> Value {
    standard_value(lsp::FoldingRange {
        start_line: range.start_line,
        end_line: range.end_line,
        kind: range.imports.then_some(lsp::FoldingRangeKind::Imports),
        ..Default::default()
    })
}

pub(super) fn inlay_wire(hint: &InlayHint) -> Value {
    standard_value(lsp::InlayHint {
        position: hint.position,
        label: hint.label.clone().into(),
        padding_left: Some(hint.padding_left),
        padding_right: Some(hint.padding_right),
        kind: hint.kind.map(|kind| match kind {
            InlayHintKind::Type => lsp::InlayHintKind::Type,
            InlayHintKind::Parameter => lsp::InlayHintKind::Parameter,
        }),
        tooltip: hint.tooltip.clone().map(Into::into),
        text_edits: None,
        data: None,
    })
}

/// Documentation as the client asked for it: a `MarkupContent`, or a plain string.
fn documentation_wire(kind: MarkupKind, text: &str) -> lsp::Documentation {
    match kind {
        MarkupKind::Markdown => lsp::Documentation::MarkupContent(lsp::MarkupContent {
            kind: lsp::MarkupKind::Markdown,
            value: text.into(),
        }),
        MarkupKind::PlainText => lsp::Documentation::String(text.into()),
    }
}

pub(super) fn signature_wire(help: &SignatureHelp, kind: MarkupKind) -> Value {
    standard_value(lsp::SignatureHelp {
        signatures: help
            .signatures
            .iter()
            .map(|signature| lsp::SignatureInformation {
                label: signature.label.clone(),
                documentation: signature
                    .documentation
                    .as_deref()
                    .map(|doc| documentation_wire(kind, doc)),
                parameters: Some(
                    signature
                        .parameters
                        .iter()
                        .enumerate()
                        .map(|(i, label)| lsp::ParameterInformation {
                            label: label.clone().into(),
                            documentation: signature
                                .parameter_docs
                                .get(i)
                                .and_then(Option::as_deref)
                                .map(|doc| documentation_wire(kind, doc)),
                        })
                        .collect(),
                ),
                active_parameter: None,
            })
            .collect(),
        active_signature: Some(
            u32::try_from(help.active_signature).expect("signature index fits protocol uinteger"),
        ),
        active_parameter: Some(
            u32::try_from(help.active_parameter)
                .expect("parameter index fits protocol uinteger")
                .into(),
        ),
    })
}

fn command_wire(command: &Command) -> lsp::Command {
    lsp::Command {
        title: command.title.clone(),
        command: command.command.clone(),
        arguments: Some(command.arguments.clone()),
        tooltip: None,
    }
}

pub(super) fn code_action_wire(action: &CodeAction) -> Value {
    standard_value(lsp::CodeAction {
        title: action.title.clone(),
        kind: action.kind.map(lsp::CodeActionKind::new),
        edit: action.edit.as_ref().map(|(uri, edits)| lsp::WorkspaceEdit {
            changes: Some(
                [(
                    standard_uri(uri),
                    edits.iter().map(text_edit_model).collect(),
                )]
                .into(),
            ),
            ..Default::default()
        }),
        command: action.command.as_ref().map(command_wire),
        diagnostics: (!action.diagnostics.is_empty())
            .then(|| action.diagnostics.iter().map(diagnostic_model).collect()),
        is_preferred: action.is_preferred.then_some(true),
        data: action.rule.map(|rule| json!({"rule":rule})),
        disabled: None,
        tags: None,
    })
}

fn text_edit_model(edit: &crate::TextEdit) -> lsp::TextEdit {
    lsp::TextEdit {
        range: edit.range,
        new_text: edit.new_text.clone(),
    }
}

pub(super) fn code_lens_wire(lens: &CodeLens) -> Value {
    standard_value(lsp::CodeLens {
        range: lens.range,
        command: Some(command_wire(&lens.command)),
        data: None,
    })
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
pub(super) fn signature_kind(caps: &gen_lsp_types::ClientCapabilities) -> MarkupKind {
    preferred_markup(
        caps.text_document
            .as_ref()
            .and_then(|d| d.signature_help.as_ref())
            .and_then(|s| s.signature_information.as_ref())
            .and_then(|s| s.documentation_format.as_deref()),
    )
}

fn preferred_markup(formats: Option<&[gen_lsp_types::MarkupKind]>) -> MarkupKind {
    if formats.is_some_and(|formats| formats.contains(&gen_lsp_types::MarkupKind::Markdown)) {
        MarkupKind::Markdown
    } else {
        MarkupKind::PlainText
    }
}

/// What the server offers; diagnostics are pulled only from a client that asked to.
fn server_capabilities(pull_diagnostics: bool) -> lsp::ServerCapabilities {
    lsp::ServerCapabilities {
        position_encoding: Some(lsp::PositionEncodingKind::UTF16),
        text_document_sync: Some(
            lsp::TextDocumentSyncOptions {
                open_close: Some(true),
                change: Some(lsp::TextDocumentSyncKind::Incremental),
                save: Some(true.into()),
                ..Default::default()
            }
            .into(),
        ),
        hover_provider: Some(true.into()),
        completion_provider: Some(lsp::CompletionOptions {
            resolve_provider: Some(true),
            trigger_characters: Some([".", "#", "\"", "/"].map(str::to_owned).to_vec()),
            ..Default::default()
        }),
        definition_provider: Some(true.into()),
        type_definition_provider: Some(true.into()),
        declaration_provider: Some(true.into()),
        type_hierarchy_provider: Some(true.into()),
        linked_editing_range_provider: Some(true.into()),
        implementation_provider: Some(true.into()),
        document_formatting_provider: Some(true.into()),
        document_range_formatting_provider: Some(true.into()),
        references_provider: Some(true.into()),
        rename_provider: Some(
            lsp::RenameOptions {
                prepare_provider: Some(true),
                work_done_progress_options: lsp::WorkDoneProgressOptions::default(),
            }
            .into(),
        ),
        document_highlight_provider: Some(true.into()),
        document_symbol_provider: Some(true.into()),
        workspace_symbol_provider: Some(true.into()),
        folding_range_provider: Some(true.into()),
        document_link_provider: Some(lsp::DocumentLinkOptions {
            resolve_provider: Some(false),
            work_done_progress_options: lsp::WorkDoneProgressOptions::default(),
        }),
        inlay_hint_provider: Some(true.into()),
        signature_help_provider: Some(lsp::SignatureHelpOptions {
            trigger_characters: Some(vec!["(".into(), ",".into()]),
            retrigger_characters: Some(vec![",".into()]),
            ..Default::default()
        }),
        code_action_provider: Some(
            lsp::CodeActionOptions {
                code_action_kinds: Some(vec![
                    lsp::CodeActionKind::QuickFix,
                    lsp::CodeActionKind::RefactorExtract,
                    lsp::CodeActionKind::RefactorInline,
                    lsp::CodeActionKind::RefactorRewrite,
                    lsp::CodeActionKind::new(crate::lints::FIX_ALL_KIND),
                ]),
                ..Default::default()
            }
            .into(),
        ),
        code_lens_provider: Some(lsp::CodeLensOptions {
            resolve_provider: Some(false),
            ..Default::default()
        }),
        call_hierarchy_provider: Some(true.into()),
        selection_range_provider: Some(true.into()),
        execute_command_provider: Some(lsp::ExecuteCommandOptions {
            commands: ServerCommand::NAMES
                .iter()
                .map(|(name, _)| (*name).to_owned())
                .collect(),
            ..Default::default()
        }),
        semantic_tokens_provider: Some(
            lsp::SemanticTokensOptions {
                legend: lsp::SemanticTokensLegend {
                    token_types: SemanticTokenKind::ALL
                        .iter()
                        .map(|kind| kind.name().to_owned())
                        .collect(),
                    token_modifiers: TOKEN_MODIFIERS
                        .iter()
                        .map(|name| (*name).to_owned())
                        .collect(),
                },
                range: Some(true.into()),
                full: Some(
                    lsp::SemanticTokensFullDelta {
                        delta: Some(true),
                    }
                    .into(),
                ),
                ..Default::default()
            }
            .into(),
        ),
        diagnostic_provider: pull_diagnostics.then(|| {
            lsp::DiagnosticOptions {
                identifier: Some("jai".into()),
                inter_file_dependencies: true,
                workspace_diagnostics: true,
                ..Default::default()
            }
            .into()
        }),
        experimental: Some(json!({"jai": {
            "analysis": "compiler-source-syntax", "compileTimeExecution": false,
            "typeInference": false, "filesystemReads": false, "moduleSearch": false,
            "expansions": true,
        }})),
        ..Default::default()
    }
}

/// Standard initialize result from the generated protocol schema.
pub(super) fn initialize_wire(pull_diagnostics: bool) -> Value {
    standard_value(lsp::InitializeResult {
        capabilities: server_capabilities(pull_diagnostics),
        server_info: Some(lsp::ServerInfo {
            name: "jai-language-server".into(),
            version: Some(env!("CARGO_PKG_VERSION").into()),
        }),
    })
}

pub(super) fn edit_wire(edit: &crate::TextEdit) -> Value {
    standard_value(text_edit_model(edit))
}

pub(super) fn workspace_edit_wire(documents: crate::features::RenameEdits) -> Value {
    standard_value(lsp::WorkspaceEdit {
        changes: Some(
            documents
                .into_iter()
                .map(|(uri, edits)| {
                    (
                        standard_uri(&uri),
                        edits
                            .into_iter()
                            .map(|edit| lsp::TextEdit {
                                range: edit.range,
                                new_text: edit.new_text,
                            })
                            .collect(),
                    )
                })
                .collect(),
        ),
        ..Default::default()
    })
}

pub(super) fn hover_kind(caps: &gen_lsp_types::ClientCapabilities) -> MarkupKind {
    preferred_markup(
        caps.text_document
            .as_ref()
            .and_then(|d| d.hover.as_ref())
            .and_then(|h| h.content_format.as_deref()),
    )
}

pub(super) fn hover_wire(hover: &Hover) -> Value {
    let kind = match hover.contents.kind {
        MarkupKind::PlainText => lsp::MarkupKind::PlainText,
        MarkupKind::Markdown => lsp::MarkupKind::Markdown,
    };
    standard_value(lsp::Hover {
        contents: lsp::MarkupContent {
            kind,
            value: hover.contents.value.clone(),
        }
        .into(),
        range: Some(hover.range),
    })
}

pub(super) fn semantic_tokens_wire(tokens: &[SemanticToken]) -> Vec<lsp::SemanticToken> {
    let mut data = Vec::with_capacity(tokens.len());
    let mut previous = Position::new(0, 0);
    for token in tokens {
        let delta_line = token.position.line - previous.line;
        data.push(lsp::SemanticToken {
            delta_line,
            delta_start: if delta_line == 0 {
                token.position.character - previous.character
            } else {
                token.position.character
            },
            length: token.length,
            token_type: token.kind as u32,
            token_modifiers_bitset: u32::from(token.declaration)
                | (u32::from(token.readonly) << 1)
                | (u32::from(token.expand) << 2),
        });
        previous = token.position;
    }
    data
}

fn token_integers(tokens: &[lsp::SemanticToken]) -> impl DoubleEndedIterator<Item = u32> + '_ {
    tokens.iter().flat_map(|token| {
        [
            token.delta_line,
            token.delta_start,
            token.length,
            token.token_type,
            token.token_modifiers_bitset,
        ]
    })
}

/// One minimal edit, with offsets measured in integers rather than whole tokens.
/// For example, changing only a token length inserts one integer at offset two.
fn token_edit(
    old: &[lsp::SemanticToken],
    new: &[lsp::SemanticToken],
) -> (usize, usize, std::ops::Range<usize>) {
    let prefix = token_integers(old)
        .zip(token_integers(new))
        .take_while(|(a, b)| a == b)
        .count();
    let old_len = old.len() * 5;
    let new_len = new.len() * 5;
    let room = old_len.min(new_len) - prefix;
    let suffix = token_integers(old)
        .rev()
        .zip(token_integers(new).rev())
        .take(room)
        .take_while(|(a, b)| a == b)
        .count();
    (prefix, old_len - prefix - suffix, prefix..new_len - suffix)
}

/// Stream the integer span without allocating a second flat array.
/// Standard-schema parity tests cover the complete generated `SemanticTokensDelta` shape.
struct TokenSpan<'a> {
    tokens: &'a [lsp::SemanticToken],
    range: std::ops::Range<usize>,
}

impl serde::Serialize for TokenSpan<'_> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_seq(
            token_integers(self.tokens)
                .skip(self.range.start)
                .take(self.range.len()),
        )
    }
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct TokenDelta<'a> {
    result_id: String,
    edits: [TokenDeltaEdit<'a>; 1],
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct TokenDeltaEdit<'a> {
    start: usize,
    delete_count: usize,
    data: TokenSpan<'a>,
}

/// Serialize one integer-granular edit; a length-only change produces `data: [length]`.
pub(super) fn token_delta_wire(
    id: u64,
    old: &[lsp::SemanticToken],
    new: &[lsp::SemanticToken],
) -> Result<Value, serde_json::Error> {
    let (start, delete_count, range) = token_edit(old, new);
    serde_json::to_value(&TokenDelta {
        result_id: id.to_string(),
        edits: [TokenDeltaEdit {
            start,
            delete_count,
            data: TokenSpan {
                tokens: new,
                range,
            },
        }],
    })
}

#[cfg(test)]
mod standard_capability_tests {
    use super::*;

    #[test]
    fn semantic_delta_preserves_single_integer_edits() {
        let before = vec![
            lsp::SemanticToken {
                delta_start: 2,
                length: 3,
                token_type: 1,
                ..Default::default()
            },
            lsp::SemanticToken {
                delta_start: 8,
                length: 4,
                token_type: 2,
                ..Default::default()
            },
        ];
        let mut after = before.clone();
        after[0].length = 7;
        let (start, delete, data) = token_edit(&before, &after);
        assert_eq!((start, delete, data), (2, 1, 2..3));
        let delta: Value = token_delta_wire(3, &before, &after).unwrap();
        assert_eq!(
            delta,
            json!({"resultId":"3", "edits":[{"start":2,"deleteCount":1,"data":[7]}]})
        );
        let typed: lsp::SemanticTokensDelta = serde_json::from_value(delta.clone()).unwrap();
        assert_eq!(standard_value(typed), delta);
        let full = standard_value(lsp::SemanticTokens {
            result_id: Some("4".into()),
            data: after,
        });
        assert_eq!(full, json!({"resultId":"4", "data":[0,2,7,1,0,0,8,4,2,0]}));
    }

    #[test]
    fn standard_capabilities_preserve_the_advertised_wire_shape() {
        for pull in [false, true] {
            assert_eq!(
                initialize_wire(pull)["capabilities"],
                previous_capabilities(pull)
            );
        }
    }

    #[test]
    fn standard_results_preserve_existing_diagnostic_action_and_documentation_shapes() {
        let range = crate::Range::new(Position::new(1, 2), Position::new(1, 4));
        let diagnostic = Diagnostic {
            range,
            severity: DiagnosticSeverity::Hint,
            code: DiagnosticCode::Unreachable,
            message: "unused path".into(),
            related: vec![crate::RelatedInformation {
                uri: "file:///other.jai".into(),
                range,
                message: "related".into(),
            }],
        };
        let expected = json!({"range":range,"severity":4,"code":"unreachable","source":"jaic",
            "message":"unused path","tags":[1],"relatedInformation":[{
                "location":{"uri":"file:///other.jai","range":range},"message":"related"}]});
        assert_eq!(diagnostic_wire(&diagnostic), expected);
        let action = CodeAction {
            title: "Fix".into(),
            kind: Some("quickfix"),
            edit: Some((
                "file:///main.jai".into(),
                vec![crate::TextEdit {
                    range,
                    new_text: "fixed".into(),
                }],
            )),
            command: Some(Command {
                title: "Go".into(),
                command: "jai.go".into(),
                arguments: vec![],
            }),
            diagnostics: vec![diagnostic],
            is_preferred: true,
            rule: Some("rule"),
        };
        assert_eq!(
            code_action_wire(&action),
            json!({"title":"Fix","kind":"quickfix",
            "edit":{"changes":{"file:///main.jai":[{"range":range,"newText":"fixed"}]}},
            "command":{"title":"Go","command":"jai.go","arguments":[]},
            "diagnostics":[expected],"isPreferred":true,"data":{"rule":"rule"}})
        );
        let help = SignatureHelp {
            signatures: vec![crate::SignatureInformation {
                label: "f :: (value: int)".into(),
                parameters: vec!["value: int".into()],
                documentation: Some("proc docs".into()),
                parameter_docs: vec![Some("arg docs".into())],
            }],
            active_signature: 0,
            active_parameter: 0,
        };
        assert_eq!(
            signature_wire(&help, MarkupKind::PlainText),
            json!({"signatures":[{
            "label":"f :: (value: int)","documentation":"proc docs",
            "parameters":[{"label":"value: int","documentation":"arg docs"}]}],
            "activeSignature":0,"activeParameter":0})
        );
        assert_eq!(
            inlay_wire(&InlayHint {
                position: range.start,
                label: ": int".into(),
                kind: Some(InlayHintKind::Type),
                padding_left: false,
                padding_right: true,
                tooltip: Some("type".into())
            }),
            json!({"position":range.start,"label":": int","kind":1,"paddingLeft":false,"paddingRight":true,"tooltip":"type"})
        );
        assert_eq!(
            folding_wire(&FoldingRange {
                start_line: 1,
                end_line: 3,
                imports: true
            }),
            json!({"startLine":1,"endLine":3,"kind":"imports"})
        );
    }

    fn previous_capabilities(pull_diagnostics: bool) -> Value {
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
            "declarationProvider": true,
            "linkedEditingRangeProvider": true,
            "implementationProvider": true,
            "documentFormattingProvider": true,
            "documentRangeFormattingProvider": true,
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
            "typeHierarchyProvider": true,
            "selectionRangeProvider": true,
            "executeCommandProvider": { "commands": commands },
            "semanticTokensProvider": {
                "legend": {
                    "tokenTypes": token_types,
                    "tokenModifiers": TOKEN_MODIFIERS,
                },
                "range": true,
                "full": { "delta": true },
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
}
