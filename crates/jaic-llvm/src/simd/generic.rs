//! The portable lowering: plain LLVM IR that works on every target and is the reference for
//! what the target-specific backends must compute.

use super::{Cx, R};
use inkwell::IntPredicate;
use inkwell::values::{BasicValueEnum, IntValue, PointerValue, VectorValue};

/// `ByteLookup` one lane at a time (`extractelement`/`select`/`insertelement`).
pub fn byte_lookup<'ctx>(
    cx: &Cx<'_, 'ctx>,
    width: u8,
    dst: PointerValue<'ctx>,
    table: VectorValue<'ctx>,
    idx: PointerValue<'ctx>,
) -> R<bool> {
    let b = cx.builder;
    let i8t = cx.ctx.i8_type();
    let i32t = cx.ctx.i32_type();
    let zero = i8t.const_zero();
    cx.map_blocks(width, 16, idx, dst, |indices| {
        let mut out = i8t.vec_type(16).get_undef();
        for lane in 0..16u64 {
            let at = i32t.const_int(lane, false);
            let i = b.build_extract_element(indices, at, "")?.into_int_value();
            let in_range =
                b.build_int_compare(IntPredicate::ULT, i, i8t.const_int(16, false), "")?;
            let sel = b.build_and(i, i8t.const_int(15, false), "")?;
            let sel = b.build_int_z_extend(sel, i32t, "")?;
            let v = b.build_extract_element(table, sel, "")?.into_int_value();
            let v = b.build_select(in_range, v, zero, "")?.into_int_value();
            out = b.build_insert_element(out, v, at, "")?;
        }
        Ok(BasicValueEnum::from(out))
    })?;
    Ok(true)
}

/// `SignMask` as a vector compare and a bitcast of the `<N x i1>` to `iN`.
pub fn sign_mask<'ctx>(cx: &Cx<'_, 'ctx>, width: u8, src: PointerValue<'ctx>) -> R<IntValue<'ctx>> {
    let b = cx.builder;
    let v = cx.load_bytes(src, u32::from(width))?;
    let zero = cx.ctx.i8_type().vec_type(u32::from(width)).const_zero();
    let neg = b.build_int_compare(IntPredicate::SLT, v, zero, "")?;
    let bits = cx
        .ctx
        .custom_width_int_type(std::num::NonZeroU32::new(u32::from(width)).ok_or("zero width")?)?;
    let packed = b.build_bit_cast(neg, bits, "")?.into_int_value();
    Ok(b.build_int_z_extend_or_bit_cast(packed, cx.ctx.i64_type(), "")?)
}
