//! Intrinsic and libc calls, and the null and trap checks emitted around loads and stores.
use super::error::Error;
use super::inst::basic;
use super::plan::ParamPlan;
use super::state::FnState;
use super::{Backend, R};
use inkwell::IntPredicate;
use inkwell::attributes::{Attribute, AttributeLoc};
use inkwell::intrinsics::Intrinsic as LlvmIntrinsic;
use inkwell::module::Linkage;
use inkwell::types::{BasicTypeEnum, FunctionType};
use inkwell::values::{
    BasicMetadataValueEnum, BasicValueEnum, CallSiteValue, FunctionValue, IntValue, PointerValue,
};
use jaic::ir::{Ty, Val};

impl<'ctx> Backend<'ctx, '_> {
    fn intrinsic_fn(&self, name: &str, tys: &[BasicTypeEnum<'ctx>]) -> R<FunctionValue<'ctx>> {
        LlvmIntrinsic::find(name)
            .and_then(|i| i.get_declaration(self.module, tys))
            .ok_or_else(|| Error(format!("LLVM intrinsic '{name}' is unavailable")))
    }

    pub(super) fn call_intrinsic(
        &self,
        name: &str,
        tys: &[BasicTypeEnum<'ctx>],
        args: &[BasicMetadataValueEnum<'ctx>],
    ) -> R<Option<BasicValueEnum<'ctx>>> {
        let f = self.intrinsic_fn(name, tys)?;
        Ok(basic(self.builder.build_call(f, args, "")?))
    }

    /// Call a C library function, declaring it on first use. The call goes through `ty`, not
    /// the declaration's type: a Jai definition of the same name (a wasm runtime's `memcmp`)
    /// may already exist with a different but compatible signature.
    pub(super) fn libc_call(
        &self,
        name: &str,
        ty: FunctionType<'ctx>,
        args: &[BasicMetadataValueEnum<'ctx>],
    ) -> R<CallSiteValue<'ctx>> {
        self.declare_named(name);
        let f = self.module.get_function(name).unwrap_or_else(|| {
            let f = self.module.add_function(name, ty, Some(Linkage::External));
            if self.arch.is_wasm() {
                self.wasm_import(f, name, None);
            }
            f
        });
        let ptr = f.as_global_value().as_pointer_value();
        Ok(self.builder.build_indirect_call(ty, ptr, args, "")?)
    }

    /// An intrinsic operand sema always emits as a constant (a check's reason or fatality).
    pub(super) fn constant_operand(&self, v: BasicValueEnum<'ctx>, what: &str) -> R<u64> {
        self.as_int(v)?
            .get_zero_extended_constant()
            .ok_or_else(|| format!("{what} is not a constant").into())
    }

    /// The private global holding the path of source file `file`, as failed checks print it.
    fn check_file(&self, file: u32) -> PointerValue<'ctx> {
        if let Some(&p) = self.check_files.borrow().get(&file) {
            return p;
        }
        let name = format!("__jaic_check_file.{file}");
        let pointer = if let Some(g) = self.module.get_global(&name) {
            g.as_pointer_value()
        } else {
            let path = self
                .program
                .file_paths
                .get(file as usize)
                .map(|p| jaic::display_path(std::path::Path::new(p)))
                .unwrap_or_default();
            let text = self.ctx.const_string(path.as_bytes(), true);
            let g = self.module.add_global(text.get_type(), None, &name);
            g.set_initializer(&text);
            g.set_constant(true);
            g.set_linkage(Linkage::Private);
            g.as_pointer_value()
        };
        self.check_files.borrow_mut().insert(file, pointer);
        pointer
    }

    /// The module's `jaic.null_fail(desc, address, file)`: reports a null pointer access through
    /// `Runtime_Support` and traps. `desc` is the access kind in its low byte and the source line
    /// above it. One call per check, where the check itself would need the handler's six
    /// arguments (a null check is the most common instruction sequence in an unoptimized build).
    fn null_fail_fn(&self) -> R<FunctionValue<'ctx>> {
        if let Some(&f) = self.null_fail.get() {
            return Ok(f);
        }
        let i64t = self.ctx.i64_type();
        let ty = self
            .ctx
            .void_type()
            .fn_type(&[i64t.into(), i64t.into(), self.ptr_ty().into()], false);
        let f = self
            .module
            .add_function("jaic.null_fail", ty, Some(Linkage::Internal));
        for attr in ["noreturn", "noinline", "cold", "nounwind"] {
            let kind = Attribute::get_named_enum_kind_id(attr);
            f.add_attribute(
                AttributeLoc::Function,
                self.ctx.create_enum_attribute(kind, 0),
            );
        }
        let b = self.ctx.create_builder();
        b.position_at_end(self.ctx.append_basic_block(f, "entry"));
        if self.program.check_failed.is_none() {
            // Disabling runtime support removes diagnostics, not memory checks.
            let trap = self.intrinsic_fn("llvm.trap", &[])?;
            b.build_call(trap, &[], "")?;
            b.build_unreachable()?;
            let _ = self.null_fail.set(f);
            return Ok(f);
        }
        let handler = self
            .program
            .check_failed
            .ok_or("null checks need Runtime_Support's check handler")?;
        let sig = self
            .program
            .func(handler)
            .map(|f| &f.sig)
            .ok_or("private native check reporter has no body")?;
        if sig.params.len() != 6 || sig.params[5] != Ty::Ptr {
            return Err("private native check reporter must take (reason: s64, a: s64, b: s64, fatal: bool, line: s64, filename: *u8)".into());
        }
        let lowered = self.lower_sig(sig);
        if !lowered
            .params
            .iter()
            .all(|p| matches!(p, ParamPlan::Scalar(_)))
        {
            return Err("Runtime_Support's check handler takes only scalars".into());
        }
        let param = |i: u32| -> R<BasicValueEnum<'ctx>> {
            f.get_nth_param(i)
                .ok_or_else(|| "jaic.null_fail parameter".into())
        };
        let desc = param(0)?.into_int_value();
        let address = param(1)?;
        let kind = b.build_and(desc, i64t.const_int(0xff, false), "")?;
        let line = b.build_right_shift(desc, i64t.const_int(8, false), false, "")?;
        let ints = [
            i64t.const_int(jaic::ir::CheckReason::NullPointer as u64, false),
            kind,
            address.into_int_value(),
            self.ctx.i8_type().const_int(1, false),
            line,
        ];
        let mut args: Vec<BasicMetadataValueEnum<'ctx>> = Vec::new();
        for (v, &want) in ints.iter().zip(&sig.params) {
            let want = self.ll(want).into_int_type();
            let v = if v.get_type().get_bit_width() > want.get_bit_width() {
                b.build_int_truncate(*v, want, "")?
            } else {
                b.build_int_z_extend_or_bit_cast(*v, want, "")?
            };
            args.push(v.into());
        }
        args.push(param(2)?.into());
        let target = self.func_ptr(handler)?;
        b.build_indirect_call(lowered.fn_ty, target, &args, "")?;
        let trap = self.intrinsic_fn("llvm.trap", &[])?;
        b.build_call(trap, &[], "")?;
        b.build_unreachable()?;
        let _ = self.null_fail.set(f);
        Ok(f)
    }

    /// Report a failed check through `Runtime_Support` (`Program::check_failed`) with the
    /// current source location; nothing when the program has no reporting procedure.
    pub(super) fn report_check(
        &self,
        st: &FnState<'ctx>,
        reason: u64,
        a: IntValue<'ctx>,
        b: IntValue<'ctx>,
        fatal: IntValue<'ctx>,
    ) -> R<()> {
        let Some(handler) = self.program.check_failed else {
            return Ok(());
        };
        let Some(sig) = self.program.func(handler).map(|f| &f.sig) else {
            return Err("private native check reporter has no body".into());
        };
        if sig.params.len() != 6 || sig.params[5] != Ty::Ptr {
            return Err("private native check reporter must take (reason: s64, a: s64, b: s64, fatal: bool, line: s64, filename: *u8)".into());
        }
        let (file, line) = st.loc;
        let filename = self.check_file(file);
        let i64t = self.ctx.i64_type();
        let ints = [
            i64t.const_int(reason, false),
            a,
            b,
            fatal,
            i64t.const_int(u64::from(line), false),
        ];
        let mut args: Vec<BasicValueEnum<'ctx>> = Vec::new();
        for (v, &ty) in ints.iter().zip(&sig.params) {
            let want = self.ll(ty).into_int_type();
            let v = if v.get_type().get_bit_width() > want.get_bit_width() {
                self.builder.build_int_truncate(*v, want, "")?
            } else {
                self.builder.build_int_z_extend_or_bit_cast(*v, want, "")?
            };
            args.push(v.into());
        }
        args.push(filename.into());
        self.call(st, self.func_ptr(handler)?, sig, &args)?;
        Ok(())
    }

    /// Stop with a null pointer error when `addr` lies in the never-mapped first page. Addresses
    /// of slots and globals, and offsets into them, need no check.
    pub(super) fn null_check(&self, st: &mut FnState<'ctx>, addr: Val, kind: u64) -> R<()> {
        if !st.null_checks || st.known_nonnull(addr) {
            return Ok(());
        }
        // Past this point `addr` is a real address; so is what the slot it came from holds,
        // until the next store to that slot.
        st.facts.mark(addr);
        let (from, version) = st.facts.loaded_from[addr.0 as usize];
        if let Some(slot) = from.checked_sub(1)
            && st.facts.slot_version[slot as usize] == version
        {
            st.facts.slot_checked[slot as usize] = st.facts.epoch;
        }
        let i64t = self.ctx.i64_type();
        let value = self.get_int(st, addr)?;
        let low = self.builder.build_int_compare(
            IntPredicate::ULT,
            value,
            i64t.const_int(4096, false),
            "",
        )?;
        let fail = self.null_fail_fn()?;
        let fail_block = self.ctx.append_basic_block(st.function, "trap");
        let cont = self.ctx.append_basic_block(st.function, "cont");
        self.builder
            .build_conditional_branch(low, fail_block, cont)?;
        self.builder.position_at_end(fail_block);
        let (file, line) = st.loc;
        let desc = i64t.const_int(kind | (u64::from(line) << 8), false);
        self.builder.build_call(
            fail,
            &[desc.into(), value.into(), self.check_file(file).into()],
            "",
        )?;
        self.builder.build_unreachable()?;
        self.builder.position_at_end(cont);
        Ok(())
    }

    /// Branch to a failure block when `cond` holds: it reports `reason` (with `a`, `b`) and
    /// traps.
    pub(super) fn trap_if(
        &self,
        st: &FnState<'ctx>,
        cond: IntValue<'ctx>,
        reason: jaic::ir::CheckReason,
        a: IntValue<'ctx>,
        b: IntValue<'ctx>,
    ) -> R<()> {
        let trap = self.ctx.append_basic_block(st.function, "trap");
        let cont = self.ctx.append_basic_block(st.function, "cont");
        self.builder.build_conditional_branch(cond, trap, cont)?;
        self.builder.position_at_end(trap);
        let fatal = self.ctx.i8_type().const_int(1, false);
        self.report_check(st, reason as u64, a, b, fatal)?;
        self.call_intrinsic("llvm.trap", &[], &[])?;
        self.builder.build_unreachable()?;
        self.builder.position_at_end(cont);
        Ok(())
    }
}
