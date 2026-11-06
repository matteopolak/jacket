//! Byte correspondence at checked read/write epochs, separate from ownership.
use jaic::fxhash::HashMap;
use jaic::ir::Val;
use jaic::sema::{Compiler, ProcId};
use jaic::types::TypeId;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(in crate::rules) struct EventSite {
    pub procedure: ProcId,
    pub block: usize,
    pub event: usize,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(in crate::rules) struct Site {
    pub chain: Vec<EventSite>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(in crate::rules) struct ByteSlot {
    pub offset: u64,
    pub width: u64,
    pub ty: TypeId,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(in crate::rules) enum Length {
    InputStringCount(usize),
    Constant(i128),
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(in crate::rules) enum RangeBase {
    InputStringData(usize),
    CopyDestination(Site),
    DescriptorHeader {
        procedure: ProcId,
        place: Val,
        callers: Vec<EventSite>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(in crate::rules) struct ByteRange {
    pub base: RangeBase,
    pub offset: i128,
    pub length: Length,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::rules) struct Separation {
    pub left: ByteRange,
    pub right: ByteRange,
    pub store: Option<Site>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::rules) struct Preconditions {
    pub nonnegative_length: bool,
    pub readable: ByteRange,
    pub writable: ByteRange,
    pub separations: Vec<Separation>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::rules) struct CopyRelation {
    pub source_formal: usize,
    pub result: usize,
    pub source_data: ByteSlot,
    pub source_count: ByteSlot,
    pub result_data: ByteSlot,
    pub result_count: ByteSlot,
    pub source_offset: i128,
    pub length: Length,
    pub read_epoch: Site,
    pub write_epoch: Site,
    pub preconditions: Preconditions,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(in crate::rules) struct ReturnedCopy {
    pub alternatives: Vec<CopyRelation>,
    pub incomplete: bool,
}

#[derive(Default)]
pub(in crate::rules) struct Analysis {
    copies: HashMap<ProcId, ReturnedCopy>,
}

impl Analysis {
    /// Build run-local byte relations; for example, query `analysis.returned_copy(id)`.
    pub fn build(c: &Compiler) -> Self {
        if jaic::interrupt::should_stop() {
            return Self::default();
        }
        let Some(layout) = domain::Layout::of(c) else {
            return Self::default();
        };
        let mut solver = checked::Solver::new(c, layout);
        let mut out = Self::default();
        for i in 0..c.pr.procs.len() {
            if jaic::interrupt::should_stop() {
                return Self::default();
            }
            let id = ProcId(i as u32);
            let Some(sig) = &c.pr.procs[i].sig else {
                continue;
            };
            let positions: Vec<_> = sig
                .returns
                .iter()
                .enumerate()
                .filter_map(|(i, ty)| (*ty == TypeId::STRING).then_some(i))
                .collect();
            if positions.is_empty() {
                continue;
            }
            if let Some(summary) = solver.summary(id)
                && let Some(copy) = ranges::returned(&summary, &positions, layout)
            {
                out.copies.insert(id, copy);
            }
        }
        if jaic::interrupt::should_stop() {
            Self::default()
        } else {
            out
        }
    }

    /// Alternatives cover normal returns for their result. None is unknown, never purity.
    pub fn returned_copy(&self, id: ProcId) -> Option<&ReturnedCopy> {
        self.copies.get(&id)
    }
}

#[cfg(test)]
#[path = "content_relations/tests.rs"]
mod tests;

#[path = "content_relations/bindings.rs"]
mod bindings;
#[path = "content_relations/checked.rs"]
mod checked;
#[path = "content_relations/domain.rs"]
mod domain;
#[path = "content_relations/ranges.rs"]
mod ranges;
