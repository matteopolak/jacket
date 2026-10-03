//! Lexical external declarations resolve existing types and library identities.
use super::*;

impl Resolver<'_> {
    pub(crate) fn bind_local_external_statement(
        &mut self,
        declaration: &syntax::Declaration,
    ) -> Result<Statement, Diagnostic> {
        let syntax::Declaration::External {
            name,
            ty,
            binding,
            attributes,
        } = declaration
        else {
            unreachable!("external statement adapter requires an external declaration")
        };
        let id = self
            .local_scopes
            .frames
            .last()
            .and_then(|frame| frame.external.get(name))
            .copied()
            .ok_or_else(|| {
                Diagnostic::new(
                    self.span,
                    "external data lacks its registered local source identity",
                )
            })?;
        let value = self.define_local_external_data(id, *name, ty, binding, attributes)?;
        self.bind_name(*name, value)?;
        Ok(Statement::Block(Block {
            statements: vec![],
            flow: Flow::FallsThrough,
        }))
    }
    pub(in crate::local_declarations) fn define_local_external_data(
        &mut self,
        declaration: LocalDeclarationId,
        name: Symbol,
        annotation: &syntax::TypeSyntax,
        binding: &syntax::ExternalDataBinding,
        attributes: &[syntax::DeclarationAttribute],
    ) -> Result<Binding, Diagnostic> {
        let location = location(declaration)?;
        let Some(base) = self
            .meta
            .external_globals
            .ready_file_prefix()
            .map(<[Global]>::to_vec)
        else {
            if let Some(context) = self.compile_time {
                context.record_pending(vec![jai_vm::Dependency::GlobalDefinitions]);
            }
            return Err(Diagnostic::at_source(
                location,
                "external data waits for the complete file-global prefix",
            ));
        };
        if self.symbols.name(name) == "_" {
            return Err(Diagnostic::at_source(
                location,
                "external data requires a named source declaration",
            ));
        }
        let ty = self.lexical_annotation(annotation, location.span)?;
        let alignment = self.declaration_attributes_alignment(attributes)?;
        let source = match &binding.source {
            syntax::ExternalDataSource::Program => jai_ir::ExternalDataSource::Program,
            syntax::ExternalDataSource::Library(path) => {
                jai_ir::ExternalDataSource::Library(self.foreign_library_path(path, binding.span)?)
            }
        };
        let identity = self.meta.external_globals.local_identity(declaration)?;
        let metadata = ExternalData::new(
            identity,
            ty,
            source,
            binding
                .symbol
                .clone()
                .unwrap_or_else(|| self.symbols.name(name).to_owned()),
            location,
            self.types,
        )
        .map_err(|error| {
            if let jai_ir::ExternalDataError::Type(jai_ir::IrError::Type(
                jai_types::TypeError::Incomplete(ty),
            )) = &error
                && let Some(context) = self.compile_time
            {
                context.record_pending(vec![jai_vm::Dependency::Type(*ty)]);
            }
            Diagnostic::at_source(location, error.to_string())
        })?;
        let global =
            self.meta
                .external_globals
                .publish(declaration, &base, metadata, self.types)?;
        if let Some(alignment) = alignment {
            self.meta
                .storage_alignments
                .set_global(global.id(), alignment)
                .map_err(|error| Diagnostic::at_source(location, error.to_string()))?;
        }
        Ok(Binding::Storage(global.storage()))
    }
}
