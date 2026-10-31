//! Checking blocks and dispatching on the kind of statement.
use crate::ast::{ExprKind as E, StmtKind as S};
use crate::diagnostics::Code;
use crate::ide::Recorder;
use crate::ide::StmtRecovery;
use crate::ir::Ty;
use crate::sema::lower::{DeferAction, DeferEntry, FnCtx, Operand};
use crate::sema::scope::{EntityKind, ScopeKind};
use crate::sema::{Compiler, Diagnostic, Rc, Result, ScopeId, Span, ast, err, scope};

impl Compiler {
    /// Check statements in `scope` (no new scope is opened).
    pub fn check_block_stmts(
        &mut self,
        f: &mut FnCtx,
        scope: ScopeId,
        stmts: &[ast::Stmt],
    ) -> Result<()> {
        // `#import` in a body is visible to the whole module, also before the statement.
        let file_scope = self.file_scope_of(scope);
        self.hoist_body_imports(file_scope, stmts, false);
        // So is a `#load`: the code before it can use what the file declares.
        for stmt in stmts {
            if let S::Load {
                path,
                span,
            } = &stmt.kind
            {
                self.load_from_body(scope, path, *span)?;
            }
        }
        // Constants are visible throughout their block, also before their declaration.
        let mut failed = None;
        for stmt in stmts {
            if let S::Decl(decl) = &stmt.kind
                && decl.kind == ast::DeclKind::Const
                && !decl.backtick
                && f.hoisted_consts.insert((scope, decl.id))
                && let Err(error) = self.declare_local_consts(scope, scope, decl)
            {
                self.check_block_error_ide(f, &mut failed, error)?;
            }
        }
        for stmt in stmts {
            self.check_block_stmt_ide(f, scope, stmt, &mut failed)?;
        }
        failed.map_or(Ok(()), Err)
    }

    /// Keep checking independent statements for editor facts, but return a real error
    /// for the body afterward. Invalid IR must never become an executable procedure.
    fn check_block_stmt_ide(
        &mut self,
        f: &mut FnCtx,
        scope: ScopeId,
        stmt: &ast::Stmt,
        failed: &mut Option<Box<Diagnostic>>,
    ) -> Result<()> {
        if let Err(error) = self.check_block_stmt(f, scope, stmt) {
            match (
                self.ed
                    .stmt_failed(error.code, matches!(stmt.kind, S::Decl(_))),
                &stmt.kind,
            ) {
                (StmtRecovery::Propagate, _) => return Err(error),
                // Unknown values, not wrong ones: the statement's names stay declared (as
                // stand-ins) so the statements after it check without cascading errors.
                (StmtRecovery::Salvage, _) => self.sketch_salvage(f, scope, stmt),
                // A declaration whose initializer failed still declares its names (as the
                // declared type or a stand-in), so later uses do not report them unknown.
                (StmtRecovery::Declare, S::Decl(decl)) => self.ide_sketch_declare(f, scope, decl),
                _ => {}
            }
            self.check_block_error_ide(f, failed, error)?;
        }
        Ok(())
    }

    fn register_defer(&mut self, f: &mut FnCtx, mut entry: DeferEntry, caller_owned: bool) {
        if caller_owned && let Some(frame) = f.macros.last() {
            let at = frame.defer_depth.position();
            let caller = frame.caller_cleanup_serial;
            let active = f.b.entry_zeroed_slot(Ty::I8, 1, 1);
            let yes = f.b.iconst(Ty::I8, 1);
            f.b.store(Ty::I8, active, yes);
            entry.active = Some(active);
            f.insert_caller_defer(at, caller, entry);
        } else {
            f.defers.push(entry);
        }
    }

    fn check_block_error_ide(
        &mut self,
        f: &mut FnCtx,
        failed: &mut Option<Box<Diagnostic>>,
        error: Box<Diagnostic>,
    ) -> Result<()> {
        if !self.ed.continues_past_errors() {
            return Err(error);
        }
        self.ed.error(&error);
        if failed.is_none() {
            *failed = Some(error);
        }
        self.sketch_reopen(f);
        Ok(())
    }

