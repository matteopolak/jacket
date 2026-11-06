//! Optional semantic lifetime sidecar. Events are emitted in lowering order, in the
//! checker's actual blocks. No AST control flow is reconstructed by consumers.
use crate::ast::{self, ExprKind as E};
use crate::fxhash::{HashMap, HashSet};
use crate::ir::{self, Val};
use crate::sema::lower::{FnCtx, Operand};
use crate::sema::{Compiler, EntityId, ProcId, ScopeId};
use crate::source::Span;
use crate::types::{ArrayKind, TypeId, TypeKind};
use std::sync::Arc;

mod addresses;
mod booleans;
mod controls;
mod generated;
mod recording;
pub use addresses::LifetimeAddressCast;
pub use controls::LifetimeControls;
pub use generated::{LifetimeContextTransfer, LifetimeGeneratedEntryCall};

#[derive(Clone, Copy, Debug)]
pub enum LifetimeOperand {
    Place(Val),
    Value(Val),
    Other,
}

impl LifetimeOperand {
    /// A checked call result, including callable metadata around a procedure value.
    pub(crate) fn result(op: &Operand) -> Option<(Val, TypeId)> {
        match op {
            Operand::Callable {
                operand, ..
            } => Self::result(operand),
            Operand::Value {
                val,
                ty,
            } => Some((*val, *ty)),
            _ => None,
        }
    }

    pub(crate) fn of(op: &Operand) -> Self {
        match op {
            Operand::Callable {
                operand, ..
            } => Self::of(operand),
            Operand::Place {
                addr, ..
            } => Self::Place(*addr),
            Operand::Value {
                val, ..
            } => Self::Value(*val),
            _ => Self::Other,
        }
    }
}

/// Selected argument mapping, before named/spread/variadic syntax is erased.
#[derive(Clone, Debug)]
pub struct LifetimeArgument {
    pub formal: usize,
    pub ty: TypeId,
    pub constant: Option<u64>,
    pub span: Span,
    pub operand: LifetimeOperand,
    pub named: bool,
    pub spread: bool,
}

/// Byte correspondence of the memory primitive actually selected by lowering.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LifetimeMemoryCopy {
    pub destination_formal: u8,
    pub source_formal: u8,
    pub count_formal: u8,
    pub requires_disjoint: bool,
}

/// A selected write-only primitive touches only this destination byte range on
/// a valid invocation and performs no hidden callbacks or resource operations.
/// Consumers must validate count, extent and no-wrap, and apply write interference.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LifetimeBoundedWrite {
    pub destination_formal: u8,
    pub count_formal: u8,
}

/// Mutually exclusive selected-operation contracts, separate from access masks.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LifetimeMemoryOperation {
    Unspecified,
    Copy(LifetimeMemoryCopy),
    BoundedWrite(LifetimeBoundedWrite),
}

/// Effects of the memory intrinsic actually selected by lowering, not an opaque callee.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LifetimeMemoryEffects {
    /// Memory-access masks over runtime formal positions. Resource-protocol
    /// effects are separate; these masks are not a general purity proof.
    pub reads: u8,
    pub writes: u8,
    /// A zero byte count performs no memory access; absent for fixed-width atomics.
    pub count: Option<u8>,
    /// Precise operation semantics cannot be reconstructed from access masks.
    pub operation: LifetimeMemoryOperation,
}

impl LifetimeMemoryEffects {
    /// The selected copy contract, e.g. `effects.copy().map(|copy| copy.source_formal)`.
    pub const fn copy(&self) -> Option<LifetimeMemoryCopy> {
        match self.operation {
            LifetimeMemoryOperation::Copy(copy) => Some(copy),
            LifetimeMemoryOperation::Unspecified | LifetimeMemoryOperation::BoundedWrite(_) => None,
        }
    }

    /// The selected write-only range, e.g. `effects.bounded_write().map(|write| write.count_formal)`.
    pub const fn bounded_write(&self) -> Option<LifetimeBoundedWrite> {
        match self.operation {
            LifetimeMemoryOperation::BoundedWrite(write) => Some(write),
            LifetimeMemoryOperation::Unspecified | LifetimeMemoryOperation::Copy(_) => None,
        }
    }
}

/// Checked boolean value relations. And/Or retain short-circuit semantics: the
/// right SSA value exists only on its evaluation edge, and must not be demanded
/// when the left operand already determines the result.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LifetimeBoolean {
    Not(Val),
    And(Val, Val),
    Or(Val, Val),
}

