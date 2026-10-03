//! Call-site policy belongs to the selected call, never its arguments or a helper.
use super::*;
use jai_types::InlineHint;

enum HintedCall {
    Direct(Signature, Call),
    Indirect {
        callee: ValueExpr,
        arguments: Vec<(ParameterId, ValueExpr)>,
        results: Vec<ResultSignature>,
        contracts: Vec<Option<crate::procedure_values::contracts::ValueContract>>,
        hint: InlineHint,
    },
}
impl HintedCall {
    fn results(&self) -> &[ResultSignature] {
        match self {
            Self::Direct(signature, _) => &signature.results,
            Self::Indirect { results, .. } => results,
        }
    }
    fn statement(self, destinations: Vec<Option<Place>>) -> Statement {
        match self {
            Self::Direct(signature, call) if signature.results.is_empty() => {
                Statement::CallVoid(call)
            }
            Self::Direct(_, call) => Statement::CallResults { call, destinations },
            Self::Indirect {
                callee,
                arguments,
                hint,
                ..
            } => Statement::IndirectCallResults {
                callee: Box::new(callee),
                arguments,
                destinations,
                inline_hint: hint,
            },
        }
    }
}
impl Resolver<'_> {
    pub(crate) fn validate_call_hint(
        &self,
        procedure: ProcedureId,
        hint: InlineHint,
        span: Span,
    ) -> Result<(), Diagnostic> {
        // Source demonstrates Never overriding Always, but not the reverse.
        if self
            .meta
            .procedure_hints
            .get(&procedure)
            .copied()
            .or_else(|| {
                self.graph_scope
                    .and_then(|scope| scope.declared_inline_hint(procedure))
            })
            .is_some_and(|declaration| {
                matches!((declaration, hint), (InlineHint::Never, InlineHint::Always))
            })
        {
            return Err(Diagnostic::new(
                span,
                "contradictory declaration and call-site inlining policy is unsupported pending Jai precedence evidence",
            ));
        }
        if hint == InlineHint::Always
            && (self
                .compile_time
                .is_some_and(|context| context.foreign.contains(&procedure))
                || self
                    .meta
                    .local_declarations
                    .prototypes()
                    .iter()
                    .any(|prototype| {
                        prototype.id == procedure
                            && matches!(
                                prototype.origin,
                                PrototypeOrigin::Foreign { .. }
                                    | PrototypeOrigin::SourceContract { .. }
                            )
                    }))
        {
            return Err(Diagnostic::new(
                span,
                "inline call requires a source procedure body; foreign target has no body",
            ));
        }
        if hint == InlineHint::Always
            && (self
                .graph_scope
                .is_some_and(|scope| scope.has_inline_body(procedure) == Some(false))
                || self
                    .meta
                    .local_declarations
                    .prototypes()
                    .iter()
                    .any(|prototype| {
                        prototype.id == procedure
                            && matches!(prototype.origin, PrototypeOrigin::Compiler)
                    }))
        {
            return Err(Diagnostic::new(
                span,
                "inline call requires an available procedure body",
            ));
        }
        Ok(())
    }
    fn bind_hinted_call(
        &mut self,
        hint: InlineHint,
        expression: &syntax::Expression,
        hint_span: Span,
    ) -> Result<HintedCall, Diagnostic> {
        let span = expression.span;
        let (callee, args) = match &expression.kind {
            syntax::ExpressionKind::ContextCall {
                callee,
                args,
                overrides,
            } => {
                let (signature, call) =
                    self.context_call_binding_with_hint(callee, args, overrides, span, hint)?;
                return Ok(HintedCall::Direct(signature, call));
            }
            syntax::ExpressionKind::Call(name, args) => (
                syntax::Expression {
                    kind: syntax::ExpressionKind::Name(*name),
                    span,
                },
                args,
            ),
            syntax::ExpressionKind::QualifiedCall(path, args) => (
                syntax::Expression {
                    kind: syntax::ExpressionKind::QualifiedName(path.clone()),
                    span,
                },
                args,
            ),
            syntax::ExpressionKind::IndirectCall { callee, args } => ((**callee).clone(), args),
            _ => {
                return Err(Diagnostic::new(
                    hint_span,
                    "call inlining modifier requires a procedure call",
                ));
            }
        };
        let path = match &callee.kind {
            syntax::ExpressionKind::Name(name) => Some(syntax::NamePath {
                root: *name,
                members: vec![],
            }),
            syntax::ExpressionKind::QualifiedName(path) => Some(path.clone()),
            _ => None,
        };
        if let Some(path) = path
            && !self.call_is_indirect(&path, span)
        {
            self.reject_value_expansion(&path, hint_span)?;
            let (signature, call) = self.resolve_call_binding(&path, args, span)?;
            self.validate_call_hint(signature.id, hint, hint_span)?;
            return Ok(HintedCall::Direct(signature, call.with_inline_hint(hint)));
        }
        let value = self.expr(&callee)?;
        let (target, arguments, results) =
            self.resolve_indirect_call_binding_from_source(value, args, span, Some(&callee))?;
        let contracts = self
            .callback_expression_contract(&callee, &target, span)?
            .map(|contract| contract.returned())
            .unwrap_or_default();
        let callee = target;
        if hint == InlineHint::Always && !matches!(callee, ValueExpr::ProcedureValue { .. }) {
            return Err(Diagnostic::new(
                hint_span,
                "inline requires a constant procedure target",
            ));
        }
        if let ValueExpr::ProcedureValue { procedure, .. } = callee {
            self.validate_call_hint(procedure, hint, hint_span)?;
        }
        Ok(HintedCall::Indirect {
            callee,
            arguments,
            results,
            contracts,
            hint,
        })
    }
    pub(crate) fn hinted_call_expression(
        &mut self,
        hint: InlineHint,
        expression: &syntax::Expression,
        span: Span,
    ) -> Result<Expr, Diagnostic> {
        let call = self.bind_hinted_call(hint, expression, span)?;
        let ty = match call.results() {
            [result] => result.ty,
            [] => {
                return match call {
                    HintedCall::Direct(_, call) => Ok(Expr::Void(call)),
                    HintedCall::Indirect {
                        callee,
                        arguments,
                        hint,
                        ..
                    } => Ok(Expr::IndirectVoid {
                        callee: Box::new(callee),
                        arguments,
                        inline_hint: hint,
                    }),
                };
            }
            _ => {
                return Err(Diagnostic::new(
                    span,
                    "multiple procedure results require result binding",
                ));
            }
        };
        let value = match call {
            HintedCall::Direct(_, call) => ValueExpr::Call { call, ty },
            HintedCall::Indirect {
                callee,
                arguments,
                hint,
                ..
            } => ValueExpr::IndirectCall {
                callee: Box::new(callee),
                arguments,
                ty,
                inline_hint: hint,
            },
        };
        self.typed_value(value, ty, span)
    }
    pub(crate) fn discard_hinted_call(
        &mut self,
        hint: InlineHint,
        expression: &syntax::Expression,
        span: Span,
    ) -> Result<Statement, Diagnostic> {
        let call = self.bind_hinted_call(hint, expression, span)?;
        crate::result_obligations::check_result_use(call.results(), &[], 0, span)?;
        let destinations = vec![None; call.results().len()];
        Ok(call.statement(destinations))
    }
    pub(crate) fn capture_hinted_call(
        &mut self,
        hint: InlineHint,
        expression: &syntax::Expression,
        span: Span,
        used: &[bool],
        statements: &mut Vec<Statement>,
        values: &mut Vec<Place>,
    ) -> Result<(), Diagnostic> {
        let call = self.bind_hinted_call(hint, expression, span)?;
        crate::result_obligations::check_result_use(call.results(), used, values.len(), span)?;
        if call.results().is_empty() {
            return Err(Diagnostic::new(
                span,
                "void call supplies no assignment results",
            ));
        }
        let contracts = match &call {
            HintedCall::Direct(_, call) => {
                let source = match &expression.kind {
                    syntax::ExpressionKind::Call(_, arguments)
                    | syntax::ExpressionKind::QualifiedCall(_, arguments)
                    | syntax::ExpressionKind::ContextCall {
                        args: arguments, ..
                    }
                    | syntax::ExpressionKind::IndirectCall {
                        args: arguments, ..
                    } => Some(arguments.as_slice()),
                    _ => None,
                };
                self.call_result_contracts_for_source(
                    call.procedure,
                    &call.arguments,
                    source,
                    span,
                    0,
                )?
            }
            HintedCall::Indirect { contracts, .. } => contracts.clone(),
        };
        let mut destinations = vec![];
        for (index, result) in call.results().iter().enumerate() {
            let local = self.allocate_typed(result.ty)?;
            self.bind_value_contract(local.place(), contracts.get(index).cloned().flatten(), span)?;
            destinations.push(Some(local.place()));
            values.push(local.place());
        }
        statements.push(call.statement(destinations));
        Ok(())
    }
}
