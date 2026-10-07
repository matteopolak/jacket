//! Green threads for WASI programs (`docs/native/wasm-threads.md`).
//!
//! WASI preview 1 has no threads, and wasm code cannot switch stacks, so Wasi_Runtime runs a
//! program's threads one at a time on the host's only thread
//! (`stdlib/Extensions/Wasi_Runtime/threads.jai`). A thread that blocks makes way by unwinding:
//! every procedure between the thread's entry and the blocking call returns to the scheduler and
//! leaves its frame behind. To resume the thread the scheduler calls its entry again, and each
//! procedure on the way down jumps back to the call it left from.
//!
//! [`instrument`] rewrites the procedures that may reach a blocking call so they can do that:
//!
//! - Their locals (`Slot`s) move from the native stack to a frame on the running thread's own
//!   frame stack (`Green_Control.top`), which outlives the unwinding.
//! - The calls that may block split the body into stretches. A value used outside the stretch
//!   that defines it is stored in the frame and loaded again where it is used, so a resumed
//!   procedure finds every value it still needs; constants and slot addresses are recomputed.
//! - After each call that may block, if the runtime is unwinding, the procedure records which
//!   call it was in the frame and returns. On entry, if the runtime is rewinding, it jumps
//!   straight to the recorded call, which makes the same call again with the stored arguments.
//!
//! Only programs that can start a thread (they reach Wasi_Runtime's `pthread_create`) are
//! rewritten; the others are left as they are and the runtime never unwinds.
use jaic::ir::{
    Block, BlockId, CallInst, Callee, CmpOp, Conv, ConvOp, Func, FuncId, GlobalId, Inst, Linkage,
    Program, RelocTarget, Sig, Term, Ty, Val,
};
use std::collections::{HashMap, HashSet};

/// Wasi_Runtime's `#program_export`s the rewrite relies on.
const SUSPEND: &str = "jaic_wasi_suspend";

const SCHEDULER: &str = "jaic_wasi_run_threads";
const CONTROL: &str = "jaic_wasi_green_control";
const EXHAUSTED: &str = "jaic_wasi_frames_exhausted";
const SPAWN: &str = "pthread_create";

/// `Green_Control` field offsets: `state` is at 0, then `top`, `limit` and `enabled`.
const TOP: u64 = 8;

const LIMIT: u64 = 16;
const ENABLED: u64 = 24;
const CONTROL_SIZE: u64 = 32;

/// `Green_Control.state` values.
const UNWINDING: u64 = 1;

const REWINDING: u64 = 2;

type SigKey = (Vec<Ty>, Vec<Ty>, Conv);

fn sig_key(sig: &Sig) -> SigKey {
    (sig.params.clone(), sig.returns.clone(), sig.conv)
}

/// What a call may reach: one procedure, or any whose address was taken and has the signature.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
enum Reach {
    Func(usize),
    Sig(usize),
}

