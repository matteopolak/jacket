//! Resolve declarative constants before runtime statements and allocate globals.
use super::{Binding, ConstantValue, Diagnostic, Global, HashMap, Resolver, Span, Symbol, syntax};
use super::{GlobalInitializer, TypeRegistry};

#[derive(Clone, Copy)]
enum State {
    Visiting,
    Ready(ConstantValue),
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
    fn ready_value(&self, name: Symbol, span: Span) -> Result<ConstantValue, Diagnostic> {
        match self.states.get(&name) {
            Some(State::Ready(value)) => Ok(*value),
            Some(State::Visiting) => Err(Diagnostic::new(span, "cyclic constant dependencies")),
            None => match self.outer.get(&name) {
                Some(Binding::Constant(value)) => Ok(*value),
                Some(Binding::Storage(_)) => Err(Diagnostic::new(
                    span,
                    "mutable storage cannot supply a compile-time constant",
                )),
                None => match self.graph_scope {
                    Some(scope) => match scope.value(
                        &syntax::NamePath {
                            root: name,
                            members: Vec::new(),
                        },
                        span,
                    )? {
                        Binding::Constant(value) => Ok(value),
                        Binding::Storage(_) => Err(Diagnostic::new(
                            span,
                            "mutable storage cannot supply a compile-time constant",
                        )),
                    },
                    None => Err(Diagnostic::new(span, "unknown constant")),
                },
            },
        }
    }
    fn ready_path(&self, path: &syntax::NamePath, span: Span) -> Result<ConstantValue, Diagnostic> {
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
            Binding::Constant(value) => Ok(value),
            Binding::Storage(_) => Err(Diagnostic::new(
                span,
                "mutable storage cannot supply a compile-time constant",
            )),
        }
    }
    fn value(&mut self, name: Symbol, span: Span) -> Result<ConstantValue, Diagnostic> {
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
                            | syntax::ExpressionKind::Member { .. } => {}
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
                    if let Some(ty) = declaration.ty {
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
    for global in module.globals() {
        let (name, value) = match &global.declaration {
            syntax::Declaration::Inferred { name, initializer } => (
                *name,
                jai_eval::evaluate(initializer, |name, span| constants.value(name, span))?,
            ),
            syntax::Declaration::Explicit {
                name,
                ty,
                initializer,
            } => {
                let value = match initializer {
                    Some(e) => jai_eval::evaluate(e, |name, span| constants.value(name, span))?,
                    None => ConstantValue::zero(*ty),
                };
                let value = value.coerce(*ty, global.span)?;
                (*name, value)
            }
            syntax::Declaration::UnresolvedExplicit { .. } => {
                return Err(Diagnostic::new(
                    global.span,
                    "global aggregate types are not implemented",
                ));
            }
        };
        let value = value.coerce(value.ty(), global.span)?;
        let initializer = match value {
            ConstantValue::Int(n) => GlobalInitializer::Int(n),
            ConstantValue::Bool(b) => GlobalInitializer::Bool(b),
            ConstantValue::Literal(_) => unreachable!("global value is coerced before allocation"),
        };
        let global = Global::new(globals.len(), initializer, types);
        let storage = global.storage();
        globals.push(global);
        bindings.insert(name, Binding::Storage(storage));
    }
    Ok((globals, bindings))
}
impl Resolver<'_> {
    pub(super) fn bind_block_constants(
        &mut self,
        statements: &[syntax::Statement],
    ) -> Result<(), Diagnostic> {
        let declarations: Vec<_> = statements
            .iter()
            .filter_map(|s| match s {
                syntax::Statement::Constant(c) => Some(c),
                _ => None,
            })
            .collect();
        if declarations.is_empty() {
            return Ok(());
        }
        let mut visible = HashMap::clone(self.globals);
        for scope in &self.scopes {
            visible.extend(scope.iter().map(|(name, value)| (*name, *value)));
        }
        let mut constants = Constants::new(&declarations, &visible, self.graph_scope)?;
        for declaration in declarations {
            let value = constants.value(declaration.name, declaration.span)?;
            self.bind_name(declaration.name, Binding::Constant(value))?;
        }
        Ok(())
    }
}
