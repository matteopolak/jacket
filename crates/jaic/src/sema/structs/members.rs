//! Finding members by name: fields, constants and promoted members.
use super::model::{MemberResolution, PathStep, UsingFilters, using_candidates};
use crate::ir::Ty;
use crate::sema::lower::FnCtx;
use crate::sema::scope::EntityKind;
use crate::sema::{
    Compiler, EntityId, PreludeType, ProcTypeParams, Rc, Result, Span, Sym, TypeId, err, ir,
};
use crate::types::{ArrayKind, LayoutState, StructId, TypeKind};

impl Compiler {
    /// Built-in aggregate members of non-struct types.
    pub(in crate::sema) fn builtin_members(
        &mut self,
        ty: TypeId,
        span: Span,
    ) -> Result<Vec<(Sym, TypeId, u64)>> {
        let s = |n: &str| Sym::intern(n);
        Ok(match self.ty.types.kind(ty).clone() {
            TypeKind::String => vec![(s("count"), TypeId::S64, 0), (s("data"), TypeId::U8_PTR, 8)],
            TypeKind::Array {
                elem,
                kind: ArrayKind::View,
            } => {
                let p = self.ty.types.pointer(elem);
                vec![(s("count"), TypeId::S64, 0), (s("data"), p, 8)]
            }
            TypeKind::Array {
                elem,
                kind: ArrayKind::Resizable,
            } => {
                let p = self.ty.types.pointer(elem);
                let alloc = self.preload_type(PreludeType::Allocator, span)?;
                vec![
                    (s("count"), TypeId::S64, 0),
                    (s("data"), p, 8),
                    (s("allocated"), TypeId::S64, 16),
                    (s("allocator"), alloc, 24),
                ]
            }
            TypeKind::Any => {
                let ti = self.preload_type(PreludeType::TypeInfo, span)?;
                let p = self.ty.types.pointer(ti);
                vec![(s("type"), p, 0), (s("value_pointer"), TypeId::VOID_PTR, 8)]
            }
            _ => Vec::new(),
        })
    }

    /// Find `name` among the fields of `ty`, following `using` fields.
    pub fn find_member(
        &mut self,
        ty: TypeId,
        name: Sym,
        span: Span,
    ) -> Result<Option<(Vec<PathStep>, TypeId)>> {
        Ok(self
            .find_member_in(ty, name, span, &mut Vec::new())?
            .map(|(path, ty, _)| (path, ty)))
    }

    /// The header of the selected field, following the same filtered/using
    /// paths as actual member access. Equal offsets do not imply equal headers.
    pub fn callable_member_parameters(
        &mut self,
        ty: TypeId,
        name: Sym,
        span: Span,
    ) -> Result<Option<Rc<ProcTypeParams>>> {
        let ty = self.ty.types.pointee(ty).unwrap_or(ty);
        Ok(self
            .find_member_in(ty, name, span, &mut Vec::new())?
            .and_then(|(_, _, origin)| origin)
            .and_then(|origin| self.pr.callable_fields.get(&origin).cloned()))
    }

    /// The byte offset and type of field `name` of `ty`, when it sits inline (not behind a
    /// `using` pointer).
    pub fn member_offset(
        &mut self,
        ty: TypeId,
        name: &str,
        span: Span,
    ) -> Result<Option<(u64, TypeId)>> {
        let Some((path, fty)) = self.find_member(ty, Sym::intern(name), span)? else {
            return Ok(None);
        };
        let mut offset = 0;
        for step in &path {
            match step {
                PathStep::Offset(o) => offset += o,
                PathStep::Deref => return Ok(None),
            }
        }
        Ok(Some((offset, fty)))
    }

