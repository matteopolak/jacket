//! Dispatch of an `#asm` instruction to its implementation and operand sizing.
use super::helpers::{
    bin, bit_intrinsic, bits, cmp, flag_not, is_zero, konst, resize, select, wide_mul,
};
use super::model::{Alu, AsmArgs, AsmCtx, Flags, Op, Opd, lookup_op};
use crate::ast::{AsmInst, AsmSize};
use crate::ir::{BinOp, CmpOp, Intrinsic, Ty, UnOp};
use crate::sema::lower::{FnCtx, Operand};
use crate::sema::{Compiler, Result, Span, TargetCpu, Value, err};

impl Compiler {
    pub(super) fn asm_inst(
        &mut self,
        f: &mut FnCtx,
        cx: &mut AsmCtx,
        inst: &AsmInst,
    ) -> Result<()> {
        let name = inst.mnemonic.name.as_str();
        let (lock, base) = match name.strip_prefix("lock_") {
            Some(rest) => (true, rest),
            None => (false, name),
        };
        let span = inst.span;
        let Some(op) = lookup_op(base) else {
            if self.asm_scalar_inst(f, cx, inst, base, lock)? {
                return Ok(());
            }
            if !lock
                && (self.asm_mask_inst(f, cx, inst, base)?
                    || self.asm_vec_inst_floats(f, cx, inst, base)?)
            {
                return Ok(());
            }
            return err(
                inst.mnemonic.span,
                format!("unsupported #asm instruction `{name}`"),
            );
        };
        let mut opds = Vec::with_capacity(inst.operands.len());
        for o in &inst.operands {
            opds.push(self.asm_operand(f, cx, o)?);
        }
        let c = AsmArgs {
            name,
            base,
            lock,
            span,
            opds,
        };
        match op {
            Op::Nop
            | Op::Pause
            | Op::Fence(..)
            | Op::Int3
            | Op::SetCarry(..)
            | Op::Rdtsc
            | Op::Rdtscp
            | Op::Rdrand
            | Op::Rdseed
            | Op::Cpuid
            | Op::Syscall => self.asm_system(f, cx, inst, &c, op)?,
            Op::Mov
            | Op::Movbe
            | Op::Movzx
            | Op::Movsx
            | Op::Lea
            | Op::Xchg
            | Op::Xadd
            | Op::Cmpxchg
            | Op::Setcc(..)
            | Op::Cmovcc(..) => self.asm_data_move(f, cx, inst, &c, op)?,
            Op::Alu(..)
            | Op::Inc
            | Op::Dec
            | Op::Neg
            | Op::Not
            | Op::Shift(..)
            | Op::Bt(..)
            | Op::Imul
            | Op::Mul => self.asm_arith(f, cx, inst, &c, op)?,
            Op::Bsf
            | Op::Bsr
            | Op::Popcnt
            | Op::Lzcnt
            | Op::Tzcnt
            | Op::Bswap
            | Op::Blsr
            | Op::Blsi
            | Op::Blsmsk => self.asm_bits(f, cx, inst, &c, op)?,
        }
        Ok(())
    }

