//! Operator overloading: finding and calling `operator` procedures.

use super::{
    CallArg, Compiler, FnCtx, Operand, ProcId, Result, ScopeId, Span, Sym, TypeId, TypeKind, Value,
    ast, err, ir, scope,
};
use crate::ast::ExprKind as E;
use crate::sema::TypeState;
use crate::sema::binary::Binary;
use crate::sema::expr::takes_sibling_type;

/// One way to call the `operator` overloads of a binary operator.
struct OperatorAttempt {
    procs: Vec<ProcId>,
    args: Vec<CallArg>,
    /// `!=` through `==`: the result is negated.
    negate: bool,
    /// The operands of a `#symmetric` overload are exchanged.
    swapped: bool,
}

/// An already checked operand as a call argument.
fn operand_arg(scope: ScopeId, span: Span, op: &Operand) -> CallArg {
    CallArg {
        name: None,
        spread: false,
        expr: None,
        op: Some(op.clone()),
        span,
        scope,
    }
}

impl Compiler {
    /// Operators obey ordinary visibility and filtered reexports. A canonical
    /// operand type alias does not import the defining scope's overloads.
    pub(in crate::sema) fn operator_candidates(
        &mut self,
        scope: ScopeId,
        text: &str,
    ) -> Result<Vec<ProcId>> {
        let name = Sym::intern(&format!("operator{text}"));
        let ids = self.lookup(scope, name)?;
        if ids.len() > 1 {
            self.check_identical_overloads(&ids)?;
        }
        let mut procs = Vec::new();
        for id in ids {
            let found = match self.resolve_entity(id)? {
                scope::Resolved::Proc(p)
                | scope::Resolved::Const {
                    value: Value::Proc(p),
                    ..
                } => vec![p],
                scope::Resolved::ProcSet(set) => set,
                _ => continue,
            };
            for p in found {
                if !procs.contains(&p) {
                    procs.push(p);
                }
            }
        }
        Ok(procs)
    }

    pub fn try_binary_operator_overload(
        &mut self,
        f: &mut FnCtx,
        scope: ScopeId,
        op: ast::BinOp,
        lhs: &Operand,
        rhs: &Operand,
        span: Span,
    ) -> Result<Option<Operand>> {
        let (lt, rt) = (lhs.ty(), rhs.ty());
        if !(self.ty.overloadable(lt) || self.ty.overloadable(rt))
            || matches!(lhs, Operand::Type(_))
        {
            return Ok(None);
        }
        if (self.ty.types.is_pointer(lt) || matches!(self.ty.types.kind(lt), TypeKind::Enum(_)))
            && !matches!(self.ty.types.kind(rt), TypeKind::Struct(_))
        {
            return Ok(None);
        }
        let args = vec![operand_arg(scope, span, lhs), operand_arg(scope, span, rhs)];
        let reports_failure = matches!(self.ty.types.kind(lt), TypeKind::Struct(_))
            || matches!(self.ty.types.kind(rt), TypeKind::Struct(_));
        self.call_operator(f, scope, op, args, reports_failure, span)
    }

    /// `.{...} op x` and `x op .{...}`: an operand with no type of its own next to a checked one
    /// takes the type its parameter has in the best `operator` overload for the pair, as the
    /// argument of a call does. The overloads are ranked with the other operand checked but not
    /// emitted; the operands are then checked for real, the literal as that type. `None` when
    /// there is no such operand or no overload takes the pair, and the operands are checked as
    /// for a builtin operator.
    pub(in crate::sema) fn overload_typed_operands(
        &mut self,
        f: &mut FnCtx,
        scope: ScopeId,
        node: Binary<'_>,
        span: Span,
    ) -> Result<Option<(Operand, Operand)>> {
        let Binary {
            op,
            a,
            b,
        } = node;
        let (literal_first, literal, other) = if takes_sibling_type(a, b) {
            (true, a, b)
        } else if takes_sibling_type(b, a) {
            (false, b, a)
        } else {
            return Ok(None);
        };
        if self.operator_candidates(scope, binop_text(op))?.is_empty()
            && (op != ast::BinOp::Ne || self.operator_candidates(scope, "==")?.is_empty())
        {
            return Ok(None);
        }
        let mut scratch = self.scratch_ctx(scope);
        // An operand that fails here fails again, with its own message, as a builtin operand.
        let Ok(probe) = self.check_expr(&mut scratch, scope, other, None) else {
            return Ok(None);
        };
        let deferred = CallArg {
            name: None,
            spread: false,
            expr: Some(literal.clone()),
            op: None,
            span: literal.span,
            scope,
        };
        let checked = operand_arg(scope, span, &probe);
        let args = if literal_first {
            vec![deferred, checked]
        } else {
            vec![checked, deferred]
        };
        let attempts = self.operator_attempts(scope, op, &args)?;
        if attempts.is_empty() {
            return Ok(None);
        }
        if matches!(probe, Operand::Type(_)) {
            return err(
                span,
                "a type cannot be an operand of an overloaded operator",
            );
        }
        let position = usize::from(!literal_first);
        let mut matched = false;
        let mut taken = None;
        for attempt in &attempts {
            let (best, _) = self.rank_candidates(&attempt.procs, &attempt.args, span);
            if best.is_empty() {
                continue;
            }
            matched = true;
            let (chosen, _) = self.choose_candidate(best, &attempt.args, span)?;
            // The literal is at the other position when the operands are swapped.
            let at = if attempt.swapped {
                1 - position
            } else {
                position
            };
            taken = self.argument_param_type(&chosen, at, span)?;
            break;
        }
        // `.{...}` and `.[...]` build aggregates, which only an overload takes; `.NAME` may be
        // a member of an enum a builtin operator compares.
        if !matched && !matches!(literal.kind, E::InferredMember(_)) {
            let first = attempts.into_iter().next().expect("attempts is not empty");
            return self
                .call_procs(f, scope, &first.procs, first.args, None, span)
                .map(|_| None);
        }
        let Some(ty) = taken else {
            return Ok(None);
        };
        let other = self.check_expr(f, scope, other, None)?;
        let literal = self.check_expr(f, scope, literal, Some(ty))?;
        Ok(Some(if literal_first {
            (literal, other)
        } else {
            (other, literal)
        }))
    }