#[derive(Clone, Debug)]
pub enum LifetimeEvent {
    /// A selected fully inline member path, using the actual captured base address
    /// and checked layout. Pointer-following `using` paths are not inline proofs.
    FieldProjection {
        base: Val,
        result: Val,
        base_ty: TypeId,
        result_ty: TypeId,
        offset: u64,
    },
    /// A semantic bool operation, independent of its executable lowering shape.
    Boolean {
        value: Val,
        operation: LifetimeBoolean,
    },
    /// Actual lexical interface of an indirect call; never a structurally equal header.
    CallContract {
        callee: Val,
        declaration: Arc<ast::ProcHeader>,
    },
    /// A freshly materialized empty string: its descriptor contains no storage.
    EmptyDescriptor {
        value: Val,
        ty: TypeId,
    },
    /// Runtime context selection, including caller-owned conditional macro defers.
    ContextSet {
        ty: TypeId,
        place: Val,
        value: Val,
        constant: Option<u64>,
    },
    ContextRead {
        ty: TypeId,
        place: Val,
        value: Val,
    },
    ContextAlias {
        value: Val,
        source: Val,
    },
    /// A local/field/view's real per-instantiation identity and checked type.
    Expression {
        span: Span,
        ty: TypeId,
        operand: LifetimeOperand,
        entity: Option<EntityId>,
    },
    Parameter {
        formal: usize,
        place: Val,
        incoming: Val,
        ty: TypeId,
        span: Span,
    },
    Use {
        place: Val,
        ty: TypeId,
        span: Span,
    },
    Address {
        place: Val,
        ty: TypeId,
        span: Span,
    },
    /// Exact supported semantic leaves of the immediately preceding selected default write.
    /// Incomplete coverage never implies that omitted leaves are zero or unchanged.
    DefaultImage {
        place: Val,
        ty: TypeId,
        /// Exact emitted write width, independent of later mutable type layouts.
        bytes: u64,
        /// This exact emitted byte range is zeroed. False is unknown, including
        /// copied images with zero bytes; typed leaf coverage is independent.
        zeroed: bool,
        leaves: Vec<LifetimeDefaultLeaf>,
        complete: bool,
    },
    /// A default image (including zero initialization) replaces this storage now.
    Initialize {
        place: Val,
        ty: TypeId,
        span: Span,
    },
    Store {
        place: Val,
        value: Val,
        ty: TypeId,
        span: Span,
        constant: Option<u64>,
    },
    Load {
        place: Val,
        value: Val,
        ty: TypeId,
        span: Span,
    },
    Borrow {
        value: Val,
        region: Val,
        region_ty: TypeId,
        span: Span,
    },
    /// The source and result have the same semantic value/address. A nonzero
    /// checked member adjustment is a `FieldProjection`, never a `Copy`.
    Copy {
        value: Val,
        source: Val,
    },
    Offset {
        value: Val,
        source: Val,
        stride: TypeId,
        /// Logical element delta, not a byte offset.
        offset: Option<i128>,
        /// Selected element-delta SSA when retained, before stride scaling.
        subscript: Option<Val>,
        span: Span,
    },
    IntegerBinary {
        /// Checked operation width and signedness, before executable IR release.
        ty: TypeId,
        value: Val,
        left: Val,
        right: Val,
        op: ir::BinOp,
        left_constant: Option<u64>,
        right_constant: Option<u64>,
    },
    Index {
        base: Val,
        result: Val,
        base_ty: TypeId,
        result_ty: TypeId,
        index: Option<i128>,
        subscript: Val,
    },
    View {
        value: Val,
        source: Val,
        from: TypeId,
        to: TypeId,
        span: Span,
    },
    /// Selected collection-loop cursor; `advance` is emitted on its actual step edge.
    Iteration {
        region: Val,
        ty: TypeId,
        index: Val,
        reverse: bool,
        name: crate::intern::Sym,
        index_name: crate::intern::Sym,
        span: Span,
        advance: bool,
    },
    ReadMemory {
        value: Val,
        span: Span,
    },
    Indirect {
        place: Val,
        ty: TypeId,
    },
    Return {
        constant: Option<u64>,
        value: Val,
        index: usize,
        ty: TypeId,
        span: Span,
    },
    Local {
        place: Val,
        ty: TypeId,
        scope: crate::sema::ScopeId,
        name: crate::intern::Sym,
        span: Span,
    },
    ScopeEnd {
        scope: crate::sema::ScopeId,
        span: Span,
    },
    BoxAny {
        value: Val,
        place: Val,
        ty: TypeId,
        temporary: bool,
        span: Span,
    },
    StatementEnd {
        span: Span,
    },
    Procedure {
        value: Val,
        callee: ProcId,
        ty: TypeId,
        span: Span,
    },
    Branch {
        condition: Val,
        span: Span,
        then_block: usize,
        else_block: usize,
    },
    Call {
        callee: ProcId,
        memory: Option<LifetimeMemoryEffects>,
        args: Vec<LifetimeArgument>,
        span: Span,
        /// Runtime effects are emitted in the caller; declaration contracts remain independent.
        expands: bool,
        results: Vec<(usize, Val, TypeId)>,
    },
    IndirectCall {
        callee: Val,
        ty: TypeId,
        args: Vec<LifetimeArgument>,
        results: Vec<(usize, Val, TypeId)>,
        allocator: bool,
        /// Sparse boxed metadata avoids enlarging every retained event.
        allocator_protocol: Option<Box<LifetimeAllocatorCall>>,
        span: Span,
    },
    /// Declared growth effect of the exact associated call, not a separate
    /// memory write before argument evaluation. Expanded bodies precede this pair.
    Grow {
        region: Val,
        ty: TypeId,
        span: Span,
        /// Following Call event in this same retained block. Only nonexpanded
        /// calls can use the state before this marker as their invocation prestate.
        call_event: usize,
    },
    /// Unknown memory writes cannot establish a must proof.
    Barrier,
    StaticChoice {
        span: Span,
        taken: bool,
    },
    Expansion {
        site: Span,
        output: Vec<Span>,
    },
    Cleanup {
        exit: Span,
        deferred: Option<Span>,
    },
    Field {
        base: LifetimeOperand,
        result: LifetimeOperand,
        name: crate::intern::Sym,
        indirect: bool,
        descriptor: bool,
        base_ty: TypeId,
        result_ty: TypeId,
        span: Span,
    },
}

