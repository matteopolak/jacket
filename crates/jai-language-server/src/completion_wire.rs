//! The wire form of completion lists.
use crate::{CompletionKind, CompletionList};
use serde_json::json;

/// The LSP `CompletionItemKind` of a completion.
pub(crate) fn completion_kind_wire(kind: CompletionKind) -> lsp_types::CompletionItemKind {
    match kind {
        CompletionKind::Function => lsp_types::CompletionItemKind::FUNCTION,
        CompletionKind::Field => lsp_types::CompletionItemKind::FIELD,
        CompletionKind::Variable => lsp_types::CompletionItemKind::VARIABLE,
        CompletionKind::TypeAlias => lsp_types::CompletionItemKind::CLASS,
        CompletionKind::Module => lsp_types::CompletionItemKind::MODULE,
        CompletionKind::Enum => lsp_types::CompletionItemKind::ENUM,
        CompletionKind::Keyword => lsp_types::CompletionItemKind::KEYWORD,
        CompletionKind::Snippet => lsp_types::CompletionItemKind::SNIPPET,
        CompletionKind::File => lsp_types::CompletionItemKind::FILE,
        CompletionKind::Folder => lsp_types::CompletionItemKind::FOLDER,
        CompletionKind::EnumMember => lsp_types::CompletionItemKind::ENUM_MEMBER,
        CompletionKind::Constant => lsp_types::CompletionItemKind::CONSTANT,
        CompletionKind::Struct => lsp_types::CompletionItemKind::STRUCT,
        // An `#asm` mnemonic is an operation, not a language keyword.
        CompletionKind::Instruction => lsp_types::CompletionItemKind::OPERATOR,
    }
}

pub(crate) fn completion_wire(completion: CompletionList) -> lsp_types::CompletionList {
    use lsp_types as lsp;
    let items = completion
        .items
        .into_iter()
        .map(|item| {
            let insert_text_format = item.insert_text.as_ref().map(|_| {
                if item.kind == CompletionKind::Snippet || item.snippet {
                    lsp::InsertTextFormat::SNIPPET
                } else {
                    lsp::InsertTextFormat::PLAIN_TEXT
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
                }),
                ..Default::default()
            }
        })
        .collect();
    lsp::CompletionList {
        is_incomplete: completion.is_incomplete,
        items,
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
