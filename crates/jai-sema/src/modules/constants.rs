//! Resolve scalar constant dependencies by declaration identity.
use super::*;
use syntax::{Expression, ExpressionKind};
#[derive(Clone, Copy)]
enum ConstantState {
    Visiting,
    Ready(ConstantValue),
}
enum Work {
    Enter(DeclarationId, SourceSpan),
    Finish(DeclarationId),
}
pub(super) struct Constants<'a> {
    graph: &'a ModuleGraph,
    states: HashMap<DeclarationId, ConstantState>,
}
impl<'a> Constants<'a> {
    pub(super) fn new(graph: &'a ModuleGraph) -> Self {
        Self {
            graph,
            states: HashMap::new(),
        }
    }
    pub(super) fn ready(
        &self,
        id: DeclarationId,
        site: SourceSpan,
    ) -> Result<ConstantValue, LocatedDiagnostic> {
        match self.states.get(&id) {
            Some(ConstantState::Ready(value)) => Ok(*value),
            Some(ConstantState::Visiting) => Err(LocatedDiagnostic {
                location: site,
                message: "cyclic constant dependencies".into(),
            }),
            None => Err(LocatedDiagnostic {
                location: site,
                message: "declaration cannot supply a compile-time constant".into(),
            }),
        }
    }
    pub(super) fn value(
        &mut self,
        id: DeclarationId,
        site: SourceSpan,
    ) -> Result<ConstantValue, LocatedDiagnostic> {
        let mut work = vec![Work::Enter(id, site)];
        while let Some(step) = work.pop() {
            match step {
                Work::Enter(id, site) => {
                    match self.states.get(&id) {
                        Some(ConstantState::Ready(_)) => continue,
                        Some(ConstantState::Visiting) => {
                            self.ready(id, site)?;
                            unreachable!();
                        }
                        None => {}
                    }
                    let declaration = self
                        .graph
                        .declaration(id)
                        .expect("resolved declaration exists");
                    let FileDeclarationKind::Constant(constant) = &declaration.syntax().kind else {
                        return self.ready(id, site);
                    };
                    self.states.insert(id, ConstantState::Visiting);
                    work.push(Work::Finish(id));
                    let file = declaration.file();
                    let source = declaration.location().source;
                    let mut expressions = vec![&constant.initializer];
                    let mut dependencies = Vec::new();
                    while let Some(expression) = expressions.pop() {
                        let name = match &expression.kind {
                            ExpressionKind::Name(name) => Some(path(*name)),
                            ExpressionKind::QualifiedName(path) => Some(path.clone()),
                            ExpressionKind::Unary(_, inner) | ExpressionKind::Cast(_, _, inner) => {
                                expressions.push(inner);
                                None
                            }
                            ExpressionKind::Binary(_, left, right) => {
                                expressions.push(right);
                                expressions.push(left);
                                None
                            }
                            ExpressionKind::Conditional(conditional) => {
                                if let Some(otherwise) = &conditional.else_value {
                                    expressions.push(otherwise);
                                }
                                expressions.push(&conditional.then_value);
                                expressions.push(&conditional.condition);
                                None
                            }
                            _ => None,
                        };
                        if let Some(path) = name {
                            let dependency =
                                declaration_id(self.graph, file, &path, expression.span)
                                    .map_err(|error| located(self.graph, file, error))?;
                            dependencies.push(Work::Enter(
                                dependency,
                                SourceSpan {
                                    source,
                                    span: expression.span,
                                },
                            ));
                        }
                    }
                    work.extend(dependencies.into_iter().rev());
                }
                Work::Finish(id) => {
                    let declaration = self.graph.declaration(id).unwrap();
                    let FileDeclarationKind::Constant(constant) = &declaration.syntax().kind else {
                        unreachable!()
                    };
                    let mut value = self.evaluate(declaration.file(), &constant.initializer)?;
                    if let Some(ty) = constant.ty {
                        value = value
                            .coerce(ty, constant.span)
                            .map_err(|error| located(self.graph, declaration.file(), error))?;
                    }
                    self.states.insert(id, ConstantState::Ready(value));
                }
            }
        }
        self.ready(id, site)
    }
    pub(super) fn evaluate(
        &self,
        file: FileInstanceId,
        expression: &Expression,
    ) -> Result<ConstantValue, LocatedDiagnostic> {
        jai_eval::evaluate_paths(expression, |path, span| {
            let id = declaration_id(self.graph, file, path, span)?;
            self.ready(
                id,
                SourceSpan {
                    source: self.graph.file(file).unwrap().source(),
                    span,
                },
            )
            .map_err(|error| Diagnostic::new(error.location.span, error.message))
        })
        .map_err(|error| located(self.graph, file, error))
    }
}
