//! Certified inline byte slots rooted at immutable incoming pointer values.
//!
//! Symbolic field spellings cannot identify bytes after pointer reinterpretation.
//! These certificates use the checked projection's own layout and stop at every
//! pointer/descriptor hop. Leaving the formal pointee extent yields unknown, not
//! proof that the address is unreachable or disjoint from a larger allocation.
use super::{Def, HashSet, LifetimeArgument, Projection, Resolver, Val, value};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(in crate::rules) struct Slot {
    pub formal: usize,
    pub offset: u64,
    pub width: u64,
}

/// The borrow consumer may use a builtin descriptor's data role. Exact storage
/// sources continue to require an actual selected pointer-field projection.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Leaf {
    Address,
    Pointer,
    Borrow,
}

impl Slot {
    pub(super) fn disjoint(self, other: Self) -> bool {
        use crate::rules::lifetime_facts::ByteRange;
        self.formal == other.formal
            && ByteRange {
                offset: self.offset,
                width: self.width,
            }
            .separated(ByteRange {
                offset: other.offset,
                width: other.width,
            })
    }
}

impl Resolver<'_> {
    pub(super) fn physical_slot(&self, place: usize, width: u64) -> Option<Slot> {
        if !self.places.unresolved(place) {
            return None;
        }
        self.physical_place(place, width, &mut HashSet::default(), 0)
    }

    fn physical_place(
        &self,
        place: usize,
        width: u64,
        seen: &mut HashSet<Val>,
        depth: usize,
    ) -> Option<Slot> {
        let ((root, root_ty), steps) = &self.places.paths[place];
        let mut ty = *root_ty;
        let mut slot =
            if let Some(address) = self.physical_address(*root, &mut seen.clone(), depth + 1) {
                address
            } else if steps.first() == Some(&Projection::Deref) {
                // This root is the address of a private pointer variable. Read its
                // unique immutable SSA initialization before entering pointed storage.
                let root_place = self.places.root_of(place);
                if !self.immutable_private_slot(root_place) {
                    return None;
                }
                let [source] = self.writes.get(&root_place)?.as_slice() else {
                    return None;
                };
                self.physical_address(*source, seen, depth + 1)?
            } else {
                return None;
            };
        let mut at_root = true;
        for step in steps {
            match step {
                Projection::Deref if at_root => {
                    ty = self.compiler.ty.types.pointee(ty)?;
                }
                Projection::Field(name) => {
                    let (offset, field_ty) =
                        field_projection(self.compiler, self.body, &self.places, ty, *name)?;
                    slot.offset = slot.offset.checked_add(offset)?;
                    ty = field_ty;
                }
                Projection::Element(index) => {
                    let jaic::types::TypeKind::Array {
                        kind: jaic::types::ArrayKind::Fixed(count),
                        elem,
                    } = self.compiler.ty.types.kind(ty)
                    else {
                        return None;
                    };
                    let index = u64::try_from(*index).ok()?;
                    if index >= *count {
                        return None;
                    }
                    slot.offset = slot
                        .offset
                        .checked_add(index.checked_mul(self.compiler.ty.types.size_of(*elem))?)?;
                    ty = *elem;
                }
                Projection::Deref
                | Projection::DynamicElement(_)
                | Projection::Inline {
                    ..
                } => return None,
            }
            at_root = false;
        }
        slot.width = width;
        self.valid_slot(slot)
    }

    fn physical_address(&self, value: Val, seen: &mut HashSet<Val>, depth: usize) -> Option<Slot> {
        if depth > 24 || !seen.insert(value) || self.address_casts.contains_key(&value)
            || self.body.blocks.iter().flat_map(|b| &b.events).any(|event|
                matches!(event, super::Event::Offset {value: result, offset, ..} if *result == value && *offset != Some(0)))
        {
            return None;
        }
        let mut result = None;
        for definition in self.values.get(&value)? {
            // Aggregate loads can attach a typed address view to the very same
            // SSA pointer. It adds no origin and must not turn a proven incoming
            // pointer into a recursive self-definition.
            if let Def::Address(place) = definition
                && self.places.paths[*place].0.0 == value
                && self.places.paths[*place].1.is_empty()
            {
                continue;
            }
            let current = match definition {
                Def::Parameter(formal, ..) => Slot {
                    formal: *formal,
                    offset: 0,
                    width: 0,
                },
                Def::Copy(source) => {
                    self.physical_address(*source, &mut seen.clone(), depth + 1)?
                }
                Def::Address(place) => {
                    self.physical_place(*place, 0, &mut seen.clone(), depth + 1)?
                }
                Def::Load(place, _) if !self.places.unresolved(*place) => {
                    let source = if let Some(sources) = self.storage_versions.get(&value) {
                        let mut sources = sources.iter();
                        let super::storage_versions::Source::Value(source) = sources.next()? else {
                            return None;
                        };
                        if sources.next().is_some() {
                            return None;
                        }
                        *source
                    } else {
                        if !self.immutable_private_slot(*place) {
                            return None;
                        }
                        let [source] = self.writes.get(place)?.as_slice() else {
                            return None;
                        };
                        *source
                    };
                    self.physical_address(source, &mut seen.clone(), depth + 1)?
                }
                _ => return None,
            };
            if result.is_some_and(|prior| prior != current) {
                return None;
            }
            result = Some(current);
        }
        self.valid_slot(result?)
    }

    /// Actual unexposed root storage, without walking a field/cast layout.
    pub(super) fn private_root(&self, place: usize, width: u64) -> bool {
        self.places.paths[place].1.is_empty()
            && self.immutable_private_slot(place)
            && super::storage_versions::private_range(
                self.compiler,
                self.body,
                &self.places,
                place,
                width,
                &self.private_bytes,
            )
    }

    pub(super) fn immutable_private_slot(&self, place: usize) -> bool {
        self.core()
            .private_slot(self.compiler, self.body, &self.places, place)
    }

    /// A by-value formal's private header is an input value snapshot, not
    /// incoming pointed storage. This does not certify a caller's writable slot.
    pub(super) fn input_value_field(&self, place: usize) -> bool {
        self.immutable_private_slot(place)
            && self
                .params
                .get(&self.places.root_of(place))
                .is_some_and(|(_, ty)| !self.compiler.ty.types.is_pointer(*ty))
    }

    pub(super) fn aggregate_place_slot(
        &self,
        place: usize,
        offset: u64,
        width: u64,
    ) -> Option<Slot> {
        let mut slot = self.physical_place(place, 0, &mut HashSet::default(), 0)?;
        slot.offset = slot.offset.checked_add(offset)?;
        slot.width = width;
        self.valid_slot(slot)
    }

    pub(super) fn aggregate_slot(&self, value: Val, offset: u64, width: u64) -> Option<Slot> {
        let mut slot = self.physical_address(value, &mut HashSet::default(), 0)?;
        slot.offset = slot.offset.checked_add(offset)?;
        slot.width = width;
        self.valid_slot(slot)
    }

    pub(super) fn substitute_slot(&self, slot: Slot, args: &[LifetimeArgument]) -> Option<Slot> {
        let argument = args
            .iter()
            .find(|arg| arg.formal == slot.formal && !arg.spread)?;
        // ABI-addressed by-value aggregates are snapshots, not writable caller slots.
        if !self.compiler.ty.types.is_pointer(argument.ty) {
            return None;
        }
        let mut actual = self.physical_address(value(argument)?, &mut HashSet::default(), 0)?;
        actual.offset = actual.offset.checked_add(slot.offset)?;
        actual.width = slot.width;
        self.valid_slot(actual)
    }

    pub(super) fn valid_slot(&self, slot: Slot) -> Option<Slot> {
        let (_, ty) = self
            .params
            .values()
            .find(|(formal, _)| *formal == slot.formal)?;
        let pointee = self.compiler.ty.types.pointee(*ty)?;
        let extent = self.compiler.ty.types.size_of(pointee);
        (extent != 0 && slot.offset.checked_add(slot.width)? <= extent).then_some(slot)
    }
}

