//! Checked and explicit casts.
use super::{int_constant_fits, int_to_float};
use crate::ir::{ConvOp, Ty};
use crate::sema::SourceState;
use crate::sema::lower::{FnCtx, Operand};
use crate::sema::{Compiler, Diagnostic, Result, Span, TypeId, Value, ast, expr, ir};
use crate::source::{Applicability, Suggestion};
use crate::types::{ArrayKind, TypeKind};

impl Compiler {
    /// `Build_Options.cast_bounds_check`: before `cast(T) x` (or `xx x`) of a runtime integer,
    /// check that the value fits `T`. That covers narrowing and every change of sign:
    /// `cast(u64)` of a negative `s8` and `cast(s64)` of a `u64` above `S64_MAX` fail too.
    /// `cast,no_check` and `cast,trunc` keep the bits unchecked. Returns the operand, evaluated.
    pub(crate) fn emit_cast_check(
        &mut self,
        f: &mut FnCtx,
        op: Operand,
        to: TypeId,
        span: Span,
    ) -> Result<Operand> {
        let mode = self.options.cast_bounds_check;
        let from = op.ty();
        let plain_int = |s: &Self, t: TypeId| {
            !matches!(s.ty.types.kind(t), TypeKind::Enum(_))
                && !matches!(s.ty.types.kind(s.ty.types.repr(t)), TypeKind::Enum(_))
        };
        let (Some((sbits, ssigned)), Some((tbits, tsigned))) =
            (self.ty.types.int_info(from), self.ty.types.int_info(to))
        else {
            return Ok(op);
        };
        if mode == 0
            || f.type_only
            || op.is_const()
            || !plain_int(self, from)
            || !plain_int(self, to)
            || sbits > 64
            || tbits > 64
        {
            return Ok(op);
        }
        let range = |bits: u8, signed: bool| -> (i128, i128) {
            if signed {
                (-(1i128 << (bits - 1)), (1i128 << (bits - 1)) - 1)
            } else {
                (0, (1i128 << bits) - 1)
            }
        };
        let (smin, smax) = range(sbits, ssigned);
        let (tmin, tmax) = range(tbits, tsigned);
        if tmin <= smin && smax <= tmax {
            return Ok(op);
        }
        let Some(t) = self.ir_ty(from) else {
            return Ok(op);
        };
        let (ty, x) = self.rvalue(f, op, span)?;
        // Constants are the bits of the source type.
        let bits = |v: i128| (v as u64) & (u64::MAX >> (64 - u32::from(sbits)));
        let mut bad = None;
        if tmin > smin {
            // Only a signed value can be below the target's minimum.
            let min = f.b.iconst(t, bits(tmin));
            bad = Some(f.b.cmp(ir::CmpOp::SLt, t, x, min));
        }
        if tmax < smax {
            let max = f.b.iconst(t, bits(tmax));
            let over = if ssigned {
                ir::CmpOp::SGt
            } else {
                ir::CmpOp::UGt
            };
            let high = f.b.cmp(over, t, x, max);
            bad = Some(match bad {
                Some(low) => f.b.bin(ir::BinOp::Or, Ty::I8, low, high),
                None => high,
            });
        }
        if let Some(bad) = bad {
            let fail = f.b.new_block();
            let cont = f.b.new_block();
            f.b.branch(bad, fail, cont);
            f.b.switch_to(fail);
            let value = if t == Ty::I64 {
                x
            } else {
                let widen = if ssigned {
                    ConvOp::SExt
                } else {
                    ConvOp::ZExt
                };
                f.b.conv(widen, t, Ty::I64, x)
            };
            let code =
                ir::cast_check_code(u64::from(tbits / 8), tsigned, ssigned, u64::from(sbits / 8));
            let code = f.b.iconst(Ty::I64, code);
            self.emit_check_failed(
                f,
                ir::CheckReason::CastOverflow,
                value,
                code,
                mode == 2,
                span,
            )?;
            f.b.jump(cont);
            f.b.switch_to(cont);
        }
        Ok(Operand::Value {
            ty,
            val: x,
        })
    }