    fn asm_system(
        &mut self,
        f: &mut FnCtx,
        cx: &mut AsmCtx,
        inst: &AsmInst,
        c: &AsmArgs,
        op: Op,
    ) -> Result<()> {
        let span = c.span;
        let name = c.name;
        let opds = &c.opds;
        let n = c.opds.len();
        let want = |lo: usize, hi: usize| c.want(lo, hi);
        match op {
            Op::Nop => {
                want(0, 0)?;
            }
            Op::Pause | Op::Fence(_) => {
                want(0, 0)?;
                if self.options.cpu != TargetCpu::X64 {
                    return err(span, format!("{name} requires an x86-64 target"));
                }
                let operation = match op {
                    Op::Fence(kind) => Intrinsic::X86Fence(kind),
                    _ => Intrinsic::X86Pause,
                };
                f.b.intrinsic(operation, Vec::new(), &[]);
            }
            Op::Int3 => {
                want(0, 0)?;
                f.b.intrinsic(Intrinsic::DebugBreak, Vec::new(), &[]);
            }
            Op::SetCarry(value) => {
                want(0, 0)?;
                cx.flags.cf = Some(if let Some(v) = value {
                    konst(f, Ty::I8, v as u64)
                } else {
                    let cf = Self::flag(f, cx.flags.cf);
                    flag_not(f, cf)
                });
            }
            Op::Rdtsc | Op::Rdtscp => {
                let max = if op == Op::Rdtscp {
                    3
                } else {
                    2
                };
                want(2, max)?;
                if self.options.cpu != TargetCpu::X64 {
                    return err(span, format!("{name} requires an x86-64 target"));
                }
                if opds
                    .iter()
                    .any(|o| !matches!(o, Opd::Reg(r) if matches!(r.storage, Ty::I32 | Ty::I64)))
                {
                    return err(
                        span,
                        format!("{name} outputs must be 32- or 64-bit integer registers"),
                    );
                }
                let aux = op == Op::Rdtscp;
                let result_types: &[Ty] = if aux {
                    &[Ty::I64, Ty::I32]
                } else {
                    &[Ty::I64]
                };
                let results = f.b.intrinsic(
                    Intrinsic::X86Timestamp {
                        aux,
                    },
                    Vec::new(),
                    result_types,
                );
                let t = results[0];
                let s32 = konst(f, Ty::I64, 32);
                let hi = bin(f, BinOp::LShr, Ty::I64, t, s32);
                let hi = resize(f, hi, Ty::I64, Ty::I32, false);
                let lo = resize(f, t, Ty::I64, Ty::I32, false);
                self.asm_write(f, opds[0], Ty::I32, hi, span)?;
                self.asm_write(f, opds[1], Ty::I32, lo, span)?;
                if n == 3 {
                    self.asm_write(f, opds[2], Ty::I32, results[1], span)?;
                }
            }
            Op::Rdrand | Op::Rdseed => {
                want(1, 1)?;
                if self.options.cpu != TargetCpu::X64 {
                    return err(span, format!("{name} requires an x86-64 target"));
                }
                let sz = self.asm_size(f, cx, inst, opds)?;
                if !matches!(sz, Ty::I16 | Ty::I32 | Ty::I64)
                    || !matches!(opds[0], Opd::Reg(r) if matches!(r.storage, Ty::I16 | Ty::I32 | Ty::I64) && r.storage.size() >= sz.size())
                {
                    return err(
                        span,
                        format!(
                            "{name} requires a 16-, 32- or 64-bit integer register destination"
                        ),
                    );
                }
                let result = f.b.intrinsic(
                    Intrinsic::X86Random {
                        seed: op == Op::Rdseed,
                        width: sz,
                    },
                    Vec::new(),
                    &[sz, Ty::I8],
                );
                self.asm_write(f, opds[0], sz, result[0], span)?;
                let zero = konst(f, Ty::I8, 0);
                cx.flags = Flags {
                    cf: Some(result[1]),
                    zf: Some(zero),
                    sf: Some(zero),
                    of: Some(zero),
                    // This field encodes parity through a result byte, not the
                    // flag bit. Encode the architecturally cleared bit explicitly.
                    pf: Some(Self::parity_byte_for_bit(f, zero)),
                };
            }
            Op::Cpuid => {
                want(4, 4)?;
                if self.options.cpu != TargetCpu::X64 {
                    return err(span, "cpuid requires an x86-64 target");
                }
                if opds
                    .iter()
                    .any(|o| !matches!(o, Opd::Reg(r) if matches!(r.storage,Ty::I32|Ty::I64)))
                {
                    return err(
                        span,
                        "cpuid operands must be 32- or 64-bit integer registers",
                    );
                }
                let leaf = self.asm_read(f, opds[0], Ty::I32, span)?;
                let subleaf = self.asm_read(f, opds[2], Ty::I32, span)?;
                let results =
                    f.b.intrinsic(Intrinsic::CpuId, vec![leaf, subleaf], &[Ty::I32; 4]);
                for (&o, value) in opds.iter().zip(results) {
                    self.asm_write(f, o, Ty::I32, value, span)?;
                }
            }
            Op::Syscall => {
                // `syscall rcx_out, r11_out, rax_inout`: rax is the number in and the raw result
                // out; rdi, rsi, rdx, r10, r8 and r9 are the pinned registers; rcx and r11 are
                // clobbered (they read 0 afterwards).
                want(3, 3)?;
                let mut args = vec![self.asm_read(f, opds[2], Ty::I64, span)?];
                for reg in [7u8, 6, 2, 10, 8, 9] {
                    let pinned = self.asm_pins.iter().rev().find(|(r, _)| *r == reg);
                    args.push(match pinned {
                        Some(&(_, addr)) => f.b.load(Ty::I64, addr),
                        None => konst(f, Ty::I64, 0),
                    });
                }
                let ret = f.b.intrinsic(Intrinsic::Syscall, args, &[Ty::I64])[0];
                for &o in &opds[..2] {
                    let z = konst(f, Ty::I64, 0);
                    self.asm_write(f, o, Ty::I64, z, span)?;
                }
                self.asm_write(f, opds[2], Ty::I64, ret, span)?;
                if let Some(&(_, addr)) = self.asm_pins.iter().rev().find(|(r, _)| *r == 0) {
                    f.b.store(Ty::I64, addr, ret);
                }
            }
            _ => unreachable!("dispatched only for this group's operations"),
        }
        Ok(())
    }

