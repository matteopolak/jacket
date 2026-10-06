//! The interpreter's own form of a procedure, made from its IR on first call.
//!
//! The IR is built for the native backend: locals live in stack slots, so most statements
//! are `SlotAddr` + `Load`/`Store`, and constants are separate `IConst` instructions. Run
//! as is, every one of those is a dispatch. Here each block becomes a run of 16-byte `Op`s
//! where
//!
//! - an address that is a constant offset from the frame (a slot, or a field of one) is
//!   folded into the load or store that uses it,
//! - a constant pointer offset (`PtrAdd` of an `IConst`) is folded into the load or store
//!   that follows it in the same block,
//! - an integer constant that fits 32 bits becomes an immediate of the arithmetic or
//!   comparison that uses it,
//! - a comparison feeding only the block's branch becomes part of the branch, and
//! - instructions whose results end up unused (the folded constants) are dropped.
//!
//! Calls and intrinsics, which are rare and slow anyway, run through `Interp::step` on the
//! original instruction. Profile counts (`JAIC_PROFILE`) stay in IR instructions.
//!
//! Folding a value into a later instruction reads its operands later than the IR would.
//! That is only done for values defined exactly once (parameters count as a definition)
//! whose definition cannot run again in between: constants anywhere, and other values
//! within one block.
use super::{Frame, Interp, Res, Rets, cmp, mask};
use crate::ir::{self, BinOp, CmpOp, ConvOp, ForeignId, GlobalId, Inst, Term, Ty, UnOp, Val};

#[derive(Clone, Copy, Debug)]
pub(super) enum Op {
    Const {
        dst: u32,
        value: u64,
    },
    /// `dst = stack_base + off`
    FrameAddr {
        dst: u32,
        off: u64,
    },
    /// 64-bit arithmetic that needs no masking.
    Add {
        dst: u32,
        a: u32,
        b: u32,
    },
    AddImm {
        dst: u32,
        a: u32,
        imm: i32,
    },
    Sub {
        dst: u32,
        a: u32,
        b: u32,
    },
    Mul {
        dst: u32,
        a: u32,
        b: u32,
    },
    MulImm {
        dst: u32,
        a: u32,
        imm: i32,
    },
    Bin {
        op: BinOp,
        ty: Ty,
        dst: u32,
        a: u32,
        b: u32,
    },
    /// `imm` is sign-extended, then masked to `ty`.
    BinImm {
        op: BinOp,
        ty: Ty,
        dst: u32,
        a: u32,
        imm: i32,
    },
    Un {
        op: UnOp,
        ty: Ty,
        dst: u32,
        a: u32,
    },
    Cmp {
        op: CmpOp,
        ty: Ty,
        dst: u32,
        a: u32,
        b: u32,
    },
    CmpImm {
        op: CmpOp,
        ty: Ty,
        dst: u32,
        a: u32,
        imm: i32,
    },
    Conv {
        op: ConvOp,
        from: Ty,
        to: Ty,
        dst: u32,
        src: u32,
    },
    GlobalAddr {
        dst: u32,
        global: GlobalId,
    },
    ForeignAddr {
        dst: u32,
        foreign: ForeignId,
    },
    /// `dst = *(base + off)`
    Load {
        ty: Ty,
        dst: u32,
        base: u32,
        off: i32,
    },
    /// `dst = *(stack_base + off)`: always a valid address.
    LoadFrame {
        ty: Ty,
        dst: u32,
        off: u32,
    },
    Store {
        ty: Ty,
        base: u32,
        off: i32,
        value: u32,
    },
    StoreFrame {
        ty: Ty,
        off: u32,
        value: u32,
    },
    /// Stores of a constant (`imm` sign-extended; the store keeps the low bytes).
    StoreImm {
        ty: Ty,
        base: u32,
        off: i32,
        imm: i32,
    },
    StoreFrameImm {
        ty: Ty,
        off: u32,
        imm: i32,
    },
    Copy {
        dst: u32,
        src: u32,
        size: u32,
    },
    Zero {
        dst: u32,
        size: u32,
    },
    Loc {
        file: u32,
        line: u32,
        col: u32,
    },
    /// Run IR instruction `inst` of block `block` (calls, intrinsics, oversized operands).
    Ir {
        block: u32,
        inst: u32,
    },
}

