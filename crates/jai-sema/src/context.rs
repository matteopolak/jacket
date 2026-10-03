//! Context records share one workspace schema and lexical availability contract.
use super::*;
#[cfg(test)]
mod tests;
use jai_types::FieldId;

pub(crate) struct Schema {
    pub definition: ContextDefinition,
    pub fields: HashMap<Symbol, FieldId>,
    pub constants: HashMap<Symbol, ContextConstant>,
    metadata: crate::local_declarations::RecordMetadata,
    origins: HashMap<FieldId, jai_modules::FileInstanceId>,
    notes: HashMap<FieldId, Box<[Box<[u8]>]>>,
}
#[derive(Clone)]
pub(crate) enum ContextConstant {
    Value(jai_ir::ConstantValue),
    Procedure { procedure: ProcedureId, ty: TypeId },
}
pub(crate) struct Field {
    pub name: Symbol,
    pub value: jai_ir::ConstantValue,
    pub span: Span,
    pub syntax: syntax::FieldDeclaration,
    pub alignment: Option<u32>,
    pub file: jai_modules::FileInstanceId,
    pub notes: Box<[Box<[u8]>]>,
}
impl Schema {
    pub(crate) fn record_metadata(&self) -> crate::local_declarations::RecordMetadata {
        self.metadata.clone()
    }
    pub(crate) fn field_origin(&self, field: FieldId) -> Option<jai_modules::FileInstanceId> {
        self.origins.get(&field).copied()
    }
    pub(crate) fn append_reflection_metadata(
        &self,
        metadata: &mut jai_types::ReflectionMetadata,
        symbols: &Symbols,
    ) {
        let ty = self.definition.record_type;
        metadata.name(ty, b"Context".as_slice());
        metadata.record(ty, jai_types::ReflectedRecordMetadata::default());
        for field in &self.metadata.fields {
            metadata.field(
                field.id,
                jai_types::ReflectedFieldMetadata {
                    name: field.name.map(|name| symbols.name(name).as_bytes().into()),
                    using: field.syntax.using(),
                },
            );
            if let Some(notes) = self.notes.get(&field.id) {
                metadata.field_notes(field.id, notes.clone());
            }
        }
    }
    pub(crate) fn new(
        types: &mut TypeRegistry,
        record_type: TypeId,
        fields: Vec<Field>,
        constants: HashMap<Symbol, ContextConstant>,
    ) -> Result<Self, Diagnostic> {
        let mut seen = std::collections::HashSet::new();
        for field in &fields {
            if constants.contains_key(&field.name) || !seen.insert(field.name) {
                return Err(Diagnostic::new(
                    field.span,
                    "conflicting context member name",
                ));
            }
        }
        types
            .define_record_with_layout(
                record_type,
                fields
                    .iter()
                    .map(|field| field.value.ty)
                    .collect::<Vec<_>>(),
                jai_types::RecordLayout {
                    field_alignments: if fields.iter().any(|field| field.alignment.is_some()) {
                        fields.iter().map(|field| field.alignment).collect()
                    } else {
                        Box::default()
                    },
                    ..jai_types::RecordLayout::default()
                },
            )
            .map_err(|error| Diagnostic::new(Span::default(), error.to_string()))?;
        let pointer_type = types
            .pointer(record_type)
            .map_err(|error| Diagnostic::new(Span::default(), error.to_string()))?;
        let mut names = HashMap::new();
        let mut defaults = Vec::new();
        let mut metadata = Vec::new();
        let mut origins = HashMap::new();
        let mut notes = HashMap::new();
        for (index, field) in fields.into_iter().enumerate() {
            let descriptor = types
                .field(record_type, index)
                .map_err(|error| Diagnostic::new(field.span, error.to_string()))?;
            names.insert(field.name, descriptor.id);
            origins.insert(descriptor.id, field.file);
            notes.insert(descriptor.id, field.notes);
            metadata.push(crate::local_declarations::FieldMetadata {
                name: Some(field.name),
                id: descriptor.id,
                ty: descriptor.ty,
                syntax: field.syntax.into(),
            });
            defaults.push(field.value);
        }
        Ok(Self {
            definition: ContextDefinition {
                record_type,
                pointer_type,
                default: jai_ir::ConstantValue {
                    ty: record_type,
                    kind: ConstantKind::Record(defaults),
                },
            },
            fields: names,
            origins,
            notes,
            metadata: crate::local_declarations::RecordMetadata {
                name: None,
                kind: jai_types::RecordKind::Struct,
                fields: metadata,
            },
            constants,
        })
    }
}

