//! Defining a function body: parameters, value slots and block order.
use super::error::Error;
use super::plan::{ParamPlan, RetPlan};
use super::state::{FnState, NullFacts};
use super::{Backend, R};
use inkwell::basic_block::BasicBlock;
use inkwell::types::{BasicTypeEnum, IntType};
use inkwell::values::{BasicValueEnum, FunctionValue, IntValue, PointerValue};
use jaic::abi::Passing;
use jaic::ir::{Func, Linkage as IrLinkage, Term, Ty, Val};

impl<'ctx> Backend<'ctx, '_> {
    pub(super) fn define_function(&self, func: &Func, function: FunctionValue<'ctx>) -> R<()> {
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
            loc: (func.source_file, 0),
            null_checks: func.null_checks,
            nonnull: vec![false; func.vals.len()],
            facts: NullFacts::default(),
            flags: vec![None; func.vals.len()],
        };
        for v in &func.trusted {
            st.nonnull[v.0 as usize] = true;
        }
        if st.null_checks {
            st.facts = NullFacts::new(func);
        }
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
            st.facts.enter_block();
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
                ParamPlan::AggArray(layout, pieces) => {
                    let array = params[k].into_array_value();
                    k += 1;
                    let tmp = self.call_temp(st, layout.size, layout.align)?;
                    for (n, p) in pieces.iter().enumerate() {
                        let piece = self.builder.build_extract_value(array, n as u32, "")?;
                        self.builder
                            .build_store(self.gep_const(tmp, p.offset)?, piece)?;
                    }
                    tmp.into()
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

    pub(super) fn entry_alloca(
        &self,
        st: &FnState<'ctx>,
        size: u64,
        align: u64,
    ) -> R<PointerValue<'ctx>> {
        let ptr = st
            .allocas
            .build_alloca(self.bytes_ty(size.max(1)), "slot")?;
        if let Some(inst) = ptr.as_instruction() {
            inst.set_alignment(align.clamp(1, 1 << 16).next_power_of_two() as u32)
                .map_err(|e| Error(e.to_string()))?;
        }
        Ok(ptr)
    }

    pub(super) fn get(&self, st: &FnState<'ctx>, v: Val) -> R<BasicValueEnum<'ctx>> {
        if let Some(value) = st.vals[v.0 as usize] {
            return Ok(value);
        }
        // A comparison's byte is widened where it is used, if it is: a branch on it uses the
        // `i1` and leaves the compare with one user, which is what lets instruction selection
        // fuse the compare into the branch.
        match st.flags[v.0 as usize] {
            Some(flag) => Ok(self
                .builder
                .build_int_z_extend(flag, self.ctx.i8_type(), "")?
                .into()),
            None => Err(Error(format!("use of undefined value %{}", v.0))),
        }
    }

    pub(super) fn set(&self, st: &mut FnState<'ctx>, v: Val, value: BasicValueEnum<'ctx>) {
        st.vals[v.0 as usize] = Some(value);
        // A variable whose address is computed here: copy it where its debug info looks.
        if let Some(slot) = st.dbg.as_ref().and_then(|d| d.watched(v)) {
            let _ = self.builder.build_store(slot, value);
        }
    }

    pub(super) fn get_int(&self, st: &FnState<'ctx>, v: Val) -> R<IntValue<'ctx>> {
        self.as_int(self.get(st, v)?)
    }

    pub(super) fn get_ptr(&self, st: &FnState<'ctx>, v: Val) -> R<PointerValue<'ctx>> {
        let value = self.get(st, v)?;
        self.coerce(value, Ty::Ptr).map(|p| p.into_pointer_value())
    }

    /// View any scalar as an integer (pointers via `ptrtoint`).
    pub(super) fn as_int(&self, v: BasicValueEnum<'ctx>) -> R<IntValue<'ctx>> {
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
    /// The integer type an atomic access of an IR `ty` goes through: LLVM's atomic operations
    /// compare and exchange bits, so floats and pointers use the integer of their width.
    pub(super) fn atomic_int_ty(&self, ty: Ty) -> R<IntType<'ctx>> {
        Ok(match ty {
            Ty::I8 => self.ctx.i8_type(),
            Ty::I16 => self.ctx.i16_type(),
            Ty::I32 | Ty::F32 => self.ctx.i32_type(),
            Ty::I64 | Ty::F64 | Ty::Ptr => self.ctx.i64_type(),
            _ => return Err("an atomic operation takes 1, 2, 4 or 8 bytes".into()),
        })
    }

    /// `v` as the integer of its width (see `atomic_int_ty`).
    pub(super) fn atomic_bits(&self, v: BasicValueEnum<'ctx>) -> R<IntValue<'ctx>> {
        match v {
            BasicValueEnum::FloatValue(f) => {
                let int = match f.get_type().get_bit_width() {
                    16 => self.ctx.i16_type(),
                    32 => self.ctx.i32_type(),
                    64 => self.ctx.i64_type(),
                    _ => return Err("an atomic operation takes 1, 2, 4 or 8 bytes".into()),
                };
                Ok(self.builder.build_bit_cast(f, int, "")?.into_int_value())
            }
            other => self.as_int(other),
        }
    }

    pub(super) fn coerce(&self, v: BasicValueEnum<'ctx>, ty: Ty) -> R<BasicValueEnum<'ctx>> {
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

    pub(super) fn gep(
        &self,
        base: PointerValue<'ctx>,
        offset: IntValue<'ctx>,
    ) -> R<PointerValue<'ctx>> {
        // Inkwell marks GEP construction unsafe; an i8 GEP is plain pointer arithmetic.
        #[allow(unsafe_code)]
        // SAFETY: inkwell marks GEP construction unsafe; an i8 GEP is plain pointer arithmetic.
        let p = unsafe {
            self.builder
                .build_gep(self.ctx.i8_type(), base, &[offset], "")?
        };
        Ok(p)
    }

    pub(super) fn gep_const(&self, base: PointerValue<'ctx>, offset: u64) -> R<PointerValue<'ctx>> {
        if offset == 0 {
            return Ok(base);
        }
        self.gep(base, self.ctx.i64_type().const_int(offset, false))
    }

    pub(super) fn i64_of(&self, v: IntValue<'ctx>) -> R<IntValue<'ctx>> {
        Ok(self
            .builder
            .build_int_z_extend_or_bit_cast(v, self.ctx.i64_type(), "")?)
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
    // Iterative DFS: (block, successors visited so far). Successors are taken last to first, so
    // that in the reversed order the first one (a branch's `then`, a loop's body) follows its
    // block and unoptimised code (no block placement) falls through into it.
    let mut stack = vec![(0usize, 0usize)];
    visited[0] = true;
    while let Some(&mut (block, ref mut next)) = stack.last_mut() {
        let s = succs(&func.blocks[block].term);
        if *next < s.len() {
            let t = s[s.len() - 1 - *next];
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