#[cfg(target_pointer_width = "64")]
const _: () = assert!(std::mem::size_of::<Op>() == 16);

/// The right operand of a fused compare-and-branch.
#[derive(Clone, Copy, Debug)]
pub(super) enum Rhs {
    Val(u32),
    Imm(u64),
}

#[derive(Clone, Debug)]
pub(super) enum End {
    Jump(u32),
    Branch {
        cond: u32,
        then_block: u32,
        else_block: u32,
    },
    CmpBranch {
        op: CmpOp,
        ty: Ty,
        a: u32,
        b: Rhs,
        then_block: u32,
        else_block: u32,
    },
    /// Index into the IR block's terminator (switches, returns, unreachable).
    Ir,
}

#[derive(Debug)]
pub(super) struct CodeBlock {
    pub start: u32,
    pub end: u32,
    pub end_op: End,
}

/// A procedure ready to run, plus what identifies the IR it was made from.
#[derive(Debug)]
pub(super) struct Code {
    pub ops: Vec<Op>,
    pub blocks: Vec<CodeBlock>,
    /// `(blocks pointer, block count, value count, slot count)` of the source `Func`: a
    /// procedure replaced or rewritten since gets new code (see `Interp::frame`).
    pub source: (usize, usize, usize, usize),
}

pub(super) fn fingerprint(func: &ir::Func) -> (usize, usize, usize, usize) {
    (
        func.blocks.as_ptr() as usize,
        func.blocks.len(),
        func.vals.len(),
        func.slots.len(),
    )
}

/// What is known about a value defined once.
#[derive(Clone, Copy)]
enum Known {
    Nothing,
    /// Integer constant, already masked to its type.
    Int(u64),
    /// `stack_base + offset`.
    Frame(u64),
}

/// Values of a block: `base + off` for values this block computed by adding a constant.
#[derive(Clone, Copy)]
struct Offset {
    base: u32,
    off: i64,
}

struct Builder<'a> {
    func: &'a ir::Func,
    /// Whether a procedure's value is decided at run time (`Interp::proc_value`), so its
    /// `FuncAddr` cannot become a constant.
    late_value: &'a dyn Fn(ir::FuncId) -> bool,
    defs: Vec<u32>,
    /// Uses not folded away yet.
    uses: Vec<u32>,
    known: Vec<Known>,
    offsets: Vec<Option<Offset>>,
    /// The values with an entry in `offsets`, cleared at the end of each block.
    offset_vals: Vec<u32>,
    ops: Vec<Op>,
    /// The value each op defines, when the op has no other effect (so it can be dropped).
    pure_def: Vec<Option<u32>>,
    /// First op of the block being built.
    block_start_mark: usize,
}

pub(super) fn build(
    func: &ir::Func,
    frame_offsets: &[u64],
    late_value: &dyn Fn(ir::FuncId) -> bool,
) -> Code {
    let n = func.vals.len();
    let mut defs = vec![0u32; n];
    let mut uses = vec![0u32; n];
    for d in defs.iter_mut().take(func.sig.params.len()) {
        *d += 1;
    }
    for block in &func.blocks {
        for inst in &block.insts {
            inst_vals(inst, &mut |v| defs[v.0 as usize] += 1, &mut |v| {
                uses[v.0 as usize] += 1
            });
        }
        term_uses(&block.term, &mut |v| uses[v.0 as usize] += 1);
    }
    let mut known = vec![Known::Nothing; n];
    for block in &func.blocks {
        for inst in &block.insts {
            match *inst {
                Inst::IConst {
                    dst,
                    ty,
                    value,
                } if defs[dst.0 as usize] == 1 => {
                    known[dst.0 as usize] = Known::Int(mask(ty, value))
                }
                Inst::SlotAddr {
                    dst,
                    slot,
                } if defs[dst.0 as usize] == 1 => {
                    known[dst.0 as usize] = Known::Frame(frame_offsets[slot.0 as usize])
                }
                _ => {}
            }
        }
    }
    let mut b = Builder {
        func,
        late_value,
        defs,
        uses,
        known,
        offsets: vec![None; n],
        offset_vals: Vec::new(),
        ops: Vec::new(),
        pure_def: Vec::new(),
        block_start_mark: 0,
    };
    let mut blocks = Vec::with_capacity(func.blocks.len());
    for (bi, block) in func.blocks.iter().enumerate() {
        let start = b.ops.len() as u32;
        b.block_start_mark = b.ops.len();
        for (ii, inst) in block.insts.iter().enumerate() {
            b.inst(bi, ii, inst);
        }
        let end_op = b.end(block);
        // Offsets are only trusted inside the block that computed them.
        for v in std::mem::take(&mut b.offset_vals) {
            b.offsets[v as usize] = None;
        }
        blocks.push(CodeBlock {
            start,
            end: b.ops.len() as u32,
            end_op,
        });
    }
    // Drop definitions nobody reads any more (which may leave their operands unread too),
    // and renumber the block ranges.
    let mut dropped = vec![false; b.ops.len()];
    loop {
        let mut changed = false;
        for i in (0..b.ops.len()).rev() {
            if !dropped[i] && b.pure_def[i].is_some_and(|v| b.uses[v as usize] == 0) {
                dropped[i] = true;
                changed = true;
                for src in op_srcs(&b.ops[i]).into_iter().flatten() {
                    b.uses[src as usize] -= 1;
                }
            }
        }
        if !changed {
            break;
        }
    }
    let mut ops = Vec::with_capacity(b.ops.len());
    for block in &mut blocks {
        let (start, end) = (block.start as usize, block.end as usize);
        block.start = ops.len() as u32;
        ops.extend((start..end).filter(|&i| !dropped[i]).map(|i| b.ops[i]));
        block.end = ops.len() as u32;
    }
    Code {
        ops,
        blocks,
        source: fingerprint(func),
    }
}

