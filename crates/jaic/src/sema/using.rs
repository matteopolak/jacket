//! Compile-time using filters and aggregate name maps. Names refer back to the
//! original member; mapping never creates copied runtime fields.
use super::scope::UsingEntry;
use super::value::Aggregate;
use super::{
    Compiler, Diagnostic, EntityId, ModuleId, Rc, Result, ScopeId, SourcePath, Span, Sym, TypeId,
    Value, ast, err, ir, scope,
};
use crate::types::{ArrayKind, TypeKind};

pub(crate) fn filter_target(filter: &ast::UsingFilter, name: Sym) -> Option<Sym> {
    let public_name = public_filter_name(name);
    match filter {
        ast::UsingFilter::Only(names) => {
            names.iter().any(|n| n.name == public_name).then_some(name)
        }
        ast::UsingFilter::Except(names) => {
            names.iter().all(|n| n.name != public_name).then_some(name)
        }
        ast::UsingFilter::Map(pairs) => pairs
            .iter()
            .find(|(new, _)| new.name == name)
            .map(|(_, old)| old.name),
        ast::UsingFilter::None => Some(name),
        ast::UsingFilter::Computed {
            ..
        } => None,
    }
}

/// Filters use public operator symbols; lookup still uses the original declaration.
/// Translate only operator forms, leaving ordinary names intact.
fn public_filter_name(name: Sym) -> Sym {
    match name.as_str().strip_prefix("operator") {
        Some(
            symbol @ ("+" | "-" | "*" | "/" | "!" | "==" | "!=" | "<" | "<=" | ">" | ">=" | "[]"
            | "[]=" | "*[]"),
        ) => Sym::intern(symbol),
        _ => name,
    }
}

#[expect(
    clippy::unnecessary_box_returns,
    reason = "the diagnostics are boxed because the error type carries them boxed; callers pass them through `Err`"
)]
impl Compiler {
    /// Resolve a computed only/except list once. Mapper calls require an aggregate.
    pub(super) fn resolve_name_filter(
        &mut self,
        scope: ScopeId,
        filter: &ast::UsingFilter,
    ) -> Result<ast::UsingFilter> {
        let ast::UsingFilter::Computed {
            kind,
            expr,
        } = filter
        else {
            return Ok(filter.clone());
        };
        if *kind == ast::UsingFilterKind::Map {
            return err(
                expr.span,
                "using,map requires an aggregate member target; module mapping is not supported",
            );
        }
        let (value, ty) = self.eval_const_typed(scope, expr)?;
        let names = self
            .filter_strings(&value, ty, expr.span)?
            .into_iter()
            .map(|name| ast::Ident {
                name,
                span: expr.span,
            })
            .collect();
        Ok(match kind {
            ast::UsingFilterKind::Only => ast::UsingFilter::Only(names),
            ast::UsingFilterKind::Except => ast::UsingFilter::Except(names),
            ast::UsingFilterKind::Map => unreachable!(),
        })
    }

