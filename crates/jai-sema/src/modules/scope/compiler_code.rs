//! Compiler-only source lookup has no native signature adapter.
use super::*;

impl<'a> FileScope<'a> {
    pub(crate) fn compiler_code_template(
        &self,
        path: &NamePath,
        registry: &crate::compiler_code::CompilerCodeRegistry,
        span: Span,
    ) -> Result<Option<crate::compiler_code::CompilerCodeTemplate>, Diagnostic> {
        let Ok(id) = self.declaration(path, span) else {
            return Ok(None);
        };
        Ok(registry.template(id).cloned())
    }

    pub(crate) fn in_file(self, file: FileInstanceId) -> Self {
        Self {
            file,
            substitution: None,
            ..self
        }
    }

    pub(crate) fn has_compiler_quote_source(
        &self,
        source_file: FileInstanceId,
        location: jai_source::SourceSpan,
    ) -> bool {
        let graph = self.declarations.graph;
        graph
            .file(source_file)
            .is_some_and(|file| file.source() == location.source)
            && graph.sources().get(location.source).is_some_and(|record| {
                let text = record.text();
                location.span.start <= location.span.end
                    && location.span.end <= text.len()
                    && text.is_char_boundary(location.span.start)
                    && text.is_char_boundary(location.span.end)
            })
    }
}
