//! Conditional bump-storage leases and mark observations, distinct from native frees.
use super::allocator_providers::{ByteSlot, Captured, Site};
use jaic::fxhash::HashMap;
use jaic::ir::{CmpOp, Ty};
use jaic::sema::{Compiler, ProcId};
use jaic::types::TypeId;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(in crate::rules) struct Instance {
    pub pointer: Captured,
    pub view_ty: TypeId,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(in crate::rules) struct ProtocolKey {
    pub procedure: ProcId,
    pub data: ByteSlot,
    pub cursor: ByteSlot,
    pub capacity: Option<ByteSlot>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::rules) struct FieldCapture {
    pub result_slot: ByteSlot,
    pub source_slot: ByteSlot,
    pub read_site: Site,
}

/// A copied tuple is an observation; it is not independently a valid mark.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::rules) struct SnapshotObservation {
    pub site: Site,
    pub result: usize,
    pub result_ty: TypeId,
    pub instance: Option<Instance>,
    pub fields: Vec<FieldCapture>,
    pub conditions: Vec<Condition>,
}

/// Caller ghost inputs, never integer slots or structural runtime identities.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::rules) struct ValidityRequirement {
    pub instance: Instance,
    pub reset_generation: GenerationInput,
    pub lineage: LineageInput,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::rules) enum GenerationInput {
    CallerCurrent,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::rules) enum LineageInput {
    CallerCurrentFrontier,
    CallerReachableMark,
}

