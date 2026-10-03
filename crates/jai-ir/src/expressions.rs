use super::*;
use jai_types::{FieldId, Integer as IntegerValue, IntegerType, ScalarType, TypeId, TypeView};
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct IntExpr {
    ty: IntegerType,
    kind: IntExprKind,
    overflow_check: CheckMode,
}
impl IntExpr {
    #[doc(hidden)]
    pub fn new(ty: IntegerType, kind: IntExprKind) -> Self {
        Self {
            ty,
            kind,
            overflow_check: CheckMode::Disabled,
        }
    }
    /// Select overflow behavior for this operation without changing its operands.
    pub fn with_overflow_check(mut self, check: CheckMode) -> Self {
        self.overflow_check = check;
        self
    }
    pub fn overflow_check(&self) -> CheckMode {
        self.overflow_check
    }
    pub fn ty(&self) -> IntegerType {
        self.ty
    }
    pub fn type_id(&self, types: &dyn TypeView) -> TypeId {
        types.scalar(ScalarType::Int(self.ty))
    }
    pub fn kind(&self) -> &IntExprKind {
        &self.kind
    }
    pub(crate) fn into_kind(self) -> IntExprKind {
        self.kind
    }
    pub fn constant(n: IntegerValue) -> Self {
        Self::new(n.ty(), IntExprKind::Constant(n))
    }
    pub fn load(place: IntPlace) -> Self {
        Self::new(place.ty(), IntExprKind::Load(place))
    }
}
#[derive(Clone, Debug)]
pub enum IntExprKind {
    Value(Box<ValueExpr>),
    EnumValue(Box<ValueExpr>),
    PointerDifference {
        left: Box<ValueExpr>,
        right: Box<ValueExpr>,
    },
    FromFloat(CastMode, Box<FloatExpr>),
    FromPointer {
        value: Box<ValueExpr>,
        mode: CastMode,
    },
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
#[derive(Clone, Debug)]
pub enum BoolExpr {
    CompileTime,
    Value(Box<ValueExpr>),
    Constant(bool),
    FromInt(Box<IntExpr>),
    Load(BoolPlace),
    Call(Call),
    Not(Box<BoolExpr>),
    CompareInts(Relation, Box<IntExpr>, Box<IntExpr>),
    CompareFloats(Relation, Box<FloatExpr>, Box<FloatExpr>),
    ComparePointers(Equality, Box<ValueExpr>, Box<ValueExpr>),
    CompareStrings(Equality, Box<ValueExpr>, Box<ValueExpr>),
    FromPointer(Box<ValueExpr>),
    CompareBools(Equality, Box<BoolExpr>, Box<BoolExpr>),
    And(Box<BoolExpr>, Box<BoolExpr>),
    Or(Box<BoolExpr>, Box<BoolExpr>),
    Conditional(Box<Conditional<BoolExpr>>),
}
impl BoolExpr {
    pub fn type_id(&self, types: &dyn TypeView) -> TypeId {
        types.scalar(ScalarType::Bool)
    }
}
#[derive(Clone, Debug)]
pub struct Conditional<T> {
    pub condition: BoolExpr,
    pub then_value: T,
    pub else_value: T,
}
/// Preserve a source place image without loading unrelated, potentially unwritten bytes.
#[derive(Clone, Debug)]
pub enum StorageBitcastSource {
    Place(Place),
    Value(Box<ValueExpr>),
}

#[derive(Clone, Debug)]
pub enum ValueExpr {
    /// Interpret the checked source image using the destination storage layout.
    StorageBitcast {
        source: StorageBitcastSource,
        cast: jai_types::StorageBitcast,
    },
    /// Capture producers once in vector order, then evaluate the body in scope.
    Bind {
        bindings: Vec<(ExpressionBindingId, ValueExpr)>,
        body: Box<ValueExpr>,
        ty: TypeId,
    },
    Bound {
        binding: ExpressionBindingId,
        ty: TypeId,
    },
    NativePointer(crate::NativePointerConstant),
    RuntimeType(RuntimeTypeConstant),
    TypeDescriptor {
        value: Box<ValueExpr>,
        ty: TypeId,
    },
    Context {
        ty: TypeId,
    },
    Conditional {
        ty: TypeId,
        expression: Box<Conditional<ValueExpr>>,
    },
    StaticAddress {
        data: Arc<StaticData>,
        address: StaticAddress,
        ty: TypeId,
    },
    Float(FloatExpr),
    Array {
        ty: TypeId,
        elements: Vec<ValueExpr>,
    },
    StringBytes {
        ty: TypeId,
        bytes: Vec<u8>,
    },
    SequenceField {
        base: Box<ValueExpr>,
        field: SequenceField,
        ty: TypeId,
    },
    ArrayToSlice {
        array: Place,
        ty: TypeId,
    },
    ArrayView {
        array: Box<ValueExpr>,
        ty: TypeId,
    },
    SequenceView {
        sequence: Box<ValueExpr>,
        ty: TypeId,
    },
    Index {
        base: Box<ValueExpr>,
        index: IntExpr,
        ty: TypeId,
        check: CheckMode,
    },
    SequenceBuild {
        ty: TypeId,
        initializers: Vec<(SequenceField, ValueExpr)>,
    },
    AddressOf {
        place: Place,
        ty: TypeId,
    },
    /// Evaluate once and retain the snapshot in the evaluating frame.
    AddressOfValue {
        value: Box<ValueExpr>,
        ty: TypeId,
    },
    PointerCast {
        value: Box<ValueExpr>,
        ty: TypeId,
        mode: CastMode,
    },
    PointerOffset {
        pointer: Box<ValueExpr>,
        offset: IntExpr,
        subtract: bool,
        ty: TypeId,
    },
    PointerOffsetLeft {
        offset: IntExpr,
        pointer: Box<ValueExpr>,
        ty: TypeId,
    },
    PointerFromInteger {
        value: IntExpr,
        ty: TypeId,
        mode: CastMode,
    },
    SequenceConcat {
        ty: TypeId,
        parts: Vec<SequencePackPart>,
    },
    Distinct {
        ty: TypeId,
        value: Box<ValueExpr>,
    },
    UnwrapDistinct {
        value: Box<ValueExpr>,
        ty: TypeId,
    },
    ProcedureValue {
        procedure: ProcedureId,
        ty: TypeId,
    },
    IndirectCall {
        inline_hint: jai_types::InlineHint,
        callee: Box<ValueExpr>,
        arguments: Vec<(ParameterId, ValueExpr)>,
        ty: TypeId,
    },
    Int(IntExpr),
    Bool(BoolExpr),
    Load(Place),
    Zero(TypeId),
    Record {
        ty: TypeId,
        fields: Vec<ValueExpr>,
    },
    Union {
        ty: TypeId,
        field: FieldId,
        value: Box<ValueExpr>,
    },
    RecordBuild {
        ty: TypeId,
        initializers: Vec<(FieldId, ValueExpr)>,
    },
    /// Apply checked field paths in journal order, retaining overlapping writes.
    OrderedRecord {
        ty: TypeId,
        backing: crate::OrderedRecordBacking,
        initializers: Vec<(Box<[FieldId]>, ValueExpr)>,
    },
    Field {
        base: Box<ValueExpr>,
        field: FieldId,
        ty: TypeId,
    },
    Call {
        call: Call,
        ty: TypeId,
    },
    Enum {
        ty: TypeId,
        value: IntegerValue,
    },
    EnumFromInt {
        ty: TypeId,
        value: IntExpr,
    },
}
impl ValueExpr {
    pub fn type_id(&self, types: &dyn TypeView) -> TypeId {
        match self {
            Self::StorageBitcast {
                cast, ..
            } => cast.target_type(),
            Self::NativePointer(value) => value.type_id(),
            Self::RuntimeType(value) => value.ty(),
            Self::Int(e) => e.type_id(types),
            Self::Float(e) => e.type_id(types),
            Self::Bool(e) => e.type_id(types),
            Self::Load(place) => place.ty(),
            Self::Context {
                ty,
            }
            | Self::Bind {
                ty, ..
            }
            | Self::Bound {
                ty, ..
            }
            | Self::TypeDescriptor {
                ty, ..
            }
            | Self::Conditional {
                ty, ..
            }
            | Self::StaticAddress {
                ty, ..
            }
            | Self::Array {
                ty, ..
            }
            | Self::StringBytes {
                ty, ..
            }
            | Self::SequenceField {
                ty, ..
            }
            | Self::ArrayToSlice {
                ty, ..
            }
            | Self::ArrayView {
                ty, ..
            }
            | Self::SequenceView {
                ty, ..
            }
            | Self::Index {
                ty, ..
            }
            | Self::SequenceBuild {
                ty, ..
            }
            | Self::AddressOf {
                ty, ..
            }
            | Self::AddressOfValue {
                ty, ..
            }
            | Self::PointerCast {
                ty, ..
            }
            | Self::PointerOffset {
                ty, ..
            }
            | Self::PointerOffsetLeft {
                ty, ..
            }
            | Self::PointerFromInteger {
                ty, ..
            }
            | Self::SequenceConcat {
                ty, ..
            }
            | Self::Distinct {
                ty, ..
            }
            | Self::UnwrapDistinct {
                ty, ..
            }
            | Self::ProcedureValue {
                ty, ..
            }
            | Self::IndirectCall {
                ty, ..
            }
            | Self::Zero(ty)
            | Self::Record {
                ty, ..
            }
            | Self::Union {
                ty, ..
            }
            | Self::RecordBuild {
                ty, ..
            }
            | Self::OrderedRecord {
                ty, ..
            }
            | Self::Field {
                ty, ..
            }
            | Self::Call {
                ty, ..
            }
            | Self::Enum {
                ty, ..
            }
            | Self::EnumFromInt {
                ty, ..
            } => *ty,
        }
    }
}
impl ConstantValue {
    pub fn into_expression(self) -> ValueExpr {
        match self.kind {
            ConstantKind::NativePointer(value) => ValueExpr::NativePointer(value),
            ConstantKind::RuntimeType(value) => ValueExpr::RuntimeType(value),
            ConstantKind::Float(value) => ValueExpr::Float(FloatExpr::constant(value)),
            ConstantKind::Array(elements) => ValueExpr::Array {
                ty: self.ty,
                elements: elements.into_iter().map(Self::into_expression).collect(),
            },
            ConstantKind::StringBytes(bytes) => ValueExpr::StringBytes {
                ty: self.ty,
                bytes,
            },
            ConstantKind::Distinct(value) => ValueExpr::Distinct {
                ty: self.ty,
                value: Box::new(value.into_expression()),
            },
            ConstantKind::Union {
                field,
                value,
            } => ValueExpr::Union {
                ty: self.ty,
                field,
                value: Box::new(value.into_expression()),
            },
            ConstantKind::Int(value) => ValueExpr::Int(IntExpr::constant(value)),
            ConstantKind::Bool(value) => ValueExpr::Bool(BoolExpr::Constant(value)),
            ConstantKind::Record(fields) => ValueExpr::Record {
                ty: self.ty,
                fields: fields.into_iter().map(Self::into_expression).collect(),
            },
            ConstantKind::Enum(value) => ValueExpr::Enum {
                ty: self.ty,
                value,
            },
            ConstantKind::Procedure(procedure) => ValueExpr::ProcedureValue {
                procedure,
                ty: self.ty,
            },
            ConstantKind::Zero => ValueExpr::Zero(self.ty),
        }
    }
}
#[derive(Clone, Debug)]
pub struct Call {
    pub procedure: ProcedureId,
    pub arguments: Vec<(ParameterId, ValueExpr)>,
    inline_hint: jai_types::InlineHint,
}
impl Call {
    pub fn new(procedure: ProcedureId, arguments: Vec<(ParameterId, ValueExpr)>) -> Self {
        Self {
            procedure,
            arguments,
            inline_hint: jai_types::InlineHint::Automatic,
        }
    }
    pub fn inline_hint(&self) -> jai_types::InlineHint {
        self.inline_hint
    }
    pub fn with_inline_hint(mut self, hint: jai_types::InlineHint) -> Self {
        self.inline_hint = hint;
        self
    }
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
