//! Define source-owned nominal shapes and declaration-site initializer values.
use super::*;

#[derive(Clone, Copy)]
pub(super) struct RecordSource<'a> {
    pub(super) id: LocalDeclarationId,
    pub(super) name: Option<Symbol>,
    pub(super) kind: RecordKind,
    pub(super) attributes: &'a [syntax::RecordAttribute],
    pub(super) notes: &'a [syntax::NoteSyntax],
    pub(super) span: Span,
}

impl Resolver<'_> {
    pub(super) fn define_local_record(
        &mut self,
        source: RecordSource<'_>,
        fields: &[FieldSource],
        members: &[syntax::RecordMember],
        has_namespace_members: bool,
    ) -> Result<TypeId, Diagnostic> {
        let ty = self.define_local_record_shape(source, fields, members, has_namespace_members)?;
        self.finish_local_record_defaults(ty)?;
        Ok(ty)
    }

    pub(super) fn define_local_record_shape(
        &mut self,
        source: RecordSource<'_>,
        fields: &[FieldSource],
        members: &[syntax::RecordMember],
        has_namespace_members: bool,
    ) -> Result<TypeId, Diagnostic> {
        let RecordSource {
            id,
            name,
            kind,
            attributes,
            notes,
            span,
        } = source;
        let ty = self.meta.local_declarations.entries[&id].nominal.unwrap();
        if self.types.record_definition(ty).is_err() {
            let mut names = HashSet::new();
            let mut field_types = Vec::new();
            for field in fields {
                if kind == RecordKind::Union && field.conversion() != syntax::FieldConversion::None
                {
                    return Err(Diagnostic::new(
                        field.span(),
                        "union #as fields require an active alternative conversion policy",
                    ));
                }
                if field
                    .as_ref()
                    .name()
                    .is_some_and(|name| !names.insert(name))
                {
                    return Err(Diagnostic::new(field.span(), "duplicate record field"));
                }
                let field_ty = self.record_field_annotation(ty, |resolver| match field {
                    FieldSource::Named(field) => match &field.binding {
                        syntax::FieldBinding::Explicit {
                            ty, ..
                        } => resolver.lexical_annotation(ty, field.span),
                        syntax::FieldBinding::Inferred(expression) => {
                            resolver.local_default_type(expression)
                        }
                    },
                    FieldSource::AnonymousRecord(record) => resolver.local_inline_record(record),
                })?;
                field_types.push(field_ty);
            }
            let mut layout = jai_types::RecordLayout::default();
            for attribute in attributes {
                match attribute {
                    syntax::RecordAttribute::Alignment(expression) => {
                        layout.minimum_alignment = Some(self.local_alignment(expression)?);
                    }
                    syntax::RecordAttribute::NoPadding => layout.packed = true,
                    syntax::RecordAttribute::TypeInfoNone
                    | syntax::RecordAttribute::Reflection(_) => {}
                }
            }
            let mut alignments = Vec::new();
            for field in fields {
                let mut alignment = None;
                for attribute in field.as_ref().attributes() {
                    match attribute {
                        syntax::FieldAttribute::Alignment(expression) => {
                            alignment = Some(self.local_alignment(expression)?)
                        }
                        syntax::FieldAttribute::Placement(_) => {}
                    }
                }
                alignments.push(alignment);
            }
            if alignments.iter().any(Option::is_some) {
                layout.field_alignments = alignments.into_boxed_slice();
            }
            self.types
                .define_record_with_placements(
                    ty,
                    field_types,
                    layout,
                    crate::record_placements::source_placement_ordinals(members)?,
                )
                .map_err(|error| Diagnostic::new(span, error.to_string()))?;
        }
        crate::reflection::apply_source_record_attributes(self.types, ty, attributes)
            .map_err(|error| Diagnostic::new(span, error.to_string()))?;
        let mut metadata = Vec::new();
        for (index, field) in fields.iter().enumerate() {
            let descriptor = self
                .types
                .field(ty, index)
                .map_err(|error| Diagnostic::new(field.span(), error.to_string()))?;
            let source = match field {
                FieldSource::AnonymousRecord(_) => None,
                FieldSource::Named(field) => match &field.binding {
                    syntax::FieldBinding::Explicit {
                        ty, ..
                    } => Some(ty),
                    syntax::FieldBinding::Inferred(expression) => match &expression.kind {
                        syntax::ExpressionKind::TypeCast {
                            ty, ..
                        } => Some(ty),
                        _ => None,
                    },
                },
            };
            if let Some(source) = source
                && let Some(contract) =
                    self.annotation_value_contract(descriptor.ty, source, field.span())?
            {
                self.meta
                    .callbacks
                    .field_contracts
                    .insert(descriptor.id, contract);
            }
            metadata.push(FieldMetadata {
                name: field.as_ref().name(),
                id: descriptor.id,
                ty: descriptor.ty,
                syntax: field.clone(),
            });
        }
        // Install the complete shape before materializing nested defaults.
        self.meta.local_declarations.records.insert(
            ty,
            RecordMetadata {
                using: members
                    .iter()
                    .filter_map(|member| {
                        if let syntax::RecordMember::Using(value) = member {
                            Some(value.clone())
                        } else {
                            None
                        }
                    })
                    .collect(),
                name,
                kind,
                fields: metadata.clone(),
            },
        );
        self.validate_field_conversion_record(ty, span)?;
        let textual_flags = attributes.iter().fold(
            if kind == RecordKind::Union {
                2
            } else {
                0
            },
            |flags, attribute| {
                flags
                    | match attribute {
                        syntax::RecordAttribute::NoPadding => 4,
                        syntax::RecordAttribute::TypeInfoNone => 8,
                        syntax::RecordAttribute::Alignment(_)
                        | syntax::RecordAttribute::Reflection(_) => 0,
                    }
            },
        );
        let record_notes = self.local_note_bytes(notes)?;
        self.meta.local_declarations.record_reflection.insert(
            ty,
            jai_types::ReflectedRecordMetadata {
                notes: record_notes,
                textual_flags,
                status_flags: 4,
                nontextual_flags: if kind == RecordKind::Union {
                    64
                } else {
                    0
                },
                unsupported_members: has_namespace_members,
            },
        );
        for field in &metadata {
            let notes = self.local_note_bytes(field.syntax.notes())?;
            self.meta
                .local_declarations
                .field_notes
                .insert(field.id, notes);
        }
        Ok(ty)
    }

    pub(super) fn finish_local_record_defaults(&mut self, ty: TypeId) -> Result<(), Diagnostic> {
        let metadata = self.meta.local_declarations.records[&ty].fields.clone();
        for field in metadata {
            if self
                .meta
                .local_declarations
                .defaults
                .contains_key(&field.id)
            {
                continue;
            }
            let initializer = field.syntax.initializer();
            if let Some(expression) = initializer {
                if matches!(expression.kind, syntax::ExpressionKind::Uninitialized) {
                    // The original field recipe means no write. It supplies no
                    // immutable value, but its shape can still be used by an
                    // explicit whole-record uninitialized declaration.
                    continue;
                }
                let value = self.local_typed_constant(expression, field.ty)?;
                self.meta
                    .local_declarations
                    .defaults
                    .insert(field.id, value);
            }
        }
        Ok(())
    }

    fn local_alignment(&mut self, expression: &syntax::Expression) -> Result<u32, Diagnostic> {
        let alignment = crate::storage_alignment::integer_alignment(ScalarConstant::Literal(
            i128::from(self.local_integer_count(expression)?),
        ))
        .ok_or_else(|| {
            Diagnostic::new(
                expression.span,
                "alignment requires a nonzero power-of-two integer constant representable as u32",
            )
        })?;
        Ok(alignment)
    }

    pub(crate) fn local_inline_record(
        &mut self,
        record: &syntax::RecordTypeSyntax,
    ) -> Result<TypeId, Diagnostic> {
        if !record.parameters.is_empty() {
            return Err(Diagnostic::new(
                record.span,
                "anonymous parameterized records require a named template declaration",
            ));
        }
        if let Some(modifier) = &record.modify {
            return Err(Diagnostic::new(
                modifier.span,
                "local record #modify requires checked modifier execution",
            ));
        }
        while self.local_scopes.frames.len() < self.scopes.len() {
            self.push_local_scope();
        }
        let scope = self.local_scopes.frames.last().unwrap().id;
        let id = LocalDeclarationId {
            scope,
            start: record.span.start,
            end: record.span.end,
            ordinal: usize::MAX,
        };
        let entry = self.meta.local_declarations.entries.entry(id).or_default();
        if entry.nominal.is_none() {
            entry.nominal = Some(self.types.reserve_record(record.kind));
        }
        let ty = entry.nominal.unwrap();
        self.remember_local_type_origin(id, None, ty, record.span);
        self.define_local_record_body(
            RecordSource {
                id,
                name: None,
                kind: record.kind,
                attributes: &record.attributes,
                notes: &record.notes,
                span: record.span,
            },
            &record.members,
        )
    }

    fn local_note_bytes(
        &self,
        notes: &[syntax::NoteSyntax],
    ) -> Result<Box<[Box<[u8]>]>, Diagnostic> {
        let Some(scope) = self.graph_scope else {
            if let Some(note) = notes.first() {
                return Err(Diagnostic::new(
                    note.span,
                    "local declaration notes require their defining source text",
                ));
            }
            return Ok(Box::new([]));
        };
        notes
            .iter()
            .map(|note| {
                let source = self.debug.source().unwrap_or_else(|| scope.source());
                let text = scope
                    .source_record(source)
                    .ok_or_else(|| {
                        Diagnostic::new(note.span, "local declaration note source is unavailable")
                    })?
                    .text()
                    .get(note.span.start..note.span.end)
                    .ok_or_else(|| {
                        Diagnostic::new(
                            note.span,
                            "local declaration note source range is unavailable",
                        )
                    })?;
                Ok(text.strip_prefix('@').unwrap_or(text).as_bytes().into())
            })
            .collect()
    }

    pub(crate) fn validate_local_using_fields(&self, span: Span) -> Result<(), Diagnostic> {
        for (&ty, record) in &self.meta.local_declarations.records {
            let mut pending = vec![(ty, HashSet::new())];
            let mut names = HashSet::new();
            while let Some((ty, mut ancestors)) = pending.pop() {
                if !ancestors.insert(ty) {
                    return Err(Diagnostic::new(span, "cyclic using field promotion"));
                }
                let shape = self.record_metadata(ty, span)?;
                for field in shape.fields {
                    if let Some(name) = field.name
                        && !names.insert(name)
                    {
                        return Err(Diagnostic::new(
                            field.syntax.span(),
                            format!(
                                "ambiguous promoted record member '{}'",
                                self.symbols.name(name)
                            ),
                        ));
                    }
                    if field.syntax.using() {
                        self.record_metadata(field.ty, field.syntax.span())
                            .map_err(|_| {
                                Diagnostic::new(
                                    field.syntax.span(),
                                    "using field requires a record value",
                                )
                            })?;
                        pending.push((field.ty, ancestors.clone()));
                    }
                }
            }
            let _ = record;
        }
        Ok(())
    }
}
