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
pub use addresses::LifetimeAddressCast;
pub use controls::LifetimeControls;

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

/// Effects of the memory intrinsic actually selected by lowering, not an opaque callee.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LifetimeMemoryEffects {
    /// Bit masks over runtime formal positions. These operations cannot release allocations.
    pub reads: u8,
    pub writes: u8,
    /// A zero byte count performs no memory access; absent for fixed-width atomics.
    pub count: Option<u8>,
}

#[derive(Clone, Debug)]
pub enum LifetimeEvent {
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
        value: Val,
    },
    ContextRead {
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
    Copy {
        value: Val,
        source: Val,
    },
    Offset {
        value: Val,
        source: Val,
        stride: TypeId,
        offset: Option<i128>,
        span: Span,
    },
    IntegerBinary {
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
        expands: bool,
        results: Vec<(usize, Val, TypeId)>,
    },
    IndirectCall {
        callee: Val,
        ty: TypeId,
        args: Vec<LifetimeArgument>,
        results: Vec<(usize, Val, TypeId)>,
        allocator: bool,
        span: Span,
    },
    Grow {
        region: Val,
        ty: TypeId,
        span: Span,
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

/// A successful real body only. `ProcId` distinguishes polymorphic instantiations;
/// keeping its Arc also lets consumers reject an obsolete retry's record.
#[derive(Clone, Debug)]
pub struct LifetimeBody {
    pub procedure: ProcId,
    pub syntax: Arc<ast::ProcLit>,
    /// Actual context types used while this body was checked, before later cache generations.
    pub context_types: Vec<TypeId>,
    pub blocks: Vec<LifetimeBlock>,
    /// Stack slots selected by lowering, including anonymous argument/literal spills.
    pub stack_places: Vec<Val>,
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
    pub context_types: Vec<TypeId>,
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

impl Compiler {
    /// Freeze only procedure-slot initializer facts, rather than retaining executable data.
    pub(crate) fn lifetime_release_globals(&mut self) {
        use crate::sema::scope::{EntityState, Resolved};
        let Some(facts) = self.ed.facts() else {
            return;
        };
        let ids: HashSet<_> = facts
            .lifetime_bodies
            .values()
            .flat_map(|b| b.blocks.iter().flat_map(|b| &b.events))
            .filter_map(|event| {
                let LifetimeEvent::Expression {
                    ty,
                    entity: Some(entity),
                    ..
                } = event
                else {
                    return None;
                };
                if !matches!(self.ty.types.kind(*ty), TypeKind::Proc(_)) {
                    return None;
                }
                let EntityState::Done(Resolved::Global {
                    storage: ir::Storage::Data(id),
                    ..
                }) = self.entity(*entity).state
                else {
                    return None;
                };
                Some(id)
            })
            .collect();
        let retained = ids
            .into_iter()
            .map(|id| {
                let global = &self.out.program.globals[id.0 as usize];
                (
                    id,
                    (global.init.iter().any(|&b| b != 0), global.relocs.clone()),
                )
            })
            .collect::<Vec<_>>();
        for (id, initial) in retained {
            self.ed
                .facts_mut()
                .unwrap()
                .lifetime_procedure_globals
                .entry(id)
                .or_insert(initial);
        }
    }

    pub(crate) fn lifetime_expression(&self, f: &mut FnCtx, expr: &ast::Expr, op: &Operand) {
        if f.type_only {
            return;
        }
        let Some(record) = &mut f.lifetimes else {
            return;
        };
        if matches!(expr.kind, E::Context) && !record.context_types.contains(&op.ty()) {
            record.context_types.push(op.ty());
        }
        let operand = LifetimeOperand::of(op);
        let entity_span = match &expr.kind {
            E::Member(_, member) => member.span,
            _ => expr.span,
        };
        let entity = self
            .ed
            .facts()
            .and_then(|i| i.last_entity)
            .filter(|(at, _)| *at == entity_span)
            .map(|(_, id)| id);
        let block = f.b.current;
        if matches!(operand, LifetimeOperand::Place(_)) || self.ty.types.is_pointer(op.ty()) {
            record.push(
                block,
                LifetimeEvent::Expression {
                    span: expr.span,
                    ty: op.ty(),
                    operand,
                    entity,
                },
            );
        }
        if let LifetimeOperand::Place(place) = operand {
            if matches!(
                self.ty.types.kind(op.ty()),
                TypeKind::Array {
                    kind: ArrayKind::Resizable,
                    ..
                }
            ) {
                record.resizable.insert(place, op.ty());
            }
            let comparison_reads_storage = f.lifetime_comparing
                && matches!(
                    self.ty.types.kind(op.ty()),
                    TypeKind::String
                        | TypeKind::Array {
                            kind: ArrayKind::View,
                            ..
                        }
                );
            let nonuse = f.lifetime_nonuses.contains(&Some(expr.span)) && !comparison_reads_storage;
            let metadata_only =
                f.lifetime_metadata_base == Some(expr.span) && !self.ty.types.is_pointer(op.ty());
            if matches!(expr.kind, E::Ident(_) | E::Member(..)) && !nonuse && !metadata_only {
                record.push(
                    block,
                    LifetimeEvent::Use {
                        place,
                        ty: op.ty(),
                        span: expr.span,
                    },
                );
            }
        }
        if matches!(expr.kind, E::Asm(_)) {
            record.push(block, LifetimeEvent::Barrier);
        }
    }
}

impl FnCtx {
    pub(crate) fn lifetime_branch(
        &mut self,
        condition: Val,
        span: Span,
        then_block: ir::BlockId,
        else_block: ir::BlockId,
    ) {
        if !self.type_only
            && let Some(record) = &mut self.lifetimes
        {
            record.push(
                self.b.current,
                LifetimeEvent::Branch {
                    condition,
                    span,
                    then_block: then_block.0 as usize,
                    else_block: else_block.0 as usize,
                },
            );
        }
    }

    /// Collection-loop metadata at the body entry (`element`) or its step edge (`None`).
    pub(crate) fn lifetime_iteration(
        &mut self,
        region: (Val, TypeId),
        cursor: (Val, crate::intern::Sym, crate::intern::Sym),
        reverse: bool,
        span: Span,
        element: Option<(Val, TypeId, Val)>,
    ) {
        if self.type_only {
            return;
        }
        let Some(record) = &mut self.lifetimes else {
            return;
        };
        if let Some((result, result_ty, subscript)) = element {
            record.push(
                self.b.current,
                LifetimeEvent::Index {
                    base: region.0,
                    base_ty: region.1,
                    result,
                    result_ty,
                    subscript,
                    index: None,
                },
            );
        }
        record.push(
            self.b.current,
            LifetimeEvent::Iteration {
                region: region.0,
                ty: region.1,
                index: cursor.0,
                name: cursor.1,
                index_name: cursor.2,
                reverse,
                span,
                advance: element.is_none(),
            },
        );
    }


    pub(crate) fn lifetime_store(&mut self, place: Val, value: Val, ty: TypeId, span: Span) {
        if !self.type_only
            && let Some(record) = &mut self.lifetimes
        {
            record.push(
                self.b.current,
                LifetimeEvent::Store {
                    place,
                    value,
                    ty,
                    span,
                    constant: None,
                },
            );
        }
    }

    pub(crate) fn lifetime_view(
        &mut self,
        value: Val,
        source: Val,
        from: TypeId,
        to: TypeId,
        span: Span,
    ) {
        if !self.type_only
            && let Some(record) = &mut self.lifetimes
        {
            record.push(
                self.b.current,
                LifetimeEvent::View {
                    value,
                    source,
                    from,
                    to,
                    span,
                },
            );
        }
    }

    pub(crate) fn lifetime_copy(&mut self, value: Val, source: Val) {
        if value == source {
            return;
        }
        if !self.type_only
            && let Some(record) = &mut self.lifetimes
        {
            record.push(
                self.b.current,
                LifetimeEvent::Copy {
                    value,
                    source,
                },
            );
        }
    }
}

impl LifetimeBody {
    /// Retained vector payload/capacity (including nested argument/expansion vectors).
    /// Excludes shared syntax, `HashMap` buckets and allocator rounding; not process RSS.
    pub fn vector_bytes(&self) -> usize {
        self.context_types.capacity() * std::mem::size_of::<TypeId>()
            + self.source_ranges.capacity() * std::mem::size_of::<LifetimeSourceRange>()
            + self.comparisons.capacity() * std::mem::size_of::<LifetimeComparison>()
            + self.boolean_branches.capacity() * std::mem::size_of::<(Val, Val, bool)>()
            + self.controls.vector_bytes()
            + self.macro_owners.capacity() * std::mem::size_of::<(ScopeId, ScopeId)>()
            + self.globals.capacity() * std::mem::size_of::<(Val, ir::GlobalId)>()
            + self.address_casts.capacity() * std::mem::size_of::<LifetimeAddressCast>()
            + self.slot_aliases.capacity() * std::mem::size_of::<(Val, Val)>()
            + self.stack_places.capacity() * std::mem::size_of::<Val>()
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
                            .map(|event| match event {
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
                                    args.capacity() * std::mem::size_of::<LifetimeArgument>()
                                        + results.capacity()
                                            * std::mem::size_of::<(usize, Val, TypeId)>()
                                }
                                LifetimeEvent::Expansion {
                                    output, ..
                                } => output.capacity() * std::mem::size_of::<Span>(),
                                _ => 0,
                            })
                            .sum::<usize>()
                })
                .sum::<usize>()
    }
}

