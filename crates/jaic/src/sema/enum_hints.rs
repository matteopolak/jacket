//! Hints for enum declarations that do not fit: a value its type cannot hold and a member
//! declared twice.
use super::scope::EntityKind;
use super::{Compiler, Diagnostic, ScopeId, Span, Sym, TypeId};
use crate::source::{Applicability, Suggestion};

/// The name of the integer type with `bits` bits: `u8`, `s64`.
fn int_name(bits: u32, signed: bool) -> String {
    format!(
        "{}{bits}",
        if signed {
            "s"
        } else {
            "u"
        }
    )
}

/// The smallest and largest value an integer type holds, written for a note.
fn int_range(bits: u32, signed: bool) -> String {
    if signed {
        let top = 1i128 << (bits - 1);
        format!("{} to {}", -top, top - 1)
    } else {
        format!("0 to {}", (1i128 << bits) - 1)
    }
}

#[expect(
    clippy::unnecessary_box_returns,
    reason = "the diagnostics are boxed because the error type carries them boxed; callers pass them through `Err`"
)]
impl Compiler {
    /// The error for the enum value `value` that the backing type of `ty` cannot hold, written
    /// at `at`. When the enum names its type, the fix widens it to the narrowest that fits.
    pub(super) fn enum_value_out_of_range(
        &self,
        ty: TypeId,
        value: i128,
        (bits, signed): (u32, bool),
        at: Span,
    ) -> Box<Diagnostic> {
        let current = int_name(bits, signed);
        let mut d = Diagnostic::error(
            at,
            format!("enum value {value} is out of range for the enum's type `{current}`"),
        )
        .with_note(
            Span::NONE,
            format!("`{current}` holds {}", int_range(bits, signed)),
        );
        // The same signedness first; a negative value in an unsigned enum needs a signed type.
        let wider = [bits * 2, bits * 4, bits * 8]
            .into_iter()
            .filter(|&b| b <= 64)
            .map(|b| (b, signed));
        let signed_instead = std::iter::once(bits)
            .chain([bits * 2, bits * 4, bits * 8])
            .filter(|&b| b <= 64)
            .map(|b| (b, true));
        let fits = |&(b, s): &(u32, bool)| {
            u8::try_from(b).is_ok_and(|b| super::convert::int_constant_fits(value, b, s, false))
        };
        let better = wider.clone().find(fits).or_else(|| {
            (!signed)
                .then(|| signed_instead.clone().find(fits))
                .flatten()
        });
        let Some((b, s)) = better else {
            return Box::new(d.with_help("no integer type holds this value"));
        };
        let name = int_name(b, s);
        let help = format!("widen the enum's type to `{name}`");
        d = match self.ty.enum_base_spans.get(&ty) {
            Some(&base) => d.with_suggestion(
                Suggestion::replace(help, base, name.clone(), Applicability::MaybeIncorrect)
                    .with_action(format!("Widen the enum to `{name}`")),
            ),
            None => d.with_help(format!(
                "give the enum a wider backing type, such as `{name}`"
            )),
        };
        Box::new(d)
    }

    /// The error for a second enum member called `name`, written at `span`, with where the
    /// first one is.
    pub(super) fn duplicate_enum_member(
        &self,
        scope: ScopeId,
        name: Sym,
        span: Span,
    ) -> Box<Diagnostic> {
        let mut d = Diagnostic::error(
            span,
            format!("enum member `{name}` is declared more than once"),
        )
        .with_label("declared again here");
        let first = self.scope(scope).names.get(&name).and_then(|ids| {
            ids.iter()
                .map(|&id| self.entity(id))
                .find(|e| matches!(e.kind, EntityKind::EnumMember { .. }))
        });
        if let Some(first) = first
            && first.span != Span::NONE
            && first.span != span
        {
            d = d.with_note(first.span, format!("`{name}` is first declared here"));
        }
        Box::new(d)
    }
}
