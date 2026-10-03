//! Nested member nominals are reserved from their owning source declaration.
use super::*;
use jai_types::{Integer, IntegerType, TypeKind};
use state::MemberEnum;

pub(crate) fn reserve_static_members(
    graph: &ModuleGraph,
    nominals: &Nominals<'_>,
    types: &mut TypeRegistry,
    records: &mut RecordSpecializations,
    evaluate: &mut impl FnMut(
        FileInstanceId,
        &syntax::Expression,
    ) -> Result<ScalarConstant, LocatedDiagnostic>,
) -> Result<(), LocatedDiagnostic> {
    let mut resolver = TypeResolver {
        graph,
        types,
        nominals,
        records,
        evaluate,
        scalar_pending: None,
        aliases: HashSet::new(),
        lexical: None,
        lexical_active: false,
        nominal_context: NominalAnnotationContext::None,
    };
    for declaration in graph.declarations() {
        if let syntax::FileDeclarationKind::Record(record) = &declaration.syntax().kind
            && record.parameters.is_empty()
        {
            let ty = nominals.declarations[&declaration.id()];
            resolver
                .reserve_member_names(ty, declaration.file(), record, &Substitution::default())
                .map_err(|error| error.into_diagnostic(graph))?;
        }
    }
    Ok(())
}

pub(crate) fn materialize_static_record(
    graph: &ModuleGraph,
    declaration: DeclarationId,
    types: &mut TypeRegistry,
    nominals: &Nominals<'_>,
    records: &mut RecordSpecializations,
    evaluate: &mut impl FnMut(
        FileInstanceId,
        &syntax::Expression,
    ) -> Result<ScalarConstant, LocatedDiagnostic>,
) -> Result<(), LocatedDiagnostic> {
    prepare_static_record(graph, declaration, types, nominals, records, evaluate)
        .map_err(|error| error.into_diagnostic(graph))
}

pub(super) fn prepare_static_record(
    graph: &ModuleGraph,
    declaration: DeclarationId,
    types: &mut TypeRegistry,
    nominals: &Nominals<'_>,
    records: &mut RecordSpecializations,
    evaluate: &mut impl FnMut(
        FileInstanceId,
        &syntax::Expression,
    ) -> Result<ScalarConstant, LocatedDiagnostic>,
) -> TypeResult<()> {
    let source = graph
        .declaration(declaration)
        .expect("static source record exists");
    let syntax::FileDeclarationKind::Record(record) = &source.syntax().kind else {
        unreachable!("static record materialization uses source record");
    };
    let ty = nominals.declarations[&declaration];
    if let Some(existing) = records.record(ty) {
        if existing.origin != Some(RecordTemplateId(declaration)) {
            return Err(failure(
                graph,
                source.file(),
                Diagnostic::new(record.span, "record metadata has a different source owner"),
            ));
        }
        types.record_definition(ty).map_err(|error| {
            failure(
                graph,
                source.file(),
                Diagnostic::new(record.span, error.to_string()),
            )
        })?;
        return Ok(());
    }
    let mut resolver = TypeResolver {
        graph,
        types,
        nominals,
        records,
        evaluate,
        scalar_pending: None,
        aliases: HashSet::new(),
        lexical: None,
        lexical_active: false,
        nominal_context: NominalAnnotationContext::None,
    };
    let (shape, substitution) = resolver.materialize(
        ty,
        RecordTemplateId(declaration),
        source.file(),
        record,
        &Substitution::default(),
    )?;
    resolver.records.complete_nested(
        ty,
        SpecializedRecord {
            origin: Some(RecordTemplateId(declaration)),
            file: source.file(),
            shape,
            substitution,
            nested: true,
            defaults: HashMap::new(),
        },
    );
    Ok(())
}

