//! Writes a speculative body makes to shared tables, journaled so that they are undone when it
//! ends and replayed when it commits.
use super::{
    BASES, Cell, FIRST_USE, FirstUse, OPS, Shift, THUNK_KEYS, TOUCHED_PROCS, UNDO, W, abandon,
    active, defer, journal, local_globals_len,
};
use crate::sema::scope::EntityId;
use crate::sema::scope::EntityState;
use crate::sema::{Compiler, ProcId, ScopeId, Span};

impl Compiler {
    /// `local_consts[e] = (value, ty)` for a constant entity of the body being checked.
    pub(in crate::sema) fn set_local_const(
        &mut self,
        e: EntityId,
        value: crate::sema::Value,
        ty: crate::types::TypeId,
    ) {
        if active() {
            let kept = value.clone();
            journal(
                move |c| {
                    c.pr.local_consts.quiet().remove(&e);
                },
                move |c, shift| {
                    c.pr.local_consts.insert(shift.entity(e), (kept, ty));
                },
            );
            self.pr.local_consts.quiet().insert(e, (value, ty));
            return;
        }
        self.pr.local_consts.insert(e, (value, ty));
    }

    /// Move entity `id` to `state` while it is resolved. A speculative body that is the first to
    /// name a global entity resolves it in place; the entity is put back when the body ends, and
    /// when the body is committed it is resolved there too, to the same answer. A resolution that
    /// asked for a provisional global (`globals_before` is how many the body had asked for when
    /// it began) cannot be replayed: its answer may name that global.
    pub(in crate::sema) fn set_entity_state(
        &mut self,
        id: EntityId,
        state: EntityState,
        globals_before: usize,
    ) {
        if !active() || id.0 as usize >= BASES.with(Cell::get)[1] {
            self.entity_mut(id).state = state;
            return;
        }
        if matches!(state, EntityState::Done(_)) && local_globals_len() != globals_before {
            abandon(W::NsEntities as usize);
        }
        let slot = &mut self.ns.entities.quiet()[id.0 as usize].state;
        let previous = std::mem::replace(slot, state.clone());
        UNDO.with(|u| {
            u.borrow_mut().push(Box::new(move |c| {
                c.ns.entities.quiet()[id.0 as usize].state = previous;
            }));
        });
        if let EntityState::Done(resolved) = state {
            defer(move |c, _| {
                let slot = &mut c.ns.entities.quiet()[id.0 as usize].state;
                if matches!(slot, EntityState::Unresolved) {
                    *slot = EntityState::Done(resolved);
                }
            });
        }
    }

    /// Run `step`, the first run of `what`. A speculative body that stops inside it can have it
    /// run at its turn instead (`resolve_ahead`).
    pub(in crate::sema) fn first_use<R>(
        &mut self,
        what: FirstUse,
        step: impl FnOnce(&mut Self) -> R,
    ) -> R {
        if !active() || FIRST_USE.with(|f| f.get().is_some()) {
            return step(self);
        }
        let resumable = match what {
            FirstUse::Entity(e) => (e.0 as usize) < BASES.with(Cell::get)[1],
            FirstUse::Target(p) => self.target_is_resumable(p),
            FirstUse::Signature(_) => true,
        };
        if !resumable {
            return step(self);
        }
        FIRST_USE.with(|f| f.set(Some(what)));
        let result = step(self);
        FIRST_USE.with(|f| f.set(None));
        result
    }

    /// Whether `proc_func` for `id` can be run on its own, ahead of the body that asks for it:
    /// the plain case of a procedure with a body, whose only failures come before it changes
    /// anything.
    pub(super) fn target_is_resumable(&self, id: ProcId) -> bool {
        let p = self.proc(id);
        let header = &p.lit.header;
        p.lit.body.is_some()
            && header.foreign.is_none()
            && !header.flags.compiler()
            && !header.flags.intrinsic()
            && !crate::sema::procs::lit_is_entry_point(header)
            && self.pr.runtime_interpreter_assertion != Some(id)
    }

    /// The procedure `id`, to fill in the memo fields of its signature (`poly_checked`,
    /// `is_poly`, `sig_resolving`, `sig`). Under speculation the fields are changed in place for
    /// the rest of the body and restored when it ends, and the memo it ends with is left for the
    /// commit: a signature is a function of the declaration and of the types it names.
    pub(in crate::sema) fn proc_memo_mut(
        &mut self,
        id: ProcId,
    ) -> &mut crate::sema::procs::ProcInfo {
        if active() {
            let first = TOUCHED_PROCS.with(|t| {
                let mut t = t.borrow_mut();
                let first = !t.contains(&id);
                if first {
                    t.push(id);
                }
                first
            });
            if first {
                let p = &self.pr.procs[id.0 as usize];
                let before = (p.poly_checked, p.is_poly, p.sig_resolving, p.sig.clone());
                UNDO.with(|u| {
                    u.borrow_mut().push(Box::new(move |c| {
                        let p = &mut c.pr.procs.quiet()[id.0 as usize];
                        (p.poly_checked, p.is_poly, p.sig_resolving, p.sig) = before;
                    }));
                });
            }
            return &mut self.pr.procs.quiet()[id.0 as usize];
        }
        &mut self.pr.procs[id.0 as usize]
    }

