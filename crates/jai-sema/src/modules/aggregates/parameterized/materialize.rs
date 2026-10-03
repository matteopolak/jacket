use super::*;

impl<F> TypeResolver<'_, '_, F>
where
    F: FnMut(FileInstanceId, &syntax::Expression) -> Result<ScalarConstant, LocatedDiagnostic>,
{
    pub(super) fn instantiate(
        &mut self,
        id: DeclarationId,
        substitution: Substitution,
        span: Span,
    ) -> TypeResult<TypeId> {
        let file = self
            .graph
            .declaration(id)
            .expect("template origin exists")
            .file();
        self.instantiate_at(
            id,
            substitution,
            jai_source::SourceSpan {
                source: self.graph.file(file).unwrap().source(),
                span,
            },
        )
    }
    pub(super) fn instantiate_at(
        &mut self,
        id: DeclarationId,
        substitution: Substitution,
        location: jai_source::SourceSpan,
    ) -> TypeResult<TypeId> {
        self.in_lexical_scope(false, |resolver| {
            resolver.instantiate_inner(id, substitution, location)
        })
    }
    pub(super) fn instantiate_inner(
        &mut self,
        id: DeclarationId,
        substitution: Substitution,
        location: jai_source::SourceSpan,
    ) -> TypeResult<TypeId> {
        let span = location.span;
        let declaration = self
            .graph
            .declaration(id)
            .expect("template origin belongs to graph");
        let syntax::FileDeclarationKind::Record(record) = &declaration.syntax().kind else {
            return Err(failure(
                self.graph,
                declaration.file(),
                Diagnostic::new(span, "specialization origin is not a record template"),
            ));
        };
        let normalized = self.normalize_bindings(
            declaration.file(),
            record,
            substitution,
            span,
            BindingPolicy::Initial,
        )?;
        let normalized = if record.modify.is_some() {
            let initial_key = RecordSpecializationKey {
                template: RecordTemplateId(id),
                arguments: normalized.arguments.clone(),
            };
            // A recursive reference inside the accepted body's own definition
            // denotes that exact reservation; it must not rerun its modifier.
            if let Some(owner) = self.records.active_owner()
                && self.records.key_for_type(owner) == Some(&initial_key)
            {
                return Ok(owner);
            }
            let intent = modifier_intents::RecordModifierIntent::new(
                self.graph,
                initial_key,
                normalized.substitution,
            )?;
            match self.records.modifiers.prepare(intent, location) {
                modifier_intents::RecordModifierReadiness::Pending(id) => {
                    return Err(TypeFailure::Pending(
                        PendingRecordModifier { id, location }.into(),
                    ));
                }
                modifier_intents::RecordModifierReadiness::Failed(error) => {
                    return Err(error.into());
                }
                modifier_intents::RecordModifierReadiness::Ready(accepted) => {
                    // A modifier can change a preceding type slot. Re-resolve
                    // every dependent formal and normalize the accepted values
                    // before computing the immutable specialization key.
                    let mut normalized = self.normalize_bindings(
                        declaration.file(),
                        record,
                        accepted,
                        span,
                        BindingPolicy::Accepted,
                    )?;
                    // A declaration default is an input to its own modifier
                    // recipe. It cannot become a per-TypeId proof merely by
                    // evaluating it beside whichever accepted instance won
                    // canonical reservation first.
                    normalized.defaults.clear();
                    normalized
                }
            }
        } else {
            normalized
        };
        let NormalizedRecord {
            substitution,
            defaults,
            arguments,
        } = normalized;
        let key = RecordSpecializationKey {
            template: RecordTemplateId(id),
            arguments,
        };
        let ty = match self.records.reserve(key.clone(), record.kind, self.types) {
            Reservation::Existing(ty) => return Ok(ty),
            Reservation::Resolve(ty) => ty,
        };
        if !self.records.enter() {
            self.records.retry(&key);
            return Err(failure(
                self.graph,
                declaration.file(),
                Diagnostic::new(span, "record specialization exceeds compiler depth budget"),
            ));
        }
        let body = RecordBody {
            modify: None,
            ..record.into()
        };
        let result = self.in_nominal_context(NominalAnnotationContext::None, |resolver| {
            resolver.materialize_body(
                ty,
                Some(RecordTemplateId(id)),
                declaration.file(),
                body,
                &substitution,
            )
        });
        self.records.leave();
        match result {
            Ok((shape, substitution)) => {
                self.records.complete(
                    &key,
                    SpecializedRecord {
                        origin: Some(RecordTemplateId(id)),
                        file: declaration.file(),
                        shape,
                        substitution,
                        nested: false,
                        defaults,
                    },
                );
                if let Err(error) =
                    self.records
                        .validate_using(self.graph, self.nominals, self.types, true)
                {
                    self.records.retry(&key);
                    return Err(error.into());
                }
                Ok(ty)
            }
            Err(error) => {
                self.records.retry(&key);
                Err(error)
            }
        }
    }
    fn normalize_bindings(
        &mut self,
        file: FileInstanceId,
        record: &syntax::RecordDeclaration,
        supplied: Substitution,
        span: Span,
        policy: BindingPolicy,
    ) -> TypeResult<NormalizedRecord> {
        let mut substitution = Substitution::default();
        let mut defaults = HashMap::new();
        let mut arguments = Vec::new();
        for parameter in &record.parameters {
            let expected = self.parameter_type(file, parameter, &substitution)?;
            let default_expression = match &parameter.binding {
                syntax::RecordParameterBinding::Typed { default, .. } => default.as_ref(),
                syntax::RecordParameterBinding::InferredDefault(expression) => Some(expression),
            };
            let default = default_expression
                .map(|expression| self.baked(file, expression, expected, Some(&substitution)))
                .transpose()?;
            if let Some(default) = &default {
                defaults.insert(parameter.name, default.clone());
            }
            let value = supplied
                .constant(parameter.name)
                .cloned()
                .or_else(|| supplied.ty(parameter.name).map(BakedValue::Type))
                .or(default)
                .ok_or_else(|| {
                    failure(
                        self.graph,
                        file,
                        Diagnostic::new(span, "unbound record template argument"),
                    )
                })?;
            let value = match value {
                BakedValue::Type(ty) if expected == self.types.meta_type() => BakedValue::Type(ty),
                BakedValue::Code(id) if expected == self.types.code_type() => BakedValue::Code(id),
                value if matches!(policy, BindingPolicy::Accepted) => {
                    crate::overloads::recheck_baked_value(
                        self.types,
                        self.records,
                        value,
                        expected,
                        &substitution,
                        span,
                    )
                    .map_err(|error| {
                        failure(
                            self.graph,
                            file,
                            Diagnostic::new(
                                span,
                                format!(
                                    "baked record argument differs from its formal parameter type: {}",
                                    error.message,
                                ),
                            ),
                        )
                    })?
                }
                value => {
                    let value = value.into_runtime(expected, self.types).map_err(|error| failure(self.graph,
                        file, Diagnostic::new(span, format!("baked record argument differs from its formal parameter type: {error}"))))?;
                    BakedValue::runtime(value, self.types).map_err(|error| {
                        failure(self.graph, file, Diagnostic::new(span, error.to_string()))
                    })?
                }
            };
            substitution.bind_constant(parameter.name, value.clone());
            arguments.push(value);
        }
        Ok(NormalizedRecord {
            substitution,
            defaults,
            arguments: arguments.into_boxed_slice(),
        })
    }
    pub(super) fn materialize(
        &mut self,
        ty: TypeId,
        origin: RecordTemplateId,
        file: FileInstanceId,
        record: &syntax::RecordDeclaration,
        substitution: &Substitution,
    ) -> TypeResult<(RecordMetadata, Substitution)> {
        // A referenced declaration's fields have their own defining context;
        // a procedure header ban must not leak into that separate definition.
        self.in_nominal_context(NominalAnnotationContext::None, |resolver| {
            resolver.materialize_body(ty, Some(origin), file, record.into(), substitution)
        })
    }
    pub(super) fn materialize_body(
        &mut self,
        ty: TypeId,
        origin: Option<RecordTemplateId>,
        file: FileInstanceId,
        record: RecordBody<'_>,
        substitution: &Substitution,
    ) -> TypeResult<(RecordMetadata, Substitution)> {
        self.records.push_owner(ty);
        let result = (|| {
            let retained = self.records.selected_source(ty).cloned();
            let members = match &retained {
                Some(source) => std::borrow::Cow::Borrowed(source.members.as_ref()),
                None => self.checked_members(file, record.members, substitution)?,
            };
            if members.len() > 65_536 {
                return Err(failure(
                    self.graph,
                    file,
                    Diagnostic::new(
                        record.span,
                        "selected record source exceeds compiler declaration budget",
                    ),
                ));
            }
            let result = self.materialize_body_inner(
                ty,
                origin,
                file,
                RecordBody {
                    members: members.as_ref(),
                    ..record
                },
                substitution,
            )?;
            if self.records.selected_source(ty).is_none() {
                self.records
                    .remember_selected_source(state::SelectedRecordSource {
                        owner: ty,
                        file,
                        location: jai_source::SourceSpan {
                            source: self
                                .graph
                                .file(file)
                                .expect("record source file exists")
                                .source(),
                            span: record.span,
                        },
                        parameters: substitution.clone(),
                        members: members.into_owned().into(),
                    });
            }
            Ok(result)
        })();
        self.records.pop_owner(ty);
        if result.is_ok() {
            self.records.remember_source(
                ty,
                jai_source::SourceSpan {
                    source: self
                        .graph
                        .file(file)
                        .expect("record source file exists")
                        .source(),
                    span: record.span,
                },
            );
        }
        result
    }
    fn materialize_body_inner(
        &mut self,
        ty: TypeId,
        origin: Option<RecordTemplateId>,
        file: FileInstanceId,
        record: RecordBody<'_>,
        substitution: &Substitution,
    ) -> TypeResult<(RecordMetadata, Substitution)> {
        if let Some(modify) = record.modify {
            return Err(failure(
                self.graph,
                file,
                Diagnostic::new(
                    modify.span,
                    "record #modify requires checked specialization modifier execution",
                ),
            ));
        }
        let substitution = self.member_scope(ty, origin, file, record, substitution)?;
        if self.types.record_definition(ty).is_err() {
            let mut names = HashSet::new();
            let mut fields = Vec::new();
            for field in record.physical_fields() {
                if record.kind == jai_types::RecordKind::Union
                    && field.conversion() == syntax::FieldConversion::Implicit
                {
                    return Err(failure(
                        self.graph,
                        file,
                        Diagnostic::new(
                            field.span(),
                            "union #as field conversions are not supported",
                        ),
                    ));
                }
                if field.name().is_some_and(|name| !names.insert(name)) {
                    return Err(failure(
                        self.graph,
                        file,
                        Diagnostic::new(field.span(), "duplicate record field"),
                    ));
                }
                let field_ty = self.in_record_annotation(ty, |resolver| match field {
                    FieldSourceRef::Named(field) => match &field.binding {
                        syntax::FieldBinding::Explicit { ty, .. } => {
                            resolver.resolve(file, ty, Some(&substitution), field.span)
                        }
                        syntax::FieldBinding::Inferred(expression) => {
                            resolver.inferred_default_type(file, expression, &substitution)
                        }
                    },
                    FieldSourceRef::AnonymousRecord(record) => {
                        resolver.inline_record(file, record, Some(&substitution))
                    }
                })?;
                fields.push(field_ty);
            }
            let graph = self.graph;
            let layout = diagnostic_bridge(self.graph, |diagnostic| {
                super::super::layout::record_body_layout(
                    graph,
                    file,
                    record.span,
                    record.attributes,
                    record.members,
                    &mut |file, expression| {
                        self.scalar(file, expression, Some(&substitution))
                            .map_err(&mut *diagnostic)
                    },
                )
            })?;
            self.types
                .define_record_with_layout(ty, fields, layout)
                .map_err(|e| {
                    failure(
                        self.graph,
                        file,
                        Diagnostic::new(record.span, e.to_string()),
                    )
                })?;
        }
        let mut fields = Vec::new();
        for (index, field) in record.physical_fields().enumerate() {
            let descriptor = self.types.field(ty, index).map_err(|e| {
                failure(
                    self.graph,
                    file,
                    Diagnostic::new(field.span(), e.to_string()),
                )
            })?;
            fields.push(FieldMetadata {
                name: field.name(),
                id: descriptor.id,
                ty: descriptor.ty,
                syntax: field.to_owned(),
            });
        }
        let source = self
            .graph
            .sources()
            .get(
                self.graph
                    .file(file)
                    .expect("record defining file exists")
                    .source(),
            )
            .expect("record defining source exists")
            .text();
        self.records.define_reflection(
            ty,
            reflection::record_body_metadata(record, source),
            fields
                .iter()
                .map(|field| (field.id, reflection::notes(field.syntax.notes(), source)))
                .collect(),
        );
        let shape = RecordMetadata {
            name: record.name,
            kind: record.kind,
            fields,
        };
        let overrides = default_overrides::collect(
            self.graph,
            default_overrides::OverrideSource {
                file,
                owner: ty,
                shape: &shape,
                members: record.members,
            },
            self.types,
            self.nominals,
            self.records,
        )?;
        self.records.define_default_overrides(ty, overrides);
        Ok((shape, substitution))
    }
}

enum BindingPolicy {
    Initial,
    Accepted,
}

struct NormalizedRecord {
    substitution: Substitution,
    defaults: HashMap<jai_source::Symbol, BakedValue>,
    arguments: Box<[BakedValue]>,
}
