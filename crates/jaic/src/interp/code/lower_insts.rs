//! Lowering of straight-line instructions: calls, arithmetic, loads, stores and address math.
use super::super::{divides, mask, shift_of};
use super::build::{Address, Builder, Known, Offset, imm};
use super::op::{NO_LOC, Op};
use crate::ir::{self, BinOp, ConvOp, Inst, Ty, Val, inst_vals};

impl Builder<'_> {
    pub(super) fn inst(&mut self, bi: usize, ii: usize, inst: &Inst) {
        match *inst {
            Inst::StackAlloc {
                ..
            }
            | Inst::SlotAddr {
                ..
            }
            | Inst::GlobalAddr {
                ..
            }
            | Inst::ForeignAddr {
                ..
            }
            | Inst::Load {
                ..
            }
            | Inst::Store {
                ..
            }
            | Inst::PtrAdd {
                ..
            }
            | Inst::Copy {
                ..
            }
            | Inst::Zero {
                ..
            } => self.inst_memory(bi, ii, inst),
            Inst::IConst {
                ..
            }
            | Inst::FConst {
                ..
            }
            | Inst::FuncAddr {
                ..
            }
            | Inst::Bin {
                ..
            }
            | Inst::Un {
                ..
            }
            | Inst::Cmp {
                ..
            }
            | Inst::Conv {
                ..
            } => self.inst_values(bi, ii, inst),
            Inst::Loc {
                ..
            }
            | Inst::Call(..)
            | Inst::Intrinsic(..) => self.inst_calls(bi, ii, inst),
        }
    }

    pub(super) fn inst_memory(&mut self, bi: usize, ii: usize, inst: &Inst) {
        let slow = Op::Ir {
            block: bi as u32,
            inst: ii as u32,
            loc: NO_LOC,
        };
        match *inst {
            Inst::StackAlloc {
                dst,
                size,
                ..
            } => {
                self.v(size);
                self.v(dst);
                self.push(slow, None);
            }
            Inst::SlotAddr {
                dst, ..
            } => {
                let Known::Frame(off) = self.known[dst.0 as usize] else {
                    return self.push(slow, None);
                };
                self.push(
                    Op::FrameAddr {
                        dst: self.v(dst),
                        off,
                    },
                    Some(dst),
                );
            }
            Inst::GlobalAddr {
                dst,
                global,
            } => {
                let op = Op::GlobalAddr {
                    dst: self.v(dst),
                    global,
                    off: 0,
                };
                self.push(op, self.single(dst).then_some(dst));
            }
            Inst::ForeignAddr {
                dst,
                foreign,
            } => {
                let op = Op::ForeignAddr {
                    dst: self.v(dst),
                    foreign,
                };
                self.push(op, None);
            }
            Inst::Load {
                dst,
                ty,
                addr,
            } => {
                let dst = self.v(dst);
                let op = match self.address(addr) {
                    Address::Frame(off) => Op::LoadFrame {
                        ty,
                        dst,
                        off,
                    },
                    Address::Reg(base, off) => Op::Load {
                        ty,
                        dst,
                        base,
                        off,
                    },
                };
                self.push(op, None);
            }
            Inst::Store {
                ty,
                addr,
                value,
            } => {
                if let Some(x) = self.int(value)
                    && let Some(imm) = imm(x, ty)
                {
                    self.fold(value);
                    let op = match self.address(addr) {
                        Address::Frame(off) => Op::StoreFrameImm {
                            ty,
                            off,
                            imm,
                        },
                        Address::Reg(base, off) => Op::StoreImm {
                            ty,
                            base,
                            off,
                            imm,
                        },
                    };
                    return self.push(op, None);
                }
                let value = self.v(value);
                let op = match self.address(addr) {
                    Address::Frame(off) => Op::StoreFrame {
                        ty,
                        off,
                        value,
                    },
                    Address::Reg(base, off) => Op::Store {
                        ty,
                        base,
                        off,
                        value,
                    },
                };
                self.push(op, None);
            }
            Inst::PtrAdd {
                dst,
                base,
                offset,
            } => self.ptr_add(dst, base, offset),
            Inst::Copy {
                dst,
                src,
                size,
            } => match u32::try_from(size) {
                Ok(size) => {
                    let op = Op::Copy {
                        dst: self.v(dst),
                        src: self.v(src),
                        size,
                    };
                    self.push(op, None);
                }
                Err(_) => self.push(slow, None),
            },
            Inst::Zero {
                dst,
                size,
            } => match u32::try_from(size) {
                Ok(size) => {
                    let op = Op::Zero {
                        dst: self.v(dst),
                        size,
                    };
                    self.push(op, None);
                }
                Err(_) => self.push(slow, None),
            },
            _ => unreachable!("dispatched only for this group's operations"),
        }
    }

    pub(super) fn inst_values(&mut self, bi: usize, ii: usize, inst: &Inst) {
        let slow = Op::Ir {
            block: bi as u32,
            inst: ii as u32,
            loc: NO_LOC,
        };
        match *inst {
            Inst::IConst {
                dst,
                ty,
                value,
            } => {
                let op = Op::Const {
                    dst: self.v(dst),
                    value: mask(ty, value),
                };
                self.push(op, self.single(dst).then_some(dst));
            }
            Inst::FConst {
                dst,
                ty,
                value,
            } => {
                let bits = if ty == Ty::F32 {
                    (value as f32).to_bits() as u64
                } else {
                    value.to_bits()
                };
                self.push(
                    Op::Const {
                        dst: self.v(dst),
                        value: bits,
                    },
                    None,
                );
            }
            Inst::FuncAddr {
                func, ..
            } if (self.late_value)(func) => self.push(slow, None),
            Inst::FuncAddr {
                dst,
                func,
            } => self.push(
                Op::Const {
                    dst: self.v(dst),
                    value: super::super::FUNC_TAG | func.0 as u64,
                },
                None,
            ),
            Inst::Bin {
                dst,
                op,
                ty,
                a,
                b,
            } => self.bin(dst, op, ty, a, b),
            Inst::Un {
                dst,
                op,
                ty,
                a,
            } => {
                let op = Op::Un {
                    op,
                    ty,
                    dst: self.v(dst),
                    a: self.v(a),
                };
                self.push(op, self.single(dst).then_some(dst));
            }
            Inst::Cmp {
                dst,
                op,
                ty,
                a,
                b,
            } => {
                let pure = self.single(dst).then_some(dst);
                if let Some(x) = self.int(b)
                    && let Some(imm) = imm(x, ty)
                {
                    self.fold(b);
                    let op = Op::CmpImm {
                        op,
                        shift: shift_of(ty),
                        dst: self.v(dst),
                        a: self.v(a),
                        imm,
                    };
                    return self.push(op, pure);
                }
                let op = Op::Cmp {
                    op,
                    shift: shift_of(ty),
                    dst: self.v(dst),
                    a: self.v(a),
                    b: self.v(b),
                };
                self.push(op, pure);
            }
            Inst::Conv {
                dst,
                op,
                from,
                to,
                src,
            } => {
                // Registers hold values masked to their type, so these three only mask.
                let width = match op {
                    ConvOp::Trunc | ConvOp::Bitcast => Some(to),
                    ConvOp::ZExt => Some(from),
                    _ => None,
                };
                if let Some(width) = width {
                    let keep = mask(width, u64::MAX);
                    let (dst_reg, src_reg) = (self.v(dst), self.v(src));
                    let op = if keep == u64::MAX {
                        Op::Mov {
                            dst: dst_reg,
                            src: src_reg,
                        }
                    } else {
                        Op::Ext {
                            dst: dst_reg,
                            src: src_reg,
                            mask: keep as u32,
                        }
                    };
                    return self.push(op, self.single(dst).then_some(dst));
                }
                let op = Op::Conv {
                    op,
                    from,
                    to,
                    dst: self.v(dst),
                    src: self.v(src),
                };
                self.push(op, self.single(dst).then_some(dst));
            }
            _ => unreachable!("dispatched only for this group's operations"),
        }
    }

    pub(super) fn inst_calls(&mut self, bi: usize, ii: usize, inst: &Inst) {
        let slow = Op::Ir {
            block: bi as u32,
            inst: ii as u32,
            loc: NO_LOC,
        };
        match *inst {
            Inst::Loc {
                line,
                col,
                file,
                ..
            } => {
                // A location overwritten before anything could observe it is dead.
                if let Some(Op::Loc {
                    ..
                }) = self.ops.last()
                    && self.ops.len() > self.block_start_mark
                {
                    self.ops.pop();
                    self.pure_def.pop();
                }
                self.push(
                    Op::Loc {
                        file,
                        line,
                        col,
                    },
                    None,
                );
            }
            Inst::Call(_) | Inst::Intrinsic(_) => {
                inst_vals(inst, &mut |_| {}, &mut |v| {
                    self.v(v);
                });
                let op = match inst {
                    Inst::Call(call)
                        if matches!(call.callee, ir::Callee::Func(_))
                            && u16::try_from(call.args.len()).is_ok()
                            && u16::try_from(call.results.len()).is_ok() =>
                    {
                        let ir::Callee::Func(callee) = call.callee else {
                            unreachable!("matched above");
                        };
                        let at = self.pool.len() as u32;
                        self.pool.push(NO_LOC);
                        self.pool.extend(call.args.iter().map(|v| v.0));
                        self.pool.extend(call.results.iter().map(|v| v.0));
                        Op::Call {
                            callee: callee.0,
                            at,
                            nargs: call.args.len() as u16,
                            nrets: call.results.len() as u16,
                        }
                    }
                    Inst::Intrinsic(call)
                        if call.op == ir::Intrinsic::BoundsCheck
                            && call.args.len() == 2
                            && call.results.is_empty() =>
                    {
                        Op::BoundsCheck {
                            index: call.args[0].0,
                            count: call.args[1].0,
                        }
                    }
                    Inst::Intrinsic(call)
                        if call.op == ir::Intrinsic::Memcpy
                            && call.args.len() == 3
                            && call.results.is_empty() =>
                    {
                        Op::MemCopy {
                            dst: call.args[0].0,
                            src: call.args[1].0,
                            len: call.args[2].0,
                        }
                    }
                    Inst::Intrinsic(call)
                        if call.op == ir::Intrinsic::Memset
                            && call.args.len() == 3
                            && call.results.is_empty() =>
                    {
                        Op::MemSet {
                            dst: call.args[0].0,
                            value: call.args[1].0,
                            len: call.args[2].0,
                        }
                    }
                    _ => slow,
                };
                self.push(op, None);
            }
            _ => unreachable!("dispatched only for this group's operations"),
        }
    }

    fn bin(&mut self, dst: Val, op: BinOp, ty: Ty, a: Val, b: Val) {
        if let Known::Int(value) = self.known[dst.0 as usize] {
            self.fold(a);
            self.fold(b);
            let op = Op::Const {
                dst: self.v(dst),
                value,
            };
            return self.push(op, Some(dst));
        }
        let pure = if divides(op) {
            None
        } else {
            self.single(dst).then_some(dst)
        };
        let wide = matches!(ty, Ty::I64 | Ty::Ptr);
        let commutes = matches!(
            op,
            BinOp::Add | BinOp::Mul | BinOp::And | BinOp::Or | BinOp::Xor
        );
        let (a, b) = if commutes && self.int(a).is_some() && self.int(b).is_none() {
            (b, a)
        } else {
            (a, b)
        };
        let d = self.v(dst);
        if let Some(x) = self.int(b)
            && let Some(imm) = imm(x, ty)
        {
            self.fold(b);
            let a = self.v(a);
            let op = match op {
                BinOp::Add if wide => Op::AddImm {
                    dst: d,
                    a,
                    imm,
                },
                BinOp::Sub if wide && imm != i32::MIN => Op::AddImm {
                    dst: d,
                    a,
                    imm: -imm,
                },
                BinOp::Mul if wide => Op::MulImm {
                    dst: d,
                    a,
                    imm,
                },
                _ if divides(op) => Op::DivImm {
                    op,
                    ty,
                    dst: d,
                    a,
                    imm,
                },
                _ => Op::BinImm {
                    op,
                    ty,
                    dst: d,
                    a,
                    imm,
                },
            };
            return self.push(op, pure);
        }
        let (a, b) = (self.v(a), self.v(b));
        let op = match op {
            BinOp::Add if wide => Op::Add {
                dst: d,
                a,
                b,
            },
            BinOp::Sub if wide => Op::Sub {
                dst: d,
                a,
                b,
            },
            BinOp::Mul if wide => Op::Mul {
                dst: d,
                a,
                b,
            },
            _ if divides(op) => Op::Div {
                op,
                ty,
                dst: d,
                a,
                b,
            },
            _ => Op::Bin {
                op,
                ty,
                dst: d,
                a,
                b,
            },
        };
        self.push(op, pure);
    }

    fn ptr_add(&mut self, dst: Val, base: Val, offset: Val) {
        let d = self.v(dst);
        let single = self.single(dst) && self.single(base);
        if let Some(x) = self.int(offset) {
            let x = x as i64;
            // A constant offset from the frame is itself a frame address.
            if let Known::Frame(off) = self.known[base.0 as usize]
                && self.single(dst)
                && let Some(sum) = (off as i64).checked_add(x).filter(|s| *s >= 0)
            {
                self.known[dst.0 as usize] = Known::Frame(sum as u64);
                self.fold(base);
                self.fold(offset);
                return self.push(
                    Op::FrameAddr {
                        dst: d,
                        off: sum as u64,
                    },
                    Some(dst),
                );
            }
            // A constant offset from a global is the global's address with that offset.
            if let Known::Global(global, off) = self.known[base.0 as usize]
                && self.single(dst)
                && let Some(sum) = i32::try_from(x).ok().and_then(|x| off.checked_add(x))
            {
                self.known[dst.0 as usize] = Known::Global(global, sum);
                self.fold(base);
                self.fold(offset);
                let op = Op::GlobalAddr {
                    dst: d,
                    global,
                    off: sum,
                };
                return self.push(op, Some(dst));
            }
            // Remember `base + x` for loads and stores later in this block.
            let (root, total) = match self.offsets[base.0 as usize] {
                Some(o) => (o.base, o.off.checked_add(x)),
                None => (base.0, Some(x)),
            };
            if single && let Some(total) = total {
                self.offsets[dst.0 as usize] = Some(Offset {
                    base: root,
                    off: total,
                });
                self.offset_vals.push(dst.0);
            }
            if let Ok(imm) = i32::try_from(x) {
                self.fold(offset);
                let op = Op::AddImm {
                    dst: d,
                    a: self.v(base),
                    imm,
                };
                return self.push(op, single.then_some(dst));
            }
        }
        // `base + i * size` when the multiply was the op just made, for this add alone.
        if self.ops.len() > self.block_start_mark
            && self.single(offset)
            && self.uses[offset.0 as usize] == 1
            && let Some(&Op::MulImm {
                dst: product,
                a: index,
                imm,
            }) = self.ops.last()
            && product == self.v(offset)
            && let Ok(scale) = u16::try_from(imm)
        {
            self.ops.pop();
            self.pure_def.pop();
            let op = Op::AddScaled {
                dst: d,
                a: self.v(base),
                b: index,
                scale,
            };
            return self.push(op, None);
        }
        let op = Op::Add {
            dst: d,
            a: self.v(base),
            b: self.v(offset),
        };
        self.push(op, None);
    }
}