    /// A statement of a block. A sketch goes on past a statement it cannot check, for the
    /// facts of the rest.
    fn check_block_stmt(&mut self, f: &mut FnCtx, scope: ScopeId, stmt: &ast::Stmt) -> Result<()> {
        match self.check_stmt(f, scope, stmt) {
            Err(_) if self.ed.sketching() => {
                self.sketch_reopen(f);
                self.sketch_salvage(f, scope, stmt);
                Ok(())
            }
            result => result,
        }
    }

    /// After a `return` (or a failure that ended the block), give a sketch an unreachable
    /// block to check the rest into.
    fn sketch_reopen(&mut self, f: &mut FnCtx) {
        if f.b.is_terminated() {
            let rest = f.b.new_block();
            f.b.switch_to(rest);
        }
    }

    /// A statement of a sketch failed: check what it did not get to, for the facts, and
    /// declare the names a declaration did not.
    fn sketch_salvage(&mut self, f: &mut FnCtx, scope: ScopeId, stmt: &ast::Stmt) {
        let checked = |c: &Self, e: &ast::Expr| c.ed.sketch_checked(e.span);
        // A condition that has a type was fine: the failure was in a body, which salvaged
        // its own statements.
        let typed = |c: &Self, e: &ast::Expr| c.ed.sketch_typed(e.span);
        let check = |c: &mut Self, f: &mut FnCtx, e: &ast::Expr| {
            if !checked(c, e) {
                let _ = c.check_expr(f, scope, e, None);
            }
        };
        match &stmt.kind {
            S::Decl(decl) => {
                decl.value
                    .iter()
                    .chain(decl.extra_values())
                    .for_each(|e| check(self, f, e));
                self.ide_sketch_declare(f, scope, decl);
            }
            S::Expr(e) => check(self, f, e),
            S::Assign {
                lhs,
                rhs,
                ..
            } => lhs.iter().chain(rhs).for_each(|e| check(self, f, e)),
            S::Return {
                values, ..
            } => values.iter().for_each(|a| check(self, f, &a.value)),
            S::If {
                cond,
                then_branch,
                else_branch,
            } if !typed(self, cond) => {
                check(self, f, cond);
                for branch in std::iter::once(then_branch).chain(else_branch) {
                    self.sketch_reopen(f);
                    let _ = self.check_scoped(f, scope, branch);
                }
            }
            S::While(w) if !typed(self, &w.cond) => {
                check(self, f, &w.cond);
                self.sketch_reopen(f);
                let _ = self.check_scoped(f, scope, &w.body);
            }
            _ => {}
        }
    }

    /// The address of the context a `push_context` installs.
    fn push_context_addr(
        &mut self,
        f: &mut FnCtx,
        scope: ScopeId,
        context: &ast::Expr,
        bare: bool,
        span: Span,
    ) -> Result<crate::ir::Val> {
        let ctx_ty = self.context_type(span)?;
        let op = if matches!(context.kind, E::Context) && f.context_binding().is_none() {
            // `push_context { }` in #c_call code: a fresh default context.
            let size = self.size_of(ctx_ty, span)?;
            let align = self.align_of(ctx_ty, span)?;
            let addr = f.b.alloca(size, align);
            self.init_default(f, ctx_ty, addr, span)?;
            Operand::Place {
                ty: ctx_ty,
                addr,
            }
        } else if bare {
            // No context named: push a copy of the current one, so what the block changes
            // stays inside it.
            let current = self.check_expr(f, scope, context, None)?;
            let size = self.size_of(ctx_ty, span)?;
            let align = self.align_of(ctx_ty, span)?;
            let addr = f.b.alloca(size, align);
            let (_, from) = self.address_of(f, current, span)?;
            f.b.copy(addr, from, size);
            Operand::Place {
                ty: ctx_ty,
                addr,
            }
        } else {
            self.check_expr(f, scope, context, None)?
        };
        let op = match op.ty() {
            t if t == ctx_ty => op,
            t if self.ty.types.pointee(t) == Some(ctx_ty) => {
                let (_, p) = self.rvalue(f, op, span)?;
                Operand::Place {
                    ty: ctx_ty,
                    addr: p,
                }
            }
            t => {
                return err(
                    Code::PushContext,
                    context.span,
                    format!(
                        "push_context needs a Context, found {}",
                        self.ty.types.name(t)
                    ),
                );
            }
        };
        let (_, addr) = self.address_of(f, op, span)?;
        Ok(addr)
    }