/// Calls `def` for each value `inst` defines and `use_` for each it reads.
fn inst_vals(inst: &Inst, def: &mut impl FnMut(Val), use_: &mut impl FnMut(Val)) {
    match inst {
        Inst::IConst {
            dst, ..
        }
        | Inst::FConst {
            dst, ..
        }
        | Inst::SlotAddr {
            dst, ..
        }
        | Inst::GlobalAddr {
            dst, ..
        }
        | Inst::FuncAddr {
            dst, ..
        }
        | Inst::ForeignAddr {
            dst, ..
        } => def(*dst),
        Inst::Bin {
            dst,
            a,
            b,
            ..
        }
        | Inst::Cmp {
            dst,
            a,
            b,
            ..
        } => {
            use_(*a);
            use_(*b);
            def(*dst);
        }
        Inst::Un {
            dst,
            a,
            ..
        } => {
            use_(*a);
            def(*dst);
        }
        Inst::Conv {
            dst,
            src,
            ..
        } => {
            use_(*src);
            def(*dst);
        }
        Inst::Load {
            dst,
            addr,
            ..
        } => {
            use_(*addr);
            def(*dst);
        }
        Inst::Store {
            addr,
            value,
            ..
        } => {
            use_(*addr);
            use_(*value);
        }
        Inst::PtrAdd {
            dst,
            base,
            offset,
        } => {
            use_(*base);
            use_(*offset);
            def(*dst);
        }
        Inst::Copy {
            dst,
            src,
            ..
        } => {
            use_(*dst);
            use_(*src);
        }
        Inst::Zero {
            dst, ..
        } => use_(*dst),
        Inst::Call(call) => {
            if let ir::Callee::Indirect(target, _) = &call.callee {
                use_(*target);
            }
            call.args.iter().copied().for_each(&mut *use_);
            call.results.iter().copied().for_each(def);
        }
        Inst::Intrinsic(call) => {
            call.args.iter().copied().for_each(&mut *use_);
            call.results.iter().copied().for_each(def);
        }
        Inst::Loc {
            ..
        } => {}
    }
}

fn term_uses(term: &Term, use_: &mut impl FnMut(Val)) {
    match term {
        Term::Branch {
            cond, ..
        } => use_(*cond),
        Term::Switch {
            value, ..
        } => use_(*value),
        Term::Ret(values) => values.iter().copied().for_each(use_),
        Term::Jump(_) | Term::Unreachable => {}
    }
}

fn imm(value: u64, ty: Ty) -> Option<i32> {
    let small = value as i64 as i32;
    (mask(ty, small as i64 as u64) == value).then_some(small)
}

