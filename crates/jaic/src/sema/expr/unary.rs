//! Unary operators, untyped constants, conditions and the traps checks emit.
use super::names_variable_path;
use crate::ast::{ExprKind as E, UnOp};
use crate::diagnostics::Code;
use crate::int_arith::wrap;
use crate::ir::{CmpOp, Ty};
use crate::sema::lower::{FnCtx, Operand, Origin};
use crate::sema::{Compiler, Diagnostic, Result, ScopeId, Span, TypeId, Value, ast, err, ir};
use crate::source::{Applicability, Suggestion};
use crate::types::{ArrayKind, TypeKind};

impl Compiler {
    /// `x.*` of a value that is not a pointer: the value as it is would do.
    fn cannot_dereference(&self, ty: TypeId, inner: Span, span: Span) -> Diagnostic {
        let d = Diagnostic::new(
            Code::CannotDereferenceValue,
            span,
            format!(
                "cannot dereference a value of type {}",
                self.ty.types.name(ty)
            ),
        );
        let text = self.src.sources.snippet(inner).trim();
        if text.is_empty() {
            return d;
        }
        d.with_suggestion(
            Suggestion::replace(
                format!("`{text}` is not a pointer: remove the dereference"),
                span,
                text,
                Applicability::MaybeIncorrect,
            )
            .with_action("Remove the dereference"),
        )
    }

    pub(super) fn check_unary(
        &mut self,
        f: &mut FnCtx,
        scope: ScopeId,
        op: UnOp,
        a: &ast::Expr,
        expected: Option<TypeId>,
        span: Span,
    ) -> Result<Operand> {
        match op {
            UnOp::Star => self.check_address_of(f, scope, a, span),
            UnOp::Deref => self.check_deref(f, scope, a, span),
            UnOp::Plus => self.check_expr(f, scope, a, expected),
            UnOp::Neg | UnOp::Not | UnOp::BitNot => {
                self.check_arithmetic_unary(f, scope, op, a, expected, span)
            }
        }
    }

    /// `*x`: a pointer type for a type operand, the address of a place or procedure, or of a
    /// temporary.
    fn check_address_of(
        &mut self,
        f: &mut FnCtx,
        scope: ScopeId,
        a: &ast::Expr,
        span: Span,
    ) -> Result<Operand> {
        // Address-taking selects the pointer getter even when an ordinary
        // getter is available. Reuse its place normalization, and evaluate
        // each operand only once if builtin indexing is the fallback.
        let inner = if let E::Index(base, index) = &a.kind
            && self.pointer_index_reachable_from(scope, base)?
        {
            let base_op = self.check_expr(f, scope, base, None)?;
            let index_op = self.check_expr(f, scope, index, Some(TypeId::S64))?;
            match self.try_pointer_index_operator(f, scope, &base_op, &index_op, span)? {
                Some(place) => place,
                None => self.index_operand(f, scope, base_op, index_op, index.span, a.span)?,
            }
        } else {
            if !self.pr.local_consts.is_empty()
                && let E::Ident(name) = &a.kind
                && let Some(&id) = self.lookup(scope, *name)?.first()
                && matches!(
                    self.pr.local_consts.get(&id),
                    Some((Value::Int(_) | Value::Float(_, _) | Value::Bool(_), _))
                )
            {
                self.warn_address_of_macro_constant(scope, span, name.as_str());
            }
            // The address of a variable (`*global`, `*global.field`) is constant even
            // though the variable is not: naming it reads no value.
            let constant_only = f.constant_only;
            if names_variable_path(a) {
                f.constant_only = None;
            }
            let inner = self.check_expr(f, scope, a, None);
            f.constant_only = constant_only;
            inner?
        };
        self.address_of_operand(f, inner, span)
    }

