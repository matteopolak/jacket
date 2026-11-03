//! Closed program-data procedure slots. Exports and exposed addresses keep an open-world member.
use super::{
    Compiler, Event, HashMap, HashSet, LifetimeBody, LifetimeOperand, ProcId, Set, Val, procedure,
};
use jaic::ir::{GlobalId, RelocTarget, Storage};
use jaic::types::TypeId;

#[derive(Clone, Copy)]
pub(super) struct Slot {
    pub global: GlobalId,
    pub ty: TypeId,
    pub private: bool,
}

use jaic::sema::{
    procs::ProcTarget,
    scope::{EntityState, Resolved},
};

#[derive(Default)]
pub(in crate::rules) struct Globals {
    pub initial: HashMap<GlobalId, Set>,
    pub users: HashMap<GlobalId, HashSet<ProcId>>,
    pub used_by: HashMap<ProcId, HashSet<GlobalId>>,
}

pub(super) fn slots(compiler: &Compiler, body: &LifetimeBody) -> HashMap<Val, Slot> {
    body.blocks
        .iter()
        .flat_map(|b| &b.events)
        .filter_map(|e| {
            let Event::Expression {
                operand: LifetimeOperand::Place(v),
                ty,
                entity: Some(entity),
                ..
            } = e
            else {
                return None;
            };
            if !procedure(compiler, *ty) {
                return None;
            }
            let EntityState::Done(Resolved::Global {
                storage: Storage::Data(id),
                ..
            }) = compiler.entity(*entity).state
            else {
                return None;
            };
            Some((
                *v,
                Slot {
                    global: id,
                    ty: *ty,
                    private: compiler.entity(*entity).file_private
                        || !compiler.entity(*entity).exported,
                },
            ))
        })
        .collect()
}

impl Globals {
    pub(in crate::rules) fn build(
        compiler: &Compiler,
        valid: &HashMap<ProcId, &LifetimeBody>,
    ) -> Self {
        let mut this = Self::default();
        let mut exposed = HashSet::default();
        let mut private = HashSet::default();
        for (&id, body) in valid {
            let slots = slots(compiler, body);
            for slot in slots.values() {
                let global = slot.global;
                if slot.private {
                    private.insert(global);
                }
                this.users.entry(global).or_default().insert(id);
                this.used_by.entry(id).or_default().insert(global);
            }
            for event in body.blocks.iter().flat_map(|b| &b.events) {
                if let Event::Address {
                    place, ..
                }
                | Event::BoxAny {
                    place, ..
                } = event
                    && let Some(global) = slots.get(place)
                {
                    exposed.insert(global.global);
                }
            }
        }
        if this.users.is_empty() {
            return this;
        }
        let mut targets = HashMap::default();
        for (id, proc) in compiler.pr.procs.iter().enumerate() {
            let key = match proc.target {
                Some(ProcTarget::Func(f)) => Some(RelocTarget::Func(f)),
                Some(ProcTarget::Foreign(f)) => Some(RelocTarget::Foreign(f)),
                None => None,
            };
            if let Some(key) = key {
                targets.insert(key, ProcId(id as u32));
            }
        }
        for &id in this.users.keys() {
            let global = &compiler.out.program.globals[id.0 as usize];
            let mut set = Set {
                unknown: global.export.is_some()
                    || exposed.contains(&id)
                    || (!global.read_only && !private.contains(&id)),
                ..Set::default()
            };
            if global.relocs.is_empty() {
                set.unknown |= global.init.iter().any(|&b| b != 0);
            } else {
                for reloc in &global.relocs {
                    if reloc.offset == 0
                        && reloc.addend == 0
                        && let Some(&proc) = targets.get(&reloc.target)
                    {
                        set.procedures.insert(proc);
                    } else {
                        set.unknown = true;
                    }
                }
            }
            this.initial.insert(id, set);
        }
        this
    }
}
