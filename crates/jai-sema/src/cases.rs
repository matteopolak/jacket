//! Checked case dispatch: one hidden subject slot and independent arm scopes.
use super::*;
impl Resolver<'_> {
    pub(super) fn resolve_cases(
        &mut self,
        case: &syntax::CaseStatement,
    ) -> Result<Statement, Diagnostic> {
        let span = case.value.span;
        let (subject, place) = match self.expr(&case.value)? {
            Expr::Bool(value) => {
                let id = self
                    .allocate(ScalarType::Bool)
                    .boolean(self.types)
                    .expect("boolean subject");
                (
                    Statement::StoreBool(id.place(), value),
                    Storage::Bool(id.place()),
                )
            }
            value => {
                let value = value.int(span)?;
                let id = self
                    .allocate(ScalarType::Int(value.ty()))
                    .integer(self.types)
                    .expect("integer subject");
                (
                    Statement::StoreInt(id.place(), value),
                    Storage::Int(id.place()),
                )
            }
        };
        let mut seen = Vec::new();
        let mut arms = Vec::new();
        for (label, body, through) in &case.arms {
            let value = jai_eval::evaluate(label, |name, span| match self.lookup(name)? {
                Binding::Constant(value) => Ok(value),
                _ => Err(Diagnostic::new(
                    span,
                    "case label must be a compile-time constant",
                )),
            })?;
            let ty = match place {
                Storage::Int(id) => ScalarType::Int(id.ty()),
                Storage::Bool(_) => ScalarType::Bool,
            };
            let value = value.coerce(ty, label.span)?;
            if seen.contains(&value) {
                return Err(Diagnostic::new(label.span, "duplicate case label"));
            }
            seen.push(value);
            let lhs = match place {
                Storage::Int(id) => Expr::Int(IntExpr::load(id)),
                Storage::Bool(id) => Expr::Bool(BoolExpr::Load(id)),
            };
            let op = match case.operator {
                syntax::CaseOperator::Equal => BinaryOp::Equal,
                syntax::CaseOperator::NotEqual => BinaryOp::NotEqual,
            };
            let condition = self
                .binary(op, lhs, Self::constant(value), label.span)?
                .bool(label.span)?;
            let body = self.block(body, true)?;
            if *through && body.flow == Flow::Terminates {
                return Err(Diagnostic::new(label.span, "unreachable #through"));
            }
            arms.push(CaseArm {
                condition,
                body,
                through: *through,
            });
        }
        let default = case
            .default
            .as_ref()
            .map(|b| self.block(b, true))
            .transpose()?;
        if arms.last().is_some_and(|a| a.through) && default.is_none() {
            return Err(Diagnostic::new(
                span,
                "last case cannot #through without a following case",
            ));
        }
        if case.complete {
            let complete = matches!(place, Storage::Bool(_))
                && seen.contains(&ConstantValue::Bool(true))
                && seen.contains(&ConstantValue::Bool(false))
                && case.operator == syntax::CaseOperator::Equal;
            if !complete {
                return Err(Diagnostic::new(
                    span,
                    "#complete currently requires both bool labels with ==; enum types are not implemented",
                ));
            }
        }
        let exhaustive = case.complete;
        let terminal_fallback = default
            .as_ref()
            .map_or(exhaustive, |b| b.flow == Flow::Terminates);
        let flow = if terminal_fallback
            && arms
                .iter()
                .all(|a| a.through || a.body.flow == Flow::Terminates)
        {
            Flow::Terminates
        } else {
            Flow::FallsThrough
        };
        Ok(Statement::Cases(Cases {
            subject: Box::new(subject),
            arms,
            default,
            flow,
            exhaustive,
        }))
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn rejects_invalid_case_labels_and_control() {
        for source in [
            "main :: () { n := 1; if n == { case n; } }",
            "f :: ()->int { return 1; } main :: () { if 1 == { case f(); } }",
            "main :: () { if 1 == { case 1; case 1+0; } }",
            "main :: () { if true == { case 1; } }",
            "main :: () { if 1 == { case 1; #through; } }",
            "main :: () { if 1 == { case 1; return; #through; case; } }",
            "main :: () { if #complete true == { case true; } }",
            "main :: () { if #complete 1 == { case 1; case; } }",
            "main :: () { if 1 == { case 1; x := 2; case; x = 3; } }",
            "main :: ()->int { if #complete true == { case true; return 1; case false; } }",
            "main :: ()->int { if true == { case true; return 1; case false; return 2; } }",
            "main :: ()->int { if #complete true == { case true; return 1; case false; return 2; } return 3; }",
            "main :: () { if #complete true != { case true; case false; } }",
            "main :: () { if #complete true == { } }",
            "main :: () { if #complete true == { case; } }",
            "main :: ()->int { if 1 == { } }",
        ] {
            let module = jai_syntax::parse(source).unwrap();
            assert!(crate::resolve(&module).is_err(), "{source}");
        }
    }
}