    /// The pointer `*` makes of an already checked operand.
    fn address_of_operand(&mut self, f: &mut FnCtx, inner: Operand, span: Span) -> Result<Operand> {
        match inner {
            Operand::Type(t) => Ok(Operand::Type(self.ty.types.pointer(t))),
            // Concrete default pointees share the explicit instance's identity.
            // Required arguments still leave a source type for generic patterns.
            Operand::PolyStruct(ps) => {
                let t = if self.ty.poly_struct_has_defaults(ps) {
                    self.instantiate_struct(ps, Vec::new(), span)?
                } else {
                    self.ty.poly_struct_type(ps)
                };
                Ok(Operand::Type(self.ty.types.pointer(t)))
            }
            Operand::Place {
                ty,
                addr,
            } => Ok(Operand::Value {
                ty: self.ty.types.pointer(ty),
                val: addr,
            }),
            Operand::Procs(p) if p.len() == 1 => {
                let op = Operand::Procs(p);
                let parameters = self.callable_operand_parameters(&op)?;
                let (ty, v) = self.rvalue(f, op, span)?;
                Ok(Operand::Value {
                    ty,
                    val: v,
                }
                .with_callable(parameters))
            }
            Operand::Const {
                value: Value::Int(_) | Value::Float(_, _) | Value::Bool(_),
                ..
            } => err(
                Code::CannotTakeAddress,
                span,
                "cannot take the address of a number or bool constant: it has no storage (assign it to a variable first)",
            ),
            // An aggregate constant lives in static data, so its address outlives the
            // procedure and is the same on every evaluation.
            Operand::Const {
                value: Value::Bytes(agg),
                ty,
                ..
            } if self.is_memory_type(ty) => {
                let key = (ty, agg.bytes.clone(), agg.relocs.clone());
                let global = if let Some(&g) = self.out.const_addr_globals.get(&key) {
                    g
                } else {
                    let size = self.size_of(ty, span)?;
                    let align = self.align_of(ty, span)?;
                    let g = self.out.program.add_global(ir::Global {
                        name: format!("const.{}", self.out.program.globals.len()),
                        size,
                        align,
                        init: agg.bytes.clone(),
                        relocs: agg.relocs.clone(),
                        read_only: false,
                        export: None,
                    });
                    self.out.const_addr_globals.insert(key, g);
                    g
                };
                let addr = f.b.global_addr(global);
                Ok(Operand::Value {
                    ty: self.ty.types.pointer(ty),
                    val: addr,
                })
            }
            other => {
                // Address of a temporary.
                let (ty, addr) = self.address_of(f, other, span)?;
                Ok(Operand::Value {
                    ty: self.ty.types.pointer(ty),
                    val: addr,
                })
            }
        }
    }

    /// `x.*`: the place a pointer points at.
    fn check_deref(
        &mut self,
        f: &mut FnCtx,
        scope: ScopeId,
        a: &ast::Expr,
        span: Span,
    ) -> Result<Operand> {
        let inner = self.check_expr(f, scope, a, None)?;
        let (ty, v) = self.rvalue(f, inner, span)?;
        let Some(pointee) = self.ty.types.pointee(ty) else {
            return Err(Box::new(self.cannot_dereference(ty, a.span, span)));
        };
        // `<< ptr` on a `*void` is a `void` place (it prints as `void`).
        Ok(Operand::Place {
            ty: pointee,
            addr: v,
        })
    }

    /// `-x`, `!x` and `~x`: folded for a constant, an operator overload, or the IR operation.
    fn check_arithmetic_unary(
        &mut self,
        f: &mut FnCtx,
        scope: ScopeId,
        op: UnOp,
        a: &ast::Expr,
        expected: Option<TypeId>,
        span: Span,
    ) -> Result<Operand> {
        let inner = self.check_expr(
            f,
            scope,
            a,
            if op == UnOp::Not {
                None
            } else {
                expected
            },
        )?;
        if let Operand::Const {
            ty,
            value,
            untyped,
            origin,
        } = &inner
            && let Some(folded) =
                self.fold_unary_const(op, (*ty, value, *untyped, *origin), expected)
        {
            return Ok(folded);
        }
        if let Some(result) =
            self.try_unary_operator_overload(f, scope, op, &inner, (span, a.span))?
        {
            return Ok(result);
        }
        let inner = self.settle_untyped(inner, expected);
        let (ty, v) = self.rvalue(f, inner, span)?;
        self.lower_unary(f, op, ty, v, span)
    }

