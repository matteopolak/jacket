//! Members reached through `using`, including override targets.
use crate::ast::ExprKind as E;
use crate::sema::lower::{FnCtx, Operand};
use crate::sema::scope::{EntityKind, Found, Resolved, UsingEntry};
use crate::sema::{Compiler, Result, ScopeId, Span, Sym, TypeId, ast, err};
use crate::types::StructId;

impl Compiler {
    pub fn using_member(
        &mut self,
        f: &mut FnCtx,
        entry: UsingEntry,
        member: Sym,
        span: Span,
    ) -> Result<Operand> {
        match entry {
            UsingEntry::Filtered(base, filter) => {
                let Some(target) = crate::sema::using::filter_target(&filter, member) else {
                    return err(span, format!("using filter excludes `{member}`"));
                };
                self.using_member(f, *base, target, span)
            }
            UsingEntry::Type(t) => self.type_member(f, ScopeId(0), t, member, span),
            UsingEntry::Module(m, _) => match self.module_lookup(m, member)? {
                Found::Using(entry, member) => self.using_member(f, entry, member, span),
                Found::Entities(ids) => self.entities_operand(f, ScopeId(0), &ids, span),
            },
            UsingEntry::Place {
                ty,
                entity,
            } => {
                let base = match self.entity(entity).kind.clone() {
                    EntityKind::Local {
                        ty,
                        addr,
                        ..
                    } => Operand::Place {
                        ty,
                        addr,
                    },
                    _ => match self.resolve_entity(entity)? {
                        Resolved::Global {
                            storage,
                            ty,
                        } => Operand::Place {
                            ty,
                            addr: f.b.storage_addr(storage),
                        },
                        _ => return err(span, "invalid using target"),
                    },
                };
                let _ = ty;
                self.member_access(f, ScopeId(0), base, member, span)
            }
        }
    }

    /// Resolve the target of a `using` statement at file/struct scope.
    pub fn using_target(&mut self, scope: ScopeId, value: &ast::Expr) -> Result<UsingEntry> {
        // `using global_var;` names a variable, which is not a constant but is a valid target.
        // Its type is all that matters, so it is not run to read its value.
        if let E::Ident(name) = &value.kind
            && let Some(&id) = self.lookup(scope, *name)?.first()
            && let EntityKind::Decl {
                decl, ..
            } = &self.ns.entities[id.0 as usize].kind
            && decl.kind == ast::DeclKind::Var
            && let Resolved::Global {
                ty, ..
            } = self.resolve_entity(id)?
        {
            return Ok(UsingEntry::Place {
                ty,
                entity: id,
            });
        }
        match self.eval_const_or_run(scope, value, None)? {
            Operand::Module(m) => Ok(UsingEntry::Module(m, None)),
            Operand::Type(t) => Ok(UsingEntry::Type(t)),
            other => {
                // `using global_var;`
                if let E::Ident(name) = &value.kind {
                    let ids = self.lookup(scope, *name)?;
                    if let Some(&id) = ids.first()
                        && let Resolved::Global {
                            ty, ..
                        } = self.resolve_entity(id)?
                    {
                        return Ok(UsingEntry::Place {
                            ty,
                            entity: id,
                        });
                    }
                }
                // `using module.global_var;`
                if let E::Member(base, member) = &value.kind
                    && let Operand::Module(m) = self.eval_const(scope, base, None)?
                    && let Found::Entities(ids) = self.module_member_lookup(m, member.name)?
                    && let Some(&id) = ids.first()
                    && let Resolved::Global {
                        ty, ..
                    } = self.resolve_entity(id)?
                {
                    return Ok(UsingEntry::Place {
                        ty,
                        entity: id,
                    });
                }
                err(
                    value.span,
                    format!("cannot use `using` on {}", self.describe(&other)),
                )
            }
        }
    }

    /// Whether `name`, reached through the `using` place `entry`, lies behind a pointer field
    /// instead of inside the place itself.
    pub(in crate::sema) fn member_through_pointer(
        &mut self,
        entry: &UsingEntry,
        name: Sym,
    ) -> Result<bool> {
        let UsingEntry::Place {
            ty, ..
        } = entry
        else {
            return Ok(false);
        };
        let ty = self.ty.types.repr_struct(*ty);
        let Some(s) = self.ty.types.as_struct(ty) else {
            return Ok(false);
        };
        Ok(self.struct_member_behind_pointer(s, name, false, &mut Vec::new())? == Some(true))
    }

