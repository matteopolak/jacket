//! Compile-time using filters and aggregate name maps. Names refer back to the
//! original member; mapping never creates copied runtime fields.
use super::scope::UsingEntry;
use super::value::Aggregate;
use super::*;
use crate::types::{ArrayKind, TypeKind};

pub(super) fn filter_target(filter: &ast::UsingFilter, name: Sym) -> Option<Sym> {
    match filter {
        ast::UsingFilter::Only(names) => names.iter().any(|n| n.name == name).then_some(name),
        ast::UsingFilter::Except(names) => names.iter().all(|n| n.name != name).then_some(name),
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
        } = *self.types.kind(ty)
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
        let g = self.program.add_global(ir::Global {
            name: "using.names".into(),
            size: size.max(1),
            align,
            init: image.bytes,
            relocs: image.relocs,
            read_only: false,
            export: None,
        });
        let addr = self.interp.global_addr(&self.program, g).map_err(|trap| {
            Box::new(self.trap_diagnostic(&trap, "cannot read using filter", Some(span)))
        })?;
        let (count, data) = match kind {
            ArrayKind::Fixed(n) => (n, addr),
            _ => {
                let Some(n) = self.interp.read_u64(addr) else {
                    return err(span, "unreadable using filter count");
                };
                let Some(data) = self.interp.read_u64(addr + 8) else {
                    return err(span, "unreadable using filter data");
                };
                (n, data)
            }
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
        if let UsingEntry::Module(m, _) = entry {
            let filter = self.resolve_name_filter(scope, filter)?;
            if matches!(filter, ast::UsingFilter::Map(_)) {
                return err(
                    span,
                    "using,map on modules is not supported by the reviewed contract",
                );
            }
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
            let ty = self.types.pointee(ty).unwrap_or(ty);
            self.ensure_complete(ty, span)?;
            let names = self.aggregate_using_names(ty)?;
            let mapper = self.eval_const_value(scope, expr)?;
            if !matches!(mapper, Value::Proc(_)) {
                return err(
                    expr.span,
                    "using mapper must be a single compile-time procedure",
                );
            }
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
            let file = self
                .sources
                .add("<using mapper>".into(), source.clone().into());
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
            ast::UsingFilter::Map(
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
                                name: old,
                                span,
                            },
                        )
                    })
                    .collect(),
            )
        } else {
            self.resolve_name_filter(scope, filter)?
        };
        Ok(UsingEntry::Filtered(Box::new(entry), Rc::new(filter)))
    }

    fn aggregate_using_names(&mut self, ty: TypeId) -> Result<Vec<Sym>> {
        let mut names = Vec::new();
        if let TypeKind::Enum(e) = *self.types.kind(ty) {
            names.extend(self.types.enum_info(e).members.iter().map(|(n, _)| *n));
        } else if let Some(s) = self.types.as_struct(self.types.repr_struct(ty)) {
            for field in self.types.struct_info(s).fields.clone() {
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
}