    /// The constant `op value` folds to, if it does.
    fn fold_unary_const(
        &self,
        op: UnOp,
        (ty, value, untyped, origin): (TypeId, &Value, bool, Origin),
        expected: Option<TypeId>,
    ) -> Option<Operand> {
        let folded = match (op, value) {
            (UnOp::Neg, Value::Int(v)) => Some(Value::Int(v.wrapping_neg())),
            (UnOp::Neg, Value::Float(v, required)) => Some(Value::Float(-v, *required)),
            (UnOp::Not, Value::Bool(v)) => Some(Value::Bool(!v)),
            (UnOp::Not, Value::Int(v)) => Some(Value::Bool(*v == 0)),
            (UnOp::Not, Value::Null) => Some(Value::Bool(true)),
            (UnOp::BitNot, Value::Int(v)) => {
                // Non-integer constant types (an enum value) complement as 64-bit signed.
                let (bits, signed) = self.ty.types.int_info(ty).unwrap_or((64, true));
                let bits = if untyped {
                    expected
                        .and_then(|e| self.ty.types.int_info(e))
                        .map_or(bits, |i| i.0)
                } else {
                    bits
                };
                Some(Value::Int(wrap(!v, bits, signed && !untyped)))
            }
            _ => None,
        }?;
        Some(Operand::Const {
            ty: if op == UnOp::Not {
                TypeId::BOOL
            } else {
                ty
            },
            value: folded,
            untyped: untyped && op != UnOp::Not,
            // A sign keeps the spelling of the number it applies to, a complement is a pattern
            // by its operator; any other operator leaves nothing to say.
            origin: match op {
                UnOp::Neg => origin,
                UnOp::BitNot => Origin::Complement,
                _ => Origin::Computed,
            },
        })
    }

    /// The IR operation for `op` on the value `v` of type `ty`.
    fn lower_unary(
        &mut self,
        f: &mut FnCtx,
        op: UnOp,
        ty: TypeId,
        v: ir::Val,
        span: Span,
    ) -> Result<Operand> {
        // `-` and `~` are arithmetic: a `Type`, `bool`, pointer or other non-number has no meaning
        // under them, and a `Type` would otherwise become a garbage type id at run time.
        if op != UnOp::Not
            && !self.ty.types.is_integer(ty)
            && !self.ty.types.is_float(ty)
            && self.ty.types.wide_float(ty).is_none()
        {
            let what = if op == UnOp::Neg {
                "negate"
            } else {
                "bit-complement"
            };
            return Err(Box::new(Diagnostic::new(
                Code::UnsupportedOperation,
                span,
                format!("cannot {what} {}", self.ty.types.name(ty)),
            )));
        }
        match op {
            UnOp::Not => {
                let b = self.truthy(f, ty, v, span)?;
                let one = f.b.iconst(Ty::I8, 1);
                Ok(Operand::Value {
                    ty: TypeId::BOOL,
                    val: f.b.bin(ir::BinOp::Xor, Ty::I8, b, one),
                })
            }
            UnOp::Neg if self.ty.types.wide_float(ty).is_some() => {
                let fmt = self.ty.types.wide_float(ty).expect("checked above");
                Ok(self.wide_negate(f, fmt, ty, v))
            }
            UnOp::Neg => {
                let t = self.ir_ty(ty).ok_or_else(|| {
                    Box::new(Diagnostic::new(
                        Code::CannotNegate,
                        span,
                        format!("cannot negate {}", self.ty.types.name(ty)),
                    ))
                })?;
                let op = if t.is_float() {
                    ir::UnOp::FNeg
                } else {
                    ir::UnOp::Neg
                };
                Ok(Operand::Value {
                    ty,
                    val: f.b.un(op, t, v),
                })
            }
            _ => {
                let t = self.ir_ty(ty).filter(|t| !t.is_float()).ok_or_else(|| {
                    Box::new(Diagnostic::new(
                        Code::CannotBitComplement,
                        span,
                        format!("cannot bit-complement {}", self.ty.types.name(ty)),
                    ))
                })?;
                Ok(Operand::Value {
                    ty,
                    val: f.b.un(ir::UnOp::Not, t, v),
                })
            }
        }
    }


