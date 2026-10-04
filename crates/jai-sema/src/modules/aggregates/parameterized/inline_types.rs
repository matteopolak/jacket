//! Source-located anonymous schemas share the nominal reservation state machine.
use super::*;
use jai_types::{IntegerType, TypeKind};
use state::{InlineTypeKey, MemberEnum};

impl<F> TypeResolver<'_, '_, F>
where
    F: FnMut(FileInstanceId, &syntax::Expression) -> Result<ScalarConstant, LocatedDiagnostic>,
{
    pub(super) fn inline_record(
        &mut self,
        file: FileInstanceId,
        record: &syntax::RecordTypeSyntax,
        substitution: Option<&Substitution>,
    ) -> TypeResult<TypeId> {
        if !record.parameters.is_empty() {
            return Err(failure(
                self.graph,
                file,
                Diagnostic::new(
                    record.span,
                    "anonymous record templates require a named declaration",
                ),
            ));
        }
        let scope = substitution.cloned().unwrap_or_default();
        let key = self.inline_key(file, record.span, substitution);
        let ty = match self
            .records
            .reserve_inline_record(key.clone(), record.kind, self.types)
        {
            Reservation::Existing(ty) => return Ok(ty),
            Reservation::Resolve(ty) => ty,
        };
        if !self.records.enter() {
            self.records.retry_inline(&key);
            return Err(failure(
                self.graph,
                file,
                Diagnostic::new(
                    record.span,
                    "anonymous record resolution exceeds compiler depth budget",
                ),
            ));
        }
        let result = self.materialize_body(ty, None, file, record.into(), &scope);
        self.records.leave();
        match result {
            Ok((shape, substitution)) => {
                self.records.complete_nested(
                    ty,
                    SpecializedRecord {
                        origin: None,
                        file,
                        shape,
                        substitution,
                        nested: true,
                        defaults: HashMap::new(),
                    },
                );
                self.records.complete_inline(&key);
                Ok(ty)
            }
            Err(error) => {
                self.records.retry_inline(&key);
                Err(error)
            }
        }
    }
    pub(super) fn inline_enum(
        &mut self,
        file: FileInstanceId,
        enumeration: &syntax::EnumTypeSyntax,
        substitution: Option<&Substitution>,
    ) -> TypeResult<TypeId> {
        let scope = substitution.cloned().unwrap_or_default();
        let key = self.inline_key(file, enumeration.span, substitution);
        let representation = match &enumeration.representation {
            None => IntegerType::S64,
            Some(annotation) => {
                let ty = self.resolve(file, annotation, Some(&scope), enumeration.span)?;
                match self
                    .types
                    .kind(ty)
                    .expect("resolved representation belongs to registry")
                {
                    TypeKind::Integer(integer) => *integer,
                    _ => {
                        return Err(failure(
                            self.graph,
                            file,
                            Diagnostic::new(
                                enumeration.span,
                                "enum representation requires an integer type",
                            ),
                        ));
                    }
                }
            }
        };
        let ty = match self
            .records
            .reserve_inline_enum(key.clone(), representation, self.types)
        {
            Reservation::Existing(ty) => return Ok(ty),
            Reservation::Resolve(ty) => ty,
        };
        let result = self.define_inline_enum(ty, file, enumeration, representation, &scope);
        match result {
            Ok(()) => {
                self.records.complete_inline(&key);
                Ok(ty)
            }
            Err(error) => {
                self.records.retry_inline(&key);
                Err(error)
            }
        }
    }
    fn inline_key(
        &self,
        file: FileInstanceId,
        span: Span,
        substitution: Option<&Substitution>,
    ) -> InlineTypeKey {
        // Equivalent lexical environments may reach a site in different lookup
        // orders. Keep names and typed values, but canonicalize their ordering.
        let owner = self.records.active_owner();
        // Forward-resolved member bindings may grow; the containing instance is
        // the stable owner of each anonymous field site throughout resolution.
        let mut substitution = if owner.is_some() {
            Substitution::default()
        } else {
            substitution.cloned().unwrap_or_default()
        };
        substitution.types.sort_by(|left, right| {
            self.graph
                .symbols()
                .name(left.name)
                .cmp(self.graph.symbols().name(right.name))
        });
        substitution.constants.sort_by(|left, right| {
            self.graph
                .symbols()
                .name(left.name)
                .cmp(self.graph.symbols().name(right.name))
        });
        InlineTypeKey {
            file,
            start: span.start,
            end: span.end,
            owner,
            substitution,
        }
    }
    fn define_inline_enum(
        &mut self,
        ty: TypeId,
        file: FileInstanceId,
        enumeration: &syntax::EnumTypeSyntax,
        representation: IntegerType,
        scope: &Substitution,
    ) -> TypeResult<()> {
        if self.records.member_enum(ty).is_some() {
            return Ok(());
        }
        let flags = enumeration.kind == syntax::EnumKind::Flags;
        let mut next = Some(if flags {
            1i128
        } else {
            0
        });
        let mut values = Vec::new();
        let mut names = HashSet::new();
        let mut scope = scope.clone();
        let graph = self.graph;
        let mut cursor = syntax::EnumMemberCursor::new(&enumeration.members);
        while let Some(member) = cursor.next_member(
            |expression| self.record_condition(file, expression, &scope),
            |error| failure(graph, file, error),
        )? {
            if !names.insert(member.name) {
                return Err(failure(
                    self.graph,
                    file,
                    Diagnostic::new(member.span, "duplicate enum member"),
                ));
            }
            if enumeration.specified && member.initializer.is_none() {
                return Err(failure(
                    self.graph,
                    file,
                    Diagnostic::new(
                        member.span,
                        "specified enum member requires an explicit value",
                    ),
                ));
            }
            let value = match &member.initializer {
                Some(expression) => self.scalar(file, expression, Some(&scope))?,
                None => ScalarConstant::Literal(next.ok_or_else(|| {
                    failure(
                        self.graph,
                        file,
                        Diagnostic::new(
                            member.span,
                            "enum automatic value overflow or unsupported flags progression",
                        ),
                    )
                })?),
            }
            .coerce(crate::ScalarType::Int(representation), member.span)
            .map_err(|error| failure(self.graph, file, error))?;
            let ScalarConstant::Int(value) = value else {
                unreachable!("integer representation");
            };
            next = if flags {
                if value.value() > 0 && (value.value() as u128).is_power_of_two() {
                    value.value().checked_mul(2)
                } else {
                    None
                }
            } else {
                value.value().checked_add(1)
            };
            scope
                .constants
                .retain(|binding| binding.name != member.name);
            scope.types.retain(|binding| binding.name != member.name);
            scope.bind_constant(member.name, BakedValue::integer(value, self.types));
            values.push((member.name, value));
        }
        self.types
            .define_enum(
                ty,
                values.iter().map(|(_, value)| *value).collect::<Vec<_>>(),
            )
            .map_err(|error| {
                failure(
                    self.graph,
                    file,
                    Diagnostic::new(enumeration.span, error.to_string()),
                )
            })?;
        self.records.define_member_enum(
            ty,
            MemberEnum {
                name: None,
                representation,
                flags,
                values,
            },
        );
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    struct Fixture(std::path::PathBuf);
    impl Fixture {
        fn new(source: &str) -> Self {
            static NEXT: AtomicUsize = AtomicUsize::new(0);
            let path = std::env::temp_dir().join(format!(
                "jai-inline-identity-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            std::fs::create_dir(&path).unwrap();
            std::fs::write(path.join("main.jai"), source).unwrap();
            Self(path)
        }
        fn graph(&self) -> ModuleGraph {
            ModuleGraph::load(
                &self.0.join("main.jai"),
                jai_modules::GraphOptions::default(),
            )
            .unwrap()
        }
    }
    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn source_sites_reuse_identity_and_retain_real_field_metadata() {
        let fixture = Fixture::new(
            r#"
            Root :: struct {
                left: struct { next:*Root; amount:int=7; @Range(7) } #no_padding;
                right: struct { next:*Root; amount:int=7; } #no_padding;
                mode: enum u8 { OFF; ON; ALIAS :: ON; } = .ALIAS;
            }
        "#,
        );
        let graph = fixture.graph();
        let mut types = TypeRegistry::new();
        let mut nominals = Nominals::reserve(&graph, &mut types).unwrap();
        let mut records = RecordSpecializations::default();
        let mut evaluate = |file, expression: &syntax::Expression| {
            jai_eval::evaluate_paths(expression, |_, span| {
                Err(Diagnostic::new(span, "unexpected scalar dependency"))
            })
            .map_err(|error| located(&graph, file, error))
        };
        nominals
            .define_records_with_specializations(&graph, &mut types, &mut records, &mut evaluate)
            .unwrap();
        let declaration = &graph.declarations()[0];
        let root = nominals.declarations[&declaration.id()];
        let source_record = records.record(root).unwrap();
        let mut scope = source_record.substitution.clone();
        let left = source_record.shape.fields[0].ty;
        let right = source_record.shape.fields[1].ty;
        assert_ne!(left, right);
        assert_eq!(records.record(left).unwrap().shape.name, None);
        assert_eq!(types.record_definition(left).unwrap().fields.len(), 2);
        assert!(types.record_definition(left).unwrap().layout.packed);
        let field = &records.record(left).unwrap().shape.fields[1];
        assert_eq!(types.field_type(field.id).unwrap(), field.ty);
        assert_eq!(
            records.reflected_field_notes(field.id).unwrap(),
            &[Box::<[u8]>::from(&b"Range(7)"[..])]
        );
        let syntax::FieldBinding::Explicit {
            ty: annotation, ..
        } = source_record.shape.fields[0]
            .syntax
            .named_binding()
            .unwrap()
        else {
            panic!()
        };
        let annotation = annotation.clone();
        scope.bind_type(graph.symbols().find("mode").unwrap(), types.meta_type());
        records.push_owner(root);
        let repeated = resolve_type(
            &graph,
            TypeRequest::new(declaration.file(), &annotation, declaration.location().span)
                .with_substitution(Some(&scope)),
            &mut types,
            &nominals,
            &mut records,
            &mut evaluate,
        )
        .unwrap();
        records.pop_owner(root);
        assert_eq!(repeated, left);
        let enum_ty = records.record(root).unwrap().shape.fields[2].ty;
        let enumeration = records.member_enum(enum_ty).unwrap();
        assert_eq!(enumeration.name, None);
        assert_eq!(enumeration.representation, IntegerType::U8);
        assert_eq!(
            enumeration
                .values
                .iter()
                .map(|(_, value)| value.value())
                .collect::<Vec<_>>(),
            [0, 1, 1]
        );
        types.freeze().unwrap();
    }

    #[test]
    fn anonymous_member_metadata_preserves_physical_fields_and_union_offsets() {
        let fixture = Fixture::new(
            "Root::struct { lead:u8; union { struct { x,y:u64; } struct { r,g:u64; } }; tail:u8; }",
        );
        let graph = fixture.graph();
        let mut types = TypeRegistry::new();
        let mut nominals = Nominals::reserve(&graph, &mut types).unwrap();
        let mut records = RecordSpecializations::default();
        let mut evaluate = |file, expression: &syntax::Expression| {
            jai_eval::evaluate_paths(expression, |_, span| {
                Err(Diagnostic::new(span, "unexpected scalar dependency"))
            })
            .map_err(|error| located(&graph, file, error))
        };
        nominals
            .define_records_with_specializations(&graph, &mut types, &mut records, &mut evaluate)
            .unwrap();
        let declaration = &graph.declarations()[0];
        let root = nominals.declarations[&declaration.id()];
        let root_shape = &records.record(root).unwrap().shape;
        assert_eq!(root_shape.fields.len(), 3);
        let embedded = &root_shape.fields[1];
        assert_eq!(embedded.name, None);
        assert!(embedded.syntax.using());
        assert!(matches!(
            embedded.syntax,
            crate::local_declarations::FieldSource::AnonymousRecord(_)
        ));
        assert_eq!(
            types.validate_field(root, embedded.id).unwrap(),
            embedded.ty
        );
        let union_shape = &records.record(embedded.ty).unwrap().shape;
        assert_eq!(union_shape.kind, jai_types::RecordKind::Union);
        assert_eq!(union_shape.fields.len(), 2);
        assert_ne!(union_shape.fields[0].ty, union_shape.fields[1].ty);
        assert!(
            union_shape
                .fields
                .iter()
                .all(|field| field.name.is_none() && field.syntax.using())
        );
        let mut metadata = jai_types::ReflectionMetadata::default();
        records.append_reflection_metadata(&mut metadata, graph.symbols());
        let jai_types::ReflectionReadiness::Ready(reflection) = jai_types::ReflectionGraph::build(
            &types,
            root,
            Some(jai_types::LayoutPolicy::lp64()),
            &metadata,
        )
        .unwrap() else {
            panic!("complete source shapes must be ready");
        };
        let jai_types::DescriptorKind::Record {
            fields, ..
        } = &reflection.get(reflection.root()).unwrap().kind
        else {
            panic!()
        };
        assert_eq!(
            fields
                .iter()
                .map(|field| field.offset_in_bytes)
                .collect::<Vec<_>>(),
            [0, 8, 24]
        );
        assert_eq!(fields[1].name, None);
        assert!(fields[1].using);
        let child = reflection.get(fields[1].ty).unwrap();
        assert_eq!(child.name, None);
        let jai_types::DescriptorKind::Record {
            fields, ..
        } = &child.kind
        else {
            panic!()
        };
        assert_eq!(
            fields
                .iter()
                .map(|field| field.offset_in_bytes)
                .collect::<Vec<_>>(),
            [0, 0]
        );
        assert!(
            fields
                .iter()
                .all(|field| field.name.is_none() && field.using)
        );
        assert!(graph.symbols().find("anonymous").is_none());
        types.freeze().unwrap();
    }
}
