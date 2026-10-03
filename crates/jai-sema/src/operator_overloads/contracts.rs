//! Operator contracts retain source formals independently of runtime ABI slots.
use super::*;

struct OperatorFormal {
    ty: TypeId,
    evaluation: syntax::ParameterEvaluation,
    has_default: bool,
}

pub(crate) fn validate_signature(
    types: &TypeRegistry,
    kind: OperatorKind,
    signature: &Signature,
    span: Span,
) -> Result<(), Diagnostic> {
    let parameters = signature
        .parameters
        .iter()
        .map(|parameter| OperatorFormal {
            ty: parameter.ty,
            evaluation: parameter.evaluation,
            has_default: parameter.default.is_some(),
        })
        .collect::<Vec<_>>();
    validate_contract(types, kind, &parameters, &signature.results, span)
}

impl Resolver<'_> {
    pub(crate) fn validate_source_operator_signature(
        &self,
        kind: OperatorKind,
        signature: &Signature,
        source_parameters: &[syntax::Parameter],
        span: Span,
    ) -> Result<(), Diagnostic> {
        if signature.parameters.len() != source_parameters.len() {
            return Err(Diagnostic::new(
                span,
                "local operator source formals differ from its prepared signature",
            ));
        }
        let parameters = signature
            .parameters
            .iter()
            .zip(source_parameters)
            .map(|(parameter, source)| OperatorFormal {
                ty: parameter.ty,
                evaluation: parameter.evaluation,
                has_default: matches!(
                    source.binding,
                    syntax::ParameterBinding::Defaulted { .. }
                        | syntax::ParameterBinding::DefaultedType { .. }
                ),
            })
            .collect::<Vec<_>>();
        validate_contract(self.types, kind, &parameters, &signature.results, span)
    }

    pub(super) fn validate_file_operator_match(
        &mut self,
        kind: OperatorKind,
        matched: &Match,
        signature: &Signature,
        span: Span,
    ) -> Result<(), Diagnostic> {
        let scope = self
            .graph_scope
            .ok_or_else(|| Diagnostic::new(span, "operator requires its defining source graph"))?;
        let candidate = scope.candidate(matched.declaration, self.types, span)?;
        let mut parameters = Vec::new();
        for parameter in &candidate.parameters {
            let ty = scope.materialize_pattern(
                &parameter.ty,
                &matched.substitution,
                self.types,
                &mut self.meta.record_specializations,
                span,
            )?;
            parameters.push(OperatorFormal {
                ty,
                evaluation: parameter.evaluation,
                has_default: parameter.default.is_some(),
            });
        }
        validate_contract(self.types, kind, &parameters, &signature.results, span)
    }

    pub(super) fn check_captured_operator_projection(
        &self,
        selection: &SelectedOperator,
        span: Span,
    ) -> Result<(), Diagnostic> {
        if let OperatorOrigin::File(matched) = &selection.origin {
            let scope = self.graph_scope.ok_or_else(|| {
                Diagnostic::new(span, "operator requires its defining source graph")
            })?;
            let candidate = scope.candidate(matched.declaration, self.types, span)?;
            if candidate
                .parameters
                .iter()
                .any(|parameter| parameter.is_baked(&matched.substitution))
            {
                return Err(Diagnostic::new(
                    span,
                    "baked operator operands require source-aware captured argument projection",
                ));
            }
        }
        Ok(())
    }
}

fn validate_contract(
    types: &TypeRegistry,
    kind: OperatorKind,
    parameters: &[OperatorFormal],
    results: &[ResultSignature],
    span: Span,
) -> Result<(), Diagnostic> {
    if parameters
        .iter()
        .any(|parameter| parameter.evaluation != syntax::ParameterEvaluation::Evaluate)
    {
        return Err(Diagnostic::new(
            span,
            "discarded operator parameters require source-aware captured argument binding",
        ));
    }
    let pointer_mutation = matches!(kind, OperatorKind::IndexAssign | OperatorKind::Compound(_))
        && parameters.first().is_some_and(|parameter| {
            matches!(types.kind(parameter.ty), Ok(TypeKind::Pointer(pointee)) if nominal_type(types, *pointee))
        });
    let result_count = usize::from(!pointer_mutation);
    if parameters.len() < kind.arity()
        || parameters
            .iter()
            .skip(kind.arity())
            .any(|parameter| !parameter.has_default)
        || results.len() != result_count
    {
        return Err(Diagnostic::new(
            span,
            if pointer_mutation {
                "mutating operator requires its declared operands and no result"
            } else {
                "operator requires its declared operands and one result"
            },
        ));
    }
    if !parameters
        .iter()
        .take(kind.arity())
        .any(|parameter| nominal_type(types, parameter.ty))
    {
        return Err(Diagnostic::new(
            span,
            "operator overload requires a nominal record operand",
        ));
    }
    if pointer_mutation {
        return Ok(());
    }
    let result = results[0].ty;
    if kind == OperatorKind::IndexAssign {
        return Err(Diagnostic::new(
            span,
            "index assignment requires a pointer to nominal storage",
        ));
    }
    if kind == OperatorKind::IndexAddress
        && (!matches!(types.kind(result), Ok(TypeKind::Pointer(_)))
            || !parameters.first().is_some_and(|parameter| matches!(types.kind(parameter.ty), Ok(TypeKind::Pointer(pointee)) if nominal_type(types, *pointee)))) {
        return Err(Diagnostic::new(
            span,
            "index address operator requires nominal storage and a pointer result",
        ));
    }
    if matches!(
        kind,
        OperatorKind::Binary(
            BinaryOp::Equal
                | BinaryOp::NotEqual
                | BinaryOp::Less
                | BinaryOp::LessEqual
                | BinaryOp::Greater
                | BinaryOp::GreaterEqual
        ) | OperatorKind::Unary(UnaryOp::LogicalNot)
    ) && result != types.scalar(ScalarType::Bool)
    {
        return Err(Diagnostic::new(
            span,
            "comparison operator must return bool",
        ));
    }
    if matches!(
        kind,
        OperatorKind::Binary(
            BinaryOp::Add
                | BinaryOp::Subtract
                | BinaryOp::Multiply
                | BinaryOp::Divide
                | BinaryOp::Remainder
        ) | OperatorKind::Unary(UnaryOp::Positive | UnaryOp::Negate)
    ) && !parameters.iter().take(kind.arity()).any(|parameter| {
        parameter.ty == result
            && matches!(
                types.kind(parameter.ty),
                Ok(TypeKind::Record(_) | TypeKind::Distinct(_))
            )
    }) {
        return Err(Diagnostic::new(
            span,
            "arithmetic operator must return its nominal operand type",
        ));
    }
    Ok(())
}