/// The program with its blocking procedures rewritten, or `None` when nothing needs it: the
/// program cannot start a thread, or it has no Wasi_Runtime.
pub fn instrument(program: &Program) -> Option<Program> {
    let exports: HashMap<&str, usize> = program
        .funcs
        .iter()
        .enumerate()
        .filter_map(|(i, f)| match &f.as_ref()?.linkage {
            Linkage::Export(name) => Some((name.as_str(), i)),
            Linkage::Internal => None,
        })
        .collect();
    let suspend = *exports.get(SUSPEND)?;
    let scheduler = *exports.get(SCHEDULER)?;
    // `_start` calls the scheduler, or `main` directly when the program cannot start threads.
    let start = exports.get("_start").copied();
    let exhausted = *exports.get(EXHAUSTED)?;
    let spawn = *exports.get(SPAWN)?;
    let control = control_global(program.funcs[*exports.get(CONTROL)?].as_ref()?)?;
    if program.globals.get(control.0 as usize)?.size < CONTROL_SIZE {
        return None;
    }
    // A `#foreign` procedure that the program itself `#program_export`s is that procedure.
    let bound: Vec<Option<usize>> = program
        .foreigns
        .iter()
        .map(|f| {
            if f.is_data {
                None
            } else {
                exports.get(f.symbol.as_str()).copied()
            }
        })
        .collect();

    let n = program.funcs.len();
    let mut sigs: HashMap<SigKey, usize> = HashMap::new();
    let mut intern = |sig: &Sig| {
        let next = sigs.len();
        *sigs.entry(sig_key(sig)).or_insert(next)
    };
    let mut taken = vec![false; n];
    let mut reaches: Vec<Vec<Reach>> = vec![Vec::new(); n];
    for (i, func) in program.funcs.iter().enumerate() {
        let Some(func) = func else {
            continue;
        };
        let mut seen = HashSet::new();
        for inst in func.blocks.iter().flat_map(|b| &b.insts) {
            match inst {
                Inst::FuncAddr {
                    func, ..
                } => taken[func.0 as usize] = true,
                Inst::ForeignAddr {
                    foreign, ..
                } => {
                    if let Some(f) = bound[foreign.0 as usize] {
                        taken[f] = true;
                    }
                }
                Inst::Call(call) => {
                    let reach = match &call.callee {
                        Callee::Func(f) => Some(Reach::Func(f.0 as usize)),
                        Callee::Foreign(f) => bound[f.0 as usize].map(Reach::Func),
                        Callee::Indirect(_, sig) => Some(Reach::Sig(intern(sig))),
                    };
                    if let Some(reach) = reach
                        && seen.insert(reach)
                    {
                        reaches[i].push(reach);
                    }
                }
                _ => {}
            }
        }
    }
    for reloc in program.globals.iter().flat_map(|g| &g.relocs) {
        match reloc.target {
            RelocTarget::Func(f) => taken[f.0 as usize] = true,
            RelocTarget::Foreign(f) => {
                if let Some(f) = bound[f.0 as usize] {
                    taken[f] = true;
                }
            }
            RelocTarget::Global(_) => {}
        }
    }
    let spawns = taken[spawn] || reaches.iter().flatten().any(|&r| r == Reach::Func(spawn));
    if !spawns {
        return None;
    }
    let func_sig: Vec<Option<usize>> = program
        .funcs
        .iter()
        .map(|f| f.as_ref().map(|f| intern(&f.sig)))
        .collect();

    // Which procedures may block: the suspend primitive, and whatever may call one that may.
    // The scheduler is the bottom of every thread, where unwinding stops, and `_start` below it.
    let mut blocks = vec![false; n];
    blocks[suspend] = true;
    let mut blocking_sigs = vec![false; sigs.len()];
    loop {
        let mut changed = false;
        for f in 0..n {
            if blocks[f]
                && taken[f]
                && let Some(s) = func_sig[f]
                && !blocking_sigs[s]
            {
                blocking_sigs[s] = true;
                changed = true;
            }
        }
        for f in 0..n {
            if blocks[f] || f == scheduler || Some(f) == start || program.funcs[f].is_none() {
                continue;
            }
            let reaches_block = reaches[f].iter().any(|&r| match r {
                Reach::Func(g) => blocks[g],
                Reach::Sig(s) => blocking_sigs[s],
            });
            if reaches_block {
                blocks[f] = true;
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }

    let mut out = program.clone();
    let global = &mut out.globals[control.0 as usize];
    global
        .init
        .resize(global.init.len().max(CONTROL_SIZE as usize), 0);
    let enabled = ENABLED as usize;
    global.init[enabled..enabled + 8].copy_from_slice(&1u64.to_le_bytes());
    let may_block = |callee: &Callee| match callee {
        Callee::Func(f) => blocks[f.0 as usize],
        Callee::Foreign(f) => bound[f.0 as usize].is_some_and(|f| blocks[f]),
        Callee::Indirect(_, sig) => sigs.get(&sig_key(sig)).is_some_and(|&s| blocking_sigs[s]),
    };
    let names = Runtime {
        control,
        exhausted: FuncId(exhausted as u32),
    };
    for (f, func) in out.funcs.iter_mut().enumerate() {
        if blocks[f]
            && f != suspend
            && let Some(func) = func
        {
            rewrite(func, &may_block, &names);
        }
    }
    Some(out)
}

/// The global `jaic_wasi_green_control` returns the address of.
fn control_global(func: &Func) -> Option<GlobalId> {
    let mut found = None;
    for inst in func.blocks.iter().flat_map(|b| &b.insts) {
        if let Inst::GlobalAddr {
            global, ..
        } = inst
        {
            if found.is_some_and(|g| g != *global) {
                return None;
            }
            found = Some(*global);
        }
    }
    found
}

struct Runtime {
    control: GlobalId,
    exhausted: FuncId,
}

/// Values the rewritten body may use anywhere: computed once, in the new entry block.
fn hoistable(inst: &Inst) -> Option<Val> {
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
        } => Some(*dst),
        _ => None,
    }
}

