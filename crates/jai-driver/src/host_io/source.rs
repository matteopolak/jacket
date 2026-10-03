use super::*;
impl crate::CompilationUnit {
    /// Resolve source bodies with explicit stdio provenance and host root authority.
    /// The configured graph sources are inspected; no supplied library is loaded.
    pub fn resolve_with_file_session(
        &self,
        target: jai_types::BuildTarget,
        session: &mut FileCompilerSession,
    ) -> Result<jai_sema::Program, crate::Error> {
        let file_abi = jai_sema::FileAbiBindingContext::from_graph(
            &self.graph,
            &self.options.import_dirs,
            target.clone(),
        );
        let options = jai_sema::ResolveOptions {
            compiler: Some(jai_sema::CompilerBindingContext::from_graph(
                &self.graph,
                &self.options.import_dirs,
                session.compiler().root(),
            )),
            target: Some(target),
            file_abi,
            ..Default::default()
        };
        let program = jai_sema::resolve_graph_with_options(&self.graph, &options, session)
            .map_err(|error| self.located(error))?;
        if let Some(error) = session.compiler().error() {
            return Err(crate::Error::CompilerReport(error.clone()));
        }
        Ok(program)
    }
}
