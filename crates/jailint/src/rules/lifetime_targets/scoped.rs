//! Closed pointer slots publishing activation-owned resizable-array descriptors.
use super::{Compiler, Event, HashMap, HashSet, LifetimeBody, LifetimeOperand, ProcId};
use jaic::ir::{GlobalId, Storage};
use jaic::sema::scope::{EntityState, Resolved};

pub(super) fn build(
    compiler: &Compiler,
    bodies: &HashMap<ProcId, &LifetimeBody>,
    entry_module: Option<jaic::sema::ModuleId>,
) -> HashSet<GlobalId> {
    let mut candidates = HashSet::default();
    let mut rejected = HashSet::default();
    let mut written = HashSet::default();
    for body in bodies.values() {
        let mut slots = HashMap::default();
        for event in body.blocks.iter().flat_map(|b| &b.events) {
            if let Event::Expression {
                operand: LifetimeOperand::Place(value),
                ty,
                entity: Some(entity),
                ..
            } = event
                && compiler.ty.types.pointee(*ty).is_some_and(|t| {
                    matches!(
                        compiler.ty.types.kind(t),
                        jaic::types::TypeKind::Array {
                            kind: jaic::types::ArrayKind::Resizable,
                            ..
                        }
                    )
                })
            {
                let entity = compiler.entity(*entity);
                if let EntityState::Done(Resolved::Global {
                    storage: Storage::Data(id),
                    ..
                }) = entity.state
                {
                    slots.insert(*value, id);
                    if entity.file_private
                        || !entity.exported
                        || [compiler.ns.main_module, entry_module]
                            .contains(&Some(compiler.ns.scopes[entity.scope.0 as usize].module))
                    {
                        candidates.insert(id);
                    } else {
                        rejected.insert(id);
                    }
                }
            }
        }
        for event in body.blocks.iter().flat_map(|b| &b.events) {
            match event {
                Event::Store {
                    place,
                    value,
                    constant,
                    ..
                } if slots.contains_key(place) => {
                    let id = slots[place];
                    if *constant == Some(0) {
                        continue;
                    }
                    if body.stack_places.contains(value) {
                        written.insert(id);
                    } else {
                        rejected.insert(id);
                    }
                }
                Event::Address {
                    place, ..
                }
                | Event::BoxAny {
                    place, ..
                } if slots.contains_key(place) => {
                    rejected.insert(slots[place]);
                }
                _ => {}
            }
        }
    }
    candidates.retain(|id| {
        let global = &compiler.out.program.globals[id.0 as usize];
        written.contains(id)
            && !rejected.contains(id)
            && global.export.is_none()
            && global.relocs.is_empty()
            && global.init.iter().all(|&byte| byte == 0)
    });
    candidates
}
