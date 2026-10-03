//! Identify constant declarations that need the typed procedure readiness phase.
use super::*;
use std::collections::{HashSet, VecDeque};

pub(super) fn classify(graph: &ModuleGraph) -> HashSet<DeclarationId> {
    let mut deferred = classify_matching(
        graph,
        |expression| {
            matches!(
                expression.kind,
                syntax::ExpressionKind::CompileTime(_)
                    | syntax::ExpressionKind::Code(_)
                    | syntax::ExpressionKind::AnonymousProcedure(_)
                    | syntax::ExpressionKind::BakeArguments(_)
            )
        },
        false,
    );
    deferred.extend(classify_matching(
        graph,
        |expression| matches!(expression.kind, syntax::ExpressionKind::ShortLambda(_)),
        true,
    ));
    deferred
}

pub(super) fn code_constants(graph: &ModuleGraph) -> HashSet<DeclarationId> {
    classify_matching(
        graph,
        |expression| matches!(expression.kind, syntax::ExpressionKind::Code(_)),
        false,
    )
}

/// An unannotated lambda and its pure aliases retain source until a caller
/// supplies a procedure signature or argument types. Other dependent recipes
/// remain ordinary readiness jobs and can materialize with their own context.
pub(super) fn contextual_lambdas(graph: &ModuleGraph) -> HashSet<DeclarationId> {
    let mut contextual = HashSet::new();
    let mut aliases: HashMap<DeclarationId, Vec<DeclarationId>> = HashMap::new();
    for declaration in graph.declarations() {
        let FileDeclarationKind::Constant(constant) = &declaration.syntax().kind else {
            continue;
        };
        if constant.ty.is_some() {
            continue;
        }
        let path = match &constant.initializer.kind {
            syntax::ExpressionKind::ShortLambda(_) => {
                contextual.insert(declaration.id());
                None
            }
            syntax::ExpressionKind::Name(name) => Some(path(*name)),
            syntax::ExpressionKind::QualifiedName(path) => Some(path.clone()),
            _ => None,
        };
        if let Some(path) = path
            && let Ok(jai_modules::Binding::Declaration(dependency)) =
                graph.lookup(declaration.file(), &path)
        {
            aliases
                .entry(dependency)
                .or_default()
                .push(declaration.id());
        }
    }
    let mut queue: VecDeque<_> = contextual.iter().copied().collect();
    while let Some(dependency) = queue.pop_front() {
        for &alias in aliases.get(&dependency).into_iter().flatten() {
            if contextual.insert(alias) {
                queue.push_back(alias);
            }
        }
    }
    contextual
}

fn classify_matching(
    graph: &ModuleGraph,
    is_root: impl Fn(&syntax::Expression) -> bool,
    callable_dependencies: bool,
) -> HashSet<DeclarationId> {
    let mut deferred = HashSet::new();
    let mut users: HashMap<DeclarationId, Vec<DeclarationId>> = HashMap::new();
    for declaration in graph.declarations() {
        let FileDeclarationKind::Constant(constant) = &declaration.syntax().kind else {
            continue;
        };
        visit(&constant.initializer, |expression| {
            // Quotation and contextual lambda bodies retain source and are
            // deliberately not traversed until expansion/specialization.
            if is_root(expression) {
                deferred.insert(declaration.id());
            }
            let path = match &expression.kind {
                syntax::ExpressionKind::Name(name) => Some(path(*name)),
                syntax::ExpressionKind::QualifiedName(path) => Some(path.clone()),
                syntax::ExpressionKind::Call(name, _) if callable_dependencies => Some(path(*name)),
                syntax::ExpressionKind::QualifiedCall(path, _) if callable_dependencies => {
                    Some(path.clone())
                }
                _ => None,
            };
            if let Some(path) = path
                && let Ok(jai_modules::Binding::Declaration(dependency)) =
                    graph.lookup(declaration.file(), &path)
            {
                users.entry(dependency).or_default().push(declaration.id());
            }
        });
    }
    let mut queue: VecDeque<_> = deferred.iter().copied().collect();
    while let Some(dependency) = queue.pop_front() {
        for &user in users.get(&dependency).into_iter().flatten() {
            if deferred.insert(user) {
                queue.push_back(user);
            }
        }
    }
    deferred
}

pub(super) fn visit(
    expression: &syntax::Expression,
    mut callback: impl FnMut(&syntax::Expression),
) {
    let mut work = vec![expression];
    while let Some(expression) = work.pop() {
        callback(expression);
        use syntax::ExpressionKind as E;
        match &expression.kind {
            E::CompileTime(syntax::CompileTimeRun {
                body: syntax::CompileTimeBody::Expression(value),
                ..
            })
            | E::Unary(_, value)
            | E::Cast(_, _, value)
            | E::AddressOf(value)
            | E::Dereference(value)
            | E::Member {
                base: value, ..
            }
            | E::TypeCast {
                value, ..
            }
            | E::TypeQuery {
                value, ..
            } => work.push(value),
            E::Binary(_, lhs, rhs)
            | E::Index {
                base: lhs,
                index: rhs,
            } => {
                work.push(rhs);
                work.push(lhs);
            }
            E::Conditional(value) => {
                work.push(&value.condition);
                work.push(&value.then_value);
                if let Some(otherwise) = &value.else_value {
                    work.push(otherwise);
                }
            }
            E::CallHint {
                call, ..
            } => work.push(call),
            E::Call(_, arguments) | E::QualifiedCall(_, arguments) => {
                work.extend(arguments.iter().map(|argument| &argument.value))
            }
            E::BakeArguments(value) => {
                work.push(&value.callee);
                work.extend(value.arguments.iter().map(|argument| &argument.value));
            }
            E::IndirectCall {
                callee,
                args,
            } => {
                work.push(callee);
                work.extend(args.iter().map(|argument| &argument.value));
            }
            E::ArrayLiteral(value) => work.extend(&value.elements),
            E::StructLiteral(value) => work.extend(value.fields.iter().map(|field| &field.value)),
            E::PositionalStructLiteral(value) => work.extend(&value.values),
            _ => {}
        }
    }
}

impl FileScope<'_> {
    pub(crate) fn pending_constant(
        &self,
        path: &NamePath,
        deferred: &HashSet<DeclarationId>,
    ) -> Option<DeclarationId> {
        for count in (0..=path.members.len()).rev() {
            let prefix = NamePath {
                root: path.root,
                members: path.members[..count].to_vec(),
            };
            if let Ok(jai_modules::Binding::Declaration(id)) =
                self.declarations.graph.lookup(self.file, &prefix)
                && deferred.contains(&id)
                && !self.declarations.values.contains_key(&id)
            {
                return Some(id);
            }
        }
        None
    }
}
