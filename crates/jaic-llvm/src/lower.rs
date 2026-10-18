//! IR to LLVM IR translation.
//!
//! Every IR `Val` becomes an LLVM SSA value (IR values are defined once and
//! blocks are visited in reverse post-order, so definitions dominate uses),
//! every `Slot` an entry-block alloca, and every aggregate stays in memory.
//! `Conv::C` signatures are translated to the real C ABI by [`Backend::lower_sig`].
#![allow(clippy::too_many_arguments)]
mod checks;
mod control;
mod error;
mod function;
mod inst;
mod intrinsics;
mod plan;
mod state;
mod symbols;
mod types;

pub use self::error::Error;
pub use self::plan::{Plan, Shard, lower_program};
use crate::debuginfo::DebugInfo;
use inkwell::attributes::{Attribute, AttributeLoc};
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::{Linkage, Module};
use inkwell::values::{FunctionValue, GlobalValue, IntValue, PointerValue};
use jaic::abi::Arch;
use jaic::fxhash::HashMap;
use jaic::ir::Program;
use std::cell::{OnceCell, RefCell};

type R<T> = Result<T, Error>;

struct Backend<'ctx, 'p> {
    ctx: &'ctx Context,
    module: &'p Module<'ctx>,
    builder: Builder<'ctx>,
    program: &'p Program,
    arch: Arch,
    target: jaic::target::Target,
    shard: Option<Shard<'p>>,
    /// Functions of other modules this one holds a copy of (empty without a shard).
    imported: Vec<bool>,
    imported_globals: Vec<bool>,
    /// Functions kept out of their callers so none outgrows the optimizer's reach (`partition::inline_cuts`).
    no_inline: Vec<bool>,
    /// Declaration of each program function, made on first use: a codegen unit declares the
    /// functions it defines and the ones those refer to, not the whole program.
    funcs: Vec<OnceCell<FunctionValue<'ctx>>>,
    globals: Vec<OnceCell<GlobalValue<'ctx>>>,
    /// Address of each foreign function or variable.
    foreigns: Vec<OnceCell<PointerValue<'ctx>>>,
    /// Foreign symbols declared so far: ids with one symbol share a declaration.
    foreign_symbols: RefCell<HashMap<&'p str, PointerValue<'ctx>>>,
    exports: HashMap<&'p str, usize>,
    foreign_ids: HashMap<&'p str, usize>,
    /// `jaic.null_fail`, once a null check needs it.
    null_fail: OnceCell<FunctionValue<'ctx>>,
    /// The global holding each source file's name for failed checks (`check_file`).
    check_files: RefCell<HashMap<u32, PointerValue<'ctx>>>,
    /// Debug info builder when the program is built with debug info (`debuginfo.rs`).
    debug: Option<DebugInfo<'ctx, 'p>>,
    /// Unoptimized code: write copies and zero fills of up to `INLINE_COPY_MAX` bytes as
    /// loads and stores. `FastISel` turns a `memcpy` whose pointers are not known to be aligned
    /// into a library call (and every `memset` into one), which costs more than the copy.
    inline_small_copies: bool,
    simd: crate::simd::Target,
}

/// The largest copy or fill that unoptimized code writes inline.
const INLINE_COPY_MAX: u64 = 64;

