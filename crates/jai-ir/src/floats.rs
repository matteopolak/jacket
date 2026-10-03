use crate::{Call, Conditional, IntExpr, IrError, Place, ValueExpr};
use jai_types::{FloatOp, FloatType, FloatValue, TypeId, TypeKind, TypeView};

#[derive(Clone, Debug)]
pub struct FloatExpr {
    ty: FloatType,
    kind: FloatExprKind,
}
impl FloatExpr {
    #[doc(hidden)]
    pub fn new(ty: FloatType, kind: FloatExprKind) -> Self {
        Self { ty, kind }
    }
    pub fn ty(&self) -> FloatType {
        self.ty
    }
    pub fn type_id(&self, types: &dyn TypeView) -> TypeId {
        types.float(self.ty)
    }
    pub fn kind(&self) -> &FloatExprKind {
        &self.kind
    }
    pub(crate) fn into_kind(self) -> FloatExprKind {
        self.kind
    }
    pub fn constant(value: FloatValue) -> Self {
        Self::new(value.ty(), FloatExprKind::Constant(value))
    }
    pub fn load(place: Place, types: &dyn TypeView) -> Result<Self, IrError> {
        let TypeKind::Float(ty) = *types.kind(place.ty())? else {
            return Err(IrError::InvalidValue(place.ty()));
        };
        Ok(Self::new(ty, FloatExprKind::Load(place)))
    }
}
#[derive(Clone, Debug)]
pub enum FloatExprKind {
    Constant(FloatValue),
    Value(Box<ValueExpr>),
    Load(Place),
    Call(Call),
    Negate(Box<FloatExpr>),
    Binary(FloatOp, Box<FloatExpr>, Box<FloatExpr>),
    Cast(Box<FloatExpr>),
    FromInt(Box<IntExpr>),
    Conditional(Box<Conditional<FloatExpr>>),
}