    /// Queue what the body ended with for the procedures it filled in, for its commit.
    pub(super) fn leave_proc_memos(&mut self) {
        for id in TOUCHED_PROCS.with(|t| std::mem::take(&mut *t.borrow_mut())) {
            let p = &self.pr.procs[id.0 as usize];
            let memo = (p.poly_checked, p.is_poly, p.sig.clone());
            OPS.with(|o| {
                o.borrow_mut().push(Box::new(move |c, _| {
                    let p = &mut c.pr.procs[id.0 as usize];
                    if memo.0 && !p.poly_checked {
                        p.poly_checked = true;
                        p.is_poly = memo.1;
                    }
                    if p.sig.is_none() {
                        p.sig = memo.2;
                    }
                }));
            });
        }
    }

    /// The thunk scope `thunk` of `scope` was just made. Under speculation it is recorded for
    /// the body's commit; `false`: not speculating, the caller records it.
    pub(in crate::sema) fn note_thunk_scope(&mut self, scope: ScopeId, thunk: ScopeId) -> bool {
        if !active() {
            return false;
        }
        THUNK_KEYS.with(|t| t.borrow_mut().push(scope));
        UNDO.with(|u| {
            u.borrow_mut().push(Box::new(move |c| {
                c.ns.thunk_scopes.quiet().remove(&scope);
            }));
        });
        defer(move |c, shift| {
            c.ns.thunk_scopes
                .insert(shift.scope(scope), shift.scope(thunk));
        });
        self.ns.thunk_scopes.quiet().insert(scope, thunk);
        true
    }

    /// A macro parameter `e` holds a constant argument the macro never writes.
    pub(in crate::sema) fn set_const_macro_param(
        &mut self,
        e: EntityId,
        value: crate::sema::Value,
        ty: crate::types::TypeId,
    ) {
        if active() {
            let kept = value.clone();
            journal(
                move |c| {
                    c.pr.const_macro_params.quiet().remove(&e);
                },
                move |c, shift| {
                    c.pr.const_macro_params.insert(shift.entity(e), (kept, ty));
                },
            );
            self.pr.const_macro_params.quiet().insert(e, (value, ty));
            return;
        }
        self.pr.const_macro_params.insert(e, (value, ty));
    }

    /// Mark the parameter entity `e` as one whose value is discarded.
    pub(in crate::sema) fn set_discard_param(&mut self, e: EntityId) {
        if active() {
            journal(
                move |c| {
                    c.pr.discard_params.quiet().remove(&e);
                },
                move |c, shift| {
                    c.pr.discard_params.insert(shift.entity(e));
                },
            );
            self.pr.discard_params.quiet().insert(e);
            return;
        }
        self.pr.discard_params.insert(e);
    }

    /// Remember what a callable parameter or alias `e` was declared as.
    pub(in crate::sema) fn set_callable_entity(
        &mut self,
        e: EntityId,
        info: std::sync::Arc<crate::sema::ProcTypeParams>,
    ) {
        if active() {
            let kept = info.clone();
            journal(
                move |c| {
                    c.pr.callable_entities.quiet().remove(&e);
                },
                move |c, shift| {
                    c.pr.callable_entities.insert(shift.entity(e), kept);
                },
            );
            self.pr.callable_entities.quiet().insert(e, info);
            return;
        }
        self.pr.callable_entities.insert(e, info);
    }

    /// The `ifx` at `span` is an operand of a binary operator.
    pub(in crate::sema) fn set_ifx_operand(&mut self, span: Span) {
        if self.pr.ifx_operands.contains(&span) {
            return;
        }
        if active() {
            journal(
                move |c| {
                    c.pr.ifx_operands.quiet().remove(&span);
                },
                move |c, _| {
                    c.pr.ifx_operands.insert(span);
                },
            );
            self.pr.ifx_operands.quiet().insert(span);
            return;
        }
        self.pr.ifx_operands.insert(span);
    }

    /// The call at `span` was made to `instance` of a polymorphic procedure.
    pub(in crate::sema) fn note_call_instance(&mut self, span: Span, instance: ProcId) {
        let note = move |c: &mut Compiler, _: &Shift| {
            let made = c.meta.export.call_instances.entry(span).or_default();
            if !made.contains(&instance) {
                made.push(instance);
            }
        };
        if !defer(note) {
            let made = self.meta.export.call_instances.entry(span).or_default();
            if !made.contains(&instance) {
                made.push(instance);
            }
        }
    }
}
