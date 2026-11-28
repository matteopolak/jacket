//! Lowering one function body: parameters, stack slots, blocks, terminators and calls.
//!
//! IR values are defined once and blocks are visited in reverse post-order, so each `Val` is one
//! Cranelift `Value` used directly wherever it is needed (definitions dominate uses); no
//! variables or block parameters are needed besides the entry block's.
use cranelift_codegen::ir::{
    self, InstBuilder, MemFlagsData, StackSlotData, StackSlotKind, Value, condcodes::IntCC, types,
};
use cranelift_frontend::{FunctionBuilder, FunctionBuilderContext, Switch};
use cranelift_module::{DataId, FuncId};
use jaic::abi::{Arch, Piece};
use jaic::fxhash::HashMap;
use jaic::ir::{BlockId, Callee, Func, NullChecks, Sig, Term, Ty, Val};

use super::abi::{AbiArg, Lowered, ParamPlan, RetPlan, lower_sig};
use super::data::{ForeignSym, Helper};
use super::{R, Shared, clif_ty, unsupported};


/// What a call goes through.
pub(crate) enum Target {
    Direct(ir::FuncRef),
    Indirect(Value),
    /// x86-64: the C-variadic function at this address, called through `Helper::VariadicCall`.
    Variadic(Value),
}

/// The alignment Cranelift gives a function's frame and its stack slots at most.
const FRAME_ALIGN: u64 = 16;

/// The state of one function being lowered.
pub(crate) struct Cx<'a, 'f> {
    pub(crate) sh: &'a Shared<'a>,
    pub(crate) func: &'a Func,
    pub(crate) b: FunctionBuilder<'f>,
    vals: Vec<Option<Value>>,
    /// Per `Val`: the value of an integer constant (divisions and shifts by one need no check).
    pub(crate) consts: Vec<Option<u64>>,
    /// Per IR slot: its address, computed in the entry block.
    slots: Vec<Value>,
    blocks: Vec<Option<ir::Block>>,
    pub(crate) checks: NullChecks,
    /// (file, line) of the last `Inst::Loc`: where a failed check reports itself.
    pub(crate) loc: (u32, u32),
    /// A C aggregate returned in registers: its pieces and the memory the IR writes it to.
    reg_ret: Option<(Vec<Piece>, Value)>,
    /// Imported functions, by Cranelift id (one import per callee and signature).
    func_refs: HashMap<FuncId, ir::FuncRef>,
    data_refs: HashMap<DataId, ir::GlobalValue>,
}

/// Lower program function `index` to a Cranelift function.
pub(crate) fn lower(
    shared: &Shared,
    index: usize,
    fb: &mut FunctionBuilderContext,
) -> R<ir::Function> {
    let func = shared.program.funcs[index]
        .as_ref()
        .ok_or("function has no body")?;
    let lowered = shared.symbols.func_sigs[index]
        .as_ref()
        .ok_or("function has no signature")?;
    let mut f = ir::Function::with_name_signature(ir::UserFuncName::default(), lowered.sig.clone());
    let b = FunctionBuilder::new(&mut f, fb);
    let mut cx = Cx::new(shared, func, b);

    cx.body(lowered)?;
    cx.b.seal_all_blocks();
    cx.b.finalize(shared.isa.frontend_config());
    Ok(f)
}

impl<'a, 'f> Cx<'a, 'f> {
    pub(crate) fn new(sh: &'a Shared<'a>, func: &'a Func, b: FunctionBuilder<'f>) -> Self {
        Cx {
            sh,
            func,
            b,
            vals: vec![None; func.vals.len()],
            consts: vec![None; func.vals.len()],
            slots: Vec::with_capacity(func.slots.len()),
            blocks: vec![None; func.blocks.len()],
            checks: NullChecks::new(func),
            loc: (func.source_file, 0),
            reg_ret: None,
            func_refs: HashMap::default(),
            data_refs: HashMap::default(),
        }
    }