    /// The ways to call the `operator` overloads of `op` on `args`, best first: a plain match,
    /// then `!=` as the negated `==`, then a `#symmetric` overload with the operands swapped.
    fn operator_attempts(
        &mut self,
        scope: ScopeId,
        op: ast::BinOp,
        args: &[CallArg],
    ) -> Result<Vec<OperatorAttempt>> {
        let procs = self.operator_candidates(scope, binop_text(op))?;
        let mut attempts = Vec::new();
        if !procs.is_empty() {
            attempts.push(OperatorAttempt {
                procs: procs.clone(),
                args: args.to_vec(),
                negate: false,
                swapped: false,
            });
        }
        // `a != b` without a matching `operator !=` is `!(a == b)`.
        if op == ast::BinOp::Ne {
            let equal = self.operator_candidates(scope, "==")?;
            if !equal.is_empty() {
                attempts.push(OperatorAttempt {
                    procs: equal,
                    args: args.to_vec(),
                    negate: true,
                    swapped: false,
                });
            }
        }
        let symmetric: Vec<ProcId> = procs
            .into_iter()
            .filter(|&p| self.proc(p).lit.header.flags.symmetric())
            .collect();
        if !symmetric.is_empty() {
            attempts.push(OperatorAttempt {
                procs: symmetric,
                args: vec![args[1].clone(), args[0].clone()],
                negate: false,
                swapped: true,
            });
        }
        Ok(attempts)
    }

    /// Call the `operator` overloads of `op` with checked `args` by the first attempt that
    /// takes them. A failing emission counts as no match.
    fn call_operator(
        &mut self,
        f: &mut FnCtx,
        scope: ScopeId,
        op: ast::BinOp,
        args: Vec<CallArg>,
        reports_failure: bool,
        span: Span,
    ) -> Result<Option<Operand>> {
        let attempts = self.operator_attempts(scope, op, &args)?;
        for attempt in attempts {
            let (best, _) = self.rank_candidates(&attempt.procs, &attempt.args, span);
            if best.is_empty() {
                continue;
            }
            let (chosen, _) = self.choose_candidate(best, &attempt.args, span)?;
            let Ok(r) = self.emit_call(f, scope, chosen, attempt.args, span) else {
                continue;
            };
            if !attempt.negate {
                return Ok(Some(r));
            }
            let (ty, v) = self.rvalue(f, r, span)?;
            let b = self.truthy(f, ty, v, span)?;
            let one = f.b.iconst(ir::Ty::I8, 1);
            return Ok(Some(Operand::Value {
                ty: TypeId::BOOL,
                val: f.b.bin(ir::BinOp::Xor, ir::Ty::I8, b, one),
            }));
        }
        let procs = self.operator_candidates(scope, binop_text(op))?;
        if reports_failure && !procs.is_empty() {
            // Report the overload failure for struct operands.
            return self
                .call_procs(f, scope, &procs, args, None, span)
                .map(Some);
        }
        Ok(None)
    }

    /// Does any of `procs` accept `args`? Nothing is emitted.
    pub(in crate::sema) fn any_accepts(
        &mut self,
        procs: &[ProcId],
        args: &[CallArg],
        span: Span,
    ) -> bool {
        procs
            .iter()
            .any(|&p| self.match_candidate(p, args, span).is_ok())
    }

