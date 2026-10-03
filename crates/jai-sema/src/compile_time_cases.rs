//! Canonical source case values and one immutable selection for every source scope.
use super::*;
#[cfg(test)]
mod tests;
use jai_types::TypeKind;
use syntax::{CompileTimeCaseChoice as Choice, CompileTimeCaseHeader};

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum CaseValue {
    Type(TypeId),
    Constant(ConstantValue),
}
impl CaseValue {
    fn ty(&self, types: &dyn jai_types::TypeView) -> TypeId {
        match self {
            Self::Type(_) => types.meta_type(),
            Self::Constant(value) => value.ty,
        }
    }
}

pub(crate) fn select(
    header: &CompileTimeCaseHeader,
    subject: &CaseValue,
    labels: &[CaseValue],
    types: &dyn jai_types::TypeView,
) -> Result<Choice, Diagnostic> {
    if labels.len() != header.labels.len() {
        return Err(Diagnostic::new(
            header.span,
            "source case labels do not match their original table",
        ));
    }
    let ty = subject.ty(types);
    for (index, label) in labels.iter().enumerate() {
        if label.ty(types) != ty {
            return Err(Diagnostic::new(
                header.labels[index].span,
                "source case label has a different canonical type",
            ));
        }
        if labels[..index].contains(label) {
            return Err(Diagnostic::new(
                header.labels[index].span,
                "duplicate compile-time case label",
            ));
        }
    }
    if header.complete && !header.has_default {
        if header.operator != syntax::CaseOperator::Equal {
            return Err(Diagnostic::new(
                header.span,
                "#complete source cases require == or a default case",
            ));
        }
        let exhaustive = match types.kind(ty).map_err(|error| Diagnostic::new(header.span, error.to_string()))? {
            TypeKind::Bool => [false, true].iter().all(|value| labels.iter().any(|label| matches!(label, CaseValue::Constant(ConstantValue {kind:ConstantKind::Bool(actual),..}) if actual==value))),
            TypeKind::Enum(_) => types.enum_definition(ty).map_err(|error| Diagnostic::new(header.span,error.to_string()))?.values.iter().all(|value| labels.iter().any(|label| matches!(label, CaseValue::Constant(ConstantValue {kind:ConstantKind::Enum(actual),..}) if actual==value))),
            _ => false,
        };
        if !exhaustive {
            return Err(Diagnostic::new(
                header.span,
                "#complete source cases do not cover the canonical value domain",
            ));
        }
    }
    let selected = labels.iter().position(|label| match header.operator {
        syntax::CaseOperator::Equal => label == subject,
        syntax::CaseOperator::NotEqual => label != subject,
    });
    Ok(match selected {
        Some(index) => Choice::Arm(index),
        None if header.has_default => Choice::Default,
        None => Choice::None,
    })
}

impl Resolver<'_> {
    pub(crate) fn compile_time_case_selection(
        &mut self,
        header: &CompileTimeCaseHeader,
    ) -> Result<Choice, Diagnostic> {
        if let Some(scope) = self.graph_scope {
            let specialization = self.meta.source_specialization_keys.get(&self.procedure);
            if let Some(choice) = scope.selected_case(header.span, specialization) {
                return Ok(choice);
            }
        }
        let prepared = self.prepare_condition_source(&header.value)?;
        self.reject_runtime_condition_names(&prepared)?;
        let expression = self.expr(&prepared)?;
        let subject = self.case_value(expression, None, header.value.span)?;
        let ty = subject.ty(self.types);
        let supported = matches!(
            self.types.kind(ty),
            Ok(TypeKind::Type
                | TypeKind::Bool
                | TypeKind::Integer(_)
                | TypeKind::Enum(_)
                | TypeKind::String)
        );
        if !supported {
            return Err(Diagnostic::new(
                header.value.span,
                "compile-time case selector requires a type, bool, integer, enum, or string constant",
            ));
        }
        let mut labels = Vec::new();
        for label in &header.labels {
            let prepared = self.prepare_condition_source(label)?;
            self.reject_runtime_condition_names(&prepared)?;
            let expression = if matches!(subject, CaseValue::Type(_)) {
                self.expr(&prepared)?
            } else {
                self.expr_expected(&prepared, ty)?
            };
            labels.push(self.case_value(expression, Some(ty), label.span)?);
        }
        select(header, &subject, &labels, self.types)
    }
    fn case_value(
        &mut self,
        expression: Expr,
        expected: Option<TypeId>,
        span: Span,
    ) -> Result<CaseValue, Diagnostic> {
        match expression {
            Expr::Type(ty) => Ok(CaseValue::Type(ty)),
            expression => {
                let value = match expected {
                    Some(ty) => self.coerce_value(expression, ty, span)?,
                    None => expression.value(span)?,
                };
                self.evaluate_pure_constant(value, span)
                    .map(CaseValue::Constant)
            }
        }
    }
}
