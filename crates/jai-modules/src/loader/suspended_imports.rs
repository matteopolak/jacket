use super::*;

#[derive(Clone, Copy)]
pub(super) enum ImportBindingScope {
    File,
    Lexical,
}

impl Builder<'_> {
    pub(super) fn expand_import_module(
        &mut self,
        file: FileInstanceId,
        import: &ImportDeclaration,
        module: ModuleId,
        path: &Path,
        scope: ImportBindingScope,
    ) -> Result<(), GraphError> {
        match self.expand_module(module, path) {
            Err(
                error @ GraphError::Pending {
                    ..
                },
            ) => {
                if matches!(scope, ImportBindingScope::File) {
                    self.publish_import(file, import, module)?;
                }
                Err(error)
            }
            result => result,
        }
    }

    pub(super) fn publish_import(
        &mut self,
        file: FileInstanceId,
        import: &ImportDeclaration,
        module: ModuleId,
    ) -> Result<(), GraphError> {
        self.bind_import(file, import, module)?;
        if !self.graph.imports.iter().any(|edge| {
            edge.file == file && edge.module == module && edge.location == import.location
        }) {
            if import.namespace.is_none() || import.using {
                self.published_imports.push(import_cycles::PublishedImport {
                    file,
                    module,
                    namespace: import.namespace,
                    using: import.using,
                    visibility: import.visibility,
                    location: import.location,
                });
            }
            self.graph.imports.push(ImportEdge {
                file,
                module,
                location: import.location,
            });
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests;
