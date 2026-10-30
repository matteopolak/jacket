//! Checking every body for an editor analysis, and noting branches that did or did not run.
use super::facts::{IdeRef, IdeWhat};
use super::member_notes::names_target;
use crate::ast;
use crate::diagnostics::Diagnostic;
use crate::sema::scope::{EntityId, EntityKind, EntityState, Resolved, ScopeKind};
use crate::sema::value::ProcId;
use crate::sema::{Compiler, Value};
use crate::source::Span;
use crate::types::TypeId;
use crate::types::TypeKind;

impl Compiler {
    /// After compiling for an editor: check every procedure body and top-level declaration of
    /// the recorded files, not just what the program reaches. Returns errors from declarations
    /// and independent statements (a body that fails does not stop the others, and
    /// a declaration another one depends on reports its own error once, at its own span).
    pub fn ide_check_all(&mut self) -> Vec<Diagnostic> {
        let mut errors: Vec<Diagnostic> = Vec::new();
        if let Some(ide) = self.ed.facts_mut() {
            errors.append(&mut ide.errors);
        } else {
            return errors;
        }
        // Normal compilation has ended (possibly before finish_program). The IDE
        // audit has a complete source snapshot: no metaprogram is still supplying
        // placeholders. Use the compiler's existing finalization rule so a failed
        // generator cannot stay speculative and recreate nested procedures forever.
        self.epochs.placeholders_final = true;
        for i in 0..self.ns.entities.len() {
            let id = EntityId(i as u32);
            let e = self.entity(id);
            let (scope, span) = (e.scope, e.span);
            if matches!(e.kind, EntityKind::Decl { .. })
                && matches!(e.state, EntityState::Unresolved)
                && self.ide_wants(span.file)
            {
                // A record constant may depend on type_of of an earlier field.
                // Match ordinary unreferenced checking: establish the containing
                // record's fields before resolving its member constants.
                let result = if let ScopeKind::Struct(t) = self.scope(scope).kind
                    && let TypeKind::Struct(s) = self.ty.types.kind(t).clone()
                {
                    self.layout_struct(s, span)
                        .and_then(|()| self.resolve_entity(id))
                } else {
                    self.resolve_entity(id)
                };
                if let Err(e) = result {
                    errors.push(*e);
                }
            }
        }
        // A struct with a member of an unknown type fails when it is laid out. Each struct is
        // tried once: a layout that fails re-evaluates its member types, and what that
        // creates (instances, bound constants) would otherwise be laid out in turn without end.
        let mut laid_out = crate::fxhash::HashSet::default();
        let mut unused_defaults = Vec::new();
        let mut i = 0;
        while i < self.ns.entities.len() {
            let id = EntityId(i as u32);
            i += 1;
            let e = self.entity(id);
            let span = e.span;
            let ty = match &e.state {
                EntityState::Done(Resolved::Const {
                    value: Value::Type(t),
                    ..
                }) => Some(*t),
                _ => None,
            };
            if let Some(t) = ty
                && let TypeKind::Struct(s) = *self.ty.types.kind(t)
                && self.ty.types.struct_info(s).poly_args.is_empty()
                && self.ide_wants(span.file)
                && laid_out.insert(s)
            {
                match self.layout_struct(s, span) {
                    Ok(()) => unused_defaults.push((t, span)),
                    Err(e) => errors.push(*e),
                }
            }
        }
        // Field defaults are evaluated when a value of the struct is first made, so a struct
        // nothing uses has none: evaluate them for their facts (`void` in `t: Type = void` is a
        // type either way), keeping quiet about their errors as compilation does.
        unused_defaults.retain(|(t, _)| !self.ty.default_images.contains_key(t));
        let quiet = (
            self.ed.facts().map_or(0, |ide| ide.errors.len()),
            self.diag.warnings.len(),
            self.diag.deferred_errors.len(),
        );
        for (t, span) in unused_defaults {
            let _ = self.default_initializer(t, span);
        }
        if let Some(ide) = self.ed.facts_mut() {
            ide.errors.truncate(quiet.0);
        }
        self.diag.warnings.truncate(quiet.1);
        self.diag.deferred_errors.truncate(quiet.2);
        let mut i = 0;
        while i < self.pr.procs.len() {
            let id = ProcId(i as u32);
            i += 1;
            // Header classification resolves names and can run compile-time code. Apply
            // the recording policy before starting independent audit work; actual calls
            // still resolve their selected dependencies through the normal checker.
            if !self.ide_wants(self.proc(id).span.file) {
                continue;
            }
            // `t: *Table` makes a procedure polymorphic, which is only known once names
            // resolve (a compile that stopped early may not have asked yet).
            let _ = self.refresh_implicit_poly(id);
            let p = self.proc(id);
            let span = p.span;
            if p.is_poly
                // A concrete instance is not a source declaration. Matching may
                // cache it before rejecting a candidate, or prefer another overload.
                // Actual uses queue their bodies through proc_func; auditing a
                // NotNeeded instance here would impose a body obligation on a call
                // that never selected it. Keep the cache for later selected uses.
                || p.bindings.is_some()
                || p.is_macro
                || p.lit.body.is_none()
                || p.body_state != crate::sema::procs::BodyState::NotNeeded
            {
                continue;
            }
            if let Err(e) = self.proc_func(id, span) {
                errors.push(*e);
            }
        }
        errors.extend(self.drain_bodies_lenient().map(|e| *e));
        self.ide_note_bodiless();
        let mut failed: Vec<_> = self
            .pr
            .lenient_failures_list()
            .into_iter()
            .filter(|e| self.ide_wants(e.span.file))
            .collect();
        failed.sort_by_key(|e| (e.span.file.0, e.span.start));
        errors.extend(failed);
        if let Some(ide) = self.ed.facts_mut() {
            errors.append(&mut ide.errors);
        }
        self.ide_note_never_returns();
        errors
    }