impl Resolver<'_> {
    /// Canonical entry bytes use the incoming formal's layout, including descriptor
    /// fields selected by the compiler. Reinterpreted field spellings never decide
    /// which incoming pointer slot was read.
    pub(super) fn entry_path(&self, slot: Slot) -> Option<super::Path> {
        self.valid_slot(slot)?;
        let (_, ty) = self.params.values().find(|(n, _)| *n == slot.formal)?;
        let ty = self.compiler.ty.types.pointee(*ty)?;
        let mut paths = Vec::new();
        self.entry_fields(
            ty,
            (slot.offset, slot.width),
            Vec::new(),
            &mut paths,
            Leaf::Pointer,
            0,
        );
        let [steps] = paths.as_slice() else {
            return None;
        };
        super::Path {
            root: super::Root::Parameter(slot.formal),
            steps: steps.clone(),
        }
        .append(&[super::Step::Deref])
    }

    /// Address certificates identify inline bytes, without loading a pointer slot.
    pub(super) fn address_path(&self, slot: Slot) -> Option<super::Path> {
        self.valid_slot(slot)?;
        let (_, ty) = self.params.values().find(|(n, _)| *n == slot.formal)?;
        let ty = self.compiler.ty.types.pointee(*ty)?;
        let mut paths = Vec::new();
        self.entry_fields(
            ty,
            (slot.offset, slot.width),
            Vec::new(),
            &mut paths,
            Leaf::Address,
            0,
        );
        let [steps] = paths.as_slice() else {
            return None;
        };
        Some(super::Path {
            root: super::Root::Parameter(slot.formal),
            steps: steps.clone(),
        })
    }

    pub(super) fn entry_fields(
        &self,
        ty: jaic::types::TypeId,
        range: (u64, u64),
        prefix: Vec<super::Step>,
        out: &mut Vec<Vec<super::Step>>,
        leaf: Leaf,
        depth: usize,
    ) {
        let (offset, width) = range;
        if depth > super::MAX_PATH || out.len() > 1 {
            return;
        }
        if offset == 0
            && width == self.compiler.ty.types.size_of(ty)
            && (leaf == Leaf::Address || self.compiler.ty.types.is_pointer(ty))
        {
            if !out.contains(&prefix) {
                out.push(prefix);
            }
            return;
        }
        if self.compiler.ty.types.is_pointer(ty) {
            return;
        }
        if let Some(st) = self.compiler.ty.types.as_struct(ty) {
            for field in self.compiler.ty.types.struct_info(st).fields.iter() {
                let Some(name) = field.name else {
                    continue;
                };
                let Some(relative) = offset.checked_sub(field.offset) else {
                    continue;
                };
                if relative
                    .checked_add(width)
                    .is_none_or(|end| end > self.compiler.ty.types.size_of(field.ty))
                {
                    continue;
                }
                let mut path = prefix.clone();
                path.push(super::Step::Field(name));
                self.entry_fields(field.ty, (relative, width), path, out, leaf, depth + 1);
            }
            return;
        }
        match self.compiler.ty.types.kind(ty) {
            jaic::types::TypeKind::Array {
                kind: jaic::types::ArrayKind::Fixed(count),
                elem,
            } => {
                let size = self.compiler.ty.types.size_of(*elem);
                if size == 0 {
                    return;
                }
                let index = offset / size;
                if index >= *count {
                    return;
                }
                let mut path = prefix;
                path.push(super::Step::Element(i128::from(index)));
                self.entry_fields(*elem, (offset % size, width), path, out, leaf, depth + 1);
            }
            jaic::types::TypeKind::Array {
                ..
            }
            | jaic::types::TypeKind::String => {
                if leaf == Leaf::Borrow {
                    if offset == jaic::layout::view::DATA && width == jaic::layout::POINTER_SIZE {
                        let mut path = prefix;
                        path.push(super::Step::Storage);
                        if !out.contains(&path) {
                            out.push(path);
                        }
                    }
                    return;
                }
                // Descriptor roles come from selected compiler projections, not a
                // user field with a convenient spelling or an assumed byte offset.
                for event in self.body.blocks.iter().flat_map(|b| &b.events) {
                    let super::Event::FieldProjection {
                        base_ty,
                        result,
                        result_ty,
                        offset: field_offset,
                        ..
                    } = event
                    else {
                        continue;
                    };
                    if *base_ty != ty
                        || *field_offset != offset
                        || (leaf == Leaf::Pointer && !self.compiler.ty.types.is_pointer(*result_ty))
                        || self.compiler.ty.types.size_of(*result_ty) != width
                    {
                        continue;
                    }
                    let Some(place) = self.places.existing(*result, *result_ty) else {
                        continue;
                    };
                    let Some(Projection::Field(name)) = self.places.paths[place].1.last() else {
                        continue;
                    };
                    let mut path = prefix.clone();
                    path.push(super::Step::Field(*name));
                    if !out.contains(&path) {
                        out.push(path);
                    }
                }
            }
            _ => {}
        }
    }
}

