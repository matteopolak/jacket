//! Functions a speculative body reserves: the first call of a procedure gives it an IR function
//! and queues its body. The body is handed provisional function ids that start after the
//! program's last one; its commit reserves the real ones, in the order the serial path would, and
//! rewrites the finished IR.
use super::{Shift, UNDO, abandon, active, defer};
use crate::ir;
use crate::sema::body_probe::W;
use crate::sema::procs::{BodyState, ProcTarget};
use crate::sema::{Compiler, ProcId};
use std::cell::RefCell;

/// A function a speculative body reserved.
pub(super) struct LocalFunc {
    name: String,
    sig: ir::Sig,
    /// The procedure it is the target of.
    proc: ProcId,
}

/// A foreign symbol a speculative body reserved, for the procedure `proc`.
pub(super) struct LocalForeign {
    foreign: ir::Foreign,
    proc: ProcId,
}

impl Reserved {
    pub(super) fn is_empty(&self) -> bool {
        self.funcs.is_empty() && self.foreigns.is_empty()
    }
}

/// Everything a body reserved in the program, with the table lengths it began with.
#[derive(Default)]
pub(super) struct Reserved {
    pub(super) funcs_base: usize,
    pub(super) funcs: Vec<LocalFunc>,
    pub(super) foreigns_base: usize,
    pub(super) foreigns: Vec<LocalForeign>,
}

thread_local! {
    static LOCAL_FOREIGNS: RefCell<(usize, Vec<LocalForeign>)> = const { RefCell::new((0, Vec::new())) };
    /// Length of the program's function table when the body began, and the functions it reserved.
    static LOCAL_FUNCS: RefCell<(usize, Vec<LocalFunc>)> = const { RefCell::new((0, Vec::new())) };
}

/// A body begins: its functions start after the `base` the program has now.
pub(super) fn begin(funcs: usize, foreigns: usize) {
    LOCAL_FUNCS.with(|f| *f.borrow_mut() = (funcs, Vec::new()));
    LOCAL_FOREIGNS.with(|f| *f.borrow_mut() = (foreigns, Vec::new()));
}

/// What the body reserved.
pub(super) fn take() -> Reserved {
    let (funcs_base, funcs) = LOCAL_FUNCS.with(|f| std::mem::take(&mut *f.borrow_mut()));
    let (foreigns_base, foreigns) = LOCAL_FOREIGNS.with(|f| std::mem::take(&mut *f.borrow_mut()));
    Reserved {
        funcs_base,
        funcs,
        foreigns_base,
        foreigns,
    }
}

/// Stop the speculative body being checked if `target` is one of its provisional functions: the
/// caller would keep the id somewhere that outlives the body.
pub(in crate::sema) fn refuse_provisional(target: ProcTarget, program: &ir::Program) {
    if !active() {
        return;
    }
    let provisional = match target {
        ProcTarget::Func(func) => func.0 as usize >= program.funcs.len(),
        ProcTarget::Foreign(foreign) => foreign.0 as usize >= program.foreigns.len(),
    };
    if provisional {
        abandon(W::OutProgram as usize);
    }
}

impl Shift {
    /// The function a provisional id of the body became.
    pub(super) fn func(&self, id: ir::FuncId) -> ir::FuncId {
        match (id.0 as usize).checked_sub(self.func_base) {
            Some(k) => self.funcs[k].0,
            None => id,
        }
    }

    /// Whether the commit reserved the body's `k`th function (nobody had given the procedure one).
    fn reserved(&self, k: usize) -> Option<ir::FuncId> {
        let (id, fresh) = self.funcs[k];
        fresh.then_some(id)
    }

    /// The foreign symbol a provisional id of the body became.
    pub(super) fn foreign(&self, id: ir::ForeignId) -> ir::ForeignId {
        match (id.0 as usize).checked_sub(self.foreign_base) {
            Some(k) => self.foreigns[k],
            None => id,
        }
    }
}

