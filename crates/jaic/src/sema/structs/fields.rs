//! Collecting a struct's fields and the implicit context type.
use super::model::{FieldDecl, FieldItem, MemberAlias, member_path};
use crate::ast::ExprKind as E;
use crate::sema::TypeState;
use crate::sema::lower::Operand;
use crate::sema::scope::{EntityKind, Resolved};
use crate::sema::{Compiler, Rc, Result, ScopeId, Span, Sym, TypeId, Value, ast, err};

impl Compiler {
    pub(super) fn collect_fields(
        &mut self,
        scope: ScopeId,
        stmts: &[ast::Stmt],
        out: &mut Vec<FieldItem>,
    ) -> Result<()> {
        for stmt in stmts {
            match &stmt.kind {
                ast::StmtKind::Decl(decl) if decl.kind == ast::DeclKind::Var => {
                    self.ty.collect_decl_fields(scope, decl, out)?;
                }
                ast::StmtKind::StaticIf(boxed) => {
                    let crate::ast::StaticIfStmt {
                        cond,
                        then_branch,
                        else_branch,
                        ..
                    } = &**boxed;
                    let taken = self.eval_static_condition(scope, cond)?;
                    self.ide_note_branches(boxed, taken);
                    self.collect_fields(
                        scope,
                        if taken {
                            then_branch
                        } else {
                            else_branch
                        },
                        out,
                    )?;
                }
                ast::StmtKind::Insert(boxed) => {
                    let crate::ast::InsertStmt {
                        value, ..
                    } = &**boxed;
                    let inserted = self.eval_insert_stmts(scope, value)?;
                    // Constants the inserted text declares are members like any written in the
                    // body, visible to the fields that follow and to the struct's users.
                    for stmt in &inserted {
                        if let ast::StmtKind::Decl(decl) = &stmt.kind
                            && decl.kind == ast::DeclKind::Const
                        {
                            for (index, n) in decl.names.iter().enumerate() {
                                self.add_entity(
                                    scope,
                                    n.name,
                                    n.span,
                                    EntityKind::Decl {
                                        decl: decl.clone(),
                                        index,
                                    },
                                    true,
                                );
                            }
                        }
                    }
                    self.collect_fields(scope, &inserted, out)?;
                }
                ast::StmtKind::Place(e) => {
                    let E::Ident(name) = &e.kind else {
                        return err(e.span, "#place needs a field name");
                    };
                    out.push(FieldItem::Place(*name, e.span));
                }
                ast::StmtKind::Overlay(e) => {
                    let E::Ident(name) = &e.kind else {
                        return err(e.span, "#overlay needs a field name");
                    };
                    out.push(FieldItem::Overlay(*name, e.span));
                }
                ast::StmtKind::Expr(e) => match &e.kind {
                    E::Struct(lit) => {
                        // Anonymous nested struct/union: its members are reachable directly.
                        self.check_struct_modify(Sym::intern("anonymous"), lit)?;
                        let t = self.new_struct_type(
                            Sym::intern("anonymous"),
                            lit.clone(),
                            scope,
                            Vec::new(),
                            None,
                        );
                        out.push(FieldItem::Field(FieldDecl {
                            name: None,
                            ty: t,
                            using: true,
                            as_: false,
                            notes: Vec::new(),
                            span: e.span,
                            init: None,
                            scope,
                            align: None,
                            filter: None,
                        }));
                    }
                    _ => return err(e.span, "unexpected expression in struct body"),
                },
                ast::StmtKind::Using(boxed) => {
                    let crate::ast::UsingStmt {
                        value,
                        filter,
                    } = &**boxed;
                    // `using inner.member;` naming fields of this struct: an alias.
                    if let Some(path) = member_path(value)
                        && out.iter().any(|item| match item {
                            FieldItem::Field(d) => d.name == Some(path[0]),
                            FieldItem::Declaration(_, d) => {
                                d.names.iter().any(|n| n.name == path[0])
                            }
                            _ => false,
                        })
                    {
                        out.push(FieldItem::Alias(MemberAlias {
                            path,
                            filter: filter.clone(),
                        }));
                        continue;
                    }
                    // `using Other_Struct;` imports another struct's fields.
                    let t = self.eval_type(scope, value)?;
                    out.push(FieldItem::Field(FieldDecl {
                        name: None,
                        ty: t,
                        using: true,
                        as_: false,
                        notes: Vec::new(),
                        span: stmt.span,
                        init: None,
                        scope,
                        align: None,
                        filter: Some(filter.clone()),
                    }));
                }
                _ => {}
            }
        }
        Ok(())
    }

