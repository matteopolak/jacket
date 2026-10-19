//! Executing one op of a procedure.
use super::arith::{bin_total, cmp, conv, divides, mask, sext};
use super::frames::Frame;
use super::trap::TrapKind;
use super::{FOREIGN_TAG, FUNC_TAG, Interp, Res, TAG_MASK};
use crate::ir::{self, BinOp, Callee, ForeignId, FuncId, Inst, Program, Ty, UnOp};

impl Interp {
    pub(super) fn step(
        &mut self,
        program: &Program,
        inst: &Inst,
        vals: &mut [u64],
        frame: &Frame,
        stack_base: u64,
        value_types: &[Ty],
    ) -> Res<()> {
        match inst {
            Inst::StackAlloc {
                dst,
                size,
                align,
            } => {
                let size = vals[size.0 as usize];
                let base = self.stack.as_mut_ptr() as u64;
                let start = base
                    .checked_add(self.sp)
                    .and_then(|v| v.checked_add(align - 1))
                    .map(|v| (v & !(align - 1)) - base);
                let end = start.and_then(|v| v.checked_add(size.max(1)));
                let Some((start, end)) = start
                    .zip(end)
                    .filter(|(_, end)| *end <= self.stack.len() as u64 * 8)
                else {
                    return self.trap_of(
                        TrapKind::StackOverflow,
                        "dynamic interpreter stack allocation exceeds the frame stack",
                    );
                };
                self.sp = end;
                vals[dst.0 as usize] = base + start;
            }
            Inst::IConst {
                dst,
                ty,
                value,
            } => vals[dst.0 as usize] = mask(*ty, *value),
            Inst::FConst {
                dst,
                ty,
                value,
            } => {
                vals[dst.0 as usize] = if *ty == Ty::F32 {
                    (*value as f32).to_bits() as u64
                } else {
                    value.to_bits()
                };
            }
            Inst::Bin {
                dst,
                op,
                ty,
                a,
                b,
            } => {
                let (x, y) = (vals[a.0 as usize], vals[b.0 as usize]);
                vals[dst.0 as usize] = self.bin(*op, *ty, x, y)?;
            }
            Inst::Un {
                dst,
                op,
                ty,
                a,
            } => {
                let x = vals[a.0 as usize];
                vals[dst.0 as usize] = match op {
                    UnOp::Neg => mask(*ty, x.wrapping_neg()),
                    UnOp::Not => mask(*ty, !x),
                    UnOp::FNeg => {
                        if *ty == Ty::F32 {
                            (-f32::from_bits(x as u32)).to_bits() as u64
                        } else {
                            (-f64::from_bits(x)).to_bits()
                        }
                    }
                };
            }
            Inst::Cmp {
                dst,
                op,
                ty,
                a,
                b,
            } => {
                let (x, y) = (vals[a.0 as usize], vals[b.0 as usize]);
                vals[dst.0 as usize] = cmp(*op, *ty, x, y) as u64;
            }
            Inst::Conv {
                dst,
                op,
                from,
                to,
                src,
            } => vals[dst.0 as usize] = conv(*op, *from, *to, vals[src.0 as usize]),
            Inst::SlotAddr {
                dst,
                slot,
            } => vals[dst.0 as usize] = stack_base + frame.offsets[slot.0 as usize],
            Inst::GlobalAddr {
                dst,
                global,
            } => vals[dst.0 as usize] = self.global_addr(program, *global)?,
            Inst::FuncAddr {
                dst,
                func,
            } => vals[dst.0 as usize] = self.proc_value(program, *func),
            Inst::ForeignAddr {
                dst,
                foreign,
            } => vals[dst.0 as usize] = self.foreign_addr(program, *foreign)?,
            Inst::Load {
                dst,
                ty,
                addr,
            } => vals[dst.0 as usize] = self.load(*ty, vals[addr.0 as usize])?,
            Inst::Store {
                ty,
                addr,
                value,
            } => self.store(*ty, vals[addr.0 as usize], vals[value.0 as usize])?,
            Inst::PtrAdd {
                dst,
                base,
                offset,
            } => vals[dst.0 as usize] = vals[base.0 as usize].wrapping_add(vals[offset.0 as usize]),
            Inst::Copy {
                dst,
                src,
                size,
            } => {
                let (d, s) = (vals[dst.0 as usize], vals[src.0 as usize]);
                if d < 4096 || s < 4096 {
                    return self.null_trap("memory copy through a null pointer");
                }
                self.check_writable(d, *size as usize)?;
                // SAFETY: operands are host addresses of live program memory, which `ptr::copy` may
                // overlap; the null case was trapped above.
                unsafe { std::ptr::copy(s as *const u8, d as *mut u8, *size as usize) };
            }
            Inst::Zero {
                dst,
                size,
            } => {
                let d = vals[dst.0 as usize];
                if d < 4096 {
                    return self.null_trap("memory fill through a null pointer");
                }
                self.check_writable(d, *size as usize)?;
                // SAFETY: `d` is non-null (trapped above) and addresses `size` writable bytes of
                // program memory.
                unsafe { std::ptr::write_bytes(d as *mut u8, 0, *size as usize) };
            }
            Inst::Call(call) => {
                let ir::CallInst {
                    results,
                    callee,
                    args,
                } = &**call;
                let (mut small, mut heap) = ([0u64; 8], Vec::new());
                let argv = gather(vals, args, &mut small, &mut heap);
                let out = match callee {
                    Callee::Func(f) => self.exec(program, *f, argv)?,
                    Callee::Foreign(f) => {
                        let sig = &program.foreigns[f.0 as usize].sig;
                        self.call_foreign(program, *f, argv, sig)?
                    }
                    Callee::Indirect(target, sig) => {
                        let addr = vals[target.0 as usize];
                        match addr & TAG_MASK {
                            FUNC_TAG => {
                                self.exec(program, FuncId((addr & !TAG_MASK) as u32), argv)?
                            }
                            FOREIGN_TAG => self.call_foreign(
                                program,
                                ForeignId((addr & !TAG_MASK) as u32),
                                argv,
                                sig,
                            )?,
                            _ if addr < 4096 => {
                                return self.null_trap("call through a null procedure pointer");
                            }
                            _ => match self.thunk_funcs.get(&addr) {
                                // A thunk this interpreter made: no need to go through C.
                                Some(&f) => self.exec(program, f, argv)?,
                                None => self.call_native(program, addr, argv, sig, None, true)?,
                            },
                        }
                    }
                };
                for (r, &v) in results.iter().zip(out.iter()) {
                    vals[r.0 as usize] = v;
                }
            }
            Inst::Intrinsic(call) => {
                let ir::IntrinsicInst {
                    results,
                    op,
                    args,
                } = &**call;
                let (mut small, mut heap) = ([0u64; 8], Vec::new());
                let argv = gather(vals, args, &mut small, &mut heap);
                if *op == ir::Intrinsic::CheckFailed {
                    return self.check_failed(program, argv);
                }
                if matches!(
                    op,
                    ir::Intrinsic::FiberInit
                        | ir::Intrinsic::FiberSwitch
                        | ir::Intrinsic::FiberFree
                ) {
                    return self.fiber_intrinsic(program, *op, argv);
                }
                let result_type = results.first().map(|r| value_types[r.0 as usize]);
                let out = self.intrinsic(*op, argv, result_type)?;
                for (r, &v) in results.iter().zip(out.iter()) {
                    vals[r.0 as usize] = v;
                }
            }
            Inst::Loc {
                line,
                col,
                file,
                ..
            } => self.loc = Some((*file, *line, *col)),
        }
        Ok(())
    }

