//! Actual stack roots and entry snapshots, prepared before CFG interpretation.
use super::domain::*;
use super::*;
use jaic::ide::lifetimes::{LifetimeBody, LifetimeEvent as Event, LifetimeOverlayAccess};

pub(super) type Overlays<'a> = HashMap<(usize, usize), &'a LifetimeOverlayAccess>;

pub(super) struct Entry<'a> {
    pub initial: State,
    pub copies: HashMap<usize, Site>,
    pub overlays: Overlays<'a>,
}

/// Prepare original extents and certified copies, for example a by-value mark spill.
pub(super) fn prepare<'a>(
    c: &Compiler,
    b: &'a LifetimeBody,
    bypass: Option<super::paged::Inputs>,
) -> Option<Entry<'a>> {
    if b.blocks.is_empty()
        || b.blocks.len() > STEPS
        || b.stack_roots.len() > STEPS
        || b.slot_aliases.len() > STEPS
        || b.comparisons.len() > 256
        || b.overlay_accesses.len() > STEPS
    {
        return None;
    }
    let overlays: HashMap<_, _> = b
        .overlay_accesses
        .iter()
        .map(|a| ((a.block, a.event), a))
        .collect();
    if overlays.len() != b.overlay_accesses.len() {
        return None;
    }
    let mut initial = State {
        bypass,
        ..State::default()
    };
    for r in &b.stack_roots {
        if jaic::interrupt::should_stop() {
            return None;
        }
        initial.extents.insert(r.value, r.bytes);
        initial.addresses.insert(
            r.value,
            Address {
                root: Root::Local(r.value),
                offset: 0,
            },
        );
    }
    for (a, r) in &b.slot_aliases {
        if jaic::interrupt::should_stop() {
            return None;
        }
        initial.addresses.insert(
            *a,
            Address {
                root: Root::Local(*r),
                offset: 0,
            },
        );
    }
    // Aggregate Return identifies a borrowed result header, never owned storage.
    for event in b.blocks.iter().flat_map(|b| &b.events) {
        if jaic::interrupt::should_stop() {
            return None;
        }
        if let Event::Return {
            value,
            ty,
            index,
            ..
        } = event
            && !scalar(c, *ty)
            && !initial.addresses.contains_key(value)
        {
            initial.addresses.insert(
                *value,
                Address {
                    root: Root::Output {
                        value: *value,
                        ty: *ty,
                        result: *index,
                    },
                    offset: 0,
                },
            );
        }
    }
    let mut entry_copies = HashMap::default();
    let mut last_memory_write = None;
    for (event, record) in b.blocks[0].events.iter().enumerate() {
        if jaic::interrupt::should_stop() {
            return None;
        }
        if matches!(
            record,
            Event::Store { .. }
                | Event::Initialize { .. }
                | Event::Call { .. }
                | Event::IndirectCall { .. }
                | Event::Barrier
        ) {
            last_memory_write = Some((event, record));
        }
        let Event::Parameter {
            formal,
            place,
            incoming,
            ty,
            ..
        } = record
        else {
            continue;
        };
        if scalar(c, *ty)
            || !initial.address(*place).is_some_and(|a| {
                matches!(a.root, Root::Local(_)) && initial.checked(c, &a, c.ty.types.size_of(*ty))
            })
        {
            continue;
        }
        if let Some((
            copy,
            Event::Store {
                place: destination,
                value,
                ty: copied_ty,
                constant: None,
                ..
            },
        )) = last_memory_write
            && destination == place
            && value == incoming
            && copied_ty == ty
        {
            entry_copies.insert(*formal, site(b.procedure, 0, copy));
        }
    }
    Some(Entry {
        initial,
        copies: entry_copies,
        overlays,
    })
}
