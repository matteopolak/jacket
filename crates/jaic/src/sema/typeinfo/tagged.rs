//! Tagged union members and polymorphic struct bindings.
use super::views::View;
use crate::sema::scope::EntityKind;
use crate::sema::value::Aggregate;
use crate::sema::{
    Compiler, EntityId, PreludeType, Result, Span, Sym, TypeId, Value, ast, structs,
};
use crate::types::TypeKind;

impl Compiler {
    /// The members a struct's type info lists. A tagged union lists its tag, then each
    /// variant member directly (not the anonymous union holding them), with the
    /// `(tag value, member index)` pairs selecting them.
    #[allow(clippy::type_complexity)]
    pub(super) fn tagged_union_members(
        &mut self,
        s: crate::types::StructId,
        fields: &[crate::types::Field],
        span: Span,
    ) -> Result<(Vec<crate::types::Field>, Vec<(Value, usize)>)> {
        let Some(src) = self.struct_asts.get(&s).cloned() else {
            return Ok((fields.to_vec(), Vec::new()));
        };
        let (Some(_), [tag, variants]) = (&src.lit.tag, fields) else {
            return Ok((fields.to_vec(), Vec::new()));
        };
        let Some(vs) = self.types.as_struct(variants.ty) else {
            return Ok((fields.to_vec(), Vec::new()));
        };
        self.layout_struct(vs, span)?;
        let mut flat = vec![tag.clone()];
        for f in self.types.struct_info(vs).fields.iter() {
            let mut f = f.clone();
            f.offset += variants.offset;
            flat.push(f);
        }
        let tag_members = match self.types.kind(tag.ty) {
            TypeKind::Enum(e) => self.types.enum_info(*e).members.clone(),
            _ => Vec::new(),
        };
        let mut bindings = Vec::new();
        for stmt in &src.lit.body {
            let ast::StmtKind::Decl(d) = &stmt.kind else {
                continue;
            };
            let Some(t) = &d.union_tag() else {
                continue;
            };
            // `.A` names a member of the tag's enum; `4`, `-12` or `u16` is a constant of
            // the tag's type.
            let value = match &t.kind {
                ast::ExprKind::InferredMember(m) => {
                    match tag_members.iter().find(|(n, _)| *n == m.name) {
                        Some(&(_, v)) => Value::Int(v),
                        None => continue,
                    }
                }
                _ => match self.eval_const(src.scope, t, Some(tag.ty))? {
                    crate::sema::lower::Operand::Type(ty) => Value::Type(ty),
                    crate::sema::lower::Operand::Const {
                        value: v @ (Value::Int(_) | Value::Type(_)),
                        ..
                    } => v,
                    _ => continue,
                },
            };
            for n in &d.names {
                if let Some(index) = flat.iter().position(|f| f.name == Some(n.name)) {
                    bindings.push((value.clone(), index));
                }
            }
        }
        Ok((flat, bindings))
    }

    /// `tagged_union_bindings` of a tagged union's descriptor.
    pub(super) fn set_tagged_union_bindings(
        &mut self,
        agg: &mut Aggregate,
        desc: TypeId,
        bindings: &[(Value, usize)],
        span: Span,
    ) -> Result<()> {
        let binding_ty = self.preload_type(PreludeType::TypeInfoTaggedUnionBinding, span)?;
        let size = self.size_of(binding_ty, span)?;
        let align = self.align_of(binding_ty, span)?;
        let mut items = Aggregate {
            bytes: vec![0; size as usize * bindings.len()],
            relocs: Vec::new(),
        };
        for (i, (value, index)) in bindings.iter().enumerate() {
            let index = *index;
            let mut b = Aggregate {
                bytes: vec![0; size as usize],
                relocs: Vec::new(),
            };
            match value {
                // A `Type` tag's constant is the type's descriptor address.
                Value::Type(t) => {
                    self.set_info_ptr(&mut b, binding_ty, "constant_value", *t, span)?;
                }
                v => self.set_field(&mut b, binding_ty, "constant_value", v.clone(), span)?,
            }
            self.set_field(
                &mut b,
                binding_ty,
                "member_index",
                Value::Int(index as i128),
                span,
            )?;
            structs::write_agg(&mut items, i as u64 * size, &b);
        }
        self.set_view(
            agg,
            desc,
            "tagged_union_bindings",
            View::new(bindings.len(), items, align),
            span,
        )
    }

    /// The parameter bindings of a polymorphic struct instance, in declaration
    /// order (baked ones from `#bake_arguments` first).
    pub(in crate::sema) fn poly_struct_bindings(
        &self,
        s: crate::types::StructId,
    ) -> Vec<(Sym, Value, TypeId)> {
        let Some(scope) = self.struct_asts.get(&s).map(|src| src.scope) else {
            return Vec::new();
        };
        let mut bindings: Vec<(EntityId, Sym, Value, TypeId)> = self
            .scope(scope)
            .names
            .iter()
            .flat_map(|(&name, ids)| ids.iter().map(move |&e| (name, e)))
            .filter_map(|(name, e)| match &self.entity(e).kind {
                EntityKind::Const {
                    value,
                    ty,
                } => Some((e, name, value.clone(), *ty)),
                _ => None,
            })
            .collect();
        bindings.sort_by_key(|b| b.0);
        bindings.into_iter().map(|(_, n, v, t)| (n, v, t)).collect()
    }
}