impl Builder<'_> {
    fn v(&self, v: Val) -> u32 {
        assert!(
            (v.0 as usize) < self.func.vals.len(),
            "value out of range in `{}`",
            self.func.name
        );
        v.0
    }

    fn single(&self, v: Val) -> bool {
        self.defs[v.0 as usize] == 1
    }

    fn push(&mut self, op: Op, pure_def: Option<Val>) {
        self.ops.push(op);
        self.pure_def.push(pure_def.map(|v| v.0));
    }

    /// Mark one use of `v` as folded into another instruction.
    fn fold(&mut self, v: Val) {
        self.uses[v.0 as usize] -= 1;
    }

    fn int(&self, v: Val) -> Option<u64> {
        match self.known[v.0 as usize] {
            Known::Int(x) => Some(x),
            _ => None,
        }
    }

    /// Where an address operand points, if not just at a register: the frame, or a
    /// register plus a constant.
    fn address(&mut self, addr: Val) -> Address {
        if let Known::Frame(off) = self.known[addr.0 as usize]
            && let Ok(off) = u32::try_from(off)
        {
            self.fold(addr);
            return Address::Frame(off);
        }
        if let Some(o) = self.offsets[addr.0 as usize]
            && let Ok(off) = i32::try_from(o.off)
        {
            self.fold(addr);
            self.uses[o.base as usize] += 1;
            return Address::Reg(o.base, off);
        }
        Address::Reg(self.v(addr), 0)
    }

    fn inst(&mut self, bi: usize, ii: usize, inst: &Inst) {
        let slow = Op::Ir {
            block: bi as u32,
            inst: ii as u32,
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
                    value: super::FUNC_TAG | func.0 as u64,
                },
                None,
            ),
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
                self.push(op, None);
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
                        ty,
                        dst: self.v(dst),
                        a: self.v(a),
                        imm,
                    };
                    return self.push(op, pure);
                }
                let op = Op::Cmp {
                    op,
                    ty,
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
                let op = Op::Conv {
                    op,
                    from,
                    to,
                    dst: self.v(dst),
                    src: self.v(src),
                };
                self.push(op, None);
            }
            Inst::GlobalAddr {
                dst,
                global,
            } => {
                let op = Op::GlobalAddr {
                    dst: self.v(dst),
                    global,
                };
                self.push(op, None);
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
                self.push(slow, None);
            }
        }
    }

    fn bin(&mut self, dst: Val, op: BinOp, ty: Ty, a: Val, b: Val) {
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
                _ => Op::BinImm {
                    op,
                    ty,
                    dst: d,
                    a,
                    imm,
                },
            };
            return self.push(op, None);
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
            _ => Op::Bin {
                op,
                ty,
                dst: d,
                a,
                b,
            },
        };
        self.push(op, None);
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
        let op = Op::Add {
            dst: d,
            a: self.v(base),
            b: self.v(offset),
        };
        self.push(op, None);
    }

    fn end(&mut self, block: &ir::Block) -> End {
        match block.term {
            Term::Jump(t) => End::Jump(t.0),
            Term::Branch {
                cond,
                then_block,
                else_block,
            } => {
                // Checks the front end already decided (`if false` around a trap).
                if let Some(x) = self.int(cond) {
                    self.fold(cond);
                    return End::Jump(if x & 0xff != 0 {
                        then_block.0
                    } else {
                        else_block.0
                    });
                }
                let cond = self.v(cond);
                // A comparison made in this block for this branch alone.
                if self.uses[cond as usize] == 1
                    && self.defs[cond as usize] == 1
                    && let Some(i) = self.ops[self.block_start_mark..]
                        .iter()
                        .rposition(|op| op_dst(op) == Some(cond))
                {
                    let at = self.block_start_mark + i;
                    let fused = match self.ops[at] {
                        Op::Cmp {
                            op,
                            ty,
                            a,
                            b,
                            ..
                        } if self.defs[a as usize] == 1 && self.defs[b as usize] == 1 => {
                            self.uses[a as usize] += 1;
                            self.uses[b as usize] += 1;
                            Some((op, ty, a, Rhs::Val(b)))
                        }
                        Op::CmpImm {
                            op,
                            ty,
                            a,
                            imm,
                            ..
                        } if self.defs[a as usize] == 1 => {
                            self.uses[a as usize] += 1;
                            Some((op, ty, a, Rhs::Imm(mask(ty, imm as i64 as u64))))
                        }
                        _ => None,
                    };
                    if let Some((op, ty, a, b)) = fused {
                        // The comparison's own operand uses stay counted (it is dropped
                        // below only if nothing else reads its result).
                        self.uses[cond as usize] = 0;
                        return End::CmpBranch {
                            op,
                            ty,
                            a,
                            b,
                            then_block: then_block.0,
                            else_block: else_block.0,
                        };
                    }
                }
                End::Branch {
                    cond,
                    then_block: then_block.0,
                    else_block: else_block.0,
                }
            }
            Term::Switch {
                value, ..
            } => {
                self.v(value);
                End::Ir
            }
            Term::Ret(ref values) => {
                for &v in values {
                    self.v(v);
                }
                End::Ir
            }
            Term::Unreachable => End::Ir,
        }
    }
}