    pub(in crate::sema) fn resolve_field_decl_type(
        &mut self,
        scope: ScopeId,
        decl: &Rc<ast::Decl>,
    ) -> Result<TypeId> {
        if let Some(ty) = decl
            .names
            .first()
            .and_then(|n| self.ty.field_types.get(&scope)?.get(&n.name))
        {
            return Ok(*ty);
        }
        let key = (scope, decl.id);
        if !self.ty.field_type_pending.insert(key) {
            return err(decl.span, "cycle in struct field types");
        }
        let result = self.field_decl_type_inner(scope, decl);
        self.ty.field_type_pending.remove(&key);
        if let Ok(ty) = result {
            for name in &decl.names {
                self.ty
                    .field_types
                    .entry(scope)
                    .or_default()
                    .insert(name.name, ty);
            }
        }
        result
    }

    fn field_decl_type_inner(&mut self, scope: ScopeId, decl: &Rc<ast::Decl>) -> Result<TypeId> {
        let ty = match (&decl.ty, &decl.value) {
            (Some(t), _) => self.eval_type(scope, t)?,
            (None, Some(v)) => {
                let op = self.eval_const(scope, v, None)?;
                match op {
                    Operand::Const {
                        ty,
                        value,
                        untyped: true,
                    } => self.ty.default_untyped(ty, &value),
                    Operand::Type(_) => TypeId::TYPE,
                    Operand::Procs(p) if p.len() == 1 => self.proc_type(p[0], v.span)?,
                    other => other.ty(),
                }
            }
            (None, None) => return err(decl.span, "field needs a type"),
        };
        Ok(ty)
    }

    pub(super) fn resolve_decl_fields(
        &mut self,
        scope: ScopeId,
        decl: &Rc<ast::Decl>,
        out: &mut Vec<FieldItem>,
    ) -> Result<()> {
        let ty = self.resolve_field_decl_type(scope, decl)?;
        let filter = self.field_using_filter(scope, decl, ty)?;
        let align = match &decl.align() {
            Some(a) => Some(self.eval_align(scope, a)?),
            None => None,
        };
        let notes: Vec<Rc<str>> = decl.notes.iter().map(|n| n.text.clone()).collect();
        for name in &decl.names {
            out.push(FieldItem::Field(FieldDecl {
                name: Some(name.name),
                ty,
                using: decl.using,
                as_: decl.as_,
                notes: notes.clone(),
                span: name.span,
                init: decl.value.clone(),
                scope,
                align,
                filter: filter.clone(),
            }));
        }
        Ok(())
    }

    /// The synthesized `#Context` type: Preload's `FIRST_ADD_CONTEXT` followed by
    /// every `#add_context` in load order.
    pub fn context_type(&mut self, span: Span) -> Result<TypeId> {
        if let Some(t) = self.ty.context_type {
            return Ok(t);
        }
        self.expand_all()?;
        let mut extra: Vec<(Rc<ast::Decl>, ScopeId)> = Vec::new();
        if let Some(preload) = self.ns.preload {
            let ids = self.module_declarations(preload, Sym::intern("FIRST_ADD_CONTEXT"))?;
            if let Some(&id) = ids.first()
                && let Resolved::Const {
                    value: Value::Code(code),
                    ..
                } = self.resolve_entity(id)?
            {
                let body = self.pr.codes[code.0 as usize].clone();
                // The base context struct lives in Runtime_Support.
                let scope = match self.ns.runtime_support {
                    Some(m) => self.ns.modules[m.0 as usize].scope,
                    None => self.pr.code_scopes[code.0 as usize],
                };
                let statements = match &*body {
                    ast::CodeBody::Block(block) => block.stmts.as_slice(),
                    ast::CodeBody::Stmt(stmt) => std::slice::from_ref(stmt),
                    ast::CodeBody::Expr(_) => &[],
                };
                for stmt in statements {
                    if let ast::StmtKind::AddContext(decl) = &stmt.kind {
                        extra.push((decl.clone(), scope));
                    }
                }
            }
        }
        extra.extend(self.ns.add_contexts.iter().cloned());
        let lit = Rc::new(ast::StructLit {
            id: ast::AstId::fresh(),
            kind: ast::StructKind::Struct,
            params: Vec::new(),
            body: Vec::new(),
            flags: ast::StructFlags::default(),
            tag: None,
            modify: None,
            notes: Vec::new(),
            span,
        });
        let root = self.ns.root_scope;
        let ty = self.new_struct_type(Sym::intern("Context"), lit, root, Vec::new(), None);
        let s = self.ty.types.as_struct(ty).unwrap();
        // `#add_context name :: value;` adds a constant member (`#Context._Iprof.Zone()`), not a
        // field. It is resolved where it was declared.
        let (consts, extra): (Vec<_>, Vec<_>) = extra
            .into_iter()
            .partition(|(decl, _)| decl.kind == ast::DeclKind::Const);
        let struct_scope = self.ty.struct_asts[&s].scope;
        for (decl, home) in consts {
            self.add_context_constant(struct_scope, &decl, home);
        }
        self.ty.struct_asts.get_mut(&s).unwrap().extra = extra;
        self.ty.context_type = Some(ty);
        self.layout_struct(s, span)?;
        Ok(ty)
    }
}