    /// Call the best of `procs` for `args`, ranked as for an ordinary call.
    pub(in crate::sema) fn try_call(
        &mut self,
        f: &mut FnCtx,
        scope: ScopeId,
        procs: &[ProcId],
        args: Vec<CallArg>,
        span: Span,
    ) -> Result<Operand> {
        let (best, _) = self.rank_candidates(procs, &args, span);
        if best.is_empty() {
            return err(span, "no operator overload matches");
        }
        let (chosen, _) = self.choose_candidate(best, &args, span)?;
        self.emit_call(f, scope, chosen, args, span)
    }

    pub fn try_unary_operator_overload(
        &mut self,
        f: &mut FnCtx,
        scope: ScopeId,
        op: ast::UnOp,
        operand: &Operand,
        span: Span,
    ) -> Result<Option<Operand>> {
        if !matches!(
            self.ty.types.kind(self.ty.types.repr_struct(operand.ty())),
            TypeKind::Struct(_)
        ) {
            return Ok(None);
        }
        let text = match op {
            ast::UnOp::Neg => "-",
            ast::UnOp::Not => "!",
            ast::UnOp::BitNot => "~",
            _ => return Ok(None),
        };
        let procs = self.operator_candidates(scope, text)?;
        if procs.is_empty() {
            return Ok(None);
        }
        let args = vec![CallArg {
            name: None,
            spread: false,
            expr: None,
            op: Some(operand.clone()),
            span,
            scope,
        }];
        self.call_procs(f, scope, &procs, args, None, span)
            .map(Some)
    }

    pub fn try_index_operator_overload(
        &mut self,
        f: &mut FnCtx,
        scope: ScopeId,
        text: &str,
        base: &Operand,
        index: &Operand,
        span: Span,
    ) -> Result<Option<Operand>> {
        let bt = base.ty();
        let target = self.ty.types.pointee(bt).unwrap_or(bt);
        if !matches!(self.ty.types.kind(target), TypeKind::Struct(_)) {
            return Ok(None);
        }
        let mut procs = self.operator_candidates(scope, text)?;
        // Unrelated lexical operators do not hide builtin pointer indexing.
        // Keep index/type/result validation in normal call binding once a
        // receiver applies, rather than swallowing an arbitrary call error.
        procs.retain(|&p| self.first_param_accepts(p, target));
        if procs.is_empty() {
            return Ok(None);
        }
        let args = vec![
            CallArg {
                name: None,
                spread: false,
                expr: None,
                op: Some(base.clone()),
                span,
                scope,
            },
            CallArg {
                name: None,
                spread: false,
                expr: None,
                op: Some(index.clone()),
                span,
                scope,
            },
        ];
        let r = self.call_procs(f, scope, &procs, args, None, span)?;
        // `operator []` returning a pointer is an lvalue.
        Ok(Some(match r {
            Operand::Value {
                ty,
                val,
            } if self.ty.types.pointee(ty).is_some_and(|p| p != TypeId::VOID)
                && self.ty.types.pointee(ty) != Some(target) =>
            {
                let _ = val;
                Operand::Value {
                    ty,
                    val,
                }
            }
            other => other,
        }))
    }
}

pub fn binop_text(op: ast::BinOp) -> &'static str {
    use ast::BinOp::{
        Add, And, BitAnd, BitOr, BitXor, Div, Eq, Ge, Gt, Le, Lt, Mul, Ne, Or, Rem, Rotl, Rotr,
        Shl, Shr, Sub,
    };
    match op {
        Add => "+",
        Sub => "-",
        Mul => "*",
        Div => "/",
        Rem => "%",
        BitAnd => "&",
        BitOr => "|",
        BitXor => "^",
        Shl => "<<",
        Shr => ">>",
        Rotl => "<<<",
        Rotr => ">>>",
        Eq => "==",
        Ne => "!=",
        Lt => "<",
        Le => "<=",
        Gt => ">",
        Ge => ">=",
        And => "&&",
        Or => "||",
    }
}

impl TypeState {
    /// Scope a struct (or pointer to one) was declared in, for its
    /// `for_expansion` protocol.
    pub fn struct_home_scope(&self, t: TypeId) -> Option<ScopeId> {
        let t = self.types.pointee(t).unwrap_or(t);
        let s = self.types.as_struct(self.types.repr_struct(t))?;
        self.struct_asts.get(&s).map(|src| src.scope)
    }

    pub(in crate::sema) fn overloadable(&self, ty: TypeId) -> bool {
        matches!(
            self.types.kind(ty),
            TypeKind::Struct(_)
                | TypeKind::Array { .. }
                | TypeKind::Pointer(_)
                | TypeKind::Enum(_)
                | TypeKind::Distinct(_)
        )
    }
}
