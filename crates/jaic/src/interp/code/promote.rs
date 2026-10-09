//! Locals in registers.
//!
//! The IR keeps every local in a stack slot, so `i += 1` is a load, an add and a store, and a
//! parameter is stored into its slot before anything reads it. A slot whose address is never
//! taken (no `FrameAddr` op names it) is only reached by the frame loads and stores `build`
//! made, and can live in registers instead: each distinct (offset, size) piece of such a slot
//! becomes a register of its own. That turns the loads and stores into register moves, and
//! the moves are mostly removed again:
//!
//! - a parameter stored once at entry is the parameter itself;
//! - `t = op ...; r = t` writes `r` directly;
//! - `t = r; ... t ...` reads `r` directly, as long as `r` is not written in between.
//!
//! The IR instructions an `Op::Ir` runs name IR values, so a value one of them reads or writes
//! cannot be renamed; the forwarding only applies to values all of whose readers are ops.
use super::{CodeBlock, Op};
use crate::ir::{self, Inst, Term, Ty};
use std::collections::HashMap;

#[derive(Clone, Copy, PartialEq)]
pub(super) enum Use {
    Read,
    Write,
}

/// Calls `f` for each register `op` reads and writes, reads first.
pub(super) fn visit(op: &mut Op, pool: &mut [u32], f: &mut impl FnMut(Use, &mut u32)) {
    use Use::{Read, Write};
    match op {
        Op::Const {
            dst, ..
        }
        | Op::FrameAddr {
            dst, ..
        }
        | Op::GlobalAddr {
            dst, ..
        }
        | Op::ForeignAddr {
            dst, ..
        }
        | Op::LoadFrame {
            dst, ..
        }
        | Op::LoadFrame8 {
            dst, ..
        }
        | Op::LoadFrame32 {
            dst, ..
        }
        | Op::LoadFrame64 {
            dst, ..
        } => f(Write, dst),
        Op::Add {
            dst,
            a,
            b,
        }
        | Op::Sub {
            dst,
            a,
            b,
        }
        | Op::Mul {
            dst,
            a,
            b,
        }
        | Op::Bin {
            dst,
            a,
            b,
            ..
        }
        | Op::Div {
            dst,
            a,
            b,
            ..
        }
        | Op::Cmp {
            dst,
            a,
            b,
            ..
        } => {
            f(Read, a);
            f(Read, b);
            f(Write, dst);
        }
        Op::AddImm {
            dst,
            a,
            ..
        }
        | Op::MulImm {
            dst,
            a,
            ..
        }
        | Op::BinImm {
            dst,
            a,
            ..
        }
        | Op::DivImm {
            dst,
            a,
            ..
        }
        | Op::CmpImm {
            dst,
            a,
            ..
        }
        | Op::Un {
            dst,
            a,
            ..
        } => {
            f(Read, a);
            f(Write, dst);
        }
        Op::Conv {
            dst,
            src,
            ..
        }
        | Op::Mov {
            dst,
            src,
        }
        | Op::Ext {
            dst,
            src,
            ..
        } => {
            f(Read, src);
            f(Write, dst);
        }
        Op::Load {
            dst,
            base,
            ..
        }
        | Op::Load8 {
            dst,
            base,
            ..
        }
        | Op::Load32 {
            dst,
            base,
            ..
        }
        | Op::Load64 {
            dst,
            base,
            ..
        } => {
            f(Read, base);
            f(Write, dst);
        }
        Op::Store {
            base,
            value,
            ..
        }
        | Op::Store8 {
            base,
            value,
            ..
        }
        | Op::Store32 {
            base,
            value,
            ..
        }
        | Op::Store64 {
            base,
            value,
            ..
        } => {
            f(Read, base);
            f(Read, value);
        }
        Op::StoreFrame {
            value, ..
        }
        | Op::StoreFrame8 {
            value, ..
        }
        | Op::StoreFrame32 {
            value, ..
        }
        | Op::StoreFrame64 {
            value, ..
        } => f(Read, value),
        Op::StoreImm {
            base, ..
        } => f(Read, base),
        Op::Copy {
            dst,
            src,
            ..
        } => {
            f(Read, dst);
            f(Read, src);
        }
        Op::Zero {
            dst, ..
        } => f(Read, dst),
        Op::Call {
            at,
            nargs,
            nrets,
            ..
        } => {
            // After the call's location come its arguments, then its results.
            let (at, nargs, nrets) = (*at as usize + 1, *nargs as usize, *nrets as usize);
            for r in &mut pool[at..at + nargs] {
                f(Read, r);
            }
            for r in &mut pool[at + nargs..at + nargs + nrets] {
                f(Write, r);
            }
        }
        Op::BoundsCheck {
            index,
            count,
        } => {
            f(Read, index);
            f(Read, count);
        }
        Op::Branch {
            cond, ..
        } => f(Read, cond),
        Op::BranchCmp {
            a,
            b,
            ..
        } => {
            f(Read, a);
            f(Read, b);
        }
        Op::BranchCmpImm {
            a, ..
        } => f(Read, a),
        Op::Ret1 {
            src,
        } => f(Read, src),
        Op::RetN {
            at,
            n,
        } => {
            for r in &mut pool[*at as usize..(*at + *n) as usize] {
                f(Read, r);
            }
        }
        Op::StoreFrameImm {
            ..
        }
        | Op::Loc {
            ..
        }
        | Op::Ir {
            ..
        }
        | Op::Jump {
            ..
        }
        | Op::Ret0
        | Op::Term {
            ..
        } => {}
    }
}

