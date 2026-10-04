//! Read annotation types from ready declaration facts without evaluating values.
use super::*;
use jai_types::TypeView;

const MAX_QUERY_FIELDS: usize = 65_536;
const MAX_QUERY_PATH: usize = 256;

#[derive(Clone, Copy)]
struct FieldQuery<'a> {
    graph: &'a ModuleGraph,
    types: &'a dyn TypeView,
    records: Option<&'a super::super::parameterized::RecordSpecializations>,
    span: Span,
}

pub(crate) fn annotation_query_path(
    value: &syntax::Expression,
) -> Result<syntax::NamePath, Diagnostic> {
    let mut current = value;
    let mut suffix = Vec::new();
    let mut path = loop {
        match &current.kind {
            syntax::ExpressionKind::Name(name) | syntax::ExpressionKind::CompileVariable(name) => {
                break syntax::NamePath {
                    root: *name,
                    members: Vec::new(),
                };
            }
            syntax::ExpressionKind::QualifiedName(path) => {
                if path.members.len().saturating_add(suffix.len()) > MAX_QUERY_PATH {
                    return Err(Diagnostic::new(
                        value.span,
                        "annotation type_of exceeds member path depth limit",
                    ));
                }
                break path.clone();
            }
            syntax::ExpressionKind::Member {
                base,
                member,
            } => {
                if suffix.len() >= MAX_QUERY_PATH {
                    return Err(Diagnostic::new(
                        value.span,
                        "annotation type_of exceeds member path depth limit",
                    ));
                }
                suffix.push(*member);
                current = base;
            }
            _ => {
                return Err(Diagnostic::new(
                    value.span,
                    "annotation type_of requires a declared name or record member; expression typing needs a ready semantic body",
                ));
            }
        }
    };
    if path.members.len().saturating_add(suffix.len()) > MAX_QUERY_PATH {
        return Err(Diagnostic::new(
            value.span,
            "annotation type_of exceeds member path depth limit",
        ));
    }
    path.members.extend(suffix.into_iter().rev());
    Ok(path)
}

impl Nominals<'_> {
    pub(crate) fn declared_field_type(
        &self,
        graph: &ModuleGraph,
        mut ty: TypeId,
        members: &[Symbol],
        types: &dyn TypeView,
        records: Option<&super::super::parameterized::RecordSpecializations>,
        span: Span,
    ) -> Result<TypeId, Diagnostic> {
        let mut remaining = MAX_QUERY_FIELDS;
        let query = FieldQuery {
            graph,
            types,
            records,
            span,
        };
        for &member in members {
            ty = self.annotation_member_type(query, ty, member, &mut remaining)?;
        }
        Ok(ty)
    }

    fn annotation_member_type(
        &self,
        query: FieldQuery<'_>,
        mut ty: TypeId,
        member: Symbol,
        remaining: &mut usize,
    ) -> Result<TypeId, Diagnostic> {
        let FieldQuery {
            graph,
            types,
            records,
            span,
        } = query;
        while let TypeKind::Pointer(pointee) = types
            .kind(ty)
            .map_err(|error| Diagnostic::new(span, error.to_string()))?
        {
            ty = *pointee;
            charge(remaining, 1, span)?;
        }
        if let Some(schema) = self.generated_reflection.get()
            && let Some(path) = schema.field_path(ty, graph.symbols().name(member))
        {
            charge(remaining, path.len(), span)?;
            let mut result = ty;
            for field in path {
                result = types
                    .field_type(field)
                    .map_err(|error| Diagnostic::new(span, error.to_string()))?;
            }
            return Ok(result);
        }
        let path = crate::record_default_overrides::optional_field_path(
            ty,
            member,
            span,
            types,
            |owner| {
                charge(remaining, 1, span)?;
                if let Some(record) = records.and_then(|records| records.record(owner)) {
                    charge(
                        remaining,
                        record
                            .shape
                            .fields
                            .len()
                            .saturating_add(record.shape.using.len()),
                        span,
                    )?;
                    return Ok(record.shape.clone());
                }
                let record = self.records.get(&owner).ok_or_else(|| {
                    Diagnostic::new(
                        span,
                        "annotation type_of is waiting for ready record declaration metadata",
                    )
                })?;
                charge(remaining, record.fields.len(), span)?;
                Ok(crate::local_declarations::RecordMetadata {
                    name: None,
                    kind: record.kind,
                    using: Vec::new(),
                    fields: record
                        .fields
                        .iter()
                        .map(|field| crate::local_declarations::FieldMetadata {
                            name: Some(field.name),
                            id: field.id,
                            ty: field.ty,
                            syntax: field.syntax.clone().into(),
                        })
                        .collect(),
                })
            },
        )?
        .ok_or_else(|| Diagnostic::new(span, "unknown record member in annotation type_of"))?;
        let mut result = ty;
        for field in path {
            result = types
                .validate_field(result, field)
                .map_err(|e| Diagnostic::new(span, e.to_string()))?;
        }
        Ok(result)
    }

    pub(super) fn annotation_type_of(
        &self,
        site: TypeSite<'_>,
        value: &syntax::Expression,
        types: &mut TypeRegistry,
        evaluate: &mut impl FnMut(
            FileInstanceId,
            &syntax::Expression,
        ) -> Result<ConstantValue, LocatedDiagnostic>,
        visiting: &mut HashSet<DeclarationId>,
    ) -> Result<TypeId, LocatedDiagnostic> {
        let path =
            annotation_query_path(value).map_err(|error| located(site.graph, site.file, error))?;
        let mut last_error = None;
        for count in (0..=path.members.len()).rev() {
            let prefix = syntax::NamePath {
                root: path.root,
                members: path.members[..count].to_vec(),
            };
            if let Some(ty) = self.value_type(site.graph, site.file, &prefix) {
                return self
                    .declared_field_type(
                        site.graph,
                        ty,
                        &path.members[count..],
                        types,
                        None,
                        value.span,
                    )
                    .map_err(|error| located(site.graph, site.file, error));
            }
            let syntax = if prefix.members.is_empty()
                && matches!(
                    site.graph.lookup(site.file, &prefix),
                    Err(jai_modules::LookupError::UnknownName(_))
                ) {
                BuiltinType::from_spelling(site.graph.symbols().name(prefix.root))
                    .map(TypeSyntax::Builtin)
                    .unwrap_or(TypeSyntax::Named(prefix))
            } else {
                TypeSyntax::Named(prefix)
            };
            match self.resolve_type_inner(site, &syntax, types, evaluate, visiting) {
                Ok(_) if count == path.members.len() => {
                    return self
                        .runtime_type_for_graph(site.graph, types)
                        .map_err(|error| {
                            located(
                                site.graph,
                                site.file,
                                Diagnostic::new(value.span, error.to_string()),
                            )
                        });
                }
                Ok(ty) => {
                    return self
                        .declared_field_type(
                            site.graph,
                            ty,
                            &path.members[count..],
                            types,
                            None,
                            value.span,
                        )
                        .map_err(|error| located(site.graph, site.file, error));
                }
                Err(error) => last_error = Some(error),
            }
        }
        Err(last_error.expect("a query path has at least its root"))
    }
}

fn charge(remaining: &mut usize, count: usize, span: Span) -> Result<(), Diagnostic> {
    *remaining = remaining.checked_sub(count).ok_or_else(|| {
        Diagnostic::new(span, "annotation type_of exceeds field traversal budget")
    })?;
    Ok(())
}