    fn body(&mut self, lowered: &Lowered) -> R<()> {
        let entry = self.b.create_block();

        self.b.append_block_params_for_function_params(entry);
        let order = reverse_post_order(self.func);

        for &block in &order {
            self.blocks[block] = Some(self.b.create_block());
        }

        self.b.switch_to_block(entry);

        for slot in &self.func.slots {
            let addr = self.frame_memory(slot.size, slot.align);

            self.slots.push(addr);
        }

        self.bind_params(entry, lowered)?;
        let first = self.blocks[0].ok_or("the entry block is unreachable")?;

        self.b.ins().jump(first, &[]);

        for &block in &order {
            self.b
                .switch_to_block(self.blocks[block].expect("reachable"));
            self.checks.enter_block();
            let ir_block = &self.func.blocks[block];

            for inst in &ir_block.insts {
                self.inst(inst)?;
            }

            self.term(&ir_block.term)?;
        }

        Ok(())
    }

    /// The address of `size` bytes of the frame aligned to `align`. Cranelift aligns the frame
    /// to 16 bytes, so memory aligned more strictly is a larger slot whose address is rounded up.
    fn frame_memory(&mut self, size: u64, align: u64) -> Value {
        let align = align.clamp(1, 1 << 31).next_power_of_two();

        if align <= FRAME_ALIGN {
            let shift = align.trailing_zeros() as u8;
            let slot = self.stack_slot(size, shift);

            return self.b.ins().stack_addr(types::I64, slot, 0);
        }

        let slot = self.stack_slot(
            size + align - FRAME_ALIGN,
            FRAME_ALIGN.trailing_zeros() as u8,
        );
        let base = self.b.ins().stack_addr(types::I64, slot, 0);
        let end = self.b.ins().iadd_imm_s(base, (align - 1) as i64);

        self.b.ins().band_imm_s(end, -(align as i64))
    }

    /// A stack slot of `size` bytes aligned to `1 << shift`, at most `FRAME_ALIGN`.
    fn stack_slot(&mut self, size: u64, shift: u8) -> ir::StackSlot {
        let size = u32::try_from(size.max(1)).unwrap_or(u32::MAX);

        self.b
            .create_sized_stack_slot(StackSlotData::new(StackSlotKind::ExplicitSlot, size, shift))
    }

    /// The address of new frame memory for marshalling an aggregate; rounded up so register
    /// pieces may read past its last field.
    pub(crate) fn call_temp(&mut self, size: u64, align: u64) -> Value {
        self.frame_memory(size.next_multiple_of(16), align.max(8))
    }

    pub(crate) fn slot_addr(&mut self, slot: usize) -> Value {
        self.slots[slot]
    }

    /// Bind the IR parameters to the Cranelift ones. By-value C aggregates arriving in
    /// registers are stored to memory; the IR sees a pointer to each. An aggregate result is
    /// written through the IR's out-pointer: the hidden result pointer, or memory returned in
    /// registers by `Term::Ret`.
    fn bind_params(&mut self, entry: ir::Block, lowered: &Lowered) -> R<()> {
        let params: Vec<Value> = self.b.block_params(entry).to_vec();
        let mut sret = None;
        let mut temps: Vec<Option<Value>> = vec![None; lowered.params.len()];

        for (&arg, &value) in lowered.args.iter().zip(&params) {
            match arg {
                AbiArg::Sret => sret = Some(value),
                AbiArg::Pad(_) => {}
                AbiArg::Param {
                    ir,
                    piece: None,
                } => self.vals[ir] = Some(value),
                AbiArg::Param {
                    ir,
                    piece: Some(k),
                } => {
                    let ParamPlan::Pieces(layout, pieces) = &lowered.params[ir] else {
                        return Err("a register piece of a parameter that has none".into());
                    };
                    let tmp = if let Some(t) = temps[ir] {
                        t
                    } else {
                        let t = self.call_temp(layout.size, layout.align);

                        temps[ir] = Some(t);
                        self.vals[ir] = Some(t);
                        t
                    };
                    let offset = pieces[k].offset as i32;

                    self.b.ins().store(MemFlagsData::new(), value, tmp, offset);
                }
            }
        }

        for (i, plan) in lowered.params.iter().enumerate() {
            if let ParamPlan::Dropped = plan {
                let out = match &lowered.ret {
                    RetPlan::Sret => sret.ok_or("no hidden result pointer")?,
                    RetPlan::Pieces(layout, pieces) => {
                        let tmp = self.call_temp(layout.size, layout.align);

                        self.reg_ret = Some((pieces.clone(), tmp));
                        tmp
                    }
                    RetPlan::Scalars(_) => {
                        return Err("aggregate out-pointer without a C return".into());
                    }
                };

                self.vals[i] = Some(out);
            }
        }

        Ok(())
    }

