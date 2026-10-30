//! Declaring a file's statements into its scopes and expanding pending items.
use super::parameters::{ParameterContract, ParameterContractState};
use crate::diagnostics::Code;
use crate::sema::scope::{EntityKind, ImportEntry, Pending, PendingState, ScopeKind, UsingEntry};
use crate::sema::{Compiler, EntityId, Rc, Result, ScopeId, Sym, ast, err, lower, procs};

impl Compiler {
    /// Declare top-level statements. Returns the visibility state at the end.
    pub fn declare_stmts(
        &mut self,
        scope: ScopeId,
        file_scope: ScopeId,
        stmts: &[ast::Stmt],
        mut vis: ast::ScopeKind,
    ) -> Result<ast::ScopeKind> {
        for stmt in stmts {
            match self.declare_stmt(scope, file_scope, stmt, vis) {
                Ok(next) => vis = next,
                // The editor keeps the error and goes on: the directives after one bad
                // statement (`#assert`, `#run`, `#poke_name`) still check, so the names in
                // them keep their types and definitions while that statement is being typed.
                Err(error) => self.ed.keep_going(error)?,
            }
        }
        Ok(vis)
    }

    fn target_scope(&self, scope: ScopeId, file_scope: ScopeId, vis: ast::ScopeKind) -> ScopeId {
        if vis == ast::ScopeKind::File && self.scope(scope).kind == ScopeKind::Module {
            file_scope
        } else {
            scope
        }
    }

    pub fn declare_stmt(
        &mut self,
        scope: ScopeId,
        file_scope: ScopeId,
        stmt: &ast::Stmt,
        vis: ast::ScopeKind,
    ) -> Result<ast::ScopeKind> {
        let exported = vis == ast::ScopeKind::Export;
        if matches!(self.scope(scope).kind, ScopeKind::Struct(_)) {
            // Struct bodies: only constants are entities; fields are laid out separately.
            match &stmt.kind {
                ast::StmtKind::Decl(decl) if decl.kind == ast::DeclKind::Const => {
                    for (index, name) in decl.names.iter().enumerate() {
                        self.check_redeclared(scope, name.name, name.span, Some(decl))?;
                        self.add_entity(
                            scope,
                            name.name,
                            name.span,
                            EntityKind::Decl {
                                decl: decl.clone(),
                                index,
                            },
                            true,
                        );
                    }
                }
                ast::StmtKind::StaticIf(_) => {
                    self.push_pending(
                        scope,
                        Pending {
                            stmt: stmt.clone(),
                            exported,
                            file_scope,
                            state: PendingState::Waiting,
                        },
                    );
                }
                _ => {}
            }
            return Ok(vis);
        }
        match &stmt.kind {
            ast::StmtKind::Decl(..) => self.declare_decl_stmt(scope, file_scope, stmt, vis)?,
            ast::StmtKind::Import(..)
            | ast::StmtKind::Load {
                ..
            }
            | ast::StmtKind::ModuleParameters(..)
            | ast::StmtKind::Using(..) => self.declare_module_stmt(scope, file_scope, stmt, vis)?,
            ast::StmtKind::StaticIf(..)
            | ast::StmtKind::StaticSwitch(..)
            | ast::StmtKind::Insert(..)
            | ast::StmtKind::Run(..)
            | ast::StmtKind::Assert(..)
            | ast::StmtKind::AddContext(..)
            | ast::StmtKind::Placeholder(..)
            | ast::StmtKind::Directive(..)
            | ast::StmtKind::Empty => self.declare_directive_stmt(scope, file_scope, stmt, vis)?,
            ast::StmtKind::Scope(kind) => return Ok(*kind),
            _ => {
                return err(
                    Code::ModuleStatementNotAllowed,
                    stmt.span,
                    "this statement is not allowed at file scope",
                );
            }
        }
        Ok(vis)
    }

