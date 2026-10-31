//! Calls to compiler intrinsics (`#intrinsic` procedures and atomics).

use super::{
    CallArg, Compiler, Diagnostic, FnCtx, Operand, OrderedAtomic, ProcId, Result, Signature, Slot,
    Span, TargetCpu, Ty, TypeId, TypeKind, Value, ast, err, ir,
};
use crate::diagnostics::Code;

impl Compiler {
    /// Intrinsic procedures declared in Preload (`memcpy`, `compare_and_swap`...).
    pub(super) fn emit_intrinsic(
        &mut self,
        f: &mut FnCtx,
        proc: ProcId,
        (sig, header): (&Signature, &ast::ProcHeader),
        slots: &[Slot],
        args: Vec<CallArg>,
        span: Span,
    ) -> Result<Operand> {
        let name = self.proc(proc).name;
        if name.as_str() == crate::sema::private_names::RUNTIME_TEMPORARY_STORAGE_SIZE {
            return self.runtime_storage_operand(f, sig, span);
        }
        let mut values = Vec::new();
        let mut k = 0;
        for (i, slot) in slots.iter().enumerate() {
            if header.params[i].baked {
                continue;
            }
            let param = sig.params[k].clone();
            k += 1;
            values.push(self.param_value(f, sig, &param, slot, &args, span)?);
        }
        if name.as_str() == "black_box" {
            return self.emit_black_box(f, sig, &values, span);
        }
        let op = match name.as_str() {
            "memcpy" => ir::Intrinsic::Memcpy,
            "memset" => ir::Intrinsic::Memset,
            "memcmp" => ir::Intrinsic::Memcmp,
            "compare_and_swap" => ir::Intrinsic::AtomicCas(ir::ORDER_SEQ_CST, ir::ORDER_SEQ_CST),
            "atomic_load" => ir::Intrinsic::AtomicLoad(ir::ORDER_SEQ_CST),
            "atomic_store" => ir::Intrinsic::AtomicStore(ir::ORDER_SEQ_CST),
            "atomic_exchange" => ir::Intrinsic::AtomicRmw(ir::AtomicOp::Xchg, ir::ORDER_SEQ_CST),
            ordered if OrderedAtomic::from_name(ordered).is_some() => {
                let kind = OrderedAtomic::from_name(ordered).unwrap();
                // The last parameter is the memory order, a compile-time constant that the IR
                // op carries (a compare-exchange packs success | failure << 4).
                let mode = self.const_int_arg(f, slots, &args, ordered, span)?;
                if !header.params.last().is_some_and(|p| p.baked) {
                    values.pop();
                }
                self.atomic_intrinsic(kind, mode, span)?
            }
            "debug_break" => ir::Intrinsic::DebugBreak,
            "sqrt" => ir::Intrinsic::Sqrt,
            "sin" => ir::Intrinsic::Sin,
            "cos" => ir::Intrinsic::Cos,
            "floor" => ir::Intrinsic::Floor,
            "ceil" => ir::Intrinsic::Ceil,
            "round" => ir::Intrinsic::Round,
            "trunc" => ir::Intrinsic::Trunc,
            "abs" | "fabs" => ir::Intrinsic::Fabs,
            "rdtsc" => {
                if self.options.cpu != TargetCpu::X64 {
                    return err(
                        Code::AsmRequiresX8664,
                        span,
                        "rdtsc requires an x86-64 target",
                    );
                }
                ir::Intrinsic::X86Timestamp {
                    aux: false,
                }
            }
            "get_cpu_cycle_count" => {
                if self.options.cpu == TargetCpu::Wasm {
                    return err(
                        Code::CallNoArchitecturalCycle,
                        span,
                        "no architectural cycle counter is available on a WebAssembly target",
                    );
                }
                ir::Intrinsic::CycleCounter
            }
            "pause" | "mm_pause" => ir::Intrinsic::Pause,
            "fiber_init" => ir::Intrinsic::FiberInit,
            "fiber_switch" => ir::Intrinsic::FiberSwitch,
            "fiber_free" => ir::Intrinsic::FiberFree,
            "byte_lookup" | "sign_mask" => {
                // The last parameter is the vector width in bytes; the IR op carries it.
                let width = self.const_width_arg(f, slots, &args, name.as_str(), span)?;
                values.pop();
                if name.as_str() == "byte_lookup" {
                    ir::Intrinsic::ByteLookup(width)
                } else {
                    ir::Intrinsic::SignMask(width)
                }
            }
            other => {
                return err(
                    Code::CallUnknownIntrinsic,
                    span,
                    format!("unknown intrinsic `{other}`"),
                );
            }
        };
        let returns: Vec<Ty> = sig
            .returns
            .iter()
            // An aggregate result comes back by address.
            .map(|&t| self.ir_ty(t).unwrap_or(Ty::Ptr))
            .collect();
        if matches!(
            op,
            ir::Intrinsic::Sqrt
                | ir::Intrinsic::Sin
                | ir::Intrinsic::Cos
                | ir::Intrinsic::Floor
                | ir::Intrinsic::Ceil
                | ir::Intrinsic::Round
                | ir::Intrinsic::Trunc
                | ir::Intrinsic::Fabs
        ) && (values.len() != 1
            || sig.params.len() != 1
            || sig.returns.len() != 1
            || !matches!(returns.as_slice(), [Ty::F32 | Ty::F64])
            || sig.params[0].ty != sig.returns[0])
        {
            return err(
                Code::CallFloatingIntrinsicSignature,
                span,
                "floating intrinsic signature requires one float32/float64 argument and the same result type",
            );
        }
        if matches!(
            op,
            ir::Intrinsic::X86Timestamp { .. } | ir::Intrinsic::CycleCounter
        ) && (!values.is_empty() || returns.as_slice() != [Ty::I64])
        {
            return err(
                Code::CallRequiresNoArguments,
                span,
                format!("{name} requires no arguments and one 64-bit integer result"),
            );
        }
        let atomic = matches!(
            op,
            ir::Intrinsic::AtomicLoad(_)
                | ir::Intrinsic::AtomicStore(_)
                | ir::Intrinsic::AtomicRmw(..)
                | ir::Intrinsic::AtomicCas(..)
        );
        if (op == ir::Intrinsic::CompareAndSwap && values.len() == 3) || atomic {
            // The interpreter needs the operand width: 1, 2, 4 or 8 bytes. A load has only the
            // pointer; the other operations take the value as their second parameter.
            let pointee = sig
                .params
                .first()
                .and_then(|p| self.ty.types.pointee(p.ty))
                .ok_or_else(|| {
                    Box::new(Diagnostic::new(
                        Code::CallAtomicOperation,
                        span,
                        "an atomic operation requires a pointer to an atomic scalar",
                    ))
                })?;
            let repr = self.ty.types.repr(pointee);
            let width = self.size_of(pointee, span)?;
            if !matches!(
                self.ty.types.kind(repr),
                TypeKind::Int { .. }
                    | TypeKind::Float { .. }
                    | TypeKind::Bool
                    | TypeKind::Pointer(_)
                    | TypeKind::Proc(_)
            ) || !matches!(width, 1 | 2 | 4 | 8)
            {
                return err(
                    Code::CallAtomicOperation,
                    span,
                    "an atomic scalar must be an integer, enum, bool, float, pointer or procedure of 1, 2, 4 or 8 bytes",
                );
            }
            let (arguments, expected_returns) = match op {
                ir::Intrinsic::AtomicLoad(_) => (1, vec![pointee]),
                ir::Intrinsic::AtomicStore(_) => (2, vec![]),
                ir::Intrinsic::AtomicCas(..) | ir::Intrinsic::CompareAndSwap => {
                    (3, vec![TypeId::BOOL, pointee])
                }
                _ => (2, vec![pointee]),
            };
            if values.len() != arguments
                || sig.returns != expected_returns
                || sig
                    .params
                    .iter()
                    .take(arguments)
                    .skip(1)
                    .any(|p| p.ty != pointee)
            {
                return err(
                    Code::CallAtomicOperation,
                    span,
                    "atomic intrinsic signature must preserve its pointer's scalar type and documented results",
                );
            }
            let address = f.b.conv(ir::ConvOp::Bitcast, Ty::Ptr, Ty::I64, values[0]);
            let required = f.b.iconst(Ty::I64, width);
            let mask = f.b.iconst(Ty::I64, width - 1);
            let low = f.b.bin(ir::BinOp::And, Ty::I64, address, mask);
            let zero = f.b.iconst(Ty::I64, 0);
            let bad = f.b.cmp(ir::CmpOp::Ne, Ty::I64, low, zero);
            let fail = f.b.new_block();
            let ready = f.b.new_block();
            f.b.branch(bad, fail, ready);
            f.b.switch_to(fail);
            self.emit_check_failed(
                f,
                ir::CheckReason::AtomicAlignment,
                address,
                required,
                true,
                span,
            )?;
            f.b.jump(ready);
            f.b.switch_to(ready);
            values.push(f.b.iconst(Ty::I64, width));
        }
        let results = f.b.intrinsic(op, values, &returns);
        let outs = vec![None; sig.returns.len()];
        Ok(self.call_results(f, &sig.returns, results, outs, &sig.return_callables))
    }

