//! Observed allocator-provider pairs, independent of ownership and freshness.
use jaic::fxhash::HashMap;
use jaic::ir::{CmpOp, Ty, Val};
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

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(in crate::rules) enum ReadEpoch {
    Load(Site),
    AggregateCopy(Site),
    ArgumentCopy { call: Site, formal: usize },
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(in crate::rules) enum Captured {
    FormalValue {
        formal: usize,
        slot: ByteSlot,
    },
    ReceiverRead {
        formal: usize,
        receiver_ty: TypeId,
        slot: ByteSlot,
        epoch: ReadEpoch,
    },
    ContextRead {
        context_ty: TypeId,
        selection: Site,
        slot: ByteSlot,
        epoch: ReadEpoch,
    },
    Procedure(ProcId),
    Constant {
        ty: TypeId,
        bits: u64,
    },
    Unknown,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(in crate::rules) struct Pair {
    pub procedure_value: Captured,
    pub data_value: Captured,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(in crate::rules) enum Operation {
    Allocate,
    Resize,
    Free,
    OtherConstant { ty: TypeId, bits: u64 },
    Unknown,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(in crate::rules) enum ValueSource {
    Callback {
        use_site: Site,
        result: usize,
        byte_offset: i64,
    },
    Null,
    Unknown,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(in crate::rules) enum GuardValue {
    Captured(Captured),
    Source(ValueSource),
    Unknown,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::rules) enum Predicate {
    Compare {
        value: GuardValue,
        op: CmpOp,
        constant: u64,
        ty: Ty,
    },
    Not(Box<Predicate>),
    And(Box<Predicate>, Box<Predicate>),
    Or(Box<Predicate>, Box<Predicate>),
    Unknown,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::rules) struct Guard {
    pub predicate: Predicate,
    pub truth: bool,
}

/// A possible selected invocation. Its pair contains the values captured before the callback.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::rules) struct ProviderUse {
    pub site: Site,
    pub operation: Operation,
    pub provider: Pair,
    pub guards: Vec<Guard>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::rules) struct ReturnSource {
    pub result: usize,
    pub source: ValueSource,
    pub guards: Vec<Guard>,
}

/// An observed write, rather than a promise about the field at procedure exit.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::rules) struct StoreSource {
    pub site: Site,
    pub receiver_formal: usize,
    pub receiver_ty: TypeId,
    pub slot: ByteSlot,
    pub source: ValueSource,
    pub guards: Vec<Guard>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(in crate::rules) struct Relations {
    pub uses: Vec<ProviderUse>,
    pub returns: Vec<ReturnSource>,
    pub stores: Vec<StoreSource>,
    pub incomplete: bool,
}

#[derive(Default)]
pub(in crate::rules) struct Analysis {
    relations: HashMap<ProcId, Relations>,
}

impl Analysis {
    /// Build run-local observations; for example, query `analysis.provenance(id)`.
    pub fn build(c: &Compiler) -> Self {
        if jaic::interrupt::should_stop() {
            return Self::default();
        }
        let mut solver = checked::Solver::new(c);
        let mut out = Self::default();
        for i in 0..c.pr.procs.len() {
            if jaic::interrupt::should_stop() {
                return Self::default();
            }
            let id = ProcId(i as u32);
            if !solver.candidate(id) {
                continue;
            }
            if let Some(summary) = solver.summary(id)
                && let Some(relation) = checked::relations(c, &summary)
            {
                out.relations.insert(id, relation);
            }
        }
        if jaic::interrupt::should_stop() {
            Self::default()
        } else {
            out
        }
    }

    /// `None` is unknown and must never be interpreted as an absence of allocation or effects.
    pub fn provenance(&self, id: ProcId) -> Option<&Relations> {
        self.relations.get(&id)
    }
}

#[cfg(test)]
#[path = "allocator_providers/tests.rs"]
mod tests;

#[path = "allocator_providers/bindings.rs"]
mod bindings;
#[path = "allocator_providers/checked.rs"]
mod checked;
#[path = "allocator_providers/domain.rs"]
mod domain;