impl Compiler {
    /// The first call of `id` while speculating: give it a provisional function in place, queue its
    /// body for the commit. `None`: not speculating, or the procedure needs the serial path.
    pub(in crate::sema) fn provisional_target(
        &mut self,
        id: ProcId,
        name: String,
        sig: &ir::Sig,
    ) -> Option<ProcTarget> {
        if !active() || !self.target_is_resumable(id) {
            return None;
        }
        let (k, func) = LOCAL_FUNCS.with(|f| {
            let mut f = f.borrow_mut();
            let k = f.1.len();
            f.1.push(LocalFunc {
                name,
                sig: sig.clone(),
                proc: id,
            });
            (k, ir::FuncId((f.0 + k) as u32))
        });
        let target = ProcTarget::Func(func);
        let p = &mut self.pr.procs.quiet()[id.0 as usize];
        let before = p.body_state;
        p.body_state = BodyState::Queued;
        p.target = Some(target);
        UNDO.with(|u| {
            u.borrow_mut().push(Box::new(move |c| {
                let p = &mut c.pr.procs.quiet()[id.0 as usize];
                p.body_state = before;
                p.target = None;
            }));
        });
        defer(move |c, shift| {
            let Some(func) = shift.reserved(k) else {
                return;
            };
            c.pr.procs[id.0 as usize].body_state = BodyState::Queued;
            crate::sema::body_probe::enqueued(id.0);
            c.pr.body_queue.push(id);
            c.pr.set_proc_target(id, ProcTarget::Func(func));
            c.pr.func_procs.insert(func, id);
        });
        Some(target)
    }

    /// The first use of the foreign symbol `foreign` for `id` while speculating: a provisional
    /// id, in place. `None`: not speculating.
    pub(in crate::sema) fn provisional_foreign(
        &mut self,
        id: ProcId,
        foreign: ir::Foreign,
    ) -> Option<ProcTarget> {
        if !active() || self.pr.runtime_interpreter_assertion == Some(id) {
            return None;
        }
        let (k, handle) = LOCAL_FOREIGNS.with(|f| {
            let mut f = f.borrow_mut();
            let k = f.1.len();
            f.1.push(LocalForeign {
                foreign,
                proc: id,
            });
            (k, ir::ForeignId((f.0 + k) as u32))
        });
        let target = ProcTarget::Foreign(handle);
        self.pr.procs.quiet()[id.0 as usize].target = Some(target);
        UNDO.with(|u| {
            u.borrow_mut().push(Box::new(move |c| {
                c.pr.procs.quiet()[id.0 as usize].target = None;
            }));
        });
        defer(move |c, shift| {
            if shift.foreign_fresh[k] {
                c.pr.set_proc_target(id, ProcTarget::Foreign(shift.foreigns[k]));
            }
        });
        Some(target)
    }

    /// Reserve the foreign symbols of a committed body, in order, as `reserve_local_funcs` does.
    pub(super) fn reserve_local_foreigns(
        &mut self,
        foreigns: Vec<LocalForeign>,
    ) -> Vec<(ir::ForeignId, bool)> {
        foreigns
            .into_iter()
            .map(|f| match self.proc(f.proc).target {
                Some(ProcTarget::Foreign(known)) => (known, false),
                _ => (self.out.program.add_foreign(f.foreign), true),
            })
            .collect()
    }

    /// Reserve the functions of a committed body, in order. A procedure that has a function by
    /// now (an earlier body of the batch called it first) keeps it, and the body's call is
    /// pointed at it.
    pub(super) fn reserve_local_funcs(&mut self, funcs: Vec<LocalFunc>) -> Vec<(ir::FuncId, bool)> {
        funcs
            .into_iter()
            .map(|f| match self.proc(f.proc).target {
                Some(ProcTarget::Func(known)) => (known, false),
                _ => (self.out.program.reserve_func_with_sig(f.name, f.sig), true),
            })
            .collect()
    }
}

/// Point the calls and function addresses of a finished body at the functions that were
/// reserved for its provisional ones.
pub(super) fn remap_func_ids(func: &mut ir::Func, shift: &Shift) {
    for block in &mut func.blocks {
        for inst in &mut block.insts {
            match inst {
                ir::Inst::Call(call) => match &mut call.callee {
                    ir::Callee::Func(f) => *f = shift.func(*f),
                    ir::Callee::Foreign(f) => *f = shift.foreign(*f),
                    ir::Callee::Indirect(..) => {}
                },
                ir::Inst::FuncAddr {
                    func, ..
                } => *func = shift.func(*func),
                ir::Inst::ForeignAddr {
                    foreign, ..
                } => *foreign = shift.foreign(*foreign),
                _ => {}
            }
        }
    }
}
