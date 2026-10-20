//! Calls: overload resolution, polymorphic inference, argument lowering,
//! builtin procedures, macro expansion and operator overloading.
//!
//! Layout:
//! - `mod.rs`        `check_call`, candidate ranking and matching, argument costs
//! - `infer`         `$T` inference and type-pattern matching
//! - `baking`        baked (`$x`) parameter values and constant packs
//! - `emit`          lowering a resolved call and its arguments
//! - `intrinsics`    `#intrinsic` procedures and atomics
//! - `builtins`      built-in procedures (`size_of`, `type_of`, ...)
//! - `macro_expand`  `#expand` macro calls
//! - `operators`     operator overloading
use super::lower::{FnCtx, MacroFrame, Operand};
use super::procs::{ProcTarget, Signature};
use super::scope::{BuiltinProc, EntityKind, ScopeKind};
use super::{
    Compiler, Diagnostic, DiagnosticKind, PackKey, PreludeType, ProcId, ProcTypeParams, Rc, Result,
    ScopeId, Span, Sym, TargetCpu, TypeId, Value, asm, ast, err, ir, procs, scope, value,
};
use crate::ast::ExprKind as E;
use crate::ir::Ty;
use crate::macros::raw_enum;
use crate::types::{ArrayKind, TypeKind};

mod baking;
mod builtins;
mod constants;
mod cost;
mod emit;
mod infer;
mod intrinsics;
mod macro_expand;
mod matching;
mod operators;
mod precheck;
mod slots;

pub(super) use self::slots::header_poly_names;
use self::slots::{c_vararg_promote, is_poly_var_arg, poly_names};
pub use operators::binop_text;

/// How a `$$` parameter is treated at one call.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AutoBake {
    Runtime,
    Baked,
    /// Constant argument with no `Value` (`"str".data`): runtime, but `is_constant` is true.
    ConstRuntime,
}

/// The hidden constant marking parameter `name` as constant in an auto-bake variant.
fn const_marker(name: Sym) -> Sym {
    Sym::intern(&format!("$const:{name}"))
}

/// A call argument: either already checked, or deferred until the parameter
/// type is known (`.MEMBER`, `.{...}`, `xx value`...).
#[derive(Clone)]
pub struct CallArg {
    pub name: Option<Sym>,
    pub spread: bool,
    pub expr: Option<ast::Expr>,
    pub op: Option<Operand>,
    pub span: Span,
    pub scope: ScopeId,
}

/// How a parameter receives its value.
#[derive(Clone, Debug)]
pub(crate) enum Slot {
    Arg(usize),
    Variadic(Vec<usize>),
    Spread(usize),
    Default,
}

struct Candidate {
    proc: ProcId,
    slots: Vec<Slot>,
    cost: u32,
    /// How much type structure a polymorphic header pins down (`*$T` beats `$T`).
    specificity: u32,
}

fn is_deferred(expr: &ast::Expr) -> bool {
    matches!(
        &expr.kind,
        E::InferredMember(_)
            | E::StructLit {
                ty: None,
                ..
            }
            | E::ArrayLit {
                ty: None,
                ..
            }
            | E::Cast {
                ty: None,
                ..
            }
            | E::Ifx { .. }
            | E::Lambda { .. }
    ) || inferred_flags(expr)
        || autocast_arithmetic(expr)
        || literal_arithmetic(expr)
}

/// `.{300, -1} * scale`: arithmetic whose left operand is an untyped struct literal builds
/// that literal as the parameter's type, so the operator is looked up on it (`ui_builder`).
fn literal_arithmetic(expr: &ast::Expr) -> bool {
    match &expr.kind {
        E::Binary(ast::BinOp::Add | ast::BinOp::Sub | ast::BinOp::Mul | ast::BinOp::Div, a, _) => {
            matches!(
                a.kind,
                E::StructLit {
                    ty: None,
                    ..
                }
            ) || literal_arithmetic(a)
        }
        _ => false,
    }
}

