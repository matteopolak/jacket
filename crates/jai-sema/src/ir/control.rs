use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Flow {
    FallsThrough,
    Terminates,
}
#[derive(Debug)]
pub struct Block {
    pub statements: Vec<Statement>,
    pub flow: Flow,
}
#[derive(Debug)]
pub enum Statement {
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
#[derive(Debug)]
pub struct Exit {
    pub cleanups: Vec<CleanupId>,
    pub transfer: Transfer,
}
#[derive(Debug)]
pub enum Transfer {
    ReturnVoid,
    ReturnInt(IntExpr),
    ReturnBool(BoolExpr),
    Break(LoopId),
    Continue(LoopId),
}
#[derive(Debug)]
pub enum LoopCondition {
    Value(BoolExpr),
    BoundInt(IntLocal, IntExpr),
    BoundBool(BoolLocal, BoolExpr),
}
#[derive(Debug)]
pub struct RangeLoop {
    pub id: LoopId,
    pub iterator: IntLocal,
    pub start: IntExpr,
    pub end: IntExpr,
    pub direction: jai_syntax::Direction,
    pub body: Block,
}

#[derive(Debug)]
pub struct Cases {
    pub subject: Box<Statement>,
    pub arms: Vec<CaseArm>,
    pub default: Option<Block>,
    pub flow: Flow,
    pub exhaustive: bool,
}
#[derive(Debug)]
pub struct CaseArm {
    pub condition: BoolExpr,
    pub body: Block,
    pub through: bool,
}
