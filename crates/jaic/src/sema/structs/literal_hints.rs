//! What a positional struct literal with the wrong number of values tells the author: the
//! members it fills, in order, and which of them it left out or has no member for.
use crate::diagnostics::Code;
use crate::sema::{Compiler, Diagnostic, Span, Sym, TypeId, ast};
use crate::source::{Applicability, Suggestion};

/// The members as a list for a note: `` `a`, `b`, `c` ``.
fn listed(names: &[Sym]) -> String {
    names
        .iter()
        .map(|n| format!("`{n}`"))
        .collect::<Vec<_>>()
        .join(", ")
}

#[expect(
    clippy::unnecessary_box_returns,
    reason = "the diagnostics are boxed because the error type carries them boxed; callers pass them through `Err`"
)]
impl Compiler {
    /// A literal of `ty` that gives `given` of the `needed` positional values its members ask
    /// for. `members` are the members positional values fill.
    pub(super) fn too_few_values(
        &self,
        ty: TypeId,
        members: &[(Option<Sym>, TypeId, u64)],
        (given, needed): (usize, usize),
        span: Span,
    ) -> Box<Diagnostic> {
        let name = self.ty.types.name(ty);
        let asked: Vec<Sym> = members.iter().take(needed).filter_map(|m| m.0).collect();
        let missing: Vec<Sym> = members
            .iter()
            .take(needed)
            .skip(given)
            .filter_map(|m| m.0)
            .collect();
        let mut d = Diagnostic::new(
            Code::StructNotEnoughValues,
            span,
            "not enough values in this struct literal",
        );
        if !missing.is_empty() {
            d = d.with_label(format!(
                "no value for {}",
                if missing.len() == 1 {
                    format!("member {}", listed(&missing))
                } else {
                    format!("members {}", listed(&missing))
                }
            ));
        }
        if !asked.is_empty() {
            d = d.with_note(
                Span::NONE,
                format!(
                    "`{name}` needs a value for each of {} in this order; this literal gives {given}",
                    listed(&asked)
                ),
            );
        }
        Box::new(d.with_help(
            "give the missing values, or name the members you set (`.{ a = 1 }`) and leave the rest to their defaults",
        ))
    }

    /// A literal of `ty` with a value after the last of its `count` members.
    pub(super) fn too_many_values(
        &self,
        ty: TypeId,
        members: &[(Option<Sym>, TypeId, u64)],
        values: &[&ast::Arg],
        first_extra: usize,
    ) -> Box<Diagnostic> {
        let extra = values[first_extra].value.span;
        let name = self.ty.types.name(ty);
        let names: Vec<Sym> = members.iter().filter_map(|m| m.0).collect();
        let mut d = Diagnostic::new(
            Code::StructTooManyValues,
            extra,
            format!("too many values for `{name}`"),
        )
        .with_label("this value has no member");
        d = d.with_note(
            Span::NONE,
            if names.is_empty() {
                format!("`{name}` has {} members", members.len())
            } else {
                format!(
                    "`{name}` has {} member{}: {}",
                    members.len(),
                    if members.len() == 1 {
                        ""
                    } else {
                        "s"
                    },
                    listed(&names)
                )
            },
        );
        let tail = &values[first_extra..];
        let last = tail.last().map_or(extra, |a| a.value.span);
        let start = match first_extra.checked_sub(1) {
            Some(before) => values[before].value.span.end,
            None => extra.start,
        };
        if tail.iter().any(|a| a.name.is_some()) || start > last.end {
            return Box::new(d.with_help("remove the extra values"));
        }
        let (title, action) = if tail.len() == 1 {
            ("remove the extra value", "Remove the extra value")
        } else {
            ("remove the extra values", "Remove the extra values")
        };
        let span = Span {
            start,
            end: last.end,
            ..last
        };
        Box::new(d.with_suggestion(
            Suggestion::replace(title, span, "", Applicability::MaybeIncorrect).with_action(action),
        ))
    }
}