    /// Declare the names of a local constant declaration in `target` (and `scope`).
    pub(super) fn declare_local_consts(
        &mut self,
        target: ScopeId,
        scope: ScopeId,
        decl: &Rc<ast::Decl>,
    ) -> Result<()> {
        for (index, name) in decl.names.iter().enumerate() {
            if !decl.backtick {
                self.check_redeclared(target, name.name, name.span, Some(decl))?;
                self.reject_using_redeclaration(target, name.name, name.span)?;
            }
            for s in [target, scope] {
                self.add_entity(
                    s,
                    name.name,
                    name.span,
                    EntityKind::Decl {
                        decl: decl.clone(),
                        index,
                    },
                    false,
                );
                if target == scope {
                    break;
                }
            }
        }
        Ok(())
    }

    /// A `#if` condition inside a body: `#exists(name)` (possibly negated) looks in the
    /// macro's caller, which constant evaluation cannot see.
    fn body_static_condition(
        &mut self,
        f: &mut FnCtx,
        scope: ScopeId,
        cond: &ast::Expr,
    ) -> Result<bool> {
        use ast::ExprKind as E;
        match &cond.kind {
            E::Unary(ast::UnOp::Not, inner) if matches!(inner.kind, E::Exists(..)) => {
                Ok(!self.body_static_condition(f, scope, inner)?)
            }
            E::Exists(e, other) if matches!(e.kind, E::Backtick(_)) => {
                let type_only = std::mem::replace(&mut f.type_only, true);
                let other_checked = match other {
                    Some(other) => self.check_expr(f, scope, other, None).map(|_| ()),
                    None => Ok(()),
                };
                let exists = other_checked.is_ok() && self.check_expr(f, scope, e, None).is_ok();
                f.type_only = type_only;
                other_checked?;
                Ok(exists)
            }
            _ => self.eval_static_condition(scope, cond),
        }
    }

    /// A block scope for code written in the source at `covers`. The editor finds the names
    /// visible at a position through the scopes recorded this way, so every scope that holds
    /// declarations someone wrote is made here.
    pub(in crate::sema) fn new_block_scope(&mut self, parent: ScopeId, covers: Span) -> ScopeId {
        let inner = self.new_hidden_scope(parent);
        self.ide_scope_span(inner, covers);
        inner
    }

    /// A block scope with no source of its own: compiler temporaries, baked constants, code
    /// checked again where it is inserted or deferred to.
    pub(in crate::sema) fn new_hidden_scope(&mut self, parent: ScopeId) -> ScopeId {
        let module = self.scope(parent).module;
        self.new_scope(ScopeKind::Block, Some(parent), module, None)
    }

    /// Check a statement in a fresh block scope, running its defers at the end.
    pub(super) fn check_scoped(
        &mut self,
        f: &mut FnCtx,
        scope: ScopeId,
        stmt: &ast::Stmt,
    ) -> Result<()> {
        let inner = self.new_block_scope(scope, stmt.span);
        let depth = f.mark_defers();
        match &stmt.kind {
            S::Block(b) if b.checks_off.adds_to(f.checks_off) => {
                let saved = f.checks_off;
                f.checks_off = saved.with(b.checks_off);
                let result = self.check_block_stmts(f, inner, &b.stmts);
                f.checks_off = saved;
                result?;
            }
            S::Block(b) => self.check_block_stmts(f, inner, &b.stmts)?,
            _ => self.check_stmt(f, inner, stmt)?,
        }
        if !f.b.is_terminated() {
            self.emit_defers(f, depth.position(), stmt.span)?;
        }
        f.truncate_defers(depth.position());
        Ok(())
    }

