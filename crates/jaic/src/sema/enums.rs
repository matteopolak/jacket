//! Enum declarations retain source order while values resolve through the ordinary entity graph.
use super::lower::Operand;
use super::scope::{EntityKind, Resolved};
use super::{Compiler, EntityId, Rc, Result, ScopeId, Span, Sym, TypeId, Value, ast, err};
use crate::ast::ExprKind as E;
use crate::types::{EnumInfo, TypeKind};

#[derive(Clone, Debug)]
pub enum Initializer {
    Explicit(ast::Expr),
    Implicit {
        previous: Option<EntityId>,
        flags: bool,
    },
    /// A member copied by `using` from another enum.
    Constant(i128),
    /// Selection/insertion must establish the preceding declaration before this value exists.
    PendingOrder,
}

fn may_declare_members(items: &[ast::EnumItem]) -> bool {
    items.iter().any(|item| match item {
        ast::EnumItem::Member(_) | ast::EnumItem::Insert(_) | ast::EnumItem::Using(..) => true,
        ast::EnumItem::If {
            then_items,
            else_items,
            ..
        } => may_declare_members(then_items) || may_declare_members(else_items),
    })
}

impl Compiler {
    pub fn new_enum_type(
        &mut self,
        name: Sym,
        lit: &Rc<ast::EnumLit>,
        scope: ScopeId,
    ) -> Result<TypeId> {
        let base = match &lit.base {
            Some(base) => self.eval_type(scope, base)?,
            None if lit.flags_enum => TypeId::U32,
            None => TypeId::S64,
        };
        let ty = self.ty.types.new_enum(EnumInfo {
            name,
            base,
            members: Vec::new(),
            is_flags: lit.flags_enum,
            specified: lit.specified,
            complete: lit.complete,
            loose_of: None,
            span: lit.span,
        });
        self.ty.nominal_origins.insert(ty, (lit.span, scope));
        if let Some(base) = &lit.base {
            self.ty.enum_base_spans.insert(ty, base.span);
        }
        let TypeKind::Enum(enum_id) = *self.ty.types.kind(ty) else {
            unreachable!()
        };
        let members_scope = self.new_hidden_scope(scope);
        if name.as_str() != "enum" {
            self.add_const(members_scope, name, lit.span, Value::Type(ty), TypeId::TYPE);
        }
        self.ty.enum_member_scopes.insert(ty, members_scope);
        let result = (|| {
            let mut declarations = Vec::new();
            self.collect_enum_items(
                &lit.items,
                members_scope,
                ty,
                lit.flags_enum,
                &mut None,
                &mut declarations,
            )?;
            let mut members = Vec::with_capacity(declarations.len());
            for id in declarations {
                let Resolved::Const {
                    value: Value::Int(value),
                    ..
                } = self.resolve_entity(id)?
                else {
                    unreachable!()
                };
                members.push((self.entity(id).name, value));
            }
            self.ty.types.enums[enum_id.0 as usize].members = members;
            Ok(ty)
        })();
        self.ty.enum_member_scopes.remove(&ty);
        result
    }

    fn collect_enum_items(
        &mut self,
        items: &[ast::EnumItem],
        scope: ScopeId,
        ty: TypeId,
        flags: bool,
        previous: &mut Option<EntityId>,
        out: &mut Vec<EntityId>,
    ) -> Result<()> {
        // Declare a lexical list before evaluating values or its structural directives.
        // Only selected branches/actual inserted lists call this method, so inactive names
        // neither collide nor shadow outer declarations.
        let mut preceding = *previous;
        let mut known_order = true;
        let mut direct = Vec::new();
        for item in items {
            let ast::EnumItem::Member(member) = item else {
                if may_declare_members(std::slice::from_ref(item)) {
                    known_order = false;
                }
                continue;
            };
            if self
                .scope(scope)
                .names
                .get(&member.name.name)
                .is_some_and(|ids| {
                    ids.iter()
                        .any(|&id| matches!(self.entity(id).kind, EntityKind::EnumMember { .. }))
                })
            {
                return Err(self.duplicate_enum_member(scope, member.name.name, member.name.span));
            }
            let initializer = match &member.value {
                Some(value) => Initializer::Explicit(value.clone()),
                None if known_order => Initializer::Implicit {
                    previous: preceding,
                    flags,
                },
                None => Initializer::PendingOrder,
            };
            let id = self.add_entity(
                scope,
                member.name.name,
                member.name.span,
                EntityKind::EnumMember {
                    ty,
                    initializer,
                },
                true,
            );
            direct.push(id);
            preceding = Some(id);
            known_order = true;
            let TypeKind::Enum(enum_id) = *self.ty.types.kind(ty) else {
                unreachable!()
            };
            self.ide_note_enum_member(enum_id, member.name.name, member.name.span);
        }
        let mut direct = direct.into_iter();
        for item in items {
            match item {
                ast::EnumItem::Member(member) => {
                    let id = direct.next().unwrap();
                    if member.value.is_none() {
                        self.entity_mut(id).kind = EntityKind::EnumMember {
                            ty,
                            initializer: Initializer::Implicit {
                                previous: *previous,
                                flags,
                            },
                        };
                    }
                    out.push(id);
                    *previous = Some(id);
                }
                ast::EnumItem::If {
                    cond,
                    then_items,
                    else_items,
                } => {
                    let selected = self.eval_static_condition(scope, cond)?;
                    self.collect_enum_items(
                        if selected {
                            then_items
                        } else {
                            else_items
                        },
                        scope,
                        ty,
                        flags,
                        previous,
                        out,
                    )?;
                }
                ast::EnumItem::Using(using) => {
                    self.collect_enum_using(&using.value, &using.filter, scope, ty)?;
                }
                ast::EnumItem::Insert(value) => {
                    let inserted = self.eval_insert_enum_items(scope, value)?;
                    self.collect_enum_items(&inserted, scope, ty, flags, previous, out)?;
                }
            }
        }
        Ok(())
    }