enum Address {
    Frame(u32),
    Reg(u32, i32),
}

/// The register an op writes, if any.
fn op_dst(op: &Op) -> Option<u32> {
    match *op {
        Op::Const {
            dst, ..
        }
        | Op::FrameAddr {
            dst, ..
        }
        | Op::Add {
            dst, ..
        }
        | Op::AddImm {
            dst, ..
        }
        | Op::Sub {
            dst, ..
        }
        | Op::Mul {
            dst, ..
        }
        | Op::MulImm {
            dst, ..
        }
        | Op::Bin {
            dst, ..
        }
        | Op::BinImm {
            dst, ..
        }
        | Op::Un {
            dst, ..
        }
        | Op::Cmp {
            dst, ..
        }
        | Op::CmpImm {
            dst, ..
        }
        | Op::Conv {
            dst, ..
        }
        | Op::GlobalAddr {
            dst, ..
        }
        | Op::ForeignAddr {
            dst, ..
        }
        | Op::Load {
            dst, ..
        }
        | Op::LoadFrame {
            dst, ..
        } => Some(dst),
        _ => None,
    }
}

/// The registers a droppable op reads.
fn op_srcs(op: &Op) -> [Option<u32>; 2] {
    match *op {
        Op::AddImm {
            a, ..
        }
        | Op::CmpImm {
            a, ..
        } => [Some(a), None],
        Op::Cmp {
            a,
            b,
            ..
        } => [Some(a), Some(b)],
        _ => [None, None],
    }
}