    /// `cast(T) k` of an integer constant `k` checks the range at compile time, as a runtime
    /// cast would at run time: `cast(u8) -1` is an error, `cast(s8) 0xff` is `-1`.
    pub(crate) fn check_constant_cast(
        &self,
        op: &Operand,
        to: TypeId,
        (span, cast): (Span, Span),
    ) -> Result<()> {
        let Operand::Const {
            value: Value::Int(v),
            ..
        } = op
        else {
            return Ok(());
        };
        if matches!(self.ty.types.kind(to), TypeKind::Enum(_))
            || matches!(
                self.ty.types.kind(self.ty.types.repr(to)),
                TypeKind::Enum(_)
            )
        {
            return Ok(());
        }
        let Some((bits, signed)) = self.ty.types.int_info(to) else {
            return Ok(());
        };
        if int_constant_fits(*v, bits, signed, self.src.bit_pattern(span)) {
            return Ok(());
        }
        let name = self.ty.types.name(to);
        let mut d = Diagnostic::error(
            span,
            format!(
                "constant {v} is out of range for {name}; write `cast,trunc({name})` to keep the low bits"
            ),
        );
        if let Some(suggestion) = self.cast_flag_suggestion(
            cast,
            ",trunc",
            format!("to keep the low bits, write `cast,trunc({name})`"),
        ) {
            d = d.with_suggestion(suggestion);
        }
        Err(Box::new(d))
    }

    /// A suggestion that adds `flag` (`,trunc`, `,force`) to the `cast(T)` written at `cast`.
    /// `None` for any other spelling of a cast (`xx x`).
    fn cast_flag_suggestion(&self, cast: Span, flag: &str, title: String) -> Option<Suggestion> {
        if !self.src.sources.snippet(cast).starts_with("cast(") {
            return None;
        }
        let at = Span {
            start: cast.start + 4,
            end: cast.start + 4,
            ..cast
        };
        Some(
            Suggestion::replace(title, at, flag, Applicability::MaybeIncorrect)
                .with_action(format!("Write `cast{flag}`")),
        )
    }

    /// Advice for a cast between types that have no conversion.
    fn cannot_cast_help(&self, from: TypeId, to: TypeId) -> Option<String> {
        let types = &self.ty.types;
        let (from_name, to_name) = (types.name(from), types.name(to));
        let single = |t| {
            matches!(
                types.kind(types.repr(t)),
                TypeKind::Bool
                    | TypeKind::Int { .. }
                    | TypeKind::Float { .. }
                    | TypeKind::Enum(_)
                    | TypeKind::Pointer(_)
            )
        };
        let aggregate = |t| {
            matches!(
                types.kind(types.repr(t)),
                TypeKind::Struct(_) | TypeKind::Array { .. } | TypeKind::String
            )
        };
        if aggregate(from) && single(to) {
            return Some(format!(
                "{from_name} holds several values: cast one of its members to {to_name} instead"
            ));
        }
        if single(from) && matches!(types.kind(types.repr(to)), TypeKind::Struct(_)) {
            return Some(format!(
                "build a {to_name} from its members with a struct literal instead: `{to_name}.{{...}}`"
            ));
        }
        if single(from) && to == TypeId::STRING {
            return Some(
                "to turn a number into text, format it: `tprint(\"%\", value)` (from Basic)".into(),
            );
        }
        None
    }

    /// `cast(T) x` / `xx x`.
    pub fn explicit_cast(
        &mut self,
        f: &mut FnCtx,
        op: Operand,
        to: TypeId,
        flags: ast::CastFlags,
        span: Span,
    ) -> Result<Operand> {
        // A procedure name casts like the procedure value it denotes.
        let op = match op {
            Operand::Procs(_) => {
                let (ty, val) = self.rvalue(f, op, span)?;
                Operand::Value {
                    ty,
                    val,
                }
            }
            op => op,
        };
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
        if let Some(converted) = self.wide_cast(f, op.clone(), to, span)? {
            return Ok(converted);
        }
        // `cast(u8, "\u001F")`: a one-character string is its byte, as it is implicitly.
        if let Operand::Const {
            value: Value::String(s),
            ..
        } = &op
            && s.len() == 1
            && self.ty.types.is_integer(self.ty.types.repr(to))
        {
            return self.explicit_cast(f, Operand::untyped_int(s[0] as i128), to, flags, span);
        }
        // Constant folding of numeric casts.
        if let Operand::Const {
            value, ..
        } = &op
        {
            let folded = self.fold_cast(value, op.ty(), to, flags);
            if let Some(value) = folded {
                return Ok(Operand::Const {
                    ty: to,
                    value,
                    untyped: false,
                });
            }
        }
        if matches!(
            op,
            Operand::Const {
                value: Value::String(_),
                ..
            }
        ) && self.ty.types.pointee(to) == Some(TypeId::U8)
        {
            return self.convert(f, op, to, span);
        }
        if let Operand::Type(t) = op {
            if to == TypeId::TYPE {
                return Ok(Operand::Type(t));
            }
            // cast(*Type_Info) T: the descriptor address.
            let v = self.materialize(f, &Value::Type(t), TypeId::TYPE, span)?;
            return Ok(Operand::Value {
                ty: to,
                val: v,
            });
        }
        let untyped = matches!(
            op,
            Operand::Const {
                untyped: true,
                ..
            }
        );
        // A number, enum or pointer cast to `bool` tests for non-zero; keeping its low byte
        // would make `cast(bool) 2` a bool that `!` does not negate.
        if self.ty.types.repr(to) == TypeId::BOOL
            && self.ty.types.repr(from) != TypeId::BOOL
            && self.ir_ty(self.ty.types.repr(from)).is_some()
            && !untyped
        {
            let (ty, v) = self.rvalue(f, op, span)?;
            let val = self.truthy(f, ty, v, span)?;
            return Ok(Operand::Value {
                ty: to,
                val,
            });
        }
        if self.implicit_cost(from, untyped, to).is_some()
            && !self.ty.types.is_integer(to)
            && !self.ty.types.is_float(to)
        {
            return self.convert(f, op, to, span);
        }
        self.cast_between_representations(f, op, to, flags, untyped, span)
    }

