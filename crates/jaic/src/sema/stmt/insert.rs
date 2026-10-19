//! `#insert` statements and the scopes they publish into.
use crate::ast::StmtKind as S;
use crate::sema::lower::{FnCtx, InsertReplacements, LoopFrame, LoopRemove, MacroFrame, Operand};
use crate::sema::scope::{EntityId, EntityKind, ScopeKind};
use crate::sema::{
    Compiler, Diagnostic, HashMap, Rc, Result, ScopeId, Span, Sym, TypeId, Value, ast, err, scope,
};

impl Compiler {
    #[allow(clippy::too_many_arguments)]
    pub(super) fn check_insert(
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
        let string_payload = target.is_some()
            && matches!(
                &op,
                Operand::Const {
                    value: Value::String(_),
                    ..
                }
            );
        if string_payload {
            let stmts = self.insert_stmts_from(op, value.span)?;
            let block = ast::Block {
                stmts,
                span: value.span,
                checks_off: crate::ast::CheckBits::NONE,
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
                    } => {
                        let target = self.code_scopes[other.0 as usize];
                        if self.unscoped_codes.contains_key(&(code.0 as usize)) {
                            self.code_scope_at(code, target)
                        } else {
                            target
                        }
                    }
                    _ => return err(t.span, "#insert,scope(...) needs a Code value"),
                },
                None if flags.iter().any(|fl| fl.name.as_str() == "scope") => scope,
                None => self.code_scope_at(code, scope),
            };
            // `Top :: #code()` is evaluated in its file's constant-thunk scope: that file.
            let code_scope = match self.scope(code_scope).parent {
                Some(parent) if self.thunk_scopes.get(&parent) == Some(&code_scope) => parent,
                _ => code_scope,
            };
            let inner = self.new_block_scope(code_scope);
            let unbraced = string_payload || !matches!(&*body, ast::CodeBody::Block(_));
            if unbraced {
                self.scopes[inner.0 as usize].inserted_into = Some(scope);
            }
            self.scopes[inner.0 as usize].proc_depth = self.scope(scope).proc_depth;
            // Constants inserted into a file or module scope are declared there (and
            // reported to the metaprogram like any top-level declaration).
            if target.is_some()
                && matches!(
                    self.scope(code_scope).kind,
                    ScopeKind::File | ScopeKind::Module
                )
            {
                let statements = match &*body {
                    ast::CodeBody::Block(b) => b.stmts.as_slice(),
                    ast::CodeBody::Stmt(stmt) => std::slice::from_ref(stmt),
                    ast::CodeBody::Expr(_) => &[],
                };
                for stmt in statements {
                    if let S::Decl(decl) = &stmt.kind
                        && decl.kind == ast::DeclKind::Const
                        && !decl.backtick
                        && f.hoisted_consts.insert((inner, decl.id))
                    {
                        let _ = self.declare_local_consts(code_scope, code_scope, decl);
                    }
                }
            }
            let result = match &*body {
                ast::CodeBody::Expr(e) => self.check_expr(f, inner, e, None).map(|_| ()),
                ast::CodeBody::Stmt(stmt) => self
                    .check_block_stmts(f, inner, std::slice::from_ref(stmt))
                    .and_then(|()| self.ide_require_parsed_body(stmt.span)),
                ast::CodeBody::Block(b) if string_payload => {
                    self.check_block_stmts(f, inner, &b.stmts)
                }
                ast::CodeBody::Block(b) => self
                    .check_scoped(
                        f,
                        inner,
                        &ast::Stmt {
                            kind: S::Block(b.clone()),
                            span: b.span,
                            notes: ast::Notes::default(),
                        },
                    )
                    .and_then(|()| self.ide_require_parsed_body(b.span)),
            };
            // Only a brace-less payload declares in the receiving lexical block.
            // Its wrapper supplies captured lookup without adding a lexical boundary.
            result?;
            if unbraced {
                self.publish_insert_scope(inner, scope, span)?;
            }
            return Ok(());
        }
        let stmts = self.insert_stmts_from(op, value.span)?;
        self.check_block_stmts(f, scope, &stmts)
    }

    /// Keep inserted declarations' canonical entities and captured lookup scope,
    /// while making their bindings visible in the receiving lexical scope.
    fn publish_insert_scope(&mut self, source: ScopeId, target: ScopeId, at: Span) -> Result<()> {
        let mut bindings: Vec<(Sym, EntityId)> = self
            .scope(source)
            .names
            .iter()
            .flat_map(|(&name, ids)| ids.iter().map(move |&id| (name, id)))
            .collect();
        // Declaration/hoisting order, rather than hash-map iteration order,
        // determines which collision is diagnosed first.
        bindings.sort_by_key(|(_, id)| id.0);
        let mut publish = Vec::new();
        for (name, id) in bindings {
            let existing = self
                .scope(target)
                .names
                .get(&name)
                .map_or(&[][..], Vec::as_slice);
            let mut already_bound = false;
            for old in self.bound_entities(existing) {
                let new = self.entity(id);
                let previous = self.entity(old);
                let same_binding = id == old
                    || match (&new.kind, &previous.kind) {
                        // A macro backtick declaration may already have put
                        // the same local in its caller's scope.
                        (
                            EntityKind::Local {
                                ty,
                                addr,
                                depth,
                            },
                            EntityKind::Local {
                                ty: old_ty,
                                addr: old_addr,
                                depth: old_depth,
                            },
                        ) => ty == old_ty && addr == old_addr && depth == old_depth,
                        (
                            EntityKind::Decl {
                                decl, ..
                            },
                            EntityKind::Decl {
                                decl: old_decl, ..
                            },
                        ) => decl.backtick && Rc::ptr_eq(decl, old_decl),
                        _ => false,
                    };
                if same_binding {
                    already_bound = true;
                    continue;
                }
                if let (
                    EntityKind::Decl {
                        decl, ..
                    },
                    EntityKind::Decl {
                        decl: old_decl, ..
                    },
                ) = (&new.kind, &previous.kind)
                    && scope::may_overload(decl)
                    && scope::may_overload(old_decl)
                {
                    continue;
                }
                return Err(Box::new(
                    Diagnostic::error(
                        new.span,
                        format!("`{name}` is already declared in this scope"),
                    )
                    .with_label("declared again here")
                    .with_note(previous.span, format!("`{name}` is first declared here"))
                    .with_note(at, "this Code is inserted here"),
                ));
            }
            if !already_bound {
                publish.push((name, id));
            }
        }
        // Check newly formed overload sets now: an unreferenced enclosing body
        // may be checked after the driver's earlier declared-overload check.
        // The regular checker uses the logical receiving namespace while the
        // entities keep their captured lookup scopes.
        let mut sets = Vec::<Vec<EntityId>>::new();
        let mut set_indices = HashMap::<Sym, usize>::default();
        for &(name, id) in &publish {
            let index = *set_indices.entry(name).or_insert_with(|| {
                let index = sets.len();
                sets.push(
                    self.scope(target)
                        .names
                        .get(&name)
                        .cloned()
                        .unwrap_or_default(),
                );
                index
            });
            sets[index].push(id);
        }
        for ids in sets {
            if ids.len() > 1 {
                self.check_identical_overloads(&ids)?;
            }
        }
        // Validate the complete namespace before publishing any of it. A bad
        // insertion must not leave a partially published set of declarations.
        for (name, id) in publish {
            self.bind_name(target, name, id);
        }
        let usings = self.scope(source).usings.clone();
        if !usings.is_empty() {
            self.scope_mut(target).usings.extend(usings);
            self.namespace_changed(target);
        }
        Ok(())
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
                    .is_none_or(std::vec::Vec::is_empty);
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
        // originals are hidden from the body, so an enclosing `it_index` stays visible. The
        // aliases go away again afterwards: a macro may insert the body more than once.
        let mut hidden = Vec::new();
        let aliases = [(body.it_name, "it"), (body.index_name, "it_index")];
        for ((alias, original), at) in aliases.into_iter().zip(body.name_spans) {
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
                let alias_id = self.add_entity(body.scope, alias, at.unwrap_or(span), kind, false);
                if at.is_some() && self.ide.is_some() {
                    self.ide_note_entity(alias_id);
                }
                borrowed.push((alias, alias_id));
                if let Some(names) = self.scope_mut(body.scope).names.get_mut(&original) {
                    names.retain(|&e| e != id);
                }
                self.namespace_changed(body.scope);
                hidden.push((original, id));
            }
        }
        // Inserted loop controls target the macro's innermost loop. Forward its array
        // removal state as well as jump/defer state; a numeric loop still has no remove.
        let (brk, cont, defer_depth, remove) = match f.loops.last() {
            Some(l) if f.loops.len() > frame.loop_depth => (
                l.break_block,
                l.continue_block,
                l.defer_depth.clone(),
                l.remove.clone(),
            ),
            _ => (
                frame.exit_block,
                frame.exit_block,
                frame.defer_depth.clone(),
                LoopRemove::Unsupported(
                    "remove needs a for loop over an array around the inserted code".into(),
                ),
            ),
        };
        f.loops.push(LoopFrame {
            label: body.label,
            break_block: brk,
            continue_block: cont,
            defer_depth,
            remove,
        });
        let saved = f.macros.split_off(frame_index);
        f.insert_replacements.push(replacements);
        let result = self.check_scoped(f, body.scope, &body.body);
        for (original, id) in hidden {
            self.bind_name(body.scope, original, id);
        }
        for (name, id) in borrowed {
            if let Some(names) = self.scope_mut(body.scope).names.get_mut(&name) {
                names.retain(|&e| e != id);
            }
            self.namespace_changed(body.scope);
        }
        f.insert_replacements.pop();
        f.macros.extend(saved);
        f.loops.pop();
        result
    }

    /// A `break` / `continue` / `remove` aimed at an inserted for-loop body whose
    /// `#insert` replaced it: check the replacement at the insertion site instead.
    /// `which` picks the replacement; returns false when there is none.
    pub(super) fn try_insert_replacement(
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
}
