use super::Work;
use crate::{
    BoolExpr, Call, Conditional, FloatExpr, FloatExprKind, IntExpr, IntExprKind, ValueExpr,
};

pub(super) fn value(value: ValueExpr, pending: &mut Vec<Work>) {
    match value {
        ValueExpr::StorageBitcast { source, .. } => {
            if let crate::StorageBitcastSource::Value(value) = source {
                pending.push(Work::Value(*value));
            }
        }
        ValueExpr::Bind { bindings, body, .. } => {
            pending.extend(bindings.into_iter().map(|(_, value)| Work::Value(value)));
            pending.push(Work::Value(*body));
        }
        ValueExpr::Bound { .. } => {}
        ValueExpr::Context { .. }
        | ValueExpr::NativePointer(_)
        | ValueExpr::RuntimeType(_)
        | ValueExpr::StaticAddress { .. }
        | ValueExpr::StringBytes { .. }
        | ValueExpr::ArrayToSlice { .. }
        | ValueExpr::AddressOf { .. }
        | ValueExpr::ProcedureValue { .. }
        | ValueExpr::Load(_)
        | ValueExpr::Zero(_)
        | ValueExpr::Enum { .. } => {}
        ValueExpr::Conditional { expression, .. } => {
            let Conditional {
                condition,
                then_value,
                else_value,
            } = *expression;
            pending.push(Work::Bool(condition));
            pending.push(Work::Value(then_value));
            pending.push(Work::Value(else_value));
        }
        ValueExpr::Float(expression) => pending.push(Work::Float(expression)),
        ValueExpr::Int(expression)
        | ValueExpr::EnumFromInt {
            value: expression, ..
        } => {
            pending.push(Work::Int(expression));
        }
        ValueExpr::Bool(expression) => pending.push(Work::Bool(expression)),
        ValueExpr::Array { elements, .. }
        | ValueExpr::Record {
            fields: elements, ..
        } => {
            pending.extend(elements.into_iter().map(Work::Value));
        }
        ValueExpr::SequenceField { base: value, .. }
        | ValueExpr::ArrayView { array: value, .. }
        | ValueExpr::SequenceView {
            sequence: value, ..
        }
        | ValueExpr::AddressOfValue { value, .. }
        | ValueExpr::TypeDescriptor { value, .. }
        | ValueExpr::PointerCast { value, .. }
        | ValueExpr::Distinct { value, .. }
        | ValueExpr::UnwrapDistinct { value, .. }
        | ValueExpr::Union { value, .. }
        | ValueExpr::Field { base: value, .. } => pending.push(Work::Value(*value)),
        ValueExpr::Index { base, index, .. } => {
            pending.push(Work::Value(*base));
            pending.push(Work::Int(index));
        }
        ValueExpr::SequenceBuild { initializers, .. } => {
            pending.extend(
                initializers
                    .into_iter()
                    .map(|(_, value)| Work::Value(value)),
            );
        }
        ValueExpr::PointerOffset {
            pointer, offset, ..
        } => {
            pending.push(Work::Value(*pointer));
            pending.push(Work::Int(offset));
        }
        ValueExpr::PointerOffsetLeft {
            offset, pointer, ..
        } => {
            pending.push(Work::Int(offset));
            pending.push(Work::Value(*pointer));
        }
        ValueExpr::PointerFromInteger { value, .. } => pending.push(Work::Int(value)),
        ValueExpr::SequenceConcat { parts, .. } => {
            pending.extend(parts.into_iter().map(|part| match part {
                crate::SequencePackPart::Element(value)
                | crate::SequencePackPart::Spread(value) => Work::Value(value),
            }));
        }
        ValueExpr::IndirectCall {
            callee, arguments, ..
        } => {
            pending.push(Work::Value(*callee));
            pending.extend(arguments.into_iter().map(|(_, value)| Work::Value(value)));
        }
        ValueExpr::OrderedRecord { initializers, .. } => {
            pending.extend(
                initializers
                    .into_iter()
                    .map(|(_, value)| Work::Value(value)),
            );
        }
        ValueExpr::RecordBuild { initializers, .. } => {
            pending.extend(
                initializers
                    .into_iter()
                    .map(|(_, value)| Work::Value(value)),
            );
        }
        ValueExpr::Call { call, .. } => pending.push(Work::Call(call)),
    }
}