/// Predicates refer to captured operands, never a later field reread.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::rules) enum Predicate {
    Constant(bool),
    Compare {
        left: Operand,
        right: Operand,
        op: CmpOp,
        ty: Ty,
    },
    Not(Box<Predicate>),
    And(Box<Predicate>, Box<Predicate>),
    Or(Box<Predicate>, Box<Predicate>),
    Unknown,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::rules) struct Condition {
    pub site: Site,
    pub predicate: Predicate,
    pub truth: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::rules) struct SnapshotTemplate {
    pub observation: SnapshotObservation,
    pub protocol: ProtocolKey,
    pub validity: ValidityRequirement,
    pub obligations: Vec<Obligation>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::rules) enum Operand {
    Captured(Captured),
    Constant {
        ty: TypeId,
        bits: u64,
    },
    Add {
        left: Box<Operand>,
        right: Box<Operand>,
        ty: TypeId,
    },
    Sub {
        left: Box<Operand>,
        right: Box<Operand>,
        ty: TypeId,
    },
    Unknown,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::rules) struct AllocationRange {
    pub base: Captured,
    pub cursor: Captured,
    pub length: Operand,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::rules) enum Obligation {
    ExistingMarkOrigin {
        formal: usize,
    },
    CallerArgumentSnapshot {
        formal: usize,
        ty: TypeId,
    },
    /// Resolve the ghost source's actual full state extent, never the Mark type size.
    ForwardedOutputHeaderSeparation {
        result: usize,
        source_formal: usize,
    },
    ReadableWritableState {
        instance: Instance,
        bytes: u64,
    },
    ActiveProtocol {
        instance: Instance,
        protocol: ProtocolKey,
    },
    NonNegativeCursorAndLength {
        cursor: Captured,
        length: Operand,
    },
    OutputHeaderSeparation {
        result: usize,
        instance: Instance,
        state: ByteSlot,
    },
    CheckedArithmetic {
        expressions: Vec<Operand>,
    },
    /// Capacity alone does not establish the actual backing allocation extent.
    ReadableWritableBacking {
        instance: Instance,
        range: AllocationRange,
    },
    NonNullBacking {
        base: Captured,
    },
    CheckedAlignmentAndAddition {
        range: AllocationRange,
    },
    HeaderDataSeparation {
        instance: Instance,
        state: ByteSlot,
        range: AllocationRange,
    },
    SameInstanceAndValidLineage {
        mark_formal: usize,
        validity: ValidityRequirement,
    },
    FrontierBounds {
        instance: Instance,
        cursor: ByteSlot,
        mark_slot: ByteSlot,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::rules) struct ContiguousBump {
    pub site: Site,
    pub instance: Instance,
    pub protocol: ProtocolKey,
    pub result: usize,
    pub range: AllocationRange,
    pub written: Vec<ByteSlot>,
    pub obligations: Vec<Obligation>,
    pub conditions: Vec<Condition>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::rules) struct MarkInput {
    pub formal: usize,
    pub ty: TypeId,
    pub cursor_slot: ByteSlot,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::rules) struct RewindTemplate {
    pub site: Site,
    pub instance: Instance,
    pub protocol: ProtocolKey,
    pub mark: MarkInput,
    pub obligations: Vec<Obligation>,
    /// Descendant marks and post-cut allocation identities must be retired.
    pub discard_descendants: bool,
    pub conditions: Vec<Condition>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::rules) struct ResetTemplate {
    pub site: Site,
    pub instance: Instance,
    pub protocol: ProtocolKey,
    /// Full reset advances its reset generation; this is not physical deallocation.
    pub obligations: Vec<Obligation>,
    pub retire_all_prior_leases: bool,
    pub conditions: Vec<Condition>,
}

/// Transport an existing argument ghost, including its original validity state.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::rules) struct ForwardedMark {
    pub result: usize,
    pub result_ty: TypeId,
    pub source_formal: usize,
    pub source_ty: TypeId,
    pub entry_copy_site: Site,
    /// Last certified whole copy feeding the result, never a new mark occurrence.
    pub return_copy_site: Site,
    pub fields: Vec<ByteSlot>,
    pub conditions: Vec<Condition>,
    pub obligations: Vec<Obligation>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::rules) enum LoopHypothesis {
    ReceiverAddressSnapshot {
        formal: usize,
        ty: TypeId,
    },
    ReadableWritableReceiver {
        formal: usize,
        ty: TypeId,
        bytes: u64,
    },
    EntryPointerZero {
        formal: usize,
        slot: ByteSlot,
    },
    NullArgumentSnapshot {
        formal: usize,
        ty: TypeId,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::rules) struct ControlEdge {
    pub from: usize,
    pub to: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::rules) struct HeaderWrite {
    pub site: Site,
    pub slot: ByteSlot,
    pub value: Operand,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::rules) struct HeaderWriteCase {
    pub conditions: Vec<Condition>,
    pub writes: Vec<HeaderWrite>,
}

/// A checked loop bypass under explicit zero-input hypotheses, not a cleanup effect.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::rules) struct LoopBypass {
    pub procedure: ProcId,
    pub receiver_formal: usize,
    pub receiver_ty: TypeId,
    pub head_slot: ByteSlot,
    pub stop_formal: usize,
    pub guard: Site,
    pub component: Vec<usize>,
    pub excluded_body_edges: Vec<ControlEdge>,
    pub hypotheses: Vec<LoopHypothesis>,
    pub paths: Vec<HeaderWriteCase>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(in crate::rules) struct Relations {
    pub loop_bypasses: Vec<LoopBypass>,
    pub forwards: Vec<ForwardedMark>,
    pub protocols: Vec<ContiguousBump>,
    pub observations: Vec<SnapshotObservation>,
    pub snapshots: Vec<SnapshotTemplate>,
    pub rewinds: Vec<RewindTemplate>,
    pub resets: Vec<ResetTemplate>,
    pub incomplete: bool,
}

#[derive(Default)]
pub(in crate::rules) struct Analysis {
    relations: HashMap<ProcId, Relations>,
}

impl Analysis {
    /// Build run-local conditional relations, for example `analysis.relations(id)`.
    pub fn build(compiler: &Compiler) -> Self {
        checked::build(compiler)
    }

    /// Absence is unknown, never an absence of writes, retirement or allocation.
    pub fn relations(&self, procedure: ProcId) -> Option<&Relations> {
        self.relations.get(&procedure)
    }
}

#[cfg(test)]
#[path = "temporary_epochs/tests.rs"]
mod tests;

#[path = "temporary_epochs/checked.rs"]
mod checked;
#[path = "temporary_epochs/domain.rs"]
mod domain;

#[path = "temporary_epochs/paged.rs"]
mod paged;

#[path = "temporary_epochs/entry.rs"]
mod entry;
#[path = "temporary_epochs/transfer.rs"]
mod transfer;