/// Whether an op with no effect besides writing its register (so it can go when nobody reads it).
fn pure(op: &Op) -> bool {
    matches!(
        op,
        Op::Const { .. }
            | Op::Mov { .. }
            | Op::Ext { .. }
            | Op::FrameAddr { .. }
            | Op::Add { .. }
            | Op::AddImm { .. }
            | Op::Sub { .. }
            | Op::Mul { .. }
            | Op::MulImm { .. }
            | Op::Cmp { .. }
            | Op::CmpImm { .. }
    )
}

/// The registers an IR instruction run by `Op::Ir` reads and writes.
fn ir_regs(func: &ir::Func, op: &Op, reads: &mut Vec<u32>, writes: &mut Vec<u32>) {
    match *op {
        Op::Ir {
            block,
            inst,
            ..
        } => super::inst_vals(
            &func.blocks[block as usize].insts[inst as usize],
            &mut |v| writes.push(v.0),
            &mut |v| reads.push(v.0),
        ),
        Op::Term {
            block,
        } => {
            if let Term::Switch {
                value, ..
            } = &func.blocks[block as usize].term
            {
                reads.push(value.0);
            }
        }
        _ => {}
    }
}

/// How many ops past a move `forward_copies` looks for readers.
const WINDOW: usize = 64;

struct Pass<'a> {
    func: &'a ir::Func,
    ops: &'a mut Vec<Op>,
    pool: &'a mut Vec<u32>,
    blocks: &'a mut [CodeBlock],
    /// IR definitions of each value.
    defs: &'a [u32],
    /// Registers below this are IR values; the rest are slot pieces.
    base: u32,
    dead: Vec<bool>,
    /// Reads of each register by any live op (the ones an `Op::Ir` makes included).
    reads: Vec<u32>,
    writes: Vec<u32>,
}

