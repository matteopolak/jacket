//! Named and anonymous enums share source identity and lexical constant lookup.
use super::*;

struct EnumSource<'a> {
    id: LocalDeclarationId,
    name: Option<Symbol>,
    representation: Option<&'a syntax::TypeSyntax>,
    kind: syntax::EnumKind,
    specified: bool,
    members: &'a [syntax::EnumMember],
    span: Span,
}

impl Resolver<'_> {
    pub(super) fn define_local_enum(
        &mut self,
        id: LocalDeclarationId,
        enumeration: &syntax::EnumDeclaration,
    ) -> Result<TypeId, Diagnostic> {
        self.define_local_enum_body(EnumSource {
            id,
            name: Some(enumeration.name),
            representation: enumeration.representation.as_ref(),
            kind: enumeration.kind,
            specified: enumeration.specified,
            members: &enumeration.members,
            span: enumeration.span,
        })
    }

    pub(crate) fn local_inline_enum(
        &mut self,
        enumeration: &syntax::EnumTypeSyntax,
    ) -> Result<TypeId, Diagnostic> {
        while self.local_scopes.frames.len() < self.scopes.len() {
            self.push_local_scope();
        }
        let id = LocalDeclarationId {
            scope: self.local_scopes.frames.last().unwrap().id,
            start: enumeration.span.start,
            end: enumeration.span.end,
            ordinal: usize::MAX,
        };
        self.meta.local_declarations.entries.entry(id).or_default();
        self.define_local_enum_body(EnumSource {
            id,
            name: None,
            representation: enumeration.representation.as_ref(),
            kind: enumeration.kind,
            specified: enumeration.specified,
            members: &enumeration.members,
            span: enumeration.span,
        })
    }

    fn define_local_enum_body(&mut self, source: EnumSource<'_>) -> Result<TypeId, Diagnostic> {
        let EnumSource {
            id,
            name,
            representation,
            kind,
            specified,
            members: source_members,
            span,
        } = source;
        let representation = match representation {
            None => IntegerType::S64,
            Some(syntax) => {
                let ty = self.lexical_annotation(syntax, span)?;
                let TypeKind::Integer(integer) = *self
                    .types
                    .kind(ty)
                    .map_err(|error| Diagnostic::new(span, error.to_string()))?
                else {
                    return Err(Diagnostic::new(
                        span,
                        "enum representation requires an integer type",
                    ));
                };
                integer
            }
        };
        let entry = self.meta.local_declarations.entries.get_mut(&id).unwrap();
        let ty = *entry
            .nominal
            .get_or_insert_with(|| self.types.reserve_enum(representation));
        if let Some(name) = name {
            self.meta.local_declarations.names.insert(ty, name);
        }
        self.remember_local_type_origin(id, name, ty, span);
        if self.meta.local_declarations.enumerations.contains_key(&ty) {
            return Ok(ty);
        }
        let mut members: HashMap<Symbol, Integer> = HashMap::new();
        let mut values = Vec::new();
        let mut next = Some(if kind == syntax::EnumKind::Flags {
            1i128
        } else {
            0i128
        });
        for member in source_members {
            if members.contains_key(&member.name) {
                return Err(Diagnostic::new(member.span, "duplicate enum member"));
            }
            if specified && member.initializer.is_none() {
                return Err(Diagnostic::new(
                    member.span,
                    "specified enum member requires an explicit value",
                ));
            }
            let value = match &member.initializer {
                Some(expression) => jai_eval::evaluate_paths(expression, |path, span| {
                    let own_member = if path.members.is_empty() {
                        Some(path.root)
                    } else if name == Some(path.root) && path.members.len() == 1 {
                        Some(path.members[0])
                    } else {
                        None
                    };
                    if let Some(value) = own_member.and_then(|name| members.get(&name)).copied() {
                        return Ok(ScalarConstant::Int(value));
                    }
                    self.local_scalar_path(path, span)
                })?,
                None => ScalarConstant::Literal(next.ok_or_else(|| {
                    Diagnostic::new(
                        member.span,
                        "enum automatic value overflow or unsupported flags progression",
                    )
                })?),
            }
            .coerce(ScalarType::Int(representation), member.span)?;
            let ScalarConstant::Int(value) = value else {
                unreachable!("integer enum representation was coerced");
            };
            next = match kind {
                syntax::EnumKind::Values => value.value().checked_add(1),
                syntax::EnumKind::Flags
                    if value.value() > 0 && (value.value() as u128).is_power_of_two() =>
                {
                    value.value().checked_mul(2)
                }
                syntax::EnumKind::Flags => None,
            };
            members.insert(member.name, value);
            values.push((member.name, value));
        }
        self.types
            .define_enum(
                ty,
                values.iter().map(|&(_, value)| value).collect::<Vec<_>>(),
            )
            .map_err(|error| Diagnostic::new(span, error.to_string()))?;
        self.meta.local_declarations.enumerations.insert(
            ty,
            EnumMetadata {
                name,
                flags: kind == syntax::EnumKind::Flags,
                members,
                values,
            },
        );
        Ok(ty)
    }
}