fn is_blocking_call(inst: &Inst, may_block: &dyn Fn(&Callee) -> bool) -> bool {
    matches!(inst, Inst::Call(call) if may_block(&call.callee))
}

fn defs(inst: &Inst) -> Vec<Val> {
    match inst {
        Inst::IConst {
            dst, ..
        }
        | Inst::FConst {
            dst, ..
        }
        | Inst::Bin {
            dst, ..
        }
        | Inst::Un {
            dst, ..
        }
        | Inst::Cmp {
            dst, ..
        }
        | Inst::Conv {
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
        }
        | Inst::Load {
            dst, ..
        }
        | Inst::PtrAdd {
            dst, ..
        } => vec![*dst],
        Inst::Call(call) => call.results.clone(),
        Inst::Intrinsic(call) => call.results.clone(),
        Inst::Store {
            ..
        }
        | Inst::Copy {
            ..
        }
        | Inst::Zero {
            ..
        }
        | Inst::Loc {
            ..
        } => Vec::new(),
    }
}

fn uses_mut(inst: &mut Inst, f: &mut impl FnMut(&mut Val)) {
    match inst {
        Inst::Bin {
            a,
            b,
            ..
        }
        | Inst::Cmp {
            a,
            b,
            ..
        } => {
            f(a);
            f(b);
        }
        Inst::Un {
            a, ..
        } => f(a),
        Inst::Conv {
            src, ..
        } => f(src),
        Inst::Load {
            addr, ..
        } => f(addr),
        Inst::Store {
            addr,
            value,
            ..
        } => {
            f(addr);
            f(value);
        }
        Inst::PtrAdd {
            base,
            offset,
            ..
        } => {
            f(base);
            f(offset);
        }
        Inst::Copy {
            dst,
            src,
            ..
        } => {
            f(dst);
            f(src);
        }
        Inst::Zero {
            dst, ..
        } => f(dst),
        Inst::Call(call) => {
            if let Callee::Indirect(v, _) = &mut call.callee {
                f(v);
            }
            call.args.iter_mut().for_each(f);
        }
        Inst::Intrinsic(call) => call.args.iter_mut().for_each(f),
        Inst::IConst {
            ..
        }
        | Inst::FConst {
            ..
        }
        | Inst::SlotAddr {
            ..
        }
        | Inst::GlobalAddr {
            ..
        }
        | Inst::FuncAddr {
            ..
        }
        | Inst::ForeignAddr {
            ..
        }
        | Inst::Loc {
            ..
        } => {}
    }
}

fn term_uses_mut(term: &mut Term, f: &mut impl FnMut(&mut Val)) {
    match term {
        Term::Branch {
            cond, ..
        } => f(cond),
        Term::Switch {
            value, ..
        } => f(value),
        Term::Ret(values) => values.iter_mut().for_each(f),
        Term::Jump(_) | Term::Unreachable => {}
    }
}

fn shift_targets(term: &mut Term, by: u32) {
    match term {
        Term::Jump(t) => t.0 += by,
        Term::Branch {
            then_block,
            else_block,
            ..
        } => {
            then_block.0 += by;
            else_block.0 += by;
        }
        Term::Switch {
            cases,
            default,
            ..
        } => {
            for (_, t) in cases {
                t.0 += by;
            }
            default.0 += by;
        }
        Term::Ret(_) | Term::Unreachable => {}
    }
}

/// `def_stretch` value for a parameter: it is stored on a fresh entry and loaded wherever used.
const PARAM: u32 = u32::MAX;

/// For a hoisted value, defined in the new entry block.
const HOISTED: u32 = u32::MAX - 1;

/// For a value no reachable instruction defines.
const UNDEFINED: u32 = u32::MAX - 2;

/// Builds the rewritten body: block 0 is the new entry, original block `b` is `b + 1`.
struct Body {
    vals: Vec<Ty>,
    blocks: Vec<Block>,
}

impl Body {
    fn val(&mut self, ty: Ty) -> Val {
        self.vals.push(ty);
        Val(self.vals.len() as u32 - 1)
    }

    fn block(&mut self) -> usize {
        self.blocks.push(Block {
            insts: Vec::new(),
            term: Term::Unreachable,
        });
        self.blocks.len() - 1
    }