/// Move the scalar locals of `ops` (the ops of `blocks`, endings not yet resolved to op
/// indices) into registers, and tidy up after. Returns the number of registers now needed.
pub(super) fn promote(
    func: &ir::Func,
    offsets: &[u64],
    defs: &[u32],
    ops: &mut Vec<Op>,
    pool: &mut Vec<u32>,
    blocks: &mut Vec<CodeBlock>,
) -> usize {
    let base = func.vals.len() as u32;
    let Some(pieces) = pieces(func, offsets, ops) else {
        return base as usize;
    };
    // The pieces' registers, in the order of their offsets.
    let mut order: Vec<(u64, (u8, u32))> = pieces.into_iter().collect();
    order.sort_by_key(|(off, _)| *off);
    let mut reg_of = HashMap::new();
    for (i, (off, (size, _))) in order.iter().enumerate() {
        reg_of.insert(*off, (base + i as u32, *size));
    }
    let nregs = base as usize + order.len();
    for op in ops.iter_mut() {
        *op = match *op {
            Op::LoadFrame {
                dst,
                off,
                ..
            } => match reg_of.get(&(off as u64)) {
                Some(&(r, _)) => Op::Mov {
                    dst,
                    src: r,
                },
                None => *op,
            },
            Op::StoreFrame {
                off,
                value,
                ..
            } => match reg_of.get(&(off as u64)) {
                Some(&(r, size)) => store_to(r, size, value),
                None => *op,
            },
            Op::StoreFrameImm {
                ty,
                off,
                imm,
            } => match reg_of.get(&(off as u64)) {
                Some(&(r, _)) => Op::Const {
                    dst: r,
                    value: super::mask(ty, imm as i64 as u64),
                },
                None => *op,
            },
            other => other,
        };
    }
    let mut pass = Pass {
        func,
        ops,
        pool,
        blocks,
        defs,
        base,
        dead: Vec::new(),
        reads: vec![0; nregs],
        writes: vec![0; nregs],
    };
    pass.run();
    nregs
}

/// `r = value` for a piece of `size` bytes: what a store of that width leaves there.
fn store_to(r: u32, size: u8, value: u32) -> Op {
    if size == 8 {
        Op::Mov {
            dst: r,
            src: value,
        }
    } else {
        Op::Ext {
            dst: r,
            src: value,
            mask: ((1u64 << (size as u32 * 8)) - 1) as u32,
        }
    }
}

/// The (frame offset -> size, slot) pieces of the slots that can live in registers; `None`
/// when there are none.
fn pieces(func: &ir::Func, offsets: &[u64], ops: &[Op]) -> Option<HashMap<u64, (u8, u32)>> {
    let slot_of = |off: u64| offsets.partition_point(|&o| o <= off).checked_sub(1);
    // Slots whose address is taken, or that are reached any other way than by frame ops.
    let mut bad = vec![false; func.slots.len()];
    let mut accesses: Vec<Vec<(u64, u8)>> = vec![Vec::new(); func.slots.len()];
    for op in ops {
        let (off, ty) = match *op {
            Op::FrameAddr {
                off, ..
            } => {
                if let Some(s) = slot_of(off) {
                    bad[s] = true;
                }
                continue;
            }
            Op::Ir {
                block,
                inst,
                ..
            } => {
                if let Inst::SlotAddr {
                    slot, ..
                } = &func.blocks[block as usize].insts[inst as usize]
                {
                    bad[slot.0 as usize] = true;
                }
                continue;
            }
            Op::LoadFrame {
                ty,
                off,
                ..
            }
            | Op::StoreFrame {
                ty,
                off,
                ..
            }
            | Op::StoreFrameImm {
                ty,
                off,
                ..
            } => (off as u64, ty),
            _ => continue,
        };
        let Some(s) = slot_of(off) else {
            continue;
        };
        let size = ty.size();
        let end = offsets[s] + func.slots[s].size;
        if !matches!(
            ty,
            Ty::I8 | Ty::I16 | Ty::I32 | Ty::F32 | Ty::I64 | Ty::F64 | Ty::Ptr
        ) || off + size > end
        {
            bad[s] = true;
            continue;
        }
        accesses[s].push((off, size as u8));
    }
    let mut pieces = HashMap::new();
    for (s, list) in accesses.iter_mut().enumerate() {
        if bad[s] || list.is_empty() {
            continue;
        }
        list.sort_unstable();
        list.dedup();
        // Every access to an offset has one size, and none overlaps another.
        let overlaps = list.windows(2).any(|w| w[0].0 + w[0].1 as u64 > w[1].0);
        if overlaps {
            continue;
        }
        for &(off, size) in list.iter() {
            pieces.insert(off, (size, s as u32));
        }
    }
    (!pieces.is_empty()).then_some(pieces)
}

