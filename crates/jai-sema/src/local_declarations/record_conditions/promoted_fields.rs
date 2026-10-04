//! Runtime guard names come from selected, canonical physical field shapes.
use super::*;

impl Resolver<'_> {
    pub(super) fn prepare_record_promotions<'a>(
        &mut self,
        members: impl Iterator<Item = &'a syntax::RecordMember>,
        ready: &mut HashSet<(usize, usize)>,
    ) -> (bool, Option<Diagnostic>) {
        let mut progress = false;
        let mut pending = None;
        let members: Vec<_> = members.collect();
        for member in &members {
            let member = *member;
            if !matches!(
                member,
                syntax::RecordMember::AnonymousRecord(_) | syntax::RecordMember::Using(_)
            ) && !matches!(member, syntax::RecordMember::Field(field) if field.using)
            {
                continue;
            }
            let span = member_span(member);
            let site = (span.start, span.end);
            if ready.contains(&site) {
                continue;
            }
            let result = (|| -> Result<HashSet<Symbol>, Diagnostic> {
                let (ty, selection) = match member {
                    syntax::RecordMember::AnonymousRecord(record) => {
                        (self.local_inline_record(record)?, None)
                    }
                    syntax::RecordMember::Field(field) if field.using => (
                        self.promotion_source_type(field)?,
                        Some(&field.using_selection),
                    ),
                    syntax::RecordMember::Using(directive) => {
                        let names = crate::record_using::target_names(&directive.target)?;
                        let root = members.iter().find_map(|member| match member {
                            syntax::RecordMember::Field(field) if field.name == names[0] => {
                                Some(field)
                            }
                            _ => None,
                        });
                        if let Some(field) = root {
                            let mut ty = self.promotion_source_type(field)?;
                            for name in &names[1..] {
                                for id in self.field_path(ty, *name, directive.span)? {
                                    ty = self.types.validate_field(ty, id).map_err(|error| {
                                        Diagnostic::new(directive.span, error.to_string())
                                    })?;
                                }
                            }
                            (ty, Some(&directive.selection))
                        } else {
                            // A genuine type namespace has no runtime storage setup.
                            let target = self.expr(&directive.target)?;
                            if !matches!(target, Expr::Type(_)) {
                                return Err(Diagnostic::new(
                                    directive.span,
                                    "record namespace using requires a checked type target",
                                ));
                            }
                            self.using_directive(directive)?;
                            return Ok(HashSet::new());
                        }
                    }
                    _ => return Ok(HashSet::new()),
                };
                let names = self.promoted_runtime_names(ty, span)?;
                match selection {
                    None => Ok(names),
                    Some(selection) => {
                        let mut visible = HashSet::new();
                        for name in names {
                            if crate::record_using::selected(selection, name, span)? {
                                visible.insert(name);
                            }
                        }
                        Ok(visible)
                    }
                }
            })();
            match result {
                Ok(names) => {
                    for name in names {
                        self.declare_local_runtime_symbol(name);
                    }
                    ready.insert(site);
                    progress = true;
                }
                Err(error) => {
                    pending.get_or_insert(error);
                }
            }
        }
        (progress, pending)
    }

    fn promotion_source_type(
        &mut self,
        field: &syntax::FieldDeclaration,
    ) -> Result<TypeId, Diagnostic> {
        match &field.binding {
            syntax::FieldBinding::Explicit {
                ty, ..
            } => self.lexical_annotation(ty, field.span),
            syntax::FieldBinding::Inferred(expression) => self
                .expr(expression)
                .and_then(|value| self.expression_type(&value, expression.span)),
        }
    }

    fn promoted_runtime_names(
        &self,
        ty: TypeId,
        span: Span,
    ) -> Result<HashSet<Symbol>, Diagnostic> {
        crate::record_using::visible_names(ty, span, self.types, |ty| {
            self.record_metadata(ty, span)
        })
    }
}

/// A pending child can eventually introduce a runtime name. Until its actual
/// shape is known, only source operands that cannot bind names may advance.
/// This schedules readiness; it never assigns names from inactive source arms.
pub(super) fn independent_of_unready_fields(source: &syntax::Expression) -> bool {
    let mut pending = vec![source];
    while let Some(expression) = pending.pop() {
        use syntax::ExpressionKind as E;
        match &expression.kind {
            E::Integer(_)
            | E::Float(_)
            | E::String(_)
            | E::HereString(_)
            | E::Character(_)
            | E::Null
            | E::Bool(_)
            | E::CompileTimePredicate => {}
            E::Unary(_, value) | E::Cast(_, _, value) => pending.push(value),
            E::Binary(_, left, right) => pending.extend([left.as_ref(), right.as_ref()]),
            E::Conditional(value) => {
                pending.push(value.condition.as_ref());
                pending.extend(value.explicit_then());
                if let Some(value) = &value.else_value {
                    pending.push(value);
                }
            }
            _ => return false,
        }
    }
    true
}
