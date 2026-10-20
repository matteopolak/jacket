//! Operator overloading: finding and calling `operator` procedures.

use super::{
    CallArg, Candidate, Compiler, FnCtx, Operand, ProcId, Result, ScopeId, Span, Sym, TypeId,
    TypeKind, Value, ast, err, ir, scope,
};

impl Compiler {
    /// Scope a struct (or pointer to one) was declared in, for its
    /// `for_expansion` protocol.
    pub fn struct_home_scope(&self, t: TypeId) -> Option<ScopeId> {
        let t = self.ty.types.pointee(t).unwrap_or(t);
        let s = self.ty.types.as_struct(self.ty.types.repr_struct(t))?;
        self.ty.struct_asts.get(&s).map(|src| src.scope)
    }

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

    pub(in crate::sema) fn overloadable(&self, ty: TypeId) -> bool {
        matches!(
            self.ty.types.kind(ty),
            TypeKind::Struct(_)
                | TypeKind::Array { .. }
                | TypeKind::Pointer(_)
                | TypeKind::Enum(_)
                | TypeKind::Distinct(_)
        )
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
        if !(self.overloadable(lt) || self.overloadable(rt)) || matches!(lhs, Operand::Type(_)) {
            return Ok(None);
        }
        if (self.ty.types.is_pointer(lt) || matches!(self.ty.types.kind(lt), TypeKind::Enum(_)))
            && !matches!(self.ty.types.kind(rt), TypeKind::Struct(_))
        {
            return Ok(None);
        }
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
        let args = vec![
            CallArg {
                name: None,
                spread: false,
                expr: None,
                op: Some(lhs.clone()),
                span,
                scope,
            },
            CallArg {
                name: None,
                spread: false,
                expr: None,
                op: Some(rhs.clone()),
                span,
                scope,
            },
        ];
        if !procs.is_empty()
            && let Ok(r) = self.try_call(f, scope, &procs, args.clone(), span)
        {
            return Ok(Some(r));
        }
        if !ne_fallback.is_empty()
            && let Ok(r) = self.try_call(f, scope, &ne_fallback, args.clone(), span)
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
            if let Ok(r) = self.try_call(f, scope, &symmetric, swapped, span) {
                return Ok(Some(r));
            }
        }
        if matches!(self.ty.types.kind(lt), TypeKind::Struct(_))
            || matches!(self.ty.types.kind(rt), TypeKind::Struct(_))
        {
            // Report the overload failure for struct operands.
            self.call_procs(f, scope, &procs, args, None, span)
                .map(Some)
        } else {
            Ok(None)
        }
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