    /// Read an independently evaluated array of strings, including frozen views.
    fn filter_strings(&mut self, value: &Value, ty: TypeId, span: Span) -> Result<Vec<Sym>> {
        let TypeKind::Array {
            elem,
            kind,
        } = *self.ty.types.kind(ty)
        else {
            return err(span, "a computed using filter must be an array of strings");
        };
        if elem != TypeId::STRING {
            return err(span, "a computed using filter must contain strings");
        }
        let size = self.size_of(ty, span)?;
        let align = self.align_of(ty, span)?;
        let mut image = Aggregate::zeroed(size, span)?;
        self.write_value(&mut image, 0, value, ty, span)?;
        let g = self.out.program.add_global(ir::Global {
            name: "using.names".into(),
            size: size.max(1),
            align,
            init: image.bytes,
            relocs: image.relocs,
            read_only: false,
            export: None,
        });
        let addr = self
            .ct
            .interp
            .global_addr(&self.out.program, g)
            .map_err(|trap| {
                Box::new(self.trap_diagnostic(&trap, "cannot read using filter", Some(span)))
            })?;
        let (count, data) = if let ArrayKind::Fixed(n) = kind {
            (n, addr)
        } else {
            let Some(n) = self.ct.interp.read_u64(addr) else {
                return err(span, "unreadable using filter count");
            };
            let Some(data) = self.ct.interp.read_u64(addr + 8) else {
                return err(span, "unreadable using filter data");
            };
            (n, data)
        };
        if count > super::value::MAX_IMAGE / 16 {
            return err(span, "using filter array is too large");
        }
        let mut names = Vec::new();
        for i in 0..count {
            let Some(at) = i
                .checked_mul(16)
                .and_then(|offset| data.checked_add(offset))
            else {
                return err(span, "using filter address overflow");
            };
            let Value::String(bytes) = self.read_value(at, TypeId::STRING, span)? else {
                return err(span, "using filter element is not a string");
            };
            let text = std::str::from_utf8(&bytes)
                .map_err(|_| Box::new(Diagnostic::error(span, "using member name is not UTF-8")))?;
            names.push(Sym::intern(text));
        }
        Ok(names)
    }

    pub(super) fn filter_using_entry(
        &mut self,
        scope: ScopeId,
        entry: UsingEntry,
        filter: &ast::UsingFilter,
        span: Span,
    ) -> Result<UsingEntry> {
        if matches!(filter, ast::UsingFilter::None) {
            return Ok(entry);
        }
        self.ide_note_filter_names(&entry, filter);
        if let UsingEntry::Module(m, _) = entry {
            let filter = self.resolve_module_filter(scope, m, filter, span)?;
            return Ok(UsingEntry::Module(m, Some(Rc::new(filter))));
        }
        let filter = if let ast::UsingFilter::Computed {
            kind: ast::UsingFilterKind::Map,
            expr,
        } = filter
        {
            let ty = match &entry {
                UsingEntry::Place {
                    ty, ..
                }
                | UsingEntry::Type(ty) => *ty,
                _ => return err(span, "using mapper requires an aggregate"),
            };
            let ty = self.ty.types.pointee(ty).unwrap_or(ty);
            self.ensure_complete(ty, span)?;
            let names = self.aggregate_using_names(ty)?;
            let mapper = self.eval_const_value(scope, expr)?;
            if !matches!(mapper, Value::Proc(_)) {
                return err(
                    expr.span,
                    "using mapper must be a single compile-time procedure",
                );
            }
            self.map_member_names(scope, expr, mapper, &names, span)?
        } else {
            self.resolve_name_filter(scope, filter)?
        };
        Ok(UsingEntry::Filtered(Box::new(entry), Rc::new(filter)))
    }

    /// `resolve_name_filter` for a filter on `module`, where a mapper also sees the
    /// names the module exports.
    pub(super) fn resolve_module_filter(
        &mut self,
        scope: ScopeId,
        module: ModuleId,
        filter: &ast::UsingFilter,
        span: Span,
    ) -> Result<ast::UsingFilter> {
        let ast::UsingFilter::Computed {
            kind: ast::UsingFilterKind::Map,
            expr,
        } = filter
        else {
            return self.resolve_name_filter(scope, filter);
        };
        // Naming the mapper must not expand the importing module's other pending items: they
        // can need the names this very mapper is about to rename.
        let saved = std::mem::replace(&mut self.epochs.lookup_without_expansion, true);
        let quick = self.eval_const_value(scope, expr);
        self.epochs.lookup_without_expansion = saved;
        let mapper = match quick {
            Ok(mapper) => mapper,
            Err(_) => self.eval_const_value(scope, expr)?,
        };
        if !matches!(mapper, Value::Proc(_)) {
            return err(
                expr.span,
                "using mapper must be a single compile-time procedure",
            );
        }
        let mut names = Vec::new();
        self.module_exported_names(module, &mut Vec::new(), &mut names)?;
        names.sort_by_key(|name| name.as_str());
        self.map_member_names(scope, expr, mapper, &names, span)
    }

