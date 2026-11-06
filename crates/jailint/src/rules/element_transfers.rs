//! Checked logical element transfers, independent of allocation relocation.
use super::lifetime_facts::{Address, EventSite as Site, StackRoots, current_body};
use jaic::fxhash::HashMap;
use jaic::sema::{Compiler, ProcId};
use jaic::types::TypeId;
use std::rc::Rc;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::rules) struct Preconditions {
    pub valid_incoming_index: bool,
    pub descriptor_disjoint_from_storage: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::rules) struct Evidence {
    pub count_write: Site,
    pub element_write: Site,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::rules) struct UnorderedRemove {
    pub receiver_formal: usize,
    pub index_formal: usize,
    pub element: TypeId,
    pub preconditions: Preconditions,
    pub count_delta: i128,
    pub evidence: Vec<Evidence>,
}

impl UnorderedRemove {
    /// Given the relation preconditions, the incoming logical identity occupying a surviving output position.
    /// This maps identities; it does not prove header/storage separation at a call.
    /// For example, removing index one from four elements maps output one to old three.
    pub fn old_index(&self, new: i128, removed: i128, old_count: i128) -> Option<i128> {
        if removed < 0 || removed >= old_count || new < 0 || new >= old_count.checked_sub(1)? {
            return None;
        }
        Some(if new == removed {
            old_count - 1
        } else {
            new
        })
    }
}

#[derive(Default)]
pub(in crate::rules) struct Analysis {
    transfers: HashMap<ProcId, UnorderedRemove>,
}

impl Analysis {
    /// Infer conditional maps from current checked bodies, discarding partial cancellation.
    /// For example, build `let analysis = Analysis::build(compiler);`, then query `analysis.transfer(id)`.
    pub fn build(c: &Compiler) -> Self {
        use jaic::types::{ArrayKind, TypeKind};
        if jaic::interrupt::should_stop() {
            return Self::default();
        }
        let Some(facts) = c.ed.facts() else {
            return Self::default();
        };
        let mut out = Self::default();
        for (id, body) in &facts.lifetime_bodies {
            if jaic::interrupt::should_stop() {
                return Self::default();
            }
            if !current_body(c, *id, body) {
                continue;
            }
            let procedure = &c.pr.procs[id.0 as usize];
            let Some(signature) = &procedure.sig else {
                continue;
            };
            for (receiver, param) in signature.params.iter().enumerate() {
                let Some(pointee) = c.ty.types.pointee(param.ty) else {
                    continue;
                };
                let TypeKind::Array {
                    elem,
                    kind: ArrayKind::View | ArrayKind::Resizable,
                } = *c.ty.types.kind(c.ty.types.repr(pointee))
                else {
                    continue;
                };
                if c.ty.types.size_of(elem) == 0 {
                    continue;
                }
                for (index, param) in signature.params.iter().enumerate() {
                    if index == receiver || c.ty.types.int_info(param.ty) != Some((64, true)) {
                        continue;
                    }
                    if let Some(transfer) = checked::infer(c, body, receiver, index, elem) {
                        out.transfers.insert(*id, transfer);
                    }
                    if jaic::interrupt::should_stop() {
                        return Self::default();
                    }
                }
            }
        }
        out
    }

    /// A conditional exact map; absent means unknown, never absence of mutation.
    pub fn transfer(&self, id: ProcId) -> Option<&UnorderedRemove> {
        self.transfers.get(&id)
    }
}

#[cfg(test)]
#[path = "element_transfers/tests.rs"]
mod tests;

#[path = "element_transfers/checked.rs"]
mod checked;
#[path = "element_transfers/domain.rs"]
mod domain;
