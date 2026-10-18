//! Lowering terminators and calls.
use super::error::Error;
use super::inst::basic;
use super::plan::{ParamPlan, RetPlan};
use super::state::FnState;
use super::{Backend, R};
use inkwell::IntPredicate;
use inkwell::types::BasicType;
use inkwell::values::{BasicMetadataValueEnum, BasicValueEnum, PointerValue};
use jaic::abi::Passing;
use jaic::ir::{BlockId, Func, Sig, Term, Ty};

impl<'ctx> Backend<'ctx, '_> {
    pub(super) fn term(&self, st: &FnState<'ctx>, func: &Func, term: &Term) -> R<()> {
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
                let c = if let Some(flag) = st.flags[cond.0 as usize] {
                    flag
                } else {
                    let c = self.get_int(st, *cond)?;
                    b.build_int_compare(IntPredicate::NE, c, c.get_type().const_zero(), "")?
                };
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
    pub(super) fn call(
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
            let last = args.get(sig.out_index()).copied();
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
                Some(ParamPlan::AggArray(layout, pieces)) => {
                    let src = self.coerce(arg, Ty::Ptr)?.into_pointer_value();
                    let tmp = self.call_temp(st, layout.size, layout.align)?;
                    self.memcpy(tmp, src, layout.size)?;
                    let elem = self.piece_ty(pieces[0].ty);
                    let mut array: BasicValueEnum<'ctx> =
                        elem.array_type(pieces.len() as u32).get_undef().into();
                    for (n, p) in pieces.iter().enumerate() {
                        let at = self.gep_const(tmp, p.offset)?;
                        let piece = b.build_load(self.piece_ty(p.ty), at, "")?;
                        array = b
                            .build_insert_value(array.into_array_value(), piece, n as u32, "")?
                            .into_array_value()
                            .into();
                    }
                    ll_args.push(array.into());
                }
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
    pub(super) fn call_temp(
        &self,
        st: &FnState<'ctx>,
        size: u64,
        align: u64,
    ) -> R<PointerValue<'ctx>> {
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
}