    pub(crate) fn get(&self, v: Val) -> R<Value> {
        self.vals[v.0 as usize].ok_or_else(|| format!("use of undefined value %{}", v.0).into())
    }

    pub(crate) fn set(&mut self, v: Val, value: Value) {
        self.vals[v.0 as usize] = Some(value);
    }

    /// `v` as a Cranelift value of `ty`, where the IR is loose about the width of integers and
    /// about integer/float identity of the same size.
    pub(crate) fn coerce(&mut self, v: Value, ty: ir::Type) -> Value {
        let have = self.b.func.dfg.value_type(v);

        if have == ty {
            return v;
        }

        if have.is_int() && ty.is_int() {
            return if have.bits() < ty.bits() {
                self.b.ins().uextend(ty, v)
            } else {
                self.b.ins().ireduce(ty, v)
            };
        }

        if have.bits() == ty.bits() {
            return self.b.ins().bitcast(ty, MemFlagsData::new(), v);
        }

        // A float of another width: reinterpret as an integer of the wanted width.
        let int = ir::Type::int(have.bits() as u16).expect("an integer as wide as a float");
        let bits = self.b.ins().bitcast(int, MemFlagsData::new(), v);

        self.coerce(bits, ty)
    }

    /// The IR value `v` as `ty`.
    pub(crate) fn get_as(&mut self, v: Val, ty: Ty) -> R<Value> {
        let value = self.get(v)?;
        Ok(self.coerce(value, clif_ty(ty)?))
    }

    /// The IR value `v` as a 64-bit integer (pointers, sizes and indices).
    pub(crate) fn get_i64(&mut self, v: Val) -> R<Value> {
        let value = self.get(v)?;
        Ok(self.coerce(value, types::I64))
    }

    pub(crate) fn iconst(&mut self, ty: ir::Type, value: u64) -> Value {
        let bits = ty.bits();
        let masked = if bits >= 64 {
            value
        } else {
            value & ((1u64 << bits) - 1)
        };

        self.b.ins().iconst(ty, masked as i64)
    }

    fn block(&self, id: BlockId) -> R<ir::Block> {
        self.blocks[id.0 as usize].ok_or_else(|| "branch to an unreachable block".into())
    }

    fn term(&mut self, term: &Term) -> R<()> {
        match term {
            Term::Jump(t) => {
                let t = self.block(*t)?;

                self.b.ins().jump(t, &[]);
            }
            Term::Branch {
                cond,
                then_block,
                else_block,
            } => {
                let c = self.get(*cond)?;
                let (t, e) = (self.block(*then_block)?, self.block(*else_block)?);

                self.b.ins().brif(c, t, &[], e, &[]);
            }
            Term::Switch {
                value,
                ty,
                cases,
                default,
            } => {
                let v = self.get_as(*value, *ty)?;
                let mut switch = Switch::new();

                for (c, t) in cases {
                    // A repeated case value keeps its first target, as a C switch would not
                    // allow and LLVM's verifier rejects.
                    let () = switch.set_entry(u128::from(*c), self.block(*t)?);
                }

                let default = self.block(*default)?;

                switch.emit(&mut self.b, v, default);
            }
            Term::Ret(values) => {
                if let Some((pieces, tmp)) = self.reg_ret.clone() {
                    let mut parts = Vec::with_capacity(pieces.len());

                    for (p, ret) in pieces.iter().zip(self.b.func.signature.returns.clone()) {
                        parts.push(self.b.ins().load(
                            ret.value_type,
                            MemFlagsData::new(),
                            tmp,
                            p.offset as i32,
                        ));
                    }

                    self.b.ins().return_(&parts);
                } else {
                    let tys = self.func.sig.returns.clone();
                    let mut out = Vec::with_capacity(values.len());

                    for (&v, &t) in values.iter().zip(&tys) {
                        out.push(self.get_as(v, t)?);
                    }

                    self.b.ins().return_(&out);
                }
            }
            Term::Unreachable => {
                self.b.ins().trap(super::checks::TRAP);
            }
        }

        Ok(())
    }

