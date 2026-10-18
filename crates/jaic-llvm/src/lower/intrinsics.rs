//! Lowering compiler intrinsics, wide floats, fibers and inline assembly.
use super::error::Error;
use super::inst::basic;
use super::state::FnState;
use super::{Backend, R};
use inkwell::AtomicOrdering;
use inkwell::AtomicRMWBinOp;
use inkwell::FloatPredicate;
use inkwell::IntPredicate;
use inkwell::module::Linkage;
use inkwell::types::{BasicType, BasicTypeEnum, FunctionType};
use inkwell::values::{
    BasicMetadataValueEnum, BasicValue, BasicValueEnum, FunctionValue, IntValue, PointerValue,
};
use jaic::abi::Arch;
use jaic::ir::{self, AtomicOp, CmpOp, Intrinsic, Ty, WideArith, WideFloat, WideOp};

impl<'ctx, 'p> Backend<'ctx, 'p> {
    /// What the SIMD intrinsics (`ByteLookup`, `SignMask`) lower through: see `simd`.
    fn simd_cx(&self) -> crate::simd::Cx<'_, 'ctx> {
        crate::simd::Cx {
            ctx: self.ctx,
            builder: &self.builder,
            module: self.module,
            target: &self.simd,
        }
    }

    /// `long double` operations: native `x86_fp80` / `fp128` arithmetic on values in memory.
    fn wide(
        &self,
        op: WideOp,
        fmt: WideFloat,
        args: &[BasicValueEnum<'ctx>],
    ) -> R<Vec<BasicValueEnum<'ctx>>> {
        let b = &self.builder;
        let ft = self.wide_ty(fmt);
        let ptr = |i: usize| -> R<PointerValue<'ctx>> {
            Ok(self.coerce(args[i], Ty::Ptr)?.into_pointer_value())
        };
        let load = |i: usize| -> R<inkwell::values::FloatValue<'ctx>> {
            Ok(b.build_load(ft, ptr(i)?, "")?.into_float_value())
        };
        let store = |v: inkwell::values::FloatValue<'ctx>| -> R<Vec<BasicValueEnum<'ctx>>> {
            b.build_store(ptr(0)?, v)?;
            Ok(Vec::new())
        };
        let scalar = |i: usize, ty: Ty| -> R<inkwell::values::FloatValue<'ctx>> {
            Ok(self.coerce(args[i], ty)?.into_float_value())
        };
        match op {
            WideOp::Arith(kind) => {
                let (x, y) = (load(1)?, load(2)?);
                store(match kind {
                    WideArith::Add => b.build_float_add(x, y, "")?,
                    WideArith::Sub => b.build_float_sub(x, y, "")?,
                    WideArith::Mul => b.build_float_mul(x, y, "")?,
                    WideArith::Div => b.build_float_div(x, y, "")?,
                })
            }
            WideOp::Neg => store(b.build_float_neg(load(1)?, "")?),
            WideOp::Cmp(cmp) => {
                let pred = match cmp {
                    CmpOp::FEq => FloatPredicate::OEQ,
                    CmpOp::FNe => FloatPredicate::UNE,
                    CmpOp::FLt => FloatPredicate::OLT,
                    CmpOp::FLe => FloatPredicate::OLE,
                    CmpOp::FGt => FloatPredicate::OGT,
                    CmpOp::FGe => FloatPredicate::OGE,
                    _ => return Err("integer comparison on a long double".into()),
                };
                let r = b.build_float_compare(pred, load(0)?, load(1)?, "")?;
                Ok(vec![
                    b.build_int_z_extend(r, self.ctx.i8_type(), "")?.into(),
                ])
            }
            WideOp::FromF64 => store(b.build_float_ext(scalar(1, Ty::F64)?, ft, "")?),
            WideOp::FromF32 => store(b.build_float_ext(scalar(1, Ty::F32)?, ft, "")?),
            WideOp::FromS64 => {
                let v = self.i64_of(self.as_int(args[1])?)?;
                store(b.build_signed_int_to_float(v, ft, "")?)
            }
            WideOp::FromU64 => {
                let v = self.i64_of(self.as_int(args[1])?)?;
                store(b.build_unsigned_int_to_float(v, ft, "")?)
            }
            WideOp::ToF64 => Ok(vec![
                b.build_float_trunc(load(0)?, self.ctx.f64_type(), "")?
                    .into(),
            ]),
            WideOp::ToF32 => Ok(vec![
                b.build_float_trunc(load(0)?, self.ctx.f32_type(), "")?
                    .into(),
            ]),
            WideOp::ToS64 => Ok(vec![
                b.build_float_to_signed_int(load(0)?, self.ctx.i64_type(), "")?
                    .into(),
            ]),
            WideOp::ToU64 => Ok(vec![
                b.build_float_to_unsigned_int(load(0)?, self.ctx.i64_type(), "")?
                    .into(),
            ]),
        }
    }

    pub(super) fn intrinsic(
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
                let r = basic(self.libc_call(
                    "memcmp",
                    ty,
                    &[ptr_arg(0)?.into(), ptr_arg(1)?.into(), size_arg(2)?.into()],
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
                // cmpxchg compares bits and takes only integers: a float goes through the
                // integer of its width, and the previous value comes back as that float.
                let float = match args[1] {
                    BasicValueEnum::FloatValue(f) => Some(f.get_type()),
                    _ => None,
                };
                let bits = |v: BasicValueEnum<'ctx>| -> R<IntValue<'ctx>> {
                    match v {
                        BasicValueEnum::FloatValue(f) => {
                            let int = match f.get_type().get_bit_width() {
                                16 => self.ctx.i16_type(),
                                32 => self.ctx.i32_type(),
                                64 => self.ctx.i64_type(),
                                _ => self.ctx.i128_type(),
                            };
                            Ok(b.build_bit_cast(f, int, "")?.into_int_value())
                        }
                        other => self.as_int(other),
                    }
                };
                let old = bits(args[1])?;
                let new = bits(args[2])?;
                let new = b.build_int_truncate_or_bit_cast(new, old.get_type(), "")?;
                let res = b.build_cmpxchg(
                    p,
                    old,
                    new,
                    AtomicOrdering::SequentiallyConsistent,
                    AtomicOrdering::SequentiallyConsistent,
                )?;
                let mut prev = b.build_extract_value(res, 0, "")?;
                if let Some(float) = float {
                    prev = b.build_bit_cast(prev, float, "")?;
                }
                let ok = b.build_extract_value(res, 1, "")?.into_int_value();
                let ok = b.build_int_z_extend(ok, self.ctx.i8_type(), "")?;
                Ok(vec![ok.into(), prev])
            }
            Intrinsic::AtomicLoad(order) => {
                // (ptr, width) -> value. LLVM wants the alignment of an atomic access stated.
                let ty = want.first().copied().ok_or("an atomic load has no result")?;
                let int = self.atomic_int_ty(ty)?;
                let load = b.build_load(int, ptr_arg(0)?, "")?;
                let inst = load
                    .as_instruction_value()
                    .ok_or("an atomic load is not an instruction")?;
                inst.set_atomic_ordering(atomic_ordering(order))
                    .map_err(|e| Error(format!("atomic ordering: {e:?}")))?;
                inst.set_alignment(int.get_bit_width() / 8)
                    .map_err(|e| Error(format!("atomic alignment: {e:?}")))?;
                Ok(vec![self.coerce(load, ty)?])
            }
            Intrinsic::AtomicStore(order) => {
                let value = self.atomic_bits(args[1])?;
                let inst = b.build_store(ptr_arg(0)?, value)?;
                inst.set_atomic_ordering(atomic_ordering(order))
                    .map_err(|e| Error(format!("atomic ordering: {e:?}")))?;
                inst.set_alignment(value.get_type().get_bit_width() / 8)
                    .map_err(|e| Error(format!("atomic alignment: {e:?}")))?;
                Ok(vec![])
            }
            Intrinsic::AtomicRmw(rmw, order) => {
                let ty = want.first().copied().ok_or("an atomic update has no result")?;
                let value = self.atomic_bits(args[1])?;
                let rmw = match rmw {
                    AtomicOp::Xchg => AtomicRMWBinOp::Xchg,
                    AtomicOp::Add => AtomicRMWBinOp::Add,
                    AtomicOp::Sub => AtomicRMWBinOp::Sub,
                    AtomicOp::And => AtomicRMWBinOp::And,
                    AtomicOp::Or => AtomicRMWBinOp::Or,
                    AtomicOp::Xor => AtomicRMWBinOp::Xor,
                };
                let old = b.build_atomicrmw(rmw, ptr_arg(0)?, value, atomic_ordering(order))?;
                Ok(vec![self.coerce(old.into(), ty)?])
            }
            Intrinsic::AtomicCas(success, failure) => {
                let ty = want.get(1).copied().ok_or("a compare-exchange has no value")?;
                let old = self.atomic_bits(args[1])?;
                let new = self.atomic_bits(args[2])?;
                let new = b.build_int_truncate_or_bit_cast(new, old.get_type(), "")?;
                let res = b.build_cmpxchg(
                    ptr_arg(0)?,
                    old,
                    new,
                    atomic_ordering(success),
                    atomic_ordering(failure),
                )?;
                let prev = b.build_extract_value(res, 0, "")?;
                let ok = b.build_extract_value(res, 1, "")?.into_int_value();
                let ok = b.build_int_z_extend(ok, self.ctx.i8_type(), "")?;
                Ok(vec![ok.into(), self.coerce(prev, ty)?])
            }
            Intrinsic::DebugBreak => {
                self.call_intrinsic("llvm.debugtrap", &[], &[])?;
                Ok(vec![])
            }
            Intrinsic::Trap => {
                if let Some(&reason) = args.first() {
                    let reason = self.constant_operand(reason, "trap reason")?;
                    let zero = i64t.const_zero();
                    let fatal = self.ctx.i8_type().const_int(1, false);
                    self.report_check(st, reason, zero, zero, fatal)?;
                }
                self.call_intrinsic("llvm.trap", &[], &[])?;
                Ok(vec![])
            }
            Intrinsic::CheckFailed => {
                let reason = self.constant_operand(args[0], "check reason")?;
                let a = self.i64_of(self.as_int(args[1])?)?;
                let bv = self.i64_of(self.as_int(args[2])?)?;
                let fatal = self.constant_operand(args[3], "check fatality")?;
                let fatal_value = self.ctx.i8_type().const_int(fatal, false);
                self.report_check(st, reason, a, bv, fatal_value)?;
                if fatal != 0 {
                    self.call_intrinsic("llvm.trap", &[], &[])?;
                }
                Ok(vec![])
            }
            Intrinsic::BoundsCheck => {
                // Unsigned compare: a negative index is out of range too.
                let index = self.i64_of(self.as_int(args[0])?)?;
                let count = self.i64_of(self.as_int(args[1])?)?;
                let out = b.build_int_compare(IntPredicate::UGE, index, count, "")?;
                self.trap_if(st, out, jaic::ir::CheckReason::Bounds, index, count)?;
                Ok(vec![])
            }
            Intrinsic::CompilerWrite if self.arch.is_wasm() => {
                // WebAssembly has no `write`; Runtime_Support's import (`wasm_write_string`)
                // takes the same three arguments, count first.
                let ty = self.ctx.void_type().fn_type(
                    &[i64t.into(), self.ptr_ty().into(), self.ctx.i8_type().into()],
                    false,
                );
                let to_err = b.build_int_truncate_or_bit_cast(
                    self.as_int(args[2])?,
                    self.ctx.i8_type(),
                    "",
                )?;
                self.libc_call(
                    "wasm_write_string",
                    ty,
                    &[size_arg(1)?.into(), ptr_arg(0)?.into(), to_err.into()],
                )?;
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
                let name = if self.arch.is_windows() {
                    "_write"
                } else {
                    "write"
                };
                self.libc_call(
                    name,
                    ty,
                    &[fd.into(), ptr_arg(0)?.into(), size_arg(1)?.into()],
                )?;
                Ok(vec![])
            }
            Intrinsic::Wide(op, fmt) => self.wide(op, fmt, args),
            Intrinsic::ByteLookup(width) => {
                let (dst, table, idx) = (ptr_arg(0)?, ptr_arg(1)?, ptr_arg(2)?);
                crate::simd::byte_lookup(&self.simd_cx(), width, dst, table, idx)?;
                Ok(vec![])
            }
            Intrinsic::SignMask(width) => {
                let mask = crate::simd::sign_mask(&self.simd_cx(), width, ptr_arg(0)?)?;
                Ok(vec![mask.into()])
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
            // WebAssembly code cannot see its own call stack.
            Intrinsic::ReturnAddress if self.arch.is_wasm() => {
                Ok(vec![self.ptr_ty().const_null().into()])
            }
            Intrinsic::ReturnAddress => {
                let zero = self.ctx.i32_type().const_zero();
                let v = self
                    .call_intrinsic("llvm.returnaddress", &[], &[zero.into()])?
                    .ok_or("returnaddress produced no value")?;
                Ok(vec![v])
            }
            Intrinsic::CpuId => {
                if !self.arch.is_x86_64() {
                    return Err("CPUID requires an x86-64 target".into());
                }
                let i32t = self.ctx.i32_type();
                let outputs = self.ctx.struct_type(&[i32t.into(); 4], false);
                let ty = self.fn_type(Some(outputs.into()), &[i32t.into(); 2], false);
                // Tied EAX/ECX inputs select leaf/subleaf. LLVM owns saving EBX under
                // each platform ABI. Side effects and the memory clobber retain
                // the architectural serialization boundary through optimization.
                let asm = self.ctx.create_inline_asm(
                    ty,
                    "cpuid".into(),
                    "={ax},={bx},={cx},={dx},0,2,~{memory},~{dirflag},~{fpsr},~{flags}".into(),
                    true, false, None, false,
                );
                let call_args = args.iter()
                    .map(|&v| self.as_int(v).map(Into::into))
                    .collect::<R<Vec<BasicMetadataValueEnum>>>()?;
                let value = basic(self.builder.build_indirect_call(ty, asm, &call_args, "")?)
                    .ok_or("CPUID produced no value")?.into_struct_value();
                (0..4).map(|i| {
                    self.builder.build_extract_value(value, i, "").map_err(Into::into)
                }).collect()
            }
            Intrinsic::X86Xgetbv => {
                if !self.arch.is_x86_64() {
                    return Err("XGETBV requires an x86-64 target".into());
                }
                let i32t = self.ctx.i32_type();
                let outputs = self.ctx.struct_type(&[i32t.into(); 2], false);
                let ty = self.fn_type(Some(outputs.into()), &[i32t.into()], false);
                // Keep the architectural read even when its outputs are unused.
                // XGETBV preserves EFLAGS and does not access process memory.
                let asm = self.ctx.create_inline_asm(
                    ty, "xgetbv".into(), "={dx},={ax},{cx}".into(),
                    true, false, None, false,
                );
                let index = self.as_int(args[0])?;
                let result = basic(self.builder.build_indirect_call(ty, asm, &[index.into()], "")?)
                    .ok_or("XGETBV produced no value")?.into_struct_value();
                (0..2).map(|i| {
                    self.builder.build_extract_value(result, i, "").map_err(Into::into)
                }).collect()
            }
            Intrinsic::X86Timestamp { aux } => {
                if !self.arch.is_x86_64() {
                    return Err("x86 timestamp instruction requires an x86-64 target".into());
                }
                let i32t = self.ctx.i32_type();
                let fields = [i32t.into(); 3];
                let outputs = self.ctx.struct_type(&fields[..if aux { 3 } else { 2 }], false);
                let (instruction, constraints) = if aux {
                    ("rdtscp", "={ax},={dx},={cx},~{memory}")
                } else {
                    ("rdtsc", "={ax},={dx},~{memory}")
                };
                let value = self.inline_asm(instruction, constraints, Some(outputs.into()))?
                    .ok_or("timestamp instruction produced no value")?.into_struct_value();
                let lo = self.builder.build_extract_value(value, 0, "")?.into_int_value();
                let hi = self.builder.build_extract_value(value, 1, "")?.into_int_value();
                let lo = self.builder.build_int_z_extend(lo, i64t, "")?;
                let hi = self.builder.build_int_z_extend(hi, i64t, "")?;
                let hi = self.builder.build_left_shift(hi, i64t.const_int(32, false), "")?;
                let timestamp = self.builder.build_or(lo, hi, "")?;
                let mut result = vec![timestamp.into()];
                if aux { result.push(self.builder.build_extract_value(value, 2, "")?); }
                Ok(result)
            }
            Intrinsic::X86Random { seed, width } => {
                if !self.arch.is_x86_64() {
                    return Err("x86 random instruction requires an x86-64 target".into());
                }
                let (ty, suffix) = match width {
                    Ty::I16 => (self.ctx.i16_type(), "w"),
                    Ty::I32 => (self.ctx.i32_type(), "l"),
                    Ty::I64 => (i64t, "q"),
                    _ => return Err("invalid x86 random instruction width".into()),
                };
                let outputs = self.ctx.struct_type(&[ty.into(), self.ctx.i8_type().into()], false);
                let instruction = if seed { "rdseed" } else { "rdrand" };
                let value = self.inline_asm(&format!("{instruction}{suffix} $0; setc $1"), "=&r,=&q,~{flags}", Some(outputs.into()))?
                    .ok_or("random instruction produced no value")?.into_struct_value();
                (0..2).map(|i| self.builder.build_extract_value(value, i, "").map_err(Into::into)).collect()
            }
            Intrinsic::X86Fence(kind) => {
                if !self.arch.is_x86_64() {
                    return Err("x86 memory fence requires an x86-64 target".into());
                }
                let instruction = match kind {
                    ir::X86Fence::Load => "lfence",
                    ir::X86Fence::Store => "sfence",
                    ir::X86Fence::Full => "mfence",
                };
                self.inline_asm(instruction, "~{memory}", None)?;
                Ok(vec![])
            }
            Intrinsic::X86Pause => {
                if !self.arch.is_x86_64() {
                    return Err("x86 PAUSE requires an x86-64 target".into());
                }
                self.inline_asm("pause", "", None)?;
                Ok(vec![])
            }
            Intrinsic::X86Prefetch(hint) => {
                if !self.arch.is_x86_64() {
                    return Err("x86 prefetch requires an x86-64 target".into());
                }
                let [address] = args else {
                    return Err("invalid x86 prefetch operands".into());
                };
                let pointer = self.coerce(*address, Ty::Ptr)?.into_pointer_value();
                let ty = self.fn_type(None, &[pointer.get_type().into()], false);
                let instruction = match hint {
                    ir::X86Prefetch::T0 => "prefetcht0",
                    ir::X86Prefetch::T1 => "prefetcht1",
                    ir::X86Prefetch::T2 => "prefetcht2",
                    ir::X86Prefetch::NonTemporal => "prefetchnta",
                };
                let asm = self.ctx.create_inline_asm(ty, format!("{instruction} ($0)"), "r".into(), true, false, None, false);
                self.builder.build_indirect_call(ty, asm, &[pointer.into()], "")?;
                Ok(vec![])
            }
            Intrinsic::CycleCounter => {
                let v = match self.arch {
                    // The user-space virtual counter; PMCCNTR is not readable from EL0.
                    Arch::Aarch64 | Arch::Win64Arm => {
                        self.inline_asm("mrs $0, cntvct_el0", "=r", Some(i64t.into()))?
                    }
                    Arch::X86_64 | Arch::Win64 => {
                        self.call_intrinsic("llvm.readcyclecounter", &[], &[])?
                    }
                    // No cycle counter is visible to wasm code.
                    Arch::Wasm64 => return Err("no architectural cycle counter is available on a WebAssembly target".into()),
                };
                Ok(vec![v.ok_or("cycle counter produced no value")?])
            }
            Intrinsic::Syscall => {
                if !matches!(self.arch, Arch::X86_64 | Arch::Win64) {
                    return Err("`syscall` in #asm needs an x86-64 target".into());
                }
                let i64t = self.ctx.i64_type();
                let ty = self.fn_type(Some(i64t.into()), &[i64t.into(); 7], false);
                // rax = number; rdi, rsi, rdx, r10, r8, r9 = arguments; rcx and r11 are clobbered.
                let asm = self.ctx.create_inline_asm(
                    ty,
                    "syscall".to_string(),
                    "={rax},{rax},{rdi},{rsi},{rdx},{r10},{r8},{r9},~{rcx},~{r11},~{memory},~{dirflag},~{fpsr},~{flags}"
                        .to_string(),
                    true,
                    false,
                    None,
                    false,
                );
                let call_args = args
                    .iter()
                    .map(|&v| self.as_int(v).map(Into::into))
                    .collect::<R<Vec<BasicMetadataValueEnum>>>()?;
                let v = basic(self.builder.build_indirect_call(ty, asm, &call_args, "")?)
                    .ok_or("syscall produced no value")?;
                Ok(vec![v])
            }
            Intrinsic::Pause => {
                let asm = match self.arch {
                    Arch::Aarch64 | Arch::Win64Arm => "yield",
                    Arch::X86_64 | Arch::Win64 => "pause",
                    Arch::Wasm64 => return Ok(vec![]),
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
            Intrinsic::FiberInit | Intrinsic::FiberSwitch | Intrinsic::FiberFree
                if self.arch.is_wasm() => Err(
                "fibers are not supported when compiling for WebAssembly: wasm code cannot switch stacks"
                    .into(),
            ),
            // Native fibers hold nothing but their stacks, which the program owns.
            Intrinsic::FiberFree => Ok(vec![]),
            Intrinsic::FiberSwitch => {
                let void = self.ctx.void_type();
                let ty = void.fn_type(&[self.ptr_ty().into(), self.ptr_ty().into()], false);
                let f = self.fiber_routine(jaic::fiber::SWITCH, ty)?;
                b.build_call(f, &[ptr_arg(0)?.into(), ptr_arg(1)?.into()], "")?;
                Ok(vec![])
            }
            Intrinsic::FiberInit => {
                // (fiber, stack, size, entry, data): `prepare` lays out the first frame below the
                // top of the stack; the context also records the stack's bounds, which Windows
                // wants in the thread information block when it switches to the fiber.
                let ptr = self.ptr_ty();
                let ty = i64t.fn_type(&[i64t.into(), ptr.into(), ptr.into()], false);
                let prepare = self.fiber_routine(jaic::fiber::PREPARE, ty)?;
                let (fiber, stack, size) = (ptr_arg(0)?, ptr_arg(1)?, size_arg(2)?);
                let low = b.build_ptr_to_int(stack, i64t, "")?;
                let high = b.build_int_add(low, size, "")?;
                let sp = basic(b.build_call(
                    prepare,
                    &[high.into(), ptr_arg(3)?.into(), ptr_arg(4)?.into()],
                    "",
                )?)
                .ok_or("fiber prepare produced no value")?
                .into_int_value();
                for (offset, value) in [
                    (jaic::fiber::SP, sp),
                    (jaic::fiber::STACK_LOW, low),
                    (jaic::fiber::STACK_HIGH, high),
                    (jaic::fiber::STACK_DEALLOC, low),
                ] {
                    b.build_store(self.gep_const(fiber, offset)?, value)?;
                }
                Ok(vec![])
            }
            Intrinsic::IsCompileTime => {
                let ty = want.first().copied().unwrap_or(Ty::I8);
                Ok(vec![self.ll(ty).into_int_type().const_zero().into()])
            }
        }
    }

    /// The fiber routine `name` (`jaic::fiber`), declared on first use. The assembly that defines
    /// both routines goes into the module with the first one, with symbols local to the module.
    fn fiber_routine(&self, name: &str, ty: FunctionType<'ctx>) -> R<FunctionValue<'ctx>> {
        if let Some(f) = self.module.get_function(name) {
            return Ok(f);
        }
        let text = jaic::fiber::module_asm(self.arch, self.target.is_apple())
            .ok_or("fibers need a native target")?;
        self.module.set_inline_assembly(&text);
        Ok(self.module.add_function(name, ty, Some(Linkage::External)))
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

/// The LLVM ordering of an `ir::ORDER_*` constant.
fn atomic_ordering(order: u8) -> AtomicOrdering {
    match order {
        ir::ORDER_RELAXED => AtomicOrdering::Monotonic,
        ir::ORDER_ACQUIRE => AtomicOrdering::Acquire,
        ir::ORDER_RELEASE => AtomicOrdering::Release,
        ir::ORDER_ACQ_REL => AtomicOrdering::AcquireRelease,
        _ => AtomicOrdering::SequentiallyConsistent,
    }
}
