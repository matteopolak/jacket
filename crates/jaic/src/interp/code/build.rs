//! Turning a procedure's IR into ops: the builder state, folding and operand selection.
use super::super::mask;
use super::op::{Code, CodeBlock, NO_LOC, Op};
use super::{flow, promote, sink};
use crate::ir::{self, BinOp, GlobalId, Inst, Ty, Val, inst_vals, term_uses};

/// The distinct locations of some code and where each applies (`Code::locs`, `Code::loc_marks`).
type LoweredLocs = (Vec<(u32, u32, u32)>, Vec<(u32, u32)>);

/// Take the `Loc` ops out of `ops`. Setting the location costs a dispatch per statement, and
/// only calls and failures look at it, so it is looked up there instead: calls and `Op::Ir`
/// carry their location, and `Code::loc_at` finds the one in effect at any op. Returns the
/// distinct locations and where each applies.
fn lower_locs(ops: &mut Vec<Op>, pool: &mut [u32], blocks: &mut [CodeBlock]) -> LoweredLocs {
    let mut locs: Vec<(u32, u32, u32)> = Vec::new();
    let mut index = crate::fxhash::HashMap::default();
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

/// The debugging switches (`-dev_no_sink`, `-dev_no_flow`, `-dev_dump_ops`), copied once so
/// compiling each procedure does not look them up again.
struct Switches {
    no_sink: bool,
    no_flow: bool,
    dump_ops: Option<String>,
}

fn switches() -> &'static Switches {
    static SWITCHES: std::sync::OnceLock<Switches> = std::sync::OnceLock::new();
    SWITCHES.get_or_init(|| Switches {
        no_sink: crate::knobs::get().no_sink,
        no_flow: crate::knobs::get().no_flow,
        dump_ops: crate::knobs::get().dump_ops.clone(),
    })
}

/// The location each block starts at when its own first instruction does not set one: the one
/// in effect at the end of a block that branches to it (the first such block, in block order).
fn entry_locs(func: &ir::Func) -> Vec<Option<(u32, u32, u32)>> {
    let mut entry: Vec<Option<(u32, u32, u32)>> = vec![None; func.blocks.len()];
    for (bi, block) in func.blocks.iter().enumerate() {
        let mut here = entry[bi];
        for inst in &block.insts {
            if let Inst::Loc {
                file,
                line,
                col,
                ..
            } = *inst
            {
                here = Some((file, line, col));
            }
        }
        let Some(here) = here else {
            continue;
        };
        for next in block.term.successors() {
            let slot = &mut entry[next.0 as usize];
            if slot.is_none() && next.0 as usize != bi {
                *slot = Some(here);
            }
        }
    }
    // A block that sets a location first needs no inherited one.
    for (bi, block) in func.blocks.iter().enumerate() {
        if matches!(block.insts.first(), Some(Inst::Loc { .. })) {
            entry[bi] = None;
        }
    }
    entry
}

pub(in crate::interp) fn fingerprint(func: &ir::Func) -> (usize, usize, usize, usize) {
    (
        func.blocks.as_ptr() as usize,
        func.blocks.len(),
        func.vals.len(),
        func.slots.len(),
    )
}

/// What is known about a value defined once.
#[derive(Clone, Copy)]
pub(super) enum Known {
    Nothing,
    /// Integer constant, already masked to its type.
    Int(u64),
    /// `stack_base + offset`.
    Frame(u64),
    /// The address of a global plus an offset.
    Global(GlobalId, i32),
}

/// Values of a block: `base + off` for values this block computed by adding a constant.
#[derive(Clone, Copy)]
pub(super) struct Offset {
    pub(super) base: u32,
    pub(super) off: i64,
}

pub(super) struct Builder<'a> {
    func: &'a ir::Func,
    /// Whether a procedure's value is decided at run time (`Interp::proc_value`), so its
    /// `FuncAddr` cannot become a constant.
    pub(super) late_value: &'a dyn Fn(ir::FuncId) -> bool,
    pub(super) defs: Vec<u32>,
    /// Uses not folded away yet.
    pub(super) uses: Vec<u32>,
    pub(super) known: Vec<Known>,
    pub(super) offsets: Vec<Option<Offset>>,
    /// The values with an entry in `offsets`, cleared at the end of each block.
    pub(super) offset_vals: Vec<u32>,
    pub(super) ops: Vec<Op>,
    pub(super) pool: Vec<u32>,
    /// The value each op defines, when the op has no other effect (so it can be dropped).
    pub(super) pure_def: Vec<Option<u32>>,
    /// First op of the block being built.
    pub(super) block_start_mark: usize,
}

