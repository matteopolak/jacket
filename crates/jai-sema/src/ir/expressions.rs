use super::*;
use jai_eval::Integer as IntegerValue;
use jai_eval::operators::{Equality, IntOp, Relation};
use jai_syntax::{CastMode, IntegerType};
use jai_types::{ScalarType, TypeId, Types};

#[derive(Debug)]
pub struct IntExpr {
    ty: IntegerType,
    kind: IntExprKind,
}
impl IntExpr {
    pub(crate) fn new(ty: IntegerType, kind: IntExprKind) -> Self {
        Self { ty, kind }
    }
    pub fn ty(&self) -> IntegerType {
        self.ty
    }
    pub fn type_id(&self, types: &Types) -> TypeId {
        types.scalar(ScalarType::Int(self.ty))
    }
    pub fn kind(&self) -> &IntExprKind {
        &self.kind
    }
    pub(crate) fn constant(n: IntegerValue) -> Self {
        Self::new(n.ty(), IntExprKind::Constant(n))
    }
    pub(crate) fn load(place: IntPlace) -> Self {
        Self::new(place.ty(), IntExprKind::Load(place))
    }
}
#[derive(Debug)]
pub enum IntExprKind {
    Constant(IntegerValue),
    InvalidCheckedCast,
    FromBool(Box<BoolExpr>),
    Cast(CastMode, Box<IntExpr>),
    Load(IntPlace),
    Call(Call),
    Negate(Box<IntExpr>),
    Complement(Box<IntExpr>),
    Binary(IntOp, Box<IntExpr>, Box<IntExpr>),
    Conditional(Box<Conditional<IntExpr>>),
}
#[derive(Debug)]
pub enum BoolExpr {
    Constant(bool),
    FromInt(Box<IntExpr>),
    Load(BoolPlace),
    Call(Call),
    Not(Box<BoolExpr>),
    CompareInts(Relation, Box<IntExpr>, Box<IntExpr>),
    CompareBools(Equality, Box<BoolExpr>, Box<BoolExpr>),
    And(Box<BoolExpr>, Box<BoolExpr>),
    Or(Box<BoolExpr>, Box<BoolExpr>),
    Conditional(Box<Conditional<BoolExpr>>),
}
impl BoolExpr {
    pub fn type_id(&self, types: &Types) -> TypeId {
        types.scalar(ScalarType::Bool)
    }
}
#[derive(Debug)]
pub struct Conditional<T> {
    pub condition: BoolExpr,
    pub then_value: T,
    pub else_value: T,
}
#[derive(Debug)]
pub enum ValueExpr {
    Int(IntExpr),
    Bool(BoolExpr),
}
impl ValueExpr {
    pub fn type_id(&self, types: &Types) -> TypeId {
        match self {
            Self::Int(e) => e.type_id(types),
            Self::Bool(e) => e.type_id(types),
        }
    }
}
#[derive(Debug)]
pub struct Call {
    pub procedure: ProcedureId,
    pub arguments: Vec<(ParameterId, ValueExpr)>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use jai_types::{Integer, TypeKind, TypeRegistry};
    #[test]
    fn scalar_expression_views_share_the_frozen_registry() {
        let types = TypeRegistry::new().freeze().unwrap();
        let integer = IntExpr::constant(Integer::checked(IntegerType::U16, 256).unwrap());
        let boolean = BoolExpr::Constant(true);
        assert!(matches!(
            types.kind(integer.type_id(&types)),
            Ok(TypeKind::Integer(IntegerType::U16))
        ));
        assert!(matches!(
            types.kind(boolean.type_id(&types)),
            Ok(TypeKind::Bool)
        ));
        let value = ValueExpr::Int(integer);
        assert_eq!(
            value.type_id(&types),
            types.scalar(ScalarType::Int(IntegerType::U16))
        );
    }
}