impl Interp {
    /// Runs `frame.code` (made from `func`) in a frame at `stack_base`. `vals` has one
    /// register per IR value, which `build` checked every operand against.
    pub(super) fn run_code(
        &mut self,
        program: &ir::Program,
        func: &ir::Func,
        frame: &Frame,
        stack_base: u64,
        vals: &mut [u64],
    ) -> Res<Rets> {
        let code = &frame.code;
        assert_eq!(vals.len(), func.vals.len());
        let regs = vals.as_mut_ptr();
        // SAFETY (all register accesses): `build` asserted every register an op names is
        // below `func.vals.len()`, and `vals` has exactly that many.
        let get = |i: u32| unsafe { *regs.add(i as usize) };
        let set = |i: u32, v: u64| unsafe { *regs.add(i as usize) = v };
        let mut block = 0usize;
        loop {
            if let Some(left) = self.block_budget.as_mut() {
                if *left == 0 {
                    return self.trap("execution budget exhausted");
                }
                *left -= 1;
            }
            if self.multi {
                if self.host.cooperative_threads() {
                    self.inline_preempt(program)?;
                } else {
                    #[cfg(not(target_arch = "wasm32"))]
                    self.preempt()?;
                }
            }
            let b = &code.blocks[block];
            let ir_block = &func.blocks[block];
            self.frame_blocks += 1;
            self.frame_insts += ir_block.insts.len() as u64;
            if let Some(counts) = self.profile.as_mut() {
                counts.block(ir_block, &code.ops[b.start as usize..b.end as usize]);
            }
            for op in &code.ops[b.start as usize..b.end as usize] {
                match *op {
                    Op::Const {
                        dst,
                        value,
                    } => set(dst, value),
                    Op::FrameAddr {
                        dst,
                        off,
                    } => set(dst, stack_base + off),
                    Op::Add {
                        dst,
                        a,
                        b,
                    } => set(dst, get(a).wrapping_add(get(b))),
                    Op::AddImm {
                        dst,
                        a,
                        imm,
                    } => set(dst, get(a).wrapping_add(imm as i64 as u64)),
                    Op::Sub {
                        dst,
                        a,
                        b,
                    } => set(dst, get(a).wrapping_sub(get(b))),
                    Op::Mul {
                        dst,
                        a,
                        b,
                    } => set(dst, get(a).wrapping_mul(get(b))),
                    Op::MulImm {
                        dst,
                        a,
                        imm,
                    } => set(dst, get(a).wrapping_mul(imm as i64 as u64)),
                    Op::Bin {
                        op,
                        ty,
                        dst,
                        a,
                        b,
                    } => set(dst, self.bin(op, ty, get(a), get(b))?),
                    Op::BinImm {
                        op,
                        ty,
                        dst,
                        a,
                        imm,
                    } => set(dst, self.bin(op, ty, get(a), mask(ty, imm as i64 as u64))?),
                    Op::Un {
                        op,
                        ty,
                        dst,
                        a,
                    } => {
                        let x = get(a);
                        set(
                            dst,
                            match op {
                                UnOp::Neg => mask(ty, x.wrapping_neg()),
                                UnOp::Not => mask(ty, !x),
                                UnOp::FNeg => {
                                    if ty == Ty::F32 {
                                        (-f32::from_bits(x as u32)).to_bits() as u64
                                    } else {
                                        (-f64::from_bits(x)).to_bits()
                                    }
                                }
                            },
                        );
                    }
                    Op::Cmp {
                        op,
                        ty,
                        dst,
                        a,
                        b,
                    } => set(dst, cmp(op, ty, get(a), get(b)) as u64),
                    Op::CmpImm {
                        op,
                        ty,
                        dst,
                        a,
                        imm,
                    } => set(dst, cmp(op, ty, get(a), mask(ty, imm as i64 as u64)) as u64),
                    Op::Conv {
                        op,
                        from,
                        to,
                        dst,
                        src,
                    } => set(dst, super::conv(op, from, to, get(src))),
                    Op::GlobalAddr {
                        dst,
                        global,
                    } => set(dst, self.global_addr(program, global)?),
                    Op::ForeignAddr {
                        dst,
                        foreign,
                    } => set(dst, self.foreign_addr(program, foreign)?),
                    Op::Load {
                        ty,
                        dst,
                        base,
                        off,
                    } => set(
                        dst,
                        self.load(ty, get(base).wrapping_add(off as i64 as u64))?,
                    ),
                    Op::LoadFrame {
                        ty,
                        dst,
                        off,
                    } => set(dst, unsafe { load_raw(ty, stack_base + off as u64) }),
                    Op::Store {
                        ty,
                        base,
                        off,
                        value,
                    } => self.store(ty, get(base).wrapping_add(off as i64 as u64), get(value))?,
                    Op::StoreFrame {
                        ty,
                        off,
                        value,
                    } => unsafe { store_raw(ty, stack_base + off as u64, get(value)) },
                    Op::StoreImm {
                        ty,
                        base,
                        off,
                        imm,
                    } => self.store(
                        ty,
                        get(base).wrapping_add(off as i64 as u64),
                        imm as i64 as u64,
                    )?,
                    Op::StoreFrameImm {
                        ty,
                        off,
                        imm,
                    } => unsafe { store_raw(ty, stack_base + off as u64, imm as i64 as u64) },
                    Op::Copy {
                        dst,
                        src,
                        size,
                    } => {
                        let (d, s) = (get(dst), get(src));
                        if d < 4096 || s < 4096 {
                            return self.trap(
                                "null pointer dereference: memory copy through a null pointer",
                            );
                        }
                        unsafe { copy_bytes(d, s, size) };
                    }
                    Op::Zero {
                        dst,
                        size,
                    } => {
                        let d = get(dst);
                        if d < 4096 {
                            return self.trap(
                                "null pointer dereference: memory fill through a null pointer",
                            );
                        }
                        unsafe { zero_bytes(d, size) };
                    }
                    Op::Loc {
                        file,
                        line,
                        col,
                    } => self.loc = Some((file, line, col)),
                    Op::Ir {
                        block,
                        inst,
                    } => {
                        let inst = &func.blocks[block as usize].insts[inst as usize];
                        // SAFETY: the same registers, borrowed for this one step only.
                        let vals = unsafe { std::slice::from_raw_parts_mut(regs, func.vals.len()) };
                        self.step(program, inst, vals, frame, stack_base)?;
                    }
                }
            }
            block = match b.end_op {
                End::Jump(t) => t as usize,
                End::Branch {
                    cond,
                    then_block,
                    else_block,
                } => {
                    if get(cond) & 0xff != 0 {
                        then_block as usize
                    } else {
                        else_block as usize
                    }
                }
                End::CmpBranch {
                    op,
                    ty,
                    a,
                    b,
                    then_block,
                    else_block,
                } => {
                    let y = match b {
                        Rhs::Val(b) => get(b),
                        Rhs::Imm(y) => y,
                    };
                    if cmp(op, ty, get(a), y) {
                        then_block as usize
                    } else {
                        else_block as usize
                    }
                }
                End::Ir => match &ir_block.term {
                    Term::Switch {
                        value,
                        ty,
                        cases,
                        default,
                    } => {
                        let v = mask(*ty, get(value.0));
                        cases
                            .iter()
                            .find(|(c, _)| mask(*ty, *c) == v)
                            .map_or(default.0, |(_, t)| t.0) as usize
                    }
                    Term::Ret(values) => {
                        return Ok(Rets::collect(values.iter().map(|v| get(v.0))));
                    }
                    Term::Unreachable => {
                        return self.trap(format!("reached unreachable code in `{}`", func.name));
                    }
                    Term::Jump(t) => t.0 as usize,
                    Term::Branch {
                        ..
                    } => unreachable!("branches are translated"),
                },
            };
        }
    }
}