    // ----- symbols ------------------------------------------------------------

    /// A reference to function `id` with `sig` for this function's calls and addresses.
    pub(crate) fn func_ref(&mut self, id: FuncId, sig: &ir::Signature) -> ir::FuncRef {
        if let Some(&r) = self.func_refs.get(&id)
            && self.b.func.dfg.signatures[self.b.func.dfg.ext_funcs[r].signature] == *sig
        {
            return r;
        }

        let signature = self.b.import_signature(sig.clone());
        let name = self
            .b
            .func
            .declare_imported_user_function(ir::UserExternalName {
                namespace: 0,
                index: id.as_u32(),
            });
        let r = self.b.import_function(ir::ExtFuncData {
            name: ir::ExternalName::user(name),
            signature,
            colocated: self.sh.symbols.is_local(id),
            patchable: false,
        });

        self.func_refs.insert(id, r);
        r
    }

    /// The address of data object `id`.
    pub(crate) fn data_addr(&mut self, id: DataId, local: bool) -> Value {
        let gv = if let Some(&gv) = self.data_refs.get(&id) {
            gv
        } else {
            let name = self
                .b
                .func
                .declare_imported_user_function(ir::UserExternalName {
                    namespace: 1,
                    index: id.as_u32(),
                });
            let gv = self.b.create_global_value(ir::GlobalValueData::Symbol {
                name: ir::ExternalName::user(name),
                offset: ir::immediates::Imm64::new(0),
                colocated: local,
                tls: false,
            });

            self.data_refs.insert(id, gv);
            gv
        };
        self.b.ins().symbol_value(types::I64, gv)
    }

    pub(crate) fn func_addr(&mut self, id: FuncId) -> Value {
        let sig = ir::Signature::new(self.sh.symbols.call_conv);
        let r = self.func_ref(id, &sig);

        self.b.ins().func_addr(types::I64, r)
    }

    /// The Cranelift id of program function `id`.
    pub(crate) fn program_func(&self, id: jaic::ir::FuncId) -> R<FuncId> {
        self.sh.symbols.funcs[id.0 as usize].ok_or_else(|| {
            format!(
                "'{}' is referenced but was never lowered",
                self.sh.program.func_names[id.0 as usize]
            )
            .into()
        })
    }

    pub(crate) fn helper(&mut self, helper: Helper) -> ir::FuncRef {
        let id = self.sh.symbols.helpers[&helper];
        let sig = helper.signature(self.sh.symbols.call_conv);

        self.func_ref(id, &sig)
    }

    pub(crate) fn lower_sig(&self, sig: &Sig) -> R<Lowered> {
        let sh = self.sh;

        lower_sig(
            sig,
            sh.arch,
            sh.target.is_apple(),
            sh.symbols.call_conv,
            &sh.layout,
        )
    }

    // ----- calls ----------------------------------------------------------------

    /// Lower an IR call: its results are set.
    pub(crate) fn call_inst(&mut self, results: &[Val], callee: &Callee, args: &[Val]) -> R<()> {
        let argv = args.iter().map(|&a| self.get(a)).collect::<R<Vec<_>>>()?;
        let out = match callee {
            Callee::Func(id) => {
                let index = id.0 as usize;
                let lowered = self.sh.symbols.func_sigs[index].as_ref().ok_or_else(|| {
                    format!(
                        "call to '{}' which was never lowered",
                        self.sh.program.func_names[index]
                    )
                })?;
                let fid = self.program_func(*id)?;
                let r = self.func_ref(fid, &lowered.sig);

                self.call(Target::Direct(r), lowered, &argv)?
            }
            Callee::Foreign(id) => {
                let foreign = &self.sh.program.foreigns[id.0 as usize];

                match &self.sh.symbols.foreigns[id.0 as usize] {
                    ForeignSym::Llvm(name) => self.llvm_intrinsic(name, &argv)?,
                    ForeignSym::Data(_) => {
                        return Err(format!("call to the variable '{}'", foreign.symbol).into());
                    }
                    &ForeignSym::Func(fid) => {
                        if let Some(lowered) = &self.sh.symbols.foreign_sigs[id.0 as usize] {
                            let r = self.func_ref(fid, &lowered.sig);
                            self.call(Target::Direct(r), lowered, &argv)?
                        } else {
                            let lowered = self.call_site_sig(&foreign.sig, args)?;
                            let target = if self.sh.arch == Arch::X86_64 {
                                Target::Variadic(self.func_addr(fid))
                            } else {
                                Target::Direct(self.func_ref(fid, &lowered.sig))
                            };

                            self.call(target, &lowered, &argv)?
                        }
                    }
                }
            }
            Callee::Indirect(target, sig) => {
                let lowered = self.call_site_sig(sig, args)?;
                let ptr = self.get_i64(*target)?;
                let target = if sig.c_varargs && self.sh.arch == Arch::X86_64 {
                    Target::Variadic(ptr)
                } else {
                    Target::Indirect(ptr)
                };

                self.call(target, &lowered, &argv)?
            }
        };

        for (r, v) in results.iter().zip(out) {
            self.set(*r, v);
        }

        Ok(())
    }

