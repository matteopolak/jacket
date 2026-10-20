//! Operator overloading: finding and calling `operator` procedures.

use super::{
    CallArg, Candidate, Compiler, FnCtx, Operand, ProcId, Result, ScopeId, Span, Sym, TypeId,
    TypeKind, Value, ast, err, ir, scope,
};
use crate::ast::ExprKind as E;
use crate::sema::binary::Binary;
use crate::sema::{Diagnostic, TypeState};

/// `.{...}` with no type written: its type comes from where it is used.
fn is_untyped_literal(e: &ast::Expr) -> bool {
    matches!(
        e.kind,
        E::StructLit {
            ty: None,
            ..
        }
    )
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

    /// `.{...} op x` and `x op .{...}`: the other operand is checked, and the untyped literal is
    /// built as the type the chosen `operator` overload takes, as a literal argument of a call is.
    /// `None` when there is no such literal, or no operator named `op` that could take it.
    pub(in crate::sema) fn try_literal_operand_overload(
        &mut self,
        f: &mut FnCtx,
        scope: ScopeId,
        node: Binary<'_>,
        span: Span,
    ) -> Result<Option<Operand>> {
        let Binary {
            op,
            a,
            b,
        } = node;
        let (literal_first, literal, other) = match (is_untyped_literal(a), is_untyped_literal(b)) {
            (true, false) => (true, a, b),
            (false, true) => (false, b, a),
            _ => return Ok(None),
        };
        if self.operator_candidates(scope, binop_text(op))?.is_empty()
            && (op != ast::BinOp::Ne || self.operator_candidates(scope, "==")?.is_empty())
        {
            return Ok(None);
        }
        let other = self.check_expr(f, scope, other, None)?;
        if matches!(other, Operand::Type(_)) {
            return err(
                span,
                "a type cannot be an operand of an overloaded operator",
            );
        }
        let deferred = CallArg {
            name: None,
            spread: false,
            expr: Some(literal.clone()),
            op: None,
            span: literal.span,
            scope,
        };
        let checked = operand_arg(scope, span, &other);
        let args = if literal_first {
            vec![deferred, checked]
        } else {
            vec![checked, deferred]
        };
        self.call_operator(f, scope, op, args, true, span)
    }

    /// Call the `operator` overloads of `op` with `args`: a plain match, then `!=` as the negated
    /// `==`, then a `#symmetric` overload with the operands swapped. An argument that is still
    /// unchecked (an untyped literal) is typed by the overload chosen, and two overloads that
    /// take it equally well are an error.
    fn call_operator(
        &mut self,
        f: &mut FnCtx,
        scope: ScopeId,
        op: ast::BinOp,
        args: Vec<CallArg>,
        reports_failure: bool,
        span: Span,
    ) -> Result<Option<Operand>> {
        let deferred = args.iter().any(|a| a.op.is_none());
        let text = binop_text(op);
        let procs = self.operator_candidates(scope, text)?;
        // `a != b` without a matching `operator !=` is `!(a == b)`.
        let ne_fallback = if op == ast::BinOp::Ne {
            self.operator_candidates(scope, "==")?
        } else {
            Vec::new()
        };
        if procs.is_empty() && ne_fallback.is_empty() {
            return Ok(None);
        }
        if !procs.is_empty()
            && let Some(r) = self.call_overload(f, scope, &procs, args.clone(), deferred, span)?
        {
            return Ok(Some(r));
        }
        if !ne_fallback.is_empty()
            && let Some(r) =
                self.call_overload(f, scope, &ne_fallback, args.clone(), deferred, span)?
        {
            let (ty, v) = self.rvalue(f, r, span)?;
            let b = self.truthy(f, ty, v, span)?;
            let one = f.b.iconst(ir::Ty::I8, 1);
            return Ok(Some(Operand::Value {
                ty: TypeId::BOOL,
                val: f.b.bin(ir::BinOp::Xor, ir::Ty::I8, b, one),
            }));
        }
        if procs.is_empty() {
            if deferred {
                return self
                    .call_procs(f, scope, &ne_fallback, args, None, span)
                    .map(Some);
            }
            return Ok(None);
        }
        // #symmetric operators accept swapped operands.
        let symmetric: Vec<ProcId> = procs
            .iter()
            .copied()
            .filter(|&p| self.proc(p).lit.header.flags.symmetric())
            .collect();
        if !symmetric.is_empty() {
            let swapped = vec![args[1].clone(), args[0].clone()];
            if let Some(r) = self.call_overload(f, scope, &symmetric, swapped, deferred, span)? {
                return Ok(Some(r));
            }
        }
        if reports_failure {
            // Report the overload failure for struct operands.
            self.call_procs(f, scope, &procs, args, None, span)
                .map(Some)
        } else {
            Ok(None)
        }
    }

    /// The call of the best of `procs` for `args`, or `None` when none takes them. With an
    /// unchecked argument the candidates are ranked as for an ordinary call, and a tie is an
    /// error; otherwise the first cheapest wins and a failing emission counts as no match.
    fn call_overload(
        &mut self,
        f: &mut FnCtx,
        scope: ScopeId,
        procs: &[ProcId],
        args: Vec<CallArg>,
        deferred: bool,
        span: Span,
    ) -> Result<Option<Operand>> {
        if !deferred {
            return Ok(self.try_call(f, scope, procs, args, span).ok());
        }
        let (mut best, _) = self.rank_candidates(procs, &args, span);
        if best.is_empty() {
            return Ok(None);
        }
        if self.order_candidates(&mut best) {
            let name = self.proc(best[0].proc).name.to_string();
            let mut d = Diagnostic::error(
                span,
                format!(
                    "Procedure call matches {} possible overloads: `{name}` is ambiguous for \
                     these operands.",
                    best.len()
                ),
            );
            for c in &best {
                d = d.with_note(self.proc(c.proc).span, "this overload matches");
            }
            return Err(Box::new(d));
        }
        let chosen = best.swap_remove(0);
        self.emit_call(f, scope, chosen, args, span).map(Some)
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

    pub(in crate::sema) fn try_call(
        &mut self,
        f: &mut FnCtx,
        scope: ScopeId,
        procs: &[ProcId],
        args: Vec<CallArg>,
        span: Span,
    ) -> Result<Operand> {
        let mut best: Option<Candidate> = None;
        for &p in procs {
            if let Ok(c) = self.match_candidate(p, &args, span)
                && best.as_ref().is_none_or(|b| c.cost < b.cost)
            {
                best = Some(c);
            }
        }
        match best {
            Some(c) => self.emit_call(f, scope, c, args, span),
            None => err(span, "no operator overload matches"),
        }
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
