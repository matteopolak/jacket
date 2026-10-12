//! WebAssembly: `i8x16.swizzle` with `simd128`.

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
    if cx.target.family != Family::Wasm || !cx.target.has(Feature::Simd128) {
        return Ok(false);
    }
    // swizzle also gives 0 for an index of 16 or more.
    cx.map_blocks(width, 16, idx, dst, |indices| {
        cx.call("llvm.wasm.swizzle", &[], &[table.into(), indices.into()])
    })?;
    Ok(true)
}
