//! Pure enum expressions retain nominal identity through aliases and defaults.
use super::*;
use jai_modules::Binding as GraphBinding;
use std::collections::{HashSet, VecDeque};
mod expressions;
use expressions::bind_one;
pub(super) use expressions::{evaluate_expression, is_pure_scalar, uses_enum};

fn referenced(graph: &ModuleGraph, file: FileInstanceId, path: &NamePath) -> Option<DeclarationId> {
    if graph.insertion_capture_value(file, path.root).is_some() {
        return None;
    }
    for count in (0..=path.members.len()).rev() {
        let prefix = NamePath {
            root: path.root,
            members: path.members[..count].to_vec(),
        };
        if let Ok(GraphBinding::Declaration(id)) = graph.lookup(file, &prefix) {
            return Some(id);
        }
    }
    None
}

fn captured_enum(graph: &ModuleGraph, file: FileInstanceId, path: &NamePath) -> bool {
    path.members.is_empty()
        && matches!(
            graph.insertion_capture_value(file, path.root),
            Some(jai_modules::SourceCaptureValue::Enumeration(_))
        )
}

pub(super) fn classify(graph: &ModuleGraph, nominals: &Nominals<'_>) -> HashSet<DeclarationId> {
    let mut selected = HashSet::new();
    let mut users: HashMap<DeclarationId, Vec<DeclarationId>> = HashMap::new();
    for declaration in graph.declarations() {
        let FileDeclarationKind::Constant(constant) = &declaration.syntax().kind else {
            continue;
        };
        deferred_constants::visit(&constant.initializer, |expression| {
            let path = match &expression.kind {
                syntax::ExpressionKind::Name(name) => path(*name),
                syntax::ExpressionKind::QualifiedName(path) => path.clone(),
                _ => return,
            };
            if captured_enum(graph, declaration.file(), &path)
                || super::target_values::is_target_path(graph, declaration.file(), &path)
                || nominals
                    .enum_member(graph, declaration.file(), &path, expression.span)
                    .ok()
                    .flatten()
                    .is_some()
                || matches!(graph.lookup(declaration.file(), &path), Ok(GraphBinding::Parameter(id))
                    if matches!(graph.parameter(id).map(|parameter| &parameter.value), Some(jai_modules::ParameterValue::Enumeration(_))))
            {
                selected.insert(declaration.id());
            }
            if let Some(dependency) = referenced(graph, declaration.file(), &path) {
                users.entry(dependency).or_default().push(declaration.id());
            }
        });
    }
    let mut queue: VecDeque<_> = selected.iter().copied().collect();
    while let Some(dependency) = queue.pop_front() {
        for &user in users.get(&dependency).into_iter().flatten() {
            if selected.insert(user) {
                queue.push_back(user);
            }
        }
    }
    selected
}

pub(super) fn bind(
    declarations: &mut ScopedDeclarations<'_>,
    types: &mut TypeRegistry,
    meta: &mut crate::reflection::MetaContext,
    selected: &HashSet<DeclarationId>,
    deferred: &HashSet<DeclarationId>,
    options: &crate::ResolveOptions,
) -> Result<(), LocatedDiagnostic> {
    let graph = declarations.graph;
    let selected: HashSet<_> = selected
        .iter()
        .copied()
        .filter(|id| {
            !deferred.contains(id)
                && !declarations.values.contains_key(id)
                && !declarations.nominals.is_type_alias(graph, *id)
                && !declarations.callable_aliases.contains_key(id)
                && !sequence_constants::is_sequence_constant(graph, graph.declaration(*id).unwrap())
        })
        .collect();
    let mut dependencies = HashMap::new();
    let mut users: HashMap<DeclarationId, Vec<DeclarationId>> = HashMap::new();
    let mut queue = VecDeque::new();
    for declaration in graph
        .declarations()
        .iter()
        .filter(|declaration| selected.contains(&declaration.id()))
    {
        let FileDeclarationKind::Constant(constant) = &declaration.syntax().kind else {
            continue;
        };
        let mut needed = HashSet::new();
        deferred_constants::visit(&constant.initializer, |expression| {
            let path = match &expression.kind {
                syntax::ExpressionKind::Name(name) => path(*name),
                syntax::ExpressionKind::QualifiedName(path) => path.clone(),
                _ => return,
            };
            if let Some(id) = referenced(graph, declaration.file(), &path)
                && selected.contains(&id)
            {
                needed.insert(id);
            }
        });
        for id in &needed {
            users.entry(*id).or_default().push(declaration.id());
        }
        dependencies.insert(declaration.id(), needed.len());
        if needed.is_empty() {
            queue.push_back(declaration.id());
        }
    }
    let mut complete = 0;
    while let Some(id) = queue.pop_front() {
        let declaration = graph
            .declaration(id)
            .expect("enum constant declaration exists");
        let FileDeclarationKind::Constant(constant) = &declaration.syntax().kind else {
            unreachable!()
        };
        let file = declaration.file();
        let result = bind_one(declarations, types, meta, file, constant, options)
            .map_err(|error| located(graph, file, error))?;
        declarations.values.insert(id, result);
        complete += 1;
        for user in users.get(&id).into_iter().flatten() {
            let remaining = dependencies
                .get_mut(user)
                .expect("selected dependency exists");
            *remaining -= 1;
            if *remaining == 0 {
                queue.push_back(*user);
            }
        }
    }
    if complete != selected.len() {
        let id = dependencies
            .iter()
            .find(|(_, count)| **count != 0)
            .unwrap()
            .0;
        return Err(LocatedDiagnostic {
            location: graph.declaration(*id).unwrap().location(),
            message: "cyclic nominal constant dependencies".into(),
        });
    }
    Ok(())
}

pub(super) fn seed_defaults(
    defaults: &mut aggregates::Defaults<'_, '_>,
    values: &HashMap<DeclarationId, Binding>,
    meta: &crate::reflection::MetaContext,
) {
    for (id, binding) in values {
        let value = match binding {
            Binding::Enum(value) => Some(jai_ir::ConstantValue {
                ty: value.ty,
                kind: jai_ir::ConstantKind::Enum(value.value),
            }),
            Binding::TypedConstant(id) => meta.constant(*id).cloned(),

            _ => None,
        };
        if let Some(value) = value {
            defaults.named.insert(*id, value);
        }
    }
}
