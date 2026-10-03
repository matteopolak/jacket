use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Flow {
    FallsThrough,
    Terminates,
}
#[derive(Clone, Debug)]
pub struct Block {
    pub statements: Vec<Statement>,
    pub flow: Flow,
}
#[derive(Clone, Debug)]
pub enum Statement {
    Simd(SimdBlock),
    PushContext {
        id: PushContextId,
        value: ValueExpr,
        body: Block,
    },
    IndirectCallResults {
        inline_hint: jai_types::InlineHint,
        callee: Box<ValueExpr>,
        arguments: Vec<(ParameterId, ValueExpr)>,
        destinations: Vec<Option<Place>>,
    },
    Store(Place, ValueExpr),
    DiscardValue(ValueExpr),
    CallResults {
        call: Call,
        destinations: Vec<Option<Place>>,
    },
    StoreInt(IntPlace, IntExpr),
    StoreBool(BoolPlace, BoolExpr),
    Exit(Exit),
    Cleanup(CleanupId),
    DiscardInt(IntExpr),
    DiscardBool(BoolExpr),
    CallVoid(Call),
    If(BoolExpr, Block, Block),
    Cases(Cases),
    While {
        id: LoopId,
        condition: LoopCondition,
        body: Block,
    },
    Range(RangeLoop),
    Block(Block),
}
#[derive(Clone, Debug)]
pub struct Exit {
    pub cleanups: Vec<CleanupId>,
    pub transfer: Transfer,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum CleanupContext {
    #[default]
    Procedure,
    Push(PushContextId),
}
#[derive(Clone, Debug)]
pub struct Cleanup {
    pub body: Block,
    pub context: CleanupContext,
}
impl From<Block> for Cleanup {
    fn from(body: Block) -> Self {
        Self {
            body,
            context: CleanupContext::Procedure,
        }
    }
}
#[derive(Clone, Debug)]
pub enum Transfer {
    ReturnValues(Vec<ValueExpr>),
    ReturnVoid,
    ReturnInt(IntExpr),
    ReturnBool(BoolExpr),
    Break(LoopId),
    Continue(LoopId),
}
#[derive(Clone, Debug)]
pub enum LoopCondition {
    Value(BoolExpr),
    BoundInt(IntLocal, IntExpr),
    BoundBool(BoolLocal, BoolExpr),
}
#[derive(Clone, Debug)]
pub struct RangeLoop {
    pub id: LoopId,
    pub iterator: IntLocal,
    pub start: IntExpr,
    pub end: IntExpr,
    pub direction: Direction,
    pub body: Block,
}

#[derive(Clone, Debug)]
pub struct Cases {
    pub subject: Box<Statement>,
    pub arms: Vec<CaseArm>,
    pub default: Option<Block>,
    pub flow: Flow,
    pub exhaustive: bool,
}
#[derive(Clone, Debug)]
pub struct CaseArm {
    pub condition: BoolExpr,
    pub body: Block,
    pub through: bool,
}
