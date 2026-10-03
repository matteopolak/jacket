//! Select record declarations before shape binding, then check active assertions.
use super::*;
mod promoted_fields;

const MAXIMUM_DEPTH: usize = 128;
const MAXIMUM_MEMBERS: usize = 65_536;

impl Resolver<'_> {
    pub(crate) fn select_local_record_members(
        &mut self,
        members: &[syntax::RecordMember],
    ) -> Result<Vec<syntax::RecordMember>, Diagnostic> {
        if members.len() > MAXIMUM_MEMBERS {
            return Err(selection_budget(member_span(&members[MAXIMUM_MEMBERS])));
        }
        self.register_active_record_members(members)?;
        let mut selected: Vec<_> = members
            .iter()
            .cloned()
            .map(|member| (member, 0usize))
            .collect();
        let mut visited = 0usize;
        let mut ready_promotions = HashSet::new();
        loop {
            let (mut progress, mut pending_error) = self.prepare_record_promotions(
                selected.iter().map(|(member, _)| member),
                &mut ready_promotions,
            );
            let mut pending_promotions = pending_error.is_some();
            let mut index = 0;
            while index < selected.len() {
                let (member, depth) = &selected[index];
                let depth = *depth;
                visited += 1;
                if depth > MAXIMUM_DEPTH
                    || visited > MAXIMUM_MEMBERS
                    || selected.len() > MAXIMUM_MEMBERS
                {
                    return Err(selection_budget(member_span(member)));
                }
                let chosen = match member {
                    syntax::RecordMember::Conditional {
                        condition,
                        then_members,
                        else_members,
                        ..
                    } => {
                        if pending_promotions
                            && !promoted_fields::independent_of_unready_fields(condition)
                        {
                            index += 1;
                            continue;
                        }
                        self.compile_time_condition(condition)
                            .map(|choice| if choice { then_members } else { else_members }.clone())
                    }
                    syntax::RecordMember::CompileTimeCases { cases, .. } => {
                        if pending_promotions
                            && (!promoted_fields::independent_of_unready_fields(&cases.value)
                                || cases.arms.iter().any(|arm| {
                                    !promoted_fields::independent_of_unready_fields(&arm.label)
                                }))
                        {
                            index += 1;
                            continue;
                        }
                        self.compile_time_case_selection(&cases.header())
                            .and_then(|choice| {
                                cases.selected_body(choice).ok_or_else(|| {
                                    Diagnostic::new(
                                        cases.span,
                                        "compile-time case selection has no valid fallthrough body",
                                    )
                                })
                            })
                    }
                    _ => {
                        index += 1;
                        continue;
                    }
                };
                let chosen = match chosen {
                    Ok(chosen) => chosen,
                    Err(error) => {
                        pending_error.get_or_insert(error);
                        index += 1;
                        continue;
                    }
                };
                if (!chosen.is_empty() && depth == MAXIMUM_DEPTH)
                    || chosen.len() > MAXIMUM_MEMBERS - (selected.len() - 1)
                {
                    return Err(selection_budget(member_span(member)));
                }
                self.register_active_record_members(&chosen)?;
                let (_, promotion_error) =
                    self.prepare_record_promotions(chosen.iter(), &mut ready_promotions);
                if let Some(error) = promotion_error {
                    pending_promotions = true;
                    pending_error.get_or_insert(error);
                }
                // These braces do not create a lexical scope. The registry
                // retains each chosen declaration's original source identity.
                selected.splice(
                    index..index + 1,
                    chosen.into_iter().map(|member| (member, depth + 1)),
                );
                progress = true;
            }
            match pending_error {
                None => {
                    return Ok(selected.into_iter().map(|(member, _)| member).collect());
                }
                Some(error) if !progress => return Err(error),
                // An independent later selection can supply a forward constant.
                // Explicit #run guards use the shared checked readiness cache.
                Some(_) => {}
            }
        }
    }

    fn register_active_record_members(
        &mut self,
        members: &[syntax::RecordMember],
    ) -> Result<(), Diagnostic> {
        self.register_local_record_declarations(members)?;
        for member in members {
            if let syntax::RecordMember::Field(field) = member {
                // A record field shadows an outer constant even before its
                // layout is available; it cannot supply a pure guard value.
                self.declare_local_runtime_symbol(field.name);
            }
        }
        Ok(())
    }

    pub(crate) fn check_local_record_assertions(
        &mut self,
        members: &[syntax::RecordMember],
    ) -> Result<(), Diagnostic> {
        for member in members {
            if let syntax::RecordMember::Assert {
                condition, message, ..
            } = member
            {
                self.compile_time_assertion(condition, message.as_ref(), condition.span)?;
            }
        }
        Ok(())
    }
}

fn selection_budget(span: Span) -> Diagnostic {
    Diagnostic::new(
        span,
        "record member selection exceeds compiler declaration budget",
    )
}

fn member_span(member: &syntax::RecordMember) -> Span {
    use syntax::RecordMember as M;
    match member {
        M::Field(value) => value.span,
        M::AnonymousRecord(value) => value.span,
        M::Constant(value) => value.span,
        M::TypeAlias(value) => value.span,
        M::Procedure(value) => value.span,
        M::ProcedurePrototype(value) => value.span,
        M::Record(value) => value.span,
        M::Enum(value) => value.span,
        M::Insert(value) => value.span,
        M::CompileTimeCases { span, .. } => *span,
        M::Assert { span, .. } | M::Conditional { span, .. } | M::DefaultOverride { span, .. } => {
            *span
        }
    }
}
