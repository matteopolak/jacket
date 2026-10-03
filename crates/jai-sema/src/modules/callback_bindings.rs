//! Recover source callback metadata in the declaration's original file scope.
use super::*;
use crate::procedure_values::contracts::{ContractOrigin, ContractOwner, ContractSyntax};
use jai_modules::Binding as GraphBinding;

impl FileScope<'_> {
    pub(crate) fn callback_imported_declarations(
        &self,
        binding: GraphBinding,
        span: Span,
    ) -> Result<Option<Vec<DeclarationId>>, Diagnostic> {
        let callable = match binding {
            GraphBinding::OverloadSet(_) => true,
            GraphBinding::Declaration(id) => {
                self.declarations.signatures.contains_key(&id)
                    || self.declarations.callable_aliases.contains_key(&id)
                    || self.declarations.generics.borrow().is_template(id)
            }
            _ => false,
        };
        if callable {
            self.imported_callable(binding, span).map(Some)
        } else {
            Ok(None)
        }
    }
    pub(crate) fn callback_record_parameters(
        &self,
        declaration: DeclarationId,
    ) -> Option<Vec<syntax::RecordParameter>> {
        let declaration = self.declarations.graph.declaration(declaration)?;
        let FileDeclarationKind::Record(record) = &declaration.syntax().kind else {
            return None;
        };
        Some(record.parameters.clone())
    }
    pub(crate) fn callback_record_alias(
        &self,
        declaration: DeclarationId,
        name: Symbol,
    ) -> Option<syntax::TypeSyntax> {
        let declaration = self.declarations.graph.declaration(declaration)?;
        let FileDeclarationKind::Record(record) = &declaration.syntax().kind else {
            return None;
        };
        record.members.iter().find_map(|member| match member {
            syntax::RecordMember::TypeAlias(alias) if alias.name == name => Some(alias.ty.clone()),
            _ => None,
        })
    }
    pub(crate) fn recheck_callback_body(&self, id: ProcedureId) -> Result<bool, Diagnostic> {
        self.declarations
            .generics
            .borrow_mut()
            .recheck_callback_body(id)
    }
    pub(crate) fn callback_contract_header(
        &self,
        procedure: ProcedureId,
    ) -> Option<(
        FileInstanceId,
        Vec<syntax::Parameter>,
        Vec<syntax::ProcedureResult>,
    )> {
        let origin = self
            .declarations
            .signatures
            .iter()
            .find_map(|(&id, signature)| {
                (signature.id == procedure)
                    .then_some((id, self.declarations.graph.declaration(id)?.file()))
            })
            .or_else(|| {
                self.declarations
                    .generics
                    .borrow()
                    .callback_source_origin(procedure)
            })?;
        let declaration = self.declarations.graph.declaration(origin.0)?;
        let (parameters, results) = match &declaration.syntax().kind {
            FileDeclarationKind::Procedure(source) => (&source.parameters, &source.results),
            FileDeclarationKind::ProcedurePrototype(source) => {
                (&source.parameters, &source.results)
            }
            _ => return None,
        };
        Some((origin.1, parameters.clone(), results.clone()))
    }
    pub(crate) fn callback_procedure_signature(&self, id: ProcedureId) -> Option<Signature> {
        self.declarations
            .signatures
            .values()
            .find(|signature| signature.id == id)
            .cloned()
            .or_else(|| self.declarations.generics.borrow().callback_signature(id))
    }
    pub(crate) fn callback_result_type_parameters(&self, id: ProcedureId) -> Vec<Symbol> {
        self.declarations
            .generics
            .borrow()
            .callback_result_type_parameters(id)
    }
    pub(crate) fn callback_specialized_type(
        &self,
        id: ProcedureId,
        name: Symbol,
    ) -> Option<TypeId> {
        self.declarations
            .generics
            .borrow()
            .callback_substitution(id)?
            .ty(name)
    }

    pub(crate) fn callback_inferred_global_signature(
        &self,
        place: Place,
        span: Span,
    ) -> Result<Option<Signature>, Diagnostic> {
        let Some((&id, _)) = self.declarations.values.iter().find(
            |(_, binding)| matches!(binding, Binding::Storage(storage) if storage.place() == place),
        ) else {
            return Ok(None);
        };
        let declaration = self.declarations.graph.declaration(id).unwrap();
        let FileDeclarationKind::Global(global) = &declaration.syntax().kind else {
            return Ok(None);
        };
        let syntax::Declaration::Inferred {
            initializer, ..
        } = global.declaration.source()
        else {
            return Ok(None);
        };
        let path = match &initializer.kind {
            syntax::ExpressionKind::Name(name) => path(*name),
            syntax::ExpressionKind::QualifiedName(path) => path.clone(),
            _ => return Ok(None),
        };
        let signature = FileScope {
            file: declaration.file(),
            ..*self
        }
        .signature(&path, span)?;
        if signature.ty != place.ty() {
            return Err(Diagnostic::new(
                span,
                "inferred callback metadata has a different canonical signature",
            ));
        }
        Ok(Some(signature.clone()))
    }
}

