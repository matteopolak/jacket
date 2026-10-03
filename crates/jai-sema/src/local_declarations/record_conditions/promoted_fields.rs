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
        for member in members {
            let span = member_span(member);
            let site = (span.start, span.end);
            if ready.contains(&site) {
                continue;
            }
            let result = match member {
                syntax::RecordMember::AnonymousRecord(record) => self.local_inline_record(record),
                syntax::RecordMember::Field(field) if field.using => match &field.binding {
                    syntax::FieldBinding::Explicit { ty, .. } => {
                        self.lexical_annotation(ty, field.span)
                    }
                    syntax::FieldBinding::Inferred(expression) => self
                        .expr(expression)
                        .and_then(|value| self.expression_type(&value, expression.span)),
                },
                _ => continue,
            }
            .and_then(|ty| self.promoted_runtime_names(ty, span));
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

    fn promoted_runtime_names(
        &self,
        ty: TypeId,
        span: Span,
    ) -> Result<HashSet<Symbol>, Diagnostic> {
        let mut names = HashSet::new();
        let mut pending = vec![(ty, HashSet::new())];
        let mut visited = 0usize;
        while let Some((ty, mut ancestors)) = pending.pop() {
            if !ancestors.insert(ty) {
                return Err(Diagnostic::new(span, "cyclic using field promotion"));
            }
            visited += 1;
            if ancestors.len() > MAXIMUM_DEPTH || visited > MAXIMUM_MEMBERS {
                return Err(selection_budget(span));
            }
            let record = self.record_metadata(ty, span)?;
            for field in record.fields {
                if let Some(name) = field.name {
                    names.insert(name);
                }
                if field.syntax.using() {
                    pending.push((field.ty, ancestors.clone()));
                }
            }
        }
        Ok(names)
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
                pending.extend([value.condition.as_ref(), value.then_value.as_ref()]);
                if let Some(value) = &value.else_value {
                    pending.push(value);
                }
            }
            _ => return false,
        }
    }
    true
}