/// `xx a + 1`: arithmetic on an autocast and literals takes the parameter's type. Bitwise
/// operators take any other operand (`xx a1 | h1` with flag constants passed as a `u64`).
fn autocast_arithmetic(expr: &ast::Expr) -> bool {
    let (a, b) = match &expr.kind {
        E::Binary(ast::BinOp::Add | ast::BinOp::Sub | ast::BinOp::Mul | ast::BinOp::Div, a, b) => {
            (a, b)
        }
        E::Binary(ast::BinOp::BitOr | ast::BinOp::BitAnd | ast::BinOp::BitXor, a, b) => {
            let bare = |e: &ast::Expr| {
                matches!(
                    e.kind,
                    E::Cast {
                        ty: None,
                        ..
                    }
                )
            };
            if bare(a) || bare(b) {
                return true;
            }
            (a, b)
        }
        _ => return false,
    };
    let autocast = |e: &ast::Expr| {
        matches!(
            e.kind,
            E::Cast {
                ty: None,
                ..
            }
        ) || autocast_arithmetic(e)
    };
    let literal = |e: &ast::Expr| matches!(e.kind, E::Int(_) | E::Float(_) | E::Char(_));
    (autocast(a) && (literal(b) || autocast(b))) || (literal(a) && autocast(b))
}

/// `.A | .B`, `.A & ~.B`: enum flag arithmetic built only from inferred members.
fn inferred_flags(expr: &ast::Expr) -> bool {
    match &expr.kind {
        E::InferredMember(_) => true,
        E::Unary(ast::UnOp::BitNot, x) => inferred_flags(x),
        E::Binary(ast::BinOp::BitOr | ast::BinOp::BitAnd | ast::BinOp::BitXor, a, b) => {
            inferred_flags(a) && inferred_flags(b)
        }
        _ => false,
    }
}

raw_enum! {
    /// The `atomic_*_order` intrinsics, whose trailing parameter is a constant memory order.
    enum OrderedAtomic by name {
        Load = "atomic_load_order",
        Store = "atomic_store_order",
        Exchange = "atomic_exchange_order",
        Add = "atomic_fetch_add_order",
        Sub = "atomic_fetch_sub_order",
        And = "atomic_fetch_and_order",
        Or = "atomic_fetch_or_order",
        Xor = "atomic_fetch_xor_order",
        CompareExchange = "atomic_compare_exchange_order",
    }
}

impl Compiler {
    pub fn check_call(
        &mut self,
        f: &mut FnCtx,
        scope: ScopeId,
        callee: &ast::Expr,
        args: &[ast::Arg],
        expected: Option<TypeId>,
        span: Span,
    ) -> Result<Operand> {
        if args.iter().any(|a| a.context) {
            // `f(x,, allocator = temp)`: call with a modified copy of the context.
            let (overrides, plain): (Vec<ast::Arg>, Vec<ast::Arg>) =
                args.iter().cloned().partition(|a| a.context);
            let ctx = self.context_with_overrides(f, scope, &overrides, span)?;
            let saved = f.current_context();
            f.set_context(Some(ctx));
            let result = self.check_call(f, scope, callee, &plain, expected, span);
            f.set_context(saved);
            return result;
        }
        let callee_op = self.check_expr(f, scope, callee, None)?;
        match callee_op {
            Operand::Builtin(b) => self.check_builtin(f, scope, b, args, expected, span),
            Operand::PolyStruct(ps) => {
                let mut values = Vec::new();
                for (i, a) in args.iter().enumerate() {
                    let expected = if matches!(a.value.kind, E::InferredMember(_)) {
                        self.poly_struct_param_type(ps, i, a.name.map(|n| n.name))
                    } else {
                        None
                    };
                    values.push((
                        a.name.map(|n| n.name),
                        self.eval_const_or_run(scope, &a.value, expected)?,
                    ));
                }
                let ty = self.instantiate_struct(ps, values, span)?;
                self.ide_note_struct_arguments(ty, args);
                Ok(Operand::Type(ty))
            }
            Operand::Procs(procs) => {
                let as_code = self.macro_code_args(&procs, args);
                let discarded = self.discarded_args(&procs, args);
                let call_args =
                    self.precheck_args_deferring(f, scope, args, &as_code, &discarded)?;
                let wants_code = procs.iter().any(|&p| {
                    let header = &self.proc(p).lit.header;
                    header.params.iter().any(|p| {
                        p.default
                            .as_ref()
                            .is_some_and(|d| matches!(d.kind, E::CallerCode))
                    })
                });
                if !wants_code {
                    return self.call_procs(f, scope, &procs, call_args, expected, span);
                }
                let call = ast::Expr {
                    kind: E::Call {
                        callee: Box::new(callee.clone()),
                        args: args.to_vec(),
                        hint: ast::CallHint::None,
                    },
                    span,
                };
                self.pr.calls_in_flight.push((Rc::new(call), scope));
                let result = self.call_procs(f, scope, &procs, call_args, expected, span);
                self.pr.calls_in_flight.pop();
                result
            }
            Operand::Type(t) => {
                // `Type(x)` is not Jai; but `T.{}` etc are handled elsewhere.
                err(span, format!("cannot call type {}", self.ty.types.name(t)))
            }
            other => {
                let parameters = self.callable_operand_parameters(&other)?;
                let (ty, v) = self.rvalue(f, other, span)?;
                let call_args = self.precheck_args(f, scope, args)?;
                self.call_indirect(f, ty, v, parameters, call_args, span)
            }
        }
    }

