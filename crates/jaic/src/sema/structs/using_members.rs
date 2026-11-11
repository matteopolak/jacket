//! Members reached through `using`, including override targets.
use crate::ast::ExprKind as E;
use crate::diagnostics::Code;
use crate::sema::lower::{FnCtx, Operand};
use crate::sema::scope::{EntityKind, Found, Resolved, UsingEntry};
use crate::sema::{Compiler, Result, ScopeId, Span, Sym, TypeId, ast, err};
use crate::types::StructId;

impl Compiler {
    /// `member` named on its own in `scope`, found through the `using` that `entry` records.
    pub fn using_member(
        &mut self,
        f: &mut FnCtx,
        scope: ScopeId,
        entry: UsingEntry,
        member: Sym,
        span: Span,
    ) -> Result<Operand> {
        match entry {
            UsingEntry::Filtered(base, filter) => {
                let Some(target) = crate::sema::using::filter_target(&filter, member) else {
                    return err(
                        Code::UsingFilterInvalid,
                        span,
                        format!("using filter excludes `{member}`"),
                    );
                };
                self.using_member(f, scope, *base, target, span)
            }
            UsingEntry::Type(t) => self.type_member(f, ScopeId(0), t, member, span),
            UsingEntry::Module(m, _) => match self.module_lookup(m, member)? {
                Found::Using(entry, member) => self.using_member(f, scope, entry, member, span),
                Found::Entities(ids) => self.entities_operand(f, ScopeId(0), &ids, span),
            },
            UsingEntry::Place {
                entity, ..
            } => {
                let base = match self.entity(entity).kind {
                    // A local reads like its name would, so `using` of an enclosing procedure's
                    // local in a nested `#run` gives its type's constants and nothing else.
                    EntityKind::Local {
                        ..
                    } => {
                        if let Some(op) =
                            self.outer_local_constant(f, scope, entity, member, span)?
                        {
                            return Ok(op);
                        }
                        self.entity_operand(f, scope, entity, span)?
                    }
                    _ => match self.resolve_entity(entity)? {
                        Resolved::Global {
                            storage,
                            ty,
                        } => Operand::Place {
                            ty,
                            addr: f.b.storage_addr(storage),
                        },
                        _ => {
                            return err(
                                Code::StructInvalidUsingTarget,
                                span,
                                "invalid using target",
                            );
                        }
                    },
                };
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
                    Code::CannotUseUsing,
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

    /// Resolve an inline default-assignment place, including promoted fields and fixed indexes.
    /// `cells[1].value = 17` writes only that subobject of the shared default image.
    pub(super) fn override_target(
        &mut self,
        root: TypeId,
        scope: ScopeId,
        expr: &ast::Expr,
    ) -> Result<(u64, TypeId)> {
        match &expr.kind {
            E::Ident(name) => self.override_member(root, *name, expr.span),
            E::Member(base, name) => {
                let (offset, ty) = self.override_target(root, scope, base)?;
                let (inner, ty) = self.override_member(ty, name.name, name.span)?;
                Ok((offset + inner, ty))
            }
            E::Index(base, index) => {
                let (offset, ty) = self.override_target(root, scope, base)?;
                let crate::types::TypeKind::Array {
                    elem,
                    kind: crate::types::ArrayKind::Fixed(count),
                } = *self.ty.types.kind(self.ty.types.repr(ty))
                else {
                    return err(
                        Code::AssignTargetInvalid,
                        base.span,
                        "a struct default can index only an inline fixed array",
                    );
                };
                let crate::sema::Value::Int(index_value) =
                    self.const_value_of_type(scope, index, TypeId::S64)?
                else {
                    unreachable!("an s64 constant is an integer");
                };
                if index_value < 0 || index_value >= i128::from(count) {
                    return err(
                        Code::ArrayIndex,
                        index.span,
                        format!(
                            "struct default index {index_value} is out of bounds for {count} elements"
                        ),
                    );
                }
                let size = self.size_of(elem, expr.span)?;
                Ok((offset + index_value as u64 * size, elem))
            }
            _ => err(
                Code::AssignTargetInvalid,
                expr.span,
                "a struct default target must be an inline member or fixed-array element",
            ),
        }
    }

    fn override_member(&mut self, owner: TypeId, name: Sym, span: Span) -> Result<(u64, TypeId)> {
        let Some((path, ty, origin)) = self.find_member_in(owner, name, span, &mut Vec::new())?
        else {
            return err(
                Code::StructNotMember,
                span,
                format!("`{name}` is not a member of {}", self.ty.types.name(owner)),
            );
        };
        let mut offset = 0;
        for step in path {
            match step {
                super::model::PathStep::Offset(bytes) => offset += bytes,
                super::model::PathStep::Deref => {
                    return err(
                        Code::AssignTargetInvalid,
                        span,
                        "a struct default cannot write through a pointer",
                    );
                }
            }
        }
        if let Some((s, index)) = origin {
            let field = &self.ty.types.struct_info(s).fields[index];
            self.ide_note_named_field(span, name, field.span, ty);
        }
        Ok((offset, ty))
    }
}
