//! x86-64: `pshufb` in its 128, 256 (AVX2) and 512-bit (AVX-512BW) forms.

use super::features::{Family, Feature};
use super::{Cx, R};
use inkwell::values::{PointerValue, VectorValue};

/// The widest `pshufb` the target has for `width` bytes: the block size and the LLVM intrinsic.
fn pshufb(cx: &Cx<'_, '_>, width: u8) -> Option<(u32, &'static str)> {
    let has = |f| cx.target.has(f);
    if width >= 64 && has(Feature::Avx512bw) {
        Some((64, "llvm.x86.avx512.pshuf.b.512"))
    } else if width >= 32 && has(Feature::Avx2) {
        Some((32, "llvm.x86.avx2.pshuf.b"))
    } else if has(Feature::Ssse3) {
        Some((16, "llvm.x86.ssse3.pshuf.b.128"))
    } else {
        None
    }
}

pub fn byte_lookup<'ctx>(
    cx: &Cx<'_, 'ctx>,
    width: u8,
    dst: PointerValue<'ctx>,
    table: VectorValue<'ctx>,
    idx: PointerValue<'ctx>,
) -> R<bool> {
    if cx.target.family != Family::X86 {
        return Ok(false);
    }
    let Some((chunk, intrinsic)) = pshufb(cx, width) else {
        return Ok(false);
    };
    // pshufb looks up within each 16-byte lane, so the table is repeated per lane.
    let table = cx.repeat_table(table, chunk)?;
    let ty = cx.ctx.i8_type().vec_type(chunk);
    cx.map_blocks(width, chunk, idx, dst, |indices| {
        // pshufb zeroes on the top bit only; a saturating add of 0x70 sets it for every index
        // of 16 or more and leaves 0..15 alone.
        let sat = cx.call(
            "llvm.uadd.sat",
            &[ty.into()],
            &[indices.into(), cx.splat(chunk, 0x70).into()],
        )?;
        cx.call(intrinsic, &[], &[table.into(), sat.into()])
    })?;
    Ok(true)
}