    /// `black_box(value)`: `value` unchanged, but unknown to the optimizer. A scalar goes through
    /// an operation that takes and returns it; an aggregate is copied to a fresh stack slot that
    /// the operation may read and write, and the slot is the result.
    fn emit_black_box(
        &mut self,
        f: &mut FnCtx,
        sig: &Signature,
        values: &[ir::Val],
        span: Span,
    ) -> Result<Operand> {
        // A `void` argument lowers to no value and no result.
        if let ([param], []) = (sig.params.as_slice(), sig.returns.as_slice())
            && self.ir_ty(param.ty).is_none()
        {
            return err(
                Code::CallBlackBox,
                span,
                "`black_box` needs a value with a runtime representation",
            );
        }
        let (&[value], &[ty], [param]) = (values, sig.returns.as_slice(), sig.params.as_slice())
        else {
            return err(
                Code::CallBlackBox,
                span,
                "`black_box` takes one value and returns it",
            );
        };
        if param.ty != ty {
            return err(
                Code::CallBlackBox,
                span,
                "`black_box` returns the type of its argument",
            );
        }
        if self.is_memory_type(ty) {
            let size = self.size_of(ty, span)?;
            let align = self.align_of(ty, span)?;
            let slot = f.b.alloca(size.max(1), align);
            f.b.copy(slot, value, size);
            f.b.intrinsic(ir::Intrinsic::BlackBoxMemory, vec![slot], &[]);
            return Ok(self.call_results(
                f,
                &sig.returns,
                Vec::new(),
                vec![Some(slot)],
                &sig.return_callables,
            ));
        }
        let Some(scalar) = self.ir_ty(ty) else {
            return err(
                Code::CallBlackBox,
                span,
                "`black_box` needs a value with a runtime representation",
            );
        };
        let results =
            f.b.intrinsic(ir::Intrinsic::BlackBox, vec![value], &[scalar]);
        Ok(self.call_results(f, &sig.returns, results, vec![None], &sig.return_callables))
    }

