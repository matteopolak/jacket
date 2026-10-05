//! IR to LLVM IR translation.
//!
//! Every IR `Val` becomes an LLVM SSA value (IR values are defined once and
//! blocks are visited in reverse post-order, so definitions dominate uses),
//! every `Slot` an entry-block alloca, and every aggregate stays in memory.
//! `Conv::C` signatures are translated to the real C ABI by [`Backend::lower_sig`].
#![allow(clippy::too_many_arguments)]
use crate::debuginfo::{DebugFormat, DebugInfo, FnDebug};
use inkwell::AddressSpace;
use inkwell::AtomicOrdering;
use inkwell::DLLStorageClass;
use inkwell::FloatPredicate;
use inkwell::GlobalVisibility;
use inkwell::IntPredicate;
use inkwell::attributes::{Attribute, AttributeLoc};
use inkwell::basic_block::BasicBlock;
use inkwell::builder::{Builder, BuilderError};
use inkwell::context::Context;
use inkwell::intrinsics::Intrinsic as LlvmIntrinsic;
use inkwell::module::{Linkage, Module};
use inkwell::types::{
    AnyType, BasicMetadataTypeEnum, BasicType, BasicTypeEnum, FunctionType, PointerType,
};
use inkwell::values::{
    BasicMetadataValueEnum, BasicValue, BasicValueEnum, CallSiteValue, FunctionValue, GlobalValue,
    IntValue, PointerValue, ValueKind,
};
use jaic::abi::{self, Arch, Passing, Piece, PieceTy};
use jaic::ir::{
    AggLayout, BinOp, BlockId, Callee, CmpOp, Conv, ConvOp, Foreign, Func, Global, Inst, Intrinsic,
    Linkage as IrLinkage, Program, RelocTarget, Sig, Term, Ty, UnOp, Val,
};

/// Backend error; converted to a `String` at the crate boundary.
pub struct Error(String);

impl From<BuilderError> for Error {
    fn from(e: BuilderError) -> Self {
        Error(format!("LLVM builder error: {e}"))
    }
}
impl From<String> for Error {
    fn from(e: String) -> Self {
        Error(e)
    }
}
impl From<&str> for Error {
    fn from(e: &str) -> Self {
        Error(e.to_string())
    }
}
type R<T> = Result<T, Error>;

/// Which part of the program a module holds when codegen is split across modules.
#[derive(Clone, Copy)]
pub struct Shard<'p> {
    /// The module index of each function; only the shard's own functions get bodies.
    pub owner: &'p [u32],
    pub index: u32,
}

/// Lower `program` into `module`. With a shard, the module defines only that shard's
/// functions (and, in shard 0, the globals); everything else is declared, and internal
/// symbols become hidden externals so the modules link together.
pub fn lower_program<'ctx>(
    context: &'ctx Context,
    module: &Module<'ctx>,
    program: &Program,
    arch: Arch,
    shard: Option<Shard>,
    debug: Option<(DebugFormat, bool)>,
) -> Result<(), String> {
    let debug = debug
        .map(|(format, optimized)| DebugInfo::new(context, module, program, format, optimized));
    let mut backend = Backend {
        ctx: context,
        module,
        builder: context.create_builder(),
        program,
        arch,
        shard,
        funcs: Vec::new(),
        globals: Vec::new(),
        foreigns: Vec::new(),
        debug,
    };
    backend.run().map_err(|e| e.0)
}

/// How one IR parameter maps onto LLVM parameters.
#[derive(Clone, Debug)]
enum ParamPlan {
    Scalar(Ty),
    /// A by-value C aggregate; the IR passes a pointer to it.
    Agg(AggLayout, Passing),
    /// The IR out-pointer of a C aggregate return that LLVM returns in registers.
    Dropped,
}

#[derive(Clone, Debug)]
enum RetPlan {
    Scalars(Vec<Ty>),
    /// C aggregate returned in register chunks; stored through the IR out-pointer.
    Registers(AggLayout, Vec<Piece>),
    /// C aggregate returned through a hidden first `sret` pointer.
    Sret,
}

/// Attributes attached to an LLVM parameter.
#[derive(Clone, Copy, Debug)]
enum ParamAttr {
    Sret(u64),
    ByVal(u64, u64),
}

struct Lowered<'ctx> {
    fn_ty: FunctionType<'ctx>,
    params: Vec<ParamPlan>,
    ret: RetPlan,
    attrs: Vec<(u32, ParamAttr)>,
}

struct Backend<'ctx, 'p> {
    ctx: &'ctx Context,
    module: &'p Module<'ctx>,
    builder: Builder<'ctx>,
    program: &'p Program,
    arch: Arch,
    shard: Option<Shard<'p>>,
    funcs: Vec<Option<FunctionValue<'ctx>>>,
    globals: Vec<GlobalValue<'ctx>>,
    /// Address of each foreign function or variable.
    foreigns: Vec<PointerValue<'ctx>>,
    /// Debug info builder when the program is built with debug info (`debuginfo.rs`).
    debug: Option<DebugInfo<'ctx, 'p>>,
}

/// Per-function lowering state.
struct FnState<'ctx> {
    function: FunctionValue<'ctx>,
    vals: Vec<Option<BasicValueEnum<'ctx>>>,
    slots: Vec<PointerValue<'ctx>>,
    blocks: Vec<Option<BasicBlock<'ctx>>>,
    /// Builder positioned in the dedicated alloca block.
    allocas: Builder<'ctx>,
    /// A C aggregate returned in registers: its pieces and the memory the IR writes it to.
    reg_ret: Option<(Vec<Piece>, PointerValue<'ctx>)>,
    /// Debug locations and variables of this function, with debug info on.
    dbg: Option<FnDebug<'ctx>>,
}

impl<'ctx, 'p> Backend<'ctx, 'p> {
    fn run(&mut self) -> R<()> {
        self.declare_functions()?;
        self.declare_foreigns()?;
        self.declare_globals()?;
        if self.shard.is_none_or(|s| s.index == 0) {
            self.init_globals()?;
            self.describe_globals();
        }
        for (i, func) in self.program.funcs.iter().enumerate() {
            if let Some(func) = func
                && self.owns_func(i)
            {
                let f = self.funcs[i].expect("declared");
                self.define_function(func, f)
                    .map_err(|e| Error(format!("in '{}': {}", func.name, e.0)))?;
            }
        }
        if let Some(debug) = &self.debug {
            debug.finalize();
        }
        Ok(())
    }

    /// Debug info for program globals (in the module that defines them).
    fn describe_globals(&self) {
        let Some(debug) = &self.debug else {
            return;
        };
        for g in &self.program.debug_globals {
            let gv = self.globals[g.global.0 as usize];
            let local = self.program.globals[g.global.0 as usize].export.is_none();
            debug.global(gv, &gv.get_name().to_string_lossy(), local, g);
        }
    }

    // ----- types ---------------------------------------------------------