    /// Where `Source_Code_Location`'s path, line and column sit in it.
    pub fn location_layout(&mut self, span: Span) -> Result<crate::interp::LocationLayout> {
        let ty = self.preload_type(PreludeType::SourceCodeLocation, span)?;
        let mut at = |name: &str| -> Result<u64> {
            match self.member_offset(ty, name, span)? {
                Some((offset, _)) => Ok(offset),
                None => err(
                    span,
                    format!("Preload's `Source_Code_Location` has no `{name}`"),
                ),
            }
        };
        Ok(crate::interp::LocationLayout {
            path: at("fully_pathed_filename")?,
            line: at("line_number")?,
            column: at("character_number")?,
        })
    }

    /// `find_member`, skipping structs already searched: `using` pointers can form a cycle
    /// (`S :: struct { using next: *S; }`), which would otherwise recurse forever.
    pub(super) fn find_member_in(
        &mut self,
        ty: TypeId,
        name: Sym,
        span: Span,
        searched: &mut Vec<StructId>,
    ) -> Result<Option<MemberResolution>> {
        let ty = self.ty.types.repr_struct(ty);
        let Some(s) = self.ty.types.as_struct(ty) else {
            let members = self.builtin_members(ty, span)?;
            return Ok(members
                .into_iter()
                .find(|(n, _, _)| *n == name)
                .map(|(_, t, o)| (vec![PathStep::Offset(o)], t, None)));
        };
        if searched.contains(&s) {
            return Ok(None);
        }
        searched.push(s);
        self.layout_struct(s, span)?;
        // Without cloning the field list: this runs for every member access.
        let fields = &self.ty.types.struct_info(s).fields;
        if let Some((index, f)) = fields
            .iter()
            .enumerate()
            .find(|(_, f)| f.name == Some(name))
        {
            return Ok(Some((
                vec![PathStep::Offset(f.offset)],
                f.ty,
                Some((s, index)),
            )));
        }
        let no_filters = UsingFilters::default();
        let filters = self.ty.using_filters.get(&s).unwrap_or(&no_filters);
        let candidates = using_candidates(fields, filters, name);
        if let Some(found) = self.find_promoted_member(candidates, span, searched)? {
            return Ok(Some(found));
        }
        let aliases = self.ty.member_aliases.get(&s).cloned().unwrap_or_default();
        for alias in aliases {
            let Some(target) = alias.target_name(name) else {
                continue;
            };
            let mut full = Vec::new();
            let mut at = ty;
            let mut origin = None;
            for &step in alias.path.iter().chain([&target]) {
                // A pointer field on the path is followed.
                if let Some(p) = self.ty.types.pointee(at)
                    && self.ty.types.as_struct(p).is_some()
                {
                    full.push(PathStep::Deref);
                    at = p;
                }
                let Some((mut path, t, field)) =
                    self.find_member_in(at, step, span, &mut Vec::new())?
                else {
                    return err(
                        span,
                        format!("`{step}` is not a member of {}", self.ty.types.name(at)),
                    );
                };
                full.append(&mut path);
                at = t;
                origin = field;
            }
            return Ok(Some((full, at, origin)));
        }
        Ok(None)
    }

    /// Shared promoted-field resolution for completed structs and the laid-out
    /// prefix at a #place/#overlay directive. A dynamic pointer step is explicit.
    pub(super) fn find_promoted_member(
        &mut self,
        candidates: Vec<(TypeId, u64, Sym)>,
        span: Span,
        searched: &mut Vec<StructId>,
    ) -> Result<Option<MemberResolution>> {
        for (field_ty, offset, name) in candidates {
            let (inner, deref) = match self.ty.types.pointee(field_ty) {
                Some(p) if self.ty.types.as_struct(p).is_some() => (p, true),
                _ => (field_ty, false),
            };
            if let Some((mut path, ty, origin)) =
                self.find_member_in(inner, name, span, searched)?
            {
                let mut full = vec![PathStep::Offset(offset)];
                if deref {
                    full.push(PathStep::Deref);
                }
                full.append(&mut path);
                return Ok(Some((full, ty, origin)));
            }
        }
        Ok(None)
    }

