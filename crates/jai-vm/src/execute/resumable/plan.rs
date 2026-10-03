//! Owned checked code for the continuation machine. Execution addresses private IDs.
mod control;
mod expressions;

use crate::{Error, LimitKind, Limits};
pub(super) use expressions::{ApplyOp, BoolApply, FloatApply, IntApply, PlaceOp};
use jai_ir::*;
use jai_types::{Direction, FieldId, IntegerType, TypeId, TypeView};
use std::{collections::HashMap, sync::Arc};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(super) struct NodeId(usize);
impl NodeId {
    pub(super) fn index(self) -> usize {
        self.0
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(super) struct BlockId(usize);
impl BlockId {
    pub(super) fn index(self) -> usize {
        self.0
    }
}

#[derive(Debug)]
pub(super) struct Node {
    pub(super) ty: Option<TypeId>,
    pub(super) kind: NodeKind,
}
#[derive(Debug)]
pub(super) enum NodeKind {
    Bind {
        bindings: Box<[(ExpressionBindingId, NodeId)]>,
        body: NodeId,
    },
    Apply {
        op: ApplyOp,
        operands: Box<[NodeId]>,
    },
    Place {
        op: PlaceOp,
        operands: Box<[NodeId]>,
    },
    Conditional {
        condition: NodeId,
        then_node: NodeId,
        else_node: NodeId,
    },
    ShortCircuit {
        op: ShortCircuitOp,
        left: NodeId,
        right: NodeId,
    },
    Call {
        target: CallTarget,
        signature: TypeId,
        arguments: Box<[(ParameterId, NodeId)]>,
    },
    /// The machine captures the base descriptor before scheduling the index.
    IndexPlace {
        base: NodeId,
        index: NodeId,
        base_type: TypeId,
        check: CheckMode,
    },
    /// Zero/default fields are installed before the first initializer executes.
    RecordBuild {
        ty: TypeId,
        initializers: Box<[(FieldId, NodeId)]>,
    },
    OrderedRecord {
        ty: TypeId,
        backing: OrderedRecordBacking,
        initializers: Box<[(Box<[FieldId]>, NodeId)]>,
    },
    /// Each place/value/spread is snapshotted before the next part executes.
    SequencePack {
        ty: TypeId,
        parts: Box<[PackPartNode]>,
    },
}
#[derive(Clone, Copy, Debug)]
pub(super) enum ShortCircuitOp {
    And,
    Or,
}
#[derive(Clone, Copy, Debug)]
pub(super) enum CallTarget {
    Direct(ProcedureId),
    Indirect(NodeId),
}
#[derive(Clone, Copy, Debug)]
pub(super) enum PackPartMode {
    ElementValue,
    ElementPlace,
    Spread,
}
#[derive(Clone, Copy, Debug)]
pub(super) struct PackPartNode {
    pub(super) mode: PackPartMode,
    pub(super) node: NodeId,
}

#[derive(Debug)]
pub(super) struct BlockCode {
    pub(super) statements: Box<[StatementCode]>,
    pub(super) flow: Flow,
}
#[derive(Debug)]
pub(super) enum StatementCode {
    Store {
        destination: NodeId,
        value: NodeId,
    },
    Discard(NodeId),
    CallResults {
        call: NodeId,
        destinations: Box<[Option<NodeId>]>,
    },
    Exit(ExitCode),
    Cleanup(CleanupId),
    If {
        condition: NodeId,
        then_block: BlockId,
        else_block: BlockId,
    },
    Cases(CasesCode),
    While {
        id: LoopId,
        condition: ConditionCode,
        body: BlockId,
    },
    Range(RangeCode),
    Block(BlockId),
    PushContext {
        id: PushContextId,
        value: NodeId,
        body: BlockId,
    },
    Simd(SimdCode),
}
#[derive(Debug)]
pub(super) struct ExitCode {
    /// Values are captured in source order before any cleanup task is scheduled.
    pub(super) values: Box<[NodeId]>,
    pub(super) cleanups: Box<[CleanupId]>,
    pub(super) transfer: TransferCode,
}
#[derive(Clone, Copy, Debug)]
pub(super) enum TransferCode {
    Return,
    Break(LoopId),
    Continue(LoopId),
}
#[derive(Clone, Copy, Debug)]
pub(super) enum ConditionCode {
    Value(NodeId),
    BoundInt { destination: NodeId, value: NodeId },
    BoundBool { destination: NodeId, value: NodeId },
}
#[derive(Debug)]
pub(super) struct RangeCode {
    pub(super) id: LoopId,
    pub(super) iterator: NodeId,
    pub(super) integer_type: IntegerType,
    pub(super) start: NodeId,
    pub(super) end: NodeId,
    pub(super) direction: Direction,
    pub(super) body: BlockId,
}
#[derive(Debug)]
pub(super) struct CasesCode {
    pub(super) subject: BlockId,
    pub(super) arms: Box<[CaseCode]>,
    pub(super) default: Option<BlockId>,
    pub(super) flow: Flow,
    pub(super) exhaustive: bool,
}
#[derive(Clone, Copy, Debug)]
pub(super) struct CaseCode {
    pub(super) condition: NodeId,
    pub(super) body: BlockId,
    pub(super) through: bool,
}
#[derive(Clone, Copy, Debug)]
pub(super) struct CleanupCode {
    pub(super) body: BlockId,
    pub(super) context: CleanupContext,
}
#[derive(Debug)]
pub(super) struct SimdCode {
    pub(super) features: SimdFeatures,
    pub(super) registers: Box<[SimdWidth]>,
    pub(super) instructions: Box<[SimdInstructionCode]>,
}
#[derive(Clone, Copy, Debug)]
pub(super) enum SimdInstructionCode {
    Trap,
    Load {
        destination: usize,
        interpretation: SimdInterpretation,
        address: NodeId,
        width: SimdWidth,
    },
    Store {
        source: usize,
        interpretation: SimdInterpretation,
        address: NodeId,
        width: SimdWidth,
    },
    Add {
        destination: usize,
        left: usize,
        right: usize,
        interpretation: SimdInterpretation,
        width: SimdWidth,
    },
}

#[derive(Clone, Copy, Debug)]
pub(super) enum Entry {
    Node(NodeId),
    Block(BlockId),
}
#[derive(Debug)]
pub(super) struct Plan {
    pub(super) binding_owner: Option<ProcedureId>,
    pub(super) nodes: Vec<Node>,
    pub(super) blocks: Vec<BlockCode>,
    pub(super) cleanups: Vec<CleanupCode>,
    pub(super) entry: Entry,
    pub(super) metadata_cells: usize,
    pub(super) planning_work: u64,
}
#[derive(Debug)]
pub(super) struct ProcedurePlan {
    pub(super) source: Arc<Procedure>,
    pub(super) code: Arc<Plan>,
    pub(super) body: BlockId,
}

struct Builder<'a> {
    binding_owner: Option<ProcedureId>,
    types: &'a dyn TypeView,
    signatures: &'a HashMap<ProcedureId, TypeId>,
    places: &'a Places,
    limits: Limits,
    nodes: Vec<Node>,
    blocks: Vec<BlockCode>,
    cleanups: Vec<CleanupCode>,
    metadata_cells: usize,
    planning_work: u64,
}
impl<'a> Builder<'a> {
    fn new(
        types: &'a dyn TypeView,
        signatures: &'a HashMap<ProcedureId, TypeId>,
        places: &'a Places,
        limits: Limits,
    ) -> Self {
        Self {
            binding_owner: None,
            types,
            signatures,
            places,
            limits,
            nodes: vec![],
            blocks: vec![],
            cleanups: vec![],
            metadata_cells: 0,
            planning_work: 0,
        }
    }
    fn reserve(&mut self, count: usize, depth: usize) -> Result<(), Error> {
        if depth > self.limits.evaluation_depth.min(256) {
            return Err(Error::Limit(LimitKind::EvaluationDepth));
        }
        self.metadata_cells = self
            .metadata_cells
            .checked_add(count)
            .filter(|cells| *cells <= self.limits.value_cells)
            .ok_or(Error::Limit(LimitKind::ValueCells))?;
        self.planning_work = self
            .planning_work
            .checked_add(u64::try_from(count).map_err(|_| Error::Limit(LimitKind::Fuel))?)
            .filter(|work| *work <= self.limits.fuel)
            .ok_or(Error::Limit(LimitKind::Fuel))?;
        Ok(())
    }
    fn push(&mut self, ty: Option<TypeId>, kind: NodeKind, depth: usize) -> Result<NodeId, Error> {
        self.reserve(1, depth)?;
        let id = NodeId(self.nodes.len());
        self.nodes.push(Node {
            ty,
            kind,
        });
        Ok(id)
    }
    fn finish(self, entry: Entry) -> Arc<Plan> {
        Arc::new(Plan {
            binding_owner: self.binding_owner,
            nodes: self.nodes,
            blocks: self.blocks,
            cleanups: self.cleanups,
            entry,
            metadata_cells: self.metadata_cells,
            planning_work: self.planning_work,
        })
    }
}

pub(super) fn compile_checked_expression(
    checked: CheckedExpression<'_>,
    limits: Limits,
) -> Result<Arc<Plan>, Error> {
    let mut builder = Builder::new(
        checked.types(),
        checked.signatures(),
        checked.places(),
        limits,
    );
    builder.binding_owner = checked.binding_owner();
    let root = builder.value(checked.expression(), 0)?;
    Ok(builder.finish(Entry::Node(root)))
}
pub(super) fn compile_checked_call(
    checked: CheckedCall<'_>,
    limits: Limits,
) -> Result<Arc<Plan>, Error> {
    let mut builder = Builder::new(
        checked.types(),
        checked.signatures(),
        checked.places(),
        limits,
    );
    builder.binding_owner = checked.binding_owner();
    let root = builder.call(checked.call(), None, 0)?;
    Ok(builder.finish(Entry::Node(root)))
}
pub(super) fn compile_checked_procedure(
    checked: CheckedProcedure<'_>,
    limits: Limits,
) -> Result<ProcedurePlan, Error> {
    let mut builder = Builder::new(
        checked.types(),
        checked.signatures(),
        checked.places(),
        limits,
    );
    let procedure = checked.procedure();
    builder.binding_owner = Some(procedure.id);
    builder.reserve(procedure.locals.len(), 0)?;
    builder.reserve(procedure.parameters.len(), 0)?;
    let body = builder.block(&procedure.body, 0)?;
    builder.reserve(procedure.cleanups.len(), 0)?;
    for cleanup in &procedure.cleanups {
        let block = builder.block(&cleanup.body, 0)?;
        builder.cleanups.push(CleanupCode {
            body: block,
            context: cleanup.context,
        });
    }
    // Every source child has just passed the plan depth check, and the lowered
    // metadata bounds the source AST copy too. Reserve its second owned copy first.
    builder.reserve(builder.metadata_cells, 0)?;
    let source = Arc::new(procedure.clone());
    Ok(ProcedurePlan {
        source,
        code: builder.finish(Entry::Block(body)),
        body,
    })
}