impl Resolver<'_> {
    pub(crate) fn context_field(
        &self,
        ty: TypeId,
        member: Symbol,
        span: Span,
    ) -> Option<Result<Vec<FieldId>, Diagnostic>> {
        let schema = self.context?;
        if ty != schema.definition.record_type {
            return None;
        }
        Some(match schema.fields.get(&member) {
            Some(field) => Ok(vec![*field]),
            None => self.field_path(ty, member, span),
        })
    }

    pub(crate) fn require_context(&self, span: Span) -> Result<&Schema, Diagnostic> {
        if !self.context_available {
            return Err(Diagnostic::new(
                span,
                "implicit context is unavailable; use push_context to establish one",
            ));
        }
        self.context
            .ok_or_else(|| Diagnostic::new(span, "context schema is unavailable"))
    }
    pub(crate) fn context_expression(&mut self, span: Span) -> Result<Expr, Diagnostic> {
        if let Some(binding) = self.source_context_binding(span)? {
            return self.binding_expression(binding, span);
        }
        let schema = self.require_context(span)?;
        Ok(Expr::Typed {
            ty: schema.definition.record_type,
            value: ValueExpr::Context {
                ty: schema.definition.record_type,
            },
        })
    }
    pub(crate) fn context_place(&mut self, span: Span) -> Result<Place, Diagnostic> {
        if let Some(binding) = self.source_context_binding(span)? {
            return match binding {
                Binding::Storage(storage) => Ok(storage.place()),
                _ => Err(Diagnostic::new(
                    span,
                    "source context binding is not writable storage",
                )),
            };
        }
        let schema = self.require_context(span)?;
        Place::context(schema.definition.record_type, self.types)
            .map_err(|error| Diagnostic::new(span, error.to_string()))
    }
    pub(crate) fn context_constant_value(
        &self,
        ty: TypeId,
        name: Symbol,
        span: Span,
    ) -> Option<Result<Expr, Diagnostic>> {
        let schema = self.context?;
        if schema.definition.record_type != ty {
            return None;
        }
        let constant = schema.constants.get(&name)?;
        Some(match constant {
            ContextConstant::Value(value) => {
                self.typed_value(value.clone().into_expression(), value.ty, span)
            }
            ContextConstant::Procedure { procedure, ty } => self.typed_value(
                ValueExpr::ProcedureValue {
                    procedure: *procedure,
                    ty: *ty,
                },
                *ty,
                span,
            ),
        })
    }
    pub(crate) fn context_member(&mut self, name: Symbol, span: Span) -> Result<Expr, Diagnostic> {
        if let Some(binding) = self.source_context_binding(span)? {
            let value = self.binding_expression(binding, span)?;
            return self.member_value(value, name, span);
        }
        let schema = self.require_context(span)?;
        if let Some(value) = self.context_constant_value(schema.definition.record_type, name, span)
        {
            return value;
        }
        let fields = self.field_path(schema.definition.record_type, name, span)?;
        let mut place = self.context_place(span)?;
        for field in fields {
            place = self
                .places
                .field(place, field, self.types)
                .map_err(|error| Diagnostic::new(span, error.to_string()))?;
        }
        self.typed_value(ValueExpr::Load(place), place.ty(), span)
    }
    pub(crate) fn check_call_context(
        &self,
        signature: &jai_types::ProcedureType,
        span: Span,
    ) -> Result<(), Diagnostic> {
        if signature.context == ContextMode::Implicit && !self.context_available {
            return Err(Diagnostic::new(
                span,
                "calling an implicit-context procedure requires push_context in a #c_call or #no_context procedure",
            ));
        }
        Ok(())
    }
    pub(crate) fn push_context(
        &mut self,
        value: Option<&syntax::Expression>,
        body: &[syntax::Statement],
        span: Span,
    ) -> Result<Statement, Diagnostic> {
        let definition = &self
            .context
            .ok_or_else(|| Diagnostic::new(span, "context schema is unavailable"))?
            .definition;
        let ty = definition.record_type;
        let value = match value {
            Some(value) => {
                let expression = self.expr_expected(value, ty)?;
                self.coerce_value(expression, ty, value.span)?
            }
            None => definition.default.clone().into_expression(),
        };
        let id = PushContextId::new(self.procedure, self.next_push);
        self.next_push += 1;
        let previous_push = self.active_push.replace(id);
        let previous = std::mem::replace(&mut self.context_available, true);
        let result = self.block(body, true);
        self.context_available = previous;
        self.active_push = previous_push;
        let body = result?;
        self.debug
            .attach_block(&[DebugPathStep::Child(DebugBranch::PushContext)]);
        Ok(Statement::PushContext { id, value, body })
    }
}
