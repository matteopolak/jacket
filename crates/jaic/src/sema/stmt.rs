//! Statement checking and lowering.
use super::calls::CallArg;
use super::lower::{
    DeferEntry, FnCtx, ForBody, InsertReplacements, LoopFrame, MacroFrame, Operand,
};
use super::scope::{EntityKind, Resolved, ScopeKind, UsingEntry};
use super::*;
use crate::ast::{ExprKind as E, StmtKind as S};
use crate::ir::{CmpOp, Ty};
use crate::types::{ArrayKind, TypeKind};

impl Compiler {
    /// Check statements in `scope` (no new scope is opened).
    pub fn check_block_stmts(
        &mut self,
        f: &mut FnCtx,
        scope: ScopeId,
        stmts: &[ast::Stmt],
    ) -> Result<()> {
        // `#import` in a body is visible to the whole file, also before the statement.
        let file_scope = self.file_scope_of(scope);
        self.hoist_body_imports(file_scope, stmts, false);
        // Constants are visible throughout their block, also before their declaration.
        for stmt in stmts {
            if let S::Decl(decl) = &stmt.kind
                && decl.kind == ast::DeclKind::Const
                && !decl.backtick
                && f.hoisted_consts.insert((scope, decl.id))
            {
                self.declare_local_consts(scope, scope, decl);
            }
        }
        for (i, stmt) in stmts.iter().enumerate() {
            if let S::PushContextDefer {
                context,
            } = &stmt.kind
            {
                // `push_context,defer_pop ctx;` holds for the rest of the block.
                let current = ast::Expr {
                    kind: E::Context,
                    span: stmt.span,
                };
                let addr = self.push_context_addr(
                    f,
                    scope,
                    context.as_ref().unwrap_or(&current),
                    stmt.span,
                )?;
                let saved = f.context.replace(addr);
                let result = self.check_block_stmts_from(f, scope, &stmts[i + 1..]);
                f.context = saved;
                return result;
            }
            self.check_stmt(f, scope, stmt)?;
        }
        Ok(())
    }

    /// The tail of a block: its constants and imports were already hoisted.
    fn check_block_stmts_from(
        &mut self,
        f: &mut FnCtx,
        scope: ScopeId,
        stmts: &[ast::Stmt],
    ) -> Result<()> {
        for (i, stmt) in stmts.iter().enumerate() {
            if matches!(stmt.kind, S::PushContextDefer { .. }) {
                return self.check_block_stmts(f, scope, &stmts[i..]);
            }
            self.check_stmt(f, scope, stmt)?;
        }
        Ok(())
    }


    /// The address of the context a `push_context` installs.
    fn push_context_addr(
        &mut self,
        f: &mut FnCtx,
        scope: ScopeId,
        context: &ast::Expr,
        span: Span,
    ) -> Result<crate::ir::Val> {
        let ctx_ty = self.context_type(span)?;
        let op = if matches!(context.kind, E::Context) && f.context.is_none() {
            // `push_context { }` in #c_call code: a fresh default context.
            let size = self.size_of(ctx_ty, span)?;
            let align = self.align_of(ctx_ty, span)?;
            let addr = f.b.alloca(size, align);
            self.init_default(f, ctx_ty, addr, span)?;
            Operand::Place {
                ty: ctx_ty,
                addr,
            }
        } else {
            self.check_expr(f, scope, context, None)?
        };
        let op = match op.ty() {
            t if t == ctx_ty => op,
            t if self.types.pointee(t) == Some(ctx_ty) => {
                let (_, p) = self.rvalue(f, op, span)?;
                Operand::Place {
                    ty: ctx_ty,
                    addr: p,
                }
            }
            t => {
                return err(
                    context.span,
                    format!("push_context needs a Context, found {}", self.types.name(t)),
                );
            }
        };
        let (_, addr) = self.address_of(f, op, span)?;
        Ok(addr)
    }

