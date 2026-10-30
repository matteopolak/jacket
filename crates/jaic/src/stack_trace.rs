//! `context.stack_trace` for compiled output: an IR pass that makes every traced function push a
//! `Stack_Trace_Node` on entry and pop it on return. The interpreter maintains the list itself
//! (`Interp::trace_enter`); native backends get it from this pass instead.
//!
//! Nodes are `Stack_Trace_Node`s, laid out as `Program::stack_trace` says; `info` points at
//! a `Stack_Trace_Procedure_Info` global holding the procedure's name, declaration site and
//! address.

use crate::ir::{
    BinOp, Block, BlockId, Callee, CmpOp, ConvOp, Func, FuncId, Global, GlobalId, Inst, Intrinsic,
    Program, Reloc, RelocTarget, Slot, SlotId, Term, TraceInfo, TraceInfoLayout, TraceLayout, Ty,
    Val, inst_vals, term_uses,
};

const HASH_SEED: u64 = 0xcbf2_9ce4_8422_2325;
const HASH_PRIME: u64 = 0x0100_0000_01b3;

/// Instrument every function that has `trace` info; the info is then cleared, so running the
/// result in the interpreter does not push a second node.
pub fn instrument(program: &mut Program, layout: &TraceLayout) {
    let quiet = quiet_funcs(&program.funcs);
    for i in 0..program.funcs.len() {
        let Some(info) = program.funcs[i].as_ref().and_then(|f| f.trace.clone()) else {
            continue;
        };
        if quiet[i] {
            // Nothing can read the stack from a procedure that only calls procedures like
            // itself, so it needs no node: the small helpers a hot loop calls cost nothing.
            program.funcs[i].as_mut().unwrap().trace = None;
            continue;
        }
        let info_global = info_global(program, FuncId(i as u32), &info, &layout.info);
        let func = program.funcs[i].as_mut().unwrap();
        func.trace = None;
        instrument_func(func, FuncId(i as u32), &info, info_global, layout);
    }
}

