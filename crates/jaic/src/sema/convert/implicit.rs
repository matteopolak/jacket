//! Implicit conversions of operands and constants.
use super::{int_constant_fits, int_to_float};
use crate::sema::TypeState;
use crate::sema::lower::{FnCtx, Operand};
use crate::sema::{Compiler, Diagnostic, DiagnosticKind, Result, Span, TypeId, Value, err};
use crate::types::TypeKind;

impl Compiler {
    /// Implicitly convert `op` to `to`, or fail with a type mismatch.
    pub fn convert(
        &mut self,
        f: &mut FnCtx,
        op: Operand,
        to: TypeId,
        span: Span,
    ) -> Result<Operand> {
        if let Operand::Callable {
            operand,
            parameters,
        } = op
        {
            let converted = self.convert(f, *operand, to, span)?;
            return Ok(if matches!(self.ty.types.kind(to), TypeKind::Proc(_)) {
                converted.with_callable(Some(parameters))
            } else {
                converted
            });
        }
        let from = op.ty();
        if from == to
            && !matches!(
                op,
                Operand::Const {
                    untyped: true,
                    ..
                }
            )
        {
            return Ok(op);
        }
        if self.is_stand_in(from) || self.is_stand_in(to) {
            return Ok(self.ide_sketch_value(f, to, span));
        }
        // A one-character string constant is a `u8` (`case "'";` on a byte).
        if to == TypeId::U8
            && let Operand::Const {
                value: Value::String(s),
                ..
            } = &op
            && s.len() == 1
        {
            return Ok(Operand::int(s[0] as i128, TypeId::U8));
        }
        match op {
            Operand::PolyStruct(ps) if to == TypeId::TYPE || to == TypeId::ANY => {
                let t = self.ty.poly_struct_type(ps);
                return self.convert(f, Operand::Type(t), to, span);
            }
            Operand::Type(t) if to == TypeId::TYPE => return Ok(Operand::Type(t)),
            Operand::Type(t) if to == TypeId::ANY => {
                return self.box_any(f, Operand::Type(t), span);
            }
            Operand::Procs(ref procs) => {
                if let TypeKind::Proc(_) = self.ty.types.kind(to) {
                    for &p in procs {
                        if self.proc(p).is_poly {
                            if let Some(inst) = self.instantiate_for_proc_type(p, to, span) {
                                return Ok(Operand::Procs(vec![inst]));
                            }
                            continue;
                        }
                        let pt = self.proc_type(p, span)?;
                        if pt == to || self.ty.proc_types_compatible(pt, to) {
                            return Ok(Operand::Procs(vec![p]));
                        }
                    }
                    return err(
                        span,
                        format!(
                            "no overload matches procedure type {}",
                            self.ty.types.name(to)
                        ),
                    );
                }
                if to == TypeId::VOID_PTR || (self.ty.types.is_pointer(to) && procs.len() == 1) {
                    let (_, v) = self.rvalue(f, op, span)?;
                    return Ok(Operand::Value {
                        ty: to,
                        val: v,
                    });
                }
                if to == TypeId::ANY {
                    return self.box_any(f, op, span);
                }
            }
            Operand::Const {
                value: Value::String(ref bytes),
                ..
            } if self.ty.types.pointee(to) == Some(TypeId::U8) => {
                // String literals are NUL-terminated: their data pointer is a C string.
                let global = self.string_global(bytes);
                let val = f.b.global_addr(global);
                return Ok(Operand::Value {
                    ty: to,
                    val,
                });
            }
            Operand::Const {
                ty,
                ref value,
                untyped,
            } => {
                if let Some(c) = self.convert_const(ty, value, untyped, to, span)? {
                    return Ok(c);
                }
            }
            _ => {}
        }
        let untyped = matches!(
            op,
            Operand::Const {
                untyped: true,
                ..
            }
        );
        if self.implicit_cost(from, untyped, to).is_none() {
            let mut d = Diagnostic::error(
                span,
                format!(
                    "type mismatch: expected `{}`, found `{}`",
                    self.ty.types.name(to),
                    self.ty.types.name(from)
                ),
            )
            .with_kind(DiagnosticKind::TypeMismatch);
            if to == TypeId::U8
                && let Operand::Const {
                    value: Value::String(s),
                    ..
                } = &op
            {
                d = d.with_help(format!(
                    "only a one-character string converts to a `u8`, and this one has {} bytes",
                    s.len()
                ));
            } else if let Some(help) = self.ty.conversion_help(from, to) {
                d = d.with_help(help);
            }
            return Err(Box::new(d));
        }
        self.coerce(f, op, to, span)
    }