    /// Give an untyped constant a concrete type from context (or its default).
    pub fn settle_untyped(&self, op: Operand, expected: Option<TypeId>) -> Operand {
        match op {
            Operand::Const {
                ty,
                value,
                untyped: true,
                ..
            } => {
                let target = expected.filter(|&e| {
                    let r = self.ty.types.repr(e);
                    match &value {
                        Value::Int(_) => self.ty.types.is_integer(r) || self.ty.types.is_float(r),
                        Value::Float(_, p) => {
                            self.ty.types.is_float(r) && !(p.requires_float64 && r == TypeId::F32)
                        }
                        Value::Null => {
                            self.ty.types.is_pointer(r)
                                || matches!(self.ty.types.kind(r), TypeKind::Proc(_))
                        }
                        _ => false,
                    }
                });
                let ty = target.unwrap_or_else(|| self.ty.default_untyped(ty, &value));
                let value = match value {
                    Value::Int(i) if self.ty.types.is_float(ty) => Value::float(i as f64),
                    v => v,
                };
                Operand::Const {
                    ty,
                    value,
                    untyped: false,
                    origin: Origin::Computed,
                }
            }
            other => other,
        }
    }

    /// Convert a scalar value to a 0/1 `I8` condition.
    pub fn truthy(&mut self, f: &mut FnCtx, ty: TypeId, v: ir::Val, span: Span) -> Result<ir::Val> {
        if self.is_stand_in(ty) {
            return Ok(f.b.iconst(Ty::I8, 1));
        }
        let Some(t) = self.ir_ty(ty) else {
            if ty == TypeId::STRING {
                let count = f.b.load(Ty::I64, v);
                let zero = f.b.iconst(Ty::I64, 0);
                return Ok(f.b.cmp(CmpOp::Ne, Ty::I64, count, zero));
            }
            if let TypeKind::Array {
                kind: ArrayKind::Fixed(n),
                ..
            } = self.ty.types.kind(ty)
            {
                // A fixed array's count is part of its type.
                return Ok(f.b.iconst(Ty::I8, (*n != 0) as u64));
            }
            if let TypeKind::Array {
                ..
            } = self.ty.types.kind(ty)
            {
                let count = f.b.load(Ty::I64, v);
                let zero = f.b.iconst(Ty::I64, 0);
                return Ok(f.b.cmp(CmpOp::Ne, Ty::I64, count, zero));
            }
            return Err(Box::new(
                Diagnostic::new(Code::ValueType,
                    span,
                    format!(
                        "a value of type {} cannot be used as a condition",
                        self.ty.types.name(ty)
                    ),
                )
                .with_help(
                    "a condition is a bool, a number, a pointer, a string or an array: test or compare a member instead",
                ),
            ));
        };
        if ty == TypeId::BOOL {
            return Ok(v);
        }
        if t.is_float() {
            let zero = f.b.fconst(t, 0.0);
            return Ok(f.b.cmp(CmpOp::FNe, t, v, zero));
        }
        let zero = f.b.iconst(t, 0);
        Ok(f.b.cmp(CmpOp::Ne, t, v, zero))
    }