    fn asm_data_move(
        &mut self,
        f: &mut FnCtx,
        cx: &mut AsmCtx,
        inst: &AsmInst,
        c: &AsmArgs,
        op: Op,
    ) -> Result<()> {
        let span = c.span;
        let opds = &c.opds;
        let base = c.base;
        let lock = c.lock;
        let want = |lo: usize, hi: usize| c.want(lo, hi);
        match op {
            Op::Mov => {
                want(2, 2)?;
                let sz = self.asm_size(f, cx, inst, opds)?;
                let v = self.asm_read(f, opds[1], sz, span)?;
                self.asm_write(f, opds[0], sz, v, span)?;
            }
            Op::Movbe => {
                want(2, 2)?;
                let sz = self.asm_size(f, cx, inst, opds)?;
                let v = self.asm_read(f, opds[1], sz, span)?;
                let v = if sz == Ty::I8 {
                    v
                } else {
                    bit_intrinsic(f, Intrinsic::Bswap, sz, v)
                };
                self.asm_write(f, opds[0], sz, v, span)?;
            }
            Op::Movzx | Op::Movsx => {
                want(2, 2)?;
                let (from, to) = Self::asm_extension_sizes(base, inst, span)?;
                let v = self.asm_read(f, opds[1], from, span)?;
                let v = resize(f, v, from, to, op == Op::Movsx);
                self.asm_write(f, opds[0], to, v, span)?;
            }
            Op::Lea => {
                want(2, 2)?;
                let Opd::Mem(addr) = opds[1] else {
                    return err(
                        span,
                        "#asm lea source must be a bracketed address expression",
                    );
                };
                let sz = self.asm_size(f, cx, inst, &opds[..1])?;
                let v = resize(f, addr, Ty::I64, sz, false);
                self.asm_write(f, opds[0], sz, v, span)?;
            }
            Op::Xchg => {
                want(2, 2)?;
                let sz = self.asm_size(f, cx, inst, opds)?;
                // Exchanging with memory is always atomic.
                let (target, other) = if matches!(opds[1], Opd::Mem(_)) {
                    (opds[1], opds[0])
                } else {
                    (opds[0], opds[1])
                };
                let new = self.asm_read(f, other, sz, span)?;
                let (rmw, old) = self.rmw_begin(f, target, sz, true, span)?;
                self.rmw_end(f, rmw, new, span)?;
                self.asm_write(f, other, sz, old, span)?;
            }
            Op::Xadd => {
                want(2, 2)?;
                let sz = self.asm_size(f, cx, inst, opds)?;
                let addend = self.asm_read(f, opds[1], sz, span)?;
                let (rmw, old) = self.rmw_begin(f, opds[0], sz, lock, span)?;
                let new = self.asm_alu(f, cx, Alu::Add, sz, old, addend);
                self.rmw_end(f, rmw, new, span)?;
                self.asm_write(f, opds[1], sz, old, span)?;
            }
            Op::Cmpxchg => {
                // `cmpxchg dest, src, acc` or `cmpxchg acc, dest, src` (the memory
                // operand is the destination). The accumulator is explicit because
                // registers are not pinned.
                want(3, 3)?;
                let (dst, src, acc) =
                    if matches!(opds[0], Opd::Mem(_)) || !matches!(opds[1], Opd::Mem(_)) {
                        (opds[0], opds[1], opds[2])
                    } else {
                        (opds[1], opds[2], opds[0])
                    };
                let sz = self.asm_size(f, cx, inst, opds)?;
                let src_v = self.asm_read(f, src, sz, span)?;
                let acc_v = self.asm_read(f, acc, sz, span)?;
                let (rmw, old) = self.rmw_begin(f, dst, sz, lock, span)?;
                let eq = cmp(f, CmpOp::Eq, sz, old, acc_v);
                let new = select(f, sz, eq, src_v, old);
                self.asm_alu(f, cx, Alu::Cmp, sz, acc_v, old);
                self.rmw_end(f, rmw, new, span)?;
                let acc_new = select(f, sz, eq, acc_v, old);
                self.asm_write(f, acc, sz, acc_new, span)?;
            }
            Op::Setcc(cond) => {
                want(1, 1)?;
                let v = Self::eval_cond(f, cx, cond);
                self.asm_write(f, opds[0], Ty::I8, v, span)?;
            }
            Op::Cmovcc(cond) => {
                want(2, 2)?;
                let sz = self.asm_size(f, cx, inst, opds)?;
                let c = Self::eval_cond(f, cx, cond);
                let src = self.asm_read(f, opds[1], sz, span)?;
                let old = self.asm_read(f, opds[0], sz, span)?;
                let v = select(f, sz, c, src, old);
                self.asm_write(f, opds[0], sz, v, span)?;
            }
            _ => unreachable!("dispatched only for this group's operations"),
        }
        Ok(())
    }