impl LifetimeEvent {
    fn resolve_constants(&mut self, constants: &HashMap<Val, u64>) {
        match self {
            Self::Store {
                value,
                constant,
                ..
            }
            | Self::Return {
                value,
                constant,
                ..
            } => *constant = constants.get(value).copied(),
            Self::IntegerBinary {
                left,
                right,
                left_constant,
                right_constant,
                ..
            } => {
                *left_constant = constants.get(left).copied();
                *right_constant = constants.get(right).copied();
            }
            _ => {}
        }
        let (Self::Call {
            args, ..
        }
        | Self::IndirectCall {
            args, ..
        }) = self
        else {
            return;
        };
        for arg in args {
            arg.constant = match arg.operand {
                LifetimeOperand::Value(value) => constants.get(&value).copied(),
                _ => None,
            };
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct LifetimeBlock {
    pub events: Vec<LifetimeEvent>,
    pub successors: Vec<usize>,
    pub normal_return: bool,
}

/// A contiguous run emitted while expanding code at a caller's source location.
#[derive(Clone, Debug)]
pub struct LifetimeSourceRange {
    pub block: usize,
    pub first_event: usize,
    pub end_event: usize,
    pub site: Span,
}

/// A selected primitive comparison, retained after executable IR is released.
#[derive(Clone, Debug)]
pub struct LifetimeComparison {
    /// Primitive operand type, including the distinction between pointers and integer handles.
    pub ty: ir::Ty,
    pub value: Val,
    pub left: Val,
    pub right: Val,
    pub op: ir::CmpOp,
    pub left_constant: Option<u64>,
    pub right_constant: Option<u64>,
}

/// An actual stack allocation's canonical address and original byte extent.
/// Derived addresses are not roots. Unknown extents authorize no private access.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LifetimeStackRoot {
    pub value: Val,
    pub bytes: Option<u64>,
}

/// One typed semantic value written by a selected default initializer.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LifetimeDefaultLeaf {
    pub offset: u64,
    pub width: u64,
    pub ty: TypeId,
    pub value: LifetimeDefaultValue,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum LifetimeDefaultValue {
    Scalar(u64),
    Procedure {
        procedure: ProcId,
        function: ir::FuncId,
    },
    Unknown,
}

/// The exact conservative barrier introduced by a selected inline overlay access.
/// This is selection metadata, not a guarantee about surrounding expressions.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LifetimeOverlayAccess {
    pub block: usize,
    pub event: usize,
    pub projection_block: usize,
    pub projection_event: usize,
    pub descriptor_ty: TypeId,
    pub result: Val,
}

/// Checked roles and nominal operation tags of the selected allocator ABI.
/// This certifies an invocation protocol, never freshness or provider behavior.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LifetimeAllocatorCall {
    pub operation_formal: usize,
    pub size_formal: usize,
    pub old_size_formal: usize,
    pub old_memory_formal: usize,
    pub data_formal: usize,
    pub operation_ty: TypeId,
    pub allocate_value: u64,
    pub resize_value: u64,
    pub free_value: u64,
    pub result_position: usize,
}

