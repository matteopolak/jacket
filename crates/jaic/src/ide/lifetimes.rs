//! Optional semantic lifetime sidecar. Events are emitted in lowering order, in the
//! checker's actual blocks. No AST control flow is reconstructed by consumers.
use crate::ast::{self, ExprKind as E};
use crate::fxhash::HashMap;
use crate::ir::{self, Val};
use crate::sema::lower::{FnCtx, Operand};
use crate::sema::{Compiler, EntityId, ProcId};
use crate::source::Span;
use crate::types::{ArrayKind, TypeId, TypeKind};
use std::sync::Arc;

#[derive(Clone, Copy, Debug)]
pub enum LifetimeOperand {
    Place(Val),
    Value(Val),
    Other,
}

impl LifetimeOperand {
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

#[derive(Clone, Debug)]
pub enum LifetimeEvent {
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
    Store {
        place: Val,
        value: Val,
        ty: TypeId,
        span: Span,
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
    Index {
        base: Val,
        result: Val,
        base_ty: TypeId,
        result_ty: TypeId,
        index: Option<i128>,
    },
    View {
        value: Val,
        source: Val,
        from: TypeId,
        to: TypeId,
        span: Span,
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
        value: Val,
        index: usize,
    },
    Call {
        callee: ProcId,
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

/// A successful real body only. `ProcId` distinguishes polymorphic instantiations;
/// keeping its Arc also lets consumers reject an obsolete retry's record.
#[derive(Clone, Debug)]
pub struct LifetimeBody {
    pub procedure: ProcId,
    pub syntax: Arc<ast::ProcLit>,
    pub blocks: Vec<LifetimeBlock>,
    /// Stack slots selected by lowering, including anonymous argument/literal spills.
    pub stack_places: Vec<Val>,
}

/// Mutable only while this `FnCtx` is being checked; discarded on error.
#[derive(Default)]
pub(crate) struct LifetimeRecording {
    pub blocks: Vec<LifetimeBlock>,
    pub resizable: HashMap<Val, TypeId>,
    pub elements: HashMap<Val, (Val, TypeId)>,
    pub calls: Vec<LifetimeCallRecording>,
}

pub(crate) struct LifetimeCallRecording {
    pub args: Vec<LifetimeArgument>,
    pub parameter_spans: Vec<Span>,
}

impl LifetimeRecording {
    pub fn push(&mut self, block: ir::BlockId, event: LifetimeEvent) {
        self.blocks.resize_with(
            self.blocks.len().max(block.0 as usize + 1),
            LifetimeBlock::default,
        );
        self.blocks[block.0 as usize].events.push(event);
    }
}

impl Compiler {
    pub(crate) fn lifetime_expression(&self, f: &mut FnCtx, expr: &ast::Expr, op: &Operand) {
        if f.type_only {
            return;
        }
        let Some(record) = &mut f.lifetimes else {
            return;
        };
        let operand = LifetimeOperand::of(op);
        let entity = self
            .ed
            .facts()
            .and_then(|i| i.last_entity)
            .filter(|(at, _)| *at == expr.span)
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
        self.stack_places.capacity() * std::mem::size_of::<Val>()
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
            f.lifetimes = Some(Box::default());
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
            let facts = self.ed.facts_mut().unwrap();
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
                    blocks: record.blocks,
                    stack_places: f
                        .b
                        .func
                        .blocks
                        .iter()
                        .flat_map(|b| &b.insts)
                        .filter_map(|inst| match inst {
                            ir::Inst::SlotAddr {
                                dst, ..
                            }
                            | ir::Inst::StackAlloc {
                                dst, ..
                            } => Some(*dst),
                            _ => None,
                        })
                        .collect(),
                },
            );
        }
    }
}