pub(super) fn integer(expression: IntExpr, pending: &mut Vec<Work>) {
    match expression.into_kind() {
        IntExprKind::Constant(_) | IntExprKind::InvalidCheckedCast | IntExprKind::Load(_) => {}
        IntExprKind::Value(value) | IntExprKind::EnumValue(value) => {
            pending.push(Work::Value(*value));
        }
        IntExprKind::PointerDifference { left, right } => {
            pending.push(Work::Value(*left));
            pending.push(Work::Value(*right));
        }
        IntExprKind::FromFloat(_, value) => pending.push(Work::Float(*value)),
        IntExprKind::FromPointer { value, .. } => pending.push(Work::Value(*value)),
        IntExprKind::FromBool(value) => pending.push(Work::Bool(*value)),
        IntExprKind::Cast(_, value)
        | IntExprKind::Negate(value)
        | IntExprKind::Complement(value) => pending.push(Work::Int(*value)),
        IntExprKind::Call(call) => pending.push(Work::Call(call)),
        IntExprKind::Binary(_, left, right) => {
            pending.push(Work::Int(*left));
            pending.push(Work::Int(*right));
        }
        IntExprKind::Conditional(expression) => {
            let Conditional {
                condition,
                then_value,
                else_value,
            } = *expression;
            pending.push(Work::Bool(condition));
            pending.push(Work::Int(then_value));
            pending.push(Work::Int(else_value));
        }
    }
}

pub(super) fn boolean(expression: BoolExpr, pending: &mut Vec<Work>) {
    match expression {
        BoolExpr::CompileTime | BoolExpr::Constant(_) | BoolExpr::Load(_) => {}
        BoolExpr::Value(value) | BoolExpr::FromPointer(value) => pending.push(Work::Value(*value)),
        BoolExpr::FromInt(value) => pending.push(Work::Int(*value)),
        BoolExpr::Call(call) => pending.push(Work::Call(call)),
        BoolExpr::Not(value) => pending.push(Work::Bool(*value)),
        BoolExpr::CompareInts(_, left, right) => {
            pending.push(Work::Int(*left));
            pending.push(Work::Int(*right));
        }
        BoolExpr::CompareFloats(_, left, right) => {
            pending.push(Work::Float(*left));
            pending.push(Work::Float(*right));
        }
        BoolExpr::ComparePointers(_, left, right) | BoolExpr::CompareStrings(_, left, right) => {
            pending.push(Work::Value(*left));
            pending.push(Work::Value(*right));
        }
        BoolExpr::CompareBools(_, left, right)
        | BoolExpr::And(left, right)
        | BoolExpr::Or(left, right) => {
            pending.push(Work::Bool(*left));
            pending.push(Work::Bool(*right));
        }
        BoolExpr::Conditional(expression) => {
            let Conditional {
                condition,
                then_value,
                else_value,
            } = *expression;
            pending.push(Work::Bool(condition));
            pending.push(Work::Bool(then_value));
            pending.push(Work::Bool(else_value));
        }
    }
}

pub(super) fn float(expression: FloatExpr, pending: &mut Vec<Work>) {
    match expression.into_kind() {
        FloatExprKind::Constant(_) | FloatExprKind::Load(_) => {}
        FloatExprKind::Value(value) => pending.push(Work::Value(*value)),
        FloatExprKind::Call(call) => pending.push(Work::Call(call)),
        FloatExprKind::Negate(value) | FloatExprKind::Cast(value) => {
            pending.push(Work::Float(*value));
        }
        FloatExprKind::Binary(_, left, right) => {
            pending.push(Work::Float(*left));
            pending.push(Work::Float(*right));
        }
        FloatExprKind::FromInt(value) => pending.push(Work::Int(*value)),
        FloatExprKind::Conditional(expression) => {
            let Conditional {
                condition,
                then_value,
                else_value,
            } = *expression;
            pending.push(Work::Bool(condition));
            pending.push(Work::Float(then_value));
            pending.push(Work::Float(else_value));
        }
    }
}

pub(super) fn call(call: Call, pending: &mut Vec<Work>) {
    pending.extend(
        call.arguments
            .into_iter()
            .map(|(_, value)| Work::Value(value)),
    );
}
