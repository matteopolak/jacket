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
                if let Some(value) = value.explicit_then() {
                    work.push(value);
                }
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
            E::StructLiteral(value) => {
                if let Some(ty) = &value.ty {
                    literal_type_dependencies(ty, &mut work);
                }
                for field in &value.fields {
                    literal_target_dependencies(&field.target, &mut work);
                    work.push(&field.value);
                }
            }
            E::PositionalStructLiteral(value) => {
                if let Some(ty) = &value.ty {
                    literal_type_dependencies(ty, &mut work);
                }
                work.extend(&value.values);
            }
            _ => {}
        }
    }
}

// This readiness classifier borrows source operands; field names are relative
// selectors, while their indices and type application arguments are lexical.
fn literal_target_dependencies<'a>(
    target: &'a syntax::PlaceSyntax,
    work: &mut Vec<&'a syntax::Expression>,
) {
    let mut expressions = Vec::new();
    match &target.kind {
        syntax::PlaceKind::Member {
            base, ..
        } => expressions.push(base.as_ref()),
        syntax::PlaceKind::Index {
            base,
            index,
        } => {
            expressions.push(base.as_ref());
            work.push(index);
        }
        _ => {} // Invalid runtime roots are rejected by the checked literal target binder.
    }
    while let Some(source) = expressions.pop() {
        match &source.kind {
            syntax::ExpressionKind::Member {
                base, ..
            } => expressions.push(base.as_ref()),
            syntax::ExpressionKind::Index {
                base,
                index,
            } => {
                expressions.push(base.as_ref());
                work.push(index);
            }
            _ => {}
        }
    }
}
fn literal_type_dependencies<'a>(
    source: &'a syntax::TypeSyntax,
    work: &mut Vec<&'a syntax::Expression>,
) {
    let mut types = vec![source];
    while let Some(source) = types.pop() {
        use syntax::TypeSyntax as T;
        match source {
            T::TypeOf(value) => work.push(value),
            T::Pointer(inner)
            | T::Slice(inner)
            | T::DynamicArray(inner)
            | T::Variant {
                base: inner, ..
            } => types.push(inner),
            T::FixedArray {
                count,
                element,
            } => {
                work.push(count);
                types.push(element);
            }
            T::Application(value) => {
                types.push(&value.base);
                work.extend(value.arguments.iter().map(|argument| &argument.value));
            }
            T::Procedure(value) => {
                types.extend(value.parameters.iter().map(|parameter| &parameter.ty));
                types.extend(value.results.iter().map(|result| &result.ty));
            }
            T::This
            | T::Builtin(_)
            | T::Named(_)
            | T::Variable(_)
            | T::Restricted {
                ..
            }
            | T::InlineRecord(_)
            | T::InlineEnum(_) => {} // Nominal bodies have their own source readiness jobs.
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