    /// Check an expression used as a condition, producing an `I8`.
    pub fn check_condition(
        &mut self,
        f: &mut FnCtx,
        scope: ScopeId,
        expr: &ast::Expr,
    ) -> Result<ir::Val> {
        let op = self.check_expr(f, scope, expr, Some(TypeId::BOOL))?;
        let op = self.settle_untyped(op, None);
        let (ty, v) = self.rvalue(f, op, expr.span)?;
        self.truthy(f, ty, v, expr.span)
    }

    /// Resolve the compiler-private native error reporter from the runtime's own
    /// file scopes. This transport hook is not a public library export.
    pub(crate) fn note_check_handler(&mut self, span: Span) -> Result<()> {
        if self.ns.check_handler_resolved {
            return Ok(());
        }
        self.ns.check_handler_resolved = true;
        let result = (|| {
            let Some(proc) =
                self.private_runtime_procedure(crate::sema::private_names::CHECK_FAILED, span)?
            else {
                return Ok(());
            };
            let signature = self.signature(proc, span)?;
            let expected = [
                TypeId::S64,
                TypeId::S64,
                TypeId::S64,
                TypeId::BOOL,
                TypeId::S64,
            ];
            if signature.has_context
                || signature.c_call
                || signature.c_varargs
                || !signature.returns.is_empty()
                || signature.params.len() != 6
                || signature.params.iter().any(|p| p.variadic || p.discard)
                || signature.params[..5]
                    .iter()
                    .zip(expected)
                    .any(|(p, ty)| p.ty != ty)
                || !matches!(self.ty.types.kind(signature.params[5].ty), TypeKind::Pointer(ty) if *ty == TypeId::U8)
            {
                return err(
                    Code::RuntimeConfigurationInvalid,
                    span,
                    "private native check reporter must take (s64, s64, s64, bool, s64, *u8), return nothing, and have no context",
                );
            }
            let crate::sema::procs::ProcTarget::Func(func) = self.proc_func(proc, span)? else {
                return err(
                    Code::RuntimeConfigurationInvalid,
                    span,
                    "private native check reporter must be defined in Jai",
                );
            };
            self.out.program.check_failed = Some(func);
            Ok(())
        })();
        if result.is_err() {
            self.ns.check_handler_resolved = false;
        }
        result
    }

    /// Stop the program at a failed check that has no details (`ir::Intrinsic::Trap` with an
    /// `ir::CheckReason` reason). Every trap with a reason goes through here or `emit_check_failed`,
    /// so native code always has the reporting procedure.
    pub(crate) fn emit_trap(
        &mut self,
        f: &mut FnCtx,
        reason: ir::CheckReason,
        span: Span,
    ) -> Result<()> {
        self.note_check_handler(span)?;
        let reason = f.b.iconst(Ty::I64, reason as u64);
        f.b.intrinsic(ir::Intrinsic::Trap, vec![reason], &[]);
        Ok(())
    }

    /// Report a failed check (`ir::Intrinsic::CheckFailed`): `reason` is an `ir::CheckReason`, `a`
    /// and `b` its `I64` details. A fatal failure stops the program.
    pub(crate) fn emit_check_failed(
        &mut self,
        f: &mut FnCtx,
        reason: ir::CheckReason,
        a: ir::Val,
        b: ir::Val,
        fatal: bool,
        span: Span,
    ) -> Result<()> {
        self.note_check_handler(span)?;
        f.b.repeat_loc();
        let reason = f.b.iconst(Ty::I64, reason as u64);
        let fatal = f.b.iconst(Ty::I8, u64::from(fatal));
        f.b.intrinsic(ir::Intrinsic::CheckFailed, vec![reason, a, b, fatal], &[]);
        Ok(())
    }
}

/// Default precision follows `lexer::float_literal_precision`: the fractional-digit count for
/// decimal spellings and the digit width for `0h` bit patterns.
pub(super) fn float_literal_type(precision: crate::lexer::FloatLiteralPrecision) -> TypeId {
    if precision.defaults_to_float64 || precision.requires_float64 {
        TypeId::F64
    } else {
        TypeId::F32
    }
}