    fn declare_decl_stmt(
        &mut self,
        scope: ScopeId,
        file_scope: ScopeId,
        stmt: &ast::Stmt,
        vis: ast::ScopeKind,
    ) -> Result<()> {
        let target = self.target_scope(scope, file_scope, vis);
        let exported = vis == ast::ScopeKind::Export;
        match &stmt.kind {
            ast::StmtKind::Decl(decl) => {
                // `using _ :: struct {...}` may appear many times: each gets its own name.
                let entity_name = |c: &Self, name: Sym| {
                    if decl.using && name.as_str() == "_" {
                        Sym::intern(&format!("__using_{}", c.ns.entities.len()))
                    } else {
                        name
                    }
                };
                let mut using_name = None;
                for (index, name) in decl.names.iter().enumerate() {
                    let named = entity_name(self, name.name);
                    using_name.get_or_insert(named);
                    self.check_redeclared(target, named, name.span, Some(decl))?;
                    let id = self.add_entity(
                        target,
                        named,
                        name.span,
                        EntityKind::Decl {
                            decl: decl.clone(),
                            index,
                        },
                        exported,
                    );
                    self.entity_mut(id).file_private = vis == ast::ScopeKind::File;
                    self.entity_mut(id).home = file_scope;
                    if let Some(ast::Expr {
                        kind: ast::ExprKind::Proc(lit),
                        ..
                    }) = &decl.value
                        && lit.header.flags.program_export.is_some()
                    {
                        self.ns.export_entities.push(id);
                    }
                    // `L :: #library,link_always "x"` is linked even if nothing names it.
                    if let Some(ast::Expr {
                        kind: ast::ExprKind::UnknownDirective(directive),
                        ..
                    }) = &decl.value
                        && directive
                            .flags
                            .iter()
                            .any(|f| f.name.as_str() == "link_always")
                    {
                        self.ns.export_entities.push(id);
                    }
                    if !decl.notes.is_empty() || decl.kind == ast::DeclKind::Const {
                        self.note_declaration(id, decl);
                    }
                }
                // A procedure's body `#import`s are visible to the whole module (`#if`-guarded
                // ones once their branch is checked).
                if let Some(ast::Expr {
                    kind: ast::ExprKind::Proc(lit),
                    ..
                }) = &decl.value
                    && let Some(body) = &lit.body
                {
                    self.hoist_body_imports(file_scope, &body.stmts, true);
                }
                if decl.using
                    && decl.kind == ast::DeclKind::Const
                    && let [name] = decl.names.as_slice()
                    && let Some(ast::Expr {
                        kind: ast::ExprKind::Enum(lit),
                        ..
                    }) = &decl.value
                    && lit
                        .items
                        .iter()
                        .all(|i| matches!(i, ast::EnumItem::Member(_)))
                {
                    // `using E :: enum { A; B; }`: the member names are known now, so declare
                    // `A :: E.A;` aliases at once; other declarations (even other enums'
                    // values) may refer to them before the enum itself is checked.
                    for item in &lit.items {
                        let ast::EnumItem::Member(member) = item else {
                            continue;
                        };
                        let span = member.name.span;
                        let alias = const_alias(
                            member.name,
                            ast::Expr {
                                kind: ast::ExprKind::Member(
                                    Box::new(ast::Expr {
                                        kind: ast::ExprKind::Ident(name.name),
                                        span,
                                        parenthesized: false,
                                    }),
                                    member.name,
                                ),
                                span,
                                parenthesized: false,
                            },
                        );
                        let id = self.add_entity(
                            target,
                            member.name.name,
                            span,
                            EntityKind::Decl {
                                decl: alias,
                                index: 0,
                            },
                            exported,
                        );
                        self.entity_mut(id).file_private = vis == ast::ScopeKind::File;
                        self.entity_mut(id).home = file_scope;
                    }
                } else if decl.using
                    && let [name] = decl.names.as_slice()
                {
                    // `using E :: enum {...}`: also bring the type's members into scope.
                    self.push_pending(
                        target,
                        Pending {
                            stmt: ast::Stmt {
                                kind: ast::StmtKind::Using(Box::new(ast::UsingStmt {
                                    value: ast::Expr {
                                        kind: ast::ExprKind::Ident(using_name.unwrap_or(name.name)),
                                        span: name.span,
                                        parenthesized: false,
                                    },
                                    filter: ast::UsingFilter::None,
                                })),
                                span: stmt.span,
                                notes: ast::Notes::default(),
                            },
                            exported,
                            file_scope,
                            state: PendingState::Waiting,
                        },
                    );
                }
            }
            _ => unreachable!("dispatched only for this group's operations"),
        }
        Ok(())
    }