    fn asm_arith(
        &mut self,
        f: &mut FnCtx,
        cx: &mut AsmCtx,
        inst: &AsmInst,
        c: &AsmArgs,
        op: Op,
    ) -> Result<()> {
        let span = c.span;
        let opds = &c.opds;
        let n = c.opds.len();
        let lock = c.lock;
        let want = |lo: usize, hi: usize| c.want(lo, hi);
        match op {
            Op::Alu(alu) => {
                want(2, 2)?;
                let sz = self.asm_size(f, cx, inst, opds)?;
                let b = self.asm_read(f, opds[1], sz, span)?;
                if matches!(alu, Alu::Cmp | Alu::Test) {
                    let a = self.asm_read(f, opds[0], sz, span)?;
                    self.asm_alu(f, cx, alu, sz, a, b);
                } else {
                    let (rmw, a) = self.rmw_begin(f, opds[0], sz, lock, span)?;
                    let res = self.asm_alu(f, cx, alu, sz, a, b);
                    self.rmw_end(f, rmw, res, span)?;
                }
            }
            Op::Inc | Op::Dec | Op::Neg | Op::Not => {
                want(1, 1)?;
                let sz = self.asm_size(f, cx, inst, opds)?;
                let (rmw, a) = self.rmw_begin(f, opds[0], sz, lock, span)?;
                let res = match op {
                    Op::Not => f.b.un(UnOp::Not, sz, a),
                    Op::Neg => {
                        let zero = konst(f, sz, 0);
                        self.asm_alu(f, cx, Alu::Sub, sz, zero, a)
                    }
                    _ => {
                        // inc/dec leave CF alone.
                        let one = konst(f, sz, 1);
                        let carry = cx.flags.cf;
                        let alu = if op == Op::Inc {
                            Alu::Add
                        } else {
                            Alu::Sub
                        };
                        let res = self.asm_alu(f, cx, alu, sz, a, one);
                        cx.flags.cf = carry;
                        res
                    }
                };
                self.rmw_end(f, rmw, res, span)?;
            }
            Op::Shift(kind) => {
                want(1, 2)?;
                let sz = self.asm_size(f, cx, inst, &opds[..1])?;
                let count = if n == 2 {
                    Some(opds[1])
                } else {
                    None
                };
                self.asm_shift(f, cx, kind, sz, opds[0], count, lock, span)?;
            }
            Op::Bt(kind) => {
                want(2, 2)?;
                let sz = self.asm_size(f, cx, inst, &opds[..1])?;
                self.asm_bit_test(f, cx, kind, sz, opds[0], opds[1], lock, span)?;
            }
            Op::Imul | Op::Mul => {
                want(
                    if op == Op::Mul {
                        3
                    } else {
                        2
                    },
                    3,
                )?;
                let sz = self.asm_size(f, cx, inst, opds)?;
                let signed = op == Op::Imul;
                if n == 2 {
                    // imul dst, src
                    let a = self.asm_read(f, opds[0], sz, span)?;
                    let b = self.asm_read(f, opds[1], sz, span)?;
                    let (hi, lo) = wide_mul(f, sz, a, b, true);
                    self.asm_mul_flags(f, cx, sz, hi, lo, true);
                    self.asm_write(f, opds[0], sz, lo, span)?;
                } else if signed && matches!(opds[2], Opd::Imm(_)) {
                    // imul dst, src, imm
                    let a = self.asm_read(f, opds[1], sz, span)?;
                    let b = self.asm_read(f, opds[2], sz, span)?;
                    let (hi, lo) = wide_mul(f, sz, a, b, true);
                    self.asm_mul_flags(f, cx, sz, hi, lo, true);
                    self.asm_write(f, opds[0], sz, lo, span)?;
                } else {
                    // mul/imul high, low, src: high:low = low * src
                    let a = self.asm_read(f, opds[1], sz, span)?;
                    let b = self.asm_read(f, opds[2], sz, span)?;
                    let (hi, lo) = wide_mul(f, sz, a, b, signed);
                    self.asm_mul_flags(f, cx, sz, hi, lo, signed);
                    self.asm_write(f, opds[1], sz, lo, span)?;
                    self.asm_write(f, opds[0], sz, hi, span)?;
                }
            }
            _ => unreachable!("dispatched only for this group's operations"),
        }
        Ok(())
    }