impl Compiler {
    pub(crate) fn lifetime_begin_body(&mut self, f: &mut FnCtx, id: ProcId) {
        use crate::ide::Recorder;
        if self.ed.facts().is_some_and(|i| i.lint) && !self.ed.sketching() {
            self.ed.facts_mut().unwrap().lifetime_bodies.remove(&id);
            f.lifetimes = Some(Box::new(LifetimeRecording {
                context_types: self.ty.context_type.into_iter().collect(),
                ..Default::default()
            }));
        }
    }

    pub(crate) fn lifetime_finish_body(
        &mut self,
        f: &mut FnCtx,
        id: ProcId,
        syntax: &Arc<ast::ProcLit>,
    ) {
        if let Some(mut record) = f.lifetimes.take() {
            record
                .blocks
                .resize_with(f.b.func.blocks.len(), Default::default);
            let constants: HashMap<_, _> =
                f.b.func
                    .blocks
                    .iter()
                    .flat_map(|b| &b.insts)
                    .filter_map(|inst| match inst {
                        ir::Inst::IConst {
                            dst,
                            value,
                            ..
                        } => Some((*dst, *value)),
                        _ => None,
                    })
                    .collect();
            for (out, block) in record.blocks.iter_mut().zip(&f.b.func.blocks) {
                out.normal_return = matches!(block.term, ir::Term::Ret(_));
                for event in &mut out.events {
                    event.resolve_constants(&constants);
                }
                if let ir::Term::Branch {
                    cond,
                    then_block,
                    else_block,
                } = block.term
                    && let Some(value) = constants.get(&cond)
                {
                    out.successors.push(
                        if *value == 0 {
                            else_block
                        } else {
                            then_block
                        }
                        .0 as usize,
                    );
                } else {
                    block
                        .term
                        .for_each_successor(|b| out.successors.push(b.0 as usize));
                }
            }
            record.prune_cleanup_edges(&f.b.func, &constants);
            let globals: Vec<_> =
                f.b.func
                    .blocks
                    .iter()
                    .flat_map(|b| &b.insts)
                    .filter_map(|inst| {
                        if let ir::Inst::GlobalAddr {
                            dst,
                            global,
                        } = *inst
                        {
                            Some((dst, global))
                        } else {
                            None
                        }
                    })
                    .collect();
            let boolean_branches =
                booleans::immutable_boolean_branches(&f.b.func, &record.blocks, &constants);
            let facts = self.ed.facts_mut().unwrap();
            for &(_, id) in &globals {
                let global = &self.out.program.globals[id.0 as usize];
                if global.read_only && !global.relocs.is_empty() {
                    facts
                        .lifetime_globals
                        .entry(id)
                        .or_insert_with(|| global.relocs.clone());
                }
            }
            let ids = facts
                .lifetime_files
                .entry(syntax.header.span.file)
                .or_default();
            if !ids.contains(&id) {
                ids.push(id);
            }
            facts.lifetime_bodies.insert(
                id,
                crate::ide::lifetimes::LifetimeBody {
                    procedure: id,
                    syntax: syntax.clone(),
                    context_types: record.context_types,
                    macro_owners: record.macro_owners,
                    blocks: record.blocks,
                    source_ranges: record.source_ranges,
                    slot_aliases: slot_aliases(&f.b.func),
                    stack_places: stack_places(&f.b.func, record.stack_subplaces),
                    globals,
                    address_casts: addresses::capture(&f.b.func),
                    boolean_branches,
                    controls: controls::capture(&f.b.func),
                    comparisons: f
                        .b
                        .func
                        .blocks
                        .iter()
                        .flat_map(|b| &b.insts)
                        .filter_map(|inst| {
                            let ir::Inst::Cmp {
                                dst,
                                op,
                                ty,
                                a,
                                b,
                            } = *inst
                            else {
                                return None;
                            };
                            Some(LifetimeComparison {
                                ty,
                                value: dst,
                                left: a,
                                right: b,
                                op,
                                left_constant: constants.get(&a).copied(),
                                right_constant: constants.get(&b).copied(),
                            })
                        })
                        .collect(),
                },
            );
        }
    }
}


