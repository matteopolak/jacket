//! Implicit conversions of operands and constants.
use super::{int_constant_fits, int_to_float};
use crate::sema::TypeState;
use crate::sema::lower::{FnCtx, Operand};
use crate::sema::{Compiler, Diagnostic, DiagnosticKind, ProcId, Result, Span, TypeId, Value, err};
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
                    let mut matching: Vec<ProcId> = Vec::new();
                    for &p in procs {
                        if self.proc(p).is_poly {
                            if let Some(inst) = self.instantiate_for_proc_type(p, to, span) {
                                matching.push(inst);
                            }
                            continue;
                        }
                        let pt = self.proc_type(p, span)?;
                        if pt == to || self.ty.proc_types_compatible(pt, to) {
                            matching.push(p);
                        }
                    }
                    let mut unique: Vec<ProcId> = Vec::new();
                    for p in matching {
                        if !unique.contains(&p) {
                            unique.push(p);
                        }
                    }
                    let matching = unique;
                    return match matching.as_slice() {
                        [one] => Ok(Operand::Procs(vec![*one])),
                        [] => err(
                            span,
                            format!(
                                "no overload matches procedure type {}",
                                self.ty.types.name(to)
                            ),
                        ),
                        [first, rest @ ..] => {
                            let mut d = Diagnostic::error(
                                span,
                                format!(
                                    "{} overloads fit procedure type {}",
                                    matching.len(),
                                    self.ty.types.name(to)
                                ),
                            )
                            .with_kind(DiagnosticKind::AmbiguousProcedureValue)
                            .with_label("it is not clear which one is meant")
                            .with_note(self.proc(*first).span, "this one fits");
                            for &other in rest {
                                d = d.with_note(self.proc(other).span, "and so does this one");
                            }
                            Err(Box::new(d.with_help(
                                "name the overload you mean: give it its own name and use that, or cast the call you need into a procedure of its own",
                            )))
                        }
                    };
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
                let global = self.out.string_global(bytes);
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
            } else {
                d = self.with_conversion_hint(d, from, to, span);
            }
            return Err(Box::new(d));
        }
        self.coerce(f, op, to, span)
    }

    /// Constant-to-constant conversions (returns None when a runtime conversion is needed).
    pub(in crate::sema) fn convert_const(
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
                if untyped && !int_constant_fits(*i, bits, signed, self.src.bit_pattern(span)) {
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
                    if untyped {
                        self.warn_float32_rounding(*x, span);
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

impl Compiler {
    /// A decimal literal that becomes a `float32` with a different value (`16777217.0` is
    /// 16777216 there): the digits written are not the number the program gets.
    fn warn_float32_rounding(&mut self, value: f64, span: Span) {
        let text = self.src.sources.snippet(span);
        let literal = !text.is_empty()
            && text.bytes().all(|b| {
                b.is_ascii_digit() || matches!(b, b'.' | b'_' | b'e' | b'E' | b'-' | b'+')
            })
            && text.bytes().any(|b| b == b'.');
        let rounded = value as f32;
        if !literal || !rounded.is_finite() || rounded.to_string().parse::<f64>() == Ok(value) {
            return;
        }
        self.warn(
            Diagnostic::warning(
                span,
                format!("this literal is {} as a float32", rounded as f64),
            )
            .with_label("float32 cannot hold these digits")
            .with_help("make it a float64, or write the float32 value"),
        );
    }
}