/// [`quiet_but_for_callees`] of every function, scanned on the shared thread pool.
fn scan_bodies(funcs: &[Option<Func>]) -> Vec<Option<Vec<u32>>> {
    use rayon::prelude::*;
    if crate::pool::threads() <= 1 {
        return funcs
            .iter()
            .map(|f| f.as_ref().and_then(quiet_but_for_callees))
            .collect();
    }
    let chunk = funcs.len().div_ceil(crate::pool::threads()).max(256);
    crate::pool::get().install(|| {
        funcs
            .par_chunks(chunk)
            .map(|part| {
                part.iter()
                    .map(|f| f.as_ref().and_then(quiet_but_for_callees))
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>()
            .into_iter()
            .flatten()
            .collect()
    })
}

/// The procedures that never let anything look at the stack trace: they touch their context
/// pointer only to pass it on, and call only other quiet procedures (directly), themselves
/// included. A procedure that calls nothing is quiet; so, in turn, is one that calls only those.
/// Runtime check failures report their own location, so they don't need a node either.
fn quiet_funcs(funcs: &[Option<Func>]) -> Vec<bool> {
    // Each body is scanned once for what it needs of its callees. A procedure is loud when it
    // looks at its context itself, calls through a value, or reaches a loud procedure; loudness
    // spreads from those to their callers, and what it never reaches is quiet. That includes
    // procedures that call only themselves or each other: nothing in such a cycle can read the
    // stack, so recursion costs no more than a loop.
    let mut loud = vec![false; funcs.len()];
    let mut callers: Vec<Vec<u32>> = vec![Vec::new(); funcs.len()];
    let mut work = Vec::new();
    for (i, callees) in scan_bodies(funcs).into_iter().enumerate() {
        let Some(callees) = callees else {
            loud[i] = true;
            work.push(i as u32);
            continue;
        };
        for callee in callees {
            if let Some(list) = callers.get_mut(callee as usize) {
                list.push(i as u32);
            } else if !loud[i] {
                // A callee that does not exist is never quiet.
                loud[i] = true;
                work.push(i as u32);
            }
        }
    }
    while let Some(i) = work.pop() {
        for &caller in &callers[i as usize] {
            if !std::mem::replace(&mut loud[caller as usize], true) {
                work.push(caller);
            }
        }
    }
    loud.iter().map(|&l| !l).collect()
}

/// The procedures `func` calls directly, when nothing else in it makes it loud: it calls
/// only through `Callee::Func` and never lets its context escape except to those callees.
fn quiet_but_for_callees(func: &Func) -> Option<Vec<u32>> {
    // A procedure with a context keeps it in a slot (`FnCtx::set_context` stores the parameter
    // there at entry, and calls pass on what they load back), so mentioning the parameter is
    // not a use by itself. The context counts as used when anything but a quiet callee gets
    // hold of it: the parameter, the slot's address or a value loaded from the slot.
    let (slot_addr, loaded) = context_values(func);
    let is_context = |v: Val| v == Val(0) || Some(v) == slot_addr || loaded.contains(&v);
    let mut uses_context = false;
    let mut callees = Vec::new();
    for block in &func.blocks {
        for inst in &block.insts {
            match inst {
                Inst::Call(call) => {
                    let Callee::Func(callee) = call.callee else {
                        return None;
                    };
                    callees.push(callee.0);
                    // Passing the context on to a quiet callee is fine, but not the slot itself.
                    uses_context |= call.args.iter().any(|&a| Some(a) == slot_addr);
                    continue;
                }
                // Keeping the parameter, or a value read back, in the slot.
                Inst::Store {
                    addr,
                    value,
                    ..
                } if Some(*addr) == slot_addr && Some(*value) != slot_addr => continue,
                // Reading the slot is a use only through what is done with the value.
                Inst::Load {
                    addr, ..
                } if Some(*addr) == slot_addr => continue,
                _ => {}
            }
            inst_vals(inst, &mut |_| {}, &mut |v| uses_context |= is_context(v));
        }
        term_uses(&block.term, &mut |v| uses_context |= is_context(v));
    }
    (!uses_context).then_some(callees)
}

/// The slot the context parameter lives in (its address) and the values loaded back from it.
fn context_values(func: &Func) -> (Option<Val>, crate::fxhash::HashSet<Val>) {
    let slot_addr = context_slot_addr(func);
    let mut loaded = crate::fxhash::HashSet::default();
    if let Some(slot_addr) = slot_addr {
        for block in &func.blocks {
            for inst in &block.insts {
                if let Inst::Load {
                    dst,
                    addr,
                    ..
                } = inst
                    && *addr == slot_addr
                {
                    loaded.insert(*dst);
                }
            }
        }
    }
    (slot_addr, loaded)
}

/// The address of the slot the context parameter is stored in, when there is exactly one.
fn context_slot_addr(func: &Func) -> Option<Val> {
    let mut found = None;
    for block in &func.blocks {
        for inst in &block.insts {
            if let Inst::Store {
                addr,
                value,
                ..
            } = inst
                && *value == Val(0)
            {
                if found.is_some_and(|known| known != *addr) {
                    return None;
                }
                found = Some(*addr);
            }
        }
    }
    found
}

fn info_global(
    program: &mut Program,
    id: FuncId,
    info: &TraceInfo,
    at: &TraceInfoLayout,
) -> GlobalId {
    let path = program
        .file_paths
        .get(info.file as usize)
        .cloned()
        .unwrap_or_default();
    let mut init = vec![0u8; at.size as usize];
    let mut relocs = Vec::new();
    for (offset, text) in [
        (at.name as usize, info.name.as_str()),
        (at.path as usize, path.as_str()),
    ] {
        init[offset..offset + 8].copy_from_slice(&(text.len() as u64).to_le_bytes());
        if !text.is_empty() {
            let mut bytes = text.as_bytes().to_vec();
            bytes.push(0);
            let g = program.add_global(Global {
                name: format!("trace_text.{}", program.globals.len()),
                size: bytes.len() as u64,
                align: 1,
                init: bytes,
                relocs: Vec::new(),
                read_only: true,
                export: None,
            });
            relocs.push(Reloc {
                offset: offset as u64 + 8,
                target: RelocTarget::Global(g),
                addend: 0,
            });
        }
    }
    let (line, column) = (at.line as usize, at.column as usize);
    init[line..line + 8].copy_from_slice(&(info.line as u64).to_le_bytes());
    init[column..column + 8].copy_from_slice(&(info.col as u64).to_le_bytes());
    relocs.push(Reloc {
        offset: at.procedure_address,
        target: RelocTarget::Func(id),
        addend: 0,
    });
    program.add_global(Global {
        name: format!("trace_info.{}", id.0),
        size: at.size,
        align: 8,
        init,
        relocs,
        read_only: true,
        export: None,
    })
}

struct Emit<'a> {
    func: &'a mut Func,
}

impl Emit<'_> {
    fn val(&mut self, ty: Ty) -> Val {
        self.func.vals.push(ty);
        Val(self.func.vals.len() as u32 - 1)
    }

    fn iconst(&mut self, insts: &mut Vec<Inst>, ty: Ty, value: u64) -> Val {
        let dst = self.val(ty);
        insts.push(Inst::IConst {
            dst,
            ty,
            value,
        });
        dst
    }

    fn offset(&mut self, insts: &mut Vec<Inst>, base: Val, by: u64) -> Val {
        let offset = self.iconst(insts, Ty::I64, by);
        let dst = self.val(Ty::Ptr);
        insts.push(Inst::PtrAdd {
            dst,
            base,
            offset,
        });
        dst
    }

    fn load(&mut self, insts: &mut Vec<Inst>, ty: Ty, addr: Val) -> Val {
        let dst = self.val(ty);
        insts.push(Inst::Load {
            dst,
            ty,
            addr,
        });
        dst
    }

    fn bin(&mut self, insts: &mut Vec<Inst>, op: BinOp, ty: Ty, a: Val, b: Val) -> Val {
        let dst = self.val(ty);
        insts.push(Inst::Bin {
            dst,
            op,
            ty,
            a,
            b,
        });
        dst
    }

    fn store_at(&mut self, insts: &mut Vec<Inst>, ty: Ty, base: Val, by: u64, value: Val) {
        let addr = self.offset(insts, base, by);
        insts.push(Inst::Store {
            ty,
            addr,
            value,
        });
    }

    fn block(&mut self, insts: Vec<Inst>, term: Term) -> BlockId {
        self.func.blocks.push(Block {
            insts,
            term,
        });
        BlockId(self.func.blocks.len() as u32 - 1)
    }
}

