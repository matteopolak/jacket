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
mod promote;

use super::{Frame, Interp, Res, Trap, TrapKind, bin_total, cmp_shifted, divides, mask, shift_of};
use crate::ir::{
    self, BinOp, CmpOp, ConvOp, ForeignId, GlobalId, Inst, Term, Ty, UnOp, Val, inst_vals,
    term_uses,
};

/// The location of an op that has none (`Op::Call::at`, `Op::Ir::loc`).
pub(super) const NO_LOC: u32 = u32::MAX;

#[derive(Clone, Copy, Debug)]
pub(super) enum Op {
    Const {
        dst: u32,
        value: u64,
    },
    /// `dst = src`
    Mov {
        dst: u32,
        src: u32,
    },
    /// `dst = src & mask`: a store of fewer than 8 bytes to a local kept in a register.
    Ext {
        dst: u32,
        src: u32,
        mask: u32,
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
    /// Division and remainder, which can fail.
    Div {
        op: BinOp,
        ty: Ty,
        dst: u32,
        a: u32,
        b: u32,
    },
    DivImm {
        op: BinOp,
        ty: Ty,
        dst: u32,
        a: u32,
        imm: i32,
    },
    /// `shift`: see `shift_of`.
    Cmp {
        op: CmpOp,
        shift: u8,
        dst: u32,
        a: u32,
        b: u32,
    },
    CmpImm {
        op: CmpOp,
        shift: u8,
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
    // The loads and stores of the common widths, which the code above is turned into when
    // `build` is done (`specialize`): no second dispatch on the type.
    Load8 {
        dst: u32,
        base: u32,
        off: i32,
    },
    Load32 {
        dst: u32,
        base: u32,
        off: i32,
    },
    Load64 {
        dst: u32,
        base: u32,
        off: i32,
    },
    LoadFrame8 {
        dst: u32,
        off: u32,
    },
    LoadFrame32 {
        dst: u32,
        off: u32,
    },
    LoadFrame64 {
        dst: u32,
        off: u32,
    },
    Store8 {
        base: u32,
        off: i32,
        value: u32,
    },
    Store32 {
        base: u32,
        off: i32,
        value: u32,
    },
    Store64 {
        base: u32,
        off: i32,
        value: u32,
    },
    StoreFrame8 {
        off: u32,
        value: u32,
    },
    StoreFrame32 {
        off: u32,
        value: u32,
    },
    StoreFrame64 {
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
    /// Call of a procedure by name. `at` indexes `Code::pool`: the call's location (an index
    /// into `Code::locs`, or `NO_LOC`), the `nargs` argument registers, then the `nrets`
    /// registers that take the results.
    Call {
        callee: u32,
        at: u32,
        nargs: u16,
        nrets: u16,
    },
    /// `Intrinsic::BoundsCheck` of `index` against `count`.
    BoundsCheck {
        index: u32,
        count: u32,
    },
    /// Run IR instruction `inst` of block `block` (other calls and intrinsics, oversized
    /// operands).
    Ir {
        block: u32,
        inst: u32,
        /// Where it runs, as for `Call`.
        loc: u32,
    },
    // Block endings. Targets are op indices; a block that ends by running into the next one
    // has no ending op for it.
    Jump {
        target: u32,
    },
    /// Go to `target` when the condition is `sense`, else on to the next op.
    Branch {
        cond: u32,
        sense: bool,
        target: u32,
    },
    /// `Branch` on a comparison. `shift`: see `shift_of`.
    BranchCmp {
        op: CmpOp,
        shift: u8,
        sense: bool,
        a: u32,
        b: u32,
        target: u32,
    },
    BranchCmpImm {
        op: CmpOp,
        shift: u8,
        sense: bool,
        a: u32,
        imm: i32,
        target: u32,
    },
    Ret0,
    Ret1 {
        src: u32,
    },
    /// Return the `n` registers at `Code::pool[at..]`.
    RetN {
        at: u32,
        n: u32,
    },
    /// The ending of IR block `block` that has no op of its own (a switch, or unreachable).
    Term {
        block: u32,
    },
}

#[cfg(target_pointer_width = "64")]
const _: () = assert!(std::mem::size_of::<Op>() == 16);

#[derive(Debug)]
pub(super) struct CodeBlock {
    /// The first op, and one past the last (the ending included).
    pub start: u32,
    pub end: u32,
}

/// A procedure ready to run, plus what identifies the IR it was made from.
#[derive(Debug)]
pub(super) struct Code {
    pub ops: Vec<Op>,
    pub blocks: Vec<CodeBlock>,
    /// For each op, its block's index plus one if it starts that block, else 0: how a run that
    /// watches blocks (`Interp::watches_blocks`) notices entering one.
    pub block_at: Vec<u32>,
    /// Register lists of calls and returns (`Op::Call`, `Op::RetN`).
    pub pool: Vec<u32>,
    /// The source locations the code runs at (file, line, column).
    pub locs: Vec<(u32, u32, u32)>,
    /// Where each starts to apply: `(op, index into locs)`, in op order. A location holds until
    /// the next one, also across blocks (`loc_at`).
    pub loc_marks: Vec<(u32, u32)>,
    /// Registers a frame needs: the IR's values, and room for the most results returned.
    pub regs: usize,
    /// `(blocks pointer, block count, value count, slot count)` of the source `Func`: a
    /// procedure replaced or rewritten since gets new code (see `Interp::frame`).
    pub source: (usize, usize, usize, usize),
}

impl Code {
    /// The location in effect at `op`: the last one set at or before it, if any.
    pub(super) fn loc_at(&self, op: usize) -> Option<(u32, u32, u32)> {
        let n = self.loc_marks.partition_point(|&(at, _)| at as usize <= op);
        let (_, loc) = *self.loc_marks.get(n.checked_sub(1)?)?;
        Some(self.locs[loc as usize])
    }

    /// The registers that take the results of the call at `op` (an `Op::Call`, or an
    /// `Op::Ir` running a call instruction of `func`).
    pub(super) fn call_results(&self, func: &ir::Func, op: usize) -> Vec<u32> {
        match self.ops[op] {
            Op::Call {
                at,
                nargs,
                nrets,
                ..
            } => {
                let from = (at + 1 + nargs as u32) as usize;
                self.pool[from..from + nrets as usize].to_vec()
            }
            Op::Ir {
                block,
                inst,
                ..
            } => match &func.blocks[block as usize].insts[inst as usize] {
                Inst::Call(call) => call.results.iter().map(|r| r.0).collect(),
                _ => unreachable!("a frame suspends in a call"),
            },
            _ => unreachable!("a frame suspends in a call"),
        }
    }
}

/// The distinct locations of some code and where each applies (`Code::locs`, `Code::loc_marks`).
type LoweredLocs = (Vec<(u32, u32, u32)>, Vec<(u32, u32)>);

/// Take the `Loc` ops out of `ops`. Setting the location costs a dispatch per statement, and
/// only calls and failures look at it, so it is looked up there instead: calls and `Op::Ir`
/// carry their location, and `Code::loc_at` finds the one in effect at any op. Returns the
/// distinct locations and where each applies.
fn lower_locs(ops: &mut Vec<Op>, pool: &mut [u32], blocks: &mut [CodeBlock]) -> LoweredLocs {
    let mut locs: Vec<(u32, u32, u32)> = Vec::new();
    let mut index = std::collections::HashMap::new();
    let mut marks = Vec::new();
    let mut current = NO_LOC;
    let mut kept = Vec::with_capacity(ops.len());
    for block in blocks.iter_mut() {
        let (start, end) = (block.start as usize, block.end as usize);
        block.start = kept.len() as u32;
        for &op in &ops[start..end] {
            match op {
                Op::Loc {
                    file,
                    line,
                    col,
                } => {
                    let loc = (file, line, col);
                    current = *index.entry(loc).or_insert_with(|| {
                        locs.push(loc);
                        locs.len() as u32 - 1
                    });
                    marks.push((kept.len() as u32, current));
                }
                Op::Call {
                    at, ..
                } => {
                    pool[at as usize] = current;
                    kept.push(op);
                }
                Op::Ir {
                    block,
                    inst,
                    ..
                } => kept.push(Op::Ir {
                    block,
                    inst,
                    loc: current,
                }),
                _ => kept.push(op),
            }
        }
        block.end = kept.len() as u32;
    }
    *ops = kept;
    (locs, marks)
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
    pool: Vec<u32>,
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
        pool: Vec::new(),
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
        b.end(bi, block);
        // Offsets are only trusted inside the block that computed them.
        for v in std::mem::take(&mut b.offset_vals) {
            b.offsets[v as usize] = None;
        }
        blocks.push(CodeBlock {
            start,
            end: b.ops.len() as u32,
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
    let nregs = promote::promote(
        func,
        frame_offsets,
        &b.defs,
        &mut ops,
        &mut b.pool,
        &mut blocks,
    );
    for op in &mut ops {
        *op = specialize(*op);
    }
    let (locs, loc_marks) = lower_locs(&mut ops, &mut b.pool, &mut blocks);
    // Endings named blocks; now they name ops.
    let mut regs = nregs;
    for op in &mut ops {
        match op {
            Op::Jump {
                target,
            }
            | Op::Branch {
                target, ..
            }
            | Op::BranchCmp {
                target, ..
            }
            | Op::BranchCmpImm {
                target, ..
            } => *target = blocks[*target as usize].start,
            Op::RetN {
                n, ..
            } => regs = regs.max(*n as usize),
            Op::Ret1 {
                ..
            } => regs = regs.max(1),
            _ => {}
        }
    }
    let mut block_at = vec![0u32; ops.len() + 1];
    for (i, block) in blocks.iter().enumerate() {
        block_at[block.start as usize] = i as u32 + 1;
    }
    Code {
        ops,
        blocks,
        block_at,
        pool: b.pool,
        locs,
        loc_marks,
        regs,
        source: fingerprint(func),
    }
}

/// The op for the loads and stores of the widths most code uses (bytes, 32 and 64 bits) that
/// needs no dispatch on the type; any other op as is.
fn specialize(op: Op) -> Op {
    match op {
        Op::Load {
            ty,
            dst,
            base,
            off,
        } => match ty.size() {
            1 => Op::Load8 {
                dst,
                base,
                off,
            },
            4 => Op::Load32 {
                dst,
                base,
                off,
            },
            8 => Op::Load64 {
                dst,
                base,
                off,
            },
            _ => op,
        },
        Op::LoadFrame {
            ty,
            dst,
            off,
        } => match ty.size() {
            1 => Op::LoadFrame8 {
                dst,
                off,
            },
            4 => Op::LoadFrame32 {
                dst,
                off,
            },
            8 => Op::LoadFrame64 {
                dst,
                off,
            },
            _ => op,
        },
        Op::Store {
            ty,
            base,
            off,
            value,
        } => match ty.size() {
            1 => Op::Store8 {
                base,
                off,
                value,
            },
            4 => Op::Store32 {
                base,
                off,
                value,
            },
            8 => Op::Store64 {
                base,
                off,
                value,
            },
            _ => op,
        },
        Op::StoreFrame {
            ty,
            off,
            value,
        } => match ty.size() {
            1 => Op::StoreFrame8 {
                off,
                value,
            },
            4 => Op::StoreFrame32 {
                off,
                value,
            },
            8 => Op::StoreFrame64 {
                off,
                value,
            },
            _ => op,
        },
        _ => op,
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
                let op = match inst {
                    Inst::Call(call)
                        if matches!(call.callee, ir::Callee::Func(_))
                            && call.args.len() <= u16::MAX as usize
                            && call.results.len() <= u16::MAX as usize =>
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
                    _ => slow,
                };
                self.push(op, None);
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

    /// Emit the ending of block `bi`. A jump to the block that follows is no op at all.
    fn end(&mut self, bi: usize, block: &ir::Block) {
        let next = bi as u32 + 1;
        match block.term {
            Term::Jump(t) => {
                if t.0 != next {
                    self.push(
                        Op::Jump {
                            target: t.0,
                        },
                        None,
                    );
                }
            }
            Term::Branch {
                cond,
                then_block,
                else_block,
            } => {
                // Checks the front end already decided (`if false` around a trap).
                if let Some(x) = self.int(cond) {
                    self.fold(cond);
                    let t = if x & 0xff != 0 {
                        then_block.0
                    } else {
                        else_block.0
                    };
                    if t != next {
                        self.push(
                            Op::Jump {
                                target: t,
                            },
                            None,
                        );
                    }
                    return;
                }
                // Fall through to whichever successor comes next, if either does.
                let (sense, target, after) = if then_block.0 == next {
                    (false, else_block.0, None)
                } else if else_block.0 == next {
                    (true, then_block.0, None)
                } else {
                    (true, then_block.0, Some(else_block.0))
                };
                let cond = self.v(cond);
                let fused = self.fuse_compare(cond);
                let op = match fused {
                    Some((op, shift, a, Rhs::Val(b))) => Op::BranchCmp {
                        op,
                        shift,
                        sense,
                        a,
                        b,
                        target,
                    },
                    Some((op, shift, a, Rhs::Imm(imm))) => Op::BranchCmpImm {
                        op,
                        shift,
                        sense,
                        a,
                        imm,
                        target,
                    },
                    None => Op::Branch {
                        cond,
                        sense,
                        target,
                    },
                };
                self.push(op, None);
                if let Some(other) = after {
                    self.push(
                        Op::Jump {
                            target: other,
                        },
                        None,
                    );
                }
            }
            Term::Switch {
                value, ..
            } => {
                self.v(value);
                self.push(
                    Op::Term {
                        block: bi as u32,
                    },
                    None,
                );
            }
            Term::Ret(ref values) => {
                for &v in values {
                    self.v(v);
                }
                let op = match values[..] {
                    [] => Op::Ret0,
                    [v] => Op::Ret1 {
                        src: v.0,
                    },
                    _ => {
                        let at = self.pool.len() as u32;
                        self.pool.extend(values.iter().map(|v| v.0));
                        Op::RetN {
                            at,
                            n: values.len() as u32,
                        }
                    }
                };
                self.push(op, None);
            }
            Term::Unreachable => self.push(
                Op::Term {
                    block: bi as u32,
                },
                None,
            ),
        }
    }

    /// The comparison `cond` is, when this block made it for its branch alone: its operands,
    /// to compare in the branch itself.
    fn fuse_compare(&mut self, cond: u32) -> Option<(CmpOp, u8, u32, Rhs)> {
        if self.uses[cond as usize] != 1 || self.defs[cond as usize] != 1 {
            return None;
        }
        let i = self.ops[self.block_start_mark..]
            .iter()
            .rposition(|op| op_dst(op) == Some(cond))?;
        let at = self.block_start_mark + i;
        let fused = match self.ops[at] {
            Op::Cmp {
                op,
                shift,
                a,
                b,
                ..
            } if self.defs[a as usize] == 1 && self.defs[b as usize] == 1 => {
                self.uses[a as usize] += 1;
                self.uses[b as usize] += 1;
                (op, shift, a, Rhs::Val(b))
            }
            Op::CmpImm {
                op,
                shift,
                a,
                imm,
                ..
            } if self.defs[a as usize] == 1 => {
                self.uses[a as usize] += 1;
                (op, shift, a, Rhs::Imm(imm))
            }
            _ => return None,
        };
        // The comparison's own operand uses stay counted (it is dropped below only if nothing
        // else reads its result).
        self.uses[cond as usize] = 0;
        Some(fused)
    }
}

/// The right operand of a fused compare-and-branch.
enum Rhs {
    Val(u32),
    Imm(i32),
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
        | Op::Div {
            dst, ..
        }
        | Op::DivImm {
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

/// How `run_ops` stopped.
enum Flow {
    /// The procedure returned this many results, at the start of its registers.
    Returned(u32),
    /// Whether to watch blocks changed: go on at this op with the other loop.
    Switch(usize),
}

impl Interp {
    /// Does anything need to look at each basic block as it is entered?
    #[inline(always)]
    fn watches_blocks(&self) -> bool {
        self.block_budget.is_some() || self.multi || self.profile.is_some()
    }

    /// Make `loc` the location in effect at op `op` of `code`, if it has one.
    #[cold]
    fn sync_loc(&mut self, code: &Code, op: usize) {
        if let Some(loc) = code.loc_at(op) {
            self.loc = Some(loc);
        }
    }

    /// `trap`, which op `op` raised, at the location it ran at.
    #[cold]
    fn located(&mut self, mut trap: Trap, code: &Code, op: usize) -> Trap {
        self.sync_loc(code, op);
        trap.loc = self.loc;
        trap
    }

    /// What entering a block costs when `watches_blocks`: the execution budget, a chance for
    /// other threads to run, and the profile counts.
    #[inline(never)]
    fn enter_block(
        &mut self,
        program: &ir::Program,
        func: &ir::Func,
        code: &Code,
        block: usize,
    ) -> Res<()> {
        let b = &code.blocks[block];
        if let Some(left) = self.block_budget.as_mut() {
            if *left == 0 {
                match self.host.refill_budget() {
                    Some(more) => *left = more,
                    None => return self.trap("execution budget exhausted"),
                }
            }
            *left -= 1;
        }
        if self.multi {
            if self.host.cooperative_threads() {
                if let Err(trap) = self.inline_preempt(program) {
                    if trap.kind == Some(TrapKind::Suspended) {
                        self.suspend_at = Some(b.start as usize);
                    }
                    return Err(trap);
                }
            } else {
                #[cfg(not(target_arch = "wasm32"))]
                self.preempt()?;
            }
        }
        if let Some(counts) = self.profile.as_mut() {
            let ir_block = &func.blocks[block];
            self.frame_blocks += 1;
            self.frame_insts += ir_block.insts.len() as u64;
            counts.block(ir_block, &code.ops[b.start as usize..b.end as usize]);
        }
        Ok(())
    }

    /// Runs `frame.code` (made from `func`) in a frame at `stack_base` whose registers start
    /// at `regs`, and returns how many results it left at the start of them. `start`: the op
    /// to continue at (a resumed thread, see `threads_inline.rs`) instead of the entry. A
    /// thread that suspends records where in `Interp::suspend_at`.
    pub(super) fn run_code(
        &mut self,
        program: &ir::Program,
        func: &ir::Func,
        frame: &Frame,
        stack_base: u64,
        regs: *mut u64,
        start: Option<usize>,
    ) -> Res<u32> {
        let mut at = start;
        let mut watched = self.watches_blocks();
        loop {
            let flow = if watched {
                self.run_ops::<true>(program, func, frame, stack_base, regs, at)?
            } else {
                self.run_ops::<false>(program, func, frame, stack_base, regs, at)?
            };
            match flow {
                Flow::Returned(n) => return Ok(n),
                Flow::Switch(pc) => {
                    watched = !watched;
                    at = Some(pc);
                }
            }
        }
    }

    /// The dispatch loop. `WATCH`: look at each block as it is entered (`enter_block`).
    ///
    /// SAFETY (all register accesses): `build` checked every register an op names against
    /// `frame.code.regs`, which `Interp::enter` made room for at `regs`.
    fn run_ops<const WATCH: bool>(
        &mut self,
        program: &ir::Program,
        func: &ir::Func,
        frame: &Frame,
        stack_base: u64,
        regs: *mut u64,
        start: Option<usize>,
    ) -> Res<Flow> {
        let code = &frame.code;
        let get = |i: u32| unsafe { *regs.add(i as usize) };
        let set = |i: u32, v: u64| unsafe { *regs.add(i as usize) = v };
        let ops = code.ops.as_ptr();
        // A resumed run has done its block's entry already.
        let (mut pc, mut entered) = match start {
            Some(pc) => (pc, true),
            None => (0, false),
        };
        // An op that fails reports where it ran, which only the failure path looks up.
        macro_rules! tri {
            ($e:expr) => {
                match $e {
                    Ok(v) => v,
                    Err(trap) => return Err(self.located(trap, code, pc - 1)),
                }
            };
        }
        loop {
            if WATCH {
                let block = code.block_at[pc];
                if block != 0 && !entered {
                    self.sync_loc(code, pc);
                    self.enter_block(program, func, code, block as usize - 1)?;
                }
                entered = false;
            }
            // SAFETY: every block ends with an op that leaves it (a jump, a return...), so
            // `pc` stays inside `code.ops`.
            let op = unsafe { &*ops.add(pc) };
            let here = pc;
            pc += 1;
            match *op {
                Op::Const {
                    dst,
                    value,
                } => set(dst, value),
                Op::Mov {
                    dst,
                    src,
                } => set(dst, get(src)),
                Op::Ext {
                    dst,
                    src,
                    mask,
                } => set(dst, get(src) & mask as u64),
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
                } => set(dst, bin_total(op, ty, get(a), get(b))),
                Op::Div {
                    op,
                    ty,
                    dst,
                    a,
                    b,
                } => set(dst, tri!(self.divide(op, ty, get(a), get(b)))),
                Op::DivImm {
                    op,
                    ty,
                    dst,
                    a,
                    imm,
                } => set(
                    dst,
                    tri!(self.divide(op, ty, get(a), mask(ty, imm as i64 as u64))),
                ),
                Op::BinImm {
                    op,
                    ty,
                    dst,
                    a,
                    imm,
                } => set(dst, bin_total(op, ty, get(a), mask(ty, imm as i64 as u64))),
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
                    shift,
                    dst,
                    a,
                    b,
                } => set(dst, cmp_shifted(op, shift, get(a), get(b)) as u64),
                Op::CmpImm {
                    op,
                    shift,
                    dst,
                    a,
                    imm,
                } => set(
                    dst,
                    cmp_shifted(op, shift, get(a), imm as i64 as u64) as u64,
                ),
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
                } => set(dst, tri!(self.global_addr(program, global))),
                Op::ForeignAddr {
                    dst,
                    foreign,
                } => set(dst, tri!(self.foreign_addr(program, foreign))),
                Op::Load {
                    ty,
                    dst,
                    base,
                    off,
                } => set(
                    dst,
                    tri!(self.load(ty, get(base).wrapping_add(off as i64 as u64))),
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
                } => tri!(self.store(ty, get(base).wrapping_add(off as i64 as u64), get(value))),
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
                } => tri!(self.store(
                    ty,
                    get(base).wrapping_add(off as i64 as u64),
                    imm as i64 as u64,
                )),
                Op::StoreFrameImm {
                    ty,
                    off,
                    imm,
                } => unsafe { store_raw(ty, stack_base + off as u64, imm as i64 as u64) },
                Op::Load8 {
                    dst,
                    base,
                    off,
                } => set(
                    dst,
                    tri!(self.load(Ty::I8, get(base).wrapping_add(off as i64 as u64))),
                ),
                Op::Load32 {
                    dst,
                    base,
                    off,
                } => set(
                    dst,
                    tri!(self.load(Ty::I32, get(base).wrapping_add(off as i64 as u64))),
                ),
                Op::Load64 {
                    dst,
                    base,
                    off,
                } => set(
                    dst,
                    tri!(self.load(Ty::I64, get(base).wrapping_add(off as i64 as u64))),
                ),
                Op::LoadFrame8 {
                    dst,
                    off,
                } => set(dst, unsafe { load_raw(Ty::I8, stack_base + off as u64) }),
                Op::LoadFrame32 {
                    dst,
                    off,
                } => set(dst, unsafe { load_raw(Ty::I32, stack_base + off as u64) }),
                Op::LoadFrame64 {
                    dst,
                    off,
                } => set(dst, unsafe { load_raw(Ty::I64, stack_base + off as u64) }),
                Op::Store8 {
                    base,
                    off,
                    value,
                } => tri!(self.store(
                    Ty::I8,
                    get(base).wrapping_add(off as i64 as u64),
                    get(value),
                )),
                Op::Store32 {
                    base,
                    off,
                    value,
                } => tri!(self.store(
                    Ty::I32,
                    get(base).wrapping_add(off as i64 as u64),
                    get(value),
                )),
                Op::Store64 {
                    base,
                    off,
                    value,
                } => tri!(self.store(
                    Ty::I64,
                    get(base).wrapping_add(off as i64 as u64),
                    get(value),
                )),
                Op::StoreFrame8 {
                    off,
                    value,
                } => unsafe { store_raw(Ty::I8, stack_base + off as u64, get(value)) },
                Op::StoreFrame32 {
                    off,
                    value,
                } => unsafe { store_raw(Ty::I32, stack_base + off as u64, get(value)) },
                Op::StoreFrame64 {
                    off,
                    value,
                } => unsafe { store_raw(Ty::I64, stack_base + off as u64, get(value)) },
                Op::Copy {
                    dst,
                    src,
                    size,
                } => {
                    let (d, s) = (get(dst), get(src));
                    if d < 4096 || s < 4096 {
                        self.sync_loc(code, here);
                        return self.null_trap("memory copy through a null pointer");
                    }
                    unsafe { copy_bytes(d, s, size) };
                }
                Op::Zero {
                    dst,
                    size,
                } => {
                    let d = get(dst);
                    if d < 4096 {
                        self.sync_loc(code, here);
                        return self.null_trap("memory fill through a null pointer");
                    }
                    unsafe { zero_bytes(d, size) };
                }
                Op::Loc {
                    ..
                } => unreachable!("`build` took the location ops out"),
                Op::Call {
                    callee,
                    at,
                    nargs,
                    nrets,
                } => {
                    let pool = &code.pool[at as usize..];
                    if pool[0] != NO_LOC {
                        self.loc = Some(code.locs[pool[0] as usize]);
                    }
                    let result = self.call_by_name(
                        program,
                        regs,
                        &pool[1..],
                        callee,
                        nargs as usize,
                        nrets as usize,
                    );
                    if let Err(trap) = result {
                        if trap.kind == Some(TrapKind::Suspended) {
                            self.suspend_at = Some(here);
                        }
                        return Err(trap);
                    }
                    if self.watches_blocks() != WATCH {
                        return Ok(Flow::Switch(pc));
                    }
                }
                Op::BoundsCheck {
                    index,
                    count,
                } => {
                    let (index, count) = (get(index), get(count));
                    if (index as i64) < 0 || (index as i64) >= count as i64 {
                        self.sync_loc(code, here);
                        return self.check_trap(ir::TRAP_BOUNDS, index, count);
                    }
                }
                Op::Ir {
                    block,
                    inst,
                    loc,
                } => {
                    if loc != NO_LOC {
                        self.loc = Some(code.locs[loc as usize]);
                    }
                    let inst = &func.blocks[block as usize].insts[inst as usize];
                    // SAFETY: the same registers, borrowed for this one step only.
                    let vals = unsafe { std::slice::from_raw_parts_mut(regs, code.regs) };
                    if let Err(trap) = self.step(program, inst, vals, frame, stack_base) {
                        if trap.kind == Some(TrapKind::Suspended) {
                            self.suspend_at = Some(here);
                        }
                        return Err(trap);
                    }
                    if self.watches_blocks() != WATCH {
                        return Ok(Flow::Switch(pc));
                    }
                }
                Op::Jump {
                    target,
                } => pc = target as usize,
                Op::Branch {
                    cond,
                    sense,
                    target,
                } => {
                    if (get(cond) & 0xff != 0) == sense {
                        pc = target as usize;
                    }
                }
                Op::BranchCmp {
                    op,
                    shift,
                    sense,
                    a,
                    b,
                    target,
                } => {
                    if cmp_shifted(op, shift, get(a), get(b)) == sense {
                        pc = target as usize;
                    }
                }
                Op::BranchCmpImm {
                    op,
                    shift,
                    sense,
                    a,
                    imm,
                    target,
                } => {
                    if cmp_shifted(op, shift, get(a), imm as i64 as u64) == sense {
                        pc = target as usize;
                    }
                }
                Op::Ret0 => return Ok(Flow::Returned(0)),
                Op::Ret1 {
                    src,
                } => {
                    set(0, get(src));
                    return Ok(Flow::Returned(1));
                }
                Op::RetN {
                    at,
                    n,
                } => {
                    let list = &code.pool[at as usize..(at + n) as usize];
                    let mut small = [0u64; 8];
                    if list.len() <= small.len() {
                        for (slot, &r) in small.iter_mut().zip(list) {
                            *slot = get(r);
                        }
                        for (i, &v) in small[..list.len()].iter().enumerate() {
                            set(i as u32, v);
                        }
                    } else {
                        let values: Vec<u64> = list.iter().map(|&r| get(r)).collect();
                        for (i, &v) in values.iter().enumerate() {
                            set(i as u32, v);
                        }
                    }
                    return Ok(Flow::Returned(n));
                }
                Op::Term {
                    block,
                } => match &func.blocks[block as usize].term {
                    Term::Switch {
                        value,
                        ty,
                        cases,
                        default,
                    } => {
                        let v = mask(*ty, get(value.0));
                        let target = cases
                            .iter()
                            .find(|(c, _)| mask(*ty, *c) == v)
                            .map_or(default.0, |(_, t)| t.0);
                        pc = code.blocks[target as usize].start as usize;
                    }
                    Term::Unreachable => {
                        self.sync_loc(code, here);
                        return self.trap(format!("reached unreachable code in `{}`", func.name));
                    }
                    _ => unreachable!("only switches and unreachable keep their IR ending"),
                },
            }
        }
    }
}

/// Reads `ty` at `addr`, which is the caller's to have checked.
#[inline(always)]
pub(super) unsafe fn load_raw(ty: Ty, addr: u64) -> u64 {
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

/// Writes `ty` at `addr`, which is the caller's to have checked.
#[inline(always)]
pub(super) unsafe fn store_raw(ty: Ty, addr: u64, v: u64) {
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