    /// Procedures without a body (`#foreign`, `#elsewhere`, `#compiler`): no body declares
    /// their parameters, so their names are recorded from the signature, as is the library
    /// `#foreign` names.
    fn ide_note_bodiless(&mut self) {
        for i in 0..self.pr.procs.len() {
            let id = ProcId(i as u32);
            if !self.ide_wants(self.proc(id).span.file) {
                continue;
            }
            let _ = self.refresh_implicit_poly(id);
            let p = self.proc(id);
            if p.is_poly || p.lit.body.is_some() {
                continue;
            }
            let (span, scope, header) = (p.span, p.scope, p.lit.header.clone());
            if !self.ide_wants(span.file) {
                continue;
            }
            if let Ok(sig) = self.signature(id, span) {
                for (param, sp) in header.params.iter().zip(&sig.params) {
                    if let Some(name) = param.name.filter(|n| n.span.end > n.span.start) {
                        self.ide_note_param_name(name.span, name.name, sp.ty);
                    }
                }
            }
            if let Some(lib) = header.foreign.as_ref().and_then(|f| f.library)
                && let Ok(ids) = self.lookup(scope, lib.name)
                && let [lib_id] = ids[..]
                && let Some(ide) = self.ed.facts_mut()
            {
                ide.refs.push(IdeRef {
                    span: lib.span,
                    what: IdeWhat::Entity(lib_id),
                    ty: TypeId::VOID,
                    decl: false,
                });
            }
        }
    }

    /// An `#if` took its then branch (`taken`) or its else branch: note whether each arm ran,
    /// for the editor to fade the ones no evaluation took, then `ide_note_skipped`. An arm
    /// spans its header too (`#if cond`, `else`), so a faded arm fades as a whole.
    pub(crate) fn ide_note_branches(&mut self, s: &ast::StaticIfStmt, taken: bool) {
        let (ran, faded, skipped) = if taken {
            (Some(s.then_arm), s.else_arm, &s.else_branch)
        } else {
            (s.else_arm, Some(s.then_arm), &s.then_branch)
        };
        for (arm, ran) in [(ran, true), (faded, false)] {
            if let Some(arm) = arm {
                self.ide_note_ran(arm, ran);
            }
        }
        self.ide_note_skipped(&s.cond, skipped);
    }

    /// The code at `span`, a branch of an `#if`, `if`, `ifx` or the body of a `while`, was
    /// checked as running (`ran`) or as skipped by a constant condition. Evaluations merge: a
    /// branch is faded only when none of them ran it.
    pub(crate) fn ide_note_ran(&mut self, span: Span, ran: bool) {
        if self.ed.is_live()
            && !self.ed.sketching()
            && self.ide_wants(span.file)
            && let Some(ide) = self.ed.facts_mut()
        {
            *ide.static_branches.entry(span).or_insert(false) |= ran;
        }
    }

    /// `#if cond` skipped `branch`: when `cond` asks about the target, record where, so the
    /// language server can ask a compile for another target about that code.
    pub(crate) fn ide_note_skipped(&mut self, cond: &ast::Expr, branch: &[ast::Stmt]) {
        let (Some(first), Some(last)) = (branch.first(), branch.last()) else {
            return;
        };
        if !self.ed.is_live() || !self.ide_wants(first.span.file) || !names_target(cond) {
            return;
        }
        let span = first.span.to(last.span);
        if let Some(ide) = self.ed.facts_mut() {
            ide.skipped.push(span);
        }
    }
}