    /// The names `module` and the modules it re-exports declare for importers.
    fn module_exported_names(
        &mut self,
        module: ModuleId,
        seen: &mut Vec<ModuleId>,
        out: &mut Vec<Sym>,
    ) -> Result<()> {
        if seen.contains(&module) {
            return Ok(());
        }
        seen.push(module);
        let scope = self.ns.modules[module.0 as usize].scope;
        self.expand_pending(scope)?;
        let mut declared: Vec<Sym> = self
            .scope(scope)
            .names
            .iter()
            .filter(|(_, ids)| {
                ids.iter().any(|&id| {
                    let e = self.entity(id);
                    e.exported && !e.file_private
                })
            })
            .map(|(name, _)| *name)
            .collect();
        out.append(&mut declared);
        for index in self.ns.modules[module.0 as usize]
            .exported_using_imports
            .clone()
        {
            if let Some(inner) = self.import_module(scope, index)? {
                self.module_exported_names(inner, seen, out)?;
            }
        }
        Ok(())
    }

    /// Run the compile-time `mapper` over the original `names` and pair each kept
    /// result with the name it replaces; an empty result drops the name.
    fn map_member_names(
        &mut self,
        scope: ScopeId,
        expr: &ast::Expr,
        mapper: Value,
        names: &[Sym],
        span: Span,
    ) -> Result<ast::UsingFilter> {
        let module = self.scope(scope).module;
        let map_scope = self.new_scope(scope::ScopeKind::Proc, Some(scope), module, None);
        let proc_ty = if let Value::Proc(p) = mapper {
            self.proc_type(p, span)?
        } else {
            unreachable!()
        };
        self.add_entity(
            map_scope,
            Sym::intern("__using_mapper"),
            expr.span,
            scope::EntityKind::Const {
                value: mapper,
                ty: proc_ty,
            },
            false,
        );
        // A writable array of copied string views is passed to the mapper. The
        // source spelling is solely original member names, never arbitrary code.
        let list = names
            .iter()
            .map(|n| format!("{:?}", n.as_str()))
            .collect::<Vec<_>>()
            .join(", ");
        let source = format!(
            "__mapped :: #run (() -> [{}] string {{ __names: [{}] string = .[{list}]; __using_mapper(__names); return __names; }})();",
            names.len(),
            names.len()
        );
        let file = self.src.sources.add(
            SourcePath::virtual_source("using mapper"),
            source.clone().into(),
        );
        let parsed = crate::parser::parse_file(file, &source).map_err(Box::new)?;
        let ast::StmtKind::Decl(decl) = &parsed.stmts[0].kind else {
            unreachable!()
        };
        let result = decl.value.as_ref().unwrap();
        let (value, ty) = self.eval_const_typed(map_scope, result)?;
        let mapped = self.filter_strings(&value, ty, expr.span)?;
        if mapped.len() != names.len() {
            return err(expr.span, "using mapper changed the number of member names");
        }
        Ok(ast::UsingFilter::Map(
            mapped
                .into_iter()
                .zip(names)
                .filter(|(new, _)| !new.as_str().is_empty())
                .map(|(new, old)| {
                    (
                        ast::Ident {
                            name: new,
                            span,
                        },
                        ast::Ident {
                            name: *old,
                            span,
                        },
                    )
                })
                .collect(),
        ))
    }

    fn aggregate_using_names(&mut self, ty: TypeId) -> Result<Vec<Sym>> {
        let mut names = Vec::new();
        if let TypeKind::Enum(e) = *self.ty.types.kind(ty) {
            names.extend(self.ty.types.enum_info(e).members.iter().map(|(n, _)| *n));
        } else if let Some(s) = self.ty.types.as_struct(self.ty.types.repr_struct(ty)) {
            for field in self.ty.types.struct_info(s).fields.clone().iter() {
                names.extend(field.name);
                if field.using {
                    names.extend(self.aggregate_using_names(field.ty)?);
                }
            }
        }
        Ok(names)
    }

