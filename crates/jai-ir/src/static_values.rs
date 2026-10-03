//! Conservative classification for immutable literal backing storage.
use crate::expression_bindings::MAX_EXPRESSION_BINDINGS;
use crate::{
    BoolExpr, ExpressionBindingId, FloatExpr, FloatExprKind, IntExpr, IntExprKind, IntOp, ValueExpr,
};
use jai_types::CastMode;
use std::collections::HashSet;

/// True only for values whose backing can be emitted as an LLVM constant.
/// Calls, loads, checked casts, ordinary control-flow expressions, division, and
/// shifts stay runtime values. This classifies backing after expression
/// evaluation; admitted checked arithmetic must still execute its overflow guard.
pub fn is_static_value(value: &ValueExpr) -> bool {
    enum Work<'a> {
        Value(&'a ValueExpr),
        Int(&'a IntExpr),
        Bool(&'a BoolExpr),
        Float(&'a FloatExpr),
        BindNext(&'a [(ExpressionBindingId, ValueExpr)], usize, &'a ValueExpr),
        Register(ExpressionBindingId),
        EndScope(usize),
    }
    let mut pending = vec![Work::Value(value)];
    let mut bindings = HashSet::new();
    let mut installed = Vec::new();
    while let Some(work) = pending.pop() {
        match work {
            Work::BindNext(values, index, body) => {
                if let Some((binding, value)) = values.get(index) {
                    pending.push(Work::BindNext(values, index + 1, body));
                    pending.push(Work::Register(*binding));
                    pending.push(Work::Value(value));
                } else {
                    pending.push(Work::Value(body));
                }
            }
            Work::Register(binding) => {
                if bindings.len() == MAX_EXPRESSION_BINDINGS || !bindings.insert(binding) {
                    return false;
                }
                installed.push(binding);
            }
            Work::EndScope(length) => {
                while installed.len() > length {
                    let binding = installed.pop().expect("capture scope has an installed ID");
                    bindings.remove(&binding);
                }
            }
            Work::Value(value) => match value {
                ValueExpr::Bind {
                    bindings: values,
                    body,
                    ..
                } => {
                    pending.push(Work::EndScope(installed.len()));
                    pending.push(Work::BindNext(values, 0, body));
                }
                ValueExpr::Bound { binding, .. } => {
                    if !bindings.contains(binding) {
                        return false;
                    }
                }
                ValueExpr::NativePointer(_)
                | ValueExpr::Zero(_)
                | ValueExpr::StringBytes { .. }
                | ValueExpr::Enum { .. }
                | ValueExpr::StaticAddress { .. }
                | ValueExpr::ProcedureValue { .. } => {}
                ValueExpr::Int(value) | ValueExpr::EnumFromInt { value, .. } => {
                    pending.push(Work::Int(value))
                }
                ValueExpr::Bool(value) => pending.push(Work::Bool(value)),
                ValueExpr::Float(value) => pending.push(Work::Float(value)),
                ValueExpr::Array { elements, .. }
                | ValueExpr::Record {
                    fields: elements, ..
                } => pending.extend(elements.iter().map(Work::Value)),
                ValueExpr::OrderedRecord { .. } => return false,
                ValueExpr::RecordBuild { initializers, .. } => {
                    pending.extend(initializers.iter().map(|(_, value)| Work::Value(value)))
                }
                ValueExpr::SequenceBuild { initializers, .. } => {
                    pending.extend(initializers.iter().map(|(_, value)| Work::Value(value)))
                }
                ValueExpr::Distinct { value, .. }
                | ValueExpr::UnwrapDistinct { value, .. }
                | ValueExpr::Field { base: value, .. } => pending.push(Work::Value(value)),
                _ => return false,
            },
            Work::Int(value) => match value.kind() {
                IntExprKind::Constant(_) => {}
                IntExprKind::Value(value) | IntExprKind::EnumValue(value) => {
                    pending.push(Work::Value(value))
                }
                IntExprKind::FromBool(value) => pending.push(Work::Bool(value)),
                IntExprKind::Negate(value)
                | IntExprKind::Complement(value)
                | IntExprKind::Cast(CastMode::Unchecked | CastMode::Truncate, value) => {
                    pending.push(Work::Int(value))
                }
                IntExprKind::Binary(
                    IntOp::Add
                    | IntOp::Subtract
                    | IntOp::Multiply
                    | IntOp::BitAnd
                    | IntOp::BitOr
                    | IntOp::BitXor,
                    left,
                    right,
                ) => {
                    pending.push(Work::Int(left));
                    pending.push(Work::Int(right));
                }
                _ => return false,
            },
            Work::Bool(value) => match value {
                BoolExpr::CompileTime | BoolExpr::Constant(_) => {}
                BoolExpr::Value(value) => pending.push(Work::Value(value)),
                BoolExpr::FromInt(value) => pending.push(Work::Int(value)),
                BoolExpr::Not(value) => pending.push(Work::Bool(value)),
                BoolExpr::CompareInts(_, left, right) => {
                    pending.push(Work::Int(left));
                    pending.push(Work::Int(right));
                }
                BoolExpr::CompareBools(_, left, right) => {
                    pending.push(Work::Bool(left));
                    pending.push(Work::Bool(right));
                }
                BoolExpr::CompareFloats(_, left, right) => {
                    pending.push(Work::Float(left));
                    pending.push(Work::Float(right));
                }
                _ => return false,
            },
            Work::Float(value) => match value.kind() {
                FloatExprKind::Constant(_) => {}
                FloatExprKind::Value(value) => pending.push(Work::Value(value)),
                FloatExprKind::Negate(value) | FloatExprKind::Cast(value) => {
                    pending.push(Work::Float(value))
                }
                FloatExprKind::FromInt(value) => pending.push(Work::Int(value)),
                FloatExprKind::Binary(_, left, right) => {
                    pending.push(Work::Float(left));
                    pending.push(Work::Float(right));
                }
                _ => return false,
            },
        }
    }
    true
}

#[cfg(test)]
mod captured_constants_tests {
    use super::*;
    use crate::ProcedureId;
    use jai_types::{IntegerType, ScalarType, TypeRegistry};

    #[test]
    fn only_ordered_in_scope_constant_captures_are_static() {
        let types = TypeRegistry::new();
        let ty = types.scalar(ScalarType::Int(IntegerType::S64));
        let first = ExpressionBindingId::new(ProcedureId::new(0), 0);
        let second = ExpressionBindingId::new(ProcedureId::new(0), 1);
        let bound = |binding| ValueExpr::Bound { binding, ty };
        assert!(is_static_value(&ValueExpr::Bind {
            bindings: vec![(first, ValueExpr::Zero(ty)), (second, bound(first))],
            body: Box::new(bound(second)),
            ty,
        }));
        assert!(!is_static_value(&bound(first)));
        assert!(!is_static_value(&ValueExpr::Bind {
            bindings: vec![(first, bound(first))],
            body: Box::new(bound(first)),
            ty,
        }));
        assert!(!is_static_value(&ValueExpr::Bind {
            bindings: vec![(first, bound(second)), (second, ValueExpr::Zero(ty))],
            body: Box::new(bound(first)),
            ty,
        }));
        assert!(!is_static_value(&ValueExpr::Conditional {
            ty,
            expression: Box::new(crate::Conditional {
                condition: BoolExpr::Constant(true),
                then_value: ValueExpr::Bind {
                    bindings: vec![(first, ValueExpr::Zero(ty))],
                    body: Box::new(bound(first)),
                    ty,
                },
                else_value: bound(first),
            }),
        }));
        assert!(!is_static_value(&ValueExpr::Record {
            ty,
            fields: vec![
                bound(first),
                ValueExpr::Bind {
                    bindings: vec![(first, ValueExpr::Zero(ty))],
                    body: Box::new(bound(first)),
                    ty,
                },
            ],
        }));
    }

    #[test]
    fn nested_and_disjoint_scopes_restore_only_their_installed_captures() {
        let types = TypeRegistry::new();
        let ty = types.scalar(ScalarType::Int(IntegerType::S64));
        let first = ExpressionBindingId::new(ProcedureId::new(0), 0);
        let second = ExpressionBindingId::new(ProcedureId::new(0), 1);
        let bound = |binding| ValueExpr::Bound { binding, ty };
        let inner = |binding, value| ValueExpr::Bind {
            bindings: vec![(binding, value)],
            body: Box::new(bound(binding)),
            ty,
        };
        assert!(is_static_value(&ValueExpr::Bind {
            bindings: vec![
                (first, inner(second, ValueExpr::Zero(ty))),
                (second, bound(first)),
            ],
            body: Box::new(ValueExpr::Record {
                ty,
                fields: vec![
                    inner(
                        ExpressionBindingId::new(ProcedureId::new(0), 2),
                        bound(first)
                    ),
                    bound(second)
                ],
            }),
            ty,
        }));
        assert!(is_static_value(&ValueExpr::Record {
            ty,
            fields: vec![
                inner(first, ValueExpr::Zero(ty)),
                inner(first, ValueExpr::Zero(ty)),
            ],
        }));
        assert!(!is_static_value(&ValueExpr::Bind {
            bindings: vec![(first, ValueExpr::Zero(ty))],
            body: Box::new(inner(first, ValueExpr::Zero(ty))),
            ty,
        }));
    }

    #[test]
    fn full_width_capture_vector_has_ordered_membership() {
        let types = TypeRegistry::new();
        let ty = types.scalar(ScalarType::Int(IntegerType::S64));
        let owner = ProcedureId::new(0);
        let mut bindings = Vec::with_capacity(65_536);
        for index in 0..65_536 {
            let producer = if index == 0 {
                ValueExpr::Zero(ty)
            } else {
                ValueExpr::Bound {
                    binding: ExpressionBindingId::new(owner, index - 1),
                    ty,
                }
            };
            bindings.push((ExpressionBindingId::new(owner, index), producer));
        }
        let mut value = ValueExpr::Bind {
            bindings,
            body: Box::new(ValueExpr::Bound {
                binding: ExpressionBindingId::new(owner, 65_535),
                ty,
            }),
            ty,
        };
        assert!(is_static_value(&value));
        let ValueExpr::Bind { bindings, .. } = &mut value else {
            unreachable!()
        };
        bindings.push((ExpressionBindingId::new(owner, 65_536), ValueExpr::Zero(ty)));
        assert!(!is_static_value(&value));
    }
}