impl LifetimeRecording {
    /// Propagate activation flags and unaliased byte locals through cleanup branches.
    /// A slot whose address escapes, is aliased, or is accessed at another width is excluded.
    fn prune_cleanup_edges(&mut self, func: &ir::Func, constants: &HashMap<Val, u64>) {
        if !func
            .blocks
            .iter()
            .any(|b| matches!(b.term, ir::Term::Branch { .. }))
        {
            return;
        }
        let mut flags: Vec<_> = self.cleanup_flags.iter().copied().collect();
        if !self.bool_slots.is_empty() {
            flags.extend(
                private_byte_flags(func)
                    .into_iter()
                    .filter(|v| self.bool_slots.contains(v)),
            );
        }
        flags.sort_unstable_by_key(|v| v.0);
        flags.dedup();
        if flags.is_empty() {
            return;
        }
        let slots: HashMap<_, _> = flags.into_iter().enumerate().map(|(i, v)| (v, i)).collect();
        let mut states = vec![None; self.blocks.len()];
        states[0] = Some(vec![1u8; slots.len()]);
        let mut queue = std::collections::VecDeque::from([0]);
        while let Some(i) = queue.pop_front() {
            let mut state = states[i].clone().unwrap();
            let loads = cleanup_loads(&func.blocks[i].insts, &mut state, &slots, constants);
            let successors = match func.blocks[i].term {
                ir::Term::Branch {
                    cond,
                    then_block,
                    else_block,
                } => match loads.get(&cond) {
                    Some(2) => vec![else_block.0 as usize],
                    Some(4) => vec![then_block.0 as usize],
                    _ => self.blocks[i].successors.clone(),
                },
                _ => self.blocks[i].successors.clone(),
            };
            for next in successors {
                let changed = if let Some(previous) = &mut states[next] {
                    let mut changed = false;
                    for (old, &new) in previous.iter_mut().zip(&state) {
                        let joined = *old | new;
                        changed |= joined != *old;
                        *old = joined;
                    }
                    changed
                } else {
                    states[next] = Some(state.clone());
                    true
                };
                if changed {
                    queue.push_back(next);
                }
            }
        }
        for (i, state) in states.into_iter().enumerate() {
            let Some(mut state) = state else {
                continue;
            };
            let loads = cleanup_loads(&func.blocks[i].insts, &mut state, &slots, constants);
            if let ir::Term::Branch {
                cond,
                then_block,
                else_block,
            } = func.blocks[i].term
            {
                match loads.get(&cond) {
                    Some(2) => self.blocks[i].successors = vec![else_block.0 as usize],
                    Some(4) => self.blocks[i].successors = vec![then_block.0 as usize],
                    _ => {}
                }
            }
        }
    }
}


