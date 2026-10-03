//! Projection allocation ordinals are not lexical storage identity.
use super::*;
use jai_ir::{
    BoolExpr, Call, FloatExpr, FloatExprKind, IntExpr, IntExprKind, Place, PlaceKind,
    PlaceRegistry, SequencePackPart, ValueExpr,
};
use jai_types::{FloatType, FloatValue, Integer, IntegerType, TypeId};

const MAX_STORAGE_KEY_NODES: usize = 65_536;
const MAX_STORAGE_KEY_BYTES: usize = 1_048_576;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(super) struct StorageKey(Vec<Token>);

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
enum Token {
    Kind(&'static str),
    Type(TypeId),
    IntegerType(IntegerType),
    FloatType(FloatType),
    Integer(Integer),
    Float(FloatValue),
    Bool(bool),
    Count(usize),
    Check(jai_types::CheckMode),
    Cast(jai_types::CastMode),
    StorageCast(jai_types::StorageBitcast),
    IntOp(jai_types::IntOp),
    FloatOp(jai_types::FloatOp),
    Relation(jai_types::Relation),
    Equality(jai_types::Equality),
    Inline(jai_types::InlineHint),
    Local(jai_ir::LocalId),
    Global(jai_ir::GlobalId),
    Field(jai_types::FieldId),
    Sequence(jai_ir::SequenceField),
    Procedure(jai_ir::ProcedureId),
    Parameter(jai_ir::ParameterId),
    ExpressionBinding(jai_ir::ExpressionBindingId),
    Address(jai_ir::StaticAddress),
    RuntimeType(jai_ir::RuntimeTypeIdentity),
    NativePointer(jai_ir::NativePointerConstant),
    Bytes(Vec<u8>),
}

enum Work<'a> {
    Place(Place),
    Int(&'a IntExpr),
    Float(&'a FloatExpr),
    Bool(&'a BoolExpr),
    Value(&'a ValueExpr),
    Call(&'a Call),
    Token(Token),
}

impl StorageKey {
    pub(super) fn nodes(&self) -> usize {
        self.0.len()
    }

    pub(super) fn bytes(&self) -> usize {
        self.0
            .iter()
            .filter_map(|token| match token {
                Token::Bytes(bytes) => Some(bytes.len()),
                _ => None,
            })
            .sum()
    }

    pub(super) fn new(
        places: &PlaceRegistry,
        place: Place,
        span: Span,
    ) -> Result<Self, Diagnostic> {
        let mut output = Vec::new();
        let mut pending = vec![Work::Place(place)];
        let mut bytes = 0usize;
        while let Some(work) = pending.pop() {
            if output.len() >= MAX_STORAGE_KEY_NODES || pending.len() >= MAX_STORAGE_KEY_NODES {
                return Err(Diagnostic::new(
                    span,
                    "lexical storage key exceeds its node limit",
                ));
            }
            let mut emit = |token| output.push(token);
            let invalid = |error: jai_ir::IrError| Diagnostic::new(span, error.to_string());
            match work {
                Work::Token(token) => emit(token),
                Work::Place(place) => {
                    emit(Token::Type(place.ty()));
                    match place.kind() {
                        PlaceKind::Context(_) => emit(Token::Kind("context-place")),
                        PlaceKind::Local(id) => emit(Token::Local(id)),
                        PlaceKind::Global(id) => emit(Token::Global(id)),
                        PlaceKind::Field(id) => {
                            let projection = places.projection(id).map_err(invalid)?;
                            emit(Token::Kind("field-place"));
                            emit(Token::Field(projection.field));
                            pending.push(Work::Place(projection.base));
                        }
                        PlaceKind::Dereference(id) => {
                            let projection = places.dereference_projection(id).map_err(invalid)?;
                            emit(Token::Kind("dereference-place"));
                            pending.push(Work::Value(&projection.pointer));
                        }
                        PlaceKind::Index(id) => {
                            let projection = places.index_projection(id).map_err(invalid)?;
                            emit(Token::Kind("index-place"));
                            emit(Token::Check(projection.check));
                            pending.push(Work::Int(&projection.index));
                            pending.push(Work::Place(projection.base));
                        }
                        PlaceKind::SequenceField(id) => {
                            let projection = places.sequence_projection(id).map_err(invalid)?;
                            emit(Token::Kind("sequence-place"));
                            emit(Token::Sequence(projection.field));
                            pending.push(Work::Place(projection.base));
                        }
                    }
                }
                Work::Int(value) => {
                    emit(Token::IntegerType(value.ty()));
                    emit(Token::Check(value.overflow_check()));
                    let kind = value.kind();
                    match kind {
                        IntExprKind::Constant(value) => emit(Token::Integer(*value)),
                        IntExprKind::InvalidCheckedCast => emit(Token::Kind("invalid-int-cast")),
                        IntExprKind::Load(place) => pending.push(Work::Place(place.place())),
                        IntExprKind::Call(call) => pending.push(Work::Call(call)),
                        IntExprKind::Value(value) | IntExprKind::EnumValue(value) => {
                            emit(Token::Kind(if matches!(kind, IntExprKind::EnumValue(_)) {
                                "enum-value"
                            } else {
                                "int-value"
                            }));
                            pending.push(Work::Value(value));
                        }
                        IntExprKind::PointerDifference {
                            left,
                            right,
                        } => {
                            emit(Token::Kind("pointer-difference"));
                            pending.push(Work::Value(right));
                            pending.push(Work::Value(left));
                        }
                        IntExprKind::FromFloat(mode, value) => {
                            emit(Token::Kind("int-from-float"));
                            emit(Token::Cast(*mode));
                            pending.push(Work::Float(value));
                        }
                        IntExprKind::FromPointer {
                            value,
                            mode,
                        } => {
                            emit(Token::Kind("int-from-pointer"));
                            emit(Token::Cast(*mode));
                            pending.push(Work::Value(value));
                        }
                        IntExprKind::FromBool(value) => {
                            emit(Token::Kind("int-from-bool"));
                            pending.push(Work::Bool(value));
                        }
                        IntExprKind::Cast(mode, value) => {
                            emit(Token::Kind("int-cast"));
                            emit(Token::Cast(*mode));
                            pending.push(Work::Int(value));
                        }
                        IntExprKind::Negate(value) | IntExprKind::Complement(value) => {
                            emit(Token::Kind(if matches!(kind, IntExprKind::Complement(_)) {
                                "complement"
                            } else {
                                "negate"
                            }));
                            pending.push(Work::Int(value));
                        }
                        IntExprKind::Binary(op, left, right) => {
                            emit(Token::IntOp(*op));
                            pending.push(Work::Int(right));
                            pending.push(Work::Int(left));
                        }
                        IntExprKind::Conditional(value) => {
                            emit(Token::Kind("int-conditional"));
                            pending.push(Work::Int(&value.else_value));
                            pending.push(Work::Int(&value.then_value));
                            pending.push(Work::Bool(&value.condition));
                        }
                    }
                }
                Work::Float(value) => {
                    emit(Token::FloatType(value.ty()));
                    let kind = value.kind();
                    match kind {
                        FloatExprKind::Constant(value) => emit(Token::Float(*value)),
                        FloatExprKind::Value(value) => pending.push(Work::Value(value)),
                        FloatExprKind::Load(place) => pending.push(Work::Place(*place)),
                        FloatExprKind::Call(call) => pending.push(Work::Call(call)),
                        FloatExprKind::Negate(value) | FloatExprKind::Cast(value) => {
                            emit(Token::Kind(if matches!(kind, FloatExprKind::Cast(_)) {
                                "float-cast"
                            } else {
                                "float-negate"
                            }));
                            pending.push(Work::Float(value));
                        }
                        FloatExprKind::Binary(op, left, right) => {
                            emit(Token::FloatOp(*op));
                            pending.push(Work::Float(right));
                            pending.push(Work::Float(left));
                        }
                        FloatExprKind::FromInt(value) => {
                            emit(Token::Kind("float-from-int"));
                            pending.push(Work::Int(value));
                        }
                        FloatExprKind::Conditional(value) => {
                            emit(Token::Kind("float-conditional"));
                            pending.push(Work::Float(&value.else_value));
                            pending.push(Work::Float(&value.then_value));
                            pending.push(Work::Bool(&value.condition));
                        }
                    }
                }
                Work::Bool(value) => match value {
                    BoolExpr::CompileTime => emit(Token::Kind("compile-time-bool")),
                    BoolExpr::Constant(value) => emit(Token::Bool(*value)),
                    BoolExpr::Value(value) => pending.push(Work::Value(value)),
                    BoolExpr::Load(place) => pending.push(Work::Place(place.place())),
                    BoolExpr::Call(call) => pending.push(Work::Call(call)),
                    BoolExpr::Not(value) => {
                        emit(Token::Kind("not"));
                        pending.push(Work::Bool(value));
                    }
                    BoolExpr::FromInt(value) => {
                        emit(Token::Kind("bool-from-int"));
                        pending.push(Work::Int(value));
                    }
                    BoolExpr::FromPointer(value) => {
                        emit(Token::Kind("bool-from-pointer"));
                        pending.push(Work::Value(value));
                    }
                    BoolExpr::CompareInts(op, left, right) => {
                        emit(Token::Kind("compare-ints"));
                        emit(Token::Relation(*op));
                        pending.push(Work::Int(right));
                        pending.push(Work::Int(left));
                    }
                    BoolExpr::CompareFloats(op, left, right) => {
                        emit(Token::Kind("compare-floats"));
                        emit(Token::Relation(*op));
                        pending.push(Work::Float(right));
                        pending.push(Work::Float(left));
                    }
                    BoolExpr::ComparePointers(op, left, right)
                    | BoolExpr::CompareStrings(op, left, right) => {
                        emit(Token::Kind(
                            if matches!(value, BoolExpr::ComparePointers(..)) {
                                "compare-pointers"
                            } else {
                                "compare-strings"
                            },
                        ));
                        emit(Token::Equality(*op));
                        pending.push(Work::Value(right));
                        pending.push(Work::Value(left));
                    }
                    BoolExpr::CompareBools(op, left, right) => {
                        emit(Token::Kind("compare-bools"));
                        emit(Token::Equality(*op));
                        pending.push(Work::Bool(right));
                        pending.push(Work::Bool(left));
                    }
                    BoolExpr::And(left, right) | BoolExpr::Or(left, right) => {
                        emit(Token::Kind(if matches!(value, BoolExpr::And(..)) {
                            "and"
                        } else {
                            "or"
                        }));
                        pending.push(Work::Bool(right));
                        pending.push(Work::Bool(left));
                    }
                    BoolExpr::Conditional(value) => {
                        emit(Token::Kind("bool-conditional"));
                        pending.push(Work::Bool(&value.else_value));
                        pending.push(Work::Bool(&value.then_value));
                        pending.push(Work::Bool(&value.condition));
                    }
                },
                Work::Call(call) => {
                    emit(Token::Kind("call"));
                    emit(Token::Procedure(call.procedure));
                    emit(Token::Inline(call.inline_hint()));
                    emit(Token::Count(call.arguments.len()));
                    for (parameter, value) in call.arguments.iter().rev() {
                        pending.push(Work::Value(value));
                        pending.push(Work::Token(Token::Parameter(*parameter)));
                    }
                }
                Work::Value(value) => {
                    let expression = value;
                    match expression {
                        ValueExpr::StorageBitcast {
                            source,
                            cast,
                        } => {
                            emit(Token::Kind("storage-cast"));
                            emit(Token::StorageCast(*cast));
                            match source {
                                jai_ir::StorageBitcastSource::Place(place) => {
                                    emit(Token::Kind("place"));
                                    pending.push(Work::Place(*place));
                                }
                                jai_ir::StorageBitcastSource::Value(value) => {
                                    emit(Token::Kind("value"));
                                    pending.push(Work::Value(value));
                                }
                            }
                        }
                        ValueExpr::Bind {
                            bindings,
                            body,
                            ty,
                        } => {
                            emit(Token::Kind("bind"));
                            emit(Token::Type(*ty));
                            emit(Token::Count(bindings.len()));
                            pending.push(Work::Value(body));
                            for (binding, value) in bindings.iter().rev() {
                                pending.push(Work::Value(value));
                                pending.push(Work::Token(Token::ExpressionBinding(*binding)));
                            }
                        }
                        ValueExpr::Bound {
                            binding,
                            ty,
                        } => {
                            emit(Token::Kind("bound"));
                            emit(Token::Type(*ty));
                            emit(Token::ExpressionBinding(*binding));
                        }
                        ValueExpr::NativePointer(value) => {
                            emit(Token::NativePointer(value.clone()))
                        }
                        ValueExpr::RuntimeType(value) => emit(Token::RuntimeType(value.identity())),
                        ValueExpr::Int(value) => {
                            emit(Token::Kind("integer"));
                            pending.push(Work::Int(value));
                        }
                        ValueExpr::Float(value) => {
                            emit(Token::Kind("float"));
                            pending.push(Work::Float(value));
                        }
                        ValueExpr::Bool(value) => {
                            emit(Token::Kind("bool"));
                            pending.push(Work::Bool(value));
                        }
                        ValueExpr::Load(place) => {
                            emit(Token::Kind("load"));
                            pending.push(Work::Place(*place));
                        }
                        ValueExpr::Context {
                            ty,
                        } => {
                            emit(Token::Kind("context"));
                            emit(Token::Type(*ty));
                        }
                        ValueExpr::Zero(ty) => {
                            emit(Token::Kind("zero"));
                            emit(Token::Type(*ty));
                        }
                        ValueExpr::StaticAddress {
                            address,
                            ty,
                            ..
                        } => {
                            emit(Token::Kind("static-address"));
                            emit(Token::Type(*ty));
                            emit(Token::Address(address.clone()));
                        }
                        ValueExpr::StringBytes {
                            bytes: value,
                            ty,
                        } => {
                            bytes = bytes.checked_add(value.len()).ok_or_else(|| {
                                Diagnostic::new(span, "lexical storage key byte budget overflow")
                            })?;
                            if bytes > MAX_STORAGE_KEY_BYTES {
                                return Err(Diagnostic::new(
                                    span,
                                    "lexical storage key exceeds its byte limit",
                                ));
                            }
                            emit(Token::Kind("string"));
                            emit(Token::Type(*ty));
                            emit(Token::Bytes(value.clone()));
                        }
                        ValueExpr::ProcedureValue {
                            procedure,
                            ty,
                        } => {
                            emit(Token::Kind("procedure"));
                            emit(Token::Type(*ty));
                            emit(Token::Procedure(*procedure));
                        }
                        ValueExpr::Enum {
                            ty,
                            value,
                        } => {
                            emit(Token::Kind("enum"));
                            emit(Token::Type(*ty));
                            emit(Token::Integer(*value));
                        }
                        ValueExpr::Array {
                            ty,
                            elements,
                        }
                        | ValueExpr::Record {
                            ty,
                            fields: elements,
                        } => {
                            emit(Token::Kind(if matches!(value, ValueExpr::Array { .. }) {
                                "array"
                            } else {
                                "record"
                            }));
                            emit(Token::Type(*ty));
                            emit(Token::Count(elements.len()));
                            pending.extend(elements.iter().rev().map(Work::Value));
                        }
                        ValueExpr::Union {
                            ty,
                            field,
                            value,
                        }
                        | ValueExpr::Field {
                            ty,
                            field,
                            base: value,
                        } => {
                            emit(Token::Kind(
                                if matches!(expression, ValueExpr::Union { .. }) {
                                    "union"
                                } else {
                                    "field"
                                },
                            ));
                            emit(Token::Type(*ty));
                            emit(Token::Field(*field));
                            pending.push(Work::Value(value));
                        }
                        ValueExpr::OrderedRecord {
                            ty,
                            backing,
                            initializers,
                        } => {
                            emit(Token::Kind("ordered-record"));
                            emit(Token::Type(*ty));
                            emit(Token::Kind(match backing {
                                jai_ir::OrderedRecordBacking::Zeroed => "zeroed",
                                jai_ir::OrderedRecordBacking::Uninitialized => "uninitialized",
                            }));
                            emit(Token::Count(initializers.len()));
                            for (path, value) in initializers.iter().rev() {
                                pending.push(Work::Value(value));
                                for field in path.iter().rev() {
                                    pending.push(Work::Token(Token::Field(*field)));
                                }
                                pending.push(Work::Token(Token::Count(path.len())));
                            }
                        }
                        ValueExpr::RecordBuild {
                            ty,
                            initializers,
                        } => {
                            emit(Token::Kind("record-build"));
                            emit(Token::Type(*ty));
                            emit(Token::Count(initializers.len()));
                            for (field, value) in initializers.iter().rev() {
                                pending.push(Work::Value(value));
                                pending.push(Work::Token(Token::Field(*field)));
                            }
                        }
                        ValueExpr::SequenceBuild {
                            ty,
                            initializers,
                        } => {
                            emit(Token::Kind("sequence-build"));
                            emit(Token::Type(*ty));
                            emit(Token::Count(initializers.len()));
                            for (field, value) in initializers.iter().rev() {
                                pending.push(Work::Value(value));
                                pending.push(Work::Token(Token::Sequence(*field)));
                            }
                        }
                        ValueExpr::SequenceConcat {
                            ty,
                            parts,
                        } => {
                            emit(Token::Kind("sequence-concat"));
                            emit(Token::Type(*ty));
                            emit(Token::Count(parts.len()));
                            for part in parts.iter().rev() {
                                match part {
                                    SequencePackPart::Element(value) => {
                                        pending.push(Work::Value(value));
                                        pending.push(Work::Token(Token::Kind("element")));
                                    }
                                    SequencePackPart::Spread(value) => {
                                        pending.push(Work::Value(value));
                                        pending.push(Work::Token(Token::Kind("spread")));
                                    }
                                }
                            }
                        }
                        ValueExpr::AddressOf {
                            place,
                            ty,
                        }
                        | ValueExpr::ArrayToSlice {
                            array: place,
                            ty,
                        } => {
                            emit(Token::Kind(
                                if matches!(value, ValueExpr::AddressOf { .. }) {
                                    "address-of"
                                } else {
                                    "array-to-slice"
                                },
                            ));
                            emit(Token::Type(*ty));
                            pending.push(Work::Place(*place));
                        }
                        ValueExpr::TypeDescriptor {
                            value: inner,
                            ty,
                        }
                        | ValueExpr::AddressOfValue {
                            value: inner,
                            ty,
                        }
                        | ValueExpr::ArrayView {
                            array: inner,
                            ty,
                        }
                        | ValueExpr::SequenceView {
                            sequence: inner,
                            ty,
                        }
                        | ValueExpr::Distinct {
                            value: inner,
                            ty,
                        }
                        | ValueExpr::UnwrapDistinct {
                            value: inner,
                            ty,
                        } => {
                            emit(Token::Kind(match value {
                                ValueExpr::TypeDescriptor {
                                    ..
                                } => "type-descriptor",
                                ValueExpr::AddressOfValue {
                                    ..
                                } => "address-of-value",
                                ValueExpr::ArrayView {
                                    ..
                                } => "array-view",
                                ValueExpr::SequenceView {
                                    ..
                                } => "sequence-view",
                                ValueExpr::Distinct {
                                    ..
                                } => "distinct",
                                _ => "unwrap-distinct",
                            }));
                            emit(Token::Type(*ty));
                            pending.push(Work::Value(inner));
                        }
                        ValueExpr::PointerCast {
                            value,
                            ty,
                            mode,
                        } => {
                            emit(Token::Kind("pointer-cast"));
                            emit(Token::Type(*ty));
                            emit(Token::Cast(*mode));
                            pending.push(Work::Value(value));
                        }
                        ValueExpr::SequenceField {
                            base,
                            field,
                            ty,
                        } => {
                            emit(Token::Kind("sequence-field"));
                            emit(Token::Type(*ty));
                            emit(Token::Sequence(*field));
                            pending.push(Work::Value(base));
                        }
                        ValueExpr::Index {
                            base,
                            index,
                            ty,
                            check,
                        } => {
                            emit(Token::Kind("index"));
                            emit(Token::Type(*ty));
                            emit(Token::Check(*check));
                            pending.push(Work::Int(index));
                            pending.push(Work::Value(base));
                        }
                        ValueExpr::PointerOffset {
                            pointer,
                            offset,
                            subtract,
                            ty,
                        } => {
                            emit(Token::Kind("pointer-offset"));
                            emit(Token::Type(*ty));
                            emit(Token::Bool(*subtract));
                            pending.push(Work::Int(offset));
                            pending.push(Work::Value(pointer));
                        }
                        ValueExpr::PointerOffsetLeft {
                            offset,
                            pointer,
                            ty,
                        } => {
                            emit(Token::Kind("pointer-offset-left"));
                            emit(Token::Type(*ty));
                            pending.push(Work::Value(pointer));
                            pending.push(Work::Int(offset));
                        }
                        ValueExpr::PointerFromInteger {
                            value,
                            ty,
                            mode,
                        } => {
                            emit(Token::Kind("pointer-from-integer"));
                            emit(Token::Type(*ty));
                            emit(Token::Cast(*mode));
                            pending.push(Work::Int(value));
                        }
                        ValueExpr::EnumFromInt {
                            value,
                            ty,
                        } => {
                            emit(Token::Kind("enum-from-int"));
                            emit(Token::Type(*ty));
                            pending.push(Work::Int(value));
                        }
                        ValueExpr::Call {
                            call,
                            ty,
                        } => {
                            emit(Token::Kind("typed-call"));
                            emit(Token::Type(*ty));
                            pending.push(Work::Call(call));
                        }
                        ValueExpr::IndirectCall {
                            inline_hint,
                            callee,
                            arguments,
                            ty,
                        } => {
                            emit(Token::Kind("indirect-call"));
                            emit(Token::Type(*ty));
                            emit(Token::Inline(*inline_hint));
                            emit(Token::Count(arguments.len()));
                            for (parameter, value) in arguments.iter().rev() {
                                pending.push(Work::Value(value));
                                pending.push(Work::Token(Token::Parameter(*parameter)));
                            }
                            pending.push(Work::Value(callee));
                        }
                        ValueExpr::Conditional {
                            ty,
                            expression,
                        } => {
                            emit(Token::Kind("conditional-value"));
                            emit(Token::Type(*ty));
                            pending.push(Work::Value(&expression.else_value));
                            pending.push(Work::Value(&expression.then_value));
                            pending.push(Work::Bool(&expression.condition));
                        }
                    }
                }
            }
        }
        Ok(Self(output))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use jai_types::{CheckMode, ScalarType, TypeRegistry};

    fn index(value: i128) -> IntExpr {
        IntExpr::constant(Integer::checked(IntegerType::S64, value).unwrap())
    }

    #[test]
    fn append_only_projections_retain_structural_identity() {
        let mut types = TypeRegistry::new();
        let integer = types.scalar(ScalarType::Int(IntegerType::S64));
        let pointer = types.pointer(integer).unwrap();
        let slice = types.slice(integer).unwrap();
        let owner = jai_ir::ProcedureId::new(0);
        let base = jai_ir::Local::new_typed(owner, 0, slice, &types)
            .unwrap()
            .place();
        let scalar = jai_ir::Local::new_typed(owner, 1, integer, &types)
            .unwrap()
            .place();
        let mut places = PlaceRegistry::new();
        let span = Span::default();

        let first = places.index(base, index(2), &types).unwrap();
        let second = places.index(base, index(2), &types).unwrap();
        assert_ne!(first, second);
        assert_eq!(
            StorageKey::new(&places, first, span).unwrap(),
            StorageKey::new(&places, second, span).unwrap()
        );
        let different_index = places.index(base, index(3), &types).unwrap();
        assert_ne!(
            StorageKey::new(&places, first, span).unwrap(),
            StorageKey::new(&places, different_index, span).unwrap()
        );

        let first = places
            .sequence_field(base, jai_ir::SequenceField::Data, &types)
            .unwrap();
        let second = places
            .sequence_field(base, jai_ir::SequenceField::Data, &types)
            .unwrap();
        assert_ne!(first, second);
        assert_eq!(
            StorageKey::new(&places, first, span).unwrap(),
            StorageKey::new(&places, second, span).unwrap()
        );

        let operand = ValueExpr::AddressOf {
            place: scalar,
            ty: pointer,
        };
        let first = places.dereference(operand.clone(), &types).unwrap();
        let second = places.dereference(operand, &types).unwrap();
        assert_ne!(first, second);
        assert_eq!(
            StorageKey::new(&places, first, span).unwrap(),
            StorageKey::new(&places, second, span).unwrap()
        );
    }

    #[test]
    fn operands_and_check_policies_remain_in_the_key() {
        let mut types = TypeRegistry::new();
        let integer = types.scalar(ScalarType::Int(IntegerType::S64));
        let slice = types.slice(integer).unwrap();
        let base = jai_ir::Local::new_typed(jai_ir::ProcedureId::new(0), 0, slice, &types)
            .unwrap()
            .place();
        let mut places = PlaceRegistry::new();
        let span = Span::default();
        let negate = IntExpr::new(IntegerType::S64, IntExprKind::Negate(Box::new(index(2))));
        let complement = IntExpr::new(
            IntegerType::S64,
            IntExprKind::Complement(Box::new(index(2))),
        );
        let first = places
            .index_with_check(base, negate.clone(), CheckMode::Enabled, &types)
            .unwrap();
        let other_operation = places
            .index_with_check(base, complement, CheckMode::Enabled, &types)
            .unwrap();
        let unchecked = places
            .index_with_check(base, negate, CheckMode::Disabled, &types)
            .unwrap();
        assert_ne!(
            StorageKey::new(&places, first, span).unwrap(),
            StorageKey::new(&places, other_operation, span).unwrap()
        );
        assert_ne!(
            StorageKey::new(&places, first, span).unwrap(),
            StorageKey::new(&places, unchecked, span).unwrap()
        );
    }
}