    pub fn check_stmt(&mut self, f: &mut FnCtx, scope: ScopeId, stmt: &ast::Stmt) -> Result<()> {
        let span = stmt.span;
        if span.file == f.file && !f.b.is_terminated() {
            let (line, col) = self.src.sources.get(span.file).line_col(span.start);
            let debug_scope = self.debug_scope(f, scope, line, col);
            f.b.loc(span.file.0, line, col, debug_scope);
        }
        match &stmt.kind {
            S::Decl(decl) => self.check_local_decl(f, scope, decl),
            S::Case(_) => err(
                Code::ParseCaseOutsideSwitch,
                span,
                "`case` outside of a switch",
            ),
            S::Expr(ast::Expr {
                kind: E::Struct(lit),
                span: lit_span,
                ..
            }) => self.declare_anonymous_aggregate(f, scope, lit, *lit_span),
            S::Expr(e) => {
                self.pr.last_call_must = None;
                self.check_expr(f, scope, e, None)?;
                if matches!(e.kind, E::Call { .. }) {
                    self.check_must_used(e.span, 0)?;
                }
                Ok(())
            }
            S::Assign {
                op,
                lhs,
                rhs,
            } => {
                let result = self.check_assign(f, scope, *op, lhs, rhs, span);
                self.ide_note_assign(scope, lhs, span);
                result
            }
            S::Block(_) => self.check_scoped(f, scope, stmt),
            S::If {
                cond,
                then_branch,
                else_branch,
            } => self.check_if(f, scope, cond, then_branch, else_branch.as_deref()),
            S::Switch(boxed) => {
                let crate::ast::SwitchStmt {
                    value,
                    cases,
                    complete,
                    header,
                } = &**boxed;
                self.check_switch(f, scope, value, cases, *complete, (*header, span))
            }
            S::StaticSwitch(boxed) => {
                let crate::ast::SwitchStmt {
                    value,
                    cases,
                    ..
                } = &**boxed;
                match self.static_switch_case(scope, value, cases)? {
                    Some(case) => self.check_block_stmts(f, scope, &case.body),
                    None => Ok(()),
                }
            }
            S::Overlay(_) => err(
                Code::OverlayOnlyAllowed,
                span,
                "`#overlay` is only allowed in a struct body",
            ),
            S::PushContextDefer(defer) => self.check_push_context_defer(f, scope, defer, span),
            S::While(while_) => self.check_while(f, scope, while_),
            S::For(for_) => self.check_for(f, scope, for_, span),
            S::Break(label) => {
                if let Some(l) = label
                    && self.ed.is_live()
                {
                    self.ide_note_label(scope, *l);
                }
                let label = label.map(|l| l.name);
                if self.try_insert_replacement(f, label, |r| &r.break_, span)? {
                    return Ok(());
                }
                self.check_break(f, label, true, span)
            }
            S::Continue(label) => {
                if let Some(l) = label
                    && self.ed.is_live()
                {
                    self.ide_note_label(scope, *l);
                }
                let label = label.map(|l| l.name);
                if self.try_insert_replacement(f, label, |r| &r.continue_, span)? {
                    return Ok(());
                }
                self.check_break(f, label, false, span)
            }
            S::Remove(label) => {
                if let Some(l) = label
                    && self.ed.is_live()
                {
                    self.ide_note_label(scope, *l);
                }
                let label = label.map(|l| l.name);
                if self.try_insert_replacement(f, label, |r| &r.remove, span)? {
                    return Ok(());
                }
                self.check_remove(f, label, span)
            }
            S::Return {
                values,
                backtick,
            } => {
                let backtick = *backtick || f.caller_code_macros == Some(f.macros.len());
                f.macro_returned |= backtick;
                self.check_return(f, scope, values, backtick, span)
            }
            S::Defer {
                body,
                backtick,
            } => self.check_defer_stmt(f, scope, body, *backtick),
            S::Using(boxed) => {
                let crate::ast::UsingStmt {
                    value,
                    filter,
                } = &**boxed;
                self.check_using(f, scope, value, filter)
            }
            S::PushContext(boxed) => self.check_push_context(f, scope, boxed, span),
            S::StaticIf(boxed) => self.check_static_if_stmt(f, scope, boxed, span),
            S::Insert(boxed) => self.check_insert(f, scope, boxed, span),
            S::Assert(boxed) => self.check_static_assert_stmt(scope, boxed, span),
            S::Run(e) => {
                self.check_expr(f, scope, e, None)?;
                Ok(())
            }
            S::Import(import) => {
                // Unnamed imports are hoisted to the file scope by `check_block_stmts`.
                if let Some(name) = import.name {
                    self.add_entity(
                        scope,
                        name.name,
                        name.span,
                        EntityKind::Import(import.clone()),
                        false,
                    );
                }
                Ok(())
            }
            S::Load {
                path,
                span: load_span,
            } => self.load_from_body(scope, path, *load_span),
            S::AddContext(_) => err(
                Code::AddContext,
                span,
                "#add_context is only allowed at file scope",
            ),
            S::ModuleParameters(_) => err(
                Code::ModuleParametersOnlyAllowed,
                span,
                "#module_parameters is only allowed at file scope",
            ),
            S::Placeholder(_)
            | S::Scope(_)
            | S::Place(_)
            | S::Through
            | S::Directive(_)
            | S::Empty => Ok(()),
        }
    }

