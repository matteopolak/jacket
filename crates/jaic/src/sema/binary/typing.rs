//! Literal fitting and the type a binary expression takes.
use crate::sema::expr::wrap_int;
use crate::sema::lower::Operand;
use crate::sema::{Compiler, Diagnostic, Result, Span, TypeId, Value};
use crate::types::TypeKind;

impl Compiler {
    /// The type an untyped literal takes next to a typed operand: the operand's
    /// type, or a 64-bit integer when the literal does not fit (`small + -1` and
    /// `small == -1` with a `u8` work in `s64`). A hex, binary or `~` literal
    /// (`bit_pattern`) may still fill the operand's bits.
    pub(in crate::sema) fn literal_meets(
        &self,
        lit: &Operand,
        ty: TypeId,
        bit_pattern: bool,
    ) -> TypeId {
        if let Operand::Const {
            value: Value::Int(v),
            ..
        } = lit
            && let Some((bits, signed)) = self.types.int_info(ty)
            && !crate::sema::convert::int_constant_fits(*v, bits, signed, bit_pattern)
            && !(bit_pattern && crate::sema::convert::int_fits(*v, bits, signed))
            && !matches!(self.types.kind(ty), TypeKind::Enum(_))
        {
            return if crate::sema::convert::int_fits(*v, 64, true) {
                TypeId::S64
            } else {
                TypeId::U64
            };
        }
        ty
    }

    /// An untyped integer literal operand of integer type `ty`, wrapped to `ty`'s bits when
    /// it only fits as a bit pattern.
    pub(in crate::sema) fn literal_as_bits(&self, op: Operand, ty: TypeId) -> Operand {
        match op {
            Operand::Const {
                value: Value::Int(v),
                untyped: true,
                ..
            } if !matches!(self.types.kind(ty), TypeKind::Enum(_))
                && let Some((bits, signed)) = self.types.int_info(ty)
                && bits <= 64
                && crate::sema::convert::int_fits(v, bits, signed) =>
            {
                Operand::Const {
                    ty,
                    value: Value::Int(wrap_int(v, bits, signed)),
                    untyped: false,
                }
            }
            op => op,
        }
    }

    pub(in crate::sema) fn shift_amount_type(&self, rhs: &Operand, lhs_ty: TypeId) -> TypeId {
        match rhs {
            Operand::Const {
                untyped: true, ..
            } => lhs_ty,
            other => {
                let t = other.ty();
                if self.types.is_integer(t) {
                    t
                } else {
                    lhs_ty
                }
            }
        }
    }

    pub(in crate::sema) fn binary_operand_type(
        &mut self,
        lhs: &Operand,
        rhs: &Operand,
        expected: Option<TypeId>,
        is_shift: bool,
        patterns: (bool, bool),
        span: Span,
    ) -> Result<TypeId> {
        let lu = matches!(
            lhs,
            Operand::Const {
                untyped: true,
                ..
            }
        );
        let ru = matches!(
            rhs,
            Operand::Const {
                untyped: true,
                ..
            }
        );
        let (lt, rt) = (lhs.ty(), rhs.ty());
        if is_shift {
            if lu {
                return Ok(expected
                    .filter(|&e| self.types.is_integer(e))
                    .unwrap_or(TypeId::S64));
            }
            return Ok(lt);
        }
        match (lu, ru) {
            (true, true) => {
                let lf = matches!(
                    lhs,
                    Operand::Const {
                        value: Value::Float(_, _),
                        ..
                    }
                );
                let rf = matches!(
                    rhs,
                    Operand::Const {
                        value: Value::Float(_, _),
                        ..
                    }
                );
                if lt == TypeId::NULL || rt == TypeId::NULL {
                    return Ok(TypeId::VOID_PTR);
                }
                Ok(if lf || rf {
                    let natural = if [lhs, rhs].iter().any(|o| {
                        o.const_value()
                            .is_some_and(|v| self.default_untyped(o.ty(), &v) == TypeId::F64)
                    }) {
                        TypeId::F64
                    } else {
                        TypeId::F32
                    };
                    expected
                        .filter(|&e| self.types.is_float(e))
                        .unwrap_or(natural)
                } else {
                    expected
                        .filter(|&e| self.types.is_integer(e) || self.types.is_float(e))
                        .unwrap_or(TypeId::S64)
                })
            }
            (true, false) => Ok(self.literal_meets(lhs, rt, patterns.0)),
            (false, true) => Ok(self.literal_meets(rhs, lt, patterns.1)),
            (false, false) => {
                if lt == rt {
                    return Ok(lt);
                }
                // Widening within the same class.
                if self.implicit_cost(rt, false, lt).is_some() {
                    return Ok(lt);
                }
                if self.implicit_cost(lt, false, rt).is_some() {
                    return Ok(rt);
                }
                // An integer meeting a float in arithmetic becomes that float.
                if self.types.is_float(lt) && self.types.is_integer(rt) {
                    return Ok(lt);
                }
                if self.types.is_integer(lt) && self.types.is_float(rt) {
                    return Ok(rt);
                }
                let (li, ri) = (self.types.int_info(lt), self.types.int_info(rt));
                if let (Some((lb, _)), Some((rb, _))) = (li, ri) {
                    // Mixed signedness: Jai requires a cast, but comparisons with
                    // literals are common; pick the wider type.
                    return Ok(if lb >= rb {
                        lt
                    } else {
                        rt
                    });
                }
                // A procedure value meets a `*void` as an address (`proc == get_caller_address()`).
                let is_proc = |c: &Self, t| matches!(c.types.kind(t), TypeKind::Proc(_));
                if (is_proc(self, lt) && rt == TypeId::VOID_PTR)
                    || (is_proc(self, rt) && lt == TypeId::VOID_PTR)
                {
                    return Ok(TypeId::VOID_PTR);
                }
                let mut d = Diagnostic::error(
                    span,
                    format!(
                        "type mismatch: `{}` and `{}` cannot be combined",
                        self.types.name(lt),
                        self.types.name(rt)
                    ),
                );
                // `c * "2"`: a one-character string where a character code was meant (`+` and `-`
                // take it as the byte).
                let one_char = |o: &Operand| match o {
                    Operand::Const {
                        value: Value::String(s),
                        ..
                    } if s.len() == 1 => Some(s[0] as char),
                    _ => None,
                };
                let (int_side, text) = match (one_char(lhs), one_char(rhs)) {
                    (_, Some(c)) => (lt, Some(c)),
                    (Some(c), _) => (rt, Some(c)),
                    _ => (lt, None),
                };
                if let Some(c) = text
                    && self.types.is_integer(int_side)
                {
                    d = d.with_help(format!(
                        "`\"{c}\"` is a string; for the character's code write `#char \"{c}\"`"
                    ));
                } else if self.types.is_integer(lt) && self.types.is_float(rt)
                    || self.types.is_float(lt) && self.types.is_integer(rt)
                {
                    d = d.with_help("convert one side with `cast(float)` or `cast(int)` first");
                } else if self.types.is_integer(lt) && self.types.is_integer(rt) {
                    d = d.with_help(
                        "integers of different sizes or signedness need a `cast` to one type",
                    );
                }
                Err(Box::new(d))
            }
        }
    }
}
