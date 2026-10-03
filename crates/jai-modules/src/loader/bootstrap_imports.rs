use super::*;

impl Builder<'_> {
    pub(super) fn shared_prelude_import(
        &mut self,
        import: &ImportDeclaration,
    ) -> Result<Option<ModuleId>, GraphError> {
        if import.mode != ImportMode::Search || import.target != "Preload" {
            return Ok(None);
        }
        let Some(module) = self.graph.prelude else {
            return Ok(None);
        };
        if [&import.arguments.instance, &import.arguments.program]
            .into_iter()
            .any(|arguments| arguments.as_ref().is_some_and(|values| !values.is_empty()))
        {
            return Err(self.located(
                import.location,
                "the shared Preload module cannot be specialized with import arguments",
            ));
        }
        let path = self.graph.source_requests[&module].path.clone();
        if self.active_modules.contains(&module) {
            self.has_import_backedges = true;
            return Ok(Some(module));
        }
        if !self.completed_modules.contains(&module) {
            self.expand_module(module, &path)?;
        }
        Ok(Some(module))
    }
}
