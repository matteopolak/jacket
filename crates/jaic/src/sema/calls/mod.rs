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
use super::expr::takes_context_type;
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
    /// How many members the literal arguments leave to their defaults: fewer fits better.
    fit: u32,
}

/// An argument that names no type of its own: one still waiting for its parameter's type
/// (`.{...}`, `.[...]`, `.NAME`) or `null`. An untyped number is not one, as every integer type
/// takes it.
fn is_untyped_argument(arg: &CallArg) -> bool {
    match &arg.op {
        None => arg.expr.as_ref().is_some_and(takes_context_type),
        Some(Operand::Const {
            value: Value::Null,
            untyped: true,
            ..
        }) => true,
        Some(_) => false,
    }
}

fn is_deferred(expr: &ast::Expr) -> bool {
    takes_context_type(expr)
        || matches!(
            &expr.kind,
            E::Cast {
                ty: None,
                ..
            } | E::Ifx { .. }
                | E::Lambda { .. }
        )
        || inferred_flags(expr)
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

    /// What still separates equally cheap candidates: a non-polymorphic one beats a polymorphic
    /// one, one that takes every argument as itself beats one that gathers some into a variadic
    /// parameter, and a more specific header beats a looser one.
    fn candidate_rank(&self, c: &Candidate) -> (bool, bool, std::cmp::Reverse<u32>) {
        let gathers = c
            .slots
            .iter()
            .any(|slot| matches!(slot, Slot::Variadic(list) if !list.is_empty()));
        (
            self.proc(c.proc).bindings.is_some(),
            gathers,
            std::cmp::Reverse(c.specificity),
        )
    }

    /// Sort equally cheap candidates so the preferred one is first, and say whether the first
    /// two are still equally good. Only the candidates tied with the first stay in `best`'s
    /// leading run.
    fn order_candidates(&self, best: &mut [Candidate]) -> bool {
        if best.len() > 1 {
            best.sort_by_key(|c| self.candidate_rank(c));
        }
        best.len() > 1 && self.candidate_rank(&best[0]) == self.candidate_rank(&best[1])
    }

    /// Two or more candidates that take the arguments equally well, and none better: the call
    /// does not say which one it means.
    fn ambiguous_overloads(&self, best: &[Candidate], span: Span) -> Diagnostic {
        let name = self.proc(best[0].proc).name.to_string();
        let mut d = Diagnostic::error(
            span,
            format!(
                "Procedure call matches {} possible overloads: `{name}` is ambiguous for these arguments.",
                best.len()
            ),
        );
        for c in best {
            d = d.with_note(self.proc(c.proc).span, "this overload matches");
        }
        d
    }

    /// The best of the ranked candidates, and whether another is as good (the editor keeps the
    /// whole overload set then). Equally good candidates declared together as different
    /// procedures are an error when the call does not say which one it means: an argument has no
    /// type of its own, or the candidates are polymorphic (an untyped number fits every numeric
    /// parameter alike, and declaration order settles that). One in an inner scope is nearer, as
    /// an inner declaration is to a name of an outer one, and two candidates of one declaration
    /// are not rivals.
    fn choose_candidate(
        &self,
        mut best: Vec<Candidate>,
        args: &[CallArg],
        span: Span,
    ) -> Result<(Candidate, bool)> {
        let ambiguous = self.order_candidates(&mut best);
        if ambiguous {
            let rank = self.candidate_rank(&best[0]);
            let home = self.proc(best[0].proc).scope;
            // Candidates of another shape are not rivals: `f(array)` and `f(array, ..items)`
            // called with one argument leave the variadic empty, and `(d: [] u8)` is closer to
            // `[] u8` than `(d: [] $T)` is.
            let shape = |c: &Candidate| {
                let header = &self.proc(c.proc).lit.header;
                (
                    header.params.iter().any(|p| p.variadic),
                    header_poly_names(header).len(),
                )
            };
            let first = shape(&best[0]);
            let tied = best
                .iter()
                .take_while(|c| {
                    self.candidate_rank(c) == rank
                        && self.proc(c.proc).scope == home
                        && shape(c) == first
                })
                .count();
            let declared = self.proc(best[0].proc).span;
            let rivals = best[..tied]
                .iter()
                .any(|c| self.proc(c.proc).span != declared);
            if rivals && (rank.0 || args.iter().any(is_untyped_argument)) {
                return Err(Box::new(self.ambiguous_overloads(&best[..tied], span)));
            }
        }
        Ok((best.swap_remove(0), ambiguous))
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
        let (chosen, ambiguous) = self.choose_candidate(best, &args, span)?;
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
