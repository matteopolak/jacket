//! Result-use contracts belong to source bindings, independently of ABI type identity.
use super::*;
mod bound_operators;

pub(crate) fn check_result_use(
    results: &[ResultSignature],
    used: &[bool],
    offset: usize,
    span: Span,
) -> Result<(), Diagnostic> {
    for (index, result) in results.iter().enumerate() {
        if result.usage == syntax::ResultUsage::Required
            && !used.get(offset + index).copied().unwrap_or(false)
        {
            return Err(Diagnostic::new(
                span,
                format!(
                    "result {} is marked #must and cannot be discarded",
                    index + 1
                ),
            ));
        }
    }
    Ok(())
}

impl Resolver<'_> {
    pub(crate) fn is_result_discard_target(&self, target: &syntax::PlaceSyntax) -> bool {
        match &target.kind {
            syntax::PlaceKind::Name(name) => self.symbols.name(*name) == "_",
            syntax::PlaceKind::Qualified(path) => {
                path.members.is_empty() && self.symbols.name(path.root) == "_"
            }
            _ => false,
        }
    }

    pub(crate) fn discard_result_declaration(
        &mut self,
        declaration: &syntax::Declaration,
    ) -> Result<Option<Statement>, Diagnostic> {
        let (name, annotation, initializer) = match declaration {
            // External storage has no initializer result to consume or discard.
            syntax::Declaration::External { .. } => return Ok(None),
            syntax::Declaration::Inferred {
                name, initializer, ..
            } => (*name, None, Some(initializer)),
            syntax::Declaration::Explicit {
                name,
                ty,
                initializer,
                ..
            } => (
                *name,
                Some(syntax::TypeSyntax::Builtin(syntax::BuiltinType::Scalar(
                    *ty,
                ))),
                initializer.as_ref(),
            ),
            syntax::Declaration::UnresolvedExplicit {
                name,
                ty,
                initializer,
                ..
            } => (*name, Some(ty.clone()), initializer.as_ref()),
        };
        if self.symbols.name(name) != "_" {
            return Ok(None);
        }
        let values = initializer
            .filter(|expression| !matches!(expression.kind, syntax::ExpressionKind::Uninitialized))
            .map(std::slice::from_ref)
            .unwrap_or(&[]);
        self.declare_results(&[name], annotation.as_ref(), values)
            .map(Some)
    }

    pub(crate) fn discard_call_results(
        &mut self,
        expression: &syntax::Expression,
    ) -> Result<Option<Statement>, Diagnostic> {
        let span = expression.span;
        if let syntax::ExpressionKind::CallHint { hint, call } = &expression.kind {
            return self.discard_hinted_call(*hint, call, span).map(Some);
        }
        if let syntax::ExpressionKind::ContextCall {
            callee,
            args,
            overrides,
        } = &expression.kind
        {
            let (signature, call) = self.context_call_binding(callee, args, overrides, span)?;
            check_result_use(&signature.results, &[], 0, span)?;
            return Ok(Some(if signature.results.is_empty() {
                Statement::CallVoid(call)
            } else {
                Statement::CallResults {
                    destinations: vec![None; signature.results.len()],
                    call,
                }
            }));
        }
        let path = match &expression.kind {
            syntax::ExpressionKind::Call(name, args) => Some((
                syntax::NamePath {
                    root: *name,
                    members: vec![],
                },
                args,
            )),
            syntax::ExpressionKind::QualifiedCall(path, args) => Some((path.clone(), args)),
            _ => None,
        };
        if let Some((path, args)) = path {
            if let Some((callee, arguments, results)) =
                self.named_short_lambda_call_binding(&path, args, span)?
            {
                check_result_use(&results, &[], 0, span)?;
                return Ok(Some(Statement::IndirectCallResults {
                    inline_hint: jai_types::InlineHint::Automatic,
                    callee: Box::new(callee),
                    arguments,
                    destinations: vec![None; results.len()],
                }));
            }
            self.resolve_local_name(path.root, span)?;
            if !self.call_is_indirect(&path, span) {
                let (signature, call) = self.resolve_call_binding(&path, args, span)?;
                check_result_use(&signature.results, &[], 0, span)?;
                return Ok(Some(if signature.results.is_empty() {
                    Statement::CallVoid(call)
                } else {
                    Statement::CallResults {
                        destinations: vec![None; signature.results.len()],
                        call,
                    }
                }));
            }
            let source = self.baked_callback_call_source(&path, span)?;
            let callee = self.path_expression(&path, span)?;
            return self
                .discard_indirect_results(callee, args, span, source.as_ref())
                .map(Some);
        }
        if let syntax::ExpressionKind::IndirectCall { callee, args } = &expression.kind {
            let target = self.expr(callee)?;
            return self
                .discard_indirect_results(target, args, span, Some(callee))
                .map(Some);
        }
        Ok(None)
    }

    fn discard_indirect_results(
        &mut self,
        callee: Expr,
        args: &[syntax::CallArgument],
        span: Span,
        source: Option<&syntax::Expression>,
    ) -> Result<Statement, Diagnostic> {
        let (callee, arguments, results) =
            self.resolve_indirect_call_binding_from_source(callee, args, span, source)?;
        check_result_use(&results, &[], 0, span)?;
        Ok(Statement::IndirectCallResults {
            inline_hint: jai_types::InlineHint::Automatic,
            callee: Box::new(callee),
            arguments,
            destinations: vec![None; results.len()],
        })
    }
}
