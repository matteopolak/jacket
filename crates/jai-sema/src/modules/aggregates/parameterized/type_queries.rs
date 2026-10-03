//! Query canonical declaration types without evaluating annotation operands.
use super::*;

impl<F> TypeResolver<'_, '_, F>
where
    F: FnMut(FileInstanceId, &syntax::Expression) -> Result<ScalarConstant, LocatedDiagnostic>,
{
    pub(super) fn annotation_type_of(
        &mut self,
        file: FileInstanceId,
        value: &syntax::Expression,
        substitution: Option<&Substitution>,
    ) -> TypeResult<TypeId> {
        let path = super::super::types::annotation_query_path(value)
            .map_err(|error| failure(self.graph, file, error))?;
        let lexical = self.lexical.filter(|_| self.lexical_active);
        let mut last_error = None;
        for count in (0..=path.members.len()).rev() {
            let prefix = syntax::NamePath {
                root: path.root,
                members: path.members[..count].to_vec(),
            };
            if let Some(binding) = lexical.and_then(|scope| scope.lookup(&prefix)) {
                match binding {
                    LexicalTypeArgument::Type(ty) => {
                        if count == path.members.len() {
                            return self.query_meta_type(file, value.span);
                        }
                        return self.query_fields(file, *ty, &path.members[count..], value.span);
                    }
                    LexicalTypeArgument::Scalar(scalar) => {
                        let ty = scalar.type_id(self.types);
                        return self.query_fields(file, ty, &path.members[count..], value.span);
                    }
                    LexicalTypeArgument::Typed(constant) => {
                        return self.query_fields(
                            file,
                            constant.ty,
                            &path.members[count..],
                            value.span,
                        );
                    }
                    LexicalTypeArgument::Code(_) => {
                        return self.query_fields(
                            file,
                            self.types.code_type(),
                            &path.members[count..],
                            value.span,
                        );
                    }
                }
            }
            if let Some(bound) = member_value(
                self.graph,
                file,
                self.nominals,
                self.records,
                substitution,
                &prefix,
                value.span,
            ) {
                let ty = match bound {
                    BakedValue::Type(ty) if count < path.members.len() => {
                        return self.query_fields(file, ty, &path.members[count..], value.span);
                    }
                    BakedValue::Type(_) => self.query_meta_type(file, value.span)?,
                    BakedValue::Value(value) => value.ty,
                    BakedValue::Float(value) => self.types.float(value.ty()),
                    BakedValue::String(_) => self.types.string(),
                    BakedValue::Code(_) => self.types.code_type(),
                };
                return self.query_fields(file, ty, &path.members[count..], value.span);
            }
            if let Some(ty) = self.nominals.value_type(self.graph, file, &prefix) {
                return self.query_fields(file, ty, &path.members[count..], value.span);
            }
            if let Some(ty) = self.explicit_annotation_value_type(file, &prefix, value.span)? {
                return self.query_fields(file, ty, &path.members[count..], value.span);
            }
            if let Ok(jai_modules::Binding::Parameter(id)) = self.graph.lookup(file, &prefix) {
                let ty = match &self.graph.parameter(id).expect("parameter binding exists").value {
                    jai_modules::ParameterValue::Scalar(value) => value.type_id(self.types),
                    jai_modules::ParameterValue::String(_) => self.types.string(),
                    jai_modules::ParameterValue::Type(module_type) if count < path.members.len() => {
                        self.nominals.resolve_module_type_with_specializations(self.graph, crate::modules::aggregates::types::ModuleTypeRequest { file, value: module_type, span: value.span }, self.types, self.records, self.evaluate,
                        )?
                    }
                    jai_modules::ParameterValue::Type(_) => self.query_meta_type(file, value.span)?,
                    jai_modules::ParameterValue::Enumeration(enumeration) => {
                        self.nominals.declarations.get(&enumeration.declaration).copied().ok_or_else(|| {
                            failure(self.graph, file, Diagnostic::new(value.span,
                                "annotation type_of is waiting for the module enum parameter's nominal type"))
                        })?
                    }
                    jai_modules::ParameterValue::ContextualMember(_) => return Err(failure(
                        self.graph, file, Diagnostic::new(value.span,
                            "annotation type_of is waiting for a contextual module parameter type"))),
                };
                return self.query_fields(file, ty, &path.members[count..], value.span);
            }
            match self.resolve(
                file,
                &syntax::TypeSyntax::Named(prefix),
                substitution,
                value.span,
            ) {
                Ok(_) if count == path.members.len() => {
                    return self.query_meta_type(file, value.span);
                }
                Ok(ty) => return self.query_fields(file, ty, &path.members[count..], value.span),
                Err(pending @ TypeFailure::Pending(_)) => return Err(pending),
                Err(TypeFailure::Diagnostic(error)) => {
                    last_error = Some(TypeFailure::Diagnostic(error))
                }
            }
        }
        if lexical.is_some_and(|scope| scope.roots.contains_key(&path.root)) {
            return Err(failure(
                self.graph,
                file,
                Diagnostic::new(
                    value.span,
                    "annotation type_of has no such lexical namespace member",
                ),
            ));
        }
        Err(last_error.expect("a query path has at least its root"))
    }

    /// Header preparation may query a global before ordinary global storage is
    /// defined. Its explicit annotation is available without evaluating its
    /// initializer, and must be resolved in its own file scope.
    fn explicit_annotation_value_type(
        &mut self,
        file: FileInstanceId,
        path: &syntax::NamePath,
        span: Span,
    ) -> TypeResult<Option<TypeId>> {
        let Ok(jai_modules::Binding::Declaration(id)) = self.graph.lookup(file, path) else {
            return Ok(None);
        };
        let declaration = self
            .graph
            .declaration(id)
            .expect("graph declaration binding exists");
        let annotation = match &declaration.syntax().kind {
            syntax::FileDeclarationKind::Constant(constant) => match &constant.ty {
                Some(annotation) => annotation.clone(),
                None => return Ok(None),
            },
            syntax::FileDeclarationKind::Global(global) => match global.declaration.source() {
                syntax::Declaration::GroupMember {
                    ..
                } => unreachable!("source() returns the original non-group declaration"),
                syntax::Declaration::Explicit {
                    ty, ..
                } => syntax::TypeSyntax::Builtin(syntax::BuiltinType::Scalar(*ty)),
                syntax::Declaration::UnresolvedExplicit {
                    ty, ..
                } => ty.clone(),
                syntax::Declaration::External {
                    ty, ..
                } => ty.clone(),
                syntax::Declaration::Inferred {
                    ..
                } => return Ok(None),
            },
            _ => return Ok(None),
        };
        let defining_file = declaration.file();
        let defining_span = declaration.location().span;
        if self.aliases.len() >= 256 || !self.aliases.insert(id) {
            return Err(failure(
                self.graph,
                file,
                Diagnostic::new(
                    span,
                    "cyclic or excessively deep annotation value type dependency",
                ),
            ));
        }
        let result = self.in_lexical_scope(false, |resolver| {
            resolver.resolve(defining_file, &annotation, None, defining_span)
        });
        self.aliases.remove(&id);
        result.map(Some)
    }

    fn query_fields(
        &mut self,
        file: FileInstanceId,
        mut ty: TypeId,
        members: &[jai_source::Symbol],
        span: Span,
    ) -> TypeResult<TypeId> {
        let mut remaining = 65_536;
        for member in members {
            self.prepare_promoted_query_metadata(file, ty, span, &mut remaining)?;
            ty = self
                .nominals
                .declared_field_type(
                    self.graph,
                    ty,
                    std::slice::from_ref(member),
                    self.types,
                    Some(self.records),
                    span,
                )
                .map_err(|error| failure(self.graph, file, error))?;
        }
        Ok(ty)
    }

    fn prepare_promoted_query_metadata(
        &mut self,
        file: FileInstanceId,
        ty: TypeId,
        span: Span,
        remaining: &mut usize,
    ) -> TypeResult<()> {
        let mut pending = vec![ty];
        let mut visited = HashSet::new();
        while let Some(mut owner) = pending.pop() {
            query_charge(self.graph, file, remaining, 1, span)?;
            let mut pointer_depth = 0usize;
            while let Ok(jai_types::TypeKind::Pointer(pointee)) = self.types.kind(owner) {
                pointer_depth += 1;
                if pointer_depth > 256 {
                    return Err(failure(
                        self.graph,
                        file,
                        Diagnostic::new(
                            span,
                            "annotation type_of exceeds pointer type depth limit",
                        ),
                    ));
                }
                owner = *pointee;
                query_charge(self.graph, file, remaining, 1, span)?;
            }
            if !visited.insert(owner) {
                continue;
            }
            if self.records.is_materializing(owner) {
                return Err(failure(
                    self.graph,
                    file,
                    Diagnostic::new(
                        span,
                        "annotation type_of is waiting for recursive record field metadata",
                    ),
                ));
            }
            if self.records.record(owner).is_none() && !self.nominals.records.contains_key(&owner)
                && let Some(declaration) = self.nominals.declarations.iter().find_map(|(id, candidate)| {
                    if *candidate != owner { return None; }
                    let source = self.graph.declaration(*id)?;
                    matches!(&source.syntax().kind, syntax::FileDeclarationKind::Record(record) if record.parameters.is_empty())
                        .then_some(*id)
                })
            {
                member_enums::prepare_static_record(self.graph, declaration, self.types, self.nominals,
                    self.records, self.evaluate)?;
            }
            if let Some(record) = self.records.record(owner) {
                query_charge(self.graph, file, remaining, record.shape.fields.len(), span)?;
                pending.extend(
                    record
                        .shape
                        .fields
                        .iter()
                        .filter(|field| field.syntax.using())
                        .map(|field| field.ty),
                );
            } else if let Some(record) = self.nominals.records.get(&owner) {
                query_charge(self.graph, file, remaining, record.fields.len(), span)?;
                pending.extend(
                    record
                        .fields
                        .iter()
                        .filter(|field| field.syntax.using)
                        .map(|field| field.ty),
                );
            }
        }
        Ok(())
    }

    fn query_meta_type(&mut self, file: FileInstanceId, span: Span) -> TypeResult<TypeId> {
        self.nominals
            .runtime_type_for_graph(self.graph, self.types)
            .map_err(|error| failure(self.graph, file, Diagnostic::new(span, error.to_string())))
    }
}

fn query_charge(
    graph: &ModuleGraph,
    file: FileInstanceId,
    remaining: &mut usize,
    count: usize,
    span: Span,
) -> Result<(), LocatedDiagnostic> {
    *remaining = remaining.checked_sub(count).ok_or_else(|| {
        located(
            graph,
            file,
            Diagnostic::new(
                span,
                "annotation type_of exceeds metadata preparation budget",
            ),
        )
    })?;
    Ok(())
}
