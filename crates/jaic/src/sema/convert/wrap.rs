//! Suggestions that wrap an expression in what converts it: `cast(T)`, `.*`, `*` or `!= 0`.
//! One helper builds the edit, so every conversion error (an assignment, an argument, an
//! operand, an explicit cast) rewrites the code the same way. See docs/compiler/diagnostics.md.
use crate::sema::{Compiler, Diagnostic, Span, TypeId};
use crate::source::{Applicability, Edit, SourceMap, Suggestion};
use crate::types::TypeKind;

use super::super::TypeState;

/// What to put around an expression.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::sema) enum Wrap {
    /// `cast(T) expr`.
    Cast(String),
    /// `expr.*`: the value a pointer points to.
    Deref,
    /// `*expr`: the address of a value.
    AddressOf,
    /// `expr != 0`: a number as a condition.
    NotZero,
}

impl Wrap {
    /// `cast(T)`.
    pub(in crate::sema) fn cast(ty: impl Into<String>) -> Self {
        Self::Cast(ty.into())
    }
}

/// Whether `text` is one primary expression (a name, a literal, a member path, a call or an
/// index of those), so a prefix or a postfix operator applies to all of it without
/// parentheses.
pub(in crate::sema) fn is_primary(text: &str) -> bool {
    let bytes = text.as_bytes();
    let Some(&first) = bytes.first() else {
        return false;
    };
    if !(first.is_ascii_alphanumeric() || first == b'_') {
        return false;
    }
    // Names, numbers and `.`, then any number of balanced `( )` / `[ ]` groups, each followed
    // by more path.
    let mut depth = 0u32;
    let mut in_string = false;
    for (i, &b) in bytes.iter().enumerate() {
        if in_string {
            in_string = b != b'"' || bytes[i - 1] == b'\\';
            continue;
        }
        match b {
            b'"' if depth > 0 => in_string = true,
            b'(' | b'[' => depth += 1,
            b')' | b']' => {
                let Some(d) = depth.checked_sub(1) else {
                    return false;
                };
                depth = d;
            }
            _ if depth > 0 => {}
            b'.' | b'_' => {}
            _ if b.is_ascii_alphanumeric() => {}
            _ => return false,
        }
    }
    depth == 0
        // `a.*` and `a.{...}` end in an operator or a brace: not a path.
        && !text.ends_with('.')
}

/// The span of `span` without surrounding whitespace, and its text.
fn trimmed(sources: &SourceMap, span: Span) -> Option<(Span, &str)> {
    let text = sources.snippet(span);
    let body = text.trim();
    if body.is_empty() {
        return None;
    }
    let lead = text.len() - text.trim_start().len();
    let start = span.start as usize + lead;
    Some((Span::new(span.file, start, start + body.len()), body))
}

/// The expression at `span` written inside `wrap`.
pub(in crate::sema) fn wrapped_text(
    sources: &SourceMap,
    span: Span,
    wrap: &Wrap,
) -> Option<String> {
    let (_, body) = trimmed(sources, span)?;
    let primary = is_primary(body);
    Some(match wrap {
        Wrap::Cast(ty) if primary => format!("cast({ty}) {body}"),
        Wrap::Cast(ty) => format!("cast({ty}) ({body})"),
        Wrap::Deref if primary => format!("{body}.*"),
        Wrap::Deref => format!("({body}).*"),
        Wrap::AddressOf if primary => format!("*{body}"),
        Wrap::AddressOf => format!("*({body})"),
        Wrap::NotZero if primary => format!("{body} != 0"),
        Wrap::NotZero => format!("({body}) != 0"),
    })
}

