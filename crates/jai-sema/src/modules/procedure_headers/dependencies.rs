//! Inferred callable headers depend on defining declaration identities.
use super::*;
use std::collections::HashSet;

pub(in crate::modules) fn ordered<'a>(
    graph: &'a ModuleGraph,
    callable_aliases: &HashMap<DeclarationId, Vec<DeclarationId>>,
) -> Result<Vec<&'a jai_modules::Declaration>, LocatedDiagnostic> {
    let mut pending = graph
        .declarations()
        .iter()
        .filter(|declaration| {
            matches!(
                declaration.syntax().kind,
                FileDeclarationKind::Procedure(_) | FileDeclarationKind::ProcedurePrototype(_)
            )
        })
        .map(|declaration| {
            (
                declaration,
                if super::identities::is_concrete(declaration) {
                    dependencies(graph, callable_aliases, declaration)
                        .into_iter()
                        .filter(|id| {
                            graph
                                .declaration(*id)
                                .is_some_and(super::identities::is_concrete)
                        })
                        .collect()
                } else {
                    HashSet::new()
                },
            )
        })
        .collect::<Vec<_>>();
    let mut ready = HashSet::new();
    let mut ordered = Vec::with_capacity(pending.len());
    while !pending.is_empty() {
        let Some(index) = pending
            .iter()
            .position(|(_, dependencies)| dependencies.iter().all(|id| ready.contains(id)))
        else {
            let declaration = pending[0].0;
            return Err(located(
                graph,
                declaration.file(),
                Diagnostic::new(
                    declaration.location().span,
                    "cyclic inferred procedure signature dependencies",
                ),
            ));
        };
        let (declaration, _) = pending.remove(index);
        ready.insert(declaration.id());
        ordered.push(declaration);
    }
    Ok(ordered)
}

pub(in crate::modules) fn dependencies(
    graph: &ModuleGraph,
    callable_aliases: &HashMap<DeclarationId, Vec<DeclarationId>>,
    declaration: &jai_modules::Declaration,
) -> HashSet<DeclarationId> {
    let (parameters, results) = match &declaration.syntax().kind {
        FileDeclarationKind::Procedure(procedure) => (&procedure.parameters, &procedure.results),
        FileDeclarationKind::ProcedurePrototype(prototype) => {
            (&prototype.parameters, &prototype.results)
        }
        _ => return HashSet::new(),
    };
    let mut pending = parameters
        .iter()
        .filter_map(|parameter| match &parameter.binding {
            syntax::ParameterBinding::Defaulted {
                ty: None,
                expression,
            }
            | syntax::ParameterBinding::DefaultedType {
                ty: None,
                expression,
            } => Some(expression),
            _ => None,
        })
        .chain(results.iter().filter_map(|result| match &result.binding {
            syntax::ResultBinding::InferredDefault(expression) => Some(expression),
            _ => None,
        }))
        .collect::<Vec<_>>();
    let mut dependencies = HashSet::new();
    while let Some(expression) = pending.pop() {
        use syntax::ExpressionKind as E;
        let name = match &expression.kind {
            E::Name(name) => Some(path(*name)),
            E::QualifiedName(path) => Some(path.clone()),
            E::Call(name, arguments) => {
                pending.extend(arguments.iter().map(|argument| &argument.value));
                Some(path(*name))
            }
            E::QualifiedCall(path, arguments) => {
                pending.extend(arguments.iter().map(|argument| &argument.value));
                Some(path.clone())
            }
            E::CallHint { call, .. }
            | E::AddressOf(call)
            | E::Dereference(call)
            | E::Member { base: call, .. }
            | E::Unary(_, call)
            | E::InferredCast { value: call, .. } => {
                pending.push(call);
                None
            }
            E::TypeQuery {
                query: syntax::TypeQueryKind::InitializerOf,
                value,
            } => {
                pending.push(value);
                None
            }
            E::IndirectCall { callee, args } => {
                pending.push(callee);
                pending.extend(args.iter().map(|argument| &argument.value));
                None
            }
            E::ContextCall {
                callee,
                args,
                overrides,
            } => {
                pending.push(callee);
                pending.extend(args.iter().chain(overrides).map(|argument| &argument.value));
                None
            }
            E::Binary(_, left, right) => {
                pending.extend([left.as_ref(), right.as_ref()]);
                None
            }
            E::Index { base, index } => {
                pending.extend([base.as_ref(), index.as_ref()]);
                None
            }
            E::Conditional(branch) => {
                pending.extend([branch.condition.as_ref(), branch.then_value.as_ref()]);
                pending.extend(branch.else_value.as_deref());
                None
            }
            E::ArrayLiteral(literal) => {
                if literal.element_type.is_none() {
                    pending.extend(literal.elements.first());
                }
                None
            }
            // Quoted bodies retain their capture and resolve when expanded.
            E::Code(_)
            | E::ShortLambda(_)
            | E::AnonymousProcedure(_)
            | E::CompileTime(_)
            | E::Insert(_)
            | E::Type(_)
            | E::Cast(..)
            | E::TypeCast { .. }
            | E::TypeQuery { .. }
            | E::StructLiteral(_)
            | E::PositionalStructLiteral(_)
            | E::Integer(_)
            | E::Float(_)
            | E::String(_)
            | E::HereString(_)
            | E::Character(_)
            | E::Null
            | E::CompileTimePredicate
            | E::CallerLocation
            | E::SourceLocation
            | E::SourceFile
            | E::SourceFilepath
            | E::SourceLine
            | E::Uninitialized
            | E::Bool(_)
            | E::Context
            | E::InferredMember(_)
            | E::CompileVariable(_) => None,
        };
        let Some(path) = name else { continue };
        let targets = match graph.lookup(declaration.file(), &path) {
            Ok(jai_modules::Binding::Declaration(id)) => callable_aliases
                .get(&id)
                .cloned()
                .unwrap_or_else(|| vec![id]),
            Ok(jai_modules::Binding::OverloadSet(id)) => graph
                .overload_set(id)
                .expect("graph overload identity exists")
                .declarations()
                .to_vec(),
            _ => continue,
        };
        dependencies.extend(targets.into_iter().filter(|id| {
            matches!(
                graph
                    .declaration(*id)
                    .map(|declaration| &declaration.syntax().kind),
                Some(
                    FileDeclarationKind::Procedure(_) | FileDeclarationKind::ProcedurePrototype(_)
                )
            )
        }));
    }
    dependencies
}