    /// The value of the constant `value` (of type `from`) cast to `to`, when the cast can be
    /// done at compile time.
    fn fold_cast(
        &self,
        value: &Value,
        from: TypeId,
        to: TypeId,
        flags: ast::CastFlags,
    ) -> Option<Value> {
        let tr = self.ty.types.repr(to);
        let fr = self.ty.types.repr(from);
        match value {
            Value::Int(i)
                if self.ty.types.is_pointer(tr)
                    && (self.ty.types.is_integer(fr) || self.ty.types.is_pointer(fr)) =>
            {
                // A raw address is a scalar bit pattern, not a request to capture
                // pointed-to memory. Use the target width, never the host's usize.
                let bits = (self.ty.types.pointer_size * 8) as u8;
                let address = expr::wrap_int(*i, bits, false);
                Some(if address == 0 {
                    Value::Null
                } else {
                    Value::Int(address)
                })
            }
            // `cast,force` between same-size ints and floats reinterprets the bits.
            Value::Int(i) if flags.force && self.ty.types.is_float(tr) => {
                Some(Value::float(if tr == TypeId::F32 {
                    f32::from_bits(*i as u32) as f64
                } else {
                    f64::from_bits(*i as u64)
                }))
            }
            Value::Float(x, _) if flags.force && self.ty.types.is_integer(tr) => {
                let bits = if fr == TypeId::F32 {
                    (*x as f32).to_bits() as i128
                } else {
                    x.to_bits() as i128
                };
                let (width, signed) = self.ty.types.int_info(tr).unwrap();
                Some(Value::Int(expr::wrap_int(bits, width, signed)))
            }
            Value::Int(i) if self.ty.types.is_integer(tr) => {
                let (bits, signed) = self.ty.types.int_info(tr).unwrap();
                Some(Value::Int(expr::wrap_int(*i, bits, signed)))
            }
            Value::Int(i) if self.ty.types.is_float(tr) => Some(Value::float(int_to_float(*i, tr))),
            Value::Float(x, _) if self.ty.types.is_float(tr) => {
                Some(Value::float(if tr == TypeId::F32 {
                    *x as f32 as f64
                } else {
                    *x
                }))
            }
            Value::Float(x, _) if self.ty.types.is_integer(tr) => {
                let (bits, signed) = self.ty.types.int_info(tr).unwrap();
                Some(Value::Int(expr::wrap_int(*x as i128, bits, signed)))
            }
            Value::Bool(b) if self.ty.types.is_integer(tr) => Some(Value::Int(*b as i128)),
            Value::Int(i) if tr == TypeId::BOOL => Some(Value::Bool(*i != 0)),
            Value::Null if self.ir_ty(tr) == Some(Ty::Ptr) => Some(Value::Null),
            _ => None,
        }
    }

