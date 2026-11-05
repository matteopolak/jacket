//! Typed request parameters as they arrive on the wire.
use super::RequestId;
use crate::{CallHierarchyItem, Position, Range, SymbolKind};
use serde::Deserialize;

pub(super) use lsp_types::{
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
pub(super) fn call_item(item: lsp_types::CallHierarchyItem) -> CallHierarchyItem {
    CallHierarchyItem {
        name: item.name,
        detail: item.detail.unwrap_or_default(),
        kind: SymbolKind::Function,
        uri: item.uri.as_str().to_owned(),
        range: item.range,
        selection_range: item.selection_range,
    }
}

pub(super) fn type_item(item: lsp_types::TypeHierarchyItem) -> CallHierarchyItem {
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
