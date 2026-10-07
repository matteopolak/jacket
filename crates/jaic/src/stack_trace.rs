//! `context.stack_trace` for compiled output: an IR pass that makes every traced function push a
//! `Stack_Trace_Node` on entry and pop it on return. The interpreter maintains the list itself
//! (`Interp::trace_enter`); native backends get it from this pass instead.
//!
//! Nodes are `Stack_Trace_Node`s, laid out as `Program::stack_trace` says; `info` points at
//! a `Stack_Trace_Procedure_Info` global holding the procedure's name, declaration site and
//! address.

use crate::ir::*;
const HASH_SEED: u64 = 0xcbf2_9ce4_8422_2325;
const HASH_PRIME: u64 = 0x0100_0000_01b3;

/// Instrument every function that has `trace` info; the info is then cleared, so running the
/// result in the interpreter does not push a second node.
pub fn instrument(program: &mut Program, layout: &TraceLayout) {
    for i in 0..program.funcs.len() {
        let Some(info) = program.funcs[i].as_ref().and_then(|f| f.trace.clone()) else {
            continue;
        };
        let info_global = info_global(program, FuncId(i as u32), &info, &layout.info);
        let func = program.funcs[i].as_mut().unwrap();
        func.trace = None;
        instrument_func(func, FuncId(i as u32), &info, info_global, layout);
    }
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
    func.slots.push(Slot {
        size: n.size,
        align: 8,
    });
    let slot = SlotId(func.slots.len() as u32 - 1);
    let mut e = Emit {
        func,
    };

    // The original entry moves to a new block; the new entry pushes the node.
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

    let mut entry = Vec::new();
    let node = e.val(Ty::Ptr);
    entry.push(Inst::SlotAddr {
        dst: node,
        slot,
    });
    let top = e.offset(&mut entry, Val(0), layout.context);
    let previous = e.load(&mut entry, Ty::Ptr, top);
    let null = e.iconst(&mut entry, Ty::Ptr, 0);
    let is_first = e.val(Ty::I8);
    entry.push(Inst::Cmp {
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
    let line_addr = e.offset(&mut link, node, n.line_number);
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
    let link_id = e.block(link, Term::Jump(body_id));

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
    let depth = e.load(&mut nested, Ty::I32, depth_addr);
    let one = e.iconst(&mut nested, Ty::I32, 1);
    let depth = e.bin(&mut nested, BinOp::Add, Ty::I32, depth, one);
    e.store_at(&mut nested, Ty::I32, node, n.call_depth, depth);
    let hash_addr = e.offset(&mut nested, previous, n.hash);
    let hash = e.load(&mut nested, Ty::I64, hash_addr);
    let caller_line_addr = e.offset(&mut nested, previous, n.line_number);
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

    e.func.blocks[0] = Block {
        insts: entry,
        term: Term::Branch {
            cond: is_first,
            then_block: first_id,
            else_block: nested_id,
        },
    };

    // The original code (its entry now lives at `body_id`): record call lines, pop on return.
    let originals = (1..original_blocks).chain(std::iter::once(body_id.0 as usize));
    for b in originals.collect::<Vec<_>>() {
        let old = std::mem::take(&mut e.func.blocks[b].insts);
        let mut insts = Vec::with_capacity(old.len() + 4);
        for (k, inst) in old.iter().enumerate() {
            insts.push(inst.clone());
            if let Inst::Loc {
                line, ..
            } = inst
                && calls_before_next_loc(&old[k + 1..])
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

/// A statement's line matters only if it makes a call (the callee reads it from our node).
fn calls_before_next_loc(rest: &[Inst]) -> bool {
    for inst in rest {
        match inst {
            Inst::Call(_) => return true,
            Inst::Loc {
                ..
            } => return false,
            _ => {}
        }
    }
    // The statement continues into another block: assume it calls.
    true
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