    fn bin(&self, op: BinOp, ty: Ty, x: u64, y: u64) -> Res<u64> {
        if divides(op) {
            self.divide(op, ty, x, y)
        } else {
            Ok(bin_total(op, ty, x, y))
        }
    }

    /// `SDiv`, `SRem`, `UDiv` or `URem`: the binary operations that can fail.
    pub(super) fn divide(&self, op: BinOp, ty: Ty, x: u64, y: u64) -> Res<u64> {
        Ok(match op {
            BinOp::SDiv | BinOp::SRem => {
                let (a, b) = (sext(ty, x), sext(ty, y));
                if b == 0 {
                    return self.check_trap(ir::CheckReason::DivideByZero, 0, 0);
                }
                let r = if op == BinOp::SDiv {
                    a.wrapping_div(b)
                } else {
                    a.wrapping_rem(b)
                };
                mask(ty, r as u64)
            }
            _ => {
                if y == 0 {
                    return self.check_trap(ir::CheckReason::DivideByZero, 0, 0);
                }
                if op == BinOp::UDiv {
                    x / y
                } else {
                    x % y
                }
            }
        })
    }

    /// `Intrinsic::CheckFailed` (reason, a, b, fatal): a trap, or a warning with the location
    /// on stderr when the check is not fatal.
    fn check_failed(&mut self, program: &Program, a: &[u64]) -> Res<()> {
        if a[3] & 0xff != 0 {
            return self.check_trap(ir::CheckReason::from_raw(a[0]), a[1], a[2]);
        }
        let message = ir::check_message(ir::CheckReason::from_raw(a[0]), a[1], a[2]);
        let at = self
            .loc
            .and_then(|(file, line, _)| {
                let path = program.file_paths.get(file as usize)?;
                Some(format!(
                    "{}:{line}: ",
                    crate::display_path(std::path::Path::new(path))
                ))
            })
            .unwrap_or_default();
        self.effects += 1;
        self.host
            .write(format!("{at}warning: {message}\n").as_bytes(), true);
        Ok(())
    }
}

/// Reads an instruction's operands. Most fit in `small` on the Rust stack; longer lists use `heap`.
pub(super) fn gather<'a>(
    vals: &[u64],
    args: &[ir::Val],
    small: &'a mut [u64; 8],
    heap: &'a mut Vec<u64>,
) -> &'a [u64] {
    if args.len() <= small.len() {
        for (slot, a) in small.iter_mut().zip(args) {
            *slot = vals[a.0 as usize];
        }
        &small[..args.len()]
    } else {
        *heap = args.iter().map(|a| vals[a.0 as usize]).collect();
        heap
    }
}