fn instrument_func(
    func: &mut Func,
    id: FuncId,
    info: &TraceInfo,
    info_global: GlobalId,
    layout: &TraceLayout,
) {
    if func.blocks.is_empty() || func.sig.params.first() != Some(&Ty::Ptr) {
        return;
    }
    let n = &layout.node;
    let original_blocks = func.blocks.len();
    // Where the node goes up, and which statements record their line, are decided on the code as
    // it was written.
    let line_stores = line_stores(&func.blocks);
    let push_at = push_block(func, &line_stores);
    func.slots.push(Slot {
        size: n.size,
        align: 8,
    });
    let slot = SlotId(func.slots.len() as u32 - 1);
    let mut e = Emit {
        func,
    };

    // The original entry moves to a new block; the new entry finds the previous node.
    let body = std::mem::replace(
        &mut e.func.blocks[0],
        Block {
            insts: Vec::new(),
            term: Term::Unreachable,
        },
    );
    let body_id = BlockId(e.func.blocks.len() as u32);
    e.func.blocks.push(body);
    for b in 1..e.func.blocks.len() {
        retarget(&mut e.func.blocks[b].term, BlockId(0), body_id);
    }
    // The push sits in front of `push_at`, wherever the paths to the first thing that can see the
    // node meet: procedures that return early without a call never pay for it. Everything that
    // enters `push_at` goes through it.
    let target = if push_at == 0 {
        body_id
    } else {
        BlockId(push_at as u32)
    };
    let push_id = BlockId(e.func.blocks.len() as u32);
    e.func.blocks.push(Block {
        insts: Vec::new(),
        term: Term::Unreachable,
    });
    if push_at != 0 {
        for b in (1..original_blocks).chain(std::iter::once(body_id.0 as usize)) {
            retarget(&mut e.func.blocks[b].term, target, push_id);
        }
    }

    let mut entry = Vec::new();
    let node = e.val(Ty::Ptr);
    entry.push(Inst::SlotAddr {
        dst: node,
        slot,
    });
    let top = e.offset(&mut entry, Val(0), layout.context);
    // Popping stores this back unconditionally: with no node pushed it is what is already there.
    let previous = e.load(&mut entry, Ty::Ptr, top);
    let line_addr = e.offset(&mut entry, node, n.line_number);
    e.func.blocks[0] = Block {
        insts: entry,
        term: Term::Jump(if push_at == 0 {
            push_id
        } else {
            body_id
        }),
    };

    let mut dispatch = Vec::new();
    let null = e.iconst(&mut dispatch, Ty::Ptr, 0);
    let is_first = e.val(Ty::I8);
    dispatch.push(Inst::Cmp {
        dst: is_first,
        op: CmpOp::Eq,
        ty: Ty::Ptr,
        a: previous,
        b: null,
    });

    // Common tail: link the node and make it the top.
    let mut link = Vec::new();
    e.store_at(&mut link, Ty::Ptr, node, n.next, previous);
    let info_addr = e.val(Ty::Ptr);
    link.push(Inst::GlobalAddr {
        dst: info_addr,
        global: info_global,
    });
    e.store_at(&mut link, Ty::Ptr, node, n.info, info_addr);
    let line = e.iconst(&mut link, Ty::I32, info.line as u64);
    link.push(Inst::Store {
        ty: Ty::I32,
        addr: line_addr,
        value: line,
    });
    link.push(Inst::Store {
        ty: Ty::Ptr,
        addr: top,
        value: node,
    });
    let link_id = e.block(link, Term::Jump(target));

    // First node: depth 1, seed hash.
    let mut first = Vec::new();
    let one = e.iconst(&mut first, Ty::I32, 1);
    e.store_at(&mut first, Ty::I32, node, n.call_depth, one);
    let seed = e.iconst(&mut first, Ty::I64, HASH_SEED ^ id.0 as u64);
    let prime = e.iconst(&mut first, Ty::I64, HASH_PRIME);
    let hash = e.bin(&mut first, BinOp::Mul, Ty::I64, seed, prime);
    e.store_at(&mut first, Ty::I64, node, n.hash, hash);
    let first_id = e.block(first, Term::Jump(link_id));

    // Nested: depth + 1; the hash mixes the caller's hash, this procedure and the call line.
    let mut nested = Vec::new();
    let depth_addr = e.offset(&mut nested, previous, n.call_depth);
    e.func.trusted.push(depth_addr);
    let depth = e.load(&mut nested, Ty::I32, depth_addr);
    let one = e.iconst(&mut nested, Ty::I32, 1);
    let depth = e.bin(&mut nested, BinOp::Add, Ty::I32, depth, one);
    e.store_at(&mut nested, Ty::I32, node, n.call_depth, depth);
    let hash_addr = e.offset(&mut nested, previous, n.hash);
    e.func.trusted.push(hash_addr);
    let hash = e.load(&mut nested, Ty::I64, hash_addr);
    let caller_line_addr = e.offset(&mut nested, previous, n.line_number);
    e.func.trusted.push(caller_line_addr);
    let caller_line = e.load(&mut nested, Ty::I32, caller_line_addr);
    let caller_line64 = e.val(Ty::I64);
    nested.push(Inst::Conv {
        dst: caller_line64,
        op: ConvOp::ZExt,
        from: Ty::I32,
        to: Ty::I64,
        src: caller_line,
    });
    let thirty_two = e.iconst(&mut nested, Ty::I64, 32);
    let shifted = e.bin(&mut nested, BinOp::Shl, Ty::I64, caller_line64, thirty_two);
    let func_id = e.iconst(&mut nested, Ty::I64, id.0 as u64);
    let mixed = e.bin(&mut nested, BinOp::Xor, Ty::I64, hash, func_id);
    let mixed = e.bin(&mut nested, BinOp::Xor, Ty::I64, mixed, shifted);
    let prime = e.iconst(&mut nested, Ty::I64, HASH_PRIME);
    let hash = e.bin(&mut nested, BinOp::Mul, Ty::I64, mixed, prime);
    e.store_at(&mut nested, Ty::I64, node, n.hash, hash);
    let nested_id = e.block(nested, Term::Jump(link_id));

    e.func.blocks[push_id.0 as usize] = Block {
        insts: dispatch,
        term: Term::Branch {
            cond: is_first,
            then_block: first_id,
            else_block: nested_id,
        },
    };

    // The original code (its entry now lives at `body_id`): record call lines, pop on return.
    let originals = (1..original_blocks).chain(std::iter::once(body_id.0 as usize));
    for (b, original) in originals.zip((1..original_blocks).chain(std::iter::once(0))) {
        let old = std::mem::take(&mut e.func.blocks[b].insts);
        let mut insts = Vec::with_capacity(old.len() + 4);
        for (k, inst) in old.iter().enumerate() {
            insts.push(inst.clone());
            if let Inst::Loc {
                line, ..
            } = inst
                && line_stores[original].contains(&k)
            {
                let value = e.iconst(&mut insts, Ty::I32, *line as u64);
                insts.push(Inst::Store {
                    ty: Ty::I32,
                    addr: line_addr,
                    value,
                });
            }
        }
        if matches!(e.func.blocks[b].term, Term::Ret(_)) {
            insts.push(Inst::Store {
                ty: Ty::Ptr,
                addr: top,
                value: previous,
            });
        }
        e.func.blocks[b].insts = insts;
    }
}

