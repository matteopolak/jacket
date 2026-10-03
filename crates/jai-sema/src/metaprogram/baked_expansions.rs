//! Partial source expansions retain original formal indices and checked constants.
use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum BakedAliasOrigin {
    Module(jai_source::DeclarationId),
    Local(crate::local_declarations::LocalDeclarationId),
}

/// Alias recursion has real declaration identities before a macro is selected.
#[derive(Default)]
pub(super) struct BakedAliasStack(Vec<BakedAliasOrigin>);

impl BakedAliasStack {
    pub(super) fn enter(
        &mut self,
        origin: BakedAliasOrigin,
        enclosing_depth: usize,
        span: Span,
    ) -> Result<(), Diagnostic> {
        if self.0.len().saturating_add(enclosing_depth) >= 128 {
            return Err(Diagnostic::new(
                span,
                "baked source target resolution exceeds depth limit",
            ));
        }
        if self.0.contains(&origin) {
            return Err(Diagnostic::new(span, "cyclic baked source target alias"));
        }
        self.0.push(origin);
        Ok(())
    }

    pub(super) fn leave(&mut self, origin: BakedAliasOrigin) {
        debug_assert_eq!(self.0.pop(), Some(origin));
    }
}

#[derive(Clone)]
pub(super) struct BakedExpandedArgument {
    pub target: MacroId,
    pub parameter_index: usize,
    pub binding: Binding,
    pub source: Option<SourceSpan>,
}

fn checked_assignment(
    target: &ExpandedTarget,
    bound: &[BakedExpandedArgument],
    span: Span,
) -> Result<Vec<bool>, Diagnostic> {
    let mut assigned = vec![false; target.procedure.parameters.len()];
    for argument in bound {
        let message = if argument.target != target.id || argument.parameter_index >= assigned.len()
        {
            Some("baked argument does not belong to the retained source expansion")
        } else if std::mem::replace(&mut assigned[argument.parameter_index], true) {
            Some("duplicate baked expansion parameter")
        } else if !matches!(
            argument.binding,
            Binding::TypedConstant(_) | Binding::Type(_) | Binding::Code(_)
        ) {
            Some("baked expansion argument has no retained constant binding")
        } else {
            None
        };
        if let Some(message) = message {
            return Err(argument.source.map_or_else(
                || Diagnostic::new(span, message),
                |source| Diagnostic::at_source(source, message),
            ));
        }
    }
    Ok(assigned)
}

pub(super) fn unbound_expansion_parameters(
    target: &ExpandedTarget,
    bound: &[BakedExpandedArgument],
    span: Span,
) -> Result<Vec<usize>, Diagnostic> {
    Ok(checked_assignment(target, bound, span)?
        .into_iter()
        .enumerate()
        .filter_map(|(index, assigned)| (!assigned).then_some(index))
        .collect())
}

/// Protocol formals are the remaining original indices, never a shortened AST.
pub(super) fn iteration_expansion_parameters(
    target: &ExpandedTarget,
    bound: &[BakedExpandedArgument],
    span: Span,
) -> Result<[usize; 3], Diagnostic> {
    let remaining = unbound_expansion_parameters(target, bound, span)?;
    if !target.procedure.results.is_empty() || remaining.len() != 3 {
        return Err(Diagnostic::new(
            span,
            "a for expansion requires three unbound parameters (source, Code body, For_Flags) and no results",
        ));
    }
    Ok([remaining[0], remaining[1], remaining[2]])
}

pub(super) fn bound_expansion_bindings(
    target: &ExpandedTarget,
    bound: &[BakedExpandedArgument],
    span: Span,
) -> Result<Vec<(Symbol, Binding)>, Diagnostic> {
    checked_assignment(target, bound, span)?;
    Ok(bound
        .iter()
        .map(|argument| {
            (
                target.procedure.parameters[argument.parameter_index].name,
                argument.binding.clone(),
            )
        })
        .collect())
}

impl Resolver<'_> {
    /// Call this in the alias's defining environment. Formal annotations still
    /// belong to the retained target's defining environment.
    pub(super) fn bind_expanded_partial_arguments(
        &mut self,
        target: &ExpandedTarget,
        arguments: &[syntax::CallArgument],
        previous: &[BakedExpandedArgument],
    ) -> Result<Vec<BakedExpandedArgument>, Diagnostic> {
        let mut assigned = checked_assignment(target, previous, self.span)?;
        let mut bound = previous.to_vec();
        for argument in arguments {
            let span = argument.value.span;
            let name = argument.name.ok_or_else(|| {
                Diagnostic::new(span, "#bake_arguments requires named constant arguments")
            })?;
            if argument.spread {
                return Err(Diagnostic::new(
                    span,
                    "#bake_arguments cannot spread a runtime argument pack",
                ));
            }
            let parameter_index = target
                .procedure
                .parameters
                .iter()
                .position(|parameter| parameter.name == name)
                .ok_or_else(|| Diagnostic::new(span, "unknown baked expansion parameter"))?;
            if std::mem::replace(&mut assigned[parameter_index], true) {
                return Err(Diagnostic::new(span, "duplicate baked expansion parameter"));
            }
            let parameter = &target.procedure.parameters[parameter_index];
            if parameter.using || parameter.variadic {
                return Err(Diagnostic::new(
                    span,
                    "baking using or variadic expansion parameters requires source pack binding",
                ));
            }
            if parameter.evaluation == syntax::ParameterEvaluation::Discard {
                return Err(Diagnostic::new(
                    span,
                    "#bake_arguments cannot supply a discarded expansion parameter",
                ));
            }
            let annotation = match &parameter.binding {
                syntax::ParameterBinding::Required(ty)
                | syntax::ParameterBinding::Defaulted {
                    ty: Some(ty), ..
                } => syntax::TypeSyntax::Builtin(syntax::BuiltinType::Scalar(*ty)),
                syntax::ParameterBinding::RequiredType(ty)
                | syntax::ParameterBinding::DefaultedType {
                    ty: Some(ty), ..
                } => ty.clone(),
                _ => {
                    return Err(Diagnostic::new(
                        span,
                        "baked expansion parameters require a defining source annotation",
                    ));
                }
            };
            let expected = self.expanded_annotation(target, &annotation, parameter.span)?;
            let binding = if expected == self.types.meta_type() {
                match self.expr(&argument.value)? {
                    Expr::Type(ty) => Binding::Type(ty),
                    _ => {
                        return Err(Diagnostic::new(
                            span,
                            "baked Type parameter requires a type value",
                        ));
                    }
                }
            } else if expected == self.types.code_type() {
                match self.expr(&argument.value)? {
                    Expr::Code(code) => Binding::Code(code),
                    _ => {
                        return Err(Diagnostic::new(
                            span,
                            "baked Code parameter requires an explicit captured Code value",
                        ));
                    }
                }
            } else {
                let value = self.expr_expected(&argument.value, expected)?;
                let value = self.coerce_value(value, expected, span)?;
                let constant = self.literal_constant(value, span).map_err(|error| {
                    Diagnostic::new(
                        span,
                        format!(
                            "baked expansion argument must be constant: {}",
                            error.message
                        ),
                    )
                })?;
                Binding::TypedConstant(self.meta.intern_constant(constant))
            };
            bound.push(BakedExpandedArgument {
                target: target.id,
                parameter_index,
                binding,
                source: self.debug.source().map(|source| SourceSpan {
                    source,
                    span,
                }),
            });
        }
        Ok(bound)
    }
}
