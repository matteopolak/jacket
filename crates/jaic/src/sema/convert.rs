//! Implicit conversions (with overload costs) and explicit casts.
mod casts;
mod coerce;
mod cost;
mod implicit;

use super::lower::{FnCtx, Operand};
use super::{Compiler, Result, Span, TypeId, Value, ir};
use crate::ir::{ConvOp, Ty};

/// Conversion costs used to rank overloads. Lower is better.
pub const EXACT: u32 = 0;

pub const LITERAL: u32 = 1;
pub const WIDEN: u32 = 2;
pub const POINTER: u32 = 3;
pub const SUBTYPE: u32 = 4;
pub const INT_TO_FLOAT: u32 = 5;
pub const TO_ANY: u32 = 6;

impl Compiler {
    /// Box a value into an `Any` (type info pointer + pointer to a copy).
    pub fn box_any(&mut self, f: &mut FnCtx, op: Operand, span: Span) -> Result<Operand> {
        if op.ty() == TypeId::ANY {
            return Ok(op);
        }
        let (ty, addr) = match op {
            Operand::Type(t) => {
                self.require_value_type(t, span)?;
                let v = self.materialize(f, &Value::Type(t), TypeId::TYPE, span)?;
                (TypeId::TYPE, self.spill(f, TypeId::TYPE, v, span)?)
            }
            // The `Any` points at the place itself, so nothing reads it here, not even a null
            // check: `p.*` of a null `p` boxes fine, and only a reader of the value stops
            // (`print` does). jaison passes such a value to an `assert` that holds.
            Operand::Place {
                ty,
                addr,
            } => (ty, addr),
            other => {
                let other = self.settle_untyped(other, None);
                let (ty, v) = self.rvalue(f, other, span)?;
                (ty, self.spill(f, ty, v, span)?)
            }
        };
        let info = self.type_info_global(ty, span)?;
        let any = f.b.alloca(16, 8);
        let ti = f.b.global_addr(info);
        f.b.store(Ty::Ptr, any, ti);
        let vp = f.b.ptr_offset(any, 8);
        f.b.store(Ty::Ptr, vp, addr);
        Ok(Operand::Value {
            ty: TypeId::ANY,
            val: any,
        })
    }
}

/// Whether an untyped integer constant converts to an integer type without a cast. Negative
/// values never go to an unsigned type, and a signed type takes the signed range only, except
/// that a `bit_pattern` constant (written in hex or binary, through `~`, or by name) may fill
/// all the bits: `x: s8 = 0xff` is `-1`, `x: s8 = 255` is an error. `~0` also fills a `u64`.
pub fn int_constant_fits(v: i128, bits: u8, signed: bool, bit_pattern: bool) -> bool {
    if bits >= 128 {
        return true;
    }
    if signed && bits == 64 {
        // Every 64-bit literal fits `s64`: `9223372036854775808` is `S64_MIN`.
        return v >= i64::MIN as i128 && v <= u64::MAX as i128;
    }
    let span = 1i128 << bits;
    if signed {
        let half = span >> 1;
        (-half..half).contains(&v) || (bit_pattern && (0..span).contains(&v))
    } else {
        (0..span).contains(&v) || (bit_pattern && bits == 64 && v >= i64::MIN as i128 && v < 0)
    }
}

pub fn int_fits(v: i128, bits: u8, signed: bool) -> bool {
    if bits >= 64 {
        // Accept both the signed and unsigned spelling of 64-bit constants.
        return v >= i64::MIN as i128 && v <= u64::MAX as i128;
    }
    if signed {
        let max = (1i128 << (bits - 1)) - 1;
        // Also accept bit patterns written as unsigned literals (e.g. 0xFFFF_FFFF for s32).
        v >= -max - 1 && v <= ((1i128 << bits) - 1)
    } else {
        v >= -(1i128 << (bits - 1)) && v < (1i128 << bits)
    }
}

/// Convert a scalar register between classes.
pub fn numeric_conv(
    f: &mut FnCtx,
    v: ir::Val,
    ft: Ty,
    tt: Ty,
    fsigned: bool,
    tsigned: bool,
) -> ir::Val {
    let _ = tsigned;
    if ft == tt {
        return v;
    }
    match (ft.is_float(), tt.is_float()) {
        (true, true) => f.b.conv(
            if tt.size() > ft.size() {
                ConvOp::FExt
            } else {
                ConvOp::FTrunc
            },
            ft,
            tt,
            v,
        ),
        (true, false) => {
            if tt == Ty::Ptr {
                let i = f.b.conv(ConvOp::FToS, ft, Ty::I64, v);
                return f.b.conv(ConvOp::Bitcast, Ty::I64, Ty::Ptr, i);
            }
            f.b.conv(
                if tsigned {
                    ConvOp::FToS
                } else {
                    ConvOp::FToU
                },
                ft,
                tt,
                v,
            )
        }
        (false, true) => {
            let v = if ft == Ty::Ptr {
                f.b.conv(ConvOp::Bitcast, Ty::Ptr, Ty::I64, v)
            } else {
                v
            };
            let ft = if ft == Ty::Ptr {
                Ty::I64
            } else {
                ft
            };
            f.b.conv(
                if fsigned {
                    ConvOp::SToF
                } else {
                    ConvOp::UToF
                },
                ft,
                tt,
                v,
            )
        }
        (false, false) => {
            if ft == Ty::Ptr || tt == Ty::Ptr {
                // Pointers are 64-bit: widen/narrow through I64.
                let (v, ft) = if ft == Ty::Ptr {
                    (f.b.conv(ConvOp::Bitcast, Ty::Ptr, Ty::I64, v), Ty::I64)
                } else {
                    (v, ft)
                };
                let v = numeric_conv(
                    f,
                    v,
                    ft,
                    if tt == Ty::Ptr {
                        Ty::I64
                    } else {
                        tt
                    },
                    fsigned,
                    tsigned,
                );
                return if tt == Ty::Ptr {
                    f.b.conv(ConvOp::Bitcast, Ty::I64, Ty::Ptr, v)
                } else {
                    v
                };
            }
            if tt.size() < ft.size() {
                f.b.conv(ConvOp::Trunc, ft, tt, v)
            } else if fsigned {
                f.b.conv(ConvOp::SExt, ft, tt, v)
            } else {
                f.b.conv(ConvOp::ZExt, ft, tt, v)
            }
        }
    }
}

/// An integer constant as a float of type `to`, rounded once to that type's precision: rounding
/// to f64 first and to f32 later would round twice (`0x8000_0080_0000_0001` would end on the
/// even f32 below instead of the one above).
fn int_to_float(i: i128, to: TypeId) -> f64 {
    if to == TypeId::F32 {
        i as f32 as f64
    } else {
        i as f64
    }
}
