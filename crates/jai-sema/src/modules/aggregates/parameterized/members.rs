//! Record member declarations are bound separately from runtime field storage.
use super::*;

impl<F> TypeResolver<'_, '_, F>
where
    F: FnMut(FileInstanceId, &syntax::Expression) -> Result<ScalarConstant, LocatedDiagnostic>,
{
    pub(super) fn member_scope(
        &mut self,
        owner: TypeId,
        origin: Option<RecordTemplateId>,
        file: FileInstanceId,
        record: RecordBody<'_>,
        outer: &Substitution,
    ) -> TypeResult<Substitution> {
        use syntax::RecordMember as M;
        let mut names = HashSet::new();
        let previous = self.records.member_bindings(owner).cloned();
        let mut scope = self.reserve_body_member_names(owner, file, record, outer)?;
        let mut pending = Vec::new();
        let mut methods = Vec::new();
        let mut source_members = Vec::new();
        for (index, member) in record.members.iter().enumerate() {
            let (name, span) = match member {
                M::DefaultOverride { .. } | M::AnonymousRecord(_) => continue,
                M::Field(field) => (field.name, field.span),
                M::Constant(value) => (value.name, value.span),
                M::TypeAlias(value) => (value.name, value.span),
                M::Record(value) => (value.name, value.span),
                M::Procedure(value) => (value.name, value.span),
                M::ProcedurePrototype(value) => (value.name, value.span),
                M::Enum(value) => (value.name, value.span),
                M::Insert(value) => {
                    return Err(failure(
                        self.graph,
                        file,
                        Diagnostic::new(
                            value.span,
                            "record #insert requires checked declaration insertion scheduling",
                        ),
                    ));
                }
                M::Assert { span, .. }
                | M::Conditional { span, .. }
                | M::CompileTimeCases { span, .. } => {
                    return Err(failure(
                        self.graph,
                        file,
                        Diagnostic::new(
                            *span,
                            "record control member was not scheduled before namespace binding",
                        ),
                    ));
                }
            };
            if !names.insert(name) {
                return Err(failure(
                    self.graph,
                    file,
                    Diagnostic::new(span, "duplicate record member"),
                ));
            }
            if !matches!(member, M::Field(_)) {
                source_members.push(name);
            }
            match member {
                M::Record(nested) => {
                    if !nested.parameters.is_empty() {
                        return Err(failure(
                            self.graph,
                            file,
                            Diagnostic::new(
                                nested.span,
                                "nested parameterized records require member template declarations",
                            ),
                        ));
                    }
                    let ty = self
                        .records
                        .reserve_nested(owner, index, nested.kind, self.types);
                    shadow(&mut scope, nested.name, BakedValue::Type(ty));
                    pending.push(index);
                }
                M::Constant(source)
                    if matches!(
                        &source.initializer.kind,
                        syntax::ExpressionKind::ShortLambda(_)
                            | syntax::ExpressionKind::AnonymousProcedure(_)
                    ) =>
                {
                    methods.push(RecordMethod {
                        id: RecordMethodId {
                            owner,
                            member: index,
                        },
                        file,
                        source: RecordMethodSource::Constant(source.clone()),
                    });
                }
                M::Constant(_) | M::TypeAlias(_) | M::Enum(_) => pending.push(index),
                M::Procedure(value) => methods.push(RecordMethod {
                    id: RecordMethodId {
                        owner,
                        member: index,
                    },
                    file,
                    source: RecordMethodSource::Procedure(value.as_ref().clone()),
                }),
                M::ProcedurePrototype(value) => methods.push(RecordMethod {
                    id: RecordMethodId {
                        owner,
                        member: index,
                    },
                    file,
                    source: RecordMethodSource::Prototype(value.clone()),
                }),
                _ => {}
            }
        }
        for method in &methods {
            let name = method.source.name();
            if let Some(
                value @ BakedValue::Value(jai_ir::ConstantValue {
                    kind: jai_ir::ConstantKind::Procedure(_),
                    ..
                }),
            ) = previous.as_ref().and_then(|scope| scope.constant(name))
            {
                shadow(&mut scope, name, value.clone());
            }
        }
        self.records.reserve_namespace(owner, scope.clone());
        self.records.define_methods(owner, methods);
        self.records
            .reserve_method_environment(owner, file, record.name, scope.clone());
        // Transparent member dependencies may be forward references. A failed
        // lookup does not allocate a replacement nominal identity on retry.
        while !pending.is_empty() {
            let mut next = Vec::new();
            let mut failure = None;
            for index in pending.iter().copied() {
                let result = match &record.members[index] {
                    M::Record(nested) => self
                        .bind_nested_record(owner, origin, index, file, nested, &scope)
                        .map(|ty| (nested.name, BakedValue::Type(ty))),
                    M::Enum(enumeration) => self
                        .define_nested_enum(owner, index, file, enumeration, &scope)
                        .map(|()| {
                            (
                                enumeration.name,
                                BakedValue::Type(
                                    scope.ty(enumeration.name).expect("member enum reserved"),
                                ),
                            )
                        }),
                    M::TypeAlias(alias) => self
                        .resolve(file, &alias.ty, Some(&scope), alias.span)
                        .map(|ty| (alias.name, BakedValue::Type(ty))),
                    M::Constant(constant) => self
                        .constant_member(file, constant, &scope)
                        .map(|value| (constant.name, value)),
                    _ => unreachable!("pending members are records, aliases, constants, or enums"),
                };
                match result {
                    Ok((name, value)) => {
                        shadow(&mut scope, name, value);
                        self.records.reserve_namespace(owner, scope.clone());
                    }
                    Err(pending @ TypeFailure::Pending(_)) => return Err(pending),
                    Err(error @ TypeFailure::Diagnostic(_)) => {
                        failure.get_or_insert(error);
                        next.push(index);
                    }
                }
            }
            if next.len() == pending.len() {
                return Err(failure.expect("pending members have a resolution error"));
            }
            pending = next;
        }
        self.records.reserve_namespace(owner, scope.clone());
        for (index, member) in record.members.iter().enumerate() {
            if let M::Enum(enumeration) = member {
                self.define_nested_enum(owner, index, file, enumeration, &scope)?;
            }
            if let M::Record(nested) = member {
                self.bind_nested_record(owner, origin, index, file, nested, &scope)?;
            }
        }
        self.records.define_source_members(owner, source_members);
        Ok(scope)
    }
    fn bind_nested_record(
        &mut self,
        owner: TypeId,
        origin: Option<RecordTemplateId>,
        index: usize,
        file: FileInstanceId,
        nested: &syntax::RecordDeclaration,
        scope: &Substitution,
    ) -> TypeResult<TypeId> {
        let ty = self
            .records
            .reserve_nested(owner, index, nested.kind, self.types);
        if self.records.record(ty).is_some() {
            self.records
                .inherit_nested_bindings(ty, scope)
                .map_err(|()| {
                    failure(
                        self.graph,
                        file,
                        Diagnostic::new(
                            nested.span,
                            "record namespace inheritance exceeds declaration budget",
                        ),
                    )
                })?;
            return Ok(ty);
        }
        if !self.records.enter() {
            return Err(failure(
                self.graph,
                file,
                Diagnostic::new(
                    nested.span,
                    "record specialization exceeds compiler depth budget",
                ),
            ));
        }
        let result = self.materialize_body(ty, origin, file, nested.into(), scope);
        self.records.leave();
        let (shape, substitution) = result?;
        self.records.complete_nested(
            ty,
            SpecializedRecord {
                origin,
                file,
                shape,
                substitution,
                nested: true,
                defaults: HashMap::new(),
            },
        );
        Ok(ty)
    }
}

fn shadow(substitution: &mut Substitution, name: jai_source::Symbol, value: BakedValue) {
    substitution
        .constants
        .retain(|binding| binding.name != name);
    substitution.types.retain(|binding| binding.name != name);
    substitution.bind_constant(name, value);
}
