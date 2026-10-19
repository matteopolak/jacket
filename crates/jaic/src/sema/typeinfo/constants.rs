//! Struct constants recorded in the type information.
use super::StructConstant;
use crate::sema::scope::{EntityKind, Resolved};
use crate::sema::value::Aggregate;
use crate::sema::{Compiler, EntityId, Rc, Result, Span, Sym, TypeId, Value, ast, ir, structs};

impl Compiler {
    /// The constants a struct body declares (`Entry :: struct {..}`, `K :: 3;`, procedures),
    /// in declaration order, with their types and values. Polymorphic procedures are listed
    /// without a value, overload set aliases not at all. All appear in
    /// `Type_Info_Struct.members` with the `CONSTANT` flag, used or not. A constant that fails to resolve
    /// is left out: type info never reports errors for code nothing uses.
    pub(super) fn struct_constants(&mut self, s: crate::types::StructId) -> Vec<StructConstant> {
        let Some(scope) = self.struct_asts.get(&s).map(|src| src.scope) else {
            return Vec::new();
        };
        let mut ids: Vec<EntityId> = self
            .scope(scope)
            .names
            .values()
            .flatten()
            .copied()
            .filter(|&id| {
                matches!(
                    &self.entity(id).kind,
                    EntityKind::Decl { decl, .. } if decl.kind == ast::DeclKind::Const
                )
            })
            .collect();
        ids.sort_by_key(|&id| {
            let span = self.entity(id).span;
            (span.file, span.start)
        });
        let mut out = Vec::new();
        for id in ids {
            let (name, span) = (self.entity(id).name, self.entity(id).span);
            let notes: Vec<Rc<[u8]>> = match &self.entity(id).kind {
                EntityKind::Decl {
                    decl, ..
                } => decl
                    .notes
                    .iter()
                    .map(|n| Rc::from(n.text.as_bytes()))
                    .collect(),
                _ => Vec::new(),
            };
            let resolved = match self.resolve_entity(id) {
                Ok(Resolved::Const {
                    value,
                    ty,
                }) => Ok((ty, Some(value))),
                // A procedure's value is its address (Objective_C reads methods from there).
                // Only once its signature is known: resolving it here can lay out the types it
                // names early (or run a metaprogram's `#insert` before it is ready). A procedure
                // nothing has used yet is listed with the shape of its header instead, so code
                // that looks a hook up by name still finds it.
                Ok(Resolved::Proc(p)) if self.proc(p).is_poly => {
                    let ty = if self.proc(p).sig.is_none() {
                        self.header_shape_type(p)
                    } else {
                        self.poly_proc_type(p)
                    };
                    Ok((ty, None))
                }
                Ok(Resolved::Proc(p)) if self.proc(p).sig.is_none() => {
                    Ok((self.header_shape_type(p), Some(Value::Proc(p))))
                }
                Ok(Resolved::Proc(p)) => {
                    self.proc_type(p, span).map(|ty| (ty, Some(Value::Proc(p))))
                }
                // A module alias (`M :: #import "Math";`) is a `Type`-typed constant.
                Ok(Resolved::Module(_)) => Ok((TypeId::TYPE, None)),
                Ok(_) => continue,
                Err(e) => Err(e),
            };
            if let Ok((ty, value)) = resolved {
                out.push((span, name, ty, value, notes));
            }
        }
        out
    }

    /// One `CONSTANT` entry of `Type_Info_Struct.members`; its value, if it has one, is
    /// appended to `storage` and `offset_into_constant_storage` points at it.
    pub(super) fn constant_member(
        &mut self,
        (member_ty, msize): (TypeId, u64),
        (name, mut ty): (Sym, TypeId),
        value: Option<Value>,
        notes: &[Rc<[u8]>],
        storage: &mut Aggregate,
        span: Span,
    ) -> Result<Aggregate> {
        let mut m = Aggregate {
            bytes: vec![0; msize as usize],
            relocs: Vec::new(),
        };
        if let Some(img) = self.default_initializer(member_ty, span)? {
            structs::write_agg(&mut m, 0, &img);
        }
        if let Some(v) = &value
            && self.size_of(ty, span).is_err()
        {
            ty = self.type_of_value(v); // Untyped literals.
        }
        let mut offset: i128 = -1;
        if let Some(value) = value
            && let (Ok(size), Ok(align)) = (self.size_of(ty, span), self.align_of(ty, span))
        {
            let at = storage.bytes.len().next_multiple_of(align.max(1) as usize);
            storage.bytes.resize(at + size as usize, 0);
            if let Value::Type(t) = value {
                let g = self.type_info_global(t, span)?;
                storage.relocs.push(ir::Reloc {
                    offset: at as u64,
                    target: ir::RelocTarget::Global(g),
                    addend: 0,
                });
                offset = at as i128;
            } else if self
                .write_value(storage, at as u64, &value, ty, span)
                .is_ok()
            {
                offset = at as i128;
            } else {
                storage.bytes.truncate(at);
            }
        }
        self.set_field(
            &mut m,
            member_ty,
            "name",
            Value::String(name.as_str().as_bytes().into()),
            span,
        )?;
        self.set_info_ptr(&mut m, member_ty, "type", ty, span)?;
        self.set_field(&mut m, member_ty, "flags", Value::Int(0x1), span)?;
        // A constant has no storage in the struct's instances.
        self.set_field(&mut m, member_ty, "offset_in_bytes", Value::Int(-1), span)?;
        self.set_notes_view(&mut m, member_ty, notes, span)?;
        self.set_field(
            &mut m,
            member_ty,
            "offset_into_constant_storage",
            Value::Int(offset),
            span,
        )?;
        Ok(m)
    }
}