/// The selected field's inline byte range, including compiler-defined descriptor
/// and promoted members. A spelling alone never supplies a descriptor role.
pub(super) fn field_projection(
    c: &jaic::sema::Compiler,
    body: &super::LifetimeBody,
    places: &super::Places,
    ty: jaic::types::TypeId,
    name: jaic::intern::Sym,
) -> Option<(u64, jaic::types::TypeId)> {
    if let Some(st) = c.ty.types.as_struct(ty)
        && let Some(field) =
            c.ty.types
                .struct_info(st)
                .fields
                .iter()
                .find(|field| field.name == Some(name))
    {
        return Some((field.offset, field.ty));
    }
    let mut selected = None;
    for event in body.blocks.iter().flat_map(|b| &b.events) {
        let super::Event::FieldProjection {
            base_ty,
            result,
            result_ty,
            offset,
            ..
        } = event
        else {
            continue;
        };
        if *base_ty != ty {
            continue;
        }
        let Some(place) = places.existing(*result, *result_ty) else {
            continue;
        };
        if places.paths[place].1.last() != Some(&Projection::Field(name)) {
            continue;
        }
        let field = (*offset, *result_ty);
        if selected.is_some_and(|prior| prior != field) {
            return None;
        }
        selected = Some(field);
    }
    selected
}

impl super::prepared::Core<'_> {
    /// One private-storage policy for actual queries and immutable read inventories.
    pub(super) fn private_slot(
        &self,
        c: &super::Compiler,
        body: &super::LifetimeBody,
        places: &super::Places,
        place: usize,
    ) -> bool {
        let root = places.root_of(place);
        self.owned.contains(&root)
            && places.selected_type(place).is_some_and(|ty| {
                super::storage_versions::private_range(
                    c,
                    body,
                    places,
                    place,
                    c.ty.types.size_of(ty),
                    &self.private_bytes,
                )
            })
            && !places.paths[place].1.contains(&Projection::Deref)
            && !self.exposed.iter().any(|p| {
                !places.paths[*p].1.contains(&Projection::Deref)
                    && places.paths[*p].0.0 == places.paths[root].0.0
            })
    }
}