impl<F> TypeResolver<'_, '_, F>
where
    F: FnMut(FileInstanceId, &syntax::Expression) -> Result<ScalarConstant, LocatedDiagnostic>,
{
    pub(super) fn reserve_member_names(
        &mut self,
        owner: TypeId,
        file: FileInstanceId,
        record: &syntax::RecordDeclaration,
        outer: &Substitution,
    ) -> TypeResult<Substitution> {
        self.reserve_body_member_names(owner, file, record.into(), outer)
    }
    pub(super) fn reserve_body_member_names(
        &mut self,
        owner: TypeId,
        file: FileInstanceId,
        record: RecordBody<'_>,
        outer: &Substitution,
    ) -> TypeResult<Substitution> {
        let members = self.checked_members(file, record.members, outer)?;
        let record = RecordBody {
            members: members.as_ref(),
            ..record
        };
        let mut scope = outer.clone();
        if let Some(name) = record.name
            && scope.ty(name).is_none()
            && scope.constant(name).is_none()
        {
            scope.bind_constant(name, BakedValue::Type(owner));
        }
        for (index, member) in record.members.iter().enumerate() {
            let (name, ty) = match member {
                syntax::RecordMember::Record(nested) if nested.parameters.is_empty() => (
                    nested.name,
                    self.records
                        .reserve_nested(owner, index, nested.kind, self.types),
                ),
                syntax::RecordMember::Enum(enumeration) => {
                    let representation = self.enum_representation(file, enumeration, &scope)?;
                    (
                        enumeration.name,
                        self.records
                            .reserve_nested_enum(owner, index, representation, self.types),
                    )
                }
                _ => continue,
            };
            scope.constants.retain(|binding| binding.name != name);
            scope.types.retain(|binding| binding.name != name);
            scope.bind_constant(name, BakedValue::Type(ty));
        }
        self.records.reserve_namespace(owner, scope.clone());
        let methods = record
            .members
            .iter()
            .enumerate()
            .filter_map(|(member, source)| {
                let source = match source {
                    syntax::RecordMember::Procedure(source) => {
                        RecordMethodSource::Procedure(source.as_ref().clone())
                    }
                    syntax::RecordMember::ProcedurePrototype(source) => {
                        RecordMethodSource::Prototype(source.clone())
                    }
                    _ => return None,
                };
                Some(RecordMethod {
                    id: RecordMethodId {
                        owner,
                        member,
                    },
                    file,
                    source,
                })
            })
            .collect();
        self.records.define_methods(owner, methods);
        self.records
            .reserve_method_environment(owner, file, record.name, scope.clone());
        for (index, member) in record.members.iter().enumerate() {
            if let syntax::RecordMember::Record(nested) = member
                && nested.parameters.is_empty()
            {
                let ty = self
                    .records
                    .reserve_nested(owner, index, nested.kind, self.types);
                self.reserve_member_names(ty, file, nested, &scope)?;
            }
        }
        Ok(scope)
    }
    fn enum_representation(
        &mut self,
        file: FileInstanceId,
        enumeration: &syntax::EnumDeclaration,
        scope: &Substitution,
    ) -> TypeResult<IntegerType> {
        let Some(annotation) = &enumeration.representation else {
            return Ok(IntegerType::S64);
        };
        let ty = self.resolve(file, annotation, Some(scope), enumeration.span)?;
        match self
            .types
            .kind(ty)
            .expect("resolved member enum representation belongs to registry")
        {
            TypeKind::Integer(integer) => Ok(*integer),
            _ => Err(failure(
                self.graph,
                file,
                Diagnostic::new(
                    enumeration.span,
                    "enum representation requires an integer type",
                ),
            )),
        }
    }
    pub(super) fn define_nested_enum(
        &mut self,
        owner: TypeId,
        index: usize,
        file: FileInstanceId,
        enumeration: &syntax::EnumDeclaration,
        scope: &Substitution,
    ) -> TypeResult<()> {
        let representation = self.enum_representation(file, enumeration, scope)?;
        let ty = self
            .records
            .reserve_nested_enum(owner, index, representation, self.types);
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
        let mut enum_scope = scope.clone();
        for member in &enumeration.members {
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
                Some(expression) => self.scalar(file, expression, Some(&enum_scope))?,
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
                unreachable!("member enum representation is an integer");
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
            enum_scope
                .constants
                .retain(|binding| binding.name != member.name);
            enum_scope.bind_constant(member.name, BakedValue::integer(value, self.types));
            values.push((member.name, value));
        }
        self.types
            .define_enum(
                ty,
                values
                    .iter()
                    .map(|(_, value)| *value)
                    .collect::<Vec<Integer>>(),
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
                name: Some(enumeration.name),
                representation,
                flags,
                values,
            },
        );
        Ok(())
    }
}
