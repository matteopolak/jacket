//! Typed request parameters as they arrive on the wire.
use super::RequestId;
use crate::{CallHierarchyItem, Position, Range, SymbolKind};
use serde::Deserialize;

pub(super) use gen_lsp_types::{
    CallHierarchyIncomingCallsParams, CallHierarchyOutgoingCallsParams, CodeActionParams,
    DidChangeTextDocumentParams as ChangeParams, DidOpenTextDocumentParams as OpenParams,
    DocumentDiagnosticParams as DiagnosticParams, ExecuteCommandParams as CommandParams,
    PreviousResultId as PreviousResult, ReferenceContext, RenameParams,
    SelectionRangeParams as SelectionParams, TextDocumentIdentifier as DocumentIdentifier,
    TextDocumentPositionParams as PositionParams, TypeHierarchySubtypesParams,
    TypeHierarchySupertypesParams, WorkspaceSymbolParams,
};

/// Shared document-only envelope for standard and Jai extension requests.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct DocumentParams {
    pub(super) text_document: DocumentIdentifier,
}

/// The compiler uses only the returned item's location and declaration extent.
pub(super) fn call_item(item: gen_lsp_types::CallHierarchyItem) -> CallHierarchyItem {
    CallHierarchyItem {
        name: item.name,
        detail: item.detail.unwrap_or_default(),
        kind: SymbolKind::Function,
        uri: item.uri.as_str().to_owned(),
        range: item.range,
        selection_range: item.selection_range,
    }
}

pub(super) fn type_item(item: gen_lsp_types::TypeHierarchyItem) -> CallHierarchyItem {
    CallHierarchyItem {
        name: item.name,
        detail: item.detail.unwrap_or_default(),
        kind: SymbolKind::Struct,
        uri: item.uri.as_str().to_owned(),
        range: item.range,
        selection_range: item.selection_range,
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct RangeParams {
    pub(super) text_document: DocumentIdentifier,
    pub(super) range: Range,
}

/// Legacy callers may omit context; its default includes declarations.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct ReferenceParams {
    pub(super) text_document: DocumentIdentifier,
    pub(super) position: Position,
    pub(super) context: Option<ReferenceContext>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct WorkspaceDiagnosticParams {
    #[serde(default)]
    pub(super) previous_result_ids: Vec<PreviousResult>,
}

#[derive(Deserialize)]
pub(super) struct CommandTarget {
    pub(super) uri: String,
    pub(super) position: Position,
}

#[derive(Deserialize)]
pub(super) struct CancelParams {
    pub(super) id: RequestId,
}

/// Admit edit shape before the generated untagged partial/whole union can discard an invalid range.
/// For example, a negative line with a replacement text must fail instead of replacing the file.
pub(super) fn change_params(
    params: serde_json::Value,
) -> Result<
    (
        gen_lsp_types::VersionedTextDocumentIdentifier,
        Vec<crate::TextChange>,
    ),
    super::methods::Failure,
> {
    use super::helpers::decode;
    let mut lengths = Vec::new();
    if let Some(changes) = params
        .get("contentChanges")
        .and_then(serde_json::Value::as_array)
    {
        for change in changes {
            if let Some(range) = change.get("range").filter(|range| !range.is_null()) {
                let _: crate::Range = decode(range.clone())?;
            }
            let length = change
                .get("rangeLength")
                .filter(|length| !length.is_null())
                .cloned()
                .map(decode::<u32>)
                .transpose()?;
            lengths.push(length);
        }
    }
    let params: ChangeParams = decode(params)?;
    let changes = params.content_changes.into_iter().zip(lengths).map(|(change, range_length)| {
        match change {
            gen_lsp_types::TextDocumentContentChangeEvent::TextDocumentContentChangePartial(change) => crate::TextChange {
                range: Some(change.range), range_length, text: change.text,
            },
            gen_lsp_types::TextDocumentContentChangeEvent::TextDocumentContentChangeWholeDocument(change) => crate::TextChange {
                range: None, range_length, text: change.text,
            },
        }
    }).collect();
    Ok((params.text_document, changes))
}