    fn asm_bits(
        &mut self,
        f: &mut FnCtx,
        cx: &mut AsmCtx,
        inst: &AsmInst,
        c: &AsmArgs,
        op: Op,
    ) -> Result<()> {
        let span = c.span;
        let opds = &c.opds;
        let want = |lo: usize, hi: usize| c.want(lo, hi);
        match op {
            Op::Bsf | Op::Bsr | Op::Popcnt | Op::Lzcnt | Op::Tzcnt => {
                want(2, 2)?;
                let sz = self.asm_size(f, cx, inst, opds)?;
                let src = self.asm_read(f, opds[1], sz, span)?;
                let src_zero = is_zero(f, sz, src);
                let zero8 = konst(f, Ty::I8, 0);
                let res = match op {
                    Op::Popcnt => {
                        let odd = konst(f, Ty::I8, 1);
                        cx.flags = Flags {
                            cf: Some(zero8),
                            zf: Some(src_zero),
                            sf: Some(zero8),
                            of: Some(zero8),
                            pf: Some(odd),
                        };
                        bit_intrinsic(f, Intrinsic::Popcount, sz, src)
                    }
                    Op::Lzcnt | Op::Tzcnt => {
                        let res = if op == Op::Lzcnt {
                            bit_intrinsic(f, Intrinsic::Ctlz, sz, src)
                        } else {
                            bit_intrinsic(f, Intrinsic::Cttz, sz, src)
                        };
                        cx.flags.cf = Some(src_zero);
                        cx.flags.zf = Some(is_zero(f, sz, res));
                        res
                    }
                    _ => {
                        // bsf/bsr: the destination is unchanged for a zero source.
                        let found = if op == Op::Bsf {
                            bit_intrinsic(f, Intrinsic::Cttz, sz, src)
                        } else {
                            let lz = bit_intrinsic(f, Intrinsic::Ctlz, sz, src);
                            let top = konst(f, sz, bits(sz) - 1);
                            bin(f, BinOp::Sub, sz, top, lz)
                        };
                        let old = self.asm_read(f, opds[0], sz, span)?;
                        cx.flags.zf = Some(src_zero);
                        select(f, sz, src_zero, old, found)
                    }
                };
                self.asm_write(f, opds[0], sz, res, span)?;
            }
            Op::Bswap => {
                want(1, 1)?;
                let sz = self.asm_size(f, cx, inst, opds)?;
                let v = self.asm_read(f, opds[0], sz, span)?;
                let v = if sz == Ty::I8 {
                    v
                } else {
                    bit_intrinsic(f, Intrinsic::Bswap, sz, v)
                };
                self.asm_write(f, opds[0], sz, v, span)?;
            }
            Op::Blsr | Op::Blsi | Op::Blsmsk => {
                want(2, 2)?;
                let sz = self.asm_size(f, cx, inst, opds)?;
                let a = self.asm_read(f, opds[1], sz, span)?;
                let one = konst(f, sz, 1);
                let zero = konst(f, sz, 0);
                let (res, cf) = match op {
                    Op::Blsr => {
                        let m = bin(f, BinOp::Sub, sz, a, one);
                        (bin(f, BinOp::And, sz, a, m), is_zero(f, sz, a))
                    }
                    Op::Blsi => {
                        let m = bin(f, BinOp::Sub, sz, zero, a);
                        let nz = is_zero(f, sz, a);
                        (bin(f, BinOp::And, sz, a, m), flag_not(f, nz))
                    }
                    _ => {
                        let m = bin(f, BinOp::Sub, sz, a, one);
                        (bin(f, BinOp::Xor, sz, a, m), is_zero(f, sz, a))
                    }
                };
                Self::set_logic_flags(f, cx, sz, res);
                cx.flags.cf = Some(cf);
                if op == Op::Blsmsk {
                    cx.flags.zf = Some(konst(f, Ty::I8, 0));
                }
                self.asm_write(f, opds[0], sz, res, span)?;
            }
            _ => unreachable!("dispatched only for this group's operations"),
        }
        Ok(())
    }