/// Per block, the positions of the `Loc`s whose line must be recorded: the statement makes a call.
fn line_stores(blocks: &[Block]) -> Vec<Vec<usize>> {
    let reaches_call = reaches_call_from_entry(blocks);
    blocks
        .iter()
        .map(|block| {
            let mut term_calls = false;
            block
                .term
                .for_each_successor(|s| term_calls |= reaches_call[s.0 as usize]);
            block
                .insts
                .iter()
                .enumerate()
                .filter(|(k, inst)| {
                    matches!(inst, Inst::Loc { .. })
                        && calls_before_next_loc(&block.insts[k + 1..], term_calls)
                })
                .map(|(k, _)| k)
                .collect()
        })
        .collect()
}

/// The block the node must be pushed in front of: the closest one that every path to a use of the
/// node goes through and that runs at most once per call (the entry when there is no such block).
/// A procedure's early exits that touch nothing the node is for then skip the push and the pop's
/// effect. It is the entry for anything that is not plainly shaped.
fn push_block(func: &Func, line_stores: &[Vec<usize>]) -> usize {
    let blocks = &func.blocks;
    let (slot_addr, loaded) = context_values(func);
    let is_context = |v: Val| v == Val(0) || Some(v) == slot_addr || loaded.contains(&v);
    let needs_node = |b: usize| {
        let block = &blocks[b];
        if !line_stores[b].is_empty() {
            return true;
        }
        for inst in &block.insts {
            match inst {
                // Callees, and what reports a failure or the place it happened, can read the node.
                Inst::Call(_) => return true,
                Inst::Intrinsic(call)
                    if matches!(
                        call.op,
                        Intrinsic::Trap
                            | Intrinsic::BoundsCheck
                            | Intrinsic::CheckFailed
                            | Intrinsic::DebugBreak
                            | Intrinsic::ReturnAddress
                    ) =>
                {
                    return true;
                }
                // Keeping the parameter, or a value read back, in the slot.
                Inst::Store {
                    addr,
                    value,
                    ..
                } if Some(*addr) == slot_addr && Some(*value) != slot_addr => continue,
                Inst::Load {
                    addr, ..
                } if Some(*addr) == slot_addr => continue,
                _ => {}
            }
            let mut used = false;
            inst_vals(inst, &mut |_| {}, &mut |v| used |= is_context(v));
            if used {
                return true;
            }
        }
        let mut used = false;
        term_uses(&block.term, &mut |v| used |= is_context(v));
        used
    };

    // Reverse post-order of the blocks reachable from the entry.
    let count = blocks.len();
    let mut order = Vec::with_capacity(count);
    let mut seen = vec![false; count];
    let mut stack = vec![(0usize, 0usize)];
    seen[0] = true;
    while let Some((b, i)) = stack.pop() {
        let mut next = None;
        let mut k = 0;
        blocks[b].term.for_each_successor(|s| {
            if k == i {
                next = Some(s);
            }
            k += 1;
        });
        if let Some(next) = next {
            stack.push((b, i + 1));
            let next = next.0 as usize;
            if !seen[next] {
                seen[next] = true;
                stack.push((next, 0));
            }
        } else {
            order.push(b);
        }
    }
    order.reverse();
    let mut rank = vec![usize::MAX; count];
    for (r, &b) in order.iter().enumerate() {
        rank[b] = r;
    }
    // Predecessors in one flat array: `pred_start[b]..pred_start[b + 1]` indexes `pred_list`.
    let mut pred_start = vec![0usize; count + 1];
    for &b in &order {
        blocks[b]
            .term
            .for_each_successor(|s| pred_start[s.0 as usize + 1] += 1);
    }
    for b in 0..count {
        pred_start[b + 1] += pred_start[b];
    }
    let mut fill = pred_start.clone();
    let mut pred_list = vec![0usize; pred_start[count]];
    for &b in &order {
        blocks[b].term.for_each_successor(|s| {
            pred_list[fill[s.0 as usize]] = b;
            fill[s.0 as usize] += 1;
        });
    }
    // Immediate dominators.
    let mut idom = vec![usize::MAX; count];
    idom[0] = 0;
    let mut changed = true;
    while changed {
        changed = false;
        for &b in order.iter().skip(1) {
            let mut new = usize::MAX;
            for &p in &pred_list[pred_start[b]..pred_start[b + 1]] {
                if idom[p] == usize::MAX {
                    continue;
                }
                new = if new == usize::MAX {
                    p
                } else {
                    let (mut x, mut y) = (p, new);
                    while x != y {
                        while rank[x] > rank[y] {
                            x = idom[x];
                        }
                        while rank[y] > rank[x] {
                            y = idom[y];
                        }
                    }
                    x
                };
            }
            if new != idom[b] {
                idom[b] = new;
                changed = true;
            }
        }
    }
    let mut target = usize::MAX;
    for &b in &order {
        if !needs_node(b) {
            continue;
        }
        target = if target == usize::MAX {
            b
        } else {
            let (mut x, mut y) = (b, target);
            while x != y {
                while rank[x] > rank[y] {
                    x = idom[x];
                }
                while rank[y] > rank[x] {
                    y = idom[y];
                }
            }
            x
        };
    }
    if target == usize::MAX {
        return 0;
    }
    // A block inside a loop would push again on every pass.
    while target != 0 && on_cycle(blocks, target) {
        target = idom[target];
    }
    target
}

