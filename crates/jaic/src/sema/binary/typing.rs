//! Literal fitting and the type a binary expression takes.
use crate::ast::Expr;
use crate::diagnostics::Code;
use crate::int_arith::wrap;
use crate::sema::TypeState;
use crate::sema::convert::{Site, Wrap, wrap_suggestion};
use crate::sema::lower::{Operand, Origin};
use crate::sema::{Compiler, Diagnostic, Result, Span, TypeId, Value};
use crate::types::TypeKind;

impl Compiler {
    pub(in crate::sema) fn binary_operand_type(
        &mut self,
        lhs: &Operand,
        rhs: &Operand,
        expected: Option<TypeId>,
        is_shift: bool,
        operands: (&Expr, &Expr),
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
        let patterns = (lhs.bit_pattern(), rhs.bit_pattern());
        if is_shift {
            if lu {
                return Ok(expected
                    .filter(|&e| self.ty.types.is_integer(e))
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
                            .is_some_and(|v| self.ty.default_untyped(o.ty(), &v) == TypeId::F64)
                    }) {
                        TypeId::F64
                    } else {
                        TypeId::F32
                    };
                    expected
                        .filter(|&e| self.ty.types.is_float(e))
                        .unwrap_or(natural)
                } else {
                    expected
                        .filter(|&e| self.ty.types.is_integer(e) || self.ty.types.is_float(e))
                        .unwrap_or(TypeId::S64)
                })
            }
            (true, false) => Ok(self.ty.literal_meets(lhs, rt, patterns.0)),
            (false, true) => Ok(self.ty.literal_meets(rhs, lt, patterns.1)),
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
                if self.ty.types.is_float(lt) && self.ty.types.is_integer(rt) {
                    return Ok(lt);
                }
                if self.ty.types.is_integer(lt) && self.ty.types.is_float(rt) {
                    return Ok(rt);
                }
                let (li, ri) = (self.ty.types.int_info(lt), self.ty.types.int_info(rt));
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
                let is_proc = |c: &Self, t| matches!(c.ty.types.kind(t), TypeKind::Proc(_));
                if (is_proc(self, lt) && rt == TypeId::VOID_PTR)
                    || (is_proc(self, rt) && lt == TypeId::VOID_PTR)
                {
                    return Ok(TypeId::VOID_PTR);
                }
                let mut d = Diagnostic::new(
                    Code::BinaryTypeMismatch,
                    span,
                    format!(
                        "type mismatch: `{}` and `{}` cannot be combined",
                        self.ty.types.name(lt),
                        self.ty.types.name(rt)
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
                    && self.ty.types.is_integer(int_side)
                {
                    d = d.with_help(format!(
                        "`\"{c}\"` is a string; for the character's code write `#char \"{c}\"`"
                    ));
                } else if let Some((side, to)) = self.binary_cast_side(lt, rt) {
                    let at = Site::from([operands.0, operands.1][side]);
                    let to_name = self.ty.types.name(to);
                    let title = format!(
                        "convert the {} operand with `cast({to_name})`",
                        ["left", "right"][side]
                    );
                    let wrap = Wrap::cast(to_name);
                    d = match wrap_suggestion(&self.src.sources, title.clone(), at, &wrap) {
                        Some(suggestion) => d.with_suggestion(suggestion),
                        None => d.with_help(title),
                    };
                }
                Err(Box::new(d))
            }
        }
    }
}

impl Compiler {
    /// Which operand of a number-like pair (integer, float, bool or enum) converts to the
    /// other's type: the side to cast (0 left, 1 right) and the type to cast it to. A float
    /// wins over an integer, a wider integer over a narrower, an integer over a `bool`.
    fn binary_cast_side(&self, lt: TypeId, rt: TypeId) -> Option<(usize, TypeId)> {
        let types = &self.ty.types;
        let number = |t| types.is_integer(t) || types.is_float(t) || types.repr(t) == TypeId::BOOL;
        if !number(lt) || !number(rt) {
            return None;
        }
        let rank = |t| {
            if types.is_float(t) {
                (3, 0)
            } else if types.repr(t) == TypeId::BOOL {
                (0, 0)
            } else {
                (2, types.int_info(t).map_or(0, |(bits, _)| bits))
            }
        };
        // The side with the lower rank is converted; the left one when equal.
        Some(if rank(rt) > rank(lt) {
            (0, rt)
        } else {
            (1, lt)
        })
    }
}

impl TypeState {
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
        // A `float32` operand meeting a float literal with more digits than `float32` keeps
        // (`x + 0.1591549431`) widens the operation to `float64`.
        if let Operand::Const {
            value,
            untyped: true,
            ..
        } = lit
            && value.requires_float64()
            && self.types.repr(ty) == TypeId::F32
        {
            return TypeId::F64;
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
                    value: Value::Int(wrap(v, bits, signed)),
                    untyped: false,
                    origin: Origin::Computed,
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
}