    /// Sizes of `movzx`/`movsx`: the mnemonic carries `<src><dst>` letters (`movzxbw`).
    fn asm_extension_sizes(base: &str, inst: &AsmInst, span: Span) -> Result<(Ty, Ty)> {
        let letters = base
            .strip_prefix("movzx")
            .or_else(|| base.strip_prefix("movsx"))
            .unwrap_or("");
        let size_of_letter = |c: char| match c {
            'b' => Some(Ty::I8),
            'w' => Some(Ty::I16),
            'd' => Some(Ty::I32),
            'q' => Some(Ty::I64),
            _ => None,
        };
        let chars: Vec<char> = letters.chars().collect();
        match (base, chars.as_slice()) {
            ("movsxd", []) => Ok((Ty::I32, Ty::I64)),
            (_, [s, d]) => match (size_of_letter(*s), size_of_letter(*d)) {
                (Some(s), Some(d)) if s.size() < d.size() => Ok((s, d)),
                _ => err(
                    span,
                    format!("invalid operand sizes in `{}`", inst.mnemonic.name),
                ),
            },
            _ => err(
                span,
                format!(
                    "`{}` needs source and destination sizes (for example movzxbw)",
                    inst.mnemonic.name
                ),
            ),
        }
    }

    /// Instruction operand size: the explicit suffix, else the size of the first Jai
    /// variable operand, else 64 bits.
    pub(super) fn asm_size(
        &mut self,
        f: &mut FnCtx,
        cx: &AsmCtx,
        inst: &AsmInst,
        opds: &[Opd],
    ) -> Result<Ty> {
        match &inst.size {
            Some(AsmSize::Suffix(s)) => match s.name.as_str() {
                "b" | "8" => Ok(Ty::I8),
                "w" | "16" => Ok(Ty::I16),
                "d" | "32" => Ok(Ty::I32),
                "q" | "64" => Ok(Ty::I64),
                other => err(
                    s.span,
                    format!(
                        "unsupported #asm operand size `.{other}`: an integer instruction takes \
                         `.b/.w/.d/.q` (`.x/.y/.z` are vector sizes)"
                    ),
                ),
            },
            Some(AsmSize::Dynamic(e)) => {
                let bits = match self.check_expr(f, cx.scope, e, None)? {
                    Operand::Type(t) => self.size_of(t, e.span)? * 8,
                    Operand::Const {
                        value: Value::Int(v),
                        ..
                    } => v as u64,
                    _ => {
                        return err(
                            e.span,
                            "the size after `?` must be a type or a number of bits",
                        );
                    }
                };
                match bits {
                    8 => Ok(Ty::I8),
                    16 => Ok(Ty::I16),
                    32 => Ok(Ty::I32),
                    64 => Ok(Ty::I64),
                    _ => err(
                        e.span,
                        format!("no #asm instruction form with a {bits}-bit operand"),
                    ),
                }
            }
            None => {
                for o in opds {
                    if let Opd::Reg(r) = o
                        && r.natural
                    {
                        return Ok(Ty::int(r.storage.size()));
                    }
                }
                Ok(Ty::I64)
            }
        }
    }
}