/// Whether `block` can be reached again after leaving it.
fn on_cycle(blocks: &[Block], block: usize) -> bool {
    let mut seen = vec![false; blocks.len()];
    let mut work: Vec<usize> = Vec::new();
    blocks[block]
        .term
        .for_each_successor(|s| work.push(s.0 as usize));
    while let Some(b) = work.pop() {
        if b == block {
            return true;
        }
        if !std::mem::replace(&mut seen[b], true) {
            blocks[b]
                .term
                .for_each_successor(|s| work.push(s.0 as usize));
        }
    }
    false
}

/// A statement's line matters only if it makes a call (the callee reads it from our node).
fn calls_before_next_loc(rest: &[Inst], successors_call: bool) -> bool {
    for inst in rest {
        match inst {
            Inst::Call(_) => return true,
            Inst::Loc {
                ..
            } => return false,
            _ => {}
        }
    }
    // The statement continues into the next blocks (checks, branches).
    successors_call
}

/// Per block: does a call happen from its start before any `Loc` on some path? (A statement's
/// line only matters if one is.)
fn reaches_call_from_entry(blocks: &[Block]) -> Vec<bool> {
    // 1: calls first, 2: a `Loc` first, 0: neither (the answer is the successors').
    let first: Vec<u8> = blocks
        .iter()
        .map(|b| {
            for inst in &b.insts {
                match inst {
                    Inst::Call(_) => return 1,
                    Inst::Loc {
                        ..
                    } => return 2,
                    _ => {}
                }
            }
            0
        })
        .collect();
    let mut result: Vec<bool> = first.iter().map(|&f| f == 1).collect();
    loop {
        let mut changed = false;
        for (b, block) in blocks.iter().enumerate() {
            let mut any = false;
            if first[b] == 0 && !result[b] {
                block
                    .term
                    .for_each_successor(|s| any |= result[s.0 as usize]);
            }
            if any {
                result[b] = true;
                changed = true;
            }
        }
        if !changed {
            return result;
        }
    }
}

fn retarget(term: &mut Term, from: BlockId, to: BlockId) {
    let fix = |b: &mut BlockId| {
        if *b == from {
            *b = to;
        }
    };
    match term {
        Term::Jump(b) => fix(b),
        Term::Branch {
            then_block,
            else_block,
            ..
        } => {
            fix(then_block);
            fix(else_block);
        }
        Term::Switch {
            cases,
            default,
            ..
        } => {
            for (_, b) in cases.iter_mut() {
                fix(b);
            }
            fix(default);
        }
        Term::Ret(_) | Term::Unreachable => {}
    }
}
