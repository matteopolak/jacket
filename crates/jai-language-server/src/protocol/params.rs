//! Typed request parameters as they arrive on the wire.
use super::RequestId;
use crate::{CallHierarchyItem, Position, Range, SymbolKind, TextChange};
use serde::Deserialize;
use serde_json::Value;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct DocumentItem {
    pub(super) uri: String,
    pub(super) language_id: String,
    pub(super) version: i32,
    pub(super) text: String,
}

#[derive(Deserialize)]
pub(super) struct DocumentIdentifier {
    pub(super) uri: String,
}

#[derive(Deserialize)]
pub(super) struct VersionedIdentifier {
    pub(super) uri: String,
    pub(super) version: i32,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct OpenParams {
    pub(super) text_document: DocumentItem,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct DocumentParams {
    pub(super) text_document: DocumentIdentifier,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct ChangeParams {
    pub(super) text_document: VersionedIdentifier,
    pub(super) content_changes: Vec<TextChange>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct PositionParams {
    pub(super) text_document: DocumentIdentifier,
    pub(super) position: Position,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct SelectionParams {
    pub(super) text_document: DocumentIdentifier,
    pub(super) positions: Vec<Position>,
}

/// The call hierarchy item a client sends back; only where it points matters.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct HierarchyItem {
    name: String,
    #[serde(default)]
    detail: Option<String>,
    uri: String,
    range: Range,
    selection_range: Range,
}

impl HierarchyItem {
    pub(super) fn into_item(self) -> CallHierarchyItem {
        CallHierarchyItem {
            name: self.name,
            detail: self.detail.unwrap_or_default(),
            kind: SymbolKind::Function,
            uri: self.uri,
            range: self.range,
            selection_range: self.selection_range,
        }
    }
}

#[derive(Deserialize)]
pub(super) struct HierarchyParams {
    pub(super) item: HierarchyItem,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct RangeParams {
    pub(super) text_document: DocumentIdentifier,
    pub(super) range: Range,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct CodeActionParams {
    pub(super) text_document: DocumentIdentifier,
    pub(super) range: Range,
    #[serde(default)]
    pub(super) context: Option<CodeActionContext>,
}

#[derive(Deserialize, Default)]
pub(super) struct CodeActionContext {
    #[serde(default)]
    pub(super) diagnostics: Vec<WireDiagnostic>,
    #[serde(default)]
    pub(super) only: Option<Vec<String>>,
}

/// The parts of a client's diagnostic that identify a lint.
#[derive(Deserialize)]
pub(super) struct WireDiagnostic {
    pub(super) range: Range,
    #[serde(default)]
    pub(super) code: Option<Value>,
    #[serde(default)]
    pub(super) source: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct ReferenceContext {
    pub(super) include_declaration: bool,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct ReferenceParams {
    pub(super) text_document: DocumentIdentifier,
    pub(super) position: Position,
    pub(super) context: Option<ReferenceContext>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct RenameParams {
    pub(super) text_document: DocumentIdentifier,
    pub(super) position: Position,
    pub(super) new_name: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct DiagnosticParams {
    pub(super) text_document: DocumentIdentifier,
    #[serde(default)]
    pub(super) previous_result_id: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct WorkspaceDiagnosticParams {
    #[serde(default)]
    pub(super) previous_result_ids: Vec<PreviousResult>,
}

#[derive(Deserialize)]
pub(super) struct PreviousResult {
    pub(super) uri: String,
    pub(super) value: String,
}

#[derive(Deserialize)]
pub(super) struct WorkspaceSymbolParams {
    pub(super) query: String,
}

#[derive(Deserialize)]
pub(super) struct CommandTarget {
    pub(super) uri: String,
    pub(super) position: Position,
}

#[derive(Deserialize)]
pub(super) struct CommandParams {
    pub(super) command: String,
    #[serde(default)]
    pub(super) arguments: Vec<CommandTarget>,
}

#[derive(Deserialize)]
pub(super) struct CancelParams {
    pub(super) id: RequestId,
}

#[derive(Deserialize)]
pub(super) struct Envelope {
    pub(super) jsonrpc: String,
    pub(super) method: String,
    #[serde(default)]
    pub(super) params: Value,
}