    fn declare_module_stmt(
        &mut self,
        scope: ScopeId,
        file_scope: ScopeId,
        stmt: &ast::Stmt,
        vis: ast::ScopeKind,
    ) -> Result<()> {
        let target = self.target_scope(scope, file_scope, vis);
        let exported = vis == ast::ScopeKind::Export;
        let file = self.scope_file(file_scope);
        match &stmt.kind {
            ast::StmtKind::Import(import) => {
                self.validate_import_program_origin(file_scope, import)?;
                self.note_program_params(file_scope, import);
                if let Some(name) = import.name {
                    self.check_redeclared(target, name.name, name.span, None)?;
                    let id = self.add_entity(
                        target,
                        name.name,
                        name.span,
                        EntityKind::Import(import.clone()),
                        exported,
                    );
                    self.entity_mut(id).home = file_scope;
                    if let Some(filter) = &import.using {
                        // `using Name :: #import "M"` also brings the module's names into scope;
                        // at module level (exported) they become the module's members too.
                        if self.scope(target).kind == ScopeKind::Module {
                            let index = self.scope(target).imports.len();
                            let module_id = self.scope(target).module;
                            let module = &mut self.ns.modules[module_id.0 as usize];
                            module.member_using_imports.push(index);
                            if exported {
                                module.exported_using_imports.push(index);
                            }
                        }
                        self.push_import(
                            target,
                            ImportEntry {
                                import: import.clone(),
                                module: None,
                                loading: false,
                                from_scope: file_scope,
                                filter: filter.clone(),
                            },
                        );
                    }
                } else {
                    // A top-level anonymous import contributes to the containing
                    // module's import environment even after #scope_file. The
                    // directive still governs named aliases and declarations.
                    // Keep contextual imports in their existing scope, and keep
                    // from_scope below for paths/parameters written in this file.
                    let target = if self.scope(scope).kind == ScopeKind::Module {
                        scope
                    } else {
                        target
                    };
                    // Anonymous `using,only/except(...) #import` exposes the
                    // selected original declarations just like its named form,
                    // without introducing a public namespace alias.
                    if import.using.is_some()
                        && vis != ast::ScopeKind::File
                        && self.scope(target).kind == ScopeKind::Module
                    {
                        let index = self.scope(target).imports.len();
                        let module_id = self.scope(target).module;
                        let module = &mut self.ns.modules[module_id.0 as usize];
                        module.member_using_imports.push(index);
                        if exported {
                            module.exported_using_imports.push(index);
                        }
                    }
                    self.push_import(
                        target,
                        ImportEntry {
                            import: import.clone(),
                            module: None,
                            loading: false,
                            from_scope: file_scope,
                            filter: import.using.clone().unwrap_or(ast::UsingFilter::None),
                        },
                    );
                }
            }
            ast::StmtKind::Load {
                path,
                span,
            } => {
                let module = self.scope(scope).module;
                let full = self.file_dir(file).join(&**path);
                if self.file_is_loaded(&full, module) {
                    return Err(Box::new(
                        crate::sema::Diagnostic::new(
                            Code::ModuleAlreadyLoaded,
                            *span,
                            format!("`{path}` is already loaded into this module"),
                        )
                        .with_help(
                            "remove this `#load`; the file's declarations are already there",
                        ),
                    ));
                }
                self.load_file(&full, module, *span)?;
            }
            ast::StmtKind::ModuleParameters(boxed) => {
                let crate::ast::ModuleParametersStmt {
                    params,
                    runtime_params,
                    body,
                } = &**boxed;
                let module = self.scope(scope).module;
                if self.ns.main_module == Some(module) {
                    return err(
                        Code::ModuleParameters,
                        stmt.span,
                        "#module_parameters can only be used at the top level of an imported module, not in a program file",
                    );
                }
                let eval_scope = file_scope_for_eval(self, scope, file_scope);
                for import in self.ns.modules[module.0 as usize].argument_imports.clone() {
                    let ordinary = params
                        .iter()
                        .filter_map(|p| p.name.map(|n| n.name))
                        .collect::<Vec<_>>();
                    let program = runtime_params
                        .iter()
                        .filter_map(|p| p.name.map(|n| n.name))
                        .collect::<Vec<_>>();
                    self.validate_import_groups(&import, &ordinary, &program)?;
                }
                for (position, param) in params.iter().chain(runtime_params).enumerate() {
                    self.declare_module_parameter(scope, module, position, param, eval_scope)?;
                }
                self.validate_module_arguments(module, stmt.span)?;
                // Reject unknown/duplicate arguments after all header names are known,
                // before checking omissions. Program parameters may bind later.
                for (position, param) in params.iter().enumerate() {
                    if param.default.is_none()
                        && let Some(name) = param.name
                        && !self.ns.modules[module.0 as usize]
                            .params
                            .iter()
                            .any(|(n, ..)| *n == name.name || n.as_str() == format!("${position}"))
                    {
                        return err(
                            Code::ModuleMissingArgumentParameter,
                            param.span,
                            format!("missing argument for module parameter `{}`", name.name),
                        );
                    }
                }
                let program = runtime_params.iter().filter_map(|p| p.name.map(|n| n.name));
                self.ns.modules[module.0 as usize]
                    .program_params
                    .extend(program);
                // Registering a typed slot may reenter the loader. Check any
                // requests queued during registration before releasing provenance.
                for import in self.ns.modules[module.0 as usize].argument_imports.clone() {
                    self.validate_or_queue_import_groups(module, &import)?;
                }
                self.ns.modules[module.0 as usize].argument_imports.clear();
                // The trailing block declares names the parameter defaults may use.
                if let Some(body) = body {
                    self.declare_stmts(scope, file_scope, &body.stmts, vis)?;
                }
                // Keep contracts separate from declaration registration. Forming
                // an importing hook type may itself need an unrelated public
                // declaration from this module. Checking the default here would
                // consume it before the configured import can finish binding.
                for param in params.iter().chain(runtime_params) {
                    if let Some(pattern) = param.ty.as_ref().filter(|p| procs::has_poly(p)) {
                        let name = param.name.unwrap().name;
                        let id = self.ns.modules[module.0 as usize]
                            .param_entities
                            .iter()
                            .copied()
                            .find(|&id| self.entity(id).name == name)
                            .unwrap();
                        self.ns.modules[module.0 as usize].parameter_contracts.push(
                            ParameterContract {
                                entity: id,
                                pattern: pattern.clone(),
                                scope: eval_scope,
                                state: ParameterContractState::Waiting,
                            },
                        );
                    }
                }
            }
            ast::StmtKind::Using(boxed) => {
                let crate::ast::UsingStmt {
                    value,
                    filter,
                } = &**boxed;
                // `using Module;` / `using SomeStruct;` at file scope: resolved lazily.
                self.push_pending(
                    target,
                    Pending {
                        stmt: ast::Stmt {
                            kind: ast::StmtKind::Using(Box::new(ast::UsingStmt {
                                value: value.clone(),
                                filter: filter.clone(),
                            })),
                            span: stmt.span,
                            notes: ast::Notes::default(),
                        },
                        exported,
                        file_scope,
                        state: PendingState::Waiting,
                    },
                );
            }
            _ => unreachable!("dispatched only for this group's operations"),
        }
        Ok(())
    }

