//! Invocation roles of the selected canonical allocator callback ABI.
use crate::fxhash::HashSet;
use crate::ide::lifetimes::LifetimeAllocatorCall;
use crate::intern::Sym;
use crate::sema::{
    Compiler, PreludeType, Value,
    scope::{EntityState, Resolved},
};
use crate::source::Span;
use crate::types::{TypeId, TypeKind};

impl Compiler {
    /// Certify protocol roles, not callback behavior. A custom prelude may choose
    /// different enum tags; `ALLOCATE :: 101` must retain 101 rather than zero.
    pub(super) fn lifetime_allocator_protocol(
        &mut self,
        ty: TypeId,
        actual: Option<&crate::ast::ProcHeader>,
        span: Span,
    ) -> Option<LifetimeAllocatorCall> {
        if !header_roles(actual?) {
            return None;
        }
        let allocator = self.preload_type(PreludeType::Allocator, span).ok()?;
        let st = self.ty.types.as_struct(allocator)?;
        let fields = &self.ty.types.struct_info(st).fields;
        let mut selected = fields
            .iter()
            .enumerate()
            .filter(|(_, field)| field.name == Some(Sym::intern("proc")));
        let (index, field) = selected.next()?;
        if selected.next().is_some() || field.ty != ty {
            return None;
        }
        let lexical = self.pr.callable_fields.get(&(st, index))?;
        let header = lexical.declaration.as_ref()?;
        let roles = ["mode", "size", "old_size", "old_memory", "allocator_data"];
        if lexical.names.len() != roles.len()
            || lexical
                .names
                .iter()
                .zip(roles)
                .any(|(name, role)| *name != Some(Sym::intern(role)))
            || !header_roles(header)
        {
            return None;
        }
        let TypeKind::Proc(signature) = self.ty.types.kind(ty) else {
            return None;
        };
        if signature.c_call
            || signature.no_context
            || signature.variadic
            || signature.c_varargs
            || signature.non_pod_return
            || signature.params.len() != roles.len()
            || signature.returns.len() != 1
        {
            return None;
        }
        let mode = self.canonical_allocator_mode()?;
        if signature.params[0] != mode
            || signature.params[1..3] != [TypeId::S64, TypeId::S64]
            || signature.params[3..]
                .iter()
                .chain(&signature.returns)
                .any(|ty| self.ty.types.pointee(*ty) != Some(TypeId::VOID))
        {
            return None;
        }
        let TypeKind::Enum(id) = self.ty.types.kind(mode) else {
            return None;
        };
        let info = self.ty.types.enum_info(*id);
        if info.is_flags || info.loose_of.is_some() {
            return None;
        }
        let (width, signed) = self.ty.types.int_info(info.base)?;
        if !matches!(width, 8 | 16 | 32 | 64) {
            return None;
        }
        let mut tags = Vec::new();
        let mut distinct = HashSet::default();
        for (name, value) in &info.members {
            let bits = tag_bits(*value, width, signed)?;
            if !distinct.insert(bits) {
                return None;
            }
            tags.push((*name, bits));
        }
        let value = |name| {
            let mut matching = tags.iter().filter(|(n, _)| *n == Sym::intern(name));
            let (_, value) = matching.next()?;
            matching.next().is_none().then_some(*value)
        };
        let role = |name| roles.iter().position(|selected| *selected == name);
        Some(LifetimeAllocatorCall {
            operation_formal: role("mode")?,
            size_formal: role("size")?,
            old_size_formal: role("old_size")?,
            old_memory_formal: role("old_memory")?,
            data_formal: role("allocator_data")?,
            operation_ty: mode,
            allocate_value: value("ALLOCATE")?,
            resize_value: value("RESIZE")?,
            free_value: value("FREE")?,
            result_position: 0,
        })
    }

    /// Require the nominal mode exported by this compiler's actual prelude.
    fn canonical_allocator_mode(&self) -> Option<TypeId> {
        let module = self.ns.preload?;
        let scope = self.ns.modules[module.0 as usize].scope;
        let ids = self
            .scope(scope)
            .names
            .get(&Sym::intern("Allocator_Mode"))?;
        let [id] = ids.as_slice() else {
            return None;
        };
        let EntityState::Done(Resolved::Const {
            value: Value::Type(ty),
            ..
        }) = &self.entity(*id).state
        else {
            return None;
        };
        Some(*ty)
    }
}

/// The selected integer width controls tag bits; negative signed tags preserve
/// two's-complement interpretation, e.g. -1 in s8 is 255.
fn tag_bits(value: i128, width: u8, signed: bool) -> Option<u64> {
    let limit = 1i128.checked_shl(u32::from(width))?;
    let (minimum, maximum) = if signed {
        (-(limit / 2), limit / 2 - 1)
    } else {
        (0, limit - 1)
    };
    if value < minimum || value > maximum {
        return None;
    }
    u64::try_from((value as u128) & ((limit as u128) - 1)).ok()
}

/// Both the canonical ABI and the actual selected callable declaration retain
/// their roles. Equal structural pointer types cannot identify swapped data.
fn header_roles(header: &crate::ast::ProcHeader) -> bool {
    let roles = ["mode", "size", "old_size", "old_memory", "allocator_data"];
    header.params.len() == roles.len()
        && header.returns.len() == 1
        && !header.uses_c_abi()
        && !header.flags.no_context()
        && !header.baked
        && header.params.iter().zip(roles).all(|(parameter, role)| {
            parameter
                .name
                .as_ref()
                .is_some_and(|name| name.name == Sym::intern(role))
                && !parameter.baked
                && !parameter.auto_bake
                && !parameter.discard
                && !parameter.variadic
                && parameter.default.is_none()
        })
}