    fn konst(&mut self, insts: &mut Vec<Inst>, ty: Ty, value: u64) -> Val {
        let dst = self.val(ty);
        insts.push(match ty {
            Ty::F32 | Ty::F64 => Inst::FConst {
                dst,
                ty,
                value: 0.0,
            },
            _ => Inst::IConst {
                dst,
                ty,
                value,
            },
        });
        dst
    }

    fn at(&mut self, insts: &mut Vec<Inst>, base: Val, offset: u64) -> Val {
        let offset = self.konst(insts, Ty::I64, offset);
        let dst = self.val(Ty::Ptr);
        insts.push(Inst::PtrAdd {
            dst,
            base,
            offset,
        });
        dst
    }

    fn int_of(&mut self, insts: &mut Vec<Inst>, ptr: Val) -> Val {
        let dst = self.val(Ty::I64);
        insts.push(Inst::Conv {
            dst,
            op: ConvOp::Bitcast,
            from: Ty::Ptr,
            to: Ty::I64,
            src: ptr,
        });
        dst
    }
}

/// Rewrite one procedure that may block (see the module comment).
fn rewrite(func: &mut Func, may_block: &dyn Fn(&Callee) -> bool, rt: &Runtime) {
    let old = std::mem::take(&mut func.blocks);
    let params = func.sig.params.len();
    let nvals = func.vals.len();

    // Number the stretches: each block starts one, and so does each call that may block.
    let mut def_stretch = vec![UNDEFINED; nvals];
    def_stretch[..params].fill(PARAM);
    let mut hoisted = Vec::new();
    let mut stretch = 0u32;
    for block in &old {
        let mut current = stretch;
        stretch += 1;
        for inst in &block.insts {
            if is_blocking_call(inst, may_block) {
                current = stretch;
                stretch += 1;
            }
            if let Some(dst) = hoistable(inst) {
                def_stretch[dst.0 as usize] = HOISTED;
                hoisted.push(inst.clone());
                continue;
            }
            for d in defs(inst) {
                def_stretch[d.0 as usize] = current;
            }
        }
    }
    // A value used in another stretch than the one defining it lives in the frame.
    let mut spilled = vec![false; nvals];
    stretch = 0;
    let mut mark = |v: Val, current: u32| {
        let d = def_stretch[v.0 as usize];
        if d != current && d != HOISTED {
            spilled[v.0 as usize] = true;
        }
    };
    for block in &old {
        let mut current = stretch;
        stretch += 1;
        for inst in &block.insts {
            if is_blocking_call(inst, may_block) {
                current = stretch;
                stretch += 1;
            }
            uses_mut(&mut inst.clone(), &mut |v| mark(*v, current));
        }
        term_uses_mut(&mut block.term.clone(), &mut |v| mark(*v, current));
    }

    // Frame: the resume point, the slots, then the stored values.
    let mut align = 16u64;
    let mut size = 8u64;
    let slot_offsets: Vec<u64> = func
        .slots
        .iter()
        .map(|slot| {
            let a = slot.align.clamp(1, 1 << 16).next_power_of_two();
            align = align.max(a);
            size = size.next_multiple_of(a);
            let offset = size;
            size += slot.size.max(1);
            offset
        })
        .collect();
    size = size.next_multiple_of(8);
    let mut spill_offsets = vec![None; nvals];
    for (v, offset) in spill_offsets.iter_mut().enumerate() {
        if spilled[v] {
            *offset = Some(size);
            size += 8;
        }
    }
    let size = size.next_multiple_of(align);

    let mut body = Body {
        vals: std::mem::take(&mut func.vals),
        blocks: Vec::new(),
    };
    for _ in 0..=old.len() {
        body.block();
    }
    let exhausted = body.block();
    let push = body.block();
    let dispatch = body.block();
    let store_params = body.block();
    let lost = body.block();

    // Entry: take a frame from the thread's frame stack.
    let mut entry = Vec::new();
    let control = body.val(Ty::Ptr);
    entry.push(Inst::GlobalAddr {
        dst: control,
        global: rt.control,
    });
    let top_addr = body.at(&mut entry, control, TOP);
    let old_top = body.val(Ty::Ptr);
    entry.push(Inst::Load {
        dst: old_top,
        ty: Ty::Ptr,
        addr: top_addr,
    });
    let old_top_int = body.int_of(&mut entry, old_top);
    let round = body.konst(&mut entry, Ty::I64, align - 1);
    let rounded = body.val(Ty::I64);
    entry.push(Inst::Bin {
        dst: rounded,
        op: jaic::ir::BinOp::Add,
        ty: Ty::I64,
        a: old_top_int,
        b: round,
    });
    let mask = body.konst(&mut entry, Ty::I64, !(align - 1));
    let frame_int = body.val(Ty::I64);
    entry.push(Inst::Bin {
        dst: frame_int,
        op: jaic::ir::BinOp::And,
        ty: Ty::I64,
        a: rounded,
        b: mask,
    });
    let frame = body.val(Ty::Ptr);
    entry.push(Inst::Conv {
        dst: frame,
        op: ConvOp::Bitcast,
        from: Ty::I64,
        to: Ty::Ptr,
        src: frame_int,
    });
    let new_top = body.at(&mut entry, frame, size);
    let new_top_int = body.int_of(&mut entry, new_top);
    let limit_addr = body.at(&mut entry, control, LIMIT);
    let limit = body.val(Ty::Ptr);
    entry.push(Inst::Load {
        dst: limit,
        ty: Ty::Ptr,
        addr: limit_addr,
    });
    let limit_int = body.int_of(&mut entry, limit);
    let over = body.val(Ty::I8);
    entry.push(Inst::Cmp {
        dst: over,
        op: CmpOp::UGt,
        ty: Ty::I64,
        a: new_top_int,
        b: limit_int,
    });
    for inst in hoisted {
        match inst {
            Inst::SlotAddr {
                dst,
                slot,
            } => {
                let offset = body.konst(&mut entry, Ty::I64, slot_offsets[slot.0 as usize]);
                entry.push(Inst::PtrAdd {
                    dst,
                    base: frame,
                    offset,
                });
            }
            other => entry.push(other),
        }
    }
    let mut spill_addr = vec![None; nvals];
    for (v, offset) in spill_offsets.iter().enumerate() {
        if let Some(offset) = *offset {
            spill_addr[v] = Some(body.at(&mut entry, frame, offset));
        }
    }
    body.blocks[0] = Block {
        insts: entry,
        term: Term::Branch {
            cond: over,
            then_block: BlockId(exhausted as u32),
            else_block: BlockId(push as u32),
        },
    };
    body.blocks[exhausted]
        .insts
        .push(Inst::Call(Box::new(CallInst {
            results: Vec::new(),
            callee: Callee::Func(rt.exhausted),
            args: Vec::new(),
        })));

    // Claim the frame, then start over or resume.
    let mut insts = vec![Inst::Store {
        ty: Ty::Ptr,
        addr: top_addr,
        value: new_top,
    }];
    let state = body.val(Ty::I64);
    insts.push(Inst::Load {
        dst: state,
        ty: Ty::I64,
        addr: control,
    });
    let rewinding = body.konst(&mut insts, Ty::I64, REWINDING);
    let resuming = body.val(Ty::I8);
    insts.push(Inst::Cmp {
        dst: resuming,
        op: CmpOp::Eq,
        ty: Ty::I64,
        a: state,
        b: rewinding,
    });
    body.blocks[push] = Block {
        insts,
        term: Term::Branch {
            cond: resuming,
            then_block: BlockId(dispatch as u32),
            else_block: BlockId(store_params as u32),
        },
    };
    let mut insts = Vec::new();
    for p in 0..params {
        if let Some(addr) = spill_addr[p] {
            insts.push(Inst::Store {
                ty: body.vals[p],
                addr,
                value: Val(p as u32),
            });
        }
    }
    body.blocks[store_params] = Block {
        insts,
        term: Term::Jump(BlockId(1)),
    };

    // The original blocks, split after each call that may block.
    let mut resume_points: Vec<(u64, BlockId)> = Vec::new();
    let mut stretch = 0u32;
    for (b, block) in old.into_iter().enumerate() {
        let mut current = stretch;
        stretch += 1;
        let mut start = b + 1;
        let mut at = b + 1;
        let mut insts: Vec<Inst> = Vec::new();
        let mut loads: Vec<Inst> = Vec::new();
        let mut reloaded: HashMap<u32, Val> = HashMap::new();
        // Uses of values from other stretches read the frame, once per stretch, at its start.
        let fetch = |v: &mut Val,
                     current: u32,
                     body: &mut Body,
                     loads: &mut Vec<Inst>,
                     reloaded: &mut HashMap<u32, Val>| {
            let i = v.0 as usize;
            let Some(addr) = spill_addr[i] else {
                return;
            };
            if def_stretch[i] == current {
                return;
            }
            *v = *reloaded.entry(v.0).or_insert_with(|| {
                let ty = body.vals[i];
                let dst = body.val(ty);
                loads.push(Inst::Load {
                    dst,
                    ty,
                    addr,
                });
                dst
            });
        };
        let store = |v: Val, body: &Body, insts: &mut Vec<Inst>| {
            if let Some(addr) = spill_addr[v.0 as usize] {
                insts.push(Inst::Store {
                    ty: body.vals[v.0 as usize],
                    addr,
                    value: v,
                });
            }
        };
        for mut inst in block.insts {
            if hoistable(&inst).is_some() {
                continue;
            }
            if is_blocking_call(&inst, may_block) {
                // Close this stretch; the call starts the next one, in a block of its own that
                // a resumed procedure jumps to.
                let call_block = body.block();
                body.blocks[at] = Block {
                    insts: std::mem::take(&mut insts),
                    term: Term::Jump(BlockId(call_block as u32)),
                };
                body.blocks[start]
                    .insts
                    .splice(0..0, std::mem::take(&mut loads));
                reloaded.clear();
                current = stretch;
                stretch += 1;
                start = call_block;
                at = call_block;
                uses_mut(&mut inst, &mut |v| {
                    fetch(v, current, &mut body, &mut loads, &mut reloaded)
                });
                let results = defs(&inst);
                insts.push(inst);
                let point = resume_points.len() as u64 + 1;
                resume_points.push((point, BlockId(call_block as u32)));
                let state = body.val(Ty::I64);
                insts.push(Inst::Load {
                    dst: state,
                    ty: Ty::I64,
                    addr: control,
                });
                let unwinding_value = body.konst(&mut insts, Ty::I64, UNWINDING);
                let unwinding = body.val(Ty::I8);
                insts.push(Inst::Cmp {
                    dst: unwinding,
                    op: CmpOp::Eq,
                    ty: Ty::I64,
                    a: state,
                    b: unwinding_value,
                });
                // Unwinding: remember this call and return; the frame stays where it is.
                let leave = body.block();
                let mut leave_insts = Vec::new();
                let point_value = body.konst(&mut leave_insts, Ty::I64, point);
                leave_insts.push(Inst::Store {
                    ty: Ty::I64,
                    addr: frame,
                    value: point_value,
                });
                let returns = func.sig.returns.clone();
                let dummies = returns
                    .iter()
                    .map(|&ty| body.konst(&mut leave_insts, ty, 0))
                    .collect();
                body.blocks[leave] = Block {
                    insts: leave_insts,
                    term: Term::Ret(dummies),
                };
                let next = body.block();
                body.blocks[at] = Block {
                    insts: std::mem::take(&mut insts),
                    term: Term::Branch {
                        cond: unwinding,
                        then_block: BlockId(leave as u32),
                        else_block: BlockId(next as u32),
                    },
                };
                at = next;
                for r in results {
                    store(r, &body, &mut insts);
                }
                continue;
            }
            uses_mut(&mut inst, &mut |v| {
                fetch(v, current, &mut body, &mut loads, &mut reloaded)
            });
            let results = defs(&inst);
            insts.push(inst);
            for r in results {
                store(r, &body, &mut insts);
            }
        }
        let mut term = block.term;
        term_uses_mut(&mut term, &mut |v| {
            fetch(v, current, &mut body, &mut loads, &mut reloaded)
        });
        shift_targets(&mut term, 1);
        if matches!(term, Term::Ret(_)) {
            // Returning normally gives the frame back.
            insts.push(Inst::Store {
                ty: Ty::Ptr,
                addr: top_addr,
                value: old_top,
            });
        }
        body.blocks[at] = Block {
            insts,
            term,
        };
        body.blocks[start].insts.splice(0..0, loads);
    }
    let index = body.val(Ty::I64);
    body.blocks[dispatch] = Block {
        insts: vec![Inst::Load {
            dst: index,
            ty: Ty::I64,
            addr: frame,
        }],
        term: Term::Switch {
            value: index,
            ty: Ty::I64,
            cases: resume_points.iter().map(|&(p, b)| (p, b)).collect(),
            default: BlockId(lost as u32),
        },
    };
    func.blocks = body.blocks;
    func.vals = body.vals;
    func.slots.clear();
    if let Some(debug) = &mut func.debug {
        // The variables' addresses are no longer slots; lines and scopes stay.
        debug.vars.clear();
    }
}
