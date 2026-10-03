//! Identity for one retained escaping diagnostic, independent of display text.
use std::{fmt, sync::Arc};

/// An opaque receipt for a diagnostic-producing operation.
///
/// Clones refer to the same operation. A fresh receipt always differs, including
/// when two errors have identical text and source spans. This is metadata only;
/// semantic declaration, type, procedure, and VM identities are unaffected.
#[derive(Clone, Default)]
pub struct DiagnosticMarker(Arc<()>);
impl DiagnosticMarker {
    pub fn new() -> Self {
        Self::default()
    }
}
impl PartialEq for DiagnosticMarker {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}
impl Eq for DiagnosticMarker {}
impl fmt::Debug for DiagnosticMarker {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("DiagnosticMarker")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Diagnostic, SourceMap, Span};

    #[test]
    fn clones_and_source_rebasing_preserve_only_the_original_receipt() {
        let marker = DiagnosticMarker::new();
        let span = Span::new(3, 7);
        let diagnostic = Diagnostic::new(span, "unavailable").with_marker(marker.clone());
        let mut sources = SourceMap::default();
        let source = sources.insert("lookup.jai".into(), "missing".into());
        let retained = diagnostic.clone().with_fallback_source(source);
        assert_eq!(retained.marker(), Some(&marker));
        assert_ne!(marker, DiagnosticMarker::new());
        assert!(Diagnostic::new(span, "type mismatch").marker().is_none());
        assert!(
            Diagnostic::at_source(crate::SourceSpan { source, span }, "type mismatch")
                .marker()
                .is_none()
        );
    }
}