pub(in crate::interp) fn build(
    func: &ir::Func,
    frame_offsets: &[u64],
    late_value: &dyn Fn(ir::FuncId) -> bool,
    read_only: &dyn Fn(GlobalId) -> bool,
    straighten: bool,
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
                uses[v.0 as usize] += 1;
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
                    known[dst.0 as usize] = Known::Int(mask(ty, value));
                }
                Inst::GlobalAddr {
                    dst,
                    global,
                } if defs[dst.0 as usize] == 1 => known[dst.0 as usize] = Known::Global(global, 0),
                Inst::SlotAddr {
                    dst,
                    slot,
                } if defs[dst.0 as usize] == 1 => {
                    known[dst.0 as usize] = Known::Frame(frame_offsets[slot.0 as usize]);
                }
                // Arithmetic on constants, such as `count * size_of(T)` or a pointer step, is
                // a constant too, so later adds and multiplies take it as an immediate.
                Inst::Bin {
                    dst,
                    op,
                    ty,
                    a,
                    b,
                } if defs[dst.0 as usize] == 1 => {
                    if let (Known::Int(x), Known::Int(y)) =
                        (known[a.0 as usize], known[b.0 as usize])
                        && let Some(value) = fold_bin(op, ty, x, y)
                    {
                        known[dst.0 as usize] = Known::Int(mask(ty, value));
                    }
                }
                _ => {}
            }
        }
    }
    let plan = if switches().no_sink {
        None
    } else {
        sink::plan(func, &defs, read_only)
    };
    if let Some(plan) = &plan {
        for &(block, inst) in &plan.deleted {
            let inst = &func.blocks[block as usize].insts[inst as usize];
            inst_vals(inst, &mut |_| {}, &mut |v| uses[v.0 as usize] -= 1);
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
    let entry_locs = entry_locs(func);
    for (bi, block) in func.blocks.iter().enumerate() {
        let start = b.ops.len() as u32;
        b.block_start_mark = b.ops.len();
        // A block with no location of its own (the failure path of a `#no_debug` macro such as
        // `assert`) reports where the code that branched to it was, not whatever block happens
        // to be laid out before it.
        let inherited = entry_locs[bi];
        if let Some((file, line, col)) = inherited {
            b.push(
                Op::Loc {
                    file,
                    line,
                    col,
                },
                None,
            );
        }
        if let Some(plan) = &plan {
            for &(from, ii) in &plan.front[bi] {
                b.inst(
                    from as usize,
                    ii as usize,
                    &func.blocks[from as usize].insts[ii as usize],
                );
            }
        }
        for (ii, inst) in block.insts.iter().enumerate() {
            if plan.as_ref().is_some_and(|plan| plan.skip[bi][ii]) {
                continue;
            }
            b.inst(bi, ii, inst);
        }
        b.end(bi, block);
        // Where nothing in the block can fail or call, the location would only keep it from
        // being seen as empty.
        if let Some((file, line, col)) = inherited
            && b.ops.len() > start as usize
            && matches!(b.ops[start as usize], Op::Loc { file: f, line: l, col: c } if (f, l, c) == (file, line, col))
            && b.ops[start as usize + 1..]
                .iter()
                .all(|op| flow::copyable(op) || matches!(op, Op::Loc { .. }))
        {
            b.ops.remove(start as usize);
            b.pure_def.remove(start as usize);
        }
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
    if straighten && !switches().no_flow {
        flow::straighten(&mut ops, &mut blocks);
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
            }
            | Op::BranchTest {
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
    if switches()
        .dump_ops
        .as_ref()
        .is_some_and(|v| func.name.contains(v.as_str()))
    {
        eprintln!("== {}\n{}", func.name, func.listing());
        for (bi, bl) in blocks.iter().enumerate() {
            eprintln!(" block {bi}");
            for i in bl.start..bl.end {
                eprintln!("  {i}: {:?}", ops[i as usize]);
            }
        }
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

/// `a op b` for the operators whose result, masked to the type, does not depend on
/// signedness or width.
fn fold_bin(op: BinOp, ty: Ty, a: u64, b: u64) -> Option<u64> {
    matches!(
        op,
        BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::And | BinOp::Or | BinOp::Xor
    )
    .then(|| super::super::int_bin(op, ty, a, b).ok())
    .flatten()
}

pub(super) fn imm(value: u64, ty: Ty) -> Option<i32> {
    let small = value as i64 as i32;
    (mask(ty, small as i64 as u64) == value).then_some(small)
}

impl Builder<'_> {
    pub(super) fn v(&self, v: Val) -> u32 {
        assert!(
            (v.0 as usize) < self.func.vals.len(),
            "value out of range in `{}`",
            self.func.name
        );
        v.0
    }

    pub(super) fn single(&self, v: Val) -> bool {
        self.defs[v.0 as usize] == 1
    }

    pub(super) fn push(&mut self, op: Op, pure_def: Option<Val>) {
        self.ops.push(op);
        self.pure_def.push(pure_def.map(|v| v.0));
    }

    /// Mark one use of `v` as folded into another instruction.
    pub(super) fn fold(&mut self, v: Val) {
        self.uses[v.0 as usize] -= 1;
    }

    pub(super) fn int(&self, v: Val) -> Option<u64> {
        match self.known[v.0 as usize] {
            Known::Int(x) => Some(x),
            _ => None,
        }
    }

    /// Where an address operand points, if not just at a register: the frame, or a
    /// register plus a constant.
    pub(super) fn address(&mut self, addr: Val) -> Address {
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
}

/// The right operand of a fused compare-and-branch.
pub(super) enum Rhs {
    Val(u32),
    Imm(i32),
}

pub(super) enum Address {
    Frame(u32),
    Reg(u32, i32),
}

/// The register an op writes, if any.
pub(super) fn op_dst(op: &Op) -> Option<u32> {
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
        | Op::AddScaled {
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