/// Reads frame memory, which is always mapped.
unsafe fn load_raw(ty: Ty, addr: u64) -> u64 {
    let p = addr as *const u8;
    unsafe {
        match ty {
            Ty::I8 => *p as u64,
            Ty::I16 => std::ptr::read_unaligned(p as *const u16) as u64,
            Ty::I32 | Ty::F32 => std::ptr::read_unaligned(p as *const u32) as u64,
            Ty::I64 | Ty::F64 | Ty::Ptr => std::ptr::read_unaligned(p as *const u64),
            Ty::F80 | Ty::F128 => unreachable!("wide floats are never loaded into registers"),
        }
    }
}

unsafe fn store_raw(ty: Ty, addr: u64, v: u64) {
    let p = addr as *mut u8;
    unsafe {
        match ty {
            Ty::I8 => *p = v as u8,
            Ty::I16 => std::ptr::write_unaligned(p as *mut u16, v as u16),
            Ty::I32 | Ty::F32 => std::ptr::write_unaligned(p as *mut u32, v as u32),
            Ty::I64 | Ty::F64 | Ty::Ptr => std::ptr::write_unaligned(p as *mut u64, v),
            Ty::F80 | Ty::F128 => unreachable!("wide floats are never stored from registers"),
        }
    }
}

/// `memmove` without the library call for the small sizes most copies have.
unsafe fn copy_bytes(dst: u64, src: u64, size: u32) {
    let (d, s) = (dst as *mut u8, src as *const u8);
    unsafe {
        match size {
            8 => {
                std::ptr::write_unaligned(d as *mut u64, std::ptr::read_unaligned(s as *const u64))
            }
            16 => {
                let v = std::ptr::read_unaligned(s as *const [u64; 2]);
                std::ptr::write_unaligned(d as *mut [u64; 2], v);
            }
            4 => {
                std::ptr::write_unaligned(d as *mut u32, std::ptr::read_unaligned(s as *const u32))
            }
            _ => std::ptr::copy(s, d, size as usize),
        }
    }
}

unsafe fn zero_bytes(dst: u64, size: u32) {
    let d = dst as *mut u8;
    unsafe {
        match size {
            1 => *d = 0,
            2 => std::ptr::write_unaligned(d as *mut u16, 0),
            4 => std::ptr::write_unaligned(d as *mut u32, 0),
            8 => std::ptr::write_unaligned(d as *mut u64, 0),
            16 => std::ptr::write_unaligned(d as *mut [u64; 2], [0; 2]),
            _ => std::ptr::write_bytes(d, 0, size as usize),
        }
    }
}
