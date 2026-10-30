//! The dispatch loop that executes a procedure's ops, and the raw memory helpers it uses.
use super::super::{Frame, Interp, Res, Trap, TrapKind, bin_total, cmp_shifted, mask};
use super::op::{Code, NO_LOC, Op};
use crate::ir::{self, Term, Ty, UnOp};

/// How `run_ops` stopped.
enum Flow {
    /// The procedure returned this many results, at the start of its registers.
    Returned(u32),
    /// Whether to watch blocks changed: go on at this op with the other loop.
    Switch(usize),
}

impl Interp {
    /// The trap for a budget that ran out. It names the limit when each evaluation has one.
    #[cold]
    fn budget_trap<T>(&mut self) -> Res<T> {
        self.budget_traps += 1;
        let message = match self.eval_steps {
            Some(limit) => format!("execution budget exhausted ({limit} steps)"),
            None => "execution budget exhausted".to_string(),
        };
        self.trap_of(TrapKind::BudgetExhausted, message)
    }

    /// Does anything need to look at each basic block as it is entered?
    #[inline(always)]
    fn watches_blocks(&self) -> bool {
        self.multi || self.profile.is_some()
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

    /// One step of the execution budget: a loop's back-edge, a call or a switch. Every infinite
    /// run passes one, so the budget is charged there and not on every block. The common case
    /// is a decrement; every 1024th step (and an empty budget) takes `tick_slow`.
    #[inline(always)]
    fn tick(&mut self) -> Res<()> {
        match self.block_budget.as_mut() {
            Some(left) if *left & 1023 != 0 => {
                *left -= 1;
                Ok(())
            }
            Some(_) => self.tick_slow(),
            None => Ok(()),
        }
    }

    #[cold]
    #[inline(never)]
    fn tick_slow(&mut self) -> Res<()> {
        if let Some(left) = self.block_budget.as_mut() {
            if *left == 0 {
                match self.host.refill_budget() {
                    Some(more) => *left = more,
                    None => return self.budget_trap(),
                }
            }
            *left -= 1;
        }
        if self.host.stop_requested() {
            return self.trap_of(TrapKind::Stopped, "checking stopped");
        }
        Ok(())
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
    pub(in crate::interp) fn run_code(
        &mut self,
        program: &ir::Program,
        func: &ir::Func,
        frame: &Frame,
        stack_base: u64,
        regs: *mut u64,
        start: Option<usize>,
    ) -> Res<u32> {
        // A call is a step of the budget (recursion has no back-edge); a resumed run is not new.
        if start.is_none() {
            self.tick()?;
        }
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
    #[expect(
        clippy::too_many_lines,
        reason = "one match over every op keeps `pc` and the register accessors in locals; \
                  a call per op group would cost a spill and a call on the hottest path"
    )]
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
        // SAFETY: `build` checked that every register an op names lies in the frame's registers,
        // which `regs` points at.
        let get = |i: u32| unsafe { *regs.add(i as usize) };
        // SAFETY: as for `get`.
        let set = |i: u32, v: u64| unsafe { *regs.add(i as usize) = v };
        // SAFETY: `build` checked that every frame offset an op names lies in the frame's slots
        // at `stack_base`, which the interpreter stack keeps mapped while the frame runs.
        let load_frame = |ty: Ty, off: u32| unsafe { load_raw(ty, stack_base + u64::from(off)) };
        // SAFETY: as for `load_frame`.
        let store_frame =
            |ty: Ty, off: u32, v: u64| unsafe { store_raw(ty, stack_base + u64::from(off), v) };
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
                    // Where the block starts is looked up only when it is needed: a thread
                    // switch saves it, and a trap (budget, suspension) reports it.
                    if self.multi {
                        self.sync_loc(code, pc);
                    }
                    if let Err(trap) = self.enter_block(program, func, code, block as usize - 1) {
                        return Err(self.located(trap, code, pc));
                    }
                }
                entered = false;
            }
            // SAFETY: every block ends with an op that leaves it (a jump, a return...), so
            // `pc` stays inside `code.ops`.
            let op = unsafe { &*ops.add(pc) };
            let here = pc;
            pc += 1;
            // Taking a jump: a backward one is a loop's back-edge, which the budget charges.
            macro_rules! goto {
                ($target:expr) => {{
                    let target = $target as usize;
                    if target <= here
                        && let Err(trap) = self.tick()
                    {
                        return Err(self.located(trap, code, here));
                    }
                    pc = target;
                }};
            }
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
                Op::AddScaled {
                    dst,
                    a,
                    b,
                    scale,
                } => set(dst, get(a).wrapping_add(get(b).wrapping_mul(scale as u64))),
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
                } => set(dst, super::super::conv(op, from, to, get(src))),
                Op::GlobalAddr {
                    dst,
                    global,
                    off,
                } => set(
                    dst,
                    tri!(self.global_addr(program, global)).wrapping_add(off as i64 as u64),
                ),
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
                } => set(dst, load_frame(ty, off)),
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
                } => store_frame(ty, off, get(value)),
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
                } => store_frame(ty, off, imm as i64 as u64),
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
                } => set(dst, load_frame(Ty::I8, off)),
                Op::LoadFrame32 {
                    dst,
                    off,
                } => set(dst, load_frame(Ty::I32, off)),
                Op::LoadFrame64 {
                    dst,
                    off,
                } => set(dst, load_frame(Ty::I64, off)),
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
                } => store_frame(Ty::I8, off, get(value)),
                Op::StoreFrame32 {
                    off,
                    value,
                } => store_frame(Ty::I32, off, get(value)),
                Op::StoreFrame64 {
                    off,
                    value,
                } => store_frame(Ty::I64, off, get(value)),
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
                    if let Err(trap) = self.check_writable(d, size as usize) {
                        self.sync_loc(code, here);
                        return Err(trap.at(self.loc));
                    }
                    // SAFETY: both addresses are non-null (checked above) host addresses of program
                    // memory that the IR sized `size` bytes.
                    unsafe { copy_bytes(d, s, size) };
                }
                Op::MemCopy {
                    dst,
                    src,
                    len,
                } => {
                    let (d, s, n) = (get(dst), get(src), get(len));
                    if n > 0 {
                        if d < 4096 || s < 4096 {
                            self.sync_loc(code, here);
                            return self.null_trap("memcpy through a null pointer");
                        }
                        let Ok(n) = usize::try_from(n) else {
                            self.sync_loc(code, here);
                            return self.trap("memcpy length exceeds the host address space");
                        };
                        if let Err(trap) = self.check_writable(d, n) {
                            self.sync_loc(code, here);
                            return Err(trap.at(self.loc));
                        }
                        // Like a load or store, a copy goes through the program's pointers
                        // directly, so a bad one faults as it would natively.
                        // SAFETY: the program's own memmove.
                        unsafe { std::ptr::copy(s as *const u8, d as *mut u8, n) };
                    }
                }
                Op::MemSet {
                    dst,
                    value,
                    len,
                } => {
                    let (d, v, n) = (get(dst), get(value), get(len));
                    if n > 0 {
                        if d < 4096 {
                            self.sync_loc(code, here);
                            return self.null_trap("memset through a null pointer");
                        }
                        let Ok(n) = usize::try_from(n) else {
                            self.sync_loc(code, here);
                            return self.trap("memset length exceeds the host address space");
                        };
                        if let Err(trap) = self.check_writable(d, n) {
                            self.sync_loc(code, here);
                            return Err(trap.at(self.loc));
                        }
                        // SAFETY: the program's own memset, as for `MemCopy`.
                        unsafe { std::ptr::write_bytes(d as *mut u8, v as u8, n) };
                    }
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
                    if let Err(trap) = self.check_writable(d, size as usize) {
                        self.sync_loc(code, here);
                        return Err(trap.at(self.loc));
                    }
                    // SAFETY: `d` is non-null (checked above) and addresses `size` writable bytes of
                    // program memory.
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
                        if trap.suspends_frames() {
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
                        return self.check_trap(ir::CheckReason::Bounds, index, count);
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
                    if let Err(trap) =
                        self.exec_ir(program, inst, vals, frame, stack_base, &func.vals)
                    {
                        if trap.suspends_frames() {
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
                } => goto!(target),
                Op::Branch {
                    cond,
                    sense,
                    target,
                } => {
                    if (get(cond) & 0xff != 0) == sense {
                        goto!(target);
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
                        goto!(target);
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
                        goto!(target);
                    }
                }
                Op::BranchTest {
                    sense,
                    a,
                    mask,
                    target,
                } => {
                    if (get(a) & mask as i64 as u64 != 0) == sense {
                        goto!(target);
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
                        goto!(code.blocks[target as usize].start);
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
pub(in crate::interp) unsafe fn load_raw(ty: Ty, addr: u64) -> u64 {
    let p = addr as *const u8;
    // SAFETY: the caller checked that `addr` is readable for `ty`'s size.
    unsafe {
        match ty {
            Ty::I8 => *p as u64,
            Ty::I16 => std::ptr::read_unaligned(p.cast::<u16>()) as u64,
            Ty::I32 | Ty::F32 => std::ptr::read_unaligned(p.cast::<u32>()) as u64,
            Ty::I64 | Ty::F64 | Ty::Ptr => std::ptr::read_unaligned(p.cast::<u64>()),
            Ty::F80 | Ty::F128 => unreachable!("wide floats are never loaded into registers"),
        }
    }
}

/// Writes `ty` at `addr`, which is the caller's to have checked.
#[inline(always)]
pub(in crate::interp) unsafe fn store_raw(ty: Ty, addr: u64, v: u64) {
    let p = addr as *mut u8;
    // SAFETY: the caller checked that `addr` is writable for `ty`'s size.
    unsafe {
        match ty {
            Ty::I8 => *p = v as u8,
            Ty::I16 => std::ptr::write_unaligned(p.cast::<u16>(), v as u16),
            Ty::I32 | Ty::F32 => std::ptr::write_unaligned(p.cast::<u32>(), v as u32),
            Ty::I64 | Ty::F64 | Ty::Ptr => std::ptr::write_unaligned(p.cast::<u64>(), v),
            Ty::F80 | Ty::F128 => unreachable!("wide floats are never stored from registers"),
        }
    }
}

/// `memmove` without the library call for the small sizes most copies have.
unsafe fn copy_bytes(dst: u64, src: u64, size: u32) {
    let (d, s) = (dst as *mut u8, src as *const u8);
    // SAFETY: the caller checked that both addresses cover `size` bytes; `ptr::copy` handles
    // overlap and the unaligned accesses need no alignment.
    unsafe {
        match size {
            8 => {
                std::ptr::write_unaligned(
                    d.cast::<u64>(),
                    std::ptr::read_unaligned(s.cast::<u64>()),
                );
            }
            16 => {
                let v = std::ptr::read_unaligned(s.cast::<[u64; 2]>());
                std::ptr::write_unaligned(d.cast::<[u64; 2]>(), v);
            }
            4 => {
                std::ptr::write_unaligned(
                    d.cast::<u32>(),
                    std::ptr::read_unaligned(s.cast::<u32>()),
                );
            }
            _ => std::ptr::copy(s, d, size as usize),
        }
    }
}

unsafe fn zero_bytes(dst: u64, size: u32) {
    let d = dst as *mut u8;
    // SAFETY: the caller checked that `dst` is writable for `size` bytes.
    unsafe {
        match size {
            1 => *d = 0,
            2 => std::ptr::write_unaligned(d.cast::<u16>(), 0),
            4 => std::ptr::write_unaligned(d.cast::<u32>(), 0),
            8 => std::ptr::write_unaligned(d.cast::<u64>(), 0),
            16 => std::ptr::write_unaligned(d.cast::<[u64; 2]>(), [0; 2]),
            _ => std::ptr::write_bytes(d, 0, size as usize),
        }
    }
}
