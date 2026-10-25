//! Standard JSON parsing belongs at this portable boundary, not in source analysis.
mod handling;
mod helpers;
mod methods;
mod notifications;
mod params;
mod request_id;
mod requests;
mod wire;

pub use self::helpers::message_too_large;
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
    /// Background work changed polymorph summaries since lenses were last refreshed.
    lenses_changed: bool,
    /// Diagnostics are pushed by `flush_publications`, not with the answer to each edit.
    defer_publications: bool,
    /// An edit since the last flush has diagnostics to publish.
    publications_pending: bool,
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
}
