//! Captured address identities for checked receivers and helper footprints.
//! Later stores cannot retarget an already evaluated SSA operand.
use super::physical_slots::Slot;
use super::{Def, Event, HashMap, HashSet, LifetimeArgument, Projection, Resolver, Summary, Val};
use crate::rules::lifetime_movement::facts::Site;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(super) enum Base {
    Input(usize),
    Local(Val),
    Entry(Slot),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(super) struct Object {
    pub base: Base,
    pub offset: u64,
}

impl Object {
    pub(super) fn input(formal: usize) -> Self {
        Self {
            base: Base::Input(formal),
            offset: 0,
        }
    }

    fn add(self, offset: u64) -> Option<Self> {
        Some(Self {
            offset: self.offset.checked_add(offset)?,
            ..self
        })
    }
}

impl Resolver<'_> {
    /// Resolve the value already evaluated at this event. Later stores cannot
    /// change a captured operand; unsupported reads remain unknown.
    pub(super) fn address_object(&self, value: Val) -> Option<Object> {
        self.address_object_inner(value)
    }

    fn address_object_inner(&self, value: Val) -> Option<Object> {
        let object = self.address_value(value, &mut HashSet::default(), 0, &mut 256)?;
        self.validate_lock_object(object)
    }

    fn address_value(
        &self,
        value: Val,
        seen: &mut HashSet<Val>,
        depth: usize,
        budget: &mut usize,
    ) -> Option<Object> {
        *budget = budget.checked_sub(1)?;
        if depth > 24 || !seen.insert(value) || self.address_casts.contains_key(&value) {
            return None;
        }
        let mut projections = self
            .body
            .blocks
            .iter()
            .flat_map(|b| &b.events)
            .filter_map(|event| match event {
                Event::FieldProjection {
                    base,
                    result,
                    offset,
                    ..
                } if *result == value && *base != value => Some((*base, *offset)),
                _ => None,
            });
        if let Some((base, offset)) = projections.next() {
            let result = self
                .address_child(base, seen, depth + 1, budget)?
                .add(offset)?;
            for (base, offset) in projections {
                if self
                    .address_child(base, seen, depth + 1, budget)?
                    .add(offset)?
                    != result
                {
                    return None;
                }
            }
            return Some(result);
        }
        if self
            .body
            .blocks
            .iter()
            .flat_map(|b| &b.events)
            .any(|event| {
                matches!(event,
            Event::Offset {value: result, offset, ..} if *result == value && *offset != Some(0))
            })
        {
            return None;
        }
        let mut result = None;
        let Some(defs) = self.values.get(&value) else {
            // A selected inline projection can use the actual stack address
            // directly, without a separate source-level address expression.
            return self.validate_lock_object(Object {
                base: Base::Local(value),
                offset: 0,
            });
        };
        for def in defs {
            if self.values[&value].len() > 1
                && matches!(def, Def::Address(place) if self.places.paths[*place].0.0 == value && self.places.paths[*place].1.is_empty())
            {
                continue;
            }
            let object = match def {
                Def::Parameter(formal, ..) => {
                    let (_, ty) = self.params.values().find(|(index, _)| index == formal)?;
                    if !self.compiler.ty.types.is_pointer(*ty) {
                        return None;
                    }
                    Object::input(*formal)
                }
                Def::Copy(source) => self.address_child(*source, seen, depth + 1, budget)?,
                Def::Address(place) => self.lock_place(*place)?,
                Def::Load(place, _) => {
                    let sources = self.storage_versions.get(&value)?;
                    let mut sources = sources.iter();
                    let source = sources.next()?;
                    if sources.next().is_some() {
                        return None;
                    }
                    match source {
                        super::storage_versions::Source::Value(source) => {
                            self.address_child(*source, seen, depth + 1, budget)?
                        }
                        super::storage_versions::Source::Initial => Object {
                            base: Base::Entry(
                                self.physical_slot(
                                    *place,
                                    self.compiler
                                        .ty
                                        .types
                                        .size_of(*self.places.types.get(place)?),
                                )?,
                            ),
                            offset: 0,
                        },
                        _ => return None,
                    }
                }
                _ => return None,
            };
            if result.is_some_and(|old| old != object) {
                return None;
            }
            result = Some(object);
        }
        result
    }

    fn address_child(
        &self,
        value: Val,
        seen: &HashSet<Val>,
        depth: usize,
        budget: &mut usize,
    ) -> Option<Object> {
        self.address_value(value, &mut seen.clone(), depth, budget)
    }

    fn lock_place(&self, place: usize) -> Option<Object> {
        if let Some(slot) = self.physical_slot(place, 0) {
            return Some(Object {
                base: Base::Input(slot.formal),
                offset: slot.offset,
            });
        }
        let ((root, root_ty), steps) = &self.places.paths[place];
        if self.places.unresolved(place) || !self.owned.contains(&self.places.root_of(place)) {
            return None;
        }
        // A positively owned inline stack object has an address even when no
        // pointer-value definition exists for its root SSA.
        let mut object = Object {
            base: Base::Local(*root),
            offset: 0,
        };
        let mut ty = *root_ty;
        for step in steps {
            match step {
                Projection::Field(name) => {
                    let (offset, field_ty) = super::physical_slots::field_projection(
                        self.compiler,
                        self.body,
                        &self.places,
                        ty,
                        *name,
                    )?;
                    object = object.add(offset)?;
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
                    object =
                        object.add(index.checked_mul(self.compiler.ty.types.size_of(*elem))?)?;
                    ty = *elem;
                }
                _ => return None,
            }
        }
        Some(object)
    }

    /// A callee's input pointer is the caller's captured actual SSA. An entry
    /// pointer-slot read is materialized in the caller immediately before call.
    pub(super) fn instantiate_lock_object(
        &self,
        object: Object,
        args: &[LifetimeArgument],
        site: Site,
        all: &HashMap<super::ProcId, Summary>,
    ) -> Option<Object> {
        let base = match object.base {
            Base::Input(formal) => {
                let arg = args.iter().find(|a| a.formal == formal && !a.spread)?;
                self.address_object(super::value(arg)?)?
            }
            Base::Entry(slot) => {
                let actual = self.substitute_slot(slot, args)?;
                let sources = self.before(site, actual, all);
                let mut chosen = None;
                for source in sources {
                    let current = match source {
                        super::value_writes::Source::Entry => Object {
                            base: Base::Entry(actual),
                            offset: 0,
                        },
                        super::value_writes::Source::Value(value) => self.address_object(value)?,
                        _ => return None,
                    };
                    if chosen.is_some_and(|old| old != current) {
                        return None;
                    }
                    chosen = Some(current);
                }
                chosen?
            }
            Base::Local(_) => return None,
        };
        self.validate_lock_object(base.add(object.offset)?)
    }

    /// A cast or callee-relative field cannot enlarge a local allocation. This
    /// certifies an interior address only, not the native object's full layout.
    pub(super) fn validate_lock_object(&self, object: Object) -> Option<Object> {
        if let Base::Local(value) = object.base {
            let bytes = self
                .places
                .paths
                .iter()
                .enumerate()
                .filter_map(|(place, ((root, _), steps))| {
                    (*root == value && steps.is_empty())
                        .then(|| self.private_bytes.get(&place).copied())
                        .flatten()
                })
                .min()?;
            if object.offset >= bytes {
                return None;
            }
        }
        Some(object)
    }
}


impl Resolver<'_> {
    /// The first retention consumer admits no repeated or merged invocation sites.
    /// This uses the existing flow driver; unknown topology loses only the capture.
    pub(in crate::rules) fn retention_capture_scope(&self, work: &mut usize) -> Option<()> {
        if self.body.blocks.is_empty() || self.body.blocks.len() > 32 {
            return None;
        }
        let mut rows = self.body.blocks.len();
        for b in &self.body.blocks {
            rows = rows
                .checked_add(b.events.len())?
                .checked_add(b.successors.len())?;
            if rows > 512 || b.successors.iter().any(|s| *s >= self.body.blocks.len()) {
                return None;
            }
        }
        *work = work.checked_sub(rows)?;
        let mut repeated = false;
        super::flow::states(
            self.body,
            &super::flow::Live::all(self.body),
            (),
            |(), ()| {
                repeated = true;
                false
            },
            |(), _, _| {},
        )?;
        (!repeated && !jaic::interrupt::should_stop()).then_some(())
    }
}
