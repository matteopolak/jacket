//! Source controls expand identically in reservation and materialization.
use super::*;
use std::borrow::Cow;

impl<F> TypeResolver<'_, '_, F>
where
    F: FnMut(FileInstanceId, &syntax::Expression) -> Result<ScalarConstant, LocatedDiagnostic>,
{
    pub(super) fn checked_members<'m>(
        &mut self,
        file: FileInstanceId,
        source: &'m [syntax::RecordMember],
        scope: &Substitution,
    ) -> TypeResult<Cow<'m, [syntax::RecordMember]>> {
        use syntax::RecordMember as M;
        if !source.iter().any(|member| {
            matches!(
                member,
                M::Insert(_)
                    | M::Assert { .. }
                    | M::Conditional { .. }
                    | M::CompileTimeCases { .. }
            )
        }) {
            return Ok(Cow::Borrowed(source));
        }
        let mut result = Vec::new();
        let mut pending: Vec<_> = source
            .iter()
            .rev()
            .cloned()
            .map(|member| (member, 0usize))
            .collect();
        let mut visited = 0usize;
        while let Some((member, depth)) = pending.pop() {
            visited += 1;
            if depth > 128 || visited > 65_536 || pending.len() > 65_536 {
                return Err(failure(
                    self.graph,
                    file,
                    Diagnostic::new(
                        span(&member),
                        "record member expansion exceeds compiler declaration budget",
                    ),
                ));
            }
            match member {
                M::CompileTimeCases {
                    cases, ..
                } => {
                    let chosen = self.record_cases(file, &cases, scope)?;
                    pending.extend(chosen.into_iter().rev().map(|member| (member, depth + 1)));
                }
                M::Assert {
                    condition,
                    message,
                    span,
                } => {
                    self.record_assertion(file, &condition, message.as_ref(), scope, span)?;
                }
                M::Conditional {
                    condition,
                    then_members,
                    else_members,
                    ..
                } => {
                    let chosen = if self.record_condition(file, &condition, scope)? {
                        then_members
                    } else {
                        else_members
                    };
                    pending.extend(chosen.into_iter().rev().map(|member| (member, depth + 1)));
                }
                M::Insert(directive) => {
                    if !directive.replacements.is_empty() {
                        return Err(failure(
                            self.graph,
                            file,
                            Diagnostic::new(
                                directive.span,
                                "record declaration insertion cannot use jump replacements",
                            ),
                        ));
                    }
                    let syntax::ExpressionKind::Code(body) = &directive.value.kind else {
                        return Err(failure(
                            self.graph,
                            file,
                            Diagnostic::new(
                                directive.value.span,
                                "record insertion currently requires direct quoted declaration syntax",
                            ),
                        ));
                    };
                    let members = crate::metaprogram::literal_record_members(body)
                        .map_err(|error| failure(self.graph, file, error))?;
                    pending.extend(members.into_iter().rev().map(|member| (member, depth + 1)));
                }
                member => result.push(member),
            }
        }
        Ok(Cow::Owned(result))
    }
}

fn span(member: &syntax::RecordMember) -> Span {
    use syntax::RecordMember as M;
    match member {
        M::Placement(value) => value.span,
        M::Field(value) => value.span,
        M::AnonymousRecord(value) => value.span,
        M::Constant(value) => value.span,
        M::TypeAlias(value) => value.span,
        M::Procedure(value) => value.span,
        M::ProcedurePrototype(value) => value.span,
        M::Record(value) => value.span,
        M::Enum(value) => value.span,
        M::Insert(value) => value.span,
        M::Assert {
            span, ..
        }
        | M::Conditional {
            span, ..
        }
        | M::CompileTimeCases {
            span, ..
        }
        | M::DefaultOverride {
            span, ..
        } => *span,
    }
}