impl FileScope<'_> {
    /// Expand only source aliases; nominal records retain their own field identities.
    pub(crate) fn callback_contract_syntax(
        &self,
        ty: &syntax::TypeSyntax,
        span: Span,
    ) -> Result<ContractSyntax, Diagnostic> {
        self.callback_contract_syntax_inner(ty, span, &mut std::collections::HashSet::new(), 0)
    }

    fn callback_contract_syntax_inner(
        &self,
        ty: &syntax::TypeSyntax,
        span: Span,
        active: &mut std::collections::HashSet<DeclarationId>,
        depth: usize,
    ) -> Result<ContractSyntax, Diagnostic> {
        if depth >= crate::constant_limits::MAX_CONSTANT_DEPTH {
            return Err(Diagnostic::new(
                span,
                "callback contract exceeds source type depth",
            ));
        }
        if let syntax::TypeSyntax::Named(path) = ty {
            if path.members.is_empty()
                && self
                    .substitution
                    .is_some_and(|substitution| substitution.ty(path.root).is_some())
            {
                return self.callback_contract_syntax_inner(
                    &syntax::TypeSyntax::Variable(path.root),
                    span,
                    active,
                    depth + 1,
                );
            }
            if let Ok(jai_modules::Binding::Declaration(id)) =
                self.declarations.graph.lookup(self.file, path)
            {
                let declaration = self.declarations.graph.declaration(id).unwrap();
                if let FileDeclarationKind::TypeAlias(alias) = &declaration.syntax().kind {
                    if !active.insert(id) {
                        return Err(Diagnostic::new(span, "cyclic callback contract alias"));
                    }
                    let result = FileScope {
                        file: declaration.file(),
                        ..*self
                    }
                    .callback_contract_syntax_inner(
                        &alias.ty,
                        span,
                        active,
                        depth + 1,
                    );
                    active.remove(&id);
                    return result;
                }
            }
        }
        let origin = ContractOrigin {
            file: Some(self.file),
            source: Some(self.declarations.graph.file(self.file).unwrap().source()),
            owner: ContractOwner::File,
            lexical_scopes: vec![],
            target: self.annotation_target(),
            substitution: self.substitution.cloned(),
        };
        let mut retained = ContractSyntax::resolve(ty, origin, |child| {
            self.callback_contract_syntax_inner(child, span, active, depth + 1)
        })?;
        if let Some(callback) = &mut retained.callback {
            use crate::procedure_values::source_annotations::{
                ProcedureAnnotationKey, ProcedureAnnotationOrigin,
            };
            let key = ProcedureAnnotationKey::new(
                ProcedureAnnotationOrigin::Graph(self.file),
                &callback.original,
                self.substitution,
                callback.environment.target,
            );
            callback.proof = self.checked_procedure_annotation(&key);
        }
        Ok(retained)
    }
    pub(crate) fn contract_file(&self) -> FileInstanceId {
        self.file
    }

    pub(crate) fn callback_global_contract_syntax(
        &self,
        place: Place,
        span: Span,
    ) -> Result<Option<ContractSyntax>, Diagnostic> {
        let Some((&id, _)) = self.declarations.values.iter().find(
            |(_, binding)| matches!(binding,Binding::Storage(storage) if storage.place()==place),
        ) else {
            return Ok(None);
        };
        let declaration = self.declarations.graph.declaration(id).unwrap();
        let FileDeclarationKind::Global(global) = &declaration.syntax().kind else {
            return Ok(None);
        };
        let ty = match global.declaration.source() {
            syntax::Declaration::UnresolvedExplicit {
                ty, ..
            }
            | syntax::Declaration::External {
                ty, ..
            } => ty,
            syntax::Declaration::Inferred {
                initializer, ..
            } => {
                let syntax::ExpressionKind::TypeCast {
                    ty, ..
                } = &initializer.kind
                else {
                    return Ok(None);
                };
                ty
            }
            _ => return Ok(None),
        };
        FileScope {
            file: declaration.file(),
            ..*self
        }
        .callback_contract_syntax(ty, span)
        .map(Some)
    }

    pub(crate) fn callback_result_contract_sources(
        &self,
        procedure: ProcedureId,
        span: Span,
    ) -> Result<Option<Vec<Option<ContractSyntax>>>, Diagnostic> {
        let origin = self
            .declarations
            .signatures
            .iter()
            .find_map(|(&id, signature)| {
                (signature.id == procedure)
                    .then_some((id, self.declarations.graph.declaration(id).unwrap().file()))
            })
            .or_else(|| {
                self.declarations
                    .generics
                    .borrow()
                    .callback_source_origin(procedure)
            });
        let Some((id, file)) = origin else {
            return Ok(None);
        };
        let declaration = self.declarations.graph.declaration(id).unwrap();
        let results = match &declaration.syntax().kind {
            FileDeclarationKind::Procedure(source) => &source.results,
            FileDeclarationKind::ProcedurePrototype(source) => &source.results,
            _ => return Ok(None),
        };
        let substitution = self
            .declarations
            .generics
            .borrow()
            .callback_substitution(procedure);
        let scope = FileScope {
            file,
            substitution: substitution.as_ref(),
            ..*self
        };
        results
            .iter()
            .map(|result| match &result.binding {
                syntax::ResultBinding::Typed {
                    ty, ..
                } => scope.callback_contract_syntax(ty, span).map(Some),
                syntax::ResultBinding::InferredDefault(_) => Ok(None),
            })
            .collect::<Result<Vec<_>, _>>()
            .map(Some)
    }

    pub(crate) fn callback_field_contract_syntax(
        &self,
        record: TypeId,
        field: FieldId,
        span: Span,
    ) -> Result<Option<ContractSyntax>, Diagnostic> {
        if let Some(schema) = self.declarations.context.as_ref()
            && schema.definition.record_type == record
            && let Some(file) = schema.field_origin(field)
        {
            let metadata = schema.record_metadata();
            let Some(metadata) = metadata.fields.iter().find(|metadata| metadata.id == field)
            else {
                return Ok(None);
            };
            let Some(ty) = metadata
                .syntax
                .named_binding()
                .and_then(field_contract_annotation)
            else {
                return Ok(None);
            };
            return FileScope {
                file,
                ..*self
            }
            .callback_contract_syntax(ty, span)
            .map(Some);
        }
        let Some(record) = self.declarations.nominals.records.get(&record) else {
            return Ok(None);
        };
        let Some(metadata) = record.fields.iter().find(|metadata| metadata.id == field) else {
            return Ok(None);
        };
        let Some(ty) = field_contract_annotation(&metadata.syntax.binding) else {
            return Ok(None);
        };
        FileScope {
            file: record.file,
            ..*self
        }
        .callback_contract_syntax(ty, span)
        .map(Some)
    }
}