    /// `using Other;` in an enum body makes `Other`'s names reachable through this enum, with
    /// their values. They are not members of this enum, and do not shift its numbering.
    fn collect_enum_using(
        &mut self,
        value: &ast::Expr,
        filter: &ast::UsingFilter,
        scope: ScopeId,
        ty: TypeId,
    ) -> Result<()> {
        if matches!(filter, ast::UsingFilter::Map(_)) {
            return err(value.span, "`using,map` is not supported in an enum body");
        }
        let filter = self.resolve_name_filter(scope, filter)?;
        let source = self.eval_type(scope, value)?;
        let TypeKind::Enum(source_id) = *self.ty.types.kind(source) else {
            return err(value.span, "`using` in an enum body needs an enum type");
        };
        let members = self.ty.types.enums[source_id.0 as usize].members.clone();
        let TypeKind::Enum(enum_id) = *self.ty.types.kind(ty) else {
            unreachable!()
        };
        for (source_name, member_value) in members {
            let Some(name) = super::using::filter_target(&filter, source_name) else {
                continue;
            };
            if self.scope(scope).names.get(&name).is_some_and(|ids| {
                ids.iter()
                    .any(|&id| matches!(self.entity(id).kind, EntityKind::EnumMember { .. }))
            }) {
                return Err(self.duplicate_enum_member(scope, name, value.span));
            }
            self.add_entity(
                scope,
                name,
                value.span,
                EntityKind::EnumMember {
                    ty,
                    initializer: Initializer::Constant(member_value),
                },
                true,
            );
            // `E.name` names `Other`'s member: go to definition and rename reach it.
            let declared = self
                .ed
                .facts()
                .and_then(|ide| ide.enum_decls.get(&(source_id, source_name)).copied())
                .unwrap_or(value.span);
            self.ide_note_enum_member(enum_id, name, declared);
            // Reachable as `E.name`, but not a member of `E`: it is not listed or counted.
            self.ty
                .enum_borrowed
                .entry(ty)
                .or_default()
                .push((name, member_value));
        }
        Ok(())
    }

    pub(super) fn resolve_enum_initializer(
        &mut self,
        scope: ScopeId,
        ty: TypeId,
        initializer: &Initializer,
        span: Span,
    ) -> Result<Resolved> {
        let (value, at) = match initializer {
            Initializer::Explicit(expr) => {
                let value = match self.eval_const(scope, expr, Some(ty))? {
                    Operand::Const {
                        value: Value::Int(value),
                        ..
                    } => value,
                    other => {
                        return err(
                            expr.span,
                            format!(
                                "enum value must be an integer constant, found {}",
                                self.describe(&other)
                            ),
                        );
                    }
                };
                (value, expr.span)
            }
            Initializer::Implicit {
                previous,
                flags,
            } => {
                let value = if let Some(id) = previous {
                    let Resolved::Const {
                        value: Value::Int(value),
                        ..
                    } = self.resolve_entity(*id)?
                    else {
                        unreachable!()
                    };
                    if *flags {
                        if value == 0 {
                            1
                        } else {
                            value << 1
                        }
                    } else {
                        value + 1
                    }
                } else {
                    i128::from(*flags)
                };
                (value, span)
            }
            Initializer::Constant(value) => (*value, span),
            Initializer::PendingOrder => {
                return err(
                    span,
                    "enum declaration-order dependency: conditional/injected declarations must establish this implicit member's predecessor",
                );
            }
        };
        if let Some((bits, signed)) = self.ty.types.int_info(ty)
            && !super::convert::int_constant_fits(value, bits, signed, true)
        {
            return Err(self.enum_value_out_of_range(ty, value, (u32::from(bits), signed), at));
        }
        Ok(Resolved::Const {
            value: Value::Int(value),
            ty,
        })
    }

    fn eval_insert_enum_items(
        &mut self,
        scope: ScopeId,
        value: &ast::Expr,
    ) -> Result<Vec<ast::EnumItem>> {
        let Operand::Const {
            value: Value::String(text),
            ..
        } = self.eval_insert_operand(scope, value)?
        else {
            return err(value.span, "#insert in an enum body needs a string");
        };
        let text = format!(
            "__jaic_enum :: enum {{\n{}\n}};",
            String::from_utf8_lossy(&text)
        );
        let file = self.src.insert_source(value.span, text.clone().into())?;
        let parsed = crate::parser::parse_file(file, &text).map_err(Box::new)?;
        match parsed.stmts.first().map(|stmt| &stmt.kind) {
            Some(ast::StmtKind::Decl(decl)) => match decl.value.as_ref().map(|value| &value.kind) {
                Some(E::Enum(lit)) => Ok(lit.items.clone()),
                _ => err(value.span, "could not parse inserted enum members"),
            },
            _ => err(value.span, "could not parse inserted enum members"),
        }
    }
}
