//! Index expressions and pointer-index operators.
use crate::ir::Ty;
use crate::sema::lower::{FnCtx, Operand};
use crate::sema::{Compiler, Result, ScopeId, Span, TypeId, Value, ast, err, ir};
use crate::types::{ArrayKind, TypeKind};

impl Compiler {
    /// The common type of a binary operation.
    /// `x[i]` through `operator *[]`: the returned pointer is the element's place.
    pub(super) fn try_pointer_index_operator(
        &mut self,
        f: &mut FnCtx,
        scope: ScopeId,
        base: &Operand,
        index: &Operand,
        span: Span,
    ) -> Result<Option<Operand>> {
        if self.operator_candidates(scope, "*[]")?.is_empty() {
            return Ok(None);
        }
        let base = match base.clone().without_callable() {
            // A pointer stored in a variable denotes the receiver, not the
            // address of the variable containing that pointer.
            operand if self.ty.types.pointee(operand.ty()).is_some() => {
                let (ty, val) = self.rvalue(f, operand, span)?;
                Operand::Value {
                    ty,
                    val,
                }
            }
            Operand::Place {
                ty,
                addr,
            } => Operand::Value {
                ty: self.ty.types.pointer(ty),
                val: addr,
            },
            // A constant or computed struct has no address of its own: a pointer receiver gets
            // a temporary copy, as with any other call that takes `*value`.
            operand
                if matches!(self.ty.types.kind(operand.ty()), TypeKind::Struct(_))
                    && self.has_pointer_receiver(scope, "*[]", operand.ty())? =>
            {
                let (ty, val) = self.address_of(f, operand, span)?;
                Operand::Value {
                    ty: self.ty.types.pointer(ty),
                    val,
                }
            }
            // By-value receivers also accept computed aggregate values. The
            // normal call binder retains their type and conversion rules.
            operand => operand,
        };
        let Some(result) = self.try_index_operator_overload(f, scope, "*[]", &base, index, span)?
        else {
            return Ok(None);
        };
        let (ty, val) = self.rvalue(f, result, span)?;
        let Some(elem) = self.ty.types.pointee(ty) else {
            return err(span, "operator *[] must return a pointer");
        };
        Ok(Some(Operand::Place {
            ty: elem,
            addr: val,
        }))
    }

    /// Does an `operator` named `text` take a pointer to `ty` as its first parameter?
    fn has_pointer_receiver(&mut self, scope: ScopeId, text: &str, ty: TypeId) -> Result<bool> {
        let mut procs = self.operator_candidates(scope, text)?;
        procs.retain(|&p| self.first_param_accepts(p, ty));
        Ok(procs.iter().any(|&p| {
            self.proc(p).lit.header.params.first().is_some_and(|param| {
                matches!(
                    param.ty.as_ref().map(|t| &t.kind),
                    Some(ast::ExprKind::Unary(ast::UnOp::Star, _))
                )
            })
        }))
    }

    /// An assignment target. An indexed target prefers `operator *[]`, so the write goes
    /// through the returned pointer; reads of the same expression keep using `operator []`.
    pub(in crate::sema) fn check_target(
        &mut self,
        f: &mut FnCtx,
        scope: ScopeId,
        target: &ast::Expr,
    ) -> Result<Operand> {
        if let ast::ExprKind::Index(base, index) = &target.kind
            && !self.operator_candidates(scope, "*[]")?.is_empty()
        {
            let base_op = self.check_expr(f, scope, base, None)?;
            let index_op = self.check_expr(f, scope, index, Some(TypeId::S64))?;
            if let Some(place) =
                self.try_pointer_index_operator(f, scope, &base_op, &index_op, target.span)?
            {
                return Ok(place);
            }
            return self.index_operand(f, scope, base_op, index_op, index.span, target.span);
        }
        self.check_expr(f, scope, target, None)
    }

    pub(super) fn check_index(
        &mut self,
        f: &mut FnCtx,
        scope: ScopeId,
        base: &ast::Expr,
        index: &ast::Expr,
        span: Span,
    ) -> Result<Operand> {
        let base_op = self.check_expr(f, scope, base, None)?;
        // `value[xx x]` on a struct with `operator []`: the cast takes its target from the
        // overload picked for the struct, not from an integer index.
        if matches!(
            index.kind,
            ast::ExprKind::Cast {
                ty: None,
                ..
            }
        ) && let Some(result) = self.auto_cast_index_operator(f, scope, &base_op, index, span)?
        {
            return Ok(result);
        }
        let index_op = self.check_expr(f, scope, index, Some(TypeId::S64))?;
        self.index_operand(f, scope, base_op, index_op, index.span, span)
    }

