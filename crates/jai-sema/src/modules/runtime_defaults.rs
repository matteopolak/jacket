//! Recognize storage paths in the declaring file; no mutable value is sampled.
use super::*;
use crate::runtime_defaults::{DefaultReadRoot, DefaultReadStep, RuntimeDefaultRead};
use std::collections::HashSet;

const MAX_PATH: usize = 256;
const MAX_FIELDS: usize = 65_536;

enum SourceRoot {
    Global(DeclarationId),
    Context,
}
struct ReadSource {
    root: SourceRoot,
    members: Vec<Symbol>,
    query: syntax::Expression,
}

fn source(
    graph: &ModuleGraph,
    file: FileInstanceId,
    expression: &syntax::Expression,
) -> Result<Option<ReadSource>, LocatedDiagnostic> {
    let mut current = expression;
    let mut suffix = Vec::new();
    let mut path = loop {
        match &current.kind {
            syntax::ExpressionKind::Member {
                base,
                member,
            } => {
                if suffix.len() >= MAX_PATH {
                    return Err(located(
                        graph,
                        file,
                        Diagnostic::new(
                            expression.span,
                            "runtime default exceeds storage path depth limit",
                        ),
                    ));
                }
                suffix.push(*member);
                current = base;
            }
            syntax::ExpressionKind::Context => {
                let Some(name) = graph.symbols().find("context") else {
                    return Ok(Some(ReadSource {
                        root: SourceRoot::Context,
                        members: suffix.into_iter().rev().collect(),
                        query: expression.clone(),
                    }));
                };
                break NamePath {
                    root: name,
                    members: Vec::new(),
                };
            }
            syntax::ExpressionKind::Name(name) => break path(*name),
            syntax::ExpressionKind::QualifiedName(path) => break path.clone(),
            _ => return Ok(None),
        }
    };
    if path.members.len().saturating_add(suffix.len()) > MAX_PATH {
        return Err(located(
            graph,
            file,
            Diagnostic::new(
                expression.span,
                "runtime default exceeds storage path depth limit",
            ),
        ));
    }
    path.members.extend(suffix.into_iter().rev());
    for count in (0..=path.members.len()).rev() {
        let prefix = NamePath {
            root: path.root,
            members: path.members[..count].to_vec(),
        };
        let (root, mut members) = match graph.lookup(file, &prefix) {
            Ok(jai_modules::Binding::Declaration(id))
                if matches!(
                    graph.declaration(id).unwrap().syntax().kind,
                    FileDeclarationKind::Global(_)
                ) =>
            {
                (SourceRoot::Global(id), Vec::new())
            }
            Ok(jai_modules::Binding::StorageMember(id)) => {
                let member = graph
                    .source_storage_member(id)
                    .expect("checked storage member identity");
                (SourceRoot::Global(member.owner()), member.path().to_vec())
            }
            _ => continue,
        };
        members.extend_from_slice(&path.members[count..]);
        if members.len() > MAX_PATH {
            return Err(located(
                graph,
                file,
                Diagnostic::new(
                    expression.span,
                    "runtime default exceeds storage path depth limit",
                ),
            ));
        }
        return Ok(Some(ReadSource {
            root,
            members,
            query: syntax::Expression {
                span: expression.span,
                kind: syntax::ExpressionKind::QualifiedName(path),
            },
        }));
    }
    let root = NamePath {
        root: path.root,
        members: Vec::new(),
    };
    if graph.symbols().name(path.root) == "context"
        && matches!(
            graph.lookup(file, &root),
            Err(jai_modules::LookupError::UnknownName(_))
        )
    {
        return Ok(Some(ReadSource {
            root: SourceRoot::Context,
            members: path.members,
            query: expression.clone(),
        }));
    }
    Ok(None)
}

/// A typed header prerequisite, recognized in the default's defining file.
/// Source bindings named `context` remain ordinary global storage reads.
pub(super) fn needs_context_schema(
    graph: &ModuleGraph,
    file: FileInstanceId,
    expression: &syntax::Expression,
    declarations: &ScopedDeclarations<'_>,
) -> Result<bool, LocatedDiagnostic> {
    if declarations.context.is_some() {
        return Ok(false);
    }
    Ok(source(graph, file, expression)?
        .is_some_and(|read| matches!(read.root, SourceRoot::Context)))
}