impl FnCtx {
    pub(crate) fn lifetime_scope_end(&mut self, scope: crate::sema::ScopeId, span: Span) {
        if !self.type_only
            && let Some(record) = &mut self.lifetimes
        {
            record.push(
                self.b.current,
                LifetimeEvent::ScopeEnd {
                    scope,
                    span,
                },
            );
        }
    }
}

// Only direct, uniquely addressed one-byte slots can be propagated without alias analysis.
fn private_byte_flags(func: &ir::Func) -> Vec<Val> {
    let mut addresses = HashMap::<ir::SlotId, Vec<Val>>::default();
    for inst in func.blocks.iter().flat_map(|b| &b.insts) {
        if let ir::Inst::SlotAddr {
            dst,
            slot,
        } = inst
            && func.slots[slot.0 as usize].size == 1
        {
            addresses.entry(*slot).or_default().push(*dst);
        }
    }
    let mut flags: HashSet<_> = addresses
        .values()
        .filter_map(|v| match v.as_slice() {
            [v] => Some(*v),
            _ => None,
        })
        .collect();
    for block in &func.blocks {
        for inst in &block.insts {
            match inst {
                ir::Inst::Load {
                    ty: ir::Ty::I8, ..
                } => {}
                ir::Inst::Store {
                    ty: ir::Ty::I8,
                    value,
                    ..
                } => {
                    flags.remove(value);
                }
                _ => ir::inst_vals(inst, &mut |_| {}, &mut |v| {
                    flags.remove(&v);
                }),
            }
        }
        ir::term_uses(&block.term, &mut |v| {
            flags.remove(&v);
        });
    }
    flags.into_iter().collect()
}