impl Compiler {
    fn add_context_constant(&mut self, struct_scope: ScopeId, decl: &Rc<ast::Decl>, home: ScopeId) {
        for (index, n) in decl.names.iter().enumerate() {
            let kind = EntityKind::Decl {
                decl: decl.clone(),
                index,
            };
            let id = self.add_entity(struct_scope, n.name, n.span, kind, true);
            self.entity_mut(id).home = home;
        }
    }

    /// Add an `#add_context` met after the Context was laid out, because compile-time code
    /// started before the file declaring it loaded. The member goes after the existing ones, so
    /// their offsets hold, and fits in the room the compile-time context was allocated with; the
    /// bytes of the new member in the live context are set to its default.
    pub fn extend_context(
        &mut self,
        context: TypeId,
        decl: &Rc<ast::Decl>,
        home: ScopeId,
        span: Span,
    ) -> Result<()> {
        let s = self.ty.types.as_struct(context).unwrap();
        let struct_scope = self.ty.struct_asts[&s].scope;
        if decl.kind == ast::DeclKind::Const {
            self.add_context_constant(struct_scope, decl, home);
            return Ok(());
        }
        let before = self.ty.types.struct_info(s).fields.len();
        self.ty
            .struct_asts
            .get_mut(&s)
            .unwrap()
            .extra
            .push((decl.clone(), home));
        self.ty.types.struct_info_mut(s).layout = crate::types::LayoutState::Pending;
        self.ty.default_images.remove(&context);
        self.layout_struct(s, span)?;
        let size = self.ty.types.struct_info(s).size;
        let Some(global) = self.ct.context_global else {
            return Ok(());
        };
        let capacity = self.out.program.globals[global.0 as usize].size;
        if size > capacity {
            return err(
                span,
                format!(
                    "The #Context has gotten bigger than the limit. The maximum size configured in Build_Options is {capacity} bytes, but this declaration would make it {size} bytes."
                ),
            );
        }
        let from = self.ty.types.struct_info(s).fields[before..]
            .iter()
            .map(|f| f.offset)
            .min()
            .unwrap_or(size);
        if let Some(image) = self.default_initializer(context, span)? {
            let live = &mut self.out.program.globals[global.0 as usize];
            live.init.resize(size as usize, 0);
            let tail = (from as usize).min(image.bytes.len());
            live.init[tail..image.bytes.len()].copy_from_slice(&image.bytes[tail..]);
            live.relocs
                .extend(image.relocs.iter().filter(|r| r.offset >= from).cloned());
        }
        self.ct
            .interp
            .initialize_global_tail(&self.out.program, global, from)
            .map_err(|t| Box::new(crate::sema::Diagnostic::error(span, t.into_message())))
    }
}

impl TypeState {
    pub(super) fn collect_decl_fields(
        &mut self,
        scope: ScopeId,
        decl: &Rc<ast::Decl>,
        out: &mut Vec<FieldItem>,
    ) -> Result<()> {
        for name in &decl.names {
            self.field_declarations
                .entry(scope)
                .or_default()
                .insert(name.name, decl.clone());
        }
        out.push(FieldItem::Declaration(scope, decl.clone()));
        Ok(())
    }
}