    /// Declare the names of a local constant declaration in `target` (and `scope`).
    fn declare_local_consts(&mut self, target: ScopeId, scope: ScopeId, decl: &Rc<ast::Decl>) {
        for (index, name) in decl.names.iter().enumerate() {
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
    }

    pub(super) fn new_block_scope(&mut self, parent: ScopeId) -> ScopeId {
        let module = self.scope(parent).module;
        self.new_scope(ScopeKind::Block, Some(parent), module, None)
    }

    /// Check a statement in a fresh block scope, running its defers at the end.
    fn check_scoped(&mut self, f: &mut FnCtx, scope: ScopeId, stmt: &ast::Stmt) -> Result<()> {
        let inner = self.new_block_scope(scope);
        self.ide_scope_span(inner, stmt.span);
        let depth = f.defers.len();
        match &stmt.kind {
            S::Block(b) if (b.no_abc && !f.no_abc) || (b.no_aoc && !f.no_aoc) => {
                let saved = (f.no_abc, f.no_aoc);
                f.no_abc |= b.no_abc;
                f.no_aoc |= b.no_aoc;
                let result = self.check_block_stmts(f, inner, &b.stmts);
                (f.no_abc, f.no_aoc) = saved;
                result?
            }
            S::Block(b) => self.check_block_stmts(f, inner, &b.stmts)?,
            _ => self.check_stmt(f, inner, stmt)?,
        }
        if !f.b.is_terminated() {
            self.emit_defers(f, depth, stmt.span)?;
        }
        f.defers.truncate(depth);
        Ok(())
    }

    pub fn check_stmt(&mut self, f: &mut FnCtx, scope: ScopeId, stmt: &ast::Stmt) -> Result<()> {
        let span = stmt.span;
        if span.file == f.file && !f.b.is_terminated() {
            let (line, col) = self.sources.get(span.file).line_col(span.start);
            f.b.loc(span.file.0, line, col);
        }
        match &stmt.kind {
            S::Decl(decl) => self.check_local_decl(f, scope, decl),
            S::Expr(e) => {
                self.last_call_must = None;
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
            } => self.check_assign(f, scope, *op, lhs, rhs, span),
            S::Block(_) => self.check_scoped(f, scope, stmt),
            S::If {
                cond,
                then_branch,
                else_branch,
            } => self.check_if(f, scope, cond, then_branch, else_branch.as_deref()),
            S::Switch {
                value,
                cases,
                ..
            } => self.check_switch(f, scope, value, cases, span),
            S::StaticSwitch {
                value,
                cases,
            } => match self.static_switch_case(scope, value, cases)? {
                Some(case) => self.check_block_stmts(f, scope, &case.body),
                None => Ok(()),
            },
            S::Overlay(_) => err(span, "#overlay is not supported"),
            S::PushContextDefer {
                ..
            } => err(
                span,
                "push_context,defer_pop must be a statement of a block",
            ),
            S::While {
                label,
                bind_label,
                cond,
                body,
            } => self.check_while(f, scope, label.map(|l| l.name), *bind_label, cond, body),
            S::For(for_) => self.check_for(f, scope, for_, span),
            S::Break(label) => {
                let label = label.map(|l| l.name);
                if self.try_insert_replacement(f, label, |r| &r.break_, span)? {
                    return Ok(());
                }
                self.check_break(f, label, true, span)
            }
            S::Continue(label) => {
                let label = label.map(|l| l.name);
                if self.try_insert_replacement(f, label, |r| &r.continue_, span)? {
                    return Ok(());
                }
                self.check_break(f, label, false, span)
            }
            S::Remove(label) => {
                let label = label.map(|l| l.name);
                if self.try_insert_replacement(f, label, |r| &r.remove, span)? {
                    return Ok(());
                }
                self.check_remove(f, label, span)
            }
            S::Return {
                values,
                backtick,
            } => self.check_return(f, scope, values, *backtick, span),
            S::Defer {
                body,
                backtick,
            } => {
                let entry = DeferEntry {
                    stmt: (**body).clone(),
                    scope,
                    caller_scope: f
                        .macros
                        .last()
                        .filter(|_| *backtick)
                        .map(|m| m.caller_scope),
                };
                if *backtick && let Some(frame) = f.macros.last_mut() {
                    // Runs when the macro caller's block exits.
                    let at = frame.defer_depth;
                    f.defers.insert(at, entry);
                    frame.defer_depth += 1;
                } else {
                    f.defers.push(entry);
                }
                Ok(())
            }
            S::Using {
                value, ..
            } => self.check_using(f, scope, value),
            S::PushContext {
                context,
                body,
            } => {
                let addr = self.push_context_addr(f, scope, context, span)?;
                let saved = f.context.replace(addr);
                let result = self.check_scoped(f, scope, body);
                f.context = saved;
                result
            }
            S::StaticIf {
                cond,
                then_branch,
                else_branch,
            } => {
                let taken = self.eval_static_condition(scope, cond)?;
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
            S::Insert {
                value,
                flags,
                scope: target,
                replacements,
            } => self.check_insert(f, scope, value, flags, target.as_ref(), replacements, span),
            S::Assert {
                cond,
                message,
            } => {
                if !self.eval_static_condition(scope, cond)? {
                    let msg = match message {
                        Some(m) => match self.eval_const_value(scope, m)? {
                            Value::String(s) => String::from_utf8_lossy(&s).into_owned(),
                            _ => String::new(),
                        },
                        None => String::new(),
                    };
                    return err(
                        span,
                        format!(
                            "#assert failed{}{msg}",
                            if msg.is_empty() {
                                ""
                            } else {
                                ": "
                            }
                        ),
                    );
                }
                Ok(())
            }
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
                ..
            } => err(span, "#load is only allowed at file scope"),
            S::AddContext(_) => err(span, "#add_context is only allowed at file scope"),
            S::ModuleParameters {
                ..
            } => err(span, "#module_parameters is only allowed at file scope"),
            S::Placeholder(_)
            | S::Scope(_)
            | S::Place(_)
            | S::Through
            | S::Directive {
                ..
            }
            | S::Empty => Ok(()),
        }
    }

    /// The case of `#if value == { case ...; }` that is compiled, if any.
    pub(super) fn static_switch_case<'a>(
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
    /// blocks and control flow, but not `#if` branches) to `file_scope`, once each.
    pub(super) fn hoist_body_imports(
        &mut self,
        file_scope: ScopeId,
        stmts: &[ast::Stmt],
        nested: bool,
    ) {
        for stmt in stmts {
            match &stmt.kind {
                // `using Name :: #import` brings the names in like an unnamed import.
                S::Import(import) if import.name.is_none() || import.using.is_some() => {
                    if self.hoisted_imports.insert(import.span) {
                        self.scope_mut(file_scope).imports.push(scope::ImportEntry {
                            import: import.clone(),
                            module: None,
                            loading: false,
                            from_scope: file_scope,
                            filter: import.using.clone().unwrap_or(ast::UsingFilter::None),
                        });
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
                S::While {
                    body, ..
                }
                | S::Defer {
                    body, ..
                } => self.hoist_body_imports(file_scope, std::slice::from_ref(body), true),
                S::For(f) => {
                    self.hoist_body_imports(file_scope, std::slice::from_ref(&f.body), true)
                }
                _ => {}
            }
        }
    }

    pub(super) fn file_scope_of(&self, mut scope: ScopeId) -> ScopeId {
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

    // -----------------------------------------------------------------------
    // Declarations and assignment
    // -----------------------------------------------------------------------

    fn check_local_decl(
        &mut self,
        f: &mut FnCtx,
        scope: ScopeId,
        decl: &Rc<ast::Decl>,
    ) -> Result<()> {
        let target = if decl.backtick {
            f.macros.last().map_or(scope, |m| m.caller_scope)
        } else {
            scope
        };
        if decl.kind == ast::DeclKind::Const {
            if !f.hoisted_consts.contains(&(scope, decl.id)) {
                self.declare_local_consts(target, scope, decl);
            }
            return Ok(());
        }
        let span = decl.span;
        let declared = match &decl.ty {
            Some(t) => Some(self.eval_type_in(f, scope, t)?),
            None => None,
        };
        // Evaluate the initializers before the names come into scope.
        let mut values = self.check_decl_values(f, scope, decl, declared)?;
        values.resize(decl.names.len(), None);
        let existing = |i: usize| decl.existing.get(i).copied().unwrap_or(false);
        for (i, (name, value)) in decl.names.iter().zip(values).enumerate() {
            if existing(i) {
                // `a=, b := ...` assigns to a variable that is already in scope.
                let Some(value) = value else {
                    return err(name.span, "assignment needs a value");
                };
                let place = self.check_ident(f, scope, name.name, name.span)?;
                let Operand::Place {
                    ty,
                    addr,
                } = place
                else {
                    return err(
                        name.span,
                        format!("cannot assign to {}", self.describe(&place)),
                    );
                };
                let value = self.convert(f, value, ty, name.span)?;
                let (_, v) = self.rvalue(f, value, name.span)?;
                self.store_value(f, ty, addr, v, name.span)?;
                continue;
            }
            let ty = match (declared, &value) {
                (Some(t), _) => t,
                (None, Some(op)) => {
                    let settled = self.settle_untyped(op.clone(), None);
                    match settled {
                        Operand::Procs(p) if p.len() == 1 => self.proc_type(p[0], span)?,
                        Operand::Type(_) => TypeId::TYPE,
                        Operand::Void => {
                            return err(
                                span,
                                "cannot declare a variable from an expression with no value",
                            );
                        }
                        // `p := null;` declares a `*void`.
                        ref o if o.ty() == TypeId::NULL => self.types.pointer(TypeId::VOID),
                        other => other.ty(),
                    }
                }
                (None, None) => return err(span, "declaration needs a type or a value"),
            };
            // Metaprograms read local declaration types from the exported syntax tree.
            self.local_decl_types.entry(decl.id).or_insert(ty);
            let size = self.size_of(ty, span)?;
            let mut align = self.align_of(ty, span)?;
            if let Some(a) = &decl.align {
                align = align.max(self.eval_int(scope, a)? as u64);
            }
            let addr = f.b.alloca(size.max(1), align);
            match value {
                Some(op) => {
                    let op = self.convert(f, op, ty, span)?;
                    let (_, v) = self.rvalue(f, op, span)?;
                    self.store_value(f, ty, addr, v, span)?;
                }
                None if decl.value.is_none() => self.init_default(f, ty, addr, span)?,
                None => {}
            }
            // `_` discards a value: it names no local (but `using _ := *x;` still uses).
            let discard = name.name.as_str() == "_";
            if discard && !decl.using {
                continue;
            }
            if !discard && self.declares_local(target, name.name) {
                return err(
                    name.span,
                    format!("'{}' is already declared in this scope", name.name),
                );
            }
            let depth = self.scope(target).proc_depth;
            let kind = EntityKind::Local {
                ty,
                addr,
                depth,
            };
            let e = self.add_entity(target, name.name, name.span, kind.clone(), false);
            if target != scope {
                // A backtick name is visible to the rest of the macro body too.
                self.add_entity(scope, name.name, name.span, kind, false);
            }
            if decl.using {
                self.scope_mut(target).usings.push(UsingEntry::Place {
                    ty,
                    entity: e,
                });
            }
        }
        Ok(())
    }

    /// Whether `scope` itself already holds a runtime local called `name`.
    fn declares_local(&self, scope: ScopeId, name: Sym) -> bool {
        self.scope(scope).names.get(&name).is_some_and(|ids| {
            ids.iter()
                .any(|&e| matches!(self.entity(e).kind, EntityKind::Local { .. }))
        })
    }

    /// The initializer operands of a declaration, one per name where the declaration gives
    /// enough values: `a, b := f()` splits a multi-value call, `a, b := 0` repeats the value,
    /// and `a, b := 1, 2` pairs them up.
    fn check_decl_values(
        &mut self,
        f: &mut FnCtx,
        scope: ScopeId,
        decl: &ast::Decl,
        declared: Option<TypeId>,
    ) -> Result<Vec<Option<Operand>>> {
        let names = decl.names.len();
        let Some(first) = &decl.value else {
            return Ok(vec![None; names]);
        };
        if matches!(first.kind, E::Uninit) {
            return Ok(vec![None; names]);
        }
        if !decl.extra_values.is_empty() {
            let exprs: Vec<&ast::Expr> = std::iter::once(first).chain(&decl.extra_values).collect();
            if exprs.len() != names {
                return err(
                    decl.span,
                    format!("expected {names} values, found {}", exprs.len()),
                );
            }
            let mut values = Vec::new();
            for e in exprs {
                let op = self.check_expr(f, scope, e, declared)?;
                values.push(Some(if decl.existing.is_empty() {
                    op
                } else {
                    // Snapshot, so assigning to an earlier name cannot change a later value.
                    let op = self.settle_untyped(op, declared);
                    let (ty, v) = self.rvalue(f, op, e.span)?;
                    let tmp = self.spill(f, ty, v, e.span)?;
                    let val = match self.ir_ty(ty) {
                        Some(t) => f.b.load(t, tmp),
                        None => tmp,
                    };
                    Operand::Value {
                        ty,
                        val,
                    }
                }));
            }
            return Ok(values);
        }
        self.last_call_must = None;
        let first_op = self.check_expr(f, scope, first, declared)?;
        if matches!(first.kind, E::Call { .. }) {
            self.check_must_used(first.span, names)?;
        }
        Ok(match first_op {
            Operand::Multi(vals) if names > 1 => vals
                .into_iter()
                .map(|(ty, val)| {
                    Some(Operand::Value {
                        ty,
                        val,
                    })
                })
                .collect(),
            // `a, b := 0;` gives every name the same value.
            op => vec![Some(op); names],
        })
    }

    /// `x[i] = v` / `x[i] op= v` through `operator []=` (and `operator []` to
    /// read the old value). Evaluation order: base, index, (get), value, set.
    fn try_index_assign(
        &mut self,
        f: &mut FnCtx,
        scope: ScopeId,
        op: ast::AssignOp,
        target: &ast::Expr,
        rhs: &ast::Expr,
        span: Span,
    ) -> Result<bool> {
        let E::Index(base, index) = &target.kind else {
            return Ok(false);
        };
        let Ok(probe) = self.check_expr_no_emit(scope, base) else {
            return Ok(false);
        };
        let bt = probe.ty();
        let st = self.types.pointee(bt).unwrap_or(bt);
        if self.types.as_struct(self.types.repr_struct(st)).is_none() {
            return Ok(false);
        }
        let mut setters = self.operator_candidates(scope, "[]=", &[bt])?;
        // Through a raw pointer only an operator taking that struct applies; otherwise
        // `p[i] = v` writes memory.
        if self.types.is_pointer(bt) {
            setters.retain(|&p| self.first_param_accepts(p, st));
        }
        if setters.is_empty() {
            return Ok(false);
        }
        let base_op = self.check_expr(f, scope, base, None)?;
        let ptr_ty = self.types.pointer(st);
        let ptr = if self.types.is_pointer(bt) {
            let (_, v) = self.rvalue(f, base_op, base.span)?;
            v
        } else {
            let (_, addr) = self.address_of(f, base_op, base.span)?;
            addr
        };
        let ptr_op = Operand::Value {
            ty: ptr_ty,
            val: ptr,
        };
        let index_op = self.check_expr(f, scope, index, None)?;
        let index_op = if index_op.is_const() {
            index_op
        } else {
            let (ty, val) = self.rvalue(f, index_op, index.span)?;
            Operand::Value {
                ty,
                val,
            }
        };
        let arg = |op: Operand| CallArg {
            name: None,
            spread: false,
            expr: None,
            op: Some(op),
            span,
            scope,
        };
        let value = match op {
            ast::AssignOp::Assign => self.check_expr(f, scope, rhs, None)?,
            ast::AssignOp::Op(bin) => {
                let getters = self.operator_candidates(scope, "[]", &[bt])?;
                let by_ptr = vec![arg(ptr_op.clone()), arg(index_op.clone())];
                let old = match self.try_call(f, scope, &getters, by_ptr, span) {
                    Ok(v) => v,
                    Err(_) => {
                        let place = Operand::Place {
                            ty: st,
                            addr: ptr,
                        };
                        let by_value = vec![arg(place), arg(index_op.clone())];
                        self.call_procs(f, scope, &getters, by_value, None, span)?
                    }
                };
                let (old_ty, old_val) = self.rvalue(f, old, span)?;
                let inner = self.new_block_scope(scope);
                let slot = self.spill(f, old_ty, old_val, span)?;
                let tmp = Sym::intern("\u{0}old");
                let depth = self.scope(inner).proc_depth;
                self.add_entity(
                    inner,
                    tmp,
                    span,
                    EntityKind::Local {
                        ty: old_ty,
                        addr: slot,
                        depth,
                    },
                    false,
                );
                let expr = ast::Expr {
                    kind: E::Binary(
                        bin,
                        Box::new(ast::Expr {
                            kind: E::Ident(tmp),
                            span: target.span,
                        }),
                        Box::new(rhs.clone()),
                    ),
                    span,
                };
                self.check_expr(f, inner, &expr, Some(old_ty))?
            }
        };
        let args = vec![arg(ptr_op), arg(index_op), arg(value)];
        self.call_procs(f, scope, &setters, args, None, span)?;
        Ok(true)
    }

    /// Can procedure `p`'s first parameter take a `ty` (or `*ty`)? Polymorphic patterns
    /// are matched; plain types compared after resolution.
    fn first_param_accepts(&mut self, p: ProcId, ty: TypeId) -> bool {
        let info = self.proc(p);
        let (scope, lit, is_poly) = (info.scope, info.lit.clone(), info.is_poly);
        let Some(param_ty) = lit.header.params.first().and_then(|p| p.ty.clone()) else {
            return false;
        };
        let pointer = self.types.pointer(ty);
        if procs::has_poly(&param_ty) || is_poly {
            [ty, pointer].into_iter().any(|t| {
                let mut bindings = Vec::new();
                self.match_pattern(&param_ty, t, &mut bindings, scope)
                    .is_ok()
            })
        } else {
            self.eval_type(scope, &param_ty)
                .is_ok_and(|t| t == ty || t == pointer)
        }
    }

    /// `a op= b` through a user-defined `operator op=` (called with `*a`, or `a` by value).
    fn try_operator_assign(
        &mut self,
        f: &mut FnCtx,
        scope: ScopeId,
        bin: ast::BinOp,
        target: &ast::Expr,
        rhs: &ast::Expr,
        span: Span,
    ) -> Result<bool> {
        let Ok(probe) = self.check_expr_no_emit(scope, target) else {
            return Ok(false);
        };
        if !self.overloadable(probe.ty()) {
            return Ok(false);
        }
        let text = format!("{}=", calls::binop_text(bin));
        let candidates = self.operator_candidates(scope, &text, &[probe.ty()])?;
        if candidates.is_empty() {
            return Ok(false);
        }
        let Operand::Place {
            ty,
            addr,
        } = self.check_expr(f, scope, target, None)?
        else {
            return err(target.span, "cannot assign to this expression");
        };
        let value = self.check_expr(f, scope, rhs, None)?;
        let arg = |op: Operand| CallArg {
            name: None,
            spread: false,
            expr: None,
            op: Some(op),
            span,
            scope,
        };
        let ptr = Operand::Value {
            ty: self.types.pointer(ty),
            val: addr,
        };
        let by_ptr = vec![arg(ptr), arg(value.clone())];
        if self.try_call(f, scope, &candidates, by_ptr, span).is_err() {
            let place = Operand::Place {
                ty,
                addr,
            };
            let by_value = vec![arg(place), arg(value)];
            self.call_procs(f, scope, &candidates, by_value, None, span)?;
        }
        Ok(true)
    }

    fn check_assign(
        &mut self,
        f: &mut FnCtx,
        scope: ScopeId,
        op: ast::AssignOp,
        lhs: &[ast::Expr],
        rhs: &[ast::Expr],
        span: Span,
    ) -> Result<()> {
        if lhs.len() == 1
            && rhs.len() == 1
            && self.try_index_assign(f, scope, op, &lhs[0], &rhs[0], span)?
        {
            return Ok(());
        }
        if let ast::AssignOp::Op(bin) = op {
            if lhs.len() > 1 && rhs.len() == 1 {
                // `a, b += 1;` applies the operation to every target.
                for target in lhs {
                    self.check_assign(f, scope, op, std::slice::from_ref(target), rhs, span)?;
                }
                return Ok(());
            }
            if lhs.len() != 1 || rhs.len() != 1 {
                return err(span, "compound assignment takes one target and one value");
            }
            if self.try_operator_assign(f, scope, bin, &lhs[0], &rhs[0], span)? {
                return Ok(());
            }
            // Evaluate the target once, then `*tmp = *tmp op rhs`.
            let place = self.check_expr(f, scope, &lhs[0], None)?;
            let Operand::Place {
                ty,
                addr,
            } = place
            else {
                return err(lhs[0].span, "cannot assign to this expression");
            };
            let inner = self.new_block_scope(scope);
            let ptr_ty = self.types.pointer(ty);
            let slot = self.spill(f, ptr_ty, addr, span)?;
            let tmp = Sym::intern("\u{0}lhs");
            let depth = self.scope(inner).proc_depth;
            self.add_entity(
                inner,
                tmp,
                span,
                EntityKind::Local {
                    ty: ptr_ty,
                    addr: slot,
                    depth,
                },
                false,
            );
            let target = ast::Expr {
                kind: E::Unary(
                    ast::UnOp::Deref,
                    Box::new(ast::Expr {
                        kind: E::Ident(tmp),
                        span: lhs[0].span,
                    }),
                ),
                span: lhs[0].span,
            };
            let value = ast::Expr {
                kind: E::Binary(bin, Box::new(target), Box::new(rhs[0].clone())),
                span,
            };
            let result = self.check_expr(f, inner, &value, Some(ty))?;
            let result = self.convert(f, result, ty, span)?;
            let (_, v) = self.rvalue(f, result, span)?;
            self.store_value(f, ty, addr, v, span)?;
            return Ok(());
        }
        if lhs.len() == 1 && rhs.len() == 1 {
            let place = self.check_expr(f, scope, &lhs[0], None)?;
            let Operand::Place {
                ty,
                addr,
            } = place
            else {
                return err(
                    lhs[0].span,
                    format!("cannot assign to {}", self.describe(&place)),
                );
            };
            let value = self.check_expr(f, scope, &rhs[0], Some(ty))?;
            let value = self.convert(f, value, ty, rhs[0].span)?;
            let (_, v) = self.rvalue(f, value, span)?;
            // Copy through a temporary when the source may overlap the target.
            if self.is_memory_type(ty) {
                let size = self.size_of(ty, span)?;
                let align = self.align_of(ty, span)?;
                let tmp = f.b.alloca(size.max(1), align);
                f.b.copy(tmp, v, size);
                f.b.copy(addr, tmp, size);
            } else {
                self.store_value(f, ty, addr, v, span)?;
            }
            return Ok(());
        }
        // Multiple assignment: evaluate every value first.
        let mut values: Vec<(TypeId, ir::Val)> = Vec::new();
        if rhs.len() == 1 {
            match self.check_expr(f, scope, &rhs[0], None)? {
                Operand::Multi(v) => values = v,
                other => {
                    // `a, b = value;` assigns the one value to every target.
                    if matches!(
                        other,
                        Operand::Value { .. } | Operand::Place { .. } | Operand::Const { .. }
                    ) {
                        for l in lhs {
                            let place = self.check_expr(f, scope, l, None)?;
                            let Operand::Place {
                                ty,
                                addr,
                            } = place
                            else {
                                return err(l.span, "cannot assign to this expression");
                            };
                            let op = self.convert(f, other.clone(), ty, rhs[0].span)?;
                            let (_, v) = self.rvalue(f, op, l.span)?;
                            self.store_value(f, ty, addr, v, l.span)?;
                        }
                        return Ok(());
                    }
                    return err(
                        rhs[0].span,
                        format!(
                            "expected {} values, found {}",
                            lhs.len(),
                            self.describe(&other)
                        ),
                    );
                }
            }
        } else {
            for (r, l) in rhs.iter().zip(lhs) {
                let op = self.check_expr(f, scope, r, None)?;
                // Untyped constants take the type of their own target (`v.x, v.y = 42, 108;`).
                let target = match &l.kind {
                    E::Ident(name) if name.as_str() == "_" => None,
                    _ => match self.check_expr(f, scope, l, None)? {
                        Operand::Place {
                            ty, ..
                        } => Some(ty),
                        _ => None,
                    },
                };
                let op = match (&op, target) {
                    (
                        Operand::Const {
                            untyped: true, ..
                        },
                        Some(ty),
                    ) => self.convert(f, op, ty, r.span)?,
                    _ => op,
                };
                let op = self.settle_untyped(op, None);
                let (ty, v) = self.rvalue(f, op, r.span)?;
                let tmp = self.spill(f, ty, v, r.span)?;
                let v = match self.ir_ty(ty) {
                    Some(t) => f.b.load(t, tmp),
                    None => tmp,
                };
                values.push((ty, v));
            }
        }
        if values.len() < lhs.len() {
            return err(
                span,
                format!("expected {} values, found {}", lhs.len(), values.len()),
            );
        }
        for (l, (vty, v)) in lhs.iter().zip(values) {
            if let E::Ident(name) = &l.kind
                && name.as_str() == "_"
            {
                continue;
            }
            let place = self.check_expr(f, scope, l, None)?;
            let Operand::Place {
                ty,
                addr,
            } = place
            else {
                return err(l.span, "cannot assign to this expression");
            };
            let op = self.convert(
                f,
                Operand::Value {
                    ty: vty,
                    val: v,
                },
                ty,
                l.span,
            )?;
            let (_, v) = self.rvalue(f, op, l.span)?;
            self.store_value(f, ty, addr, v, l.span)?;
        }
        Ok(())
    }

    fn check_using(&mut self, f: &mut FnCtx, scope: ScopeId, value: &ast::Expr) -> Result<()> {
        if let E::Ident(name) = &value.kind {
            let ids = self.lookup(scope, *name)?;
            if let Some(&id) = ids.first() {
                match self.entity(id).kind.clone() {
                    EntityKind::Local {
                        ty, ..
                    } => {
                        let target = self.types.pointee(ty).unwrap_or(ty);
                        let _ = target;
                        self.scope_mut(scope).usings.push(UsingEntry::Place {
                            ty,
                            entity: id,
                        });
                        return Ok(());
                    }
                    _ => {
                        if let Resolved::Global {
                            ty, ..
                        } = self.resolve_entity(id)?
                        {
                            self.scope_mut(scope).usings.push(UsingEntry::Place {
                                ty,
                                entity: id,
                            });
                            return Ok(());
                        }
                    }
                }
            }
        }
        let op = self.check_expr(f, scope, value, None)?;
        let entry = match op {
            Operand::Module(m) => UsingEntry::Module(m),
            Operand::Type(t) => UsingEntry::Type(t),
            op @ (Operand::Place {
                ..
            }
            | Operand::Value {
                ..
            }) => {
                // `using a.b;`: bind a hidden local pointing at the place (or holding the
                // pointer itself when `a.b` is one).
                let (ptr, ptr_value) = if self.types.pointee(op.ty()).is_some() {
                    let ty = op.ty();
                    let (_, v) = self.rvalue(f, op, value.span)?;
                    (ty, v)
                } else {
                    let ty = self.types.pointer(op.ty());
                    let (_, a) = self.address_of(f, op, value.span)?;
                    (ty, a)
                };
                let slot = self.spill(f, ptr, ptr_value, value.span)?;
                let depth = self.scope(scope).proc_depth;
                let e = self.add_entity(
                    scope,
                    Sym::intern("\u{0}using"),
                    value.span,
                    EntityKind::Local {
                        ty: ptr,
                        addr: slot,
                        depth,
                    },
                    false,
                );
                UsingEntry::Place {
                    ty: ptr,
                    entity: e,
                }
            }
            Operand::Const {
                ty, ..
            } if self.types.as_struct(ty).is_some() => {
                // `using Ice_Cream.{...};`: the literal lives in an anonymous local.
                let (ty, v) = self.rvalue(f, op, value.span)?;
                let size = self.size_of(ty, value.span)?;
                let align = self.align_of(ty, value.span)?;
                let addr = f.b.alloca(size.max(1), align);
                self.store_value(f, ty, addr, v, value.span)?;
                let ptr = self.types.pointer(ty);
                let slot = self.spill(f, ptr, addr, value.span)?;
                let depth = self.scope(scope).proc_depth;
                let e = self.add_entity(
                    scope,
                    Sym::intern("\u{0}using"),
                    value.span,
                    EntityKind::Local {
                        ty: ptr,
                        addr: slot,
                        depth,
                    },
                    false,
                );
                UsingEntry::Place {
                    ty: ptr,
                    entity: e,
                }
            }
            other => {
                return err(
                    value.span,
                    format!("cannot use 'using' on {}", self.describe(&other)),
                );
            }
        };
        self.scope_mut(scope).usings.push(entry);
        Ok(())
    }

    // -----------------------------------------------------------------------
    // Control flow
    // -----------------------------------------------------------------------

    fn const_truth(op: &Operand) -> Option<bool> {
        match op {
            Operand::Const {
                value: Value::Bool(b),
                ..
            } => Some(*b),
            Operand::Const {
                value: Value::Int(i),
                ..
            } => Some(*i != 0),
            Operand::Const {
                value: Value::Null,
                ..
            } => Some(false),
            _ => None,
        }
    }

    fn check_if(
        &mut self,
        f: &mut FnCtx,
        scope: ScopeId,
        cond: &ast::Expr,
        then_branch: &ast::Stmt,
        else_branch: Option<&ast::Stmt>,
    ) -> Result<()> {
        let c = self.check_expr(f, scope, cond, Some(TypeId::BOOL))?;
        if let Some(t) = Self::const_truth(&c) {
            if t {
                return self.check_scoped(f, scope, then_branch);
            }
            if let Some(e) = else_branch {
                return self.check_scoped(f, scope, e);
            }
            return Ok(());
        }
        let c = self.settle_untyped(c, None);
        let (ty, v) = self.rvalue(f, c, cond.span)?;
        let c = self.truthy(f, ty, v, cond.span)?;
        let then_block = f.b.new_block();
        let done = f.b.new_block();
        let else_block = if else_branch.is_some() {
            f.b.new_block()
        } else {
            done
        };
        f.b.branch(c, then_block, else_block);
        f.b.switch_to(then_block);
        self.check_scoped(f, scope, then_branch)?;
        f.b.jump(done);
        if let Some(e) = else_branch {
            f.b.switch_to(else_block);
            self.check_scoped(f, scope, e)?;
            f.b.jump(done);
        }
        f.b.switch_to(done);
        Ok(())
    }

    fn check_switch(
        &mut self,
        f: &mut FnCtx,
        scope: ScopeId,
        value: &ast::Expr,
        cases: &[ast::Case],
        span: Span,
    ) -> Result<()> {
        let v = self.check_expr(f, scope, value, None)?;
        // Constant switch (types, constants): pick the case at compile time.
        if v.is_const() {
            let mut matched = None;
            let mut default = None;
            'outer: for (i, case) in cases.iter().enumerate() {
                if case.values.is_empty() {
                    default = Some(i);
                    continue;
                }
                for cv in &case.values {
                    let c = self.check_expr(f, scope, cv, Some(v.ty()))?;
                    if !c.is_const() {
                        matched = None;
                        default = None;
                        break 'outer;
                    }
                    if self.const_equal(&v, &c) {
                        matched = Some(i);
                        break 'outer;
                    }
                }
            }
            if let Some(start) = matched.or(default) {
                let mut i = start;
                loop {
                    let inner = self.new_block_scope(scope);
                    self.ide_scope_span(inner, cases[i].span);
                    let depth = f.defers.len();
                    self.check_block_stmts(f, inner, &cases[i].body)?;
                    if !f.b.is_terminated() {
                        self.emit_defers(f, depth, span)?;
                    }
                    f.defers.truncate(depth);
                    if !cases[i].through || i + 1 >= cases.len() {
                        break;
                    }
                    i += 1;
                }
                return Ok(());
            }
            if cases.iter().all(|c| !c.values.is_empty()) {
                return Ok(());
            }
        }
        let v = self.settle_untyped(v, None);
        let vty = v.ty();
        let (_, val) = self.rvalue(f, v, value.span)?;
        let done = f.b.new_block();
        let bodies: Vec<ir::BlockId> = cases.iter().map(|_| f.b.new_block()).collect();
        let mut default = done;
        for (i, case) in cases.iter().enumerate() {
            if case.values.is_empty() {
                default = bodies[i];
                continue;
            }
            for cv in &case.values {
                let c = self.check_expr(f, scope, cv, Some(vty))?;
                let c = self.convert(f, c, vty, cv.span)?;
                let (_, cval) = self.rvalue(f, c, cv.span)?;
                let eq = self.runtime_equal(f, vty, val, cval, cv.span)?;
                let next = f.b.new_block();
                f.b.branch(eq, bodies[i], next);
                f.b.switch_to(next);
            }
        }
        f.b.jump(default);
        for (i, case) in cases.iter().enumerate() {
            f.b.switch_to(bodies[i]);
            let inner = self.new_block_scope(scope);
            self.ide_scope_span(inner, case.span);
            let depth = f.defers.len();
            self.check_block_stmts(f, inner, &case.body)?;
            if !f.b.is_terminated() {
                self.emit_defers(f, depth, span)?;
            }
            f.defers.truncate(depth);
            let next = if case.through && i + 1 < cases.len() {
                bodies[i + 1]
            } else {
                done
            };
            f.b.jump(next);
        }
        f.b.switch_to(done);
        Ok(())
    }

    fn const_equal(&self, a: &Operand, b: &Operand) -> bool {
        match (a.const_value(), b.const_value()) {
            (Some(Value::Int(x)), Some(Value::Float(y)))
            | (Some(Value::Float(y)), Some(Value::Int(x))) => x as f64 == y,
            (Some(x), Some(y)) => x == y,
            _ => false,
        }
    }

    /// Runtime `a == b` for switch cases.
    fn runtime_equal(
        &mut self,
        f: &mut FnCtx,
        ty: TypeId,
        a: ir::Val,
        b: ir::Val,
        span: Span,
    ) -> Result<ir::Val> {
        match self.ir_ty(ty) {
            Some(t) if t.is_float() => Ok(f.b.cmp(CmpOp::FEq, t, a, b)),
            Some(t) => Ok(f.b.cmp(CmpOp::Eq, t, a, b)),
            None if ty == TypeId::STRING => {
                let r = self.string_equal(f, a, b);
                Ok(r)
            }
            None => {
                let size = self.size_of(ty, span)?;
                let n = f.b.iconst(Ty::I64, size);
                let r =
                    f.b.intrinsic(ir::Intrinsic::Memcmp, vec![a, b, n], &[Ty::I16]);
                let zero = f.b.iconst(Ty::I16, 0);
                Ok(f.b.cmp(CmpOp::Eq, Ty::I16, r[0], zero))
            }
        }
    }

    fn string_equal(&mut self, f: &mut FnCtx, x: ir::Val, y: ir::Val) -> ir::Val {
        let result = f.b.alloca(1, 1);
        let lc = f.b.load(Ty::I64, x);
        let rc = f.b.load(Ty::I64, y);
        let same = f.b.cmp(CmpOp::Eq, Ty::I64, lc, rc);
        f.b.store(Ty::I8, result, same);
        let cmp_block = f.b.new_block();
        let done = f.b.new_block();
        f.b.branch(same, cmp_block, done);
        f.b.switch_to(cmp_block);
        let xp = f.b.ptr_offset(x, 8);
        let xd = f.b.load(Ty::Ptr, xp);
        let yp = f.b.ptr_offset(y, 8);
        let yd = f.b.load(Ty::Ptr, yp);
        let r =
            f.b.intrinsic(ir::Intrinsic::Memcmp, vec![xd, yd, lc], &[Ty::I16]);
        let zero = f.b.iconst(Ty::I16, 0);
        let eq = f.b.cmp(CmpOp::Eq, Ty::I16, r[0], zero);
        f.b.store(Ty::I8, result, eq);
        f.b.jump(done);
        f.b.switch_to(done);
        f.b.load(Ty::I8, result)
    }

    fn check_while(
        &mut self,
        f: &mut FnCtx,
        scope: ScopeId,
        label: Option<Sym>,
        bind_label: bool,
        cond: &ast::Expr,
        body: &ast::Stmt,
    ) -> Result<()> {
        let head = f.b.new_block();
        let body_block = f.b.new_block();
        let exit = f.b.new_block();
        f.b.jump(head);
        f.b.switch_to(head);
        let (scope, c) = match label.filter(|_| bind_label) {
            Some(name) => {
                // `while s := next()`: the value is a local of the loop, besides being its label.
                let op = self.check_expr(f, scope, cond, Some(TypeId::BOOL))?;
                let op = self.settle_untyped(op, None);
                let (ty, v) = self.rvalue(f, op, cond.span)?;
                let size = self.size_of(ty, cond.span)?;
                let align = self.align_of(ty, cond.span)?;
                let addr = f.b.alloca(size.max(1), align);
                self.store_value(f, ty, addr, v, cond.span)?;
                let inner = self.new_block_scope(scope);
                self.ide_scope_span(inner, cond.span.to(body.span));
                let depth = self.scope(inner).proc_depth;
                self.add_entity(
                    inner,
                    name,
                    cond.span,
                    EntityKind::Local {
                        ty,
                        addr,
                        depth,
                    },
                    false,
                );
                (inner, self.truthy(f, ty, v, cond.span)?)
            }
            None => (scope, self.check_condition(f, scope, cond)?),
        };
        f.b.branch(c, body_block, exit);
        f.b.switch_to(body_block);
        f.loops.push(LoopFrame {
            label,
            break_block: exit,
            continue_block: head,
            defer_depth: f.defers.len(),
            remove: None,
        });
        let result = self.check_scoped(f, scope, body);
        f.loops.pop();
        result?;
        f.b.jump(head);
        f.b.switch_to(exit);
        Ok(())
    }

    fn check_break(
        &mut self,
        f: &mut FnCtx,
        label: Option<Sym>,
        is_break: bool,
        span: Span,
    ) -> Result<()> {
        let frame = match label {
            Some(l) => f.loops.iter().rev().find(|lp| lp.label == Some(l)).cloned(),
            None => f.loops.last().cloned(),
        };
        let Some(frame) = frame else {
            let what = if is_break {
                "break"
            } else {
                "continue"
            };
            return match label {
                Some(l) => err(span, format!("{what}: no enclosing loop named '{l}'")),
                None => err(span, format!("{what} outside of a loop")),
            };
        };
        self.emit_defers(f, frame.defer_depth, span)?;
        f.b.jump(if is_break {
            frame.break_block
        } else {
            frame.continue_block
        });
        Ok(())
    }

    fn check_remove(&mut self, f: &mut FnCtx, label: Option<Sym>, span: Span) -> Result<()> {
        let frame = match label {
            Some(l) => f.loops.iter().rev().find(|lp| lp.label == Some(l)).cloned(),
            None => f.loops.last().cloned(),
        };
        let Some((container, index_slot, elem, reverse)) = frame.and_then(|fr| fr.remove) else {
            return err(span, "remove is only valid inside a for loop over an array");
        };
        // array[it_index] = array[count-1]; count -= 1; it_index -= 1 (going forward: the
        // moved element is visited next; a reverse loop has already seen it).
        let Operand::Place {
            addr, ..
        } = container
        else {
            return err(span, "remove needs an addressable array");
        };
        let count = f.b.load(Ty::I64, addr);
        let dp = f.b.ptr_offset(addr, 8);
        let data = f.b.load(Ty::Ptr, dp);
        let size = self.size_of(elem, span)?;
        let s = f.b.iconst(Ty::I64, size);
        let one = f.b.iconst(Ty::I64, 1);
        let last = f.b.bin(ir::BinOp::Sub, Ty::I64, count, one);
        let idx = f.b.load(Ty::I64, index_slot);
        let off_i = f.b.bin(ir::BinOp::Mul, Ty::I64, idx, s);
        let off_l = f.b.bin(ir::BinOp::Mul, Ty::I64, last, s);
        let dst = f.b.ptr_add(data, off_i);
        let src = f.b.ptr_add(data, off_l);
        f.b.copy(dst, src, size);
        f.b.store(Ty::I64, addr, last);
        if !reverse {
            let idx2 = f.b.bin(ir::BinOp::Sub, Ty::I64, idx, one);
            f.b.store(Ty::I64, index_slot, idx2);
        }
        Ok(())
    }

    /// Declare a loop's iterator and index. ``for `it, `it_index`` inside a macro also declares
    /// them in the scope that called the macro, where the inserted loop body can see them.
    fn declare_loop_vars(
        &mut self,
        f: &FnCtx,
        loop_scope: ScopeId,
        backtick: bool,
        names: [Sym; 2],
        kinds: [EntityKind; 2],
        span: Span,
    ) {
        let caller = f
            .macros
            .last()
            .map(|m| m.caller_scope)
            .filter(|&c| backtick && c != loop_scope);
        for (name, kind) in names.into_iter().zip(kinds) {
            if let Some(caller) = caller {
                self.add_entity(caller, name, span, kind.clone(), false);
            }
            self.add_entity(loop_scope, name, span, kind, false);
        }
    }

    fn check_for(
        &mut self,
        f: &mut FnCtx,
        scope: ScopeId,
        for_: &ast::For,
        span: Span,
    ) -> Result<()> {
        // `for *=cond` / `for <=cond`: by pointer / in reverse when the constant holds.
        if for_.pointer_if.is_some() || for_.reverse_if.is_some() {
            let mut resolved = for_.clone();
            if let Some(cond) = resolved.pointer_if.take() {
                resolved.by_pointer |= self.eval_static_condition(scope, &cond)?;
            }
            if let Some(cond) = resolved.reverse_if.take() {
                resolved.reverse |= self.eval_static_condition(scope, &cond)?;
            }
            return self.check_for(f, scope, &resolved, span);
        }
        let flagged = |name: &str| for_.flags.iter().any(|fl| fl.name.as_str() == name);
        if (!f.no_abc && flagged("no_abc")) || (!f.no_aoc && flagged("no_aoc")) {
            let saved = (f.no_abc, f.no_aoc);
            f.no_abc |= flagged("no_abc");
            f.no_aoc |= flagged("no_aoc");
            let result = self.check_for(f, scope, for_, span);
            (f.no_abc, f.no_aoc) = saved;
            return result;
        }
        let it_name = for_.it.map_or_else(|| Sym::intern("it"), |i| i.name);
        let index_name = for_
            .index
            .map_or_else(|| Sym::intern("it_index"), |i| i.name);
        let (lo, hi, collection) = match &for_.over {
            ast::ForOver::Range(a, b) => (Some(a), Some(b), None),
            ast::ForOver::Collection(c) => (None, None, Some(c)),
        };
        let loop_scope = self.new_block_scope(scope);
        self.ide_scope_span(loop_scope, span);
        let depth = self.scope(loop_scope).proc_depth;
        if let (Some(a), Some(b)) = (lo, hi) {
            // Integer range, inclusive.
            let av = self.check_expr(f, scope, a, None)?;
            let bv = self.check_expr(f, scope, b, None)?;
            let ty = match (&av, &bv) {
                (
                    Operand::Const {
                        untyped: true, ..
                    },
                    Operand::Const {
                        untyped: true, ..
                    },
                ) => TypeId::S64,
                (
                    Operand::Const {
                        untyped: true, ..
                    },
                    other,
                ) => other.ty(),
                (other, _) => other.ty(),
            };
            let ty = if self.types.is_integer(ty) {
                ty
            } else {
                TypeId::S64
            };
            let av = self.convert(f, av, ty, a.span)?;
            let bv = self.convert(f, bv, ty, b.span)?;
            let (_, start) = self.rvalue(f, av, a.span)?;
            let (_, end) = self.rvalue(f, bv, b.span)?;
            let t = self.ir_ty(ty).unwrap();
            let signed = self.types.int_info(ty).is_some_and(|i| i.1);
            let it = f.b.alloca(t.size(), t.size());
            let idx = f.b.alloca(8, 8);
            f.b.store(
                t,
                it,
                if for_.reverse {
                    end
                } else {
                    start
                },
            );
            let zero = f.b.iconst(Ty::I64, 0);
            f.b.store(Ty::I64, idx, zero);
            let head = f.b.new_block();
            let body_block = f.b.new_block();
            let step = f.b.new_block();
            let exit = f.b.new_block();
            f.b.jump(head);
            f.b.switch_to(head);
            let cur = f.b.load(t, it);
            let op = match (for_.reverse, signed) {
                (false, true) => CmpOp::SLe,
                (false, false) => CmpOp::ULe,
                (true, true) => CmpOp::SGe,
                (true, false) => CmpOp::UGe,
            };
            let c = f.b.cmp(
                op,
                t,
                cur,
                if for_.reverse {
                    start
                } else {
                    end
                },
            );
            f.b.branch(c, body_block, exit);
            f.b.switch_to(body_block);
            let names = [it_name, index_name];
            self.declare_loop_vars(
                f,
                loop_scope,
                for_.backtick_names,
                names,
                [
                    EntityKind::Local {
                        ty,
                        addr: it,
                        depth,
                    },
                    EntityKind::Local {
                        ty: TypeId::S64,
                        addr: idx,
                        depth,
                    },
                ],
                span,
            );
            f.loops.push(LoopFrame {
                label: Some(it_name),
                break_block: exit,
                continue_block: step,
                defer_depth: f.defers.len(),
                remove: None,
            });
            let result = self.check_scoped(f, loop_scope, &for_.body);
            f.loops.pop();
            result?;
            f.b.jump(step);
            f.b.switch_to(step);
            // Stop before wrapping past the bound.
            let cur = f.b.load(t, it);
            let at_end = f.b.cmp(
                CmpOp::Eq,
                t,
                cur,
                if for_.reverse {
                    start
                } else {
                    end
                },
            );
            let advance = f.b.new_block();
            f.b.branch(at_end, exit, advance);
            f.b.switch_to(advance);
            let one = f.b.iconst(t, 1);
            let next = f.b.bin(
                if for_.reverse {
                    ir::BinOp::Sub
                } else {
                    ir::BinOp::Add
                },
                t,
                cur,
                one,
            );
            f.b.store(t, it, next);
            let i = f.b.load(Ty::I64, idx);
            let one64 = f.b.iconst(Ty::I64, 1);
            let i2 = f.b.bin(ir::BinOp::Add, Ty::I64, i, one64);
            f.b.store(Ty::I64, idx, i2);
            f.b.jump(head);
            f.b.switch_to(exit);
            return Ok(());
        }
        let c = collection.unwrap();
        // `for iterate(x)`: a call to a for_expansion-shaped macro (body: Code second).
        if for_.iterator.is_none()
            && let ast::ExprKind::Call {
                callee,
                args,
                ..
            } = &c.kind
            && let Ok(Operand::Procs(procs)) = self.check_expr_no_emit(scope, callee)
            && !procs.is_empty()
            && procs.iter().all(|&p| self.proc_takes_for_body(p))
        {
            let leading = self.precheck_args(f, scope, args)?;
            return self
                .check_for_expansion(f, scope, for_, procs, leading, it_name, index_name, span);
        }
        let op = self.check_expr(f, scope, c, None)?;
        let cty = op.ty();
        let target = self.types.pointee(cty).unwrap_or(cty);
        let is_struct = matches!(
            self.types.kind(self.types.repr_struct(target)),
            TypeKind::Struct(_)
        );
        if for_.iterator.is_some() || is_struct {
            let procs = self.for_expansion_procs(scope, for_, &op, span)?;
            // `for_expansion :: (x: *T, ...)` iterates a struct value through its address.
            // Unless an overload takes this very type by value (overloads for other types
            // in the same set do not count).
            let mut by_value = false;
            let mut by_pointer = false;
            for &p in &procs {
                let Some(t) = self
                    .proc(p)
                    .lit
                    .header
                    .params
                    .first()
                    .and_then(|q| q.ty.clone())
                else {
                    continue;
                };
                if matches!(t.kind, E::Unary(ast::UnOp::Star, _)) {
                    by_pointer = true;
                } else if procs::has_poly(&t)
                    || self.eval_type(self.proc(p).scope, &t).ok() == Some(cty)
                {
                    by_value = true;
                }
            }
            let wants_pointer = by_pointer && !by_value;
            let op = if wants_pointer && !self.types.is_pointer(cty) {
                let (_, addr) = self.address_of(f, op, span)?;
                Operand::Value {
                    ty: self.types.pointer(cty),
                    val: addr,
                }
            } else {
                op
            };
            let leading = vec![CallArg {
                name: None,
                spread: false,
                expr: None,
                op: Some(op),
                span,
                scope,
            }];
            return self
                .check_for_expansion(f, scope, for_, procs, leading, it_name, index_name, span);
        }
        let (elem, fixed) = match self.types.kind(self.types.repr(cty)).clone() {
            TypeKind::Array {
                elem,
                kind: ArrayKind::Fixed(n),
            } => (elem, Some(n)),
            TypeKind::Array {
                elem, ..
            } => (elem, None),
            TypeKind::String => (TypeId::U8, None),
            _ => {
                return err(
                    c.span,
                    format!(
                        "cannot iterate over a value of type {}",
                        self.types.name(cty)
                    ),
                );
            }
        };
        let container = match op {
            p @ Operand::Place {
                ..
            } => p,
            other => {
                let (ty, addr) = self.address_of(f, other, c.span)?;
                Operand::Place {
                    ty,
                    addr,
                }
            }
        };
        let Operand::Place {
            addr: caddr, ..
        } = container
        else {
            unreachable!()
        };
        let esize = self.size_of(elem, span)?;
        let idx = f.b.alloca(8, 8);
        let load_count = |f: &mut FnCtx| match fixed {
            Some(n) => f.b.iconst(Ty::I64, n),
            None => f.b.load(Ty::I64, caddr),
        };
        let start = if for_.reverse {
            let n = load_count(f);
            let one = f.b.iconst(Ty::I64, 1);
            f.b.bin(ir::BinOp::Sub, Ty::I64, n, one)
        } else {
            f.b.iconst(Ty::I64, 0)
        };
        f.b.store(Ty::I64, idx, start);
        let head = f.b.new_block();
        let body_block = f.b.new_block();
        let step = f.b.new_block();
        let exit = f.b.new_block();
        f.b.jump(head);
        f.b.switch_to(head);
        let i = f.b.load(Ty::I64, idx);
        let c = if for_.reverse {
            let zero = f.b.iconst(Ty::I64, 0);
            f.b.cmp(CmpOp::SGe, Ty::I64, i, zero)
        } else {
            let n = load_count(f);
            f.b.cmp(CmpOp::SLt, Ty::I64, i, n)
        };
        f.b.branch(c, body_block, exit);
        f.b.switch_to(body_block);
        let data = match fixed {
            Some(_) => caddr,
            None => {
                let p = f.b.ptr_offset(caddr, 8);
                f.b.load(Ty::Ptr, p)
            }
        };
        let i = f.b.load(Ty::I64, idx);
        let s = f.b.iconst(Ty::I64, esize);
        let off = f.b.bin(ir::BinOp::Mul, Ty::I64, i, s);
        let elem_addr = f.b.ptr_add(data, off);
        let (it_ty, it_addr) = if for_.by_pointer {
            let pt = self.types.pointer(elem);
            (pt, self.spill(f, pt, elem_addr, span)?)
        } else {
            let align = self.align_of(elem, span)?;
            let slot = f.b.alloca(esize.max(1), align);
            f.b.copy(slot, elem_addr, esize);
            (elem, slot)
        };
        self.declare_loop_vars(
            f,
            loop_scope,
            for_.backtick_names,
            [it_name, index_name],
            [
                EntityKind::Local {
                    ty: it_ty,
                    addr: it_addr,
                    depth,
                },
                EntityKind::Local {
                    ty: TypeId::S64,
                    addr: idx,
                    depth,
                },
            ],
            span,
        );
        let remove = if fixed.is_none() {
            Some((container.clone(), idx, elem, for_.reverse))
        } else {
            None
        };
        f.loops.push(LoopFrame {
            label: Some(it_name),
            break_block: exit,
            continue_block: step,
            defer_depth: f.defers.len(),
            remove,
        });
        let result = self.check_scoped(f, loop_scope, &for_.body);
        f.loops.pop();
        result?;
        f.b.jump(step);
        f.b.switch_to(step);
        let i = f.b.load(Ty::I64, idx);
        let one = f.b.iconst(Ty::I64, 1);
        let next = f.b.bin(
            if for_.reverse {
                ir::BinOp::Sub
            } else {
                ir::BinOp::Add
            },
            Ty::I64,
            i,
            one,
        );
        f.b.store(Ty::I64, idx, next);
        f.b.jump(head);
        f.b.switch_to(exit);
        Ok(())
    }

    /// `for x: collection` over a struct: expand its `for_expansion` macro.
    #[allow(clippy::too_many_arguments)]
    /// `for_expansion` (or the `for :name` iterator) visible at the loop.
    fn for_expansion_procs(
        &mut self,
        scope: ScopeId,
        for_: &ast::For,
        collection: &Operand,
        span: Span,
    ) -> Result<Vec<ProcId>> {
        let macro_name = for_
            .iterator
            .map_or_else(|| Sym::intern("for_expansion"), |i| i.name);
        let mut ids = self.lookup(scope, macro_name)?;
        if ids.is_empty()
            && for_.iterator.is_none()
            && let Some(home) = self.struct_home_scope(collection.ty())
        {
            ids = self.lookup(home, macro_name)?;
        }
        let mut procs = Vec::new();
        for id in ids {
            if let Resolved::Proc(p) = self.resolve_entity(id)? {
                procs.push(p);
            }
        }
        if procs.is_empty() {
            return err(
                span,
                format!(
                    "no '{macro_name}' is visible for iterating over {}",
                    self.types.name(collection.ty())
                ),
            );
        }
        Ok(procs)
    }

    /// A macro whose second parameter is the loop body (`(x, body: Code, flags)`).
    fn proc_takes_for_body(&self, p: ProcId) -> bool {
        let info = self.proc(p);
        info.is_macro
            && info.lit.header.params.get(1).is_some_and(|param| {
                matches!(&param.ty, Some(t) if matches!(&t.kind, ast::ExprKind::Ident(n) if n.as_str() == "Code"))
            })
    }

    /// Expand a for loop through `procs`, called with `leading` arguments
    /// followed by the body as Code and the For_Flags.
    #[allow(clippy::too_many_arguments)]
    fn check_for_expansion(
        &mut self,
        f: &mut FnCtx,
        scope: ScopeId,
        for_: &ast::For,
        procs: Vec<ProcId>,
        leading: Vec<CallArg>,
        it_name: Sym,
        index_name: Sym,
        span: Span,
    ) -> Result<()> {
        // The body becomes a Code value; names declared with backticks land in the loop scope.
        let loop_scope = self.new_block_scope(scope);
        let body_stmt = Rc::new((*for_.body).clone());
        let code = self.add_code(
            Rc::new(ast::CodeBody::Block(ast::Block {
                stmts: vec![(*for_.body).clone()],
                span: for_.body.span,
                no_abc: false,
                no_aoc: false,
            })),
            loop_scope,
        );
        let flags_value = (for_.by_pointer as i128) | ((for_.reverse as i128) << 1);
        let flags_ty = self.preload_type("For_Flags", span).unwrap_or(TypeId::U8);
        let mut args = leading;
        args.extend([
            CallArg {
                name: None,
                spread: false,
                expr: None,
                op: Some(Operand::Const {
                    ty: TypeId::CODE,
                    value: Value::Code(code),
                    untyped: false,
                }),
                span,
                scope,
            },
            CallArg {
                name: None,
                spread: false,
                expr: None,
                op: Some(Operand::Const {
                    ty: flags_ty,
                    value: Value::Int(flags_value),
                    untyped: false,
                }),
                span,
                scope,
            },
        ]);
        f.pending_for_body = Some(ForBody {
            code,
            body: body_stmt,
            scope: loop_scope,
            it_name,
            index_name,
            label: Some(it_name),
        });
        let result = self.call_procs(f, loop_scope, &procs, args, None, span);
        f.pending_for_body = None;
        result.map(|_| ())
    }

    #[allow(clippy::too_many_arguments)]
    fn check_insert(
        &mut self,
        f: &mut FnCtx,
        scope: ScopeId,
        value: &ast::Expr,
        flags: &[ast::Ident],
        target: Option<&ast::Expr>,
        replacements: &[ast::Arg],
        span: Span,
    ) -> Result<()> {
        let mut op = self.eval_insert_operand(scope, value)?;
        // `#insert,scope(code) "text"`: the text becomes Code inserted there.
        if target.is_some()
            && matches!(
                op,
                Operand::Const {
                    value: Value::String(_),
                    ..
                }
            )
        {
            let stmts = self.insert_stmts_from(op, value.span)?;
            let block = ast::Block {
                stmts,
                span: value.span,
                no_abc: false,
                no_aoc: false,
            };
            let code = self.add_code(Rc::new(ast::CodeBody::Block(block)), scope);
            op = Operand::Const {
                ty: TypeId::CODE,
                value: Value::Code(code),
                untyped: false,
            };
        }
        if let Operand::Const {
            value: Value::Code(code),
            ..
        } = op
        {
            // The body of a for_expansion loop.
            if let Some(frame_index) = f
                .macros
                .iter()
                .rposition(|m| m.for_body.as_ref().is_some_and(|b| b.code == code))
            {
                let replacement = |name: &str| {
                    replacements
                        .iter()
                        .find(|r| r.name.is_some_and(|n| n.name.as_str() == name))
                        .map(|r| r.value.clone())
                };
                let replacements = InsertReplacements {
                    loop_index: f.loops.len(),
                    scope,
                    break_: replacement("break"),
                    continue_: replacement("continue"),
                    remove: replacement("remove"),
                };
                return self.insert_for_body(f, frame_index, replacements, span);
            }
            let body = self.codes[code.0 as usize].clone();
            // Inserted code resolves names where it was written, unless `,scope()` names
            // the insertion site or `,scope(other_code)` another code's scope.
            let code_scope = match target {
                Some(t) => match self.eval_const(scope, t, None)? {
                    Operand::Const {
                        value: Value::Code(other),
                        ..
                    } => self.code_scopes[other.0 as usize],
                    _ => return err(t.span, "#insert,scope(...) needs a Code value"),
                },
                None if flags.iter().any(|fl| fl.name.as_str() == "scope") => scope,
                None => self.code_scopes[code.0 as usize],
            };
            // `Top :: #code()` is evaluated in its file's constant-thunk scope: that file.
            let code_scope = match self.scope(code_scope).parent {
                Some(parent) if self.thunk_scopes.get(&parent) == Some(&code_scope) => parent,
                _ => code_scope,
            };
            let inner = self.new_block_scope(code_scope);
            self.scopes[inner.0 as usize].proc_depth = self.scope(scope).proc_depth;
            // Constants inserted into a file or module scope are declared there (and
            // reported to the metaprogram like any top-level declaration).
            if target.is_some()
                && matches!(
                    self.scope(code_scope).kind,
                    ScopeKind::File | ScopeKind::Module
                )
                && let ast::CodeBody::Block(b) = &*body
            {
                for stmt in &b.stmts {
                    if let S::Decl(decl) = &stmt.kind
                        && decl.kind == ast::DeclKind::Const
                        && !decl.backtick
                        && f.hoisted_consts.insert((inner, decl.id))
                    {
                        self.declare_local_consts(code_scope, code_scope, decl);
                    }
                }
            }
            return match &*body {
                ast::CodeBody::Expr(e) => self.check_expr(f, inner, e, None).map(|_| ()),
                ast::CodeBody::Block(b) => self.check_block_stmts(f, inner, &b.stmts),
            };
        }
        let stmts = self.insert_stmts_from(op, value.span)?;
        self.check_block_stmts(f, scope, &stmts)
    }

    fn insert_for_body(
        &mut self,
        f: &mut FnCtx,
        frame_index: usize,
        replacements: InsertReplacements,
        span: Span,
    ) -> Result<()> {
        let frame: MacroFrame = f.macros[frame_index].clone();
        let body = frame.for_body.clone().unwrap();
        // A for_expansion that forwards its body (`for_expansion(*inner, body, flags)`):
        // the macro inserting it declared `it` / `it_index` in its own caller's scope.
        let mut borrowed = Vec::new();
        if frame_index + 1 < f.macros.len() {
            let inserter = f.macros.last().unwrap().caller_scope;
            for name in ["it", "it_index"] {
                let name = Sym::intern(name);
                let missing = self
                    .scope(body.scope)
                    .names
                    .get(&name)
                    .is_none_or(|ids| ids.is_empty());
                let found = self
                    .scope(inserter)
                    .names
                    .get(&name)
                    .and_then(|ids| ids.last().copied());
                if missing && let Some(id) = found {
                    let kind = self.entity(id).kind.clone();
                    borrowed.push((name, self.add_entity(body.scope, name, span, kind, false)));
                }
            }
        }
        // Alias custom iterator names to the macro's `it` / `it_index`. The renamed
        // originals are hidden from the body, so an enclosing `it_index` stays visible.
        let mut hidden = Vec::new();
        for (alias, original) in [(body.it_name, "it"), (body.index_name, "it_index")] {
            if alias.as_str() == original {
                continue;
            }
            let original = Sym::intern(original);
            let ids = self
                .scope(body.scope)
                .names
                .get(&original)
                .cloned()
                .unwrap_or_default();
            if let Some(&id) = ids.last() {
                let kind = self.entity(id).kind.clone();
                self.add_entity(body.scope, alias, span, kind, false);
                if let Some(names) = self.scope_mut(body.scope).names.get_mut(&original) {
                    names.retain(|&e| e != id);
                }
                hidden.push((original, id));
            }
        }
        // break/continue in the body target the macro's innermost loop.
        let (brk, cont) = match f.loops.last() {
            Some(l) if f.loops.len() > frame.loop_depth => (l.break_block, l.continue_block),
            _ => (frame.exit_block, frame.exit_block),
        };
        f.loops.push(LoopFrame {
            label: body.label,
            break_block: brk,
            continue_block: cont,
            defer_depth: f.defers.len(),
            remove: None,
        });
        let saved = f.macros.split_off(frame_index);
        f.insert_replacements.push(replacements);
        let result = self.check_scoped(f, body.scope, &body.body);
        for (original, id) in hidden {
            self.scope_mut(body.scope)
                .names
                .entry(original)
                .or_default()
                .push(id);
        }
        for (name, id) in borrowed {
            if let Some(names) = self.scope_mut(body.scope).names.get_mut(&name) {
                names.retain(|&e| e != id);
            }
        }
        f.insert_replacements.pop();
        f.macros.extend(saved);
        f.loops.pop();
        result
    }

    /// A `break` / `continue` / `remove` aimed at an inserted for-loop body whose
    /// `#insert` replaced it: check the replacement at the insertion site instead.
    /// `which` picks the replacement; returns false when there is none.
    fn try_insert_replacement(
        &mut self,
        f: &mut FnCtx,
        label: Option<Sym>,
        which: fn(&InsertReplacements) -> &Option<ast::Expr>,
        span: Span,
    ) -> Result<bool> {
        let target = match label {
            Some(l) => f.loops.iter().rposition(|lp| lp.label == Some(l)),
            None => f.loops.len().checked_sub(1),
        };
        let Some(target) = target else {
            return Ok(false);
        };
        let Some(pos) = f
            .insert_replacements
            .iter()
            .rposition(|r| r.loop_index == target)
        else {
            return Ok(false);
        };
        let Some(code) = which(&f.insert_replacements[pos]).clone() else {
            return Ok(false);
        };
        let scope = f.insert_replacements[pos].scope;
        // The replacement runs as written in the macro: without the body's loop and the
        // replacements of this and inner inserts.
        let saved_loops = f.loops.split_off(target);
        let saved_reps = f.insert_replacements.split_off(pos);
        let inner = self.new_block_scope(scope);
        let result = match &code.kind {
            ast::ExprKind::Block(block) => self.check_block_stmts(f, inner, &block.stmts),
            _ => self.check_expr(f, inner, &code, None).map(|_| ()),
        };
        f.loops.extend(saved_loops);
        f.insert_replacements.extend(saved_reps);
        let _ = span;
        result.map(|()| true)
    }

    fn check_return(
        &mut self,
        f: &mut FnCtx,
        scope: ScopeId,
        values: &[ast::Arg],
        backtick: bool,
        span: Span,
    ) -> Result<()> {
        if let Some(frame) = f.macros.last().cloned()
            && !backtick
        {
            // Return from a macro: store results and leave the expansion.
            let mut ops = Vec::new();
            for (i, v) in values.iter().enumerate() {
                let expected = frame.result_slots.get(i).map(|s| s.0);
                ops.push(self.check_expr(f, scope, &v.value, expected)?);
            }
            for (op, (ty, slot)) in ops.into_iter().zip(&frame.result_slots) {
                let op = self.convert(f, op, *ty, span)?;
                let (_, v) = self.rvalue(f, op, span)?;
                self.store_value(f, *ty, *slot, v, span)?;
            }
            self.emit_defers(f, frame.defer_depth, span)?;
            f.b.jump(frame.exit_block);
            return Ok(());
        }
        let saved_macros = if backtick {
            std::mem::take(&mut f.macros)
        } else {
            Vec::new()
        };
        let result = self.check_proc_return(f, scope, values, span);
        if backtick {
            f.macros = saved_macros;
        }
        result
    }

    /// `return second = 2, first = 1;`: values are evaluated in source order
    /// and stored into their named results.
    fn check_named_return(
        &mut self,
        f: &mut FnCtx,
        scope: ScopeId,
        values: &[ast::Arg],
        span: Span,
    ) -> Result<()> {
        let Some(proc) = f.proc else {
            return err(span, "named return values need a procedure");
        };
        let names = self.signature(proc, span)?.return_names.clone();
        let types = f.return_types.clone();
        let mut positional = 0;
        for v in values {
            let index = match v.name {
                Some(n) => names
                    .iter()
                    .position(|r| *r == Some(n.name))
                    .ok_or_else(|| {
                        Box::new(Diagnostic::error(
                            n.span,
                            format!("no return value named '{}'", n.name),
                        ))
                    })?,
                None => {
                    positional += 1;
                    positional - 1
                }
            };
            let Some(&ty) = types.get(index) else {
                return err(v.value.span, "too many return values");
            };
            let Some(addr) = f.named_results[index] else {
                return err(v.value.span, "return value has no name to assign");
            };
            let op = self.check_expr(f, scope, &v.value, Some(ty))?;
            let op = self.convert(f, op, ty, v.value.span)?;
            let (_, val) = self.rvalue(f, op, v.value.span)?;
            self.store_value(f, ty, addr, val, v.value.span)?;
        }
        self.emit_fallthrough_return(f, span)
    }

    fn check_proc_return(
        &mut self,
        f: &mut FnCtx,
        scope: ScopeId,
        values: &[ast::Arg],
        span: Span,
    ) -> Result<()> {
        let types = f.return_types.clone();
        if values.iter().any(|v| v.name.is_some()) {
            return self.check_named_return(f, scope, values, span);
        }
        let mut ops: Vec<Operand> = Vec::new();
        for (i, v) in values.iter().enumerate() {
            let op = self.check_expr(f, scope, &v.value, types.get(i).copied())?;
            match op {
                // Extra results of a returned call are dropped, like in a declaration.
                Operand::Multi(vals) if values.len() == 1 => ops.extend(
                    vals.into_iter()
                        .take(types.len())
                        .map(|(ty, val)| Operand::Value {
                            ty,
                            val,
                        }),
                ),
                // `return f();` of a procedure without results.
                Operand::Void if values.len() == 1 => {}
                other => ops.push(other),
            }
        }
        if ops.is_empty() && !types.is_empty() && f.named_results.iter().all(Option::is_some) {
            self.emit_fallthrough_return(f, span)?;
            return Ok(());
        }
        if ops.len() > types.len() {
            return err(
                span,
                format!("too many return values (procedure returns {})", types.len()),
            );
        }
        if ops.len() < types.len() && !ops.is_empty() {
            // Remaining results take their named defaults.
            if let Some(i) = (ops.len()..types.len()).find(|&i| f.named_results[i].is_none()) {
                return err(
                    span,
                    format!("missing return value {} of {}", i + 1, types.len()),
                );
            }
        } else if ops.is_empty() && !types.is_empty() {
            return err(span, "missing return value");
        }
        let mut scalars = Vec::new();
        for (i, &ty) in types.iter().enumerate() {
            let v = match ops.get(i) {
                Some(op) => {
                    let op = self.convert(f, op.clone(), ty, span)?;
                    let (_, v) = self.rvalue(f, op, span)?;
                    v
                }
                None => {
                    let addr = f.named_results[i].unwrap();
                    match self.ir_ty(ty) {
                        Some(t) => f.b.load(t, addr),
                        None => addr,
                    }
                }
            };
            match f.return_outs[i] {
                Some(out) => self.store_value(f, ty, out, v, span)?,
                None => {
                    // Keep the value safe from defers that modify locals.
                    let t = self.ir_ty(ty).unwrap();
                    let slot = f.b.alloca(t.size(), t.size());
                    f.b.store(t, slot, v);
                    scalars.push((t, slot));
                }
            }
        }
        self.emit_defers(f, 0, span)?;
        let mut rets = Vec::new();
        for (t, slot) in scalars {
            rets.push(f.b.load(t, slot));
        }
        f.b.ret(rets);
        Ok(())
    }

    /// Emit deferred statements registered above `depth`, innermost first.
    pub fn emit_defers(&mut self, f: &mut FnCtx, depth: usize, _span: Span) -> Result<()> {
        if f.defers.len() <= depth {
            return Ok(());
        }
        let saved = f.defers.clone();
        for i in (depth..saved.len()).rev() {
            f.defers.truncate(i);
            let entry = saved[i].clone();
            let inner = self.new_block_scope(entry.scope);
            let saved_backtick = std::mem::replace(&mut f.backtick_scope, entry.caller_scope);
            let result = self.check_stmt(f, inner, &entry.stmt);
            f.backtick_scope = saved_backtick;
            result?;
        }
        f.defers = saved;
        Ok(())
    }
}
