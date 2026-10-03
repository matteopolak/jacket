//! An authored name reservation is a source dependency, never a semantic value.
use jai_modules::{LookupError, ModuleGraph, PlaceholderId};
use jai_source::{Diagnostic, SourceSpan, Span};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct PlaceholderDemand {
    pub(crate) placeholder: PlaceholderId,
    pub(crate) location: SourceSpan,
}

impl PlaceholderDemand {
    pub(crate) fn from_lookup(
        error: LookupError,
        location: SourceSpan,
    ) -> Result<Self, LookupError> {
        match error {
            LookupError::UnfilledPlaceholder(placeholder) => Ok(Self {
                placeholder,
                location,
            }),
            error => Err(error),
        }
    }

    pub(crate) fn diagnostic(self, graph: &ModuleGraph) -> Diagnostic {
        Diagnostic::at_source(self.location, demand_message(graph, self.placeholder))
    }
}

/// Span-only callers preserve the original AST source supplied by their
/// resolver. The namespace's lookup file need not be the quotation's source.
pub(crate) fn unfilled_placeholder(
    graph: &ModuleGraph,
    placeholder: PlaceholderId,
    span: Span,
) -> Diagnostic {
    Diagnostic::new(span, demand_message(graph, placeholder))
}

fn demand_message(graph: &ModuleGraph, placeholder: PlaceholderId) -> String {
    let Some(marker) = graph.placeholder(placeholder) else {
        return "placeholder demand belongs to a different source graph".into();
    };
    let mut message = format!(
        "unfilled #placeholder '{}' cannot supply this declaration",
        graph.symbols().name(marker.name())
    );
    if let Some(source) = graph.sources().get(marker.location().source) {
        let text = source.text();
        let at = text.floor_char_boundary(marker.location().span.start.min(text.len()));
        let prefix = &text[..at];
        let line = prefix.bytes().filter(|&byte| byte == b'\n').count() + 1;
        let column = prefix.rsplit('\n').next().unwrap_or("").chars().count() + 1;
        message.push_str(&format!(
            "\n{}:{line}:{column}: note: placeholder reserved here",
            source.path().display()
        ));
    }
    message
}