/// A successful real body only. `ProcId` distinguishes polymorphic instantiations;
/// keeping its Arc also lets consumers reject an obsolete retry's record.
/// Exact immutable bytes selected for a memory-valued aggregate constant.
/// A copy of `Item.{}` may read this full typed region. This grants no writable
/// storage, pointee validity, allocation ownership or disjointness certificate.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LifetimeReadOnlyRegion {
    pub value: Val,
    pub ty: TypeId,
    pub bytes: u64,
    /// Selected constant-image values only; runtime-immutable globals whose
    /// compiler initialization is not final retain no value certificate.
    pub leaves: Vec<LifetimeDefaultLeaf>,
    pub complete: bool,
}

/// The actual operand passed in a selected call's hidden context ABI position.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LifetimeContextArgument {
    Absent,
    Value(Val),
    /// The indirect-call lowering itself emitted this exact Ptr zero.
    Null(Val),
}

/// A real call site only; absence of a row means unavailable, not absent ABI.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LifetimeCallContext {
    pub block: usize,
    pub event: usize,
    pub argument: LifetimeContextArgument,
}

#[derive(Clone, Debug)]
pub struct LifetimeBody {
    pub procedure: ProcId,
    pub syntax: Arc<ast::ProcLit>,
    /// Actual context types used while this body was checked, before later cache generations.
    pub context_types: Vec<TypeId>,
    /// Actual selected hidden ABI parameter; symbolic until an invocation binds it.
    pub context_input: Option<Val>,
    pub call_contexts: Vec<LifetimeCallContext>,
    pub blocks: Vec<LifetimeBlock>,
    /// Stack ancestry for attribution, including derived addresses. Not ownership.
    pub stack_places: Vec<Val>,
    /// Actual allocation roots only; aliases share one original byte extent.
    pub stack_roots: Vec<LifetimeStackRoot>,
    pub read_only_regions: Vec<LifetimeReadOnlyRegion>,
    pub overlay_accesses: Vec<LifetimeOverlayAccess>,
    /// Distinct SSA addresses of the same physical stack slot.
    pub slot_aliases: Vec<(Val, Val)>,
    pub source_ranges: Vec<LifetimeSourceRange>,
    /// Selected primitive comparisons used by the recorded control flow.
    pub comparisons: Vec<LifetimeComparison>,
    /// Branch predicates on private byte slots written exactly once.
    pub boolean_branches: Vec<(Val, Val, bool)>,
    pub controls: LifetimeControls,
    pub macro_owners: Vec<(ScopeId, ScopeId)>,
    /// Pointer/integer bitcasts, retained independently of executable instructions.
    pub address_casts: Vec<LifetimeAddressCast>,
    /// Global addresses used by this body; immutable relocations live in `lifetime_globals`.
    pub globals: Vec<(Val, ir::GlobalId)>,
}

/// Mutable only while this `FnCtx` is being checked; discarded on error.
#[derive(Default)]
pub(crate) struct LifetimeRecording {
    pub read_only_regions: Vec<LifetimeReadOnlyRegion>,
    pub overlay_accesses: Vec<LifetimeOverlayAccess>,
    pub context_types: Vec<TypeId>,
    /// Actual selected hidden ABI parameter; symbolic until an invocation binds it.
    pub context_input: Option<Val>,
    pub call_contexts: Vec<LifetimeCallContext>,
    pub macro_owners: Vec<(ScopeId, ScopeId)>,
    pub blocks: Vec<LifetimeBlock>,
    pub resizable: HashMap<Val, TypeId>,
    pub elements: HashMap<Val, (Val, TypeId)>,
    pub calls: Vec<LifetimeCallRecording>,
    pub cleanup_flags: HashSet<Val>,
    bool_slots: HashSet<Val>,
    pub stack_subplaces: Vec<Val>,
    pub source_site: Option<Span>,
    pub source_ranges: Vec<LifetimeSourceRange>,
    pub last_source_range: HashMap<usize, usize>,
}

