//! Draining queued bodies and lowering everything reachable from a root.
use super::model::{BodyState, ProcTarget};
use crate::sema::ProcState;
#[cfg(debug_assertions)]
use crate::sema::scope::ScopeKind;
use crate::sema::{Compiler, Diagnostic, ProcId, Result, ir};

impl Compiler {
    /// Lower all queued procedure bodies.
    /// A body that fails is retried while other bodies still lower: one lowered later may
    /// declare what it needs (`#insert,scope(...)` into a file). The first error of a pass
    /// that made no progress is reported. Only bodies that existed when the pass started
    /// count as progress: a failing body re-creates its nested procedures on every retry,
    /// and lowering those must not keep the loop alive.
    pub fn drain_bodies(&mut self) -> Result<()> {
        self.pr.unpark_bodies();
        loop {
            let mut failed = Vec::new();
            let mut progress = false;
            let existing = self.pr.procs.len();
            while let Some(id) = crate::sema::body_probe::next_body(&mut self.pr.body_queue) {
                if self.proc(id).body_state != BodyState::Queued {
                    continue;
                }
                match self.lower_body(id) {
                    Ok(()) => progress |= (id.0 as usize) < existing,
                    Err(e) => failed.push((id, e)),
                }
            }
            if failed.is_empty() {
                return Ok(());
            }
            if !progress {
                // Only the first is reported; an editor wants the others too.
                self.ed.errors(failed.iter().map(|(_, e)| &**e));
                return Err(failed.into_iter().next().unwrap().1);
            }
            self.pr
                .body_queue
                .extend(failed.into_iter().rev().map(|(id, _)| id));
        }
    }

    /// Lower the queued bodies that can be lowered now, for compile-time code running in the
    /// middle of checking. A body may need something still being computed (the layout of the
    /// struct whose constant is being evaluated); it stays queued, so the final `drain_bodies`
    /// reports its error. Returns the first such error.
    pub fn drain_bodies_lenient(&mut self) -> Option<Box<Diagnostic>> {
        let mut first: Option<Box<Diagnostic>>;
        let mut retry = Vec::new();
        if self.lower_epoch() != self.pr.parked_epoch
            || self
                .pr
                .parked_bodies
                .iter()
                .any(|&id| !self.failure_reusable(id, self.pr.parked_epoch))
        {
            self.pr.unpark_bodies();
        }
        // A body that fails is tried again while others still lower: one lowered later may
        // declare what it needs (`#insert,scope(...)` into a file), as `drain_bodies` does.
        loop {
            let existing = self.pr.procs.len();
            let mut progress = false;
            first = None;
            while let Some(id) = self.pr.body_queue.pop() {
                if self.proc(id).body_state != BodyState::Queued {
                    continue;
                }
                // A body that failed is only worth another attempt once something it could be
                // waiting for happened (code added, a compile-time run finished). Otherwise every
                // compile-time call redoes the failing work, which compounds when that work itself
                // runs compile-time code (a body using a module that did not load). Such bodies are
                // parked, so later calls in the same epoch skip them without looking.
                let epoch = self.lower_epoch();
                if let Some((at, err)) = self.pr.lenient_failures.get(&id)
                    && *at == epoch
                    && self.failure_reusable(id, epoch)
                {
                    first.get_or_insert_with(|| err.clone());
                    if epoch != self.pr.parked_epoch {
                        retry.append(&mut self.pr.parked_bodies);
                        self.pr.parked_epoch = epoch;
                    }
                    self.pr.parked_bodies.push(id);
                    continue;
                }
                let misses = (
                    self.epochs.placeholder_misses,
                    self.epochs.in_progress_misses,
                );
                match self.lower_body(id) {
                    Ok(()) => {
                        progress |= (id.0 as usize) < existing;
                        self.pr.lenient_failures.remove(&id);
                    }
                    Err(e) => {
                        // Waiting for a `#placeholder` is settled by the metaprogram, at any time;
                        // something in progress (a signature, a layout) by the caller finishing.
                        if (
                            self.epochs.placeholder_misses,
                            self.epochs.in_progress_misses,
                        ) == misses
                        {
                            let epoch = self.lower_epoch();
                            self.pr.lenient_failures.insert(id, (epoch, e.clone()));
                        }
                        first.get_or_insert(e);
                        retry.push(id);
                    }
                }
            }
            if retry.is_empty() || !progress {
                break;
            }
            self.pr.body_queue.append(&mut retry);
        }
        self.pr.body_queue.extend(retry);
        if first.is_none()
            && let Some(id) = self.pr.parked_bodies.last()
            && let Some((_, err)) = self.pr.lenient_failures.get(id)
        {
            first = Some(err.clone());
        }
        first
    }

    /// Counts the events after which a body that failed to lower may succeed.
    pub(super) fn lower_epoch(&self) -> u64 {
        let base =
            self.src.added + self.src.files.len() + self.ns.modules.len() + self.ct.runs_done;
        // Generated declarations/using entries can become visible without loading
        // a file or completing another run. IDE failure reuse must observe that
        // progress too. Body-local scopes are recreated by each lowering attempt;
        // their bindings cannot repair another procedure and are not progress.
        let namespaces = if self.ed.counts_scope_changes() {
            #[cfg(debug_assertions)]
            self.check_namespace_generation();
            self.src.sources.len() as u64 + self.epochs.namespace_generation
        } else {
            0
        };
        base as u64 + namespaces
    }

