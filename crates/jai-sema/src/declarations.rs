//! Resolve declarative constants before runtime statements and allocate globals.
use super::{Binding, Diagnostic, Global, HashMap, ScalarConstant, Span, Symbol, syntax};
use super::{GlobalInitializer, TypeRegistry};

#[derive(Clone)]
enum State {
    Visiting,
    Ready(ScalarConstant),
}
enum Work<'a> {
    Enter(Symbol, Span),
    Finish(&'a syntax::ConstantDeclaration),
}
struct Constants<'a> {
    declarations: HashMap<Symbol, &'a syntax::ConstantDeclaration>,
    states: HashMap<Symbol, State>,
    outer: &'a HashMap<Symbol, Binding>,
    graph_scope: Option<super::modules::FileScope<'a>>,
}
impl<'a> Constants<'a> {
    fn new(
        declarations: &[&'a syntax::ConstantDeclaration],
        outer: &'a HashMap<Symbol, Binding>,
        graph_scope: Option<super::modules::FileScope<'a>>,
    ) -> Result<Self, Diagnostic> {
        let mut names = HashMap::new();
        for declaration in declarations {
            if names.insert(declaration.name, *declaration).is_some() {
                return Err(Diagnostic::new(
                    declaration.span,
                    "duplicate constant declaration",
                ));
            }
        }
        Ok(Self {
            declarations: names,
            states: HashMap::new(),
            outer,
            graph_scope,
        })
    }
    fn ready_value(&self, name: Symbol, span: Span) -> Result<ScalarConstant, Diagnostic> {
        match self.states.get(&name) {
            Some(State::Ready(value)) => Ok(value.clone()),
            Some(State::Visiting) => Err(Diagnostic::new(span, "cyclic constant dependencies")),
            None => match self.outer.get(&name) {
                Some(Binding::Constant(value)) => Ok(value.clone()),
                Some(Binding::Enum(_)) => Err(Diagnostic::new(
                    span,
                    "nominal enum block constant requires typed constant binding",
                )),
                Some(
                    Binding::Storage(_)
                    | Binding::CompilerInput {
                        ..
                    },
                ) => Err(Diagnostic::new(
                    span,
                    "mutable storage cannot supply a compile-time constant",
                )),
                Some(Binding::Discarded(_)) => {
                    Err(Diagnostic::new(span, "#discard parameter cannot be read"))
                }
                Some(Binding::Namespace(_) | Binding::Imported(_))
                | Some(Binding::Library(_) | Binding::Macro(_))
                | Some(Binding::Procedure {
                    ..
                })
                | Some(Binding::Type(_))
                | Some(Binding::LambdaPreview(_))
                | Some(Binding::TypedConstant(_) | Binding::Code(_)) => Err(Diagnostic::new(
                    span,
                    "typed value requires semantic constant binding",
                )),
                None => match self.graph_scope {
                    Some(scope) => match scope.value(
                        &syntax::NamePath {
                            root: name,
                            members: Vec::new(),
                        },
                        span,
                    )? {
                        Binding::Discarded(_) => {
                            Err(Diagnostic::new(span, "#discard parameter cannot be read"))
                        }
                        Binding::Constant(value) => Ok(value),
                        Binding::Enum(_) => Err(Diagnostic::new(
                            span,
                            "nominal enum block constant requires typed constant binding",
                        )),
                        Binding::Storage(_)
                        | Binding::CompilerInput {
                            ..
                        } => Err(Diagnostic::new(
                            span,
                            "mutable storage cannot supply a compile-time constant",
                        )),
                        Binding::Namespace(_)
                        | Binding::Imported(_)
                        | Binding::Library(_)
                        | Binding::Macro(_)
                        | Binding::Procedure {
                            ..
                        }
                        | Binding::Type(_)
                        | Binding::LambdaPreview(_)
                        | Binding::TypedConstant(_)
                        | Binding::Code(_) => Err(Diagnostic::new(
                            span,
                            "typed value requires semantic constant binding",
                        )),
                    },
                    None => Err(Diagnostic::new(span, "unknown constant")),
                },
            },
        }
    }
    fn ready_path(
        &self,
        path: &syntax::NamePath,
        span: Span,
    ) -> Result<ScalarConstant, Diagnostic> {
        if path.members.is_empty() {
            return self.ready_value(path.root, span);
        }
        if self.declarations.contains_key(&path.root) || self.outer.contains_key(&path.root) {
            return Err(Diagnostic::new(span, "scalar value is not a namespace"));
        }
        let scope = self
            .graph_scope
            .ok_or_else(|| Diagnostic::new(span, "qualified constant requires a module scope"))?;
        match scope.value(path, span)? {
            Binding::Discarded(_) => {
                Err(Diagnostic::new(span, "#discard parameter cannot be read"))
            }
            Binding::Constant(value) => Ok(value),
            Binding::Enum(_) => Err(Diagnostic::new(
                span,
                "nominal enum block constant requires typed constant binding",
            )),
            Binding::Storage(_)
            | Binding::CompilerInput {
                ..
            } => Err(Diagnostic::new(
                span,
                "mutable storage cannot supply a compile-time constant",
            )),
            Binding::Namespace(_)
            | Binding::Imported(_)
            | Binding::Library(_)
            | Binding::Macro(_)
            | Binding::Procedure {
                ..
            }
            | Binding::Type(_)
            | Binding::LambdaPreview(_)
            | Binding::TypedConstant(_)
            | Binding::Code(_) => Err(Diagnostic::new(
                span,
                "typed value requires semantic constant binding",
            )),
        }
    }
    fn value(&mut self, name: Symbol, span: Span) -> Result<ScalarConstant, Diagnostic> {
        let mut work = vec![Work::Enter(name, span)];
        while let Some(step) = work.pop() {
            match step {
                Work::Enter(name, span) => {
                    match self.states.get(&name) {
                        Some(State::Ready(_)) => continue,
                        Some(State::Visiting) => {
                            return Err(Diagnostic::new(span, "cyclic constant dependencies"));
                        }
                        None => {}
                    }
                    let Some(declaration) = self.declarations.get(&name).copied() else {
                        self.ready_value(name, span)?;
                        continue;
                    };
                    self.states.insert(name, State::Visiting);
                    work.push(Work::Finish(declaration));
                    let mut expressions = vec![&declaration.initializer];
                    let mut dependencies = Vec::new();
                    while let Some(expression) = expressions.pop() {
                        match &expression.kind {
                            syntax::ExpressionKind::Name(name) => {
                                dependencies.push((*name, expression.span))
                            }
                            syntax::ExpressionKind::Unary(_, e)
                            | syntax::ExpressionKind::Cast(_, _, e) => expressions.push(e),
                            syntax::ExpressionKind::Binary(_, lhs, rhs) => {
                                expressions.push(rhs);
                                expressions.push(lhs);
                            }
                            syntax::ExpressionKind::Conditional(e) => {
                                if let Some(otherwise) = &e.else_value {
                                    expressions.push(otherwise);
                                }
                                expressions.push(&e.then_value);
                                expressions.push(&e.condition);
                            }
                            syntax::ExpressionKind::Integer(_)
                            | syntax::ExpressionKind::Bool(_)
                            | syntax::ExpressionKind::QualifiedName(_)
                            | syntax::ExpressionKind::QualifiedCall(_, _)
                            | syntax::ExpressionKind::Call(_, _) => {}
                            syntax::ExpressionKind::StructLiteral(_)
                            | syntax::ExpressionKind::Member {
                                ..
                            } => {}
                            _ => {}
                        }
                    }
                    work.extend(
                        dependencies
                            .into_iter()
                            .rev()
                            .map(|(name, span)| Work::Enter(name, span)),
                    );
                }
                Work::Finish(declaration) => {
                    let mut value =
                        jai_eval::evaluate_paths(&declaration.initializer, |path, span| {
                            self.ready_path(path, span)
                        })?;
                    if let Some(annotation) = &declaration.ty {
                        let ty = annotation.as_scalar().ok_or_else(|| {
                            Diagnostic::new(
                                declaration.span,
                                "typed constant annotation requires a scoped semantic session",
                            )
                        })?;
                        value = value.coerce(ty, declaration.span)?;
                    }
                    self.states.insert(declaration.name, State::Ready(value));
                }
            }
        }
        self.ready_value(name, span)
    }
}
pub(super) fn check_top_level_names(module: &syntax::Module) -> Result<(), Diagnostic> {
    let mut names = HashMap::new();
    let items = module
        .procedures()
        .iter()
        .map(|p| (p.name, p.span))
        .chain(module.constants().iter().map(|c| (c.name, c.span)))
        .chain(
            module
                .globals()
                .iter()
                .map(|g| (g.declaration.name(), g.span)),
        );
    for (name, span) in items {
        if names.insert(name, span).is_some() {
            return Err(Diagnostic::new(
                span,
                format!(
                    "duplicate top-level declaration '{}'",
                    module.symbols().name(name)
                ),
            ));
        }
    }
    Ok(())
}
pub(super) fn resolve_globals(
    module: &syntax::Module,
    types: &TypeRegistry,
    alignments: &mut jai_ir::StorageAlignments,
) -> Result<(Vec<Global>, HashMap<Symbol, Binding>), Diagnostic> {
    let declarations: Vec<_> = module.constants().iter().collect();
    let empty = HashMap::new();
    let mut constants = Constants::new(&declarations, &empty, None)?;
    let mut bindings = HashMap::new();
    for declaration in &declarations {
        let value = constants.value(declaration.name, declaration.span)?;
        bindings.insert(declaration.name, Binding::Constant(value));
    }
    let mut globals = Vec::new();
    let mut shared_values = HashMap::new();
    for global in module.globals() {
        let mut alignment = None;
        for attribute in global.declaration.attributes() {
            match attribute {
                syntax::DeclarationAttribute::Alignment(expression) => {
                    alignment = Some(super::storage_alignment::constant(
                        jai_eval::evaluate(expression, |name, span| constants.value(name, span))?,
                        expression.span,
                    )?);
                }
            }
        }
        let selected_initializer = match &global.declaration {
            syntax::Declaration::GroupMember {
                group,
                ordinal,
                ..
            } => group.initializer_for(*ordinal),
            _ => None,
        };
        let copy_from = match &global.declaration {
            syntax::Declaration::GroupMember {
                group,
                ordinal,
                ..
            } if *ordinal > 0 && group.extra_initializers().is_empty() => {
                shared_values.get(&std::sync::Arc::as_ptr(group)).cloned()
            }
            _ => None,
        };
        let initializer = if let Some(initializer) = copy_from {
            initializer
        } else {
            let (_, value) = match global.declaration.source() {
                syntax::Declaration::GroupMember {
                    ..
                } => unreachable!("group source is a single declaration"),
                syntax::Declaration::Inferred {
                    name,
                    initializer,
                    ..
                } => (
                    *name,
                    jai_eval::evaluate(
                        selected_initializer.unwrap_or(initializer),
                        |name, span| constants.value(name, span),
                    )?,
                ),
                syntax::Declaration::Explicit {
                    name,
                    ty,
                    initializer,
                    ..
                } => {
                    let value = match selected_initializer.or(initializer.as_ref()) {
                        Some(e) => jai_eval::evaluate(e, |name, span| constants.value(name, span))?,
                        None => ScalarConstant::zero(*ty),
                    };
                    let value = value.coerce(*ty, global.span)?;
                    (*name, value)
                }
                syntax::Declaration::External {
                    ..
                } => {
                    return Err(Diagnostic::new(
                        global.span,
                        "external data requires source graph resolution",
                    ));
                }
                syntax::Declaration::UnresolvedExplicit {
                    ..
                } => {
                    return Err(Diagnostic::new(
                        global.span,
                        "global aggregate types are not implemented",
                    ));
                }
            };
            let scalar = value.scalar_type().ok_or_else(|| {
                Diagnostic::new(global.span, "float globals require graph type resolution")
            })?;
            let value = value.coerce(scalar, global.span)?;
            match value {
                ScalarConstant::Int(n) => GlobalInitializer::Int(n),
                ScalarConstant::Bool(b) => GlobalInitializer::Bool(b),
                ScalarConstant::Float(_) | ScalarConstant::WeakFloat(_) => {
                    return Err(Diagnostic::new(
                        global.span,
                        "float globals require graph type resolution",
                    ));
                }
                ScalarConstant::Literal(_) => {
                    unreachable!("global value is coerced before allocation")
                }
            }
        };
        let name = global.declaration.name();
        if let syntax::Declaration::GroupMember {
            group,
            ordinal,
            ..
        } = &global.declaration
            && *ordinal == 0
            && group.extra_initializers().is_empty()
        {
            shared_values.insert(std::sync::Arc::as_ptr(group), initializer.clone());
        }
        let span = global.span;
        let global = Global::new(globals.len(), initializer, types);
        if let Some(alignment) = alignment {
            alignments
                .set_global(global.id(), alignment)
                .map_err(|error| Diagnostic::new(span, error.to_string()))?;
        }
        let storage = global.storage();
        globals.push(global);
        bindings.insert(name, Binding::Storage(storage));
    }
    Ok((globals, bindings))
}