pub(crate) struct LifetimeCallRecording {
    /// Runtime effects of this selected call were emitted and recorded in its caller.
    pub effects_recorded: bool,
    pub context: Option<LifetimeContextArgument>,
    pub args: Vec<LifetimeArgument>,
    pub memory: Option<LifetimeMemoryEffects>,
    pub parameter_spans: Vec<Span>,
}

impl LifetimeRecording {
    pub fn push(&mut self, block: ir::BlockId, event: LifetimeEvent) {
        if let LifetimeEvent::Local {
            place,
            ty: TypeId::BOOL,
            ..
        } = event
        {
            self.bool_slots.insert(place);
        }

        self.blocks.resize_with(
            self.blocks.len().max(block.0 as usize + 1),
            LifetimeBlock::default,
        );
        let block = block.0 as usize;
        let ordinal = self.blocks[block].events.len();
        if let Some(site) = self.source_site {
            let last = self.last_source_range.get(&block).copied().filter(|&i| {
                self.source_ranges[i].site == site && self.source_ranges[i].end_event == ordinal
            });
            if let Some(last) = last {
                self.source_ranges[last].end_event += 1;
            } else {
                let i = self.source_ranges.len();
                self.source_ranges.push(LifetimeSourceRange {
                    block,
                    first_event: ordinal,
                    end_event: ordinal + 1,
                    site,
                });
                self.last_source_range.insert(block, i);
            }
        }
        self.blocks[block].events.push(event);
    }
}

impl LifetimeBody {
    /// Retained vector and owned protocol payload/capacity (including nested vectors).
    /// Excludes shared syntax, `HashMap` buckets and allocator rounding; not process RSS.
    pub fn vector_bytes(&self) -> usize {
        self.context_types.capacity() * std::mem::size_of::<TypeId>()
            + self.call_contexts.capacity() * std::mem::size_of::<LifetimeCallContext>()
            + self.source_ranges.capacity() * std::mem::size_of::<LifetimeSourceRange>()
            + self.comparisons.capacity() * std::mem::size_of::<LifetimeComparison>()
            + self.boolean_branches.capacity() * std::mem::size_of::<(Val, Val, bool)>()
            + self.controls.vector_bytes()
            + self.macro_owners.capacity() * std::mem::size_of::<(ScopeId, ScopeId)>()
            + self.globals.capacity() * std::mem::size_of::<(Val, ir::GlobalId)>()
            + self.address_casts.capacity() * std::mem::size_of::<LifetimeAddressCast>()
            + self.slot_aliases.capacity() * std::mem::size_of::<(Val, Val)>()
            + self.stack_places.capacity() * std::mem::size_of::<Val>()
            + self.stack_roots.capacity() * std::mem::size_of::<LifetimeStackRoot>()
            + self.read_only_regions.capacity() * std::mem::size_of::<LifetimeReadOnlyRegion>()
            + self
                .read_only_regions
                .iter()
                .map(|region| region.leaves.capacity() * std::mem::size_of::<LifetimeDefaultLeaf>())
                .sum::<usize>()
            + self.overlay_accesses.capacity() * std::mem::size_of::<LifetimeOverlayAccess>()
            + self.blocks.capacity() * std::mem::size_of::<LifetimeBlock>()
            + self
                .blocks
                .iter()
                .map(|block| {
                    block.events.capacity() * std::mem::size_of::<LifetimeEvent>()
                        + block.successors.capacity() * std::mem::size_of::<usize>()
                        + block
                            .events
                            .iter()
                            .map(|event| {
                                let protocol = match event {
                                    LifetimeEvent::IndirectCall {
                                        allocator_protocol: Some(_),
                                        ..
                                    } => std::mem::size_of::<LifetimeAllocatorCall>(),
                                    _ => 0,
                                };
                                protocol
                                    + match event {
                                        LifetimeEvent::Call {
                                            args,
                                            results,
                                            ..
                                        }
                                        | LifetimeEvent::IndirectCall {
                                            args,
                                            results,
                                            ..
                                        } => {
                                            args.capacity()
                                                * std::mem::size_of::<LifetimeArgument>()
                                                + results.capacity()
                                                    * std::mem::size_of::<(usize, Val, TypeId)>()
                                        }
                                        LifetimeEvent::Expansion {
                                            output, ..
                                        } => output.capacity() * std::mem::size_of::<Span>(),
                                        LifetimeEvent::DefaultImage {
                                            leaves, ..
                                        } => {
                                            leaves.capacity()
                                                * std::mem::size_of::<LifetimeDefaultLeaf>()
                                        }
                                        _ => 0,
                                    }
                            })
                            .sum::<usize>()
                })
                .sum::<usize>()
    }
}