    /// Resolve the exposed name back to its target, preserving storage identity.
    pub(super) fn aggregate_using_member(
        &mut self,
        entry: &UsingEntry,
        name: Sym,
    ) -> Result<Option<(UsingEntry, Sym)>> {
        match entry {
            UsingEntry::Filtered(base, filter) => match filter_target(filter, name) {
                Some(target) => self.aggregate_using_member(base, target),
                None => Ok(None),
            },
            UsingEntry::Place {
                ty, ..
            }
            | UsingEntry::Type(ty) => Ok(self
                .type_has_member(*ty, name)?
                .then(|| (entry.clone(), name))),
            UsingEntry::Module(..) => Ok(None),
        }
    }

    /// Add the `using` entry of a variable to `scope`. Constants and procedures the block
    /// already declared (they are declared before its statements run) may not share a name with
    /// a member the entry brings in.
    pub(in crate::sema) fn push_variable_using(
        &mut self,
        scope: ScopeId,
        entry: UsingEntry,
    ) -> Result<()> {
        let mut declared: Vec<(Sym, Span)> = Vec::new();
        for (name, ids) in &self.scope(scope).names {
            for &id in ids {
                if matches!(self.entity(id).kind, scope::EntityKind::Decl { .. }) {
                    declared.push((*name, self.entity(id).span));
                }
            }
        }
        for (name, span) in declared {
            if name.as_str() != "_" && self.aggregate_using_member(&entry, name)?.is_some() {
                return Err(self.using_member_clash(name, span, &entry));
            }
        }
        self.push_using(scope, entry);
        Ok(())
    }

    /// A declaration in the scope that holds a `using` of a variable may not reuse the name of
    /// one of that variable's members: `using o; x := 1;` with `x` a member of `o`. A nested
    /// scope shadows the member as it shadows any outer name.
    pub(in crate::sema) fn reject_using_redeclaration(
        &mut self,
        scope: ScopeId,
        name: Sym,
        span: Span,
    ) -> Result<()> {
        if name.as_str() == "_" {
            return Ok(());
        }
        for k in 0..self.scope(scope).usings.len() {
            let entry = self.scope(scope).usings[k].clone();
            if entry.place_entity().is_some()
                && self.aggregate_using_member(&entry, name)?.is_some()
            {
                return Err(self.using_member_clash(name, span, &entry));
            }
        }
        Ok(())
    }

    /// The error for declaring `name` at `span` in a scope where `using` already brings in a
    /// member of that name through the variable of `entry`.
    fn using_member_clash(&self, name: Sym, span: Span, entry: &UsingEntry) -> Box<Diagnostic> {
        let mut d = Diagnostic::error(span, format!("`{name}` is already declared in this scope"))
            .with_label("declared again here");
        let Some(owner) = entry.place_entity().map(|id| self.entity(id)) else {
            return Box::new(d);
        };
        let owner_name = owner.name;
        if owner.span != Span::NONE {
            d = d.with_note(
                owner.span,
                format!("`{name}` is a member of `{owner_name}`, which `using` brings into scope"),
            );
        }
        Box::new(d.with_help(format!(
            "use another name, or write `{owner_name}.{name}` for the member"
        )))
    }

    /// The error for using `name` where the `using` variables `providers` all have a member of
    /// that name.
    pub(in crate::sema) fn ambiguous_using_member(
        &self,
        name: Sym,
        span: Span,
        providers: &[EntityId],
    ) -> Box<Diagnostic> {
        let mut d = Diagnostic::error(
            span,
            format!(
                "`{name}` is ambiguous: more than one `using` variable has a member of that name"
            ),
        );
        let mut spelled = Vec::new();
        for &id in providers {
            let owner = self.entity(id);
            if owner.span != Span::NONE {
                d = d.with_note(
                    owner.span,
                    format!("`{}` has a member `{name}`", owner.name),
                );
            }
            spelled.push(format!("`{}.{name}`", owner.name));
        }
        Box::new(d.with_help(format!("say which one you mean: {}", spelled.join(" or "))))
    }
}