    /// `push_context ... defer`: switch the context now and restore it when the scope ends.
    fn check_push_context_defer(
        &mut self,
        f: &mut FnCtx,
        scope: ScopeId,
        defer: &crate::ast::PushContextDeferStmt,
        span: Span,
    ) -> Result<()> {
        let addr = self.push_context_addr(f, scope, &defer.context, defer.bare, span)?;
        let previous = f.current_context();
        let saved_slot = f.b.entry_zeroed_slot(Ty::Ptr, 8, 8);
        let saved = previous.unwrap_or_else(|| f.b.iconst(Ty::Ptr, 0));
        f.b.store(Ty::Ptr, saved_slot, saved);
        f.set_context(Some(addr));
        let entry = DeferEntry {
            action: DeferAction::RestoreContext {
                previous,
                saved_slot,
            },
            scope,
            active: None,
            caller_scope: None,
        };
        self.register_defer(f, entry, defer.backtick);
        Ok(())
    }

    /// `push_context ctx { ... }`: run the body with `ctx` as the context.
    fn check_push_context(
        &mut self,
        f: &mut FnCtx,
        scope: ScopeId,
        boxed: &crate::ast::PushContextStmt,
        span: Span,
    ) -> Result<()> {
        let crate::ast::PushContextStmt {
            context,
            body,
            bare,
            ..
        } = boxed;
        let depth = f.mark_defers();
        let addr = self.push_context_addr(f, scope, context, *bare, span)?;
        let saved = f.current_context();
        let saved_slot = f.b.entry_zeroed_slot(Ty::Ptr, 8, 8);
        let previous = saved.unwrap_or_else(|| f.b.iconst(Ty::Ptr, 0));
        f.b.store(Ty::Ptr, saved_slot, previous);
        f.set_context(Some(addr));
        f.defers.push(DeferEntry {
            action: DeferAction::RestoreContext {
                previous: saved,
                saved_slot,
            },
            active: None,
            scope,
            caller_scope: None,
        });
        let result = self.check_scoped(f, scope, body);
        if result.is_ok() && !f.b.is_terminated() {
            self.emit_defers(f, depth.position(), span)?;
        }
        f.truncate_defers(depth.position());
        result
    }

    /// `#if` inside a body: check the branch the condition selects.
    fn check_static_if_stmt(
        &mut self,
        f: &mut FnCtx,
        scope: ScopeId,
        boxed: &crate::ast::StaticIfStmt,
        span: Span,
    ) -> Result<()> {
        let crate::ast::StaticIfStmt {
            cond,
            then_branch,
            else_branch,
            ..
        } = boxed;
        if self.ed.sketching() {
            // Which branch a stand-in takes says nothing: a sketch checks both.
            let taken = self.body_static_condition(f, scope, cond).unwrap_or(true);
            let (first, second) = if taken {
                (then_branch, else_branch)
            } else {
                (else_branch, then_branch)
            };
            let _ = self.check_block_stmts(f, scope, first);
            self.sketch_reopen(f);
            return self.check_block_stmts(f, scope, second);
        }
        let taken = self.body_static_condition(f, scope, cond)?;
        f.static_taken.insert(span, taken);
        self.ide_note_branches(boxed, taken);
        self.check_block_stmts(
            f,
            scope,
            if taken {
                then_branch
            } else {
                else_branch
            },
        )
    }