    /// Constant-to-constant conversions (returns None when a runtime conversion is needed).
    fn convert_const(
        &mut self,
        ty: TypeId,
        value: &Value,
        untyped: bool,
        to: TypeId,
        span: Span,
    ) -> Result<Option<Operand>> {
        let tr = self.ty.types.repr(to);
        if let Some(fmt) = self.ty.types.wide_float(to) {
            if self.ty.types.wide_float(ty).is_none()
                && (untyped || self.implicit_cost(ty, false, to).is_some())
                && let Some(value) = Self::wide_const(fmt, value)
            {
                return Ok(Some(Operand::Const {
                    ty: to,
                    value,
                    untyped: false,
                }));
            }
            return Ok(None);
        }
        let result = match value {
            Value::Int(i)
                if self.ty.types.is_integer(tr)
                    && (untyped || self.implicit_cost(ty, false, to).is_some()) =>
            {
                let (bits, signed) = self.ty.types.int_info(tr).unwrap();
                if untyped && !int_constant_fits(*i, bits, signed, self.bit_pattern(span)) {
                    return err(
                        span,
                        format!("constant {i} does not fit in {}", self.ty.types.name(to)),
                    );
                }
                if matches!(self.ty.types.kind(to), TypeKind::Enum(_)) && untyped {
                    return Ok(None);
                }
                Value::Int(*i)
            }
            Value::Int(i) if self.ty.types.is_float(tr) && untyped => {
                Value::float(int_to_float(*i, tr))
            }
            Value::Float(x, _)
                if self.ty.types.is_float(tr)
                    && (untyped || self.implicit_cost(ty, false, to).is_some()) =>
            {
                if tr == TypeId::F32 {
                    if untyped && value.requires_float64() {
                        return err(
                            span,
                            "this floating literal requires float64 precision (use an explicit cast to round it)",
                        );
                    }
                    // A literal outside float32's normal range would become infinity or lose
                    // its precision: it needs an explicit cast.
                    let magnitude = x.abs();
                    if untyped
                        && magnitude.is_finite()
                        && magnitude != 0.0
                        && (magnitude > f32::MAX as f64 || magnitude < f32::MIN_POSITIVE as f64)
                    {
                        return err(
                            span,
                            format!(
                                "constant {x:e} does not fit in {} (convert with `cast({})` to round it)",
                                self.ty.types.name(to),
                                self.ty.types.name(to)
                            ),
                        );
                    }
                    Value::float(*x as f32 as f64)
                } else {
                    Value::float(*x)
                }
            }
            Value::Null
                if matches!(
                    self.ty.types.kind(tr),
                    TypeKind::Pointer(_) | TypeKind::Proc(_) | TypeKind::Type
                ) =>
            {
                Value::Null
            }
            Value::Bool(b) if tr == TypeId::BOOL => Value::Bool(*b),
            Value::String(s) if tr == TypeId::STRING => Value::String(s.clone()),
            Value::Type(t) if to == TypeId::TYPE => Value::Type(*t),
            Value::Proc(p) if self.implicit_cost(ty, false, to).is_some() => Value::Proc(*p),
            _ => return Ok(None),
        };
        Ok(Some(Operand::Const {
            ty: to,
            value: result,
            untyped: false,
        }))
    }
}

impl TypeState {
    /// How to get a `to` from a `from`, for the common mismatches with an obvious fix.
    pub(in crate::sema) fn conversion_help(&self, from: TypeId, to: TypeId) -> Option<String> {
        let to_name = self.types.name(to);
        let kind = |t| self.types.kind(t).clone();
        Some(match (kind(from), kind(to)) {
            (TypeKind::Float { .. }, TypeKind::Int { .. }) => {
                format!("convert with `cast({to_name})`, which drops the fraction")
            }
            (TypeKind::Int { .. }, TypeKind::Int { .. }) => {
                format!("convert with `cast({to_name})` if every value fits")
            }
            (TypeKind::Int { .. } | TypeKind::Float { .. }, TypeKind::String) => {
                "to turn a number into text, format it: `tprint(\"%\", value)` (from Basic)".into()
            }
            (TypeKind::String, TypeKind::Pointer(p)) if self.types.is_integer(p) => {
                "pass `value.data` for the bytes, or `to_c_string(value)` (from Basic) for a zero-terminated copy".into()
            }
            (TypeKind::Int { .. }, TypeKind::Bool) => "compare instead: `value != 0`".into(),
            (TypeKind::Int { .. }, TypeKind::Enum(_)) | (TypeKind::Enum(_), TypeKind::Int { .. }) => {
                format!("convert with `cast({to_name})`")
            }
            (TypeKind::Pointer(p), _) if p == to => {
                "this is a pointer to the value: dereference it with `value.*`".into()
            }
            (_, TypeKind::Pointer(p)) if p == from => {
                "this is the value itself: take its address with `*value`".into()
            }
            _ => return None,
        })
    }

    /// Structurally identical procedure types (ignoring parameter names).
    pub(in crate::sema) fn proc_types_compatible(&self, a: TypeId, b: TypeId) -> bool {
        match (self.types.kind(a), self.types.kind(b)) {
            (TypeKind::Proc(x), TypeKind::Proc(y)) => {
                x.params == y.params && x.returns == y.returns && x.c_call == y.c_call
            }
            _ => false,
        }
    }
}
