//! Standard JSON parsing belongs at this portable boundary, not in source analysis.
mod handling;
mod helpers;
mod methods;
mod notifications;
mod params;
mod progress;
mod request_id;
mod requests;
mod wire;

pub use self::helpers::{message_too_large, oversized_replies};
use crate::{MarkupKind, Session};
use std::collections::VecDeque;

pub use self::request_id::RequestId;

#[derive(Clone, Debug)]
pub enum ProtocolError {
    MessageLimit,
    OutputLimit,
    Serialization(String),
}

impl std::fmt::Display for ProtocolError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MessageLimit => f.write_str("LSP message byte limit exceeded"),
            Self::OutputLimit => f.write_str("LSP response byte limit exceeded"),
            Self::Serialization(e) => f.write_str(e),
        }
    }
}


impl std::error::Error for ProtocolError {
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Lifecycle {
    New,
    Running,
    Shutdown,
    Exited(u8),
}

pub struct JsonSession {
    session: Session,
    lifecycle: Lifecycle,
    cancelled: VecDeque<RequestId>,
    completed: VecDeque<RequestId>,
    /// Markdown when the client listed it in `textDocument.hover.contentFormat`.
    hover_kind: MarkupKind,
    /// Markdown when the client listed it in the signature help `documentationFormat`.
    signature_kind: MarkupKind,
    /// The client pulls diagnostics (`textDocument/diagnostic`): none are pushed to it.
    pull_diagnostics: bool,
    /// It can also be told to pull again (`workspace/diagnostic/refresh`).
    refresh_diagnostics: bool,
    refreshes: u64,
    /// The client can be told to ask for code lenses again (`workspace/codeLens/refresh`).
    refresh_code_lenses: bool,
    /// And for inlay hints and semantic tokens (`workspace/inlayHint/refresh`,
    /// `workspace/semanticTokens/refresh`): they are answered without a compile while edits
    /// are typed, and asked for again when one has been made.
    refresh_inlay_hints: bool,
    refresh_semantic_tokens: bool,
    /// The last semantic tokens sent for each document, by result id, so a `delta` request
    /// answers with what changed.
    token_results: std::collections::BTreeMap<String, (u64, Vec<u32>)>,
    token_result_ids: u64,
    /// Background work changed polymorph summaries since lenses were last refreshed.
    lenses_changed: bool,
    /// Diagnostics are pushed by `flush_publications`, not with the answer to each edit.
    defer_publications: bool,
    /// An edit since the last flush has diagnostics to publish.
    publications_pending: bool,
    index_progress: progress::IndexProgress,
    workspace_published: std::collections::BTreeSet<String>,
}


#[cfg(test)]
mod tests {
    use super::methods::{Notification, Request};
    use crate::SemanticTokenKind;
    use std::collections::BTreeSet;

    #[test]
    fn token_types_are_listed_in_wire_order() {
        for (index, kind) in SemanticTokenKind::ALL.iter().enumerate() {
            assert_eq!(*kind as usize, index, "{kind:?} is out of legend order");
        }
        let names: BTreeSet<&str> = SemanticTokenKind::ALL.iter().map(|k| k.name()).collect();
        assert_eq!(names.len(), SemanticTokenKind::ALL.len());
    }

    #[test]
    fn method_tables_have_no_duplicates() {
        let requests: BTreeSet<&str> = Request::METHODS.iter().map(|(name, _)| *name).collect();
        assert_eq!(requests.len(), Request::METHODS.len());
        let notifications: BTreeSet<&str> = Notification::METHODS
            .iter()
            .map(|(name, _)| *name)
            .collect();
        assert_eq!(notifications.len(), Notification::METHODS.len());
        assert!(requests.is_disjoint(&notifications));
    }

    #[test]
    fn duplicate_workspace_folders_are_kept_once() {
        let mut session = super::JsonSession::new(crate::Limits::default());
        let init = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": { "workspaceFolders": [
                { "uri": "file:///w", "name": "w" },
                { "uri": "file://localhost/w", "name": "w2" },
            ] },
        });
        session.handle_json(&init.to_string()).unwrap();
        assert_eq!(session.session().workspace_folders.len(), 1);
    }
}