    /// Constant members of a struct type (declared in its body), including through `using`.
    pub fn struct_constant(&mut self, ty: TypeId, name: Sym) -> Result<Option<Vec<EntityId>>> {
        self.struct_constant_in(ty, name, &mut Vec::new())
    }

    /// `struct_constant`, skipping structs already searched (cycles of `using` pointers).
    fn struct_constant_in(
        &mut self,
        ty: TypeId,
        name: Sym,
        searched: &mut Vec<StructId>,
    ) -> Result<Option<Vec<EntityId>>> {
        let ty = self.ty.types.repr_struct(ty);
        let Some(s) = self.ty.types.as_struct(ty) else {
            return Ok(None);
        };
        if searched.contains(&s) {
            return Ok(None);
        }
        searched.push(s);
        if let Some(src) = self.ty.struct_asts.get(&s) {
            let scope = src.scope;
            self.expand_pending(scope)?;
            if let Some(ids) = self.scope(scope).names.get(&name)
                && !ids.is_empty()
            {
                return Ok(Some(ids.clone()));
            }
        }
        // Constants of `using` fields are reachable too (`context.default_allocator`). A struct
        // being laid out has no fields yet: a name looked up from its own field types finds
        // only its declared constants.
        if self.ty.types.struct_info(s).layout == LayoutState::InProgress {
            return Ok(None);
        }
        self.layout_struct(s, Span::NONE)?;
        let usings: Vec<TypeId> = self
            .ty
            .types
            .struct_info(s)
            .fields
            .iter()
            .filter(|f| f.using)
            .map(|f| f.ty)
            .collect();
        for field_ty in usings {
            let inner = self.ty.types.pointee(field_ty).unwrap_or(field_ty);
            if let Some(ids) = self.struct_constant_in(inner, name, searched)? {
                return Ok(Some(ids));
            }
        }
        Ok(None)
    }

    pub fn type_has_member(&mut self, ty: TypeId, name: Sym) -> Result<bool> {
        let target = self.ty.types.pointee(ty).unwrap_or(ty);
        if let Some(&scope) = self.ty.enum_member_scopes.get(&target) {
            return Ok(self.scope(scope).names.get(&name).is_some_and(|ids| {
                ids.iter()
                    .any(|&id| matches!(self.entity(id).kind, EntityKind::EnumMember { .. }))
            }));
        }
        if let TypeKind::Enum(e) = self.ty.types.kind(target) {
            return Ok(self
                .ty
                .types
                .enum_info(*e)
                .members
                .iter()
                .any(|(n, _)| *n == name)
                || self
                    .ty
                    .enum_borrowed
                    .get(&target)
                    .is_some_and(|names| names.iter().any(|(n, _)| *n == name)));
        }
        if self.struct_constant(target, name)?.is_some() {
            return Ok(true);
        }
        // A name looked up while this struct is being laid out, through a `using g;` of a
        // global of its type (no_api's `using gpu_context;`), from its own body. Layout goes
        // through declarations independently of offsets; asking only whether a declared
        // field exists must not request storage layout recursively.
        if let Some(s) = self.ty.types.as_struct(self.ty.types.repr_struct(target))
            && self.ty.types.struct_info(s).layout == LayoutState::InProgress
            && let Some(src) = self.ty.struct_asts.get(&s)
        {
            let tag = src.lit.tag.as_ref().map(|t| t.name.name);
            let declared = self
                .ty
                .field_declarations
                .get(&src.scope)
                .is_some_and(|fields| fields.contains_key(&name));
            return Ok(tag == Some(name) || declared);
        }
        Ok(self.find_member(target, name, Span::NONE)?.is_some())
    }

    pub(in crate::sema) fn apply_path(
        &mut self,
        f: &mut FnCtx,
        mut addr: ir::Val,
        path: &[PathStep],
    ) -> ir::Val {
        for step in path {
            addr = match step {
                PathStep::Offset(o) => f.b.ptr_offset(addr, *o),
                PathStep::Deref => f.b.load(Ty::Ptr, addr),
            };
        }
        addr
    }
}
