//! What a failed polymorph inference tells the author: where each `$T` is declared, which
//! argument failed to bind it or bound it differently, and how to give the type by hand. See
//! docs/compiler/polymorph-reporting.md.
use super::{CallArg, Compiler, Diagnostic, E, Slot, Span, Sym, TypeId, ast};

/// Where inference first bound each type variable: the argument and the type it gave.
pub(super) type Binders = Vec<(Sym, Span, TypeId)>;

/// How `bind_pattern` begins the message of two arguments that bound one variable differently.
pub(super) const CONFLICT: &str = "conflicting types for `$";

/// The variable a conflict diagnostic is about.
pub(super) fn conflicting_variable(d: &Diagnostic) -> Option<Sym> {
    let rest = d.message.strip_prefix(CONFLICT)?;
    Some(Sym::intern(rest.split('`').next()?))
}

/// The span of the `$name` that declares the variable in `pattern`, and whether it is an array
/// count (`[$N] T`) rather than a type.
fn declaration(pattern: &ast::Expr, name: Sym) -> Option<(Span, bool)> {
    match &pattern.kind {
        E::PolyVar {
            name: n, ..
        }
        | E::PolyRestricted {
            name: n, ..
        } => (*n == name).then_some((pattern.span, false)),
        E::Unary(_, x) => declaration(x, name),
        E::ArrayType {
            size,
            elem,
        } => {
            if let ast::ArraySize::Fixed(n) = size
                && let Some((span, _)) = declaration(n, name)
            {
                return Some((span, true));
            }
            declaration(elem, name)
        }
        E::Call {
            args, ..
        } => args.iter().find_map(|a| declaration(&a.value, name)),
        E::ProcType(h) => h
            .params
            .iter()
            .filter_map(|p| p.ty.as_ref())
            .chain(h.returns.iter().filter_map(|r| r.ty.as_ref()))
            .find_map(|t| declaration(t, name)),
        _ => None,
    }
}

impl Compiler {
    /// The pattern as written, on one line.
    fn pattern_text(&self, pattern: &ast::Expr) -> String {
        let written = self.src.sources.snippet(pattern.span);
        written.split_whitespace().collect::<Vec<_>>().join(" ")
    }

    /// Notes on a pattern that `arg`, of type `ty`, failed to match: the argument's type and
    /// the pattern. For two arguments that bound `$T` differently, a note at each with the
    /// type it gave.
    pub(super) fn with_pattern_notes(
        &self,
        mut d: Diagnostic,
        pattern: &ast::Expr,
        arg: &CallArg,
        ty: TypeId,
        binders: &Binders,
    ) -> Diagnostic {
        let shown = self.ty.types.name(ty);
        if let Some(name) = conflicting_variable(&d)
            && let Some(&(_, first, first_ty)) = binders.iter().find(|b| b.0 == name)
            && first != Span::NONE
            && arg.span != Span::NONE
        {
            d = d
                .with_note(
                    first,
                    format!(
                        "`${name}` comes from this argument, of type `{}`",
                        self.ty.types.name(first_ty)
                    ),
                )
                .with_note(
                    arg.span,
                    format!("this argument, of type `{shown}`, gives a different `${name}`"),
                )
                .with_help(
                    "convert one argument so both give the same type, for example with \
                     `cast(float) value`",
                );
            return d;
        }
        if arg.span != Span::NONE {
            d = d.with_note(
                arg.span,
                format!(
                    "argument has type `{shown}`, which does not match `{}`",
                    self.pattern_text(pattern)
                ),
            );
        }
        d
    }

    /// Notes on `$name` that no argument bound: the parameter that declares it, each argument
    /// passed for that parameter, and how to give the type by hand.
    pub(super) fn with_unbound_notes(
        &self,
        mut d: Diagnostic,
        header: &ast::ProcHeader,
        slots: &[Slot],
        args: &[CallArg],
        name: Sym,
    ) -> Diagnostic {
        let Some((i, param, pattern, (at, is_count))) =
            header.params.iter().enumerate().find_map(|(i, p)| {
                let t = p.ty.as_ref()?;
                Some((i, p, t, declaration(t, name)?))
            })
        else {
            return d;
        };
        let param_name = param
            .name
            .map_or(String::new(), |n| format!(" of `{}`", n.name));
        d = d.with_note(
            at,
            format!("`${name}` is declared here, in the type{param_name}"),
        );
        let passed: Vec<usize> = match &slots[i] {
            Slot::Arg(a) | Slot::Spread(a) => vec![*a],
            Slot::Variadic(list) => list.clone(),
            Slot::Default => Vec::new(),
        };
        for a in passed {
            let arg = &args[a];
            if arg.span == Span::NONE {
                continue;
            }
            let text = match &arg.op {
                Some(op) => format!(
                    "argument has type `{}`, which gave nothing for `${name}` in `{}`",
                    self.ty.types.name(op.ty()),
                    self.pattern_text(pattern)
                ),
                None => format!(
                    "argument has no type of its own, so it gave nothing for `${name}` in `{}`",
                    self.pattern_text(pattern)
                ),
            };
            d = d.with_note(arg.span, text);
        }
        let named = header
            .params
            .iter()
            .any(|p| p.name.is_some_and(|n| n.name == name));
        if !is_count && !named {
            d = d.with_help(format!("pass the type by name, for example `{name} = s64`"));
        }
        d
    }
}
