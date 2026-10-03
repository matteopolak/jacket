//! Preserve reserved module identity across backedges and finish export linking.
use super::*;

#[derive(Clone, Copy)]
pub(super) struct PublishedImport {
    pub(super) file: FileInstanceId,
    pub(super) module: ModuleId,
    pub(super) namespace: Option<Symbol>,
    pub(super) using: bool,
    pub(super) visibility: Visibility,
    pub(super) location: SourceSpan,
}

impl Builder<'_> {
    pub(super) fn refresh_import_exports(&mut self) -> Result<(), GraphError> {
        if !self.has_import_backedges || self.published_imports.is_empty() {
            return Ok(());
        }
        // Bindings only grow: a finite source graph has finite original names
        // and declaration IDs. Existing overload merging canonicalizes sorted,
        // deduplicated member sets; repeated publications are idempotent.
        loop {
            let previous = self
                .graph
                .modules
                .iter()
                .map(|module| module.exports.clone())
                .collect::<Vec<_>>();
            for import in self.published_imports.clone() {
                self.bind_import_names(
                    import.file,
                    import.namespace,
                    import.using,
                    import.visibility,
                    import.module,
                    import.location,
                )?;
            }
            if previous
                .iter()
                .zip(&self.graph.modules)
                .all(|(exports, module)| *exports == module.exports)
            {
                return Ok(());
            }
        }
    }
}