    fn auto_cast_index_operator(
        &mut self,
        f: &mut FnCtx,
        scope: ScopeId,
        base: &Operand,
        index: &ast::Expr,
        span: Span,
    ) -> Result<Option<Operand>> {
        let ty = base.ty();
        if !matches!(
            self.ty.types.kind(self.ty.types.repr_struct(ty)),
            TypeKind::Struct(_)
        ) {
            return Ok(None);
        }
        let mut procs = self.operator_candidates(scope, "[]")?;
        procs.retain(|&p| self.first_param_accepts(p, ty));
        if procs.is_empty() {
            return Ok(None);
        }
        let arg = |op: Option<Operand>, expr: Option<ast::Expr>| crate::sema::calls::CallArg {
            name: None,
            spread: false,
            expr,
            op,
            span,
            scope,
        };
        let args = vec![
            arg(Some(base.clone()), None),
            arg(None, Some(index.clone())),
        ];
        self.call_procs(f, scope, &procs, args, None, span)
            .map(Some)
    }

    /// `base[index]` on checked operands.
    pub(in crate::sema) fn index_operand(
        &mut self,
        f: &mut FnCtx,
        scope: ScopeId,
        base_op: Operand,
        index_op: Operand,
        index_span: Span,
        span: Span,
    ) -> Result<Operand> {
        // A field named through its owning type has no instance storage. In
        // type_of, represent its array as a type-only place so indexing finds
        // the element type through the ordinary array rules, without a load.
        let base_op = match base_op {
            Operand::Type(ty)
                if f.type_only
                    && matches!(
                        self.ty.types.kind(self.ty.types.repr(ty)),
                        TypeKind::Array { .. }
                    ) =>
            {
                Operand::Place {
                    ty,
                    addr: f.b.iconst(Ty::Ptr, 0),
                }
            }
            other => other,
        };
        // Constant string indexing folds.
        if let (
            Operand::Const {
                value: Value::String(s),
                ..
            },
            Operand::Const {
                value: Value::Int(i),
                ..
            },
        ) = (&base_op, &index_op)
            && (*i as usize) < s.len()
        {
            return Ok(Operand::int(s[*i as usize] as i128, TypeId::U8));
        }
        let bty = base_op.ty();
        // `operator []` when one matches, else `operator *[]` dereferenced.
        let getter = self.try_index_operator_overload(f, scope, "[]", &base_op, &index_op, span);
        if let Ok(Some(result)) = getter {
            return Ok(result);
        }
        if let Some(result) =
            self.try_pointer_index_operator(f, scope, &base_op, &index_op, span)?
        {
            return Ok(result);
        }
        // A pointer indexes its memory when no operator for the pointee fits.
        if self.ty.types.pointee(bty).is_none() {
            getter?;
        }
        let index_op = self.settle_untyped(index_op, Some(TypeId::S64));
        let ity = index_op.ty();
        if !self.ty.types.is_integer(ity)
            && !matches!(
                self.ty.types.kind(self.ty.types.repr(ity)),
                TypeKind::Enum(_)
            )
        {
            return err(
                index_span,
                format!(
                    "array index must be an integer, found {}",
                    self.ty.types.name(ity)
                ),
            );
        }
        let index_op = self.explicit_cast(
            f,
            index_op,
            TypeId::S64,
            ast::CastFlags {
                no_check: true,
                ..Default::default()
            },
            span,
        )?;
        let (_, idx) = self.rvalue(f, index_op, span)?;
        // Views, dynamic arrays and strings start with their count.
        let check = !f.checks_off.no_abc() && !f.type_only;
        if check {
            self.note_check_handler(span)?;
        }
        let (elem, data) = match self.ty.types.kind(self.ty.types.repr(bty)).clone() {
            TypeKind::Array {
                elem,
                kind: ArrayKind::Fixed(n),
            } => {
                let (_, addr) = self.address_of(f, base_op, span)?;
                if check {
                    let count = f.b.iconst(Ty::I64, n);
                    f.b.intrinsic(ir::Intrinsic::BoundsCheck, vec![idx, count], &[]);
                }
                (elem, addr)
            }
            TypeKind::Array {
                elem, ..
            } => {
                let (_, addr) = self.address_of(f, base_op, span)?;
                if check {
                    let count = f.b.load(Ty::I64, addr);
                    f.b.intrinsic(ir::Intrinsic::BoundsCheck, vec![idx, count], &[]);
                }
                let p = f.b.ptr_offset(addr, 8);
                (elem, f.b.load(Ty::Ptr, p))
            }
            TypeKind::String => {
                let (_, addr) = self.address_of(f, base_op, span)?;
                if check {
                    let count = f.b.load(Ty::I64, addr);
                    f.b.intrinsic(ir::Intrinsic::BoundsCheck, vec![idx, count], &[]);
                }
                let p = f.b.ptr_offset(addr, 8);
                (TypeId::U8, f.b.load(Ty::Ptr, p))
            }
            TypeKind::Pointer(elem) => {
                let (_, p) = self.rvalue(f, base_op, span)?;
                (elem, p)
            }
            _ => {
                return err(
                    span,
                    format!("cannot index a value of type {}", self.ty.types.name(bty)),
                );
            }
        };
        let size = self.size_of(elem, span)?;
        let s = f.b.iconst(Ty::I64, size);
        let off = f.b.bin(ir::BinOp::Mul, Ty::I64, idx, s);
        Ok(Operand::Place {
            ty: elem,
            addr: f.b.ptr_add(data, off),
        })
    }
}