#[allow(clippy::too_many_arguments)]
pub(super) fn infer(
    graph: &ModuleGraph,
    file: FileInstanceId,
    expression: &syntax::Expression,
    types: &mut TypeRegistry,
    declarations: &ScopedDeclarations<'_>,
    constants: &mut Constants<'_>,
    meta: &mut crate::reflection::MetaContext,
) -> Result<Option<TypeId>, LocatedDiagnostic> {
    let Some(source) = source(graph, file, expression)? else {
        return Ok(None);
    };
    let ty = match source.root {
        SourceRoot::Global(_) => declarations.nominals.resolve_type_with_specializations(
            graph,
            aggregates::parameterized::TypeRequest::new(
                file,
                &syntax::TypeSyntax::TypeOf(Box::new(source.query)),
                expression.span,
            ),
            types,
            &mut meta.record_specializations,
            &mut |file, expression| constants.evaluate_lazy(file, expression),
        )?,
        SourceRoot::Context => {
            let schema = declarations.context.as_ref().ok_or_else(|| located(graph, file, Diagnostic::new(expression.span, "inferred runtime context default requires a ready Context schema; annotate the parameter type")))?;
            let (ty, _) = project(
                declarations,
                meta,
                types,
                schema.definition.record_type,
                &source.members,
                expression.span,
            )
            .map_err(|error| located(graph, file, error))?;
            ty
        }
    };
    Ok(Some(ty))
}

#[allow(clippy::too_many_arguments)]
pub(super) fn prepare(
    graph: &ModuleGraph,
    file: FileInstanceId,
    expression: &syntax::Expression,
    expected: Option<TypeId>,
    types: &mut TypeRegistry,
    declarations: &ScopedDeclarations<'_>,
    constants: &mut Constants<'_>,
    meta: &mut crate::reflection::MetaContext,
) -> Result<Option<RuntimeDefaultRead>, LocatedDiagnostic> {
    let Some(source) = source(graph, file, expression)? else {
        return Ok(None);
    };
    // This also prepares the genuine static record metadata needed by early headers.
    let inferred = infer(
        graph,
        file,
        expression,
        types,
        declarations,
        constants,
        meta,
    )?
    .expect("recognized source read");
    let root = match source.root {
        SourceRoot::Global(id) => {
            let declaration = graph.declaration(id).expect("checked global source");
            let annotation = match &declaration.syntax().kind {
                FileDeclarationKind::Global(global) => match global.declaration.source() {
                    syntax::Declaration::GroupMember {
                        ..
                    } => unreachable!("source() returns the original non-group declaration"),
                    syntax::Declaration::Explicit {
                        ty, ..
                    } => Some(syntax::TypeSyntax::Builtin(syntax::BuiltinType::Scalar(
                        *ty,
                    ))),
                    syntax::Declaration::UnresolvedExplicit {
                        ty, ..
                    }
                    | syntax::Declaration::External {
                        ty, ..
                    } => Some(ty.clone()),
                    syntax::Declaration::Inferred {
                        ..
                    } => None,
                },
                _ => unreachable!("source classifier checks global ownership"),
            };
            let ty = if let Some(Binding::Storage(storage)) = declarations.values.get(&id) {
                storage.place().ty()
            } else if let Some(annotation) = annotation {
                declarations.nominals.resolve_type_with_specializations(
                    graph,
                    aggregates::parameterized::TypeRequest::new(
                        declaration.file(),
                        &annotation,
                        declaration.location().span,
                    ),
                    types,
                    &mut meta.record_specializations,
                    &mut |file, expression| constants.evaluate_lazy(file, expression),
                )?
            } else {
                return Err(located(
                    graph,
                    file,
                    Diagnostic::new(
                        expression.span,
                        "runtime default requires the inferred global's ready canonical type",
                    ),
                ));
            };
            DefaultReadRoot::Global {
                declaration: id,
                ty,
            }
        }
        SourceRoot::Context => DefaultReadRoot::Context {
            ty: declarations
                .context
                .as_ref()
                .expect("inference checked context schema")
                .definition
                .record_type,
        },
    };
    let (actual, steps) = project(
        declarations,
        meta,
        types,
        root.ty(),
        &source.members,
        expression.span,
    )
    .map_err(|error| located(graph, file, error))?;
    if actual != inferred || expected.is_some_and(|expected| expected != actual) {
        return Err(located(
            graph,
            file,
            Diagnostic::new(
                expression.span,
                "runtime default differs from its declared parameter type",
            ),
        ));
    }
    RuntimeDefaultRead::checked(
        root,
        steps,
        actual,
        SourceSpan {
            source: graph.file(file).unwrap().source(),
            span: expression.span,
        },
        types,
    )
    .map(Some)
    .map_err(|error| located(graph, file, error))
}