    /// Every change to what the scopes hold must move `namespace_generation`.
    #[cfg(debug_assertions)]
    fn check_namespace_generation(&self) {
        let total = self
            .ns
            .scopes
            .iter()
            .filter(|scope| {
                !matches!(
                    scope.kind,
                    ScopeKind::Proc | ScopeKind::Block | ScopeKind::Macro
                )
            })
            .map(|scope| {
                scope.names.values().map(Vec::len).sum::<usize>()
                    + scope.imports.len()
                    + scope.usings.len()
                    + scope.pending_done
            })
            .sum::<usize>();
        let (last_total, last_generation) = self.namespace_check.get();
        debug_assert!(
            total == last_total || self.epochs.namespace_generation != last_generation,
            "a scope changed without `namespace_changed`"
        );
        self.namespace_check
            .set((total, self.epochs.namespace_generation));
    }

    /// The procedures whose target is IR function `func`, lowest id first.
    pub(in crate::sema) fn procs_of_func(
        &self,
        func: ir::FuncId,
    ) -> impl Iterator<Item = ProcId> + '_ {
        self.pr
            .func_targets
            .get(&func)
            .into_iter()
            .flatten()
            .copied()
            .filter(
                move |&id| matches!(self.proc(id).target, Some(ProcTarget::Func(f)) if f == func),
            )
    }

    /// The procedure behind IR function `func` if its body is queued (not lowered yet).
    pub fn queued_proc_of(&self, func: ir::FuncId) -> Option<ProcId> {
        self.procs_of_func(func)
            .find(|&id| self.proc(id).body_state == BodyState::Queued)
    }

    /// Lower the queued bodies reachable from IR function `root` (calls, procedure addresses,
    /// procedure pointers in globals), and nothing else. Returns the first error.
    pub fn lower_reachable_from(&mut self, root: ir::FuncId) -> Option<Box<Diagnostic>> {
        self.lower_reachable_roots(vec![root], Vec::new())
    }

    /// `lower_reachable_from` for what a global's initializer points at.
    pub fn lower_reachable_from_global(&mut self, root: ir::GlobalId) -> Option<Box<Diagnostic>> {
        self.lower_reachable_roots(Vec::new(), vec![root.0])
    }

    fn lower_reachable_roots(
        &mut self,
        mut work: Vec<ir::FuncId>,
        mut global_work: Vec<u32>,
    ) -> Option<Box<Diagnostic>> {
        use crate::fxhash::HashSet;
        let mut first = None;
        let (mut funcs, mut globals) = (HashSet::default(), HashSet::default());
        loop {
            while let Some(g) = global_work.pop() {
                if !globals.insert(g) {
                    continue;
                }
                let global: &ir::Global = &self.out.program.globals[g as usize];
                for r in &global.relocs {
                    match r.target {
                        ir::RelocTarget::Func(f) => work.push(f),
                        ir::RelocTarget::Global(h) => global_work.push(h.0),
                        ir::RelocTarget::Foreign(_) => {}
                    }
                }
            }
            let Some(f) = work.pop() else {
                break;
            };
            if !funcs.insert(f) {
                continue;
            }
            if self
                .out
                .program
                .funcs
                .get(f.0 as usize)
                .is_none_or(Option::is_none)
                && let Some(p) = self.queued_proc_of(f)
                && !self.ed.is_signature_only(p)
                && let Err(e) = self.lower_body(p)
            {
                first.get_or_insert(e);
            }
            let Some(Some(func)) = self.out.program.funcs.get(f.0 as usize) else {
                continue;
            };
            for block in &func.blocks {
                for inst in &block.insts {
                    match inst {
                        ir::Inst::Call(call) => {
                            if let ir::Callee::Func(g) = call.callee {
                                work.push(g);
                            }
                        }
                        ir::Inst::FuncAddr {
                            func, ..
                        } => work.push(*func),
                        ir::Inst::GlobalAddr {
                            global, ..
                        } => global_work.push(global.0),
                        _ => {}
                    }
                }
            }
        }
        first
    }

    /// Lower `id` and then the bodies its lowering queued (its callees, transitively), but
    /// nothing queued before. Returns the first error.
    pub fn lower_with_callees(&mut self, id: ProcId) -> Option<Box<Diagnostic>> {
        let mark = self.pr.body_queue.len();
        if let Err(e) = self.lower_body(id) {
            return Some(e);
        }
        let mut first = None;
        let mut failed = Vec::new();
        while self.pr.body_queue.len() > mark {
            let q = self.pr.body_queue.pop().unwrap();
            if self.proc(q).body_state != BodyState::Queued {
                continue;
            }
            if let Err(e) = self.lower_body(q) {
                first.get_or_insert(e);
                failed.push(q);
            }
        }
        self.pr.body_queue.extend(failed);
        first
    }
}

impl ProcState {
    /// The errors of the bodies that failed their last lenient attempt.
    pub(crate) fn lenient_failures_list(&self) -> Vec<Diagnostic> {
        self.lenient_failures
            .values()
            .map(|(_, e)| (**e).clone())
            .collect()
    }

    /// Put the parked bodies back in the queue.
    fn unpark_bodies(&mut self) {
        let parked = std::mem::take(&mut self.parked_bodies);
        self.body_queue.extend(parked);
    }

    pub(super) fn set_proc_target(&mut self, id: ProcId, target: ProcTarget) {
        self.procs[id.0 as usize].target = Some(target);
        if let ProcTarget::Func(func) = target {
            let procs = self.func_targets.entry(func).or_default();
            if let Err(at) = procs.binary_search(&id) {
                procs.insert(at, id);
            }
        }
    }
}