    /// The lowering of `sig` for a call with `args`: a C-variadic call's extra arguments are
    /// added as the scalars they are.
    fn call_site_sig(&self, sig: &Sig, args: &[Val]) -> R<Lowered> {
        if sig.c_varargs && args.len() > sig.params.len() {
            let mut full = sig.clone();

            full.params.extend(
                args[sig.params.len()..]
                    .iter()
                    .map(|a| self.func.vals[a.0 as usize]),
            );

            return self.lower_sig(&full);
        }

        self.lower_sig(sig)
    }

    /// Emit a call with the C ABI applied to by-value aggregates; returns the IR results.
    pub(crate) fn call(
        &mut self,
        target: Target,
        lowered: &Lowered,
        args: &[Value],
    ) -> R<Vec<Value>> {
        let out_ptr = match lowered.ret {
            RetPlan::Pieces(..) | RetPlan::Sret => {
                let i = lowered
                    .params
                    .iter()
                    .position(|p| matches!(p, ParamPlan::Dropped))
                    .ok_or("missing aggregate out-pointer")?;
                Some(args[i])
            }
            RetPlan::Scalars(_) => None,
        };
        // Copies of aggregates passed in pieces or by reference.
        let mut copies: Vec<Option<Value>> = vec![None; lowered.params.len()];

        for (i, plan) in lowered.params.iter().enumerate() {
            if let ParamPlan::Pieces(layout, _) | ParamPlan::Indirect(layout) = plan {
                let tmp = self.call_temp(layout.size, layout.align);

                self.copy(tmp, args[i], layout.size);
                copies[i] = Some(tmp);
            }
        }

        let mut values = Vec::with_capacity(lowered.args.len());

        for (k, &arg) in lowered.args.iter().enumerate() {
            let want = lowered.sig.params[k].value_type;
            let v = match arg {
                AbiArg::Sret => out_ptr.ok_or("no out-pointer for the hidden result pointer")?,
                AbiArg::Pad(ty) => {
                    if ty.is_float() {
                        self.b.ins().f64const(0.0)
                    } else {
                        self.b.ins().iconst(ty, 0)
                    }
                }
                AbiArg::Param {
                    ir,
                    piece: None,
                } => match &lowered.params[ir] {
                    ParamPlan::Indirect(_) => copies[ir].expect("copied"),
                    _ => self.coerce_arg(args[ir], want),
                },
                AbiArg::Param {
                    ir,
                    piece: Some(p),
                } => {
                    let ParamPlan::Pieces(_, pieces) = &lowered.params[ir] else {
                        return Err("a register piece of an argument that has none".into());
                    };
                    let tmp = copies[ir].expect("copied");

                    self.b
                        .ins()
                        .load(want, MemFlagsData::new(), tmp, pieces[p].offset as i32)
                }
            };

            values.push(v);
        }

        let call = match target {
            Target::Direct(r) => self.b.ins().call(r, &values),
            Target::Indirect(ptr) => {
                let sig = self.b.import_signature(lowered.sig.clone());
                self.b.ins().call_indirect(sig, ptr, &values)
            }
            Target::Variadic(ptr) => {
                // Nothing else uses the pinned register, but C callers expect it preserved.
                let saved = self.b.ins().get_pinned_reg(types::I64);

                self.b.ins().set_pinned_reg(ptr);
                let id = self.sh.symbols.helpers[&Helper::VariadicCall];
                let r = self.func_ref(id, &lowered.sig);
                let call = self.b.ins().call(r, &values);

                self.b.ins().set_pinned_reg(saved);
                call
            }
        };
        let results = self.b.inst_results(call).to_vec();

        match &lowered.ret {
            RetPlan::Scalars(_) => Ok(results),
            RetPlan::Pieces(layout, pieces) => {
                let tmp = self.call_temp(layout.size, layout.align);

                for (p, v) in pieces.iter().zip(results) {
                    self.b
                        .ins()
                        .store(MemFlagsData::new(), v, tmp, p.offset as i32);
                }

                self.copy(out_ptr.expect("set above"), tmp, layout.size);
                Ok(Vec::new())
            }
            RetPlan::Sret => Ok(Vec::new()),
        }
    }