    fn ptr_ty(&self) -> PointerType<'ctx> {
        self.ctx.ptr_type(AddressSpace::default())
    }

    fn ll(&self, ty: Ty) -> BasicTypeEnum<'ctx> {
        match ty {
            Ty::I8 => self.ctx.i8_type().into(),
            Ty::I16 => self.ctx.i16_type().into(),
            Ty::I32 => self.ctx.i32_type().into(),
            Ty::I64 => self.ctx.i64_type().into(),
            Ty::F32 => self.ctx.f32_type().into(),
            Ty::F64 => self.ctx.f64_type().into(),
            Ty::Ptr => self.ptr_ty().into(),
        }
    }

    fn piece_ty(&self, ty: PieceTy) -> BasicTypeEnum<'ctx> {
        match ty {
            PieceTy::I64 => self.ctx.i64_type().into(),
            PieceTy::F32 => self.ctx.f32_type().into(),
            PieceTy::F64 => self.ctx.f64_type().into(),
            PieceTy::V2F32 => self.ctx.f32_type().vec_type(2).into(),
        }
    }

    fn bytes_ty(&self, size: u64) -> BasicTypeEnum<'ctx> {
        self.ctx.i8_type().array_type(size as u32).into()
    }

    /// Function type and parameter mapping for an IR signature.
    fn lower_sig(&self, sig: &Sig) -> Lowered<'ctx> {
        let cabi = if sig.conv == Conv::C {
            sig.c_abi.as_deref()
        } else {
            None
        };
        let ret_agg = cabi.and_then(|c| c.ret.clone());
        let mut llvm_params: Vec<BasicMetadataTypeEnum<'ctx>> = Vec::new();
        let mut attrs = Vec::new();
        let ret = match &ret_agg {
            Some(layout) => match abi::classify_ret(self.arch, layout) {
                Some(pieces) => RetPlan::Registers(layout.clone(), pieces),
                None => {
                    attrs.push((0, ParamAttr::Sret(layout.size)));
                    llvm_params.push(self.ptr_ty().into());
                    RetPlan::Sret
                }
            },
            None => RetPlan::Scalars(sig.returns.clone()),
        };
        let mut params = Vec::new();
        // Variadic arguments are not part of the LLVM function type.
        let mut fixed = None;
        for (i, &ty) in sig.params.iter().enumerate() {
            if sig.c_varargs && i == sig.c_fixed as usize {
                fixed = Some(llvm_params.len());
            }
            if ret_agg.is_some() && i + 1 == sig.params.len() {
                params.push(ParamPlan::Dropped);
                continue;
            }
            let layout = cabi.and_then(|c| c.params.get(i).cloned().flatten());
            match layout {
                Some(layout) => {
                    let passing = abi::classify_arg(self.arch, &layout);
                    match &passing {
                        Passing::Registers(pieces) => {
                            for p in pieces {
                                llvm_params.push(self.piece_ty(p.ty).into());
                            }
                        }
                        Passing::ByVal => {
                            attrs.push((
                                llvm_params.len() as u32,
                                ParamAttr::ByVal(layout.size, layout.align),
                            ));
                            llvm_params.push(self.ptr_ty().into());
                        }
                        Passing::Indirect => llvm_params.push(self.ptr_ty().into()),
                    }
                    params.push(ParamPlan::Agg(layout, passing));
                }
                None => {
                    llvm_params.push(self.ll(ty).into());
                    params.push(ParamPlan::Scalar(ty));
                }
            }
        }
        let fixed_params = &llvm_params[..fixed.unwrap_or(llvm_params.len())];
        let fn_ty = match &ret {
            RetPlan::Scalars(tys) => self.fn_type(self.ret_type(tys), fixed_params, sig.c_varargs),
            RetPlan::Registers(_, pieces) => {
                let tys: Vec<BasicTypeEnum> = pieces.iter().map(|p| self.piece_ty(p.ty)).collect();
                self.fn_type(self.ret_type_of(&tys), fixed_params, sig.c_varargs)
            }
            RetPlan::Sret => self.fn_type(None, fixed_params, sig.c_varargs),
        };
        Lowered {
            fn_ty,
            params,
            ret,
            attrs,
        }
    }

    fn fn_type(
        &self,
        ret: Option<BasicTypeEnum<'ctx>>,
        params: &[BasicMetadataTypeEnum<'ctx>],
        varargs: bool,
    ) -> FunctionType<'ctx> {
        match ret {
            Some(t) => t.fn_type(params, varargs),
            None => self.ctx.void_type().fn_type(params, varargs),
        }
    }

    fn ret_type(&self, tys: &[Ty]) -> Option<BasicTypeEnum<'ctx>> {
        let tys: Vec<BasicTypeEnum> = tys.iter().map(|&t| self.ll(t)).collect();
        self.ret_type_of(&tys)
    }

    fn ret_type_of(&self, tys: &[BasicTypeEnum<'ctx>]) -> Option<BasicTypeEnum<'ctx>> {
        match tys {
            [] => None,
            [one] => Some(*one),
            many => Some(self.ctx.struct_type(many, false).into()),
        }
    }

    fn apply_attrs(&self, lowered: &Lowered<'ctx>, mut add: impl FnMut(AttributeLoc, Attribute)) {
        for &(index, attr) in &lowered.attrs {
            let (name, size, align) = match attr {
                ParamAttr::Sret(size) => ("sret", size, 0),
                ParamAttr::ByVal(size, align) => ("byval", size, align),
            };
            let kind = Attribute::get_named_enum_kind_id(name);
            let ty = self.bytes_ty(size).as_any_type_enum();
            add(
                AttributeLoc::Param(index),
                self.ctx.create_type_attribute(kind, ty),
            );
            if align > 0 {
                let kind = Attribute::get_named_enum_kind_id("align");
                add(
                    AttributeLoc::Param(index),
                    self.ctx.create_enum_attribute(kind, align),
                );
            }
        }
    }

    // ----- module-level declarations ---------------------------------------

    fn owns_func(&self, i: usize) -> bool {
        self.shard.is_none_or(|s| s.owner[i] == s.index)
    }

    /// Linkage of an internal symbol: hidden and external when other modules refer to it.
    fn internal_linkage(&self, gv: GlobalValue<'ctx>) {
        if self.shard.is_some() {
            gv.set_linkage(Linkage::External);
            gv.set_visibility(GlobalVisibility::Hidden);
        } else {
            gv.set_linkage(Linkage::Internal);
        }
    }

    fn declare_functions(&mut self) -> R<()> {
        for (i, func) in self.program.funcs.iter().enumerate() {
            let Some(func) = func else {
                self.funcs.push(None);
                continue;
            };
            let lowered = self.lower_sig(&func.sig);
            let name = match &func.linkage {
                IrLinkage::Export(name) => name.clone(),
                IrLinkage::Internal => format!("{}.{i}", func.name),
            };
            let f = self
                .module
                .add_function(&name, lowered.fn_ty, Some(Linkage::External));
            if func.linkage == IrLinkage::Internal {
                self.internal_linkage(f.as_global_value());
            } else if self.arch == Arch::Win64 && self.owns_func(i) {
                // `#program_export`: in a DLL's export table (harmless in an executable).
                f.as_global_value()
                    .set_dll_storage_class(DLLStorageClass::Export);
            }
            self.apply_attrs(&lowered, |loc, attr| f.add_attribute(loc, attr));
            self.funcs.push(Some(f));
        }
        Ok(())
    }

    fn declare_foreigns(&mut self) -> R<()> {
        let mut by_symbol: Vec<(&str, PointerValue<'ctx>)> = Vec::new();
        for foreign in &self.program.foreigns {
            let Foreign {
                symbol,
                sig,
                is_data,
                ..
            } = foreign;
            if let Some(&(_, p)) = by_symbol.iter().find(|(s, _)| s == symbol) {
                self.foreigns.push(p);
                continue;
            }
            let ptr = if *is_data {
                let g = self.module.add_global(self.ctx.i8_type(), None, symbol);
                g.set_linkage(Linkage::External);
                g.as_pointer_value()
            } else {
                let lowered = self.lower_sig(sig);
                self.module
                    .add_function(symbol, lowered.fn_ty, Some(Linkage::External))
                    .as_global_value()
                    .as_pointer_value()
            };
            by_symbol.push((symbol, ptr));
            self.foreigns.push(ptr);
        }
        Ok(())
    }

    /// Initial-value layout of a global: runs of plain bytes interleaved with
    /// pointer relocations.
    fn segments(g: &Global) -> Vec<Segment> {
        let size = g.size.max(g.init.len() as u64);
        let mut relocs: Vec<usize> = (0..g.relocs.len()).collect();
        relocs.sort_by_key(|&i| g.relocs[i].offset);
        let mut segs = Vec::new();
        let mut pos = 0u64;
        for i in relocs {
            let off = g.relocs[i].offset;
            if off < pos || off + 8 > size {
                continue;
            }
            if off > pos {
                segs.push(Segment::Bytes(pos, off));
            }
            segs.push(Segment::Reloc(i));
            pos = off + 8;
        }
        if size > pos || segs.is_empty() {
            segs.push(Segment::Bytes(pos, size.max(1)));
        }
        segs
    }

    fn declare_globals(&mut self) -> R<()> {
        for (i, g) in self.program.globals.iter().enumerate() {
            let fields: Vec<BasicTypeEnum> = Self::segments(g)
                .iter()
                .map(|s| match s {
                    Segment::Bytes(a, b) => self.bytes_ty(b - a),
                    Segment::Reloc(_) => self.ptr_ty().into(),
                })
                .collect();
            let ty = self.ctx.struct_type(&fields, true);
            let name = g
                .export
                .clone()
                .unwrap_or_else(|| format!("{}.{i}", g.name));
            let gv = self.module.add_global(ty, None, &name);
            gv.set_linkage(Linkage::External);
            if g.export.is_none() {
                self.internal_linkage(gv);
            }
            gv.set_alignment(g.align.max(1) as u32);
            gv.set_constant(g.read_only);
            self.globals.push(gv);
        }
        Ok(())
    }

    fn init_globals(&mut self) -> R<()> {
        for (i, g) in self.program.globals.iter().enumerate() {
            let mut fields: Vec<BasicValueEnum> = Vec::new();
            for seg in Self::segments(g) {
                match seg {
                    Segment::Bytes(a, b) => {
                        let bytes: Vec<IntValue> = (a..b)
                            .map(|k| {
                                let byte = g.init.get(k as usize).copied().unwrap_or(0);
                                self.ctx.i8_type().const_int(byte as u64, false)
                            })
                            .collect();
                        fields.push(self.ctx.i8_type().const_array(&bytes).into());
                    }
                    Segment::Reloc(r) => {
                        let reloc = &g.relocs[r];
                        let base = match reloc.target {
                            RelocTarget::Global(id) => {
                                self.globals[id.0 as usize].as_pointer_value()
                            }
                            RelocTarget::Func(id) => self.func_ptr(id)?,
                            RelocTarget::Foreign(id) => self.foreigns[id.0 as usize],
                        };
                        let ptr = if reloc.addend == 0 {
                            base
                        } else {
                            let i64t = self.ctx.i64_type();
                            let sum = base
                                .const_to_int(i64t)
                                .const_add(i64t.const_int(reloc.addend as u64, true));
                            sum.const_to_pointer(self.ptr_ty())
                        };
                        fields.push(ptr.into());
                    }
                }
            }
            let init = self.ctx.const_struct(&fields, true);
            self.globals[i].set_initializer(&init);
        }
        Ok(())
    }

    fn func_ptr(&self, id: jaic::ir::FuncId) -> R<PointerValue<'ctx>> {
        match self.funcs.get(id.0 as usize).copied().flatten() {
            Some(f) => Ok(f.as_global_value().as_pointer_value()),
            None => Err(Error(format!(
                "function '{}' is referenced but was never lowered",
                self.program
                    .func_names
                    .get(id.0 as usize)
                    .map_or("?", |s| s)
            ))),
        }
    }

    // ----- function bodies -------------------------------------------------

    fn define_function(&self, func: &Func, function: FunctionValue<'ctx>) -> R<()> {
        let alloca_block = self.ctx.append_basic_block(function, "allocas");
        let allocas = self.ctx.create_builder();
        allocas.position_at_end(alloca_block);

        // Reachable blocks in reverse post-order, so definitions precede uses.
        let order = reverse_post_order(func);
        let mut blocks = vec![None; func.blocks.len()];
        for &b in &order {
            blocks[b] = Some(self.ctx.append_basic_block(function, &format!("b{b}")));
        }
        let mut st = FnState {
            function,
            vals: vec![None; func.vals.len()],
            slots: Vec::new(),
            blocks,
            allocas,
            reg_ret: None,
            dbg: None,
        };
        if let Some(debug) = &self.debug {
            let local = func.linkage == IrLinkage::Internal;
            st.dbg = Some(debug.begin_function(func, function, local, &self.builder, &st.allocas));
        }
        self.bind_params(&mut st, func, alloca_block)?;
        for slot in &func.slots {
            let p = self.entry_alloca(&st, slot.size, slot.align)?;
            st.slots.push(p);
        }
        if let (Some(debug), Some(dbg)) = (&self.debug, &mut st.dbg) {
            dbg.declare_vars(debug, func, &st.slots, &st.vals, &st.allocas, alloca_block);
        }
        for &b in &order {
            let block = &func.blocks[b];
            self.builder
                .position_at_end(st.blocks[b].expect("reachable block"));
            if let (Some(debug), Some(dbg)) = (&self.debug, &mut st.dbg) {
                dbg.enter_block(debug, func, &self.builder, b);
            }
            for inst in &block.insts {
                self.inst(&mut st, func, inst)?;
            }
            self.term(&st, func, &block.term)?;
            if let Some(dbg) = &mut st.dbg {
                dbg.leave_block(&block.term);
            }
        }
        let entry = st.blocks[0].expect("entry block is reachable");
        if let Some(dbg) = &st.dbg {
            dbg.finish_entry(&st.allocas);
            self.builder.unset_current_debug_location();
        }
        st.allocas.build_unconditional_branch(entry)?;
        Ok(())
    }

    /// Bind the IR parameters to the LLVM ones. With the C ABI, by-value aggregates arrive
    /// in registers (stored to memory here), by `byval` pointer or as a caller copy; the IR
    /// sees a pointer to each. An aggregate result is written through the IR's last
    /// parameter: the `sret` pointer, or memory `Term::Ret` returns in registers.
    fn bind_params(&self, st: &mut FnState<'ctx>, func: &Func, entry: BasicBlock<'ctx>) -> R<()> {
        let lowered = self.lower_sig(&func.sig);
        let params: Vec<BasicValueEnum<'ctx>> = st.function.get_param_iter().collect();
        let mut k = 0;
        if matches!(lowered.ret, RetPlan::Sret) {
            k = 1;
        }
        self.builder.position_at_end(entry);
        for (i, plan) in lowered.params.iter().enumerate() {
            let value: BasicValueEnum<'ctx> = match plan {
                ParamPlan::Scalar(_) => {
                    k += 1;
                    params[k - 1]
                }
                ParamPlan::Agg(layout, passing) => match passing {
                    Passing::ByVal | Passing::Indirect => {
                        k += 1;
                        params[k - 1]
                    }
                    Passing::Registers(pieces) => {
                        let tmp = self.call_temp(st, layout.size, layout.align)?;
                        for p in pieces {
                            self.builder
                                .build_store(self.gep_const(tmp, p.offset)?, params[k])?;
                            k += 1;
                        }
                        tmp.into()
                    }
                },
                ParamPlan::Dropped => match &lowered.ret {
                    RetPlan::Sret => params[0],
                    RetPlan::Registers(layout, pieces) => {
                        let tmp = self.call_temp(st, layout.size, layout.align)?;
                        st.reg_ret = Some((pieces.clone(), tmp));
                        tmp.into()
                    }
                    RetPlan::Scalars(_) => {
                        return Err("aggregate out-pointer without a C return".into());
                    }
                },
            };
            st.vals[i] = Some(value);
        }
        Ok(())
    }

    fn entry_alloca(&self, st: &FnState<'ctx>, size: u64, align: u64) -> R<PointerValue<'ctx>> {
        let ptr = st
            .allocas
            .build_alloca(self.bytes_ty(size.max(1)), "slot")?;
        if let Some(inst) = ptr.as_instruction() {
            inst.set_alignment(align.clamp(1, 1 << 16).next_power_of_two() as u32)
                .map_err(|e| Error(e.to_string()))?;
        }
        Ok(ptr)
    }

    fn get(&self, st: &FnState<'ctx>, v: Val) -> R<BasicValueEnum<'ctx>> {
        st.vals[v.0 as usize].ok_or_else(|| Error(format!("use of undefined value %{}", v.0)))
    }

    fn set(&self, st: &mut FnState<'ctx>, v: Val, value: BasicValueEnum<'ctx>) {
        st.vals[v.0 as usize] = Some(value);
        // A variable whose address is computed here: copy it where its debug info looks.
        if let Some(slot) = st.dbg.as_ref().and_then(|d| d.watched(v)) {
            let _ = self.builder.build_store(slot, value);
        }
    }

    fn get_int(&self, st: &FnState<'ctx>, v: Val) -> R<IntValue<'ctx>> {
        self.as_int(self.get(st, v)?)
    }

    fn get_ptr(&self, st: &FnState<'ctx>, v: Val) -> R<PointerValue<'ctx>> {
        let value = self.get(st, v)?;
        self.coerce(value, Ty::Ptr).map(|p| p.into_pointer_value())
    }

    /// View any scalar as an integer (pointers via `ptrtoint`).
    fn as_int(&self, v: BasicValueEnum<'ctx>) -> R<IntValue<'ctx>> {
        match v {
            BasicValueEnum::IntValue(i) => Ok(i),
            BasicValueEnum::PointerValue(p) => {
                Ok(self.builder.build_ptr_to_int(p, self.ctx.i64_type(), "")?)
            }
            other => Err(Error(format!("expected an integer, found {other:?}"))),
        }
    }

    /// Adjust `v` to the LLVM type of `ty` where the IR is loose about
    /// pointer/integer identity.
    fn coerce(&self, v: BasicValueEnum<'ctx>, ty: Ty) -> R<BasicValueEnum<'ctx>> {
        let want = self.ll(ty);
        if v.get_type() == want {
            return Ok(v);
        }
        Ok(match (v, want) {
            (BasicValueEnum::PointerValue(p), BasicTypeEnum::IntType(t)) => {
                self.builder.build_ptr_to_int(p, t, "")?.into()
            }
            (BasicValueEnum::IntValue(i), BasicTypeEnum::PointerType(t)) => {
                let i = self
                    .builder
                    .build_int_z_extend_or_bit_cast(i, self.ctx.i64_type(), "")?;
                self.builder.build_int_to_ptr(i, t, "")?.into()
            }
            (BasicValueEnum::IntValue(i), BasicTypeEnum::IntType(t)) => self
                .builder
                .build_int_z_extend_or_bit_cast(i, t, "")?
                .into(),
            (v, want) => self
                .builder
                .build_bit_cast(v, want, "")
                .map_err(|_| Error(format!("cannot convert {v:?} to {want:?}")))?,
        })
    }

    fn gep(&self, base: PointerValue<'ctx>, offset: IntValue<'ctx>) -> R<PointerValue<'ctx>> {
        // Inkwell marks GEP construction unsafe; an i8 GEP is plain pointer arithmetic.
        #[allow(unsafe_code)]
        let p = unsafe {
            self.builder
                .build_gep(self.ctx.i8_type(), base, &[offset], "")?
        };
        Ok(p)
    }

    fn gep_const(&self, base: PointerValue<'ctx>, offset: u64) -> R<PointerValue<'ctx>> {
        if offset == 0 {
            return Ok(base);
        }
        self.gep(base, self.ctx.i64_type().const_int(offset, false))
    }

    fn i64_of(&self, v: IntValue<'ctx>) -> R<IntValue<'ctx>> {
        Ok(self
            .builder
            .build_int_z_extend_or_bit_cast(v, self.ctx.i64_type(), "")?)
    }

    fn intrinsic_fn(&self, name: &str, tys: &[BasicTypeEnum<'ctx>]) -> R<FunctionValue<'ctx>> {
        LlvmIntrinsic::find(name)
            .and_then(|i| i.get_declaration(self.module, tys))
            .ok_or_else(|| Error(format!("LLVM intrinsic '{name}' is unavailable")))
    }

    fn call_intrinsic(
        &self,
        name: &str,
        tys: &[BasicTypeEnum<'ctx>],
        args: &[BasicMetadataValueEnum<'ctx>],
    ) -> R<Option<BasicValueEnum<'ctx>>> {
        let f = self.intrinsic_fn(name, tys)?;
        Ok(basic(self.builder.build_call(f, args, "")?))
    }

    /// Declare (once) and return a libc function.
    fn libc(&self, name: &str, ty: FunctionType<'ctx>) -> FunctionValue<'ctx> {
        self.module
            .get_function(name)
            .unwrap_or_else(|| self.module.add_function(name, ty, Some(Linkage::External)))
    }

    /// Branch to a trapping block when `cond` (an `i1`) holds.
    fn trap_if(&self, st: &FnState<'ctx>, cond: IntValue<'ctx>) -> R<()> {
        let trap = self.ctx.append_basic_block(st.function, "trap");
        let cont = self.ctx.append_basic_block(st.function, "cont");
        self.builder.build_conditional_branch(cond, trap, cont)?;
        self.builder.position_at_end(trap);
        self.call_intrinsic("llvm.trap", &[], &[])?;
        self.builder.build_unreachable()?;
        self.builder.position_at_end(cont);
        Ok(())
    }

    fn inst(&self, st: &mut FnState<'ctx>, func: &Func, inst: &Inst) -> R<()> {
        let b = &self.builder;
        match inst {
            Inst::IConst {
                dst,
                ty,
                value,
            } => {
                let v: BasicValueEnum = match ty {
                    Ty::Ptr => self
                        .ctx
                        .i64_type()
                        .const_int(*value, false)
                        .const_to_pointer(self.ptr_ty())
                        .into(),
                    Ty::F32 | Ty::F64 => return Err("integer constant of float type".into()),
                    _ => self.ll(*ty).into_int_type().const_int(*value, false).into(),
                };
                self.set(st, *dst, v);
            }
            Inst::FConst {
                dst,
                ty,
                value,
            } => {
                let v = match ty {
                    Ty::F32 => self.ctx.f32_type().const_float(*value),
                    _ => self.ctx.f64_type().const_float(*value),
                };
                self.set(st, *dst, v.into());
            }
            Inst::Bin {
                dst,
                op,
                ty,
                a,
                b: rhs,
            } => {
                let v = self.bin(st, *op, *ty, *a, *rhs)?;
                self.set(st, *dst, v);
            }
            Inst::Un {
                dst,
                op,
                ty,
                a,
            } => {
                let v: BasicValueEnum = match op {
                    UnOp::Neg => b.build_int_neg(self.get_int(st, *a)?, "")?.into(),
                    UnOp::Not => b.build_not(self.get_int(st, *a)?, "")?.into(),
                    UnOp::FNeg => b
                        .build_float_neg(
                            self.coerce(self.get(st, *a)?, *ty)?.into_float_value(),
                            "",
                        )?
                        .into(),
                };
                let v = if *ty == Ty::Ptr {
                    self.coerce(v, Ty::Ptr)?
                } else {
                    v
                };
                self.set(st, *dst, v);
            }
            Inst::Cmp {
                dst,
                op,
                ty,
                a,
                b: rhs,
            } => {
                let (x, y) = (self.get(st, *a)?, self.get(st, *rhs)?);
                let r = if ty.is_float() {
                    let (x, y) = (
                        self.coerce(x, *ty)?.into_float_value(),
                        self.coerce(y, *ty)?.into_float_value(),
                    );
                    let pred = match op {
                        CmpOp::FEq => FloatPredicate::OEQ,
                        CmpOp::FNe => FloatPredicate::UNE,
                        CmpOp::FLt => FloatPredicate::OLT,
                        CmpOp::FLe => FloatPredicate::OLE,
                        CmpOp::FGt => FloatPredicate::OGT,
                        CmpOp::FGe => FloatPredicate::OGE,
                        _ => return Err("integer comparison on floats".into()),
                    };
                    b.build_float_compare(pred, x, y, "")?
                } else {
                    let pred = match op {
                        CmpOp::Eq => IntPredicate::EQ,
                        CmpOp::Ne => IntPredicate::NE,
                        CmpOp::SLt => IntPredicate::SLT,
                        CmpOp::SLe => IntPredicate::SLE,
                        CmpOp::SGt => IntPredicate::SGT,
                        CmpOp::SGe => IntPredicate::SGE,
                        CmpOp::ULt => IntPredicate::ULT,
                        CmpOp::ULe => IntPredicate::ULE,
                        CmpOp::UGt => IntPredicate::UGT,
                        CmpOp::UGe => IntPredicate::UGE,
                        _ => return Err("float comparison on integers".into()),
                    };
                    b.build_int_compare(pred, self.as_int(x)?, self.as_int(y)?, "")?
                };
                let r = b.build_int_z_extend(r, self.ctx.i8_type(), "")?;
                self.set(st, *dst, r.into());
            }
            Inst::Conv {
                dst,
                op,
                from,
                to,
                src,
            } => {
                let v = self.conv(st, *op, *from, *to, *src)?;
                self.set(st, *dst, v);
            }
            Inst::SlotAddr {
                dst,
                slot,
            } => {
                let p = st.slots[slot.0 as usize];
                self.set(st, *dst, p.into());
            }
            Inst::GlobalAddr {
                dst,
                global,
            } => {
                let p = self.globals[global.0 as usize].as_pointer_value();
                self.set(st, *dst, p.into());
            }
            Inst::FuncAddr {
                dst,
                func,
            } => {
                let p = self.func_ptr(*func)?;
                self.set(st, *dst, p.into());
            }
            Inst::ForeignAddr {
                dst,
                foreign,
            } => {
                let p = self.foreigns[foreign.0 as usize];
                self.set(st, *dst, p.into());
            }
            Inst::Load {
                dst,
                ty,
                addr,
            } => {
                let p = self.get_ptr(st, *addr)?;
                let load = b.build_load(self.ll(*ty), p, "")?;
                if let Some(i) = load.as_instruction_value() {
                    let _ = i.set_alignment(ty.size() as u32);
                }
                self.set(st, *dst, load);
            }
            Inst::Store {
                ty,
                addr,
                value,
            } => {
                let p = self.get_ptr(st, *addr)?;
                let v = self.coerce(self.get(st, *value)?, *ty)?;
                let store = b.build_store(p, v)?;
                let _ = store.set_alignment(ty.size() as u32);
            }
            Inst::PtrAdd {
                dst,
                base,
                offset,
            } => {
                let base = self.get_ptr(st, *base)?;
                let off = self.get_int(st, *offset)?;
                let off = if off.get_type().get_bit_width() < 64 {
                    b.build_int_s_extend(off, self.ctx.i64_type(), "")?
                } else {
                    off
                };
                let p = self.gep(base, off)?;
                self.set(st, *dst, p.into());
            }
            Inst::Copy {
                dst,
                src,
                size,
            } => {
                let (d, s) = (self.get_ptr(st, *dst)?, self.get_ptr(st, *src)?);
                b.build_memcpy(d, 1, s, 1, self.ctx.i64_type().const_int(*size, false))?;
            }
            Inst::Zero {
                dst,
                size,
            } => {
                let d = self.get_ptr(st, *dst)?;
                b.build_memset(
                    d,
                    1,
                    self.ctx.i8_type().const_zero(),
                    self.ctx.i64_type().const_int(*size, false),
                )?;
            }
            Inst::Call {
                results,
                callee,
                args,
            } => {
                let argv = args
                    .iter()
                    .map(|&a| self.get(st, a))
                    .collect::<R<Vec<_>>>()?;
                let (ptr, sig) = match callee {
                    Callee::Func(id) => {
                        let f = self.program.func(*id).ok_or_else(|| {
                            Error(format!(
                                "call to '{}' which was never lowered",
                                self.program.func_names[id.0 as usize]
                            ))
                        })?;
                        (self.func_ptr(*id)?, &f.sig)
                    }
                    Callee::Foreign(id) => (
                        self.foreigns[id.0 as usize],
                        &self.program.foreigns[id.0 as usize].sig,
                    ),
                    Callee::Indirect(target, sig) => (self.get_ptr(st, *target)?, sig),
                };
                let out = self.call(st, ptr, sig, &argv)?;
                for (r, v) in results.iter().zip(out) {
                    self.set(st, *r, v);
                }
            }
            Inst::Intrinsic {
                results,
                op,
                args,
            } => {
                let argv = args
                    .iter()
                    .map(|&a| self.get(st, a))
                    .collect::<R<Vec<_>>>()?;
                let want: Vec<Ty> = results.iter().map(|r| func.vals[r.0 as usize]).collect();
                let out = self.intrinsic(st, *op, &argv, &want)?;
                for (r, v) in results.iter().zip(out) {
                    self.set(st, *r, v);
                }
            }
            Inst::Loc {
                line,
                col,
                scope,
                ..
            } => {
                if let (Some(debug), Some(dbg)) = (&self.debug, &mut st.dbg) {
                    dbg.loc(debug, func, &self.builder, *line, *col, *scope);
                }
            }
        }
        Ok(())
    }

    fn bin(
        &self,
        st: &FnState<'ctx>,
        op: BinOp,
        ty: Ty,
        a: Val,
        rhs: Val,
    ) -> R<BasicValueEnum<'ctx>> {
        let b = &self.builder;
        if ty.is_float() {
            let x = self.coerce(self.get(st, a)?, ty)?.into_float_value();
            let y = self.coerce(self.get(st, rhs)?, ty)?.into_float_value();
            return Ok(match op {
                BinOp::FAdd => b.build_float_add(x, y, "")?,
                BinOp::FSub => b.build_float_sub(x, y, "")?,
                BinOp::FMul => b.build_float_mul(x, y, "")?,
                BinOp::FDiv => b.build_float_div(x, y, "")?,
                _ => return Err("integer operation on floats".into()),
            }
            .into());
        }
        let x = self.get_int(st, a)?;
        let y = self.get_int(st, rhs)?;
        let it = x.get_type();
        let bits = it.get_bit_width();
        let konst = |v: u64| it.const_int(v, false);
        let r = match op {
            BinOp::Add => b.build_int_add(x, y, "")?,
            BinOp::Sub => b.build_int_sub(x, y, "")?,
            BinOp::Mul => b.build_int_mul(x, y, "")?,
            BinOp::And => b.build_and(x, y, "")?,
            BinOp::Or => b.build_or(x, y, "")?,
            BinOp::Xor => b.build_xor(x, y, "")?,
            BinOp::UDiv | BinOp::URem => {
                let zero = b.build_int_compare(IntPredicate::EQ, y, konst(0), "")?;
                self.trap_if(st, zero)?;
                if op == BinOp::UDiv {
                    b.build_int_unsigned_div(x, y, "")?
                } else {
                    b.build_int_unsigned_rem(x, y, "")?
                }
            }
            BinOp::SDiv | BinOp::SRem => {
                let zero = b.build_int_compare(IntPredicate::EQ, y, konst(0), "")?;
                self.trap_if(st, zero)?;
                // x / -1 overflows for INT_MIN (undefined in LLVM); the IR wraps.
                let minus_one =
                    b.build_int_compare(IntPredicate::EQ, y, it.const_all_ones(), "")?;
                let safe_y = b.build_select(minus_one, konst(1), y, "")?.into_int_value();
                if op == BinOp::SDiv {
                    let q = b.build_int_signed_div(x, safe_y, "")?;
                    let neg = b.build_int_neg(x, "")?;
                    b.build_select(minus_one, neg, q, "")?.into_int_value()
                } else {
                    let r = b.build_int_signed_rem(x, safe_y, "")?;
                    b.build_select(minus_one, konst(0), r, "")?.into_int_value()
                }
            }
            BinOp::Shl | BinOp::LShr => {
                let out = b.build_int_compare(IntPredicate::UGE, y, konst(bits as u64), "")?;
                let r = if op == BinOp::Shl {
                    b.build_left_shift(x, y, "")?
                } else {
                    b.build_right_shift(x, y, false, "")?
                };
                b.build_select(out, konst(0), r, "")?.into_int_value()
            }
            BinOp::AShr => {
                let out = b.build_int_compare(IntPredicate::UGE, y, konst(bits as u64), "")?;
                let y = b
                    .build_select(out, konst(bits as u64 - 1), y, "")?
                    .into_int_value();
                b.build_right_shift(x, y, true, "")?
            }
            BinOp::Rotl | BinOp::Rotr => {
                let name = if op == BinOp::Rotl {
                    "llvm.fshl"
                } else {
                    "llvm.fshr"
                };
                self.call_intrinsic(name, &[it.into()], &[x.into(), x.into(), y.into()])?
                    .ok_or("rotate produced no value")?
                    .into_int_value()
            }
            BinOp::FAdd | BinOp::FSub | BinOp::FMul | BinOp::FDiv => {
                return Err("float operation on integers".into());
            }
        };
        if ty == Ty::Ptr {
            self.coerce(r.into(), Ty::Ptr)
        } else {
            Ok(r.into())
        }
    }

    fn conv(
        &self,
        st: &FnState<'ctx>,
        op: ConvOp,
        from: Ty,
        to: Ty,
        src: Val,
    ) -> R<BasicValueEnum<'ctx>> {
        let b = &self.builder;
        let v = self.coerce(self.get(st, src)?, from)?;
        let to_ll = self.ll(to);
        Ok(match op {
            ConvOp::Trunc => {
                let i = self.as_int(v)?;
                b.build_int_truncate_or_bit_cast(i, to_ll.into_int_type(), "")?
                    .into()
            }
            ConvOp::ZExt => b
                .build_int_z_extend_or_bit_cast(self.as_int(v)?, to_ll.into_int_type(), "")?
                .into(),
            ConvOp::SExt => b
                .build_int_s_extend_or_bit_cast(self.as_int(v)?, to_ll.into_int_type(), "")?
                .into(),
            ConvOp::FToS | ConvOp::FToU => {
                let name = if op == ConvOp::FToS {
                    "llvm.fptosi.sat"
                } else {
                    "llvm.fptoui.sat"
                };
                // Saturating, matching the interpreter's Rust `as` casts.
                self.call_intrinsic(name, &[to_ll, v.get_type()], &[v.into()])?
                    .ok_or("float conversion produced no value")?
            }
            ConvOp::SToF => b
                .build_signed_int_to_float(self.as_int(v)?, to_ll.into_float_type(), "")?
                .into(),
            ConvOp::UToF => b
                .build_unsigned_int_to_float(self.as_int(v)?, to_ll.into_float_type(), "")?
                .into(),
            ConvOp::FExt => b
                .build_float_ext(v.into_float_value(), to_ll.into_float_type(), "")?
                .into(),
            ConvOp::FTrunc => b
                .build_float_trunc(v.into_float_value(), to_ll.into_float_type(), "")?
                .into(),
            ConvOp::Bitcast => self.coerce(v, to)?,
        })
    }

    fn term(&self, st: &FnState<'ctx>, func: &Func, term: &Term) -> R<()> {
        let b = &self.builder;
        let block = |id: &BlockId| {
            st.blocks[id.0 as usize].ok_or_else(|| Error("branch to unreachable block".into()))
        };
        match term {
            Term::Jump(t) => {
                b.build_unconditional_branch(block(t)?)?;
            }
            Term::Branch {
                cond,
                then_block,
                else_block,
            } => {
                let c = self.get_int(st, *cond)?;
                let c = b.build_int_compare(IntPredicate::NE, c, c.get_type().const_zero(), "")?;
                b.build_conditional_branch(c, block(then_block)?, block(else_block)?)?;
            }
            Term::Switch {
                value,
                ty,
                cases,
                default,
            } => {
                let v = self.coerce(self.get(st, *value)?, *ty)?;
                let v = self.as_int(v)?;
                let cases = cases
                    .iter()
                    .map(|(c, t)| Ok((v.get_type().const_int(*c, false), block(t)?)))
                    .collect::<R<Vec<_>>>()?;
                b.build_switch(v, block(default)?, &cases)?;
            }
            Term::Ret(_) if st.reg_ret.is_some() => {
                let (pieces, tmp) = st.reg_ret.as_ref().expect("checked");
                let parts = pieces
                    .iter()
                    .map(|p| {
                        Ok(b.build_load(self.piece_ty(p.ty), self.gep_const(*tmp, p.offset)?, "")?)
                    })
                    .collect::<R<Vec<_>>>()?;
                match parts.as_slice() {
                    [] => b.build_return(None)?,
                    [one] => b.build_return(Some(one))?,
                    many => b.build_aggregate_return(many)?,
                };
            }
            Term::Ret(values) => {
                let tys = &func.sig.returns;
                let vals = values
                    .iter()
                    .zip(tys)
                    .map(|(&v, &t)| self.coerce(self.get(st, v)?, t))
                    .collect::<R<Vec<_>>>()?;
                match vals.as_slice() {
                    [] => {
                        b.build_return(None)?;
                    }
                    [one] => {
                        b.build_return(Some(one))?;
                    }
                    many => {
                        b.build_aggregate_return(many)?;
                    }
                }
            }
            Term::Unreachable => {
                b.build_unreachable()?;
            }
        }
        Ok(())
    }

    // ----- calls -----------------------------------------------------------

    /// Emit a call through `ptr` with IR signature `sig`, applying the C ABI
    /// for by-value aggregates. Returns the IR result values.
    fn call(
        &self,
        st: &FnState<'ctx>,
        ptr: PointerValue<'ctx>,
        sig: &Sig,
        args: &[BasicValueEnum<'ctx>],
    ) -> R<Vec<BasicValueEnum<'ctx>>> {
        let b = &self.builder;
        let lowered = self.lower_sig(sig);
        let mut out_ptr = None;
        let mut ll_args: Vec<BasicMetadataValueEnum<'ctx>> = Vec::new();
        if matches!(lowered.ret, RetPlan::Registers(..) | RetPlan::Sret) {
            let last = args.get(sig.params.len().wrapping_sub(1)).copied();
            out_ptr = Some(
                self.coerce(last.ok_or("missing aggregate out-pointer")?, Ty::Ptr)?
                    .into_pointer_value(),
            );
        }
        if matches!(lowered.ret, RetPlan::Sret) {
            ll_args.push(out_ptr.expect("set above").into());
        }
        for (i, &arg) in args.iter().enumerate() {
            match lowered.params.get(i) {
                Some(ParamPlan::Dropped) => {}
                Some(ParamPlan::Scalar(ty)) => ll_args.push(self.coerce(arg, *ty)?.into()),
                Some(ParamPlan::Agg(layout, passing)) => {
                    let src = self.coerce(arg, Ty::Ptr)?.into_pointer_value();
                    match passing {
                        Passing::ByVal => ll_args.push(src.into()),
                        Passing::Indirect => {
                            let tmp = self.call_temp(st, layout.size, layout.align)?;
                            self.memcpy(tmp, src, layout.size)?;
                            ll_args.push(tmp.into());
                        }
                        Passing::Registers(pieces) => {
                            let tmp = self.call_temp(st, layout.size, layout.align)?;
                            self.memcpy(tmp, src, layout.size)?;
                            for p in pieces {
                                let at = self.gep_const(tmp, p.offset)?;
                                ll_args.push(b.build_load(self.piece_ty(p.ty), at, "")?.into());
                            }
                        }
                    }
                }
                // Variadic extras: pass as they are.
                None => ll_args.push(arg.into()),
            }
        }
        let call = b.build_indirect_call(lowered.fn_ty, ptr, &ll_args, "")?;
        self.apply_attrs(&lowered, |loc, attr| call.add_attribute(loc, attr));
        let result = basic(call);
        match &lowered.ret {
            RetPlan::Scalars(tys) => match tys.as_slice() {
                [] => Ok(Vec::new()),
                [_] => Ok(vec![result.ok_or("call returned no value")?]),
                many => {
                    let agg = result.ok_or("call returned no value")?.into_struct_value();
                    (0..many.len())
                        .map(|i| Ok(b.build_extract_value(agg, i as u32, "")?))
                        .collect()
                }
            },
            RetPlan::Registers(layout, pieces) => {
                let tmp = self.call_temp(st, layout.size, layout.align)?;
                let parts: Vec<BasicValueEnum> = match pieces.len() {
                    0 => Vec::new(),
                    1 => vec![result.ok_or("call returned no value")?],
                    n => {
                        let agg = result.ok_or("call returned no value")?.into_struct_value();
                        (0..n)
                            .map(|i| Ok(b.build_extract_value(agg, i as u32, "")?))
                            .collect::<R<_>>()?
                    }
                };
                for (p, v) in pieces.iter().zip(parts) {
                    b.build_store(self.gep_const(tmp, p.offset)?, v)?;
                }
                self.memcpy(out_ptr.expect("set above"), tmp, layout.size)?;
                Ok(Vec::new())
            }
            RetPlan::Sret => Ok(Vec::new()),
        }
    }

    /// Scratch memory for marshalling an aggregate; rounded up so register
    /// chunks may read past the last field.
    fn call_temp(&self, st: &FnState<'ctx>, size: u64, align: u64) -> R<PointerValue<'ctx>> {
        self.entry_alloca(st, size.next_multiple_of(16), align.max(8))
    }

    fn memcpy(&self, dst: PointerValue<'ctx>, src: PointerValue<'ctx>, size: u64) -> R<()> {
        if size > 0 {
            self.builder.build_memcpy(
                dst,
                1,
                src,
                1,
                self.ctx.i64_type().const_int(size, false),
            )?;
        }
        Ok(())
    }

    // ----- intrinsics ------------------------------------------------------

    fn intrinsic(
        &self,
        st: &FnState<'ctx>,
        op: Intrinsic,
        args: &[BasicValueEnum<'ctx>],
        want: &[Ty],
    ) -> R<Vec<BasicValueEnum<'ctx>>> {
        let b = &self.builder;
        let i64t = self.ctx.i64_type();
        let ptr_arg = |i: usize| {
            self.coerce(args[i], Ty::Ptr)
                .map(|v| v.into_pointer_value())
        };
        let size_arg = |i: usize| self.as_int(args[i]).and_then(|v| self.i64_of(v));
        let float_unary = |name: &str| -> R<Vec<BasicValueEnum<'ctx>>> {
            let ty = args[0].get_type();
            let v = self
                .call_intrinsic(name, &[ty], &[args[0].into()])?
                .ok_or("math intrinsic produced no value")?;
            Ok(vec![v])
        };
        match op {
            Intrinsic::Memcpy => {
                b.build_memmove(ptr_arg(0)?, 1, ptr_arg(1)?, 1, size_arg(2)?)?;
                Ok(vec![])
            }
            Intrinsic::Memset => {
                let byte = b.build_int_truncate_or_bit_cast(
                    self.as_int(args[1])?,
                    self.ctx.i8_type(),
                    "",
                )?;
                b.build_memset(ptr_arg(0)?, 1, byte, size_arg(2)?)?;
                Ok(vec![])
            }
            Intrinsic::Memcmp => {
                let ty = self.ctx.i32_type().fn_type(
                    &[self.ptr_ty().into(), self.ptr_ty().into(), i64t.into()],
                    false,
                );
                let f = self.libc("memcmp", ty);
                let r = basic(b.build_call(
                    f,
                    &[ptr_arg(0)?.into(), ptr_arg(1)?.into(), size_arg(2)?.into()],
                    "",
                )?)
                .ok_or("memcmp returned no value")?
                .into_int_value();
                // Normalize to -1/0/1 as I16, like the interpreter.
                let i16t = self.ctx.i16_type();
                let zero = self.ctx.i32_type().const_zero();
                let lt = b.build_int_compare(IntPredicate::SLT, r, zero, "")?;
                let gt = b.build_int_compare(IntPredicate::SGT, r, zero, "")?;
                let pos = b.build_int_z_extend(gt, i16t, "")?;
                let neg = b.build_int_s_extend(lt, i16t, "")?;
                Ok(vec![b.build_or(pos, neg, "")?.into()])
            }
            Intrinsic::CompareAndSwap => {
                // (ptr, old, new, width_bytes) -> (success, previous); width comes from the operand type.
                let p = ptr_arg(0)?;
                let old = self.as_int(args[1])?;
                let new = self.as_int(args[2])?;
                let new = b.build_int_truncate_or_bit_cast(new, old.get_type(), "")?;
                let res = b.build_cmpxchg(
                    p,
                    old,
                    new,
                    AtomicOrdering::SequentiallyConsistent,
                    AtomicOrdering::SequentiallyConsistent,
                )?;
                let prev = b.build_extract_value(res, 0, "")?;
                let ok = b.build_extract_value(res, 1, "")?.into_int_value();
                let ok = b.build_int_z_extend(ok, self.ctx.i8_type(), "")?;
                Ok(vec![ok.into(), prev])
            }
            Intrinsic::DebugBreak => {
                self.call_intrinsic("llvm.debugtrap", &[], &[])?;
                Ok(vec![])
            }
            Intrinsic::Trap => {
                self.call_intrinsic("llvm.trap", &[], &[])?;
                Ok(vec![])
            }
            Intrinsic::BoundsCheck => {
                // Unsigned compare: a negative index is out of range too.
                let index = self.i64_of(self.as_int(args[0])?)?;
                let count = self.i64_of(self.as_int(args[1])?)?;
                let out = b.build_int_compare(IntPredicate::UGE, index, count, "")?;
                self.trap_if(st, out)?;
                Ok(vec![])
            }
            Intrinsic::CompilerWrite => {
                // (ptr, count, to_stderr): write(2) on fd 1 or 2.
                let to_err = self.as_int(args[2])?;
                let to_err = b.build_int_compare(
                    IntPredicate::NE,
                    to_err,
                    to_err.get_type().const_zero(),
                    "",
                )?;
                let fd = b
                    .build_select(
                        to_err,
                        self.ctx.i32_type().const_int(2, false),
                        self.ctx.i32_type().const_int(1, false),
                        "",
                    )?
                    .into_int_value();
                let ty = i64t.fn_type(
                    &[
                        self.ctx.i32_type().into(),
                        self.ptr_ty().into(),
                        i64t.into(),
                    ],
                    false,
                );
                // The Windows C runtime spells POSIX `write` with an underscore.
                let name = if self.arch == Arch::Win64 {
                    "_write"
                } else {
                    "write"
                };
                let f = self.libc(name, ty);
                b.build_call(f, &[fd.into(), ptr_arg(0)?.into(), size_arg(1)?.into()], "")?;
                Ok(vec![])
            }
            Intrinsic::Sqrt => float_unary("llvm.sqrt"),
            Intrinsic::Sin => float_unary("llvm.sin"),
            Intrinsic::Cos => float_unary("llvm.cos"),
            Intrinsic::Floor => float_unary("llvm.floor"),
            Intrinsic::Ceil => float_unary("llvm.ceil"),
            Intrinsic::Round => float_unary("llvm.round"),
            Intrinsic::Trunc => float_unary("llvm.trunc"),
            Intrinsic::Fabs => float_unary("llvm.fabs"),
            Intrinsic::Fma => {
                let ty = args[0].get_type();
                let v = self
                    .call_intrinsic(
                        "llvm.fma",
                        &[ty],
                        &[args[0].into(), args[1].into(), args[2].into()],
                    )?
                    .ok_or("fma produced no value")?;
                Ok(vec![v])
            }
            Intrinsic::ReturnAddress => {
                let zero = self.ctx.i32_type().const_zero();
                let v = self
                    .call_intrinsic("llvm.returnaddress", &[], &[zero.into()])?
                    .ok_or("returnaddress produced no value")?;
                Ok(vec![v])
            }
            Intrinsic::CycleCounter => {
                let v = match self.arch {
                    // The user-space virtual counter; PMCCNTR is not readable from EL0.
                    Arch::Aarch64 => {
                        self.inline_asm("mrs $0, cntvct_el0", "=r", Some(i64t.into()))?
                    }
                    Arch::X86_64 | Arch::Win64 => {
                        self.call_intrinsic("llvm.readcyclecounter", &[], &[])?
                    }
                };
                Ok(vec![v.ok_or("cycle counter produced no value")?])
            }
            Intrinsic::Pause => {
                let asm = match self.arch {
                    Arch::Aarch64 => "yield",
                    Arch::X86_64 | Arch::Win64 => "pause",
                };
                self.inline_asm(asm, "", None)?;
                Ok(vec![])
            }
            Intrinsic::Popcount | Intrinsic::Bswap => {
                let name = if op == Intrinsic::Popcount {
                    "llvm.ctpop"
                } else {
                    "llvm.bswap"
                };
                let ty = args[0].get_type();
                let v = self
                    .call_intrinsic(name, &[ty], &[args[0].into()])?
                    .ok_or("bit intrinsic produced no value")?;
                Ok(vec![v])
            }
            Intrinsic::Ctlz | Intrinsic::Cttz => {
                // The second IR argument is the width; LLVM's is_zero_poison flag stays false.
                let name = if op == Intrinsic::Ctlz {
                    "llvm.ctlz"
                } else {
                    "llvm.cttz"
                };
                let ty = args[0].get_type();
                let no_poison = self.ctx.bool_type().const_zero();
                let v = self
                    .call_intrinsic(name, &[ty], &[args[0].into(), no_poison.into()])?
                    .ok_or("bit intrinsic produced no value")?;
                Ok(vec![v])
            }
            Intrinsic::SAddOverflow
            | Intrinsic::UAddOverflow
            | Intrinsic::SSubOverflow
            | Intrinsic::USubOverflow
            | Intrinsic::SMulOverflow
            | Intrinsic::UMulOverflow => {
                // (a, b, width_bytes) -> overflowed, through the `*.with.overflow` intrinsics.
                let name = match op {
                    Intrinsic::SAddOverflow => "llvm.sadd.with.overflow",
                    Intrinsic::UAddOverflow => "llvm.uadd.with.overflow",
                    Intrinsic::SSubOverflow => "llvm.ssub.with.overflow",
                    Intrinsic::USubOverflow => "llvm.usub.with.overflow",
                    Intrinsic::SMulOverflow => "llvm.smul.with.overflow",
                    _ => "llvm.umul.with.overflow",
                };
                let x = self.as_int(args[0])?;
                let y = self.as_int(args[1])?;
                let pair = self
                    .call_intrinsic(name, &[x.get_type().into()], &[x.into(), y.into()])?
                    .ok_or("overflow intrinsic produced no value")?;
                let flag = b
                    .build_extract_value(pair.into_struct_value(), 1, "")?
                    .into_int_value();
                Ok(vec![
                    b.build_int_z_extend(flag, self.ctx.i8_type(), "")?.into(),
                ])
            }
            Intrinsic::IsCompileTime => {
                let ty = want.first().copied().unwrap_or(Ty::I8);
                Ok(vec![self.ll(ty).into_int_type().const_zero().into()])
            }
        }
    }

    fn inline_asm(
        &self,
        text: &str,
        constraints: &str,
        ret: Option<BasicTypeEnum<'ctx>>,
    ) -> R<Option<BasicValueEnum<'ctx>>> {
        let ty = self.fn_type(ret, &[], false);
        let asm = self.ctx.create_inline_asm(
            ty,
            text.to_string(),
            constraints.to_string(),
            true,
            false,
            None,
            false,
        );
        Ok(basic(self.builder.build_indirect_call(ty, asm, &[], "")?))
    }
}

enum Segment {
    /// Plain initial bytes `[start, end)`.
    Bytes(u64, u64),
    /// Index into `Global::relocs`.
    Reloc(usize),
}

fn basic<'ctx>(call: CallSiteValue<'ctx>) -> Option<BasicValueEnum<'ctx>> {
    match call.try_as_basic_value() {
        ValueKind::Basic(v) => Some(v),
        ValueKind::Instruction(_) => None,
    }
}

/// Reachable blocks of `func` in reverse post-order, entry first.
fn reverse_post_order(func: &Func) -> Vec<usize> {
    fn succs(term: &Term) -> Vec<usize> {
        match term {
            Term::Jump(t) => vec![t.0 as usize],
            Term::Branch {
                then_block,
                else_block,
                ..
            } => vec![then_block.0 as usize, else_block.0 as usize],
            Term::Switch {
                cases,
                default,
                ..
            } => cases
                .iter()
                .map(|(_, t)| t.0 as usize)
                .chain([default.0 as usize])
                .collect(),
            Term::Ret(_) | Term::Unreachable => Vec::new(),
        }
    }
    let mut visited = vec![false; func.blocks.len()];
    let mut post = Vec::new();
    // Iterative DFS: (block, next successor index).
    let mut stack = vec![(0usize, 0usize)];
    visited[0] = true;
    while let Some(&mut (block, ref mut next)) = stack.last_mut() {
        let s = succs(&func.blocks[block].term);
        if *next < s.len() {
            let t = s[*next];
            *next += 1;
            if !visited[t] {
                visited[t] = true;
                stack.push((t, 0));
            }
        } else {
            post.push(block);
            stack.pop();
        }
    }
    post.reverse();
    post
}
