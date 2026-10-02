//! Domain operators shared by semantic lowering and constant execution.
use jai_syntax::BinaryOp;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IntOp {
    Add,
    Subtract,
    Multiply,
    Divide,
    Remainder,
    BitAnd,
    BitOr,
    BitXor,
    ShiftLeft,
    ShiftRight,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Relation {
    Equal,
    NotEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Equality {
    Equal,
    NotEqual,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Operator {
    Integer(IntOp),
    Relation(Relation),
    Equality(Equality),
    And,
    Or,
}
impl From<BinaryOp> for Operator {
    fn from(op: BinaryOp) -> Self {
        match op {
            BinaryOp::Add => Self::Integer(IntOp::Add),
            BinaryOp::Subtract => Self::Integer(IntOp::Subtract),
            BinaryOp::Multiply => Self::Integer(IntOp::Multiply),
            BinaryOp::Divide => Self::Integer(IntOp::Divide),
            BinaryOp::Remainder => Self::Integer(IntOp::Remainder),
            BinaryOp::BitAnd => Self::Integer(IntOp::BitAnd),
            BinaryOp::BitOr => Self::Integer(IntOp::BitOr),
            BinaryOp::BitXor => Self::Integer(IntOp::BitXor),
            BinaryOp::ShiftLeft => Self::Integer(IntOp::ShiftLeft),
            BinaryOp::ShiftRight => Self::Integer(IntOp::ShiftRight),
            BinaryOp::Less => Self::Relation(Relation::Less),
            BinaryOp::LessEqual => Self::Relation(Relation::LessEqual),
            BinaryOp::Greater => Self::Relation(Relation::Greater),
            BinaryOp::GreaterEqual => Self::Relation(Relation::GreaterEqual),
            BinaryOp::Equal => Self::Equality(Equality::Equal),
            BinaryOp::NotEqual => Self::Equality(Equality::NotEqual),
            BinaryOp::LogicalAnd => Self::And,
            BinaryOp::LogicalOr => Self::Or,
        }
    }
}
