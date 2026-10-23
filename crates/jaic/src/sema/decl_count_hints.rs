//! Declarations and assignments that name a different number of variables than values.
use super::{Compiler, Diagnostic, Span, ast};
use crate::source::{Applicability, Suggestion};

/// `1 value`, `3 values`.
fn counted(count: usize, noun: &str) -> String {
    format!(
        "{count} {noun}{}",
        if count == 1 {
            ""
        } else {
            "s"
        }
    )
}

#[expect(
    clippy::unnecessary_box_returns,
    reason = "the diagnostics are boxed because the error type carries them boxed; callers pass them through `Err`"
)]
impl Compiler {
    /// `a, b := 1, 2, 3;` and its kin: `names` variables (or `targets`, called `noun`) given
    /// `count` values. `values` are the written values when the count comes from a list the
    /// source spells out, in which case extra ones can be deleted.
    pub(in crate::sema) fn count_mismatch(
        &self,
        span: Span,
        (names, noun): (usize, &str),
        count: usize,
        values: &[&ast::Expr],
    ) -> Box<Diagnostic> {
        let mut d = Diagnostic::error(
            span,
            format!("{names} {noun} but {}", counted(count, "value")),
        );
        if count < names {
            return Box::new(d.with_help(format!(
                "give each of the {names} {noun} its own value, or use fewer {noun}"
            )));
        }
        // Extra values: delete from the end of the last one that has a name.
        let usable = values.len() == count
            && values.len() > names
            && values
                .iter()
                .all(|v| v.span != Span::NONE && v.span.file == values[0].span.file);
        if !usable {
            return Box::new(d.with_help(format!(
                "remove the extra values, or add {} more {noun}",
                count - names
            )));
        }
        let last = values[count - 1].span;
        let start = values[names - 1].span.end;
        if start > last.end {
            return Box::new(d);
        }
        let span = Span {
            start,
            end: last.end,
            ..last
        };
        let extra = count - names;
        let (title, action) = if extra == 1 {
            ("remove the extra value", "Remove the extra value")
        } else {
            ("remove the extra values", "Remove the extra values")
        };
        d = d.with_suggestion(
            Suggestion::replace(title, span, "", Applicability::MaybeIncorrect).with_action(action),
        );
        Box::new(d)
    }
}