    /// `Some(true)` when `name` of `s` (or of a struct it takes in with `using`) is reached
    /// through a pointer, `Some(false)` when directly, `None` when `s` has no such member.
    fn struct_member_behind_pointer(
        &mut self,
        s: StructId,
        name: Sym,
        behind: bool,
        searched: &mut Vec<StructId>,
    ) -> Result<Option<bool>> {
        if searched.contains(&s) {
            return Ok(None);
        }
        searched.push(s);
        self.layout_struct(s, Span::NONE)?;
        let fields = self.ty.types.struct_info(s).fields.clone();
        if fields.iter().any(|f| f.name == Some(name)) {
            return Ok(Some(behind));
        }
        for field in fields.iter().filter(|f| f.using) {
            let (inner, through) = match self.ty.types.pointee(field.ty) {
                Some(p) => (p, true),
                None => (field.ty, behind),
            };
            let inner = self.ty.types.repr_struct(inner);
            if let Some(inner) = self.ty.types.as_struct(inner)
                && let Some(found) =
                    self.struct_member_behind_pointer(inner, name, through || behind, searched)?
            {
                return Ok(Some(found));
            }
        }
        Ok(None)
    }

    // Default values

    /// Offset, type and authored declaration of `name` in a `using` field of `s`.
    fn find_used_member(
        &mut self,
        s: StructId,
        name: Sym,
        span: Span,
    ) -> Result<Option<(u64, TypeId, Span)>> {
        self.layout_struct(s, span)?;
        let fields = self.ty.types.struct_info(s).fields.clone();
        for f in fields.iter().filter(|f| f.using) {
            let Some(inner) = self.ty.types.as_struct(f.ty) else {
                continue;
            };
            self.layout_struct(inner, span)?;
            let inner_fields = self.ty.types.struct_info(inner).fields.clone();
            if let Some(m) = inner_fields.iter().find(|m| m.name == Some(name)) {
                return Ok(Some((f.offset + m.offset, m.ty, m.span)));
            }
            if let Some((o, t, decl)) = self.find_used_member(inner, name, span)? {
                return Ok(Some((f.offset + o, t, decl)));
            }
        }
        Ok(None)
    }

    /// Offset and type of the member a struct-body override assigns: `name` (a field, or a
    /// member of a `using` field) or a path into a field (`base.callback`).
    pub(super) fn override_target(
        &mut self,
        s: StructId,
        lhs: &ast::Expr,
        span: Span,
    ) -> Result<Option<(u64, TypeId)>> {
        match &lhs.kind {
            E::Ident(name) => Ok(self
                .struct_member(s, *name, span)?
                .map(|(offset, ty, decl)| {
                    self.ide_note_named_field(lhs.span, *name, decl, ty);
                    (offset, ty)
                })),
            E::Member(base, name) => {
                let Some((offset, ty)) = self.override_path(s, base, span)? else {
                    return Ok(None);
                };
                let Some(inner) = self.ty.types.as_struct(ty) else {
                    return Ok(None);
                };
                Ok(self
                    .struct_member(inner, name.name, span)?
                    .map(|(o, t, decl)| {
                        self.ide_note_named_field(name.span, name.name, decl, t);
                        (offset + o, t)
                    }))
            }
            _ => Ok(None),
        }
    }

    /// `override_target` for an intermediate path segment, which may also be a direct field.
    fn override_path(
        &mut self,
        s: StructId,
        expr: &ast::Expr,
        span: Span,
    ) -> Result<Option<(u64, TypeId)>> {
        if let E::Ident(name) = &expr.kind {
            return Ok(self
                .struct_member(s, *name, span)?
                .map(|(offset, ty, decl)| {
                    self.ide_note_named_field(expr.span, *name, decl, ty);
                    (offset, ty)
                }));
        }
        self.override_target(s, expr, span)
    }

    /// Offset, type and authored declaration of a field, directly or through `using` fields.
    fn struct_member(
        &mut self,
        s: StructId,
        name: Sym,
        span: Span,
    ) -> Result<Option<(u64, TypeId, Span)>> {
        self.layout_struct(s, span)?;
        let fields = self.ty.types.struct_info(s).fields.clone();
        if let Some(f) = fields.iter().find(|f| f.name == Some(name)) {
            return Ok(Some((f.offset, f.ty, f.span)));
        }
        self.find_used_member(s, name, span)
    }
}