impl Pass<'_> {
    fn run(&mut self) {
        self.dead = vec![false; self.ops.len()];
        self.count();
        self.alias_entry_stores();
        for _ in 0..2 {
            for b in 0..self.blocks.len() {
                self.forward_stores(b);
                self.forward_copies(b);
            }
            self.drop_unread();
        }
        self.compact();
    }

    /// Count the reads and writes of each register by the live ops.
    fn count(&mut self) {
        self.reads.iter_mut().for_each(|n| *n = 0);
        self.writes.iter_mut().for_each(|n| *n = 0);
        let (mut reads, mut writes) = (Vec::new(), Vec::new());
        for i in 0..self.ops.len() {
            if self.dead[i] {
                continue;
            }
            let (r, w) = (&mut self.reads, &mut self.writes);
            visit(&mut self.ops[i], self.pool, &mut |u, reg| match u {
                Use::Read => r[*reg as usize] += 1,
                Use::Write => w[*reg as usize] += 1,
            });
            reads.clear();
            writes.clear();
            ir_regs(self.func, &self.ops[i], &mut reads, &mut writes);
            for &reg in &reads {
                self.reads[reg as usize] += 1;
            }
            for &reg in &writes {
                self.writes[reg as usize] += 1;
            }
        }
    }

    /// Whether an `Op::Ir` or `Op::Term` at `i` reads or writes `reg`.
    fn opaque(&self, i: usize, reg: u32) -> bool {
        let (mut reads, mut writes) = (Vec::new(), Vec::new());
        ir_regs(self.func, &self.ops[i], &mut reads, &mut writes);
        reads.contains(&reg) || writes.contains(&reg)
    }

    /// Whether op `i` reads `reg`, and whether it writes it.
    fn touches(&mut self, i: usize, reg: u32) -> (bool, bool) {
        let (mut read, mut write) = (false, false);
        visit(&mut self.ops[i], self.pool, &mut |u, r| {
            if *r == reg {
                match u {
                    Use::Read => read = true,
                    Use::Write => write = true,
                }
            }
        });
        (read, write)
    }

    /// `piece = t` in the entry block, where nothing else ever writes the piece and `t` is
    /// written once and before: every read of the piece reads `t`.
    fn alias_entry_stores(&mut self) {
        if self.blocks.is_empty() || self.entered_again() {
            return;
        }
        let params = self.func.sig.params.len() as u32;
        let (start, end) = (self.blocks[0].start as usize, self.blocks[0].end as usize);
        for i in start..end {
            let Op::Mov {
                dst,
                src,
            } = self.ops[i]
            else {
                continue;
            };
            if dst < self.base
                || src >= self.base
                || self.writes[dst as usize] != 1
                || self.defs[src as usize] != 1
                || self.writes[src as usize] > 1
            {
                continue;
            }
            // `src` is a parameter, or an earlier result of the entry block.
            let mut defined = src < params;
            for j in start..i {
                defined |= !self.dead[j] && self.touches(j, src).1;
            }
            if !defined {
                continue;
            }
            for j in 0..self.ops.len() {
                if self.dead[j] {
                    continue;
                }
                visit(&mut self.ops[j], self.pool, &mut |u, r| {
                    if u == Use::Read && *r == dst {
                        *r = src;
                    }
                });
            }
            self.reads[src as usize] += self.reads[dst as usize];
            self.reads[dst as usize] = 0;
            self.dead[i] = true;
            self.reads[src as usize] -= 1;
        }
    }

    /// Whether a jump goes to the entry block, which then may run more than once.
    fn entered_again(&self) -> bool {
        self.ops.iter().enumerate().any(|(i, op)| {
            !self.dead[i]
                && matches!(
                    op,
                    Op::Jump {
                        target: 0
                    } | Op::Branch {
                        target: 0,
                        ..
                    } | Op::BranchCmp {
                        target: 0,
                        ..
                    } | Op::BranchCmpImm {
                        target: 0,
                        ..
                    }
                )
        }) || self.func.blocks.iter().any(|b| match &b.term {
            Term::Switch {
                cases,
                default,
                ..
            } => default.0 == 0 || cases.iter().any(|(_, t)| t.0 == 0),
            _ => false,
        })
    }

    /// `t = op ...; piece = t` becomes `piece = op ...` when `t` has no other reader.
    fn forward_stores(&mut self, b: usize) {
        let (start, end) = (self.blocks[b].start as usize, self.blocks[b].end as usize);
        for i in start..end {
            let Op::Mov {
                dst,
                src,
            } = self.ops[i]
            else {
                continue;
            };
            if self.dead[i]
                || dst < self.base
                || src >= self.base
                || self.defs[src as usize] != 1
                || self.reads[src as usize] != 1
                || self.writes[src as usize] != 1
            {
                continue;
            }
            // The op before this one in the block that writes `src`; nothing in between may
            // touch the piece, since it would now be written earlier.
            let mut def = None;
            for j in (start..i).rev() {
                if self.dead[j] {
                    continue;
                }
                if self.opaque(j, src) {
                    break;
                }
                let (read, write) = self.touches(j, src);
                if read {
                    break;
                }
                let (reads_piece, writes_piece) = self.touches(j, dst);
                if write {
                    if !writes_piece {
                        def = Some(j);
                    }
                    break;
                }
                if reads_piece || writes_piece {
                    break;
                }
            }
            let Some(j) = def else {
                continue;
            };
            visit(&mut self.ops[j], self.pool, &mut |u, r| {
                if u == Use::Write && *r == src {
                    *r = dst;
                }
            });
            self.writes[src as usize] = 0;
            self.reads[src as usize] = 0;
            self.dead[i] = true;
        }
    }

    /// After `dst = src`, the ops that follow in the block read `src` where they read `dst`,
    /// until either is written. (The move goes when nothing reads `dst` any more.)
    fn forward_copies(&mut self, b: usize) {
        let (start, end) = (self.blocks[b].start as usize, self.blocks[b].end as usize);
        for i in start..end {
            let Op::Mov {
                dst,
                src,
            } = self.ops[i]
            else {
                continue;
            };
            if self.dead[i] || dst == src {
                continue;
            }
            for j in (i + 1..end).take(WINDOW) {
                if self.dead[j] {
                    continue;
                }
                let mut moved = 0;
                visit(&mut self.ops[j], self.pool, &mut |u, r| {
                    if u == Use::Read && *r == dst {
                        *r = src;
                        moved += 1;
                    }
                });
                self.reads[src as usize] += moved;
                self.reads[dst as usize] -= moved;
                let (_, writes_dst) = self.touches(j, dst);
                let (_, writes_src) = self.touches(j, src);
                if writes_dst || writes_src || self.writes_opaque(j, dst, src) {
                    break;
                }
            }
        }
    }

    /// Whether an `Op::Ir` at `i` writes `a` or `b`.
    fn writes_opaque(&self, i: usize, a: u32, b: u32) -> bool {
        let (mut reads, mut writes) = (Vec::new(), Vec::new());
        ir_regs(self.func, &self.ops[i], &mut reads, &mut writes);
        writes.contains(&a) || writes.contains(&b)
    }

    /// Remove effect-free ops whose result nobody reads.
    fn drop_unread(&mut self) {
        self.count();
        loop {
            let mut changed = false;
            for i in (0..self.ops.len()).rev() {
                if self.dead[i] || !pure(&self.ops[i]) {
                    continue;
                }
                let mut dst = None;
                visit(&mut self.ops[i], self.pool, &mut |u, r| {
                    if u == Use::Write {
                        dst = Some(*r);
                    }
                });
                let Some(dst) = dst else {
                    continue;
                };
                if self.reads[dst as usize] != 0 {
                    continue;
                }
                self.dead[i] = true;
                changed = true;
                let reads = &mut self.reads;
                visit(&mut self.ops[i], self.pool, &mut |u, r| {
                    if u == Use::Read {
                        reads[*r as usize] -= 1;
                    }
                });
            }
            if !changed {
                break;
            }
        }
    }

    /// Remove the dead ops and fix the block ranges.
    fn compact(&mut self) {
        let mut kept = Vec::with_capacity(self.ops.len());
        for block in self.blocks.iter_mut() {
            let (start, end) = (block.start as usize, block.end as usize);
            block.start = kept.len() as u32;
            kept.extend((start..end).filter(|&i| !self.dead[i]).map(|i| self.ops[i]));
            block.end = kept.len() as u32;
        }
        *self.ops = kept;
    }
}