    fn declare_directive_stmt(
        &mut self,
        scope: ScopeId,
        file_scope: ScopeId,
        stmt: &ast::Stmt,
        vis: ast::ScopeKind,
    ) -> Result<()> {
        let target = self.target_scope(scope, file_scope, vis);
        let exported = vis == ast::ScopeKind::Export;
        match &stmt.kind {
            ast::StmtKind::StaticIf(_)
            | ast::StmtKind::StaticSwitch(_)
            | ast::StmtKind::Insert(_) => {
                self.push_pending(
                    target,
                    Pending {
                        stmt: stmt.clone(),
                        exported,
                        file_scope,
                        state: PendingState::Waiting,
                    },
                );
            }
            ast::StmtKind::Run(expr) => self.ct.top_level_runs.push(((**expr).clone(), file_scope)),
            ast::StmtKind::Assert(boxed) => {
                let crate::ast::AssertStmt {
                    cond,
                    message,
                    args,
                } = &**boxed;
                self.ct
                    .asserts
                    .push((cond.clone(), message.clone(), args.clone(), file_scope));
            }
            ast::StmtKind::AddContext(decl) => {
                self.ns.add_contexts.push((decl.clone(), file_scope));
                // Context already made (a `#run` needed it): extend it while its layout
                // is still open.
                if let Some(ty) = self.ty.context_type {
                    let s = self.ty.types.as_struct(ty).unwrap();
                    if self.ty.types.struct_info(s).layout == crate::types::LayoutState::Pending {
                        if let Some(src) = self.ty.struct_asts.get_mut(&s) {
                            src.extra.push((decl.clone(), file_scope));
                        }
                    } else {
                        self.extend_context(ty, decl, file_scope, stmt.span)?;
                    }
                }
            }
            ast::StmtKind::Placeholder(names) => {
                for name in names {
                    self.add_entity(
                        target,
                        name.name,
                        name.span,
                        EntityKind::Placeholder,
                        exported,
                    );
                }
            }
            ast::StmtKind::Directive(directive)
                if directive.kind == ast::DirectiveKind::PokeName =>
            {
                let [
                    module,
                    ast::Expr {
                        kind: ast::ExprKind::Ident(poked),
                        ..
                    },
                ] = &directive.args[..]
                else {
                    return err(
                        Code::ModulePokeName,
                        stmt.span,
                        "#poke_name takes a module and a name",
                    );
                };
                self.ns.pokes.push((module.clone(), *poked, file_scope));
            }
            // An unnamed `#library,link_always "c++";` is linked although nothing refers to it.
            ast::StmtKind::Directive(directive)
                if directive.kind.is_library()
                    && directive
                        .flags
                        .iter()
                        .any(|f| f.name.as_str() == "link_always") =>
            {
                let ast::DirectiveStmt {
                    kind,
                    flags,
                    args,
                    ..
                } = &**directive;
                let system = matches!(
                    kind,
                    ast::DirectiveKind::Library {
                        system: true
                    }
                );
                self.declare_library(file_scope, system, flags, args.first(), stmt.span)?;
            }
            ast::StmtKind::Directive(_) | ast::StmtKind::Empty => {}
            _ => unreachable!("dispatched only for this group's operations"),
        }
        Ok(())
    }

