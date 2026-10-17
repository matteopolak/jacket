//! Lowering of the SIMD intrinsics (`ir::Intrinsic::ByteLookup`, `SignMask`) to LLVM IR.
//!
//! Each intrinsic is one function here that tries the target backends from the most to the
//! least specific and ends in the portable one (`generic`), which works everywhere. A backend
//! returns `Ok(true)` when it emitted the operation and `Ok(false)` when the target lacks
//! what it needs; it picks by the typed `features::Target`, never by a CPU name.
//! See docs/native/llvm-backend.md for how to add an intrinsic or an instruction set.

mod aarch64;
mod features;
mod generic;
mod wasm;
mod x86;

pub use features::Target;

use crate::lower::Error;
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::intrinsics::Intrinsic as LlvmIntrinsic;
use inkwell::module::Module;
use inkwell::types::{BasicTypeEnum, VectorType};
use inkwell::values::{
    BasicMetadataValueEnum, BasicValue, BasicValueEnum, IntValue, PointerValue, ValueKind,
    VectorValue,
};

type R<T> = Result<T, Error>;

/// What a backend needs from the function being lowered.
pub struct Cx<'a, 'ctx> {
    pub ctx: &'ctx Context,
    pub builder: &'a Builder<'ctx>,
    pub module: &'a Module<'ctx>,
    pub target: &'a Target,
}

impl<'a, 'ctx> Cx<'a, 'ctx> {
    /// An unaligned load of `lanes` bytes.
    pub(crate) fn load_bytes(&self, ptr: PointerValue<'ctx>, lanes: u32) -> R<VectorValue<'ctx>> {
        let ty = self.ctx.i8_type().vec_type(lanes);
        let load = self.builder.build_load(ty, ptr, "")?;
        if let Some(i) = load.as_instruction_value() {
            let _ = i.set_alignment(1);
        }
        Ok(load.into_vector_value())
    }

    /// An unaligned store.
    pub(crate) fn store_bytes(&self, ptr: PointerValue<'ctx>, v: BasicValueEnum<'ctx>) -> R<()> {
        let store = self.builder.build_store(ptr, v)?;
        let _ = store.set_alignment(1);
        Ok(())
    }

    /// `base + offset` bytes.
    pub(crate) fn offset(&self, base: PointerValue<'ctx>, offset: u64) -> R<PointerValue<'ctx>> {
        if offset == 0 {
            return Ok(base);
        }
        let at = self.ctx.i64_type().const_int(offset, false);
        // Inkwell marks GEP construction unsafe; an i8 GEP is plain pointer arithmetic.
        #[allow(unsafe_code)]
        let p = unsafe {
            self.builder
                .build_gep(self.ctx.i8_type(), base, &[at], "")?
        };
        Ok(p)
    }

    /// A vector of `lanes` bytes all equal to `v`.
    pub(crate) fn splat(&self, lanes: u32, v: u64) -> VectorValue<'ctx> {
        VectorType::const_vector(&vec![
            self.ctx.i8_type().const_int(v, false);
            lanes as usize
        ])
    }

    /// `table` (16 bytes) repeated to fill `lanes` bytes.
    pub(crate) fn repeat_table(
        &self,
        table: VectorValue<'ctx>,
        lanes: u32,
    ) -> R<VectorValue<'ctx>> {
        if lanes == 16 {
            return Ok(table);
        }
        let mask: Vec<IntValue> = (0..lanes)
            .map(|i| self.ctx.i32_type().const_int(u64::from(i % 16), false))
            .collect();
        Ok(self
            .builder
            .build_shuffle_vector(table, table, VectorType::const_vector(&mask), "")?)
    }

    /// A call to the LLVM intrinsic `name` (overloaded on `tys`) that returns a value.
    pub(crate) fn call(
        &self,
        name: &str,
        tys: &[BasicTypeEnum<'ctx>],
        args: &[BasicMetadataValueEnum<'ctx>],
    ) -> R<BasicValueEnum<'ctx>> {
        let f = LlvmIntrinsic::find(name)
            .and_then(|i| i.get_declaration(self.module, tys))
            .ok_or_else(|| Error::from(format!("LLVM intrinsic '{name}' is unavailable")))?;
        match self.builder.build_call(f, args, "")?.try_as_basic_value() {
            ValueKind::Basic(v) => Ok(v),
            ValueKind::Instruction(_) => Err(format!("'{name}' produced no value").into()),
        }
    }

    /// Run `lookup` on every `chunk`-byte block of the `width` index bytes at `idx` and store
    /// the results at `dst`.
    pub(crate) fn map_blocks(
        &self,
        width: u8,
        chunk: u32,
        idx: PointerValue<'ctx>,
        dst: PointerValue<'ctx>,
        mut lookup: impl FnMut(VectorValue<'ctx>) -> R<BasicValueEnum<'ctx>>,
    ) -> R<()> {
        for block in 0..u32::from(width) / chunk {
            let at = u64::from(block * chunk);
            let indices = self.load_bytes(self.offset(idx, at)?, chunk)?;
            let res = lookup(indices)?;
            self.store_bytes(self.offset(dst, at)?, res)?;
        }
        Ok(())
    }
}

/// `ByteLookup`: `dst[i] = idx[i] < 16 ? table[idx[i]] : 0`, for `width` bytes (16, 32 or 64)
/// with the same 16-byte `table` for every 16-byte block.
pub fn byte_lookup<'ctx>(
    cx: &Cx<'_, 'ctx>,
    width: u8,
    dst: PointerValue<'ctx>,
    table: PointerValue<'ctx>,
    idx: PointerValue<'ctx>,
) -> R<()> {
    let table = cx.load_bytes(table, 16)?;
    // Most specific first; each declines (`false`) when the target lacks its instructions.
    let _ = x86::byte_lookup(cx, width, dst, table, idx)?
        || aarch64::byte_lookup(cx, width, dst, table, idx)?
        || wasm::byte_lookup(cx, width, dst, table, idx)?
        || generic::byte_lookup(cx, width, dst, table, idx)?;
    Ok(())
}

/// `SignMask`: bit `i` of the result is the top bit of byte `i` of the `width` bytes at `src`.
pub fn sign_mask<'ctx>(cx: &Cx<'_, 'ctx>, width: u8, src: PointerValue<'ctx>) -> R<IntValue<'ctx>> {
    // The portable form (`icmp slt`, bitcast `<N x i1>` to `iN`) is already `pmovmskb` on x86
    // and a short shift/add sequence elsewhere; a target-specific backend goes first here.
    generic::sign_mask(cx, width, src)
}