    /// An argument as the parameter type: Apple's variadic arguments are widened to 8 bytes
    /// (integers sign-extended, `float32` to `float64`, as C's promotions do).
    fn coerce_arg(&mut self, v: Value, want: ir::Type) -> Value {
        let have = self.b.func.dfg.value_type(v);

        if have == types::F32 && want == types::F64 {
            return self.b.ins().fpromote(types::F64, v);
        }

        if have.is_int() && want.is_int() && have.bits() < want.bits() {
            return self.b.ins().sextend(want, v);
        }

        self.coerce(v, want)
    }

    /// Copy `size` bytes from `src` to `dst` (non-overlapping).
    pub(crate) fn copy(&mut self, dst: Value, src: Value, size: u64) {
        if size == 0 {
            return;
        }

        let config = self.sh.isa.frontend_config();

        if size <= INLINE_COPY_MAX {
            self.b
                .emit_small_memory_copy(config, dst, src, size, 1, 1, true, MemFlagsData::new());
        } else {
            let n = self.b.ins().iconst(types::I64, size as i64);

            self.b.call_memcpy(config, dst, src, n);
        }
    }

    /// Zero `size` bytes at `dst`.
    pub(crate) fn zero(&mut self, dst: Value, size: u64) {
        if size == 0 {
            return;
        }

        let config = self.sh.isa.frontend_config();

        if size <= INLINE_COPY_MAX {
            self.b
                .emit_small_memset(config, dst, 0, size, 1, MemFlagsData::new());
        } else {
            let zero = self.b.ins().iconst(types::I8, 0);
            let n = self.b.ins().iconst(types::I64, size as i64);

            self.b.call_memset(config, dst, zero, n);
        }
    }

    /// `x != 0` for a branch condition built here.
    pub(crate) fn is_nonzero(&mut self, x: Value) -> Value {
        self.b.ins().icmp_imm_s(IntCC::NotEqual, x, 0)
    }

    /// The error for an unsupported construct inside this function.
    pub(crate) fn unsupported(&self, what: impl std::fmt::Display) -> super::Error {
        unsupported(what)
    }
}

/// The largest copy or zero fill written as loads and stores rather than a C library call.
const INLINE_COPY_MAX: u64 = 64;

/// Reachable blocks of `func` in reverse post-order, entry first. Successors are taken last to
/// first, so that in the reversed order the first one (a branch's `then`, a loop's body) follows
/// its block and unoptimized code falls through into it.
fn reverse_post_order(func: &Func) -> Vec<usize> {
    let mut visited = vec![false; func.blocks.len()];
    let mut post = Vec::with_capacity(func.blocks.len());
    let mut stack = vec![(0usize, func.blocks[0].term.successors(), 0usize)];

    visited[0] = true;

    while let Some((block, succs, next)) = stack.last_mut() {
        if *next < succs.len() {
            let t = succs[succs.len() - 1 - *next].0 as usize;

            *next += 1;

            if !visited[t] {
                visited[t] = true;
                stack.push((t, func.blocks[t].term.successors(), 0));
            }
        } else {
            post.push(*block);
            stack.pop();
        }
    }

    post.reverse();
    post
}