fn field_contract_annotation(binding: &syntax::FieldBinding) -> Option<&syntax::TypeSyntax> {
    match binding {
        syntax::FieldBinding::Explicit {
            ty, ..
        } => Some(ty),
        syntax::FieldBinding::Inferred(expression) => match &expression.kind {
            syntax::ExpressionKind::TypeCast {
                ty, ..
            } => Some(ty),
            _ => None,
        },
    }
}

impl FileScope<'_> {
    pub(crate) fn callback_contract_syntax_in_specialization(
        &self,
        file: FileInstanceId,
        substitution: Option<&crate::polymorphism::Substitution>,
        ty: &syntax::TypeSyntax,
        span: Span,
    ) -> Result<ContractSyntax, Diagnostic> {
        FileScope {
            file,
            substitution,
            ..*self
        }
        .callback_contract_syntax(ty, span)
    }
    pub(crate) fn callback_contract_substitution(
        &self,
        procedure: ProcedureId,
    ) -> Option<crate::polymorphism::Substitution> {
        self.declarations
            .generics
            .borrow()
            .callback_substitution(procedure)
    }
    pub(crate) fn callback_contract_syntax_in_file(
        &self,
        file: FileInstanceId,
        ty: &syntax::TypeSyntax,
        span: Span,
    ) -> Result<ContractSyntax, Diagnostic> {
        FileScope {
            file,
            ..*self
        }
        .callback_contract_syntax(ty, span)
    }
}
