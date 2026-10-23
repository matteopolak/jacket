//! Assigning arguments to parameter slots and the polymorphic-name helpers.
use super::{CallArg, Slot};
use crate::ast::ExprKind as E;
use crate::sema::lower::{FnCtx, Operand};
use crate::sema::{Compiler, Diagnostic, Result, Span, Sym, TypeId, ast, err};

thread_local! {
    /// Where the name of each named argument is written, by the span of its value: a
    /// `CallArg` keeps only the value's span, and a fix for a wrong name replaces just the name.
    static ARGUMENT_NAMES: std::cell::RefCell<crate::fxhash::HashMap<Span, Span>> =
        std::cell::RefCell::new(crate::fxhash::HashMap::default());
}

/// Remember that the argument whose value is at `value` is named at `name`.
pub(super) fn note_argument_name(value: Span, name: Span) {
    ARGUMENT_NAMES.with(|names| {
        names.borrow_mut().insert(value, name);
    });
}

/// C default argument promotions for variadic arguments.
pub(super) fn c_vararg_promote(
    c: &mut Compiler,
    f: &mut FnCtx,
    op: Operand,
    span: Span,
) -> Result<Operand> {
    let ty = op.ty();
    if ty == TypeId::F32 {
        return c.convert(f, op, TypeId::F64, span);
    }
    if let Some((bits, signed)) = c.ty.types.int_info(ty)
        && bits < 32
    {
        let target = if signed {
            TypeId::S32
        } else {
            TypeId::U32
        };
        return c.explicit_cast(f, op, target, ast::CastFlags::default(), span);
    }
    if ty == TypeId::BOOL {
        return c.explicit_cast(f, op, TypeId::S32, ast::CastFlags::default(), span);
    }
    Ok(op)
}

/// Polymorphic names introduced by a type pattern.
pub(super) fn poly_names(expr: &ast::Expr) -> Vec<Sym> {
    let mut out = Vec::new();
    fn walk(e: &ast::Expr, out: &mut Vec<Sym>) {
        match &e.kind {
            E::PolyVar {
                name, ..
            }
            | E::PolyRestricted {
                name, ..
            } => out.push(*name),
            E::Unary(_, x) => walk(x, out),
            E::ArrayType {
                size,
                elem,
            } => {
                if let ast::ArraySize::Fixed(n) = size {
                    walk(n, out);
                }
                walk(elem, out);
            }
            E::Call {
                args, ..
            } => args.iter().for_each(|a| walk(&a.value, out)),
            E::ProcType(h) => {
                h.params
                    .iter()
                    .filter_map(|p| p.ty.as_ref())
                    .for_each(|t| walk(t, out));
                h.returns
                    .iter()
                    .filter_map(|r| r.ty.as_ref())
                    .for_each(|t| walk(t, out));
            }
            _ => {}
        }
    }
    walk(expr, &mut out);
    out
}

/// Concrete type names and constructors constrain a pattern; binder names do not.
pub(super) fn pattern_specificity(expr: &ast::Expr, variables: &[Sym]) -> u32 {
    match &expr.kind {
        E::Ident(name) if !variables.contains(name) => 1,
        E::PolyRestricted {
            ..
        } => 1,
        E::Unary(_, x) => 1 + pattern_specificity(x, variables),
        E::ArrayType {
            elem, ..
        } => 1 + pattern_specificity(elem, variables),
        E::Call {
            args, ..
        } => {
            1 + args
                .iter()
                .map(|a| pattern_specificity(&a.value, variables))
                .sum::<u32>()
        }
        E::ProcType(_) => 1,
        _ => 0,
    }
}

/// Map call arguments onto declared parameters.
/// Every `$T` binder in a header's parameter and result types.
pub(in crate::sema) fn header_poly_names(header: &ast::ProcHeader) -> Vec<Sym> {
    let mut names = Vec::new();
    for t in header
        .params
        .iter()
        .filter_map(|p| p.ty.as_ref())
        .chain(header.returns.iter().filter_map(|r| r.ty.as_ref()))
    {
        names.extend(poly_names(t));
    }
    names
}

