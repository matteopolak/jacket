//! Certified byte writes forget mutable content cells, never captured SSA values.
use super::{Cell, Cells, Event, HashMap, LifetimeBody, Places, TypeKind};
use crate::rules::lifetime_facts::{Address, ByteRange};
use crate::rules::lifetime_movement::facts::Site;
use crate::rules::lifetime_summary::no_growth::calls::Calls;
use jaic::sema::Compiler;

#[derive(Clone, Copy, PartialEq, Eq)]
struct Location {
    address: Address,
    bytes: u64,
}

/// Resolve represented contents against the final model, including slots loaded
/// only after the invocation. Unsupported addresses remain possibly overlapping.
pub(super) fn prepare(
    c: &Compiler,
    body: &LifetimeBody,
    places: &Places,
    cells: &Cells,
    calls: &mut Calls,
    sites: &[Site],
) -> Option<Vec<Vec<usize>>> {
    if calls.is_empty() {
        return Some(Vec::new());
    }
    let events = body.blocks.iter().map(|b| b.events.len()).sum::<usize>();
    calls.charge(events + cells.values.len() + places.paths.len())?;
    let addresses = calls.addresses.as_ref()?;
    let mut locations = HashMap::<usize, Option<Location>>::default();
    let locate = |value, ty| {
        let address = addresses.get(value)?;
        let bytes = c.ty.types.size_of(ty);
        (ByteRange {
            offset: address.offset,
            width: bytes,
        })
        .within(addresses.roots.extent(address.root).flatten())
        .then_some(Location {
            address,
            bytes,
        })
    };
    for (p, ((root, _), path)) in places.paths.iter().enumerate() {
        if path.is_empty() {
            locations.insert(p, places.selected_type(p).and_then(|ty| locate(*root, ty)));
        }
    }
    for e in body.blocks.iter().flat_map(|b| &b.events) {
        let (value, ty) = match e {
            Event::Load {
                place,
                ty,
                ..
            }
            | Event::Store {
                place,
                ty,
                ..
            }
            | Event::Initialize {
                place,
                ty,
                ..
            }
            | Event::Address {
                place,
                ty,
                ..
            } => (*place, *ty),
            Event::FieldProjection {
                result,
                result_ty,
                ..
            } => (*result, *result_ty),
            _ => continue,
        };
        let Some(place) = places.existing(value, ty) else {
            continue;
        };
        let current = locate(value, ty);
        if let Some(old) = locations.get_mut(&place) {
            if *old != current {
                *old = None;
            }
        } else {
            locations.insert(place, current);
        }
    }
    // A builtin descriptor lifetime cell represents its DATA pointer, not count.
    for (&place, location) in &mut locations {
        if let Some(ty) = places.selected_type(place)
            && matches!(
                c.ty.types.kind(ty),
                TypeKind::String
                    | TypeKind::Array {
                        kind: jaic::types::ArrayKind::View | jaic::types::ArrayKind::Resizable,
                        ..
                    }
            )
            && let Some(at) = location
        {
            at.address.offset = at.address.offset.checked_add(jaic::layout::view::DATA)?;
            at.bytes = jaic::layout::POINTER_SIZE;
        }
    }
    let mut work = 0usize;
    let mut updates = Vec::new();
    for &site in sites {
        let call = calls.get(site)?;
        let mut affected = Vec::new();
        for write in call.writes {
            for (cell, key) in cells.values.iter().enumerate() {
                work = work.checked_add(1)?;
                if calls.work.checked_add(work)?
                    > crate::rules::lifetime_summary::no_growth::calls::MAX_WORK
                    || jaic::interrupt::should_stop()
                {
                    return None;
                }
                let Cell::Place(place) = key else {
                    continue;
                };
                let separate = locations.get(place).and_then(|l| *l).is_some_and(|at| {
                    at.bytes == 0
                        || (at.address.root != write.address.root
                            && addresses.roots.extent(at.address.root).flatten().is_some()
                            && (addresses
                                .roots
                                .extent(write.address.root)
                                .flatten()
                                .is_some()
                                || call.native.as_deref().is_some_and(|native| {
                                    native.root == write.address.root
                                        && (ByteRange {
                                            offset: write.address.offset,
                                            width: write.bytes,
                                        })
                                        .within(Some(native.bytes))
                                })))
                        || (at.address.root == write.address.root
                            && (ByteRange {
                                offset: at.address.offset,
                                width: at.bytes,
                            })
                            .separated(ByteRange {
                                offset: write.address.offset,
                                width: write.bytes,
                            }))
                });
                if !separate {
                    affected.push(cell);
                }
            }
        }
        updates.push(affected);
    }
    calls.charge(work)?;
    Some(updates)
}
