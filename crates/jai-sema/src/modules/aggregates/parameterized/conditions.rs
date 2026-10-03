//! Record shape conditions read definition bindings and baked formal values.
use super::*;

impl<F> TypeResolver<'_, '_, F>
where
    F: FnMut(FileInstanceId, &syntax::Expression) -> Result<ScalarConstant, LocatedDiagnostic>,
{
    pub(super) fn record_cases(
        &mut self,
        file: FileInstanceId,
        cases: &syntax::CompileTimeCases<syntax::RecordMember>,
        substitution: &Substitution,
    ) -> TypeResult<Vec<syntax::RecordMember>> {
        use crate::compile_time_cases::{CaseValue, select};
        let header = cases.header();
        let subject = self.record_case_value(file, &header.value, None, substitution)?;
        let ty = match &subject {
            CaseValue::Type(_) => self.types.meta_type(),
            CaseValue::Constant(value) => value.ty,
        };
        if !matches!(
            self.types.kind(ty),
            Ok(jai_types::TypeKind::Type
                | jai_types::TypeKind::Bool
                | jai_types::TypeKind::Integer(_)
                | jai_types::TypeKind::Enum(_)
                | jai_types::TypeKind::String)
        ) {
            return Err(failure(
                self.graph,
                file,
                Diagnostic::new(
                    header.value.span,
                    "compile-time case selector requires a type, bool, integer, enum, or string constant",
                ),
            ));
        }
        let labels = header
            .labels
            .iter()
            .map(|label| self.record_case_value(file, label, Some(ty), substitution))
            .collect::<Result<Vec<_>, _>>()?;
        let selected = select(&header, &subject, &labels, self.types)
            .map_err(|error| failure(self.graph, file, error))?;
        cases.selected_body(selected).ok_or_else(|| {
            failure(
                self.graph,
                file,
                Diagnostic::new(
                    cases.span,
                    "last compile-time case cannot #through without a following case",
                ),
            )
        })
    }

    fn record_case_value(
        &mut self,
        file: FileInstanceId,
        expression: &syntax::Expression,
        expected: Option<TypeId>,
        substitution: &Substitution,
    ) -> TypeResult<crate::compile_time_cases::CaseValue> {
        use crate::compile_time_cases::CaseValue;
        let expected = match expected {
            Some(expected) => expected,
            None => match self.inferred_default_type(file, expression, substitution) {
                Ok(ty) => ty,
                Err(pending @ TypeFailure::Pending(_)) => return Err(pending),
                Err(original @ TypeFailure::Diagnostic(_)) => {
                    let Some(syntax) = type_expression(expression) else {
                        return Err(original);
                    };
                    match self.resolve(file, &syntax, Some(substitution), expression.span) {
                        Ok(ty) => return Ok(CaseValue::Type(ty)),
                        Err(pending @ TypeFailure::Pending(_)) => return Err(pending),
                        Err(TypeFailure::Diagnostic(_)) => return Err(original),
                    }
                }
            },
        };
        let value = self.baked(file, expression, expected, Some(substitution))?;
        match value {
            BakedValue::Type(ty) => Ok(CaseValue::Type(ty)),
            value => value
                .into_runtime(expected, self.types)
                .map(CaseValue::Constant)
                .map_err(|error| {
                    failure(
                        self.graph,
                        file,
                        Diagnostic::new(expression.span, error.to_string()),
                    )
                }),
        }
    }

    pub(super) fn record_assertion(
        &mut self,
        file: FileInstanceId,
        condition: &syntax::Expression,
        message: Option<&syntax::Expression>,
        substitution: &Substitution,
        span: Span,
    ) -> TypeResult<()> {
        let selected = self.record_condition(file, condition, substitution)?;
        let message = match message {
            None => None,
            Some(source) => {
                let ty = self.types.string();
                if self
                    .inferred_default_type(file, source, substitution)
                    .is_ok_and(|actual| actual != ty)
                {
                    return Err(failure(
                        self.graph,
                        file,
                        Diagnostic::new(
                            source.span,
                            "#assert message requires a compile-time string",
                        ),
                    ));
                }
                let value = super::super::defaults::Defaults::with_evaluator(
                    self.graph,
                    self.types,
                    self.nominals,
                    &mut *self.evaluate,
                )
                .with_specializations(self.records)
                .with_substitution(Some(substitution.clone()))
                .expression(file, source, ty)?;
                let jai_ir::ConstantKind::StringBytes(bytes) = value.kind else {
                    return Err(failure(
                        self.graph,
                        file,
                        Diagnostic::new(
                            source.span,
                            "#assert message requires a compile-time string",
                        ),
                    ));
                };
                Some(String::from_utf8_lossy(&bytes).into_owned())
            }
        };
        if selected {
            Ok(())
        } else {
            Err(failure(
                self.graph,
                file,
                Diagnostic::new(
                    span,
                    match message {
                        Some(message) => format!("record compile-time assertion failed: {message}"),
                        None => "record compile-time assertion failed".into(),
                    },
                ),
            ))
        }
    }

    pub(super) fn record_condition(
        &mut self,
        file: FileInstanceId,
        expression: &syntax::Expression,
        substitution: &Substitution,
    ) -> TypeResult<bool> {
        if matches!(expression.kind, syntax::ExpressionKind::Null) {
            return Ok(false);
        }
        let value = jai_eval::evaluate_paths(expression, |path, span| {
            if let Some(value) = member_value(
                self.graph,
                file,
                self.nominals,
                self.records,
                Some(substitution),
                path,
                span,
            ) {
                if let Some(value) = baked_scalar(&value) {
                    return Ok(value);
                }
                if let BakedValue::Value(value) = value {
                    use jai_ir::ConstantKind as K;
                    match value.kind {
                        K::Procedure(_) => return Ok(ScalarConstant::Bool(true)),
                        K::Enum(value) => return Ok(ScalarConstant::Int(value)),
                        K::Zero
                            if matches!(
                                self.types.kind(value.ty),
                                Ok(jai_types::TypeKind::Procedure(_)
                                    | jai_types::TypeKind::Pointer(_))
                            ) =>
                        {
                            return Ok(ScalarConstant::Bool(false));
                        }
                        _ => {}
                    }
                }
                return Err(Diagnostic::new(
                    span,
                    "record shape condition requires a scalar or nullable callable value",
                ));
            }
            (self.evaluate)(
                file,
                &syntax::Expression {
                    kind: syntax::ExpressionKind::QualifiedName(path.clone()),
                    span,
                },
            )
            .map_err(|error| Diagnostic::at_source(error.location, error.message))
        })
        .map_err(|error| failure(self.graph, file, error))?;
        match value {
            ScalarConstant::Bool(value) => Ok(value),
            ScalarConstant::Literal(value) => Ok(value != 0),
            ScalarConstant::Int(value) => Ok(value.value() != 0),
            ScalarConstant::Float(_) | ScalarConstant::WeakFloat(_) => Err(failure(
                self.graph,
                file,
                Diagnostic::new(
                    expression.span,
                    "record shape condition requires a predicate",
                ),
            )),
        }
    }
}
