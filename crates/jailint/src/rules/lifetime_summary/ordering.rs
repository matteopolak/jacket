//! Minimum/maximum selection is a property of the body, including componentwise
//! aggregate construction. Procedure spellings are irrelevant.
use super::{Event, HashMap, ProcId, Resolver, Summary};
use jaic::ast::{AssignOp, BinOp, Expr, ExprKind as E, ForOver, ProcLit, Stmt, StmtKind as S};
use jaic::intern::Sym;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::rules) enum Ordering {
    Minimum,
    Maximum,
}

fn identifier(e: &Expr) -> Option<Sym> {
    if let E::Ident(n) = e.kind {
        Some(n)
    } else {
        None
    }
}

fn type_name(e: &Expr) -> Option<Sym> {
    match &e.kind {
        E::Ident(n) => Some(*n),
        E::PolyVar {
            name, ..
        } => Some(*name),
        _ => None,
    }
}

fn single(s: &Stmt) -> &Stmt {
    if let S::Block(b) = &s.kind
        && b.stmts.len() == 1
    {
        &b.stmts[0]
    } else {
        s
    }
}

fn returned(s: &Stmt) -> Option<Sym> {
    let S::Return {
        values,
        backtick: false,
    } = &single(s).kind
    else {
        return None;
    };
    if values.len() != 1 {
        return None;
    }
    identifier(&values[0].value)
}

fn choice(cond: &Expr, selected: Sym, other: Sym) -> Option<Ordering> {
    let E::Binary(op, a, b) = &cond.kind else {
        return None;
    };
    let a = identifier(a)?;
    let b = identifier(b)?;
    let left = if a == selected && b == other {
        true
    } else if b == selected && a == other {
        false
    } else {
        return None;
    };
    match (op, left) {
        (BinOp::Lt | BinOp::Le, true) | (BinOp::Gt | BinOp::Ge, false) => Some(Ordering::Minimum),
        (BinOp::Gt | BinOp::Ge, true) | (BinOp::Lt | BinOp::Le, false) => Some(Ordering::Maximum),
        _ => None,
    }
}

pub(super) fn scalar(proc: &ProcLit) -> Option<Ordering> {
    let body = &proc.body.as_ref()?.stmts;
    if !(1..=3).contains(&body.len()) || proc.header.returns.len() != 1 {
        return None;
    }
    let mut parameters = proc.header.params.iter().filter(|p| !p.baked);
    let a = parameters.next()?;
    let b = parameters.next()?;
    if parameters.next().is_some() {
        return None;
    }
    let a_name = a.name?.name;
    let b_name = b.name?.name;
    let result = proc.header.returns.first()?;
    if proc.header.returns.len() != 1
        || type_name(a.ty.as_ref()?)? != type_name(b.ty.as_ref()?)?
        || type_name(result.ty.as_ref()?)? != type_name(a.ty.as_ref()?)?
    {
        return None;
    }
    if let [first, last] = body.as_slice()
        && let S::If {
            cond,
            then_branch,
            else_branch: None,
        } = &first.kind
    {
        let yes = returned(then_branch)?;
        let no = returned(last)?;
        if (yes == a_name && no == b_name) || (yes == b_name && no == a_name) {
            return choice(cond, yes, no);
        }
    }
    if let [only] = body.as_slice()
        && let S::If {
            cond,
            then_branch,
            else_branch: Some(other),
        } = &only.kind
    {
        let yes = returned(then_branch)?;
        let no = returned(other)?;
        if (yes == a_name && no == b_name) || (yes == b_name && no == a_name) {
            return choice(cond, yes, no);
        }
    }
    if !b.variadic {
        return None;
    }
    let [initial, iteration, finish] = body.as_slice() else {
        return None;
    };
    let S::Decl(d) = &initial.kind else {
        return None;
    };
    if d.names.len() != 1 || d.backtick || d.using || identifier(d.value.as_ref()?)? != a_name {
        return None;
    }
    let accumulator = d.names[0].name;
    if returned(finish) != Some(accumulator) {
        return None;
    }
    let S::For(f) = &iteration.kind else {
        return None;
    };
    let ForOver::Collection(over) = &f.over else {
        return None;
    };
    if identifier(over) != Some(b_name)
        || f.by_pointer
        || f.pointer_if.is_some()
        || f.iterator.is_some()
    {
        return None;
    }
    let element = f.it.map_or_else(|| Sym::intern("it"), |n| n.name);
    let S::If {
        cond,
        then_branch,
        else_branch: None,
    } = &single(&f.body).kind
    else {
        return None;
    };
    let S::Assign {
        op: AssignOp::Assign,
        lhs,
        rhs,
    } = &single(then_branch).kind
    else {
        return None;
    };
    if lhs.len() != 1
        || rhs.len() != 1
        || identifier(&lhs[0]) != Some(accumulator)
        || identifier(&rhs[0]) != Some(element)
    {
        return None;
    }
    choice(cond, element, accumulator)
}

impl Resolver<'_> {
    pub(super) fn aggregate_order(&self, all: &HashMap<ProcId, Summary>) -> Option<Ordering> {
        let sig = self.compiler.pr.procs[self.body.procedure.0 as usize]
            .sig
            .as_ref()?;
        let [result] = sig.returns.as_slice() else {
            return None;
        };
        if sig.params.len() != 2 || sig.params.iter().any(|p| p.ty != *result) {
            return None;
        }
        let st = self.compiler.ty.types.as_struct(*result)?;
        let fields: Vec<_> = self
            .compiler
            .ty
            .types
            .struct_info(st)
            .fields
            .iter()
            .filter(|f| !f.overlay)
            .collect();
        let [statement] = self.body.syntax.body.as_ref()?.stmts.as_slice() else {
            return None;
        };
        let S::Return {
            values,
            backtick: false,
        } = &statement.kind
        else {
            return None;
        };
        let [returned] = values.as_slice() else {
            return None;
        };
        let E::StructLit {
            fields: values, ..
        } = &returned.value.kind
        else {
            return None;
        };
        if values.is_empty() || values.len() != fields.len() {
            return None;
        }
        let parameters = &self.body.syntax.header.params;
        if parameters.len() != 2 {
            return None;
        }
        let names = [parameters[0].name?.name, parameters[1].name?.name];
        let mut selected = None;
        for (i, expression) in values.iter().enumerate() {
            let field = expression.name.map_or(fields[i].name, |n| Some(n.name))?;
            if !fields.iter().any(|f| f.name == Some(field)) {
                return None;
            }
            let E::Call {
                args, ..
            } = &expression.value.kind
            else {
                return None;
            };
            if args.len() != 2 {
                return None;
            }
            let input = |e: &Expr| match &e.kind {
                E::Member(base, name) if name.name == field => identifier(base),
                _ => None,
            };
            let a = input(&args[0].value)?;
            let b = input(&args[1].value)?;
            if !((a == names[0] && b == names[1]) || (a == names[1] && b == names[0])) {
                return None;
            }
            let mut calls =
                self.body
                    .blocks
                    .iter()
                    .flat_map(|b| &b.events)
                    .filter_map(|e| match e {
                        Event::Call {
                            callee,
                            span,
                            expands: false,
                            ..
                        } if *span == expression.value.span => all.get(callee)?.ordering,
                        _ => None,
                    });
            let order = calls.next()?;
            if calls.any(|o| o != order) || selected.is_some_and(|old| old != order) {
                return None;
            }
            selected = Some(order);
        }
        selected
    }
}
