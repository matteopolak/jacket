//! Lower checked expression trees into owned, typed continuation operations.
use super::*;
use crate::Value;
use jai_types::{CastMode, Equality, FloatOp, FloatType, Relation};

#[derive(Debug)]
pub(in crate::execute::resumable) enum ApplyOp {
    Bound(ExpressionBindingId),
    Literal(Value),
    StringBytes {
        ty: TypeId,
        bytes: Arc<[u8]>,
    },
    NativePointer(jai_ir::NativePointerConstant),
    RuntimeType(RuntimeTypeConstant),
    StaticAddress {
        data: Arc<StaticData>,
        address: StaticAddress,
    },
    Context(TypeId),
    Zero(TypeId),
    Load,
    StorageBitcast {
        cast: jai_types::StorageBitcast,
        from_place: bool,
    },
    Int {
        ty: IntegerType,
        op: IntApply,
    },
    Bool(BoolApply),
    Float {
        ty: FloatType,
        op: FloatApply,
    },
    TypeDescriptor(TypeId),
    Array(TypeId),
    Record(TypeId),
    Union {
        ty: TypeId,
        field: FieldId,
    },
    Field {
        ty: TypeId,
        field: FieldId,
    },
    SequenceField {
        base_type: TypeId,
        field: SequenceField,
        from_place: bool,
        static_backing: bool,
    },
    ArrayView {
        base_type: TypeId,
        ty: TypeId,
        from_place: bool,
        static_backing: bool,
    },
    SequenceView {
        ty: TypeId,
        from_place: bool,
    },
    SequenceBuild {
        ty: TypeId,
        fields: Box<[SequenceField]>,
    },
    SequenceIndex {
        ty: TypeId,
        check: CheckMode,
    },
    AddressOf,
    AddressOfValue(TypeId),
    PointerCast {
        ty: TypeId,
        mode: CastMode,
    },
    PointerOffset {
        ty: TypeId,
        subtract: bool,
        integer_first: bool,
    },
    PointerFromInteger {
        ty: TypeId,
        mode: CastMode,
    },
    Distinct(TypeId),
    UnwrapDistinct(TypeId),
    EnumFromInt(TypeId),
}

#[derive(Clone, Copy, Debug)]
pub(in crate::execute::resumable) enum IntApply {
    FromValue,
    EnumValue,
    PointerDifference,
    FromFloat(CastMode),
    FromPointer(CastMode),
    FromBool,
    Cast(CastMode),
    InvalidCheckedCast,
    Negate(CheckMode),
    Complement,
    Binary(IntOp, CheckMode),
}
#[derive(Clone, Copy, Debug)]
pub(in crate::execute::resumable) enum BoolApply {
    CompileTime,
    FromValue,
    FromInt,
    FromPointer,
    Not,
    CompareInts(Relation),
    CompareFloats(Relation),
    ComparePointers(Equality),
    CompareStrings(Equality),
    CompareBools(Equality),
}
#[derive(Clone, Copy, Debug)]
pub(in crate::execute::resumable) enum FloatApply {
    FromValue,
    Negate,
    Binary(FloatOp),
    Cast,
    FromInt,
}
#[derive(Clone, Copy, Debug)]
pub(in crate::execute::resumable) enum PlaceOp {
    Context(TypeId),
    Local(LocalId),
    Global(GlobalId),
    Field(FieldId),
    Dereference,
    SequenceField(SequenceField),
}

