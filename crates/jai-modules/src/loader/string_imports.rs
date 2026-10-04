//! Source literals are retained entry sources, never provider filenames.
use super::*;
impl Builder<'_> {
    pub(super) fn string_import_source(
        &mut self,
        file: FileInstanceId,
        import: &ImportDeclaration,
    ) -> Result<SourceId, GraphError> {
        let key = (
            import.location.source,
            import.location.span.start,
            import.location.span.end,
        );
        if let Some(&source) = self.string_imports.get(&key) {
            return Ok(source);
        }
        let original = self
            .graph
            .sources
            .get(self.graph.files[file.index()].source)
            .unwrap();
        let anchor = original.resolution_path().to_owned();
        let label = original.path().with_extension(format!(
            "string-import-{}-{}.jai",
            import.location.span.start, import.location.span.end
        ));
        let source = self.graph.sources.insert_embedded(
            label,
            import.target.clone(),
            import.location,
            anchor,
        );
        let syntax = jai_syntax::parse_file(
            self.graph.sources.get(source).unwrap(),
            &mut self.graph.symbols,
        )
        .map_err(|diagnostic| {
            let rendered = diagnostic.render(&self.graph.sources);
            GraphError::Located {
                diagnostic,
                rendered,
            }
        })?;
        self.syntax.insert(source, syntax);
        self.string_imports.insert(key, source);
        Ok(source)
    }
}
