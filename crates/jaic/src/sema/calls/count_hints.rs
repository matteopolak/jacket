//! What a call with the wrong number of arguments tells the author: the callee's parameters
//! and, for extra arguments, the edit that removes them.
use super::CallArg;
use crate::diagnostics::Code;
use crate::sema::{Compiler, Diagnostic, Span, TypeId};
use crate::source::{Applicability, Suggestion};

/// The longest signature a note quotes.
const LONGEST_SIGNATURE: usize = 100;

/// The suggestion that deletes `args[first..]`, the arguments past the last parameter. `None`
/// when a later argument is named (it may be meant) or the spans do not say where the
/// arguments are written.
pub(super) fn remove_extra_arguments(args: &[CallArg], first: usize) -> Option<Suggestion> {
    let extra = args.get(first..)?;
    let last = extra.last()?;
    if extra.iter().any(|a| a.name.is_some()) {
        return None;
    }
    // Delete from where the last accepted argument ends, which takes the comma with it.
    let start = match first.checked_sub(1) {
        Some(before) => args[before].span.end,
        None => extra[0].span.start,
    };
    let file = last.span.file;
    let written = |s: Span| s != Span::NONE && s.file == file;
    if !args[first.saturating_sub(1)..]
        .iter()
        .all(|a| written(a.span))
        || start > last.span.end
    {
        return None;
    }
    let span = Span {
        start,
        end: last.span.end,
        ..last.span
    };
    let (title, action) = if extra.len() == 1 {
        ("remove the extra argument", "Remove the extra argument")
    } else {
        ("remove the extra arguments", "Remove the extra arguments")
    };
    Some(Suggestion::replace(title, span, "", Applicability::MaybeIncorrect).with_action(action))
}

#[expect(
    clippy::unnecessary_box_returns,
    reason = "the diagnostics are boxed because the error type carries them boxed; callers pass them through `Err`"
)]
impl Compiler {
    /// A note on `d` with the parameters of the procedure whose header is at `header`.
    pub(super) fn with_signature_note(
        &self,
        d: Diagnostic,
        name: &str,
        header: Span,
    ) -> Diagnostic {
        let written = self.src.sources.snippet(header);
        let mut text = written.split_whitespace().collect::<Vec<_>>().join(" ");
        if text.is_empty() {
            return d;
        }
        if text.chars().count() > LONGEST_SIGNATURE {
            text = text.chars().take(LONGEST_SIGNATURE).collect::<String>() + "...";
        }
        d.with_note(Span::NONE, format!("`{name}` takes `{text}`"))
    }

    /// A note on a wrong-count error of a call through a procedure value: its type.
    pub(super) fn with_type_note(&self, d: Diagnostic, ty: TypeId) -> Box<Diagnostic> {
        if d.code != Code::ArgumentCount {
            return Box::new(d);
        }
        let note = format!("the procedure has type `{}`", self.ty.types.name(ty));
        Box::new(d.with_note(Span::NONE, note))
    }
}