    /// `#assert`: fail the compile when the condition does not hold.
    fn check_static_assert_stmt(
        &mut self,
        scope: ScopeId,
        boxed: &crate::ast::AssertStmt,
        span: Span,
    ) -> Result<()> {
        let crate::ast::AssertStmt {
            cond,
            message,
            args,
        } = boxed;
        // A sketch's stand-ins meet no condition.
        let holds = self.eval_static_condition_as(scope, cond, None);
        if self.ed.sketching() {
            return Ok(());
        }
        if !holds? {
            let msg = self.assert_message(scope, message.as_ref(), args)?;
            let failure = self.static_assert_failed(cond, span, msg);
            if self.src.import_trace.keeps_going() {
                return Ok(());
            }
            return Err(Box::new(failure));
        }
        Ok(())
    }

    /// `defer`: register the statement to run when the scope ends.
    fn check_defer_stmt(
        &mut self,
        f: &mut FnCtx,
        scope: ScopeId,
        body: &ast::Stmt,
        backtick: bool,
    ) -> Result<()> {
        let entry = DeferEntry {
            action: DeferAction::Statement(body.clone()),
            active: None,
            scope,
            caller_scope: f.macros.last().filter(|_| backtick).map(|m| m.caller_scope),
        };
        self.register_defer(f, entry, backtick);
        Ok(())
    }

    /// The case of `#if value == { case ...; }` that is compiled, if any.
    pub(in crate::sema) fn static_switch_case<'a>(
        &mut self,
        scope: ScopeId,
        value: &ast::Expr,
        cases: &'a [ast::Case],
    ) -> Result<Option<&'a ast::Case>> {
        let v = self.eval_const(scope, value, None)?;
        for case in cases {
            if case.values.is_empty() {
                return Ok(Some(case));
            }
            for cv in &case.values {
                let c = self.eval_const(scope, cv, Some(v.ty()))?;
                if self.const_equal(&v, &c) {
                    return Ok(Some(case));
                }
            }
        }
        Ok(None)
    }

    /// Add the unnamed (or `using`) `#import`s among `stmts` (and, when `nested`, inside their plain
    /// blocks and control flow, but not `#if` branches) to the module of `file_scope`, once each.
    pub(in crate::sema) fn hoist_body_imports(
        &mut self,
        file_scope: ScopeId,
        stmts: &[ast::Stmt],
        nested: bool,
    ) {
        for stmt in stmts {
            match &stmt.kind {
                // `using Name :: #import` brings the names in like an unnamed import.
                S::Import(import) if import.name.is_none() || import.using.is_some() => {
                    if self.src.hoisted_imports.insert(import.span) {
                        // An anonymous import belongs to the module's import environment,
                        // wherever it is written (as a top-level one does), so the module's
                        // other files see it too. Paths and parameters still resolve from
                        // the file that wrote it.
                        let module = self.scope(file_scope).module;
                        let target = self.ns.modules[module.0 as usize].scope;
                        self.push_import(
                            target,
                            scope::ImportEntry {
                                import: import.clone(),
                                module: None,
                                loading: false,
                                from_scope: file_scope,
                                filter: import.using.clone().unwrap_or(ast::UsingFilter::None),
                            },
                        );
                    }
                }
                _ if !nested => {}
                S::Block(block) => self.hoist_body_imports(file_scope, &block.stmts, true),
                S::If {
                    then_branch,
                    else_branch,
                    ..
                } => {
                    self.hoist_body_imports(file_scope, std::slice::from_ref(then_branch), true);
                    if let Some(e) = else_branch {
                        self.hoist_body_imports(file_scope, std::slice::from_ref(e), true);
                    }
                }
                S::While(w) => {
                    self.hoist_body_imports(file_scope, std::slice::from_ref(&w.body), true);
                }
                S::Defer {
                    body, ..
                } => self.hoist_body_imports(file_scope, std::slice::from_ref(body), true),
                S::For(f) => {
                    self.hoist_body_imports(file_scope, std::slice::from_ref(&f.body), true);
                }
                _ => {}
            }
        }
    }

    pub(in crate::sema) fn file_scope_of(&self, mut scope: ScopeId) -> ScopeId {
        loop {
            let s = self.scope(scope);
            if s.kind == ScopeKind::File {
                return scope;
            }
            match s.parent {
                Some(p) => scope = p,
                None => return scope,
            }
        }
    }
}