    /// Resolve and emit a call to one of `procs`.
    pub fn call_procs(
        &mut self,
        f: &mut FnCtx,
        scope: ScopeId,
        procs: &[ProcId],
        mut args: Vec<CallArg>,
        expected: Option<TypeId>,
        span: Span,
    ) -> Result<Operand> {
        let _ = expected;
        // A multi-value call result passes its first value; when nothing accepts that, the
        // values expand into several arguments (kept for older corpus code).
        let mut spread_args = None;
        if args.len() == 1
            && let Some(Operand::Multi(values)) = &args[0].op
        {
            let a = args[0].clone();
            let value_arg = |value: &Operand| CallArg {
                name: None,
                spread: false,
                expr: None,
                op: Some(value.clone()),
                span: a.span,
                scope: a.scope,
            };
            spread_args = Some(values.iter().map(value_arg).collect::<Vec<_>>());
            args = vec![CallArg {
                name: a.name,
                ..value_arg(&values[0])
            }];
        }
        let (mut best, mut errors) = self.rank_candidates(procs, &args, span);
        // A candidate's own declaration is wrong, whichever overload the call picks.
        if let Some(i) = errors
            .iter()
            .position(|e| matches!(e.kind, DiagnosticKind::ModifyWithoutPolymorphs))
        {
            return Err(Box::new(errors.swap_remove(i)));
        }
        if best.is_empty()
            && let Some(spread) = spread_args
        {
            let (b, e) = self.rank_candidates(procs, &spread, span);
            if !b.is_empty() {
                (best, errors, args) = (b, e, spread);
            }
        }
        if best.is_empty() {
            if self.ed.is_live() {
                self.ide_note_unmatched_inferred(procs, &args, span);
            }
            let name = procs
                .first()
                .map(|&p| self.proc(p).name.to_string())
                .unwrap_or_default();
            if procs.len() == 1 {
                let error = errors.into_iter().next();
                return Err(Box::new(self.call_mismatch(procs[0], &name, error, span)));
            }
            let mut d = Diagnostic::error(
                span,
                format!("no overload of `{name}` matches these arguments"),
            );
            for (i, &p) in procs.iter().enumerate().take(8) {
                d = d.with_note(
                    self.proc(p).span,
                    errors.get(i).map(|e| e.message.clone()).unwrap_or_default(),
                );
            }
            return Err(Box::new(d));
        }
        if best.len() > 1 {
            // Prefer non-polymorphic and more specific candidates; otherwise take the first.
            best.sort_by_key(|c| {
                (
                    self.proc(c.proc).bindings.is_some() as u32,
                    std::cmp::Reverse(c.specificity),
                )
            });
        }
        // Several equally good candidates: the editor keeps the whole overload set.
        let ambiguous = best.len() > 1 && {
            let key = |c: &Candidate| (self.proc(c.proc).bindings.is_some(), c.specificity);
            key(&best[0]) == key(&best[1])
        };
        let chosen = best.swap_remove(0);
        self.check_format_call(span, chosen.proc, &chosen.slots, &args)?;
        if self.ed.is_live() {
            self.ide_note_call(span, procs, chosen.proc, &chosen.slots, &args, ambiguous);
            if f.pending_for_body
                .as_ref()
                .is_some_and(|body| body.call_span == span)
            {
                self.ide_note_for_keyword(span, chosen.proc, ambiguous);
            }
        }
        self.emit_call(f, scope, chosen, args, span)
    }
}
