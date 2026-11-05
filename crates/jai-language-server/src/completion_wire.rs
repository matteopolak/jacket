//! The wire form of completion lists.
use crate::{CompletionKind, CompletionList};
use serde_json::json;

/// The LSP `CompletionItemKind` of a completion.
pub(crate) fn completion_kind_wire(kind: CompletionKind) -> gen_lsp_types::CompletionItemKind {
    match kind {
        CompletionKind::Function => gen_lsp_types::CompletionItemKind::Function,
        CompletionKind::Field => gen_lsp_types::CompletionItemKind::Field,
        CompletionKind::Variable => gen_lsp_types::CompletionItemKind::Variable,
        CompletionKind::TypeAlias => gen_lsp_types::CompletionItemKind::Class,
        CompletionKind::Module => gen_lsp_types::CompletionItemKind::Module,
        CompletionKind::Enum => gen_lsp_types::CompletionItemKind::Enum,
        CompletionKind::Keyword => gen_lsp_types::CompletionItemKind::Keyword,
        CompletionKind::Snippet => gen_lsp_types::CompletionItemKind::Snippet,
        CompletionKind::File => gen_lsp_types::CompletionItemKind::File,
        CompletionKind::Folder => gen_lsp_types::CompletionItemKind::Folder,
        CompletionKind::EnumMember => gen_lsp_types::CompletionItemKind::EnumMember,
        CompletionKind::Constant => gen_lsp_types::CompletionItemKind::Constant,
        CompletionKind::Struct => gen_lsp_types::CompletionItemKind::Struct,
        // An `#asm` mnemonic is an operation, not a language keyword.
        CompletionKind::Instruction => gen_lsp_types::CompletionItemKind::Operator,
    }
}

pub(crate) fn completion_wire(completion: CompletionList) -> gen_lsp_types::CompletionList {
    use gen_lsp_types as lsp;
    let items = completion
        .items
        .into_iter()
        .map(|item| {
            let insert_text_format = item.insert_text.as_ref().map(|_| {
                if item.kind == CompletionKind::Snippet || item.snippet {
                    lsp::InsertTextFormat::Snippet
                } else {
                    lsp::InsertTextFormat::PlainText
                }
            });
            let (text_edit, insert_text) = match item.replace {
                Some(range) => (
                    Some(
                        lsp::TextEdit {
                            range,
                            new_text: item.insert_text.unwrap_or_else(|| item.label.clone()),
                        }
                        .into(),
                    ),
                    None,
                ),
                None => (None, item.insert_text),
            };
            lsp::CompletionItem {
                label: item.label,
                kind: Some(completion_kind_wire(item.kind)),
                detail: Some(item.detail),
                documentation: item.documentation.map(|value| {
                    lsp::Documentation::MarkupContent(lsp::MarkupContent {
                        kind: lsp::MarkupKind::Markdown,
                        value,
                    })
                }),
                data: item.data.map(|data| {
                    json!({
                        "document": data.document, "path": data.path, "offset": data.offset,
                    })
                }),
                label_details: item.label_description.map(|description| {
                    lsp::CompletionItemLabelDetails {
                        detail: None,
                        description: Some(description),
                    }
                }),
                filter_text: item.filter_text,
                sort_text: item.sort_text,
                preselect: item.preselect.then_some(true),
                additional_text_edits: (!item.additional_edits.is_empty()).then(|| {
                    item.additional_edits
                        .into_iter()
                        .map(|edit| lsp::TextEdit {
                            range: edit.range,
                            new_text: edit.new_text,
                        })
                        .collect()
                }),
                insert_text,
                insert_text_format,
                text_edit,
                command: item.trigger_signature_help.then(|| lsp::Command {
                    title: "Trigger Parameter Hints".into(),
                    command: "editor.action.triggerParameterHints".into(),
                    arguments: None,
                    tooltip: None,
                }),
                ..Default::default()
            }
        })
        .collect();
    lsp::CompletionList {
        is_incomplete: completion.is_incomplete,
        items,
        ..Default::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn instructions_are_not_keywords() {
        assert_ne!(
            completion_kind_wire(CompletionKind::Instruction),
            completion_kind_wire(CompletionKind::Keyword)
        );
    }

    #[test]
    fn standard_items_preserve_optional_fields_and_edit_precedence() {
        use crate::{CompletionData, CompletionItem, Position, Range, TextEdit};
        let range = Range::new(Position::new(1, 2), Position::new(1, 5));
        let item = CompletionItem {
            label: "call".into(),
            kind: CompletionKind::Function,
            detail: "call :: ()".into(),
            documentation: Some("**docs**".into()),
            insert_text: Some("call(${1:value})".into()),
            replace: Some(range),
            label_description: Some("Module".into()),
            sort_text: Some("0call".into()),
            filter_text: Some("receiver.call".into()),
            additional_edits: vec![TextEdit {
                range,
                new_text: "#import".into(),
            }],
            data: Some(CompletionData {
                document: "file:///main.jai".into(),
                path: "module.jai".into(),
                offset: 7,
            }),
            snippet: true,
            trigger_signature_help: true,
            preselect: true,
            ..Default::default()
        };
        let wire = serde_json::to_value(completion_wire(CompletionList {
            is_incomplete: true,
            items: vec![item],
        }))
        .unwrap();
        assert_eq!(
            wire,
            json!({"isIncomplete":true,"items":[{
                "label":"call","kind":3,"detail":"call :: ()",
                "documentation":{"kind":"markdown","value":"**docs**"},
                "labelDetails":{"description":"Module"},"sortText":"0call","filterText":"receiver.call",
                "additionalTextEdits":[{"range":range,"newText":"#import"}],
                "data":{"document":"file:///main.jai","path":"module.jai","offset":7},
                "textEdit":{"range":range,"newText":"call(${1:value})"},"insertTextFormat":2,
                "command":{"title":"Trigger Parameter Hints","command":"editor.action.triggerParameterHints"},
                "preselect":true
            }]})
        );
    }
}
