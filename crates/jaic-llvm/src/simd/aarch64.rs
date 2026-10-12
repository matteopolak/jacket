//! AArch64: NEON `tbl`. SVE and SVE2 have no entry yet: NEON runs on every SVE machine, and
//! the 16-byte table form is what `byte_lookup` needs. A scalable-vector path would test
//! `Feature::Sve` here before the NEON one.

use super::features::{Family, Feature};
use super::{Cx, R};
use inkwell::values::{PointerValue, VectorValue};

pub fn byte_lookup<'ctx>(
    cx: &Cx<'_, 'ctx>,
    width: u8,
    dst: PointerValue<'ctx>,
    table: VectorValue<'ctx>,
    idx: PointerValue<'ctx>,
) -> R<bool> {
    if cx.target.family != Family::Aarch64 || !cx.target.has(Feature::Neon) {
        return Ok(false);
    }
    let ty = cx.ctx.i8_type().vec_type(16);
    // tbl already gives 0 for an index of 16 or more.
    cx.map_blocks(width, 16, idx, dst, |indices| {
        cx.call(
            "llvm.aarch64.neon.tbl1",
            &[ty.into()],
            &[table.into(), indices.into()],
        )
    })?;
    Ok(true)
}