    /// Apply `#poke_name Module name;`: the declarations called `name` visible at the directive
    /// also become declarations of `Module`, joining its overload set.
    pub fn apply_pokes(&mut self) -> Result<()> {
        while let Some((module_expr, name, file_scope)) = self.ns.pokes.pop() {
            let lower::Operand::Module(module) = self.eval_const(file_scope, &module_expr, None)?
            else {
                return err(
                    Code::ModulePokeName2,
                    module_expr.span,
                    "#poke_name needs a module",
                );
            };
            let module_scope = self.ns.modules[module.0 as usize].scope;
            // The module sees the same entity (a copy would resolve to a second,
            // distinct type).
            for id in self.lookup(file_scope, name)? {
                if !self
                    .scope(module_scope)
                    .names
                    .get(&name)
                    .is_some_and(|n| n.contains(&id))
                {
                    self.bind_name(module_scope, name, id);
                }
            }
        }
        Ok(())
    }

    /// Hook for declaration notes (e.g. `@Something` on procedures); currently informational.
    fn note_declaration(&mut self, _id: EntityId, _decl: &ast::Decl) {
    }

    /// Expand one pending item (`#if`, top-level `#insert`, file-scope `using`).
    pub(in crate::sema) fn expand_pending_item(
        &mut self,
        scope: ScopeId,
        file_scope: ScopeId,
        stmt: &ast::Stmt,
        exported: bool,
    ) -> Result<()> {
        let vis = if exported {
            ast::ScopeKind::Export
        } else if scope == file_scope {
            ast::ScopeKind::File
        } else {
            ast::ScopeKind::Module
        };
        match &stmt.kind {
            ast::StmtKind::StaticIf(boxed) => {
                let crate::ast::StaticIfStmt {
                    cond,
                    then_branch,
                    else_branch,
                    ..
                } = &**boxed;
                let taken =
                    self.eval_static_condition(file_scope_for_eval(self, scope, file_scope), cond)?;
                let branch = if taken {
                    then_branch
                } else {
                    else_branch
                };
                self.ide_note_branches(boxed, taken);
                self.declare_stmts(scope, file_scope, branch, vis)?;
            }
            ast::StmtKind::StaticSwitch(boxed) => {
                let crate::ast::SwitchStmt {
                    value,
                    cases,
                    ..
                } = &**boxed;
                let eval_scope = file_scope_for_eval(self, scope, file_scope);
                if let Some(case) = self.static_switch_case(eval_scope, value, cases)? {
                    self.declare_stmts(scope, file_scope, &case.body, vis)?;
                }
            }
            ast::StmtKind::Insert(boxed) => {
                let crate::ast::InsertStmt {
                    value, ..
                } = &**boxed;
                let stmts =
                    self.eval_insert_stmts(file_scope_for_eval(self, scope, file_scope), value)?;
                self.declare_stmts(scope, file_scope, &stmts, vis)?;
            }
            ast::StmtKind::Using(boxed) => {
                let crate::ast::UsingStmt {
                    value,
                    filter,
                } = &**boxed;
                let eval_scope = file_scope_for_eval(self, scope, file_scope);
                let entry = self.using_target(eval_scope, value)?;
                let entry = self.filter_using_entry(eval_scope, entry, filter, stmt.span)?;
                if exported
                    && entry.member_type().is_some()
                    && self.scope(scope).kind == ScopeKind::Module
                {
                    let module = self.scope(scope).module;
                    self.ns.modules[module.0 as usize]
                        .exported_usings
                        .push(entry.clone());
                }
                if let UsingEntry::Module(used, filter) = &entry
                    && self.scope(scope).kind == ScopeKind::Module
                {
                    let module = self.scope(scope).module;
                    self.ns.modules[module.0 as usize].using_modules.push(
                        crate::sema::ModuleUsing {
                            module: *used,
                            filter: filter.clone(),
                            exported,
                        },
                    );
                }
                self.push_using(scope, entry);
            }
            _ => unreachable!("only conditional items are pending"),
        }
        Ok(())
    }
}

/// Conditions inside a module-level pending item are evaluated from the file scope
/// that wrote them, so file-private names are visible.
pub(in crate::sema) fn file_scope_for_eval(
    c: &Compiler,
    scope: ScopeId,
    file_scope: ScopeId,
) -> ScopeId {
    if c.scope(scope).kind == ScopeKind::Module {
        file_scope
    } else {
        scope
    }
}

/// `name :: value;`, a synthesized constant declaration.
fn const_alias(name: ast::Ident, value: ast::Expr) -> Rc<ast::Decl> {
    Rc::new(ast::Decl {
        id: ast::AstId::fresh(),
        names: vec![name],
        kind: ast::DeclKind::Const,
        ty: None,
        span: value.span,
        value: Some(value),
        using: false,
        as_: false,
        backtick: false,
        notes: ast::Notes::default(),
        rare: None,
    })
}