/// A named argument binding a polymorphic variable (`f(x, T = float64)`).
pub(super) fn is_poly_var_arg(params: &[ast::Param], arg: &CallArg, poly_vars: &[Sym]) -> bool {
    arg.name.is_some_and(|name| {
        poly_vars.contains(&name) && !params.iter().any(|p| p.name.map(|n| n.name) == Some(name))
    })
}

pub(super) fn assign_slots(
    params: &[ast::Param],
    args: &[CallArg],
    poly_vars: &[Sym],
    span: Span,
) -> Result<Vec<Slot>> {
    let mut slots: Vec<Option<Slot>> = vec![None; params.len()];
    let variadic_index = params.iter().position(|p| p.variadic);
    let mut positional = 0usize;
    for (i, arg) in args.iter().enumerate() {
        if is_poly_var_arg(params, arg, poly_vars) {
            continue;
        }
        if let Some(name) = arg.name {
            let Some(p) = params
                .iter()
                .position(|p| p.name.map(|n| n.name) == Some(name))
            else {
                let names: Vec<&str> = params
                    .iter()
                    .filter_map(|p| p.name.map(|n| n.name.as_str()))
                    .collect();
                let mut d = Diagnostic::error(arg.span, format!("no parameter named `{name}`"));
                match crate::suggest::closest(name.as_str(), names.iter().copied()) {
                    Some(near) => {
                        let written =
                            ARGUMENT_NAMES.with(|names| names.borrow().get(&arg.span).copied());
                        d = match written {
                            // Only the name changes, not the value after it.
                            Some(name_span) => d.with_replacement(
                                format!("did you mean `{near}`?"),
                                name_span,
                                near,
                            ),
                            None => d.with_help(format!("did you mean `{near}`?")),
                        };
                        if let Some(param) = params
                            .iter()
                            .find(|p| p.name.map(|n| n.name.as_str()) == Some(near))
                        {
                            d = d.with_note(
                                param.span,
                                format!("parameter `{near}` is declared here"),
                            );
                        }
                    }
                    None if names.is_empty() => {
                        d = d.with_help("this procedure has no named parameters");
                    }
                    None => {
                        d = d.with_help(format!(
                            "its parameters are {}",
                            names
                                .iter()
                                .map(|n| format!("`{n}`"))
                                .collect::<Vec<_>>()
                                .join(", ")
                        ));
                    }
                }
                return Err(Box::new(d));
            };
            if slots[p].is_some() {
                return Err(Box::new(
                    Diagnostic::error(arg.span, format!("parameter `{name}` given twice"))
                        .with_help("remove one of the two arguments"),
                ));
            }
            slots[p] = Some(if arg.spread {
                Slot::Spread(i)
            } else if Some(p) == variadic_index {
                // `v = a, b, c`: later positional arguments extend the list.
                Slot::Variadic(vec![i])
            } else {
                Slot::Arg(i)
            });
            // Positional arguments after a named one continue from the next parameter.
            positional = if Some(p) == variadic_index {
                p
            } else {
                p + 1
            };
            continue;
        }
        while positional < params.len()
            && slots[positional].is_some()
            && Some(positional) != variadic_index
        {
            positional += 1;
        }
        if positional >= params.len() {
            return Err(Box::new(
                Diagnostic::error(
                    arg.span,
                    format!("too many arguments: it takes at most {}", params.len()),
                )
                .with_label("this argument has no parameter"),
            ));
        }
        if Some(positional) == variadic_index {
            match &mut slots[positional] {
                Some(Slot::Variadic(list)) => list.push(i),
                Some(Slot::Spread(first)) => {
                    slots[positional] = Some(Slot::Variadic(vec![*first, i]));
                }
                None => {
                    slots[positional] = Some(if arg.spread {
                        Slot::Spread(i)
                    } else {
                        Slot::Variadic(vec![i])
                    });
                }
                _ => return err(arg.span, "variadic parameter already given"),
            }
            if arg.spread && positional + 1 < params.len() {
                positional += 1;
            }
            continue;
        }
        slots[positional] = Some(if arg.spread {
            Slot::Spread(i)
        } else {
            Slot::Arg(i)
        });
        positional += 1;
    }
    let _ = span;
    Ok(slots
        .into_iter()
        .map(|s| s.unwrap_or(Slot::Default))
        .collect())
}