    /// The casts left once constants, procedures, types and `bool` are dealt with: reinterpreting
    /// bits, strings and arrays, and aggregates of the same size.
    fn cast_between_representations(
        &mut self,
        f: &mut FnCtx,
        op: Operand,
        to: TypeId,
        flags: ast::CastFlags,
        untyped: bool,
        span: Span,
    ) -> Result<Operand> {
        let from = op.ty();
        // Same-representation reinterpretations (distinct/enum/struct-of-same-size are not allowed except via pointer).
        let fr = self.ty.types.repr(from);
        let tr = self.ty.types.repr(to);
        if flags.force
            && let (Some(fi), Some(ti)) = (self.ir_ty(fr), self.ir_ty(tr))
            && fi.is_float() != ti.is_float()
            && self.size_of(fr, span)? == self.size_of(tr, span)?
        {
            let (_, v) = self.rvalue(f, op, span)?;
            let val = f.b.conv(ConvOp::Bitcast, fi, ti, v);
            return Ok(Operand::Value {
                ty: to,
                val,
            });
        }
        // A string constant cast to a fixed byte array of matching length.
        if let Operand::Const {
            value: Value::String(bytes),
            ..
        } = &op
            && let TypeKind::Array {
                elem,
                kind: ArrayKind::Fixed(n),
            } = self.ty.types.kind(tr).clone()
            && (elem == TypeId::U8 || elem == TypeId::S8)
            && n as usize == bytes.len()
        {
            return Ok(Operand::Const {
                ty: to,
                value: Value::Bytes(std::rc::Rc::new(crate::sema::value::Aggregate {
                    bytes: bytes.to_vec(),
                    relocs: Vec::new(),
                })),
                untyped: false,
            });
        }
        // Strings and arrays cast to `bool` as "is non-empty".
        if tr == TypeId::BOOL && self.ir_ty(fr).is_none() && fr != TypeId::ANY {
            let (ty, v) = self.rvalue(f, op, span)?;
            let val = self.truthy(f, ty, v, span)?;
            return Ok(Operand::Value {
                ty: to,
                val,
            });
        }
        if self.ir_ty(fr).is_some() && self.ir_ty(tr).is_some() {
            let op = self.settle_untyped(op, Some(to));
            return self.scalar_convert(f, op, to, span);
        }
        if fr == tr {
            let (_, v) = self.rvalue(f, op, span)?;
            return Ok(Operand::Value {
                ty: to,
                val: v,
            });
        }
        // Array conversions.
        if let (
            TypeKind::Array {
                ..
            },
            TypeKind::Array {
                kind: ArrayKind::View,
                ..
            },
        ) = (
            self.ty.types.kind(fr).clone(),
            self.ty.types.kind(tr).clone(),
        ) {
            let (_, addr) = self.address_of(f, op, span)?;
            return Ok(Operand::Value {
                ty: to,
                val: addr,
            });
        }
        if self.implicit_cost(from, untyped, to).is_some() {
            return self.convert(f, op, to, span);
        }
        // `string` and `[] u8` share the same layout (count, data).
        let is_bytes_view = |s: &Self, t: TypeId| {
            matches!(
                s.ty.types.kind(t),
                TypeKind::Array {
                    elem,
                    kind: ArrayKind::View,
                } if *elem == TypeId::U8
            )
        };
        if (from == TypeId::STRING && is_bytes_view(self, to))
            || (to == TypeId::STRING && is_bytes_view(self, from))
        {
            let (_, addr) = self.address_of(f, op, span)?;
            return Ok(Operand::Place {
                ty: to,
                addr,
            });
        }
        // `xx bytes` where `bytes: [N] u8` / `[..] u8`: view the array, then as a string.
        if to == TypeId::STRING
            && let TypeKind::Array {
                elem: TypeId::U8, ..
            } = self.ty.types.kind(fr)
        {
            let view = self.ty.types.array(TypeId::U8, ArrayKind::View);
            let op = self.explicit_cast(f, op, view, flags, span)?;
            return self.explicit_cast(f, op, to, flags, span);
        }
        // `cast,force` reinterprets an aggregate as another of the same size.
        if flags.force
            && self.ir_ty(fr).is_none()
            && self.ir_ty(tr).is_none()
            && self.size_of(from, span)? == self.size_of(to, span)?
        {
            let (_, addr) = self.address_of(f, op, span)?;
            return Ok(Operand::Place {
                ty: to,
                addr,
            });
        }
        let mut d = Diagnostic::error(
            span,
            format!(
                "cannot cast {} to {}",
                self.ty.types.name(from),
                self.ty.types.name(to)
            ),
        );
        // Aggregates of one size can be reinterpreted when the author asks for it.
        let same_size = self.ir_ty(fr).is_none()
            && self.ir_ty(tr).is_none()
            && matches!(
                (self.size_of(from, span), self.size_of(to, span)),
                (Ok(a), Ok(b)) if a == b
            );
        if same_size
            && let Some(suggestion) = self.cast_flag_suggestion(
                span,
                ",force",
                "both have the same size: `cast,force` reinterprets the bytes".into(),
            )
        {
            d = d.with_suggestion(suggestion);
        } else if let Some(help) = self.cannot_cast_help(from, to) {
            d = d.with_help(help);
        }
        Err(Box::new(d))
    }
}

impl SourceState {
    /// Whether the integer constant at `span` may stand for a bit pattern: it is not a plain
    /// decimal expression. Hex and binary literals, `~` and named constants count; a constant
    /// without source text is given the benefit of the doubt.
    pub(crate) fn bit_pattern(&self, span: Span) -> bool {
        let text = self.sources.snippet(span);
        text.is_empty() || text.bytes().any(|b| b.is_ascii_alphabetic() || b == b'~')
    }
}