    /// The compile-time integer passed as the last argument of an intrinsic.
    pub(super) fn const_int_arg(
        &mut self,
        f: &mut FnCtx,
        slots: &[Slot],
        args: &[CallArg],
        name: &str,
        span: Span,
    ) -> Result<i64> {
        let bad = || format!("the memory order of `{name}` must be a compile-time constant");
        let Some(Slot::Arg(a)) = slots.last() else {
            return err(Code::CallAtomicOperation, span, bad());
        };
        let arg = &args[*a];
        let op = if let Some(op) = &arg.op {
            op.clone()
        } else {
            let expr = arg.expr.as_ref().unwrap();
            match self.check_expr_no_emit(arg.scope, expr) {
                Ok(op) => op,
                Err(_) => self.check_expr(f, arg.scope, expr, Some(TypeId::S64))?,
            }
        };
        match op {
            Operand::Const {
                value: Value::Int(v),
                ..
            } => Ok(v as i64),
            _ => err(Code::CallAtomicOperation, arg.span, bad()),
        }
    }

    /// The IR operation of an `atomic_*_order` intrinsic with memory order(s) `mode`.
    pub(super) fn atomic_intrinsic(
        &self,
        kind: OrderedAtomic,
        mode: i64,
        span: Span,
    ) -> Result<ir::Intrinsic> {
        use ir::AtomicOp as Op;
        let name = kind.name();
        let order = |v: i64| -> Result<u8> {
            match u8::try_from(v) {
                Ok(o) if o <= ir::ORDER_SEQ_CST => Ok(o),
                _ => err(
                    Code::CallAtomicOperation,
                    span,
                    format!("`{name}` was given an unknown memory order"),
                ),
            }
        };
        let cas = kind == OrderedAtomic::CompareExchange;
        if cas && !(0..=0xff).contains(&mode) {
            return err(
                Code::CallAtomicOperation,
                span,
                format!("`{name}` was given an unknown memory order"),
            );
        }
        let (first, second) = if cas {
            (order(mode & 15)?, order(mode >> 4)?)
        } else {
            (order(mode)?, ir::ORDER_RELAXED)
        };
        let released = |o: u8| o == ir::ORDER_RELEASE || o == ir::ORDER_ACQ_REL;
        Ok(match kind {
            OrderedAtomic::Load => {
                if released(first) {
                    return err(
                        Code::CallAtomicOperation,
                        span,
                        "an atomic load cannot use release or acq_rel ordering",
                    );
                }
                ir::Intrinsic::AtomicLoad(first)
            }
            OrderedAtomic::Store => {
                if first == ir::ORDER_ACQUIRE || first == ir::ORDER_ACQ_REL {
                    return err(
                        Code::CallAtomicOperation,
                        span,
                        "an atomic store cannot use acquire or acq_rel ordering",
                    );
                }
                ir::Intrinsic::AtomicStore(first)
            }
            OrderedAtomic::Exchange => ir::Intrinsic::AtomicRmw(Op::Xchg, first),
            OrderedAtomic::Add => ir::Intrinsic::AtomicRmw(Op::Add, first),
            OrderedAtomic::Sub => ir::Intrinsic::AtomicRmw(Op::Sub, first),
            OrderedAtomic::And => ir::Intrinsic::AtomicRmw(Op::And, first),
            OrderedAtomic::Or => ir::Intrinsic::AtomicRmw(Op::Or, first),
            OrderedAtomic::Xor => ir::Intrinsic::AtomicRmw(Op::Xor, first),
            OrderedAtomic::CompareExchange => {
                if released(second) {
                    return err(
                        Code::CallAtomicOperation,
                        span,
                        "the failure ordering of a compare-exchange cannot be release or acq_rel",
                    );
                }
                if !matches!((first, second), (0..=4, 0) | (1 | 3 | 4, 1) | (4, 4)) {
                    return err(
                        Code::CallAtomicOperation,
                        span,
                        "compare-exchange failure ordering cannot be stronger than success ordering",
                    );
                }
                ir::Intrinsic::AtomicCas(first, second)
            }
        })
    }

    /// The compile-time `width` argument (16, 32 or 64) of `byte_lookup`/`sign_mask`.
    pub(super) fn const_width_arg(
        &mut self,
        f: &mut FnCtx,
        slots: &[Slot],
        args: &[CallArg],
        name: &str,
        span: Span,
    ) -> Result<u8> {
        let bad =
            || format!("the `width` of `{name}` must be a compile-time constant 16, 32 or 64");
        let Some(Slot::Arg(a)) = slots.last() else {
            return err(Code::CallIntrinsicWidth, span, bad());
        };
        let arg = &args[*a];
        let op = if let Some(op) = &arg.op {
            op.clone()
        } else {
            let expr = arg.expr.as_ref().unwrap();
            match self.check_expr_no_emit(arg.scope, expr) {
                Ok(op) => op,
                Err(_) => self.check_expr(f, arg.scope, expr, Some(TypeId::S64))?,
            }
        };
        match op {
            Operand::Const {
                value: Value::Int(w @ (16 | 32 | 64)),
                ..
            } => Ok(w as u8),
            _ => err(Code::CallIntrinsicWidth, arg.span, bad()),
        }
    }
}