impl<'ctx> Backend<'ctx, '_> {
    fn run(&mut self) -> R<()> {
        if self.shard.is_none() {
            // One module holds everything: declare it all, functions first.
            for i in 0..self.funcs.len() {
                self.func(i);
            }
            for i in 0..self.foreigns.len() {
                self.foreign(i);
            }
            for i in 0..self.globals.len() {
                self.global(i);
            }
        }
        // Intrinsics are declared by name as lowering meets them; a program symbol of the same
        // name has to exist first, or LLVM would rename it.
        for (&symbol, &id) in &self.foreign_ids {
            if symbol.starts_with("llvm.") {
                self.foreign(id);
            }
        }
        self.init_globals()?;
        self.describe_globals();
        if self.arch.is_wasm() && self.shard.is_none_or(|s| s.index == 0) {
            self.define_multi3()?;
        }
        for (i, func) in self.program.funcs.iter().enumerate() {
            if let Some(func) = func
                && self.owns_func(i)
            {
                let f = self.func(i).expect("declared");
                self.define_function(func, f)
                    .map_err(|e| Error(format!("in '{}': {}", func.name, e.0)))?;
            }
        }
        for (i, &import) in self.imported.iter().enumerate() {
            if import && let Some(func) = &self.program.funcs[i] {
                let f = self.func(i).expect("declared");
                self.define_function(func, f)
                    .map_err(|e| Error(format!("in '{}': {}", func.name, e.0)))?;
            }
        }
        self.init_replicated()?;
        if let Some(debug) = &self.debug {
            debug.finalize();
        }
        Ok(())
    }

    /// compiler-rt's `__multi3` (128-bit multiply), as a weak definition. wasm has no
    /// high-half multiply, so LLVM calls it for 64-bit division by a constant and similar
    /// optimizations; jaic links no compiler-rt for wasm. A real one, if linked, wins.
    /// `optnone` keeps the optimizer from recognizing the pieces as a wide multiply again.
    fn define_multi3(&self) -> R<()> {
        const NAME: &str = "__multi3";
        self.declare_named(NAME);
        if self.module.get_function(NAME).is_some() {
            return Ok(());
        }
        let i64t = self.ctx.i64_type();
        let ty = self.ctx.void_type().fn_type(
            &[
                self.ptr_ty().into(),
                i64t.into(),
                i64t.into(),
                i64t.into(),
                i64t.into(),
            ],
            false,
        );
        let f = self.module.add_function(NAME, ty, Some(Linkage::WeakAny));
        for attr in ["noinline", "optnone"] {
            let kind = Attribute::get_named_enum_kind_id(attr);
            f.add_attribute(
                AttributeLoc::Function,
                self.ctx.create_enum_attribute(kind, 0),
            );
        }
        let b = self.ctx.create_builder();
        b.position_at_end(self.ctx.append_basic_block(f, "entry"));
        let p = |i: u32| -> R<IntValue<'ctx>> {
            Ok(f.get_nth_param(i)
                .ok_or("__multi3 parameter")?
                .into_int_value())
        };
        let (a_lo, a_hi, b_lo, b_hi) = (p(1)?, p(2)?, p(3)?, p(4)?);
        let mask = i64t.const_int(0xffff_ffff, false);
        let k32 = i64t.const_int(32, false);
        // The high 64 bits of a_lo * b_lo, from four 32x32-bit products.
        let (xl, xh) = (
            b.build_and(a_lo, mask, "")?,
            b.build_right_shift(a_lo, k32, false, "")?,
        );
        let (yl, yh) = (
            b.build_and(b_lo, mask, "")?,
            b.build_right_shift(b_lo, k32, false, "")?,
        );
        let ll = b.build_int_mul(xl, yl, "")?;
        let lh = b.build_int_mul(xl, yh, "")?;
        let hl = b.build_int_mul(xh, yl, "")?;
        let hh = b.build_int_mul(xh, yh, "")?;
        let mid = b.build_int_add(
            b.build_right_shift(ll, k32, false, "")?,
            b.build_int_add(b.build_and(lh, mask, "")?, b.build_and(hl, mask, "")?, "")?,
            "",
        )?;
        let carries = b.build_int_add(
            b.build_int_add(
                b.build_right_shift(lh, k32, false, "")?,
                b.build_right_shift(hl, k32, false, "")?,
                "",
            )?,
            b.build_right_shift(mid, k32, false, "")?,
            "",
        )?;
        let high = b.build_int_add(hh, carries, "")?;
        let cross = b.build_int_add(
            b.build_int_mul(a_lo, b_hi, "")?,
            b.build_int_mul(a_hi, b_lo, "")?,
            "",
        )?;
        let high = b.build_int_add(high, cross, "")?;
        let low = b.build_int_mul(a_lo, b_lo, "")?;
        let out = f
            .get_nth_param(0)
            .ok_or("__multi3 result")?
            .into_pointer_value();
        b.build_store(out, low)?;
        // Inkwell marks GEP construction unsafe; an i8 GEP is plain pointer arithmetic.
        #[allow(unsafe_code)]
        let out_high =
            // SAFETY: inkwell marks GEP construction unsafe; an i8 GEP is plain pointer arithmetic.
            unsafe { b.build_gep(self.ctx.i8_type(), out, &[i64t.const_int(8, false)], "")? };
        b.build_store(out_high, high)?;
        b.build_return(None)?;
        Ok(())
    }

    /// Debug info for program globals (in the module that defines them).
    fn describe_globals(&self) {
        let Some(debug) = &self.debug else {
            return;
        };
        for g in &self.program.debug_globals {
            if !self.owns_global(g.global.0 as usize) {
                continue;
            }
            let gv = self.global(g.global.0 as usize);
            let local = self.program.globals[g.global.0 as usize].export.is_none();
            debug.global(gv, &gv.get_name().to_string_lossy(), local, g);
        }
    }

    // ----- types ---------------------------------------------------------

    // ----- module-level declarations ---------------------------------------

    // ----- function bodies -------------------------------------------------

    // ----- intrinsics ------------------------------------------------------
}
