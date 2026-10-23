//! The wire form of completion lists.
use crate::{CompletionKind, CompletionList};
use serde_json::{Value, json};

/// The LSP `CompletionItemKind` of a completion.
pub(crate) fn completion_kind_wire(kind: CompletionKind) -> u32 {
    match kind {
        CompletionKind::Function => 3,
        CompletionKind::Field => 5,
        CompletionKind::Variable => 6,
        CompletionKind::TypeAlias => 7,
        CompletionKind::Module => 9,
        CompletionKind::Enum => 13,
        CompletionKind::Keyword => 14,
        CompletionKind::Snippet => 15,
        CompletionKind::File => 17,
        CompletionKind::Folder => 19,
        CompletionKind::EnumMember => 20,
        CompletionKind::Constant => 21,
        CompletionKind::Struct => 22,
        // An `#asm` mnemonic is an operation, not a language keyword.
        CompletionKind::Instruction => 24,
    }
}

pub(crate) fn completion_wire(completion: &CompletionList) -> Value {
    let items: Vec<Value> = completion
        .items
        .iter()
        .map(|item| {
            let kind = completion_kind_wire(item.kind);
            let mut wire = json!({ "label": item.label, "kind": kind, "detail": item.detail });
            if let Some(doc) = &item.documentation {
                wire["documentation"] = json!({ "kind": "markdown", "value": doc });
            }
            if let Some(data) = &item.data {
                wire["data"] = json!({
                    "document": data.document,
                    "path": data.path,
                    "offset": data.offset,
                });
            }
            if let Some(description) = &item.label_description {
                wire["labelDetails"] = json!({ "description": description });
            }
            if let Some(filter) = &item.filter_text {
                wire["filterText"] = json!(filter);
            }
            if let Some(sort) = &item.sort_text {
                wire["sortText"] = json!(sort);
            }
            if item.preselect {
                wire["preselect"] = json!(true);
            }
            if !item.additional_edits.is_empty() {
                wire["additionalTextEdits"] = item
                    .additional_edits
                    .iter()
                    .map(|e| json!({ "range": e.range, "newText": e.new_text }))
                    .collect();
            }
            // `textEdit` replaces the whole word; `insertText` is for items without a range.
            let text = item.insert_text.as_deref();
            if let Some(range) = &item.replace {
                wire["textEdit"] =
                    json!({ "range": range, "newText": text.unwrap_or(&item.label) });
            } else if let Some(text) = text {
                wire["insertText"] = json!(text);
            }
            if text.is_some() {
                // 2: snippet syntax (`${1:name}`), 1: plain text.
                wire["insertTextFormat"] =
                    json!(if item.kind == CompletionKind::Snippet || item.snippet {
                        2
                    } else {
                        1
                    });
            }
            if item.trigger_signature_help {
                wire["command"] = json!({
                    "title": "Trigger Parameter Hints",
                    "command": "editor.action.triggerParameterHints",
                });
            }
            wire
        })
        .collect();
    json!({ "isIncomplete": completion.is_incomplete, "items": items })
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
}