fn project(
    declarations: &ScopedDeclarations<'_>,
    meta: &crate::reflection::MetaContext,
    types: &TypeRegistry,
    mut ty: TypeId,
    members: &[Symbol],
    span: Span,
) -> Result<(TypeId, Vec<DefaultReadStep>), Diagnostic> {
    let mut steps = Vec::new();
    let mut remaining = MAX_FIELDS;
    for member in members {
        while let TypeKind::Pointer(pointee) = *types
            .kind(ty)
            .map_err(|error| Diagnostic::new(span, error.to_string()))?
        {
            if steps.len() >= MAX_PATH {
                return Err(Diagnostic::new(
                    span,
                    "runtime default exceeds storage path depth limit",
                ));
            }
            steps.push(DefaultReadStep::Dereference(pointee));
            ty = pointee;
        }
        let mut pending = vec![(ty, Vec::new(), HashSet::new())];
        let mut found = None;
        while let Some((owner, prefix, mut ancestors)) = pending.pop() {
            if !ancestors.insert(owner) || ancestors.len() > MAX_PATH {
                return Err(Diagnostic::new(
                    span,
                    "runtime default has cyclic or excessively deep using fields",
                ));
            }
            let fields = if let Some(schema) = declarations
                .context
                .as_ref()
                .filter(|schema| schema.definition.record_type == owner)
            {
                schema
                    .record_metadata()
                    .fields
                    .into_iter()
                    .map(|field| (field.name, field.id, field.syntax.using()))
                    .collect::<Vec<_>>()
            } else if let Some(record) = meta.record_specializations.record(owner) {
                record
                    .shape
                    .fields
                    .iter()
                    .map(|field| (field.name, field.id, field.syntax.using()))
                    .collect()
            } else if let Some(record) = declarations.nominals.records.get(&owner) {
                record
                    .fields
                    .iter()
                    .map(|field| (Some(field.name), field.id, field.syntax.using))
                    .collect()
            } else {
                return Err(Diagnostic::new(
                    span,
                    "runtime default requires ready record field metadata",
                ));
            };
            remaining = remaining.checked_sub(1 + fields.len()).ok_or_else(|| {
                Diagnostic::new(span, "runtime default exceeds field traversal budget")
            })?;
            for (name, field, using) in fields {
                let child = types
                    .validate_field(owner, field)
                    .map_err(|error| Diagnostic::new(span, error.to_string()))?;
                let mut path = prefix.clone();
                path.push(field);
                if name == Some(*member) && found.replace((child, path.clone())).is_some() {
                    return Err(Diagnostic::new(
                        span,
                        "ambiguous promoted runtime default member",
                    ));
                }
                if using {
                    pending.push((child, path, ancestors.clone()));
                }
            }
        }
        let (child, path) =
            found.ok_or_else(|| Diagnostic::new(span, "unknown runtime default record member"))?;
        if steps.len().saturating_add(path.len()) > MAX_PATH {
            return Err(Diagnostic::new(
                span,
                "runtime default exceeds storage path depth limit",
            ));
        }
        steps.extend(path.into_iter().map(DefaultReadStep::Field));
        ty = child;
    }
    Ok((ty, steps))
}

impl FileScope<'_> {
    pub(crate) fn runtime_default_imported_storage(
        &self,
        binding: jai_modules::Binding,
        source: SourceSpan,
    ) -> Result<Option<(Place, Vec<Symbol>)>, Diagnostic> {
        match binding {
            jai_modules::Binding::Declaration(id)
                if self
                    .declarations
                    .graph
                    .declaration(id)
                    .is_some_and(|declaration| {
                        matches!(declaration.syntax().kind, FileDeclarationKind::Global(_))
                    }) =>
            {
                self.runtime_default_storage(id, source)
                    .map(|place| Some((place, Vec::new())))
            }
            jai_modules::Binding::StorageMember(id) => {
                let member = self
                    .declarations
                    .graph
                    .source_storage_member(id)
                    .ok_or_else(|| {
                        Diagnostic::at_source(
                            source,
                            "runtime default has an invalid imported storage member",
                        )
                    })?;
                self.runtime_default_storage(member.owner(), source)
                    .map(|place| Some((place, member.path().to_vec())))
            }
            _ => Ok(None),
        }
    }

    pub(crate) fn runtime_default_owner(
        &self,
        place: Place,
        span: Span,
    ) -> Result<DeclarationId, Diagnostic> {
        self.declarations
            .values
            .iter()
            .find_map(|(id, binding)| {
                if matches!(binding, Binding::Storage(storage) if storage.place() == place)
                    && self
                        .declarations
                        .graph
                        .declaration(*id)
                        .is_some_and(|declaration| {
                            matches!(declaration.syntax().kind, FileDeclarationKind::Global(_))
                        })
                {
                    Some(*id)
                } else {
                    None
                }
            })
            .ok_or_else(|| {
                Diagnostic::new(
                    span,
                    "runtime default storage has no retained source global owner",
                )
            })
    }

    pub(crate) fn runtime_default_storage(
        &self,
        id: DeclarationId,
        source: SourceSpan,
    ) -> Result<Place, Diagnostic> {
        if !self
            .declarations
            .graph
            .declaration(id)
            .is_some_and(|declaration| {
                matches!(declaration.syntax().kind, FileDeclarationKind::Global(_))
            })
        {
            return Err(Diagnostic::at_source(
                source,
                "runtime default global belongs to another source graph",
            ));
        }
        match self.declarations.values.get(&id) {
            Some(Binding::Storage(storage)) => Ok(storage.place()),
            _ => Err(Diagnostic::at_source(
                source,
                "runtime default global storage is not ready",
            )),
        }
    }
}
