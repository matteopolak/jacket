//! Conditional access templates over captured pointer values, never typed slots.
use super::{Builder, Event, HashSet, Resolver, TypeId, Val};
use crate::rules::lifetime_movement::facts::Site;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::rules) struct Pointer {
    pub formal: usize,
    pub displacement: i64,
}

/// The caller must prove this access fits a live original object at this epoch.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::rules) struct Access {
    pub pointer: Pointer,
    pub width: u64,
    pub ty: TypeId,
    pub site: Site,
}

/// The stored pointer remains a publication, independently of access validity.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::rules) struct Write {
    pub access: Access,
    pub value: Pointer,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(in crate::rules) struct Conditional {
    pub reads: Vec<Access>,
    pub writes: Vec<Write>,
}

impl Conditional {
    pub(super) fn payload_bytes(&self) -> usize {
        self.reads.capacity() * std::mem::size_of::<Access>()
            + self.writes.capacity() * std::mem::size_of::<Write>()
    }
}

impl Builder {
    /// Resolve a conditional input expression after neutral captured-value normalization.
    fn pointer(
        &mut self,
        r: &Resolver<'_>,
        value: Val,
        seen: &mut HashSet<Val>,
        depth: usize,
    ) -> Option<Pointer> {
        self.charge(self.events + seen.len() + 1)?;
        if depth >= 24 || !seen.insert(value) {
            return None;
        }
        let index = self.indexed(r)?;
        let mut remaining = super::MAX_WORK.checked_sub(self.work)?;
        let normalized = index.pointer_leaf(r, value, &mut remaining, depth);
        self.work = super::MAX_WORK - remaining;
        let result =
            normalized.and_then(|(leaf, depth)| self.pointer_definition(r, leaf, seen, depth));
        seen.remove(&value);
        result
    }

    fn pointer_definition(
        &mut self,
        r: &Resolver<'_>,
        value: Val,
        seen: &mut HashSet<Val>,
        depth: usize,
    ) -> Option<Pointer> {
        let mut result = None;
        for e in &r.body.blocks[0].events {
            let current = match e {
                Event::Offset {
                    value: target,
                    source,
                    stride,
                    offset,
                    ..
                } if *target == value && *source != value => {
                    let width = r.compiler.ty.types.size_of(*stride);
                    if width == 0 {
                        return None;
                    }
                    let delta = i64::try_from((*offset)?.checked_mul(i128::from(width))?).ok()?;
                    let base = self.pointer(r, *source, seen, depth + 1)?;
                    Some(Pointer {
                        displacement: base.displacement.checked_add(delta)?,
                        ..base
                    })
                }
                Event::FieldProjection {
                    result, ..
                } if *result == value => return None,
                _ => None,
            };
            if let Some(current) = current {
                if result.is_some_and(|old| old != current) {
                    return None;
                }
                result = Some(current);
            }
        }
        if result.is_some() {
            return result;
        }
        let defs = r.values.get(&value)?;
        self.charge(defs.len())?;
        let mut formal = None;
        for def in defs {
            let super::super::Def::Parameter(index, ..) = def else {
                return None;
            };
            let sig = r
                .compiler
                .pr
                .procs
                .get(r.body.procedure.0 as usize)?
                .sig
                .as_ref()?;
            if !r.compiler.ty.types.is_pointer(sig.params.get(*index)?.ty) {
                return None;
            }
            let current = Pointer {
                formal: *index,
                displacement: 0,
            };
            if formal.is_some_and(|old| old != current) {
                return None;
            }
            formal = Some(current);
        }
        formal
    }

    /// Observe one event in the existing traversal; unsupported effects reject all rows.
    pub(super) fn raw_event(
        &mut self,
        r: &Resolver<'_>,
        block: usize,
        ordinal: usize,
        event: &Event,
    ) -> Option<()> {
        match event {
            Event::Store {
                place,
                value,
                ty,
                ..
            } => {
                self.charge(r.places.paths.len() + 1)?;
                let width = r.compiler.ty.types.size_of(*ty);
                if !r.compiler.ty.types.is_pointer(*ty) && !Self::scalar(r, *ty) {
                    return None;
                }
                if r.places
                    .existing(*place, *ty)
                    .is_some_and(|p| r.private_root(p, width))
                {
                    return Some(());
                }
                if !r.compiler.ty.types.is_pointer(*ty) || width == 0 {
                    return None;
                }
                let pointer = self.pointer(r, *place, &mut HashSet::default(), 0)?;
                let value = self.pointer(r, *value, &mut HashSet::default(), 0)?;
                if self.raw.reads.len() + self.raw.writes.len() >= super::MAX_FOOTPRINTS {
                    return None;
                }
                self.raw.writes.push(Write {
                    access: Access {
                        pointer,
                        width,
                        ty: *ty,
                        site: Site {
                            block,
                            event: ordinal,
                        },
                    },
                    value,
                });
                Some(())
            }
            Event::Load {
                place,
                ty,
                ..
            }
            | Event::Indirect {
                place,
                ty,
            } => {
                self.charge(r.places.paths.len() + 1)?;
                let width = r.compiler.ty.types.size_of(*ty);
                if !r.compiler.ty.types.is_pointer(*ty) && !Self::scalar(r, *ty) {
                    return None;
                }
                if r.places
                    .existing(*place, *ty)
                    .is_some_and(|p| r.private_root(p, width))
                {
                    return Some(());
                }
                if width == 0 {
                    return None;
                }
                let pointer = self.pointer(r, *place, &mut HashSet::default(), 0)?;
                if self.raw.reads.len() + self.raw.writes.len() >= super::MAX_FOOTPRINTS {
                    return None;
                }
                self.raw.reads.push(Access {
                    pointer,
                    width,
                    ty: *ty,
                    site: Site {
                        block,
                        event: ordinal,
                    },
                });
                Some(())
            }
            Event::ReadMemory {
                value, ..
            } => self.typed_reads.contains(value).then_some(()),
            Event::View {
                from,
                to,
                ..
            } => (r.compiler.ty.types.is_pointer(*from)
                && r.compiler.ty.types.is_pointer(*to)
                && r.compiler.ty.types.size_of(*from) == r.compiler.ty.types.size_of(*to))
            .then_some(()),
            Event::ContextSet {
                ..
            }
            | Event::ContextRead {
                ..
            }
            | Event::ContextAlias {
                ..
            }
            | Event::Expression {
                ..
            }
            | Event::Parameter {
                ..
            }
            | Event::Use {
                ..
            }
            | Event::Address {
                ..
            }
            | Event::Copy {
                ..
            }
            | Event::Offset {
                ..
            }
            | Event::Local {
                ..
            }
            | Event::ScopeEnd {
                ..
            }
            | Event::StatementEnd {
                ..
            }
            | Event::StaticChoice {
                ..
            }
            | Event::Expansion {
                ..
            }
            | Event::Cleanup {
                ..
            } => Some(()),
            _ => None,
        }
    }
}