impl Builder<'_> {
    pub(super) fn bool(&mut self, expression: &BoolExpr, depth: usize) -> Result<NodeId, Error> {
        self.boolean(expression, depth)
    }

    fn apply(
        &mut self,
        ty: TypeId,
        op: ApplyOp,
        operands: Vec<NodeId>,
        depth: usize,
    ) -> Result<NodeId, Error> {
        self.reserve(operands.len(), depth)?;
        self.push(
            Some(ty),
            NodeKind::Apply {
                op,
                operands: operands.into(),
            },
            depth,
        )
    }

    pub(super) fn value(&mut self, expression: &ValueExpr, depth: usize) -> Result<NodeId, Error> {
        self.reserve(0, depth)?;
        let ty = expression.type_id(self.types);
        let (op, operands) = match expression {
            ValueExpr::StorageBitcast { source, cast } => {
                let (source, from_place) = match source {
                    StorageBitcastSource::Place(place) => (self.place(*place, depth + 1)?, true),
                    StorageBitcastSource::Value(value) => (self.value(value, depth + 1)?, false),
                };
                (
                    ApplyOp::StorageBitcast {
                        cast: *cast,
                        from_place,
                    },
                    vec![source],
                )
            }
            ValueExpr::Bind { bindings, body, .. } => {
                self.reserve(bindings.len(), depth)?;
                let mut lowered = Vec::with_capacity(bindings.len());
                for (binding, value) in bindings {
                    if Some(binding.procedure()) != self.binding_owner {
                        return Err(Error::InvalidIr("expression binding has another owner"));
                    }
                    lowered.push((*binding, self.value(value, depth + 1)?));
                }
                let body = self.value(body, depth + 1)?;
                return self.push(
                    Some(ty),
                    NodeKind::Bind {
                        bindings: lowered.into(),
                        body,
                    },
                    depth,
                );
            }
            ValueExpr::Bound { binding, .. } => {
                if Some(binding.procedure()) != self.binding_owner {
                    return Err(Error::InvalidIr("expression binding has another owner"));
                }
                (ApplyOp::Bound(*binding), vec![])
            }
            ValueExpr::NativePointer(value) => (ApplyOp::NativePointer(value.clone()), vec![]),
            ValueExpr::RuntimeType(value) => (ApplyOp::RuntimeType(value.clone()), vec![]),
            ValueExpr::TypeDescriptor { value, ty } => (
                ApplyOp::TypeDescriptor(*ty),
                vec![self.value(value, depth + 1)?],
            ),
            ValueExpr::Context { ty } => (ApplyOp::Context(*ty), vec![]),
            ValueExpr::Conditional { expression, .. } => {
                let condition = self.boolean(&expression.condition, depth + 1)?;
                let then_node = self.value(&expression.then_value, depth + 1)?;
                let else_node = self.value(&expression.else_value, depth + 1)?;
                return self.push(
                    Some(ty),
                    NodeKind::Conditional {
                        condition,
                        then_node,
                        else_node,
                    },
                    depth,
                );
            }
            ValueExpr::StaticAddress { data, address, .. } => {
                self.reserve(address.path().len(), depth)?;
                (
                    ApplyOp::StaticAddress {
                        data: Arc::clone(data),
                        address: address.clone(),
                    },
                    vec![],
                )
            }
            ValueExpr::Float(value) => (
                ApplyOp::Float {
                    ty: value.ty(),
                    op: FloatApply::FromValue,
                },
                vec![self.float(value, depth + 1)?],
            ),
            ValueExpr::Int(value) => (
                ApplyOp::Int {
                    ty: value.ty(),
                    op: IntApply::FromValue,
                },
                vec![self.int(value, depth + 1)?],
            ),
            ValueExpr::Bool(value) => (
                ApplyOp::Bool(BoolApply::FromValue),
                vec![self.boolean(value, depth + 1)?],
            ),
            ValueExpr::Array { elements, .. } => {
                self.reserve(elements.len(), depth)?;
                let mut nodes = Vec::with_capacity(elements.len());
                for element in elements {
                    nodes.push(self.value(element, depth + 1)?);
                }
                (ApplyOp::Array(ty), nodes)
            }
            ValueExpr::StringBytes { bytes, .. } => {
                self.reserve(bytes.len(), depth)?;
                (
                    ApplyOp::StringBytes {
                        ty,
                        bytes: Arc::from(bytes.as_slice()),
                    },
                    vec![],
                )
            }
            ValueExpr::SequenceField { base, field, .. } => {
                let (node, from_place) = self.sequence_base(base, depth + 1)?;
                (
                    ApplyOp::SequenceField {
                        base_type: base.type_id(self.types),
                        field: *field,
                        from_place,
                        static_backing: jai_ir::is_static_value(base),
                    },
                    vec![node],
                )
            }
            ValueExpr::ArrayToSlice { array, .. } => (
                ApplyOp::ArrayView {
                    base_type: array.ty(),
                    ty,
                    from_place: true,
                    static_backing: false,
                },
                vec![self.place(*array, depth + 1)?],
            ),
            ValueExpr::ArrayView { array, .. } => {
                let (node, from_place) = self.sequence_base(array, depth + 1)?;
                (
                    ApplyOp::ArrayView {
                        base_type: array.type_id(self.types),
                        ty,
                        from_place,
                        static_backing: jai_ir::is_static_value(array),
                    },
                    vec![node],
                )
            }
            ValueExpr::SequenceView { sequence, .. } => {
                let (node, from_place) = self.sequence_base(sequence, depth + 1)?;
                (ApplyOp::SequenceView { ty, from_place }, vec![node])
            }
            ValueExpr::Index {
                base, index, check, ..
            } => {
                let base = self.value(base, depth + 1)?;
                let index = self.int(index, depth + 1)?;
                (
                    ApplyOp::SequenceIndex { ty, check: *check },
                    vec![base, index],
                )
            }
            ValueExpr::SequenceBuild { initializers, .. } => {
                self.reserve(
                    initializers
                        .len()
                        .checked_mul(2)
                        .ok_or(Error::Limit(LimitKind::ValueCells))?,
                    depth,
                )?;
                let mut fields = Vec::with_capacity(initializers.len());
                let mut nodes = Vec::with_capacity(initializers.len());
                for (field, value) in initializers {
                    fields.push(*field);
                    nodes.push(self.value(value, depth + 1)?);
                }
                (
                    ApplyOp::SequenceBuild {
                        ty,
                        fields: fields.into(),
                    },
                    nodes,
                )
            }
            ValueExpr::AddressOf { place, .. } => {
                (ApplyOp::AddressOf, vec![self.place(*place, depth + 1)?])
            }
            ValueExpr::AddressOfValue { value, .. } => (
                ApplyOp::AddressOfValue(ty),
                vec![self.value(value, depth + 1)?],
            ),
            ValueExpr::PointerCast { value, mode, .. } => (
                ApplyOp::PointerCast { ty, mode: *mode },
                vec![self.value(value, depth + 1)?],
            ),
            ValueExpr::PointerOffset {
                pointer,
                offset,
                subtract,
                ..
            } => {
                let pointer = self.value(pointer, depth + 1)?;
                let offset = self.int(offset, depth + 1)?;
                (
                    ApplyOp::PointerOffset {
                        ty,
                        subtract: *subtract,
                        integer_first: false,
                    },
                    vec![pointer, offset],
                )
            }
            ValueExpr::PointerOffsetLeft {
                offset, pointer, ..
            } => {
                let offset = self.int(offset, depth + 1)?;
                let pointer = self.value(pointer, depth + 1)?;
                (
                    ApplyOp::PointerOffset {
                        ty,
                        subtract: false,
                        integer_first: true,
                    },
                    vec![offset, pointer],
                )
            }
            ValueExpr::PointerFromInteger { value, mode, .. } => (
                ApplyOp::PointerFromInteger { ty, mode: *mode },
                vec![self.int(value, depth + 1)?],
            ),
            ValueExpr::SequenceConcat { parts, .. } => {
                self.reserve(parts.len(), depth)?;
                let mut nodes = Vec::with_capacity(parts.len());
                for part in parts {
                    let (mode, node) = match part {
                        SequencePackPart::Element(ValueExpr::Load(place)) => {
                            (PackPartMode::ElementPlace, self.place(*place, depth + 1)?)
                        }
                        SequencePackPart::Element(value) => {
                            (PackPartMode::ElementValue, self.value(value, depth + 1)?)
                        }
                        SequencePackPart::Spread(value) => {
                            (PackPartMode::Spread, self.value(value, depth + 1)?)
                        }
                    };
                    nodes.push(PackPartNode { mode, node });
                }
                return self.push(
                    Some(ty),
                    NodeKind::SequencePack {
                        ty,
                        parts: nodes.into(),
                    },
                    depth,
                );
            }
            ValueExpr::Distinct { value, .. } => {
                (ApplyOp::Distinct(ty), vec![self.value(value, depth + 1)?])
            }
            ValueExpr::UnwrapDistinct { value, .. } => (
                ApplyOp::UnwrapDistinct(ty),
                vec![self.value(value, depth + 1)?],
            ),
            ValueExpr::ProcedureValue { procedure, .. } => (
                ApplyOp::Literal(Value::Procedure {
                    signature: ty,
                    procedure: Some(*procedure),
                }),
                vec![],
            ),
            ValueExpr::IndirectCall {
                callee, arguments, ..
            } => {
                return self.indirect_call(callee, arguments, Some(ty), depth);
            }
            ValueExpr::Load(place) => (ApplyOp::Load, vec![self.place(*place, depth + 1)?]),
            ValueExpr::Zero(ty) => (ApplyOp::Zero(*ty), vec![]),
            ValueExpr::Record { fields, .. } => {
                self.reserve(fields.len(), depth)?;
                let mut nodes = Vec::with_capacity(fields.len());
                for field in fields {
                    nodes.push(self.value(field, depth + 1)?);
                }
                (ApplyOp::Record(ty), nodes)
            }
            ValueExpr::Union { field, value, .. } => (
                ApplyOp::Union { ty, field: *field },
                vec![self.value(value, depth + 1)?],
            ),
            ValueExpr::OrderedRecord {
                backing,
                initializers,
                ..
            } => {
                self.reserve(initializers.len(), depth)?;
                let mut nodes = Vec::with_capacity(initializers.len());
                for (path, value) in initializers {
                    self.reserve(path.len(), depth)?;
                    nodes.push((path.clone(), self.value(value, depth + 1)?));
                }
                return self.push(
                    Some(ty),
                    NodeKind::OrderedRecord {
                        ty,
                        backing: *backing,
                        initializers: nodes.into(),
                    },
                    depth,
                );
            }
            ValueExpr::RecordBuild { initializers, .. } => {
                self.reserve(initializers.len(), depth)?;
                let mut nodes = Vec::with_capacity(initializers.len());
                for (field, value) in initializers {
                    nodes.push((*field, self.value(value, depth + 1)?));
                }
                return self.push(
                    Some(ty),
                    NodeKind::RecordBuild {
                        ty,
                        initializers: nodes.into(),
                    },
                    depth,
                );
            }
            ValueExpr::Field { base, field, .. } => (
                ApplyOp::Field { ty, field: *field },
                vec![self.value(base, depth + 1)?],
            ),
            ValueExpr::Call { call, .. } => return self.call(call, Some(ty), depth),
            ValueExpr::Enum { value, .. } => {
                (ApplyOp::Literal(Value::Enum { ty, value: *value }), vec![])
            }
            ValueExpr::EnumFromInt { value, .. } => {
                (ApplyOp::EnumFromInt(ty), vec![self.int(value, depth + 1)?])
            }
        };
        self.apply(ty, op, operands, depth)
    }

    fn sequence_base(
        &mut self,
        expression: &ValueExpr,
        depth: usize,
    ) -> Result<(NodeId, bool), Error> {
        match expression {
            ValueExpr::Load(place) => Ok((self.place(*place, depth)?, true)),
            value => Ok((self.value(value, depth)?, false)),
        }
    }

    pub(super) fn int(&mut self, expression: &IntExpr, depth: usize) -> Result<NodeId, Error> {
        self.reserve(0, depth)?;
        let ty = expression.type_id(self.types);
        let (op, operands) = match expression.kind() {
            IntExprKind::Value(value) => (IntApply::FromValue, vec![self.value(value, depth + 1)?]),
            IntExprKind::EnumValue(value) => {
                (IntApply::EnumValue, vec![self.value(value, depth + 1)?])
            }
            IntExprKind::PointerDifference { left, right } => {
                let left = self.value(left, depth + 1)?;
                let right = self.value(right, depth + 1)?;
                (IntApply::PointerDifference, vec![left, right])
            }
            IntExprKind::FromFloat(mode, value) => (
                IntApply::FromFloat(*mode),
                vec![self.float(value, depth + 1)?],
            ),
            IntExprKind::FromPointer { value, mode } => (
                IntApply::FromPointer(*mode),
                vec![self.value(value, depth + 1)?],
            ),
            IntExprKind::Constant(value) => {
                return self.apply(ty, ApplyOp::Literal(Value::Int(*value)), vec![], depth);
            }
            IntExprKind::InvalidCheckedCast => (IntApply::InvalidCheckedCast, vec![]),
            IntExprKind::FromBool(value) => {
                (IntApply::FromBool, vec![self.boolean(value, depth + 1)?])
            }
            IntExprKind::Cast(mode, value) => {
                (IntApply::Cast(*mode), vec![self.int(value, depth + 1)?])
            }
            IntExprKind::Load(place) => return self.load_node(place.place(), ty, depth),
            IntExprKind::Call(call) => return self.call(call, Some(ty), depth),
            IntExprKind::Negate(value) => (
                IntApply::Negate(expression.overflow_check()),
                vec![self.int(value, depth + 1)?],
            ),
            IntExprKind::Complement(value) => {
                (IntApply::Complement, vec![self.int(value, depth + 1)?])
            }
            IntExprKind::Binary(op, left, right) => {
                let left = self.int(left, depth + 1)?;
                let right = self.int(right, depth + 1)?;
                (
                    IntApply::Binary(*op, expression.overflow_check()),
                    vec![left, right],
                )
            }
            IntExprKind::Conditional(expression) => {
                let condition = self.boolean(&expression.condition, depth + 1)?;
                let then_node = self.int(&expression.then_value, depth + 1)?;
                let else_node = self.int(&expression.else_value, depth + 1)?;
                return self.push(
                    Some(ty),
                    NodeKind::Conditional {
                        condition,
                        then_node,
                        else_node,
                    },
                    depth,
                );
            }
        };
        self.apply(
            ty,
            ApplyOp::Int {
                ty: expression.ty(),
                op,
            },
            operands,
            depth,
        )
    }

    pub(super) fn boolean(&mut self, expression: &BoolExpr, depth: usize) -> Result<NodeId, Error> {
        self.reserve(0, depth)?;
        let ty = expression.type_id(self.types);
        let (op, operands) = match expression {
            BoolExpr::CompileTime => (BoolApply::CompileTime, vec![]),
            BoolExpr::Constant(value) => {
                return self.apply(ty, ApplyOp::Literal(Value::Bool(*value)), vec![], depth);
            }
            BoolExpr::Value(value) => (BoolApply::FromValue, vec![self.value(value, depth + 1)?]),
            BoolExpr::FromInt(value) => (BoolApply::FromInt, vec![self.int(value, depth + 1)?]),
            BoolExpr::Load(place) => return self.load_node(place.place(), ty, depth),
            BoolExpr::Call(call) => return self.call(call, Some(ty), depth),
            BoolExpr::Not(value) => (BoolApply::Not, vec![self.boolean(value, depth + 1)?]),
            BoolExpr::CompareInts(op, left, right) => {
                let left = self.int(left, depth + 1)?;
                let right = self.int(right, depth + 1)?;
                (BoolApply::CompareInts(*op), vec![left, right])
            }
            BoolExpr::CompareFloats(op, left, right) => {
                let left = self.float(left, depth + 1)?;
                let right = self.float(right, depth + 1)?;
                (BoolApply::CompareFloats(*op), vec![left, right])
            }
            BoolExpr::ComparePointers(op, left, right) => {
                let left = self.value(left, depth + 1)?;
                let right = self.value(right, depth + 1)?;
                (BoolApply::ComparePointers(*op), vec![left, right])
            }
            BoolExpr::CompareStrings(op, left, right) => {
                let left = self.value(left, depth + 1)?;
                let right = self.value(right, depth + 1)?;
                (BoolApply::CompareStrings(*op), vec![left, right])
            }
            BoolExpr::FromPointer(value) => {
                (BoolApply::FromPointer, vec![self.value(value, depth + 1)?])
            }
            BoolExpr::CompareBools(op, left, right) => {
                let left = self.boolean(left, depth + 1)?;
                let right = self.boolean(right, depth + 1)?;
                (BoolApply::CompareBools(*op), vec![left, right])
            }
            BoolExpr::And(left, right) | BoolExpr::Or(left, right) => {
                let left = self.boolean(left, depth + 1)?;
                let right = self.boolean(right, depth + 1)?;
                let op = if matches!(expression, BoolExpr::And(..)) {
                    ShortCircuitOp::And
                } else {
                    ShortCircuitOp::Or
                };
                return self.push(Some(ty), NodeKind::ShortCircuit { op, left, right }, depth);
            }
            BoolExpr::Conditional(expression) => {
                let condition = self.boolean(&expression.condition, depth + 1)?;
                let then_node = self.boolean(&expression.then_value, depth + 1)?;
                let else_node = self.boolean(&expression.else_value, depth + 1)?;
                return self.push(
                    Some(ty),
                    NodeKind::Conditional {
                        condition,
                        then_node,
                        else_node,
                    },
                    depth,
                );
            }
        };
        self.apply(ty, ApplyOp::Bool(op), operands, depth)
    }

    pub(super) fn float(&mut self, expression: &FloatExpr, depth: usize) -> Result<NodeId, Error> {
        self.reserve(0, depth)?;
        let ty = expression.type_id(self.types);
        let (op, operands) = match expression.kind() {
            FloatExprKind::Constant(value) => {
                return self.apply(ty, ApplyOp::Literal(Value::Float(*value)), vec![], depth);
            }
            FloatExprKind::Value(value) => {
                (FloatApply::FromValue, vec![self.value(value, depth + 1)?])
            }
            FloatExprKind::Load(place) => return self.load_node(*place, ty, depth),
            FloatExprKind::Call(call) => return self.call(call, Some(ty), depth),
            FloatExprKind::Negate(value) => {
                (FloatApply::Negate, vec![self.float(value, depth + 1)?])
            }
            FloatExprKind::Binary(op, left, right) => {
                let left = self.float(left, depth + 1)?;
                let right = self.float(right, depth + 1)?;
                (FloatApply::Binary(*op), vec![left, right])
            }
            FloatExprKind::Cast(value) => (FloatApply::Cast, vec![self.float(value, depth + 1)?]),
            FloatExprKind::FromInt(value) => {
                (FloatApply::FromInt, vec![self.int(value, depth + 1)?])
            }
            FloatExprKind::Conditional(expression) => {
                let condition = self.boolean(&expression.condition, depth + 1)?;
                let then_node = self.float(&expression.then_value, depth + 1)?;
                let else_node = self.float(&expression.else_value, depth + 1)?;
                return self.push(
                    Some(ty),
                    NodeKind::Conditional {
                        condition,
                        then_node,
                        else_node,
                    },
                    depth,
                );
            }
        };
        self.apply(
            ty,
            ApplyOp::Float {
                ty: expression.ty(),
                op,
            },
            operands,
            depth,
        )
    }

    fn load_node(&mut self, place: Place, ty: TypeId, depth: usize) -> Result<NodeId, Error> {
        let place = self.place(place, depth + 1)?;
        self.apply(ty, ApplyOp::Load, vec![place], depth)
    }

    pub(super) fn place(&mut self, place: Place, depth: usize) -> Result<NodeId, Error> {
        self.reserve(0, depth)?;
        let (op, operands) = match place.kind() {
            PlaceKind::Context(ty) => (PlaceOp::Context(ty), vec![]),
            PlaceKind::Local(id) => (PlaceOp::Local(id), vec![]),
            PlaceKind::Global(id) => (PlaceOp::Global(id), vec![]),
            PlaceKind::Field(id) => {
                let projection = self
                    .places
                    .projection(id)
                    .map_err(|error| Error::IrValidation(error.to_string()))?;
                (
                    PlaceOp::Field(projection.field),
                    vec![self.place(projection.base, depth + 1)?],
                )
            }
            PlaceKind::Dereference(id) => {
                let projection = self
                    .places
                    .dereference(id)
                    .map_err(|error| Error::IrValidation(error.to_string()))?;
                (
                    PlaceOp::Dereference,
                    vec![self.value(&projection.pointer, depth + 1)?],
                )
            }
            PlaceKind::Index(id) => {
                let projection = self
                    .places
                    .index(id)
                    .map_err(|error| Error::IrValidation(error.to_string()))?;
                let base_type = projection.base.ty();
                let base = self.place(projection.base, depth + 1)?;
                let index = self.int(&projection.index, depth + 1)?;
                return self.push(
                    Some(place.ty()),
                    NodeKind::IndexPlace {
                        base,
                        index,
                        base_type,
                        check: projection.check,
                    },
                    depth,
                );
            }
            PlaceKind::SequenceField(id) => {
                let projection = self
                    .places
                    .sequence_field(id)
                    .map_err(|error| Error::IrValidation(error.to_string()))?;
                (
                    PlaceOp::SequenceField(projection.field),
                    vec![self.place(projection.base, depth + 1)?],
                )
            }
        };
        self.reserve(operands.len(), depth)?;
        self.push(
            Some(place.ty()),
            NodeKind::Place {
                op,
                operands: operands.into(),
            },
            depth,
        )
    }

    fn arguments(
        &mut self,
        arguments: &[(ParameterId, ValueExpr)],
        depth: usize,
    ) -> Result<Box<[(ParameterId, NodeId)]>, Error> {
        self.reserve(arguments.len(), depth)?;
        let mut nodes = Vec::with_capacity(arguments.len());
        for (parameter, value) in arguments {
            nodes.push((*parameter, self.value(value, depth + 1)?));
        }
        Ok(nodes.into())
    }

    pub(super) fn call(
        &mut self,
        call: &Call,
        output_ty: Option<TypeId>,
        depth: usize,
    ) -> Result<NodeId, Error> {
        self.reserve(0, depth)?;
        let signature = self
            .signatures
            .get(&call.procedure)
            .copied()
            .ok_or(Error::MissingProcedure(call.procedure))?;
        let arguments = self.arguments(&call.arguments, depth + 1)?;
        self.push(
            output_ty,
            NodeKind::Call {
                target: CallTarget::Direct(call.procedure),
                signature,
                arguments,
            },
            depth,
        )
    }

    pub(super) fn indirect_call(
        &mut self,
        callee: &ValueExpr,
        arguments: &[(ParameterId, ValueExpr)],
        output_ty: Option<TypeId>,
        depth: usize,
    ) -> Result<NodeId, Error> {
        self.reserve(0, depth)?;
        let signature = callee.type_id(self.types);
        let callee = self.value(callee, depth + 1)?;
        let arguments = self.arguments(arguments, depth + 1)?;
        self.push(
            output_ty,
            NodeKind::Call {
                target: CallTarget::Indirect(callee),
                signature,
                arguments,
            },
            depth,
        )
    }
}

#[cfg(test)]
mod tests;