/// A suggestion that rewrites the expression at `span` into `wrap` of it, titled for the
/// `help:` line. Every wrap is a guess about what the author meant (a cast can drop bits, a
/// dereference can read a null pointer, an address needs a variable), so it is
/// [`Applicability::MaybeIncorrect`]. `None` when the expression has no source text (code
/// from a macro).
pub(in crate::sema) fn wrap_suggestion(
    sources: &SourceMap,
    title: impl Into<String>,
    span: Span,
    wrap: &Wrap,
) -> Option<Suggestion> {
    let (at, _) = trimmed(sources, span)?;
    let text = wrapped_text(sources, span, wrap)?;
    let action = match wrap {
        Wrap::Cast(ty) => format!("Convert with `cast({ty})`"),
        Wrap::Deref => "Dereference with `.*`".into(),
        Wrap::AddressOf => "Take the address with `*`".into(),
        Wrap::NotZero => "Compare with `!= 0`".into(),
    };
    Some(
        Suggestion::with_edits(
            title,
            vec![Edit::Replace {
                span: at,
                text,
            }],
            Applicability::MaybeIncorrect,
        )
        .with_action(action),
    )
}

impl Compiler {
    /// `d` with the advice for converting a `from` into a `to` (see
    /// [`TypeState::conversion_hint`]), as a rewrite of the expression at `span` when it is one.
    pub(in crate::sema) fn with_conversion_hint(
        &self,
        d: Diagnostic,
        from: TypeId,
        to: TypeId,
        span: Span,
    ) -> Diagnostic {
        let Some((help, wrap)) = self.ty.conversion_hint(from, to) else {
            return d;
        };
        match wrap.and_then(|w| wrap_suggestion(&self.src.sources, help.clone(), span, &w)) {
            Some(suggestion) => d.with_suggestion(suggestion),
            None => d.with_help(help),
        }
    }
}

impl TypeState {
    /// How to get a `to` from a `from`, for the common mismatches with an obvious fix: the
    /// advice, and the wrap that carries it out when it is a rewrite of the expression.
    pub(in crate::sema) fn conversion_hint(
        &self,
        from: TypeId,
        to: TypeId,
    ) -> Option<(String, Option<Wrap>)> {
        let to_name = self.types.name(to);
        let kind = |t| self.types.kind(t).clone();
        let cast = || Some(Wrap::cast(to_name.clone()));
        Some(match (kind(from), kind(to)) {
            (TypeKind::Float { .. }, TypeKind::Int { .. }) => (
                format!("convert with `cast({to_name})`, which drops the fraction"),
                cast(),
            ),
            (TypeKind::Int { .. }, TypeKind::Int { .. }) => (
                format!("convert with `cast({to_name})` if every value fits"),
                cast(),
            ),
            (TypeKind::Int { .. } | TypeKind::Float { .. }, TypeKind::String) => (
                "to turn a number into text, format it: `tprint(\"%\", value)` (from Basic)".into(),
                None,
            ),
            (TypeKind::String, TypeKind::Pointer(p)) if self.types.is_integer(p) => (
                "pass `value.data` for the bytes, or `to_c_string(value)` (from Basic) for a zero-terminated copy".into(),
                None,
            ),
            (TypeKind::Int { .. }, TypeKind::Bool) => (
                "compare instead: `value != 0`".into(),
                Some(Wrap::NotZero),
            ),
            (TypeKind::Int { .. }, TypeKind::Enum(_))
            | (TypeKind::Enum(_), TypeKind::Int { .. }) => {
                (format!("convert with `cast({to_name})`"), cast())
            }
            (TypeKind::Pointer(p), _) if p == to => (
                "this is a pointer to the value: dereference it with `value.*`".into(),
                Some(Wrap::Deref),
            ),
            (_, TypeKind::Pointer(p)) if p == from => (
                "this is the value itself: take its address with `*value`".into(),
                Some(Wrap::AddressOf),
            ),
            _ => return None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::is_primary;

    #[test]
    fn primary_expressions_need_no_parentheses() {
        for text in ["x", "a.b", "f(x, \"a)\")", "xs[i + 1]", "42", "a.b(c).d"] {
            assert!(is_primary(text), "{text}");
        }
        for text in ["a + b", "-x", "a.*", "f(x) + 1", "", "(a)", "x if y"] {
            assert!(!is_primary(text), "{text}");
        }
    }
}