fn cleanup_loads(
    insts: &[ir::Inst],
    state: &mut [u8],
    slots: &HashMap<Val, usize>,
    constants: &HashMap<Val, u64>,
) -> HashMap<Val, u8> {
    let mut loads = HashMap::default();
    for inst in insts {
        match inst {
            ir::Inst::Store {
                addr,
                value,
                ..
            } => {
                if let Some(&slot) = slots.get(addr) {
                    state[slot] = constants
                        .get(value)
                        .map_or(1, |v| 2 << usize::from(*v != 0));
                }
            }
            ir::Inst::Load {
                addr,
                dst,
                ..
            } => {
                if let Some(&slot) = slots.get(addr) {
                    loads.insert(*dst, state[slot]);
                }
            }
            ir::Inst::Bin {
                dst,
                op: ir::BinOp::Xor,
                ty: ir::Ty::I8,
                a,
                b,
            } => {
                let input = if constants.get(a) == Some(&1) {
                    loads.get(b)
                } else if constants.get(b) == Some(&1) {
                    loads.get(a)
                } else {
                    None
                };
                if let Some(&v) = input {
                    loads.insert(*dst, (v & 1) | ((v & 2) << 1) | ((v & 4) >> 1));
                }
            }
            ir::Inst::Cmp {
                dst,
                op: ir::CmpOp::Eq | ir::CmpOp::Ne,
                a,
                b,
                ..
            } => {
                let input = if constants.get(a) == Some(&0) {
                    loads.get(b)
                } else if constants.get(b) == Some(&0) {
                    loads.get(a)
                } else {
                    None
                };
                if let Some(&value) = input {
                    let inverted = matches!(
                        inst,
                        ir::Inst::Cmp {
                            op: ir::CmpOp::Eq,
                            ..
                        }
                    );
                    loads.insert(
                        *dst,
                        if inverted {
                            (value & 1) | ((value & 2) << 1) | ((value & 4) >> 1)
                        } else {
                            value
                        },
                    );
                }
            }
            _ => {}
        }
    }
    loads
}


impl FnCtx {
    pub(crate) fn lifetime_enter_site(&mut self, site: Option<Span>) -> Option<Span> {
        let record = self.lifetimes.as_mut()?;
        let previous = record.source_site;
        record.source_site = previous.or(site);
        previous
    }

    pub(crate) fn lifetime_restore_site(&mut self, site: Option<Span>) {
        if let Some(record) = &mut self.lifetimes {
            record.source_site = site;
        }
    }
}

/// Anonymous literal/argument stores use raw address arithmetic instead of member events.
/// Only addresses derived from this activation's allocations qualify, never loaded pointers.
fn stack_places(func: &ir::Func, subplaces: Vec<Val>) -> Vec<Val> {
    let mut stack: HashSet<Val> = subplaces.into_iter().collect();
    let mut derived: HashMap<Val, Vec<Val>> = HashMap::default();
    for instruction in func.blocks.iter().flat_map(|b| &b.insts) {
        match *instruction {
            ir::Inst::SlotAddr {
                dst, ..
            }
            | ir::Inst::StackAlloc {
                dst, ..
            } => {
                stack.insert(dst);
            }
            ir::Inst::PtrAdd {
                dst,
                base,
                ..
            } => derived.entry(base).or_default().push(dst),
            _ => {}
        }
    }
    let mut pending: Vec<_> = stack.iter().copied().collect();
    while let Some(base) = pending.pop() {
        for &dst in derived.get(&base).into_iter().flatten() {
            if stack.insert(dst) {
                pending.push(dst);
            }
        }
    }
    let mut stack: Vec<_> = stack.into_iter().collect();
    stack.sort_unstable_by_key(|v| v.0);
    stack
}

fn slot_aliases(func: &ir::Func) -> Vec<(Val, Val)> {
    let mut slots = HashMap::default();
    let mut aliases = Vec::new();
    for inst in func.blocks.iter().flat_map(|b| &b.insts) {
        if let ir::Inst::SlotAddr {
            dst,
            slot,
        } = *inst
        {
            let canonical = *slots.entry(slot).or_insert(dst);
            if dst != canonical {
                aliases.push((dst, canonical));
            }
        }
    }
    aliases
}
