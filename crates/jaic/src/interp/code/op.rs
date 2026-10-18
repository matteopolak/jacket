//! The compact op set the interpreter runs and the code built from it for one procedure.
use crate::ir::{self, BinOp, CmpOp, ConvOp, ForeignId, GlobalId, Inst, Ty, UnOp};

/// The location of an op that has none (`Op::Call::at`, `Op::Ir::loc`).
pub(in crate::interp) const NO_LOC: u32 = u32::MAX;

#[derive(Clone, Copy, Debug)]
pub(in crate::interp) enum Op {
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
    /// `dst = a + b * scale`: an element's address from its index.
    AddScaled {
        dst: u32,
        a: u32,
        b: u32,
        scale: u16,
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
    /// `dst = address of global + off`
    GlobalAddr {
        dst: u32,
        global: GlobalId,
        off: i32,
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
    /// The `Memcpy` intrinsic: `len` bytes from `src` to `dst` (they may overlap).
    MemCopy {
        dst: u32,
        src: u32,
        len: u32,
    },
    /// The `Memset` intrinsic: `len` bytes of `value`'s low byte at `dst`.
    MemSet {
        dst: u32,
        value: u32,
        len: u32,
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
    /// Branch when `a & mask` is nonzero (`sense` true) or zero (false). `mask` is sign-extended.
    BranchTest {
        sense: bool,
        a: u32,
        mask: i32,
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
pub(in crate::interp) struct CodeBlock {
    /// The first op, and one past the last (the ending included).
    pub start: u32,
    pub end: u32,
}

/// A procedure ready to run, plus what identifies the IR it was made from.
#[derive(Debug)]
pub(in crate::interp) struct Code {
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
    pub(in crate::interp) fn loc_at(&self, op: usize) -> Option<(u32, u32, u32)> {
        let n = self.loc_marks.partition_point(|&(at, _)| at as usize <= op);
        let (_, loc) = *self.loc_marks.get(n.checked_sub(1)?)?;
        Some(self.locs[loc as usize])
    }

    /// The registers that take the results of the call at `op` (an `Op::Call`, or an
    /// `Op::Ir` running a call instruction of `func`).
    pub(in crate::interp) fn call_results(&self, func: &ir::Func, op: usize) -> Vec<u32> {
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
